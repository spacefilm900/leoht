//! Writes the synthetic library to disk.

use std::fs::{self, OpenOptions};
use std::io::Write as _;

use camino::{Utf8Path, Utf8PathBuf};
use image::{ImageEncoder, RgbImage, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};
use thiserror::Error;

use crate::rng::SplitMix64;

/// Shape of the library to generate.
#[derive(Debug, Clone)]
pub struct LibrarySpec {
    /// Total number of photo files.
    pub count: usize,
    /// Seed; same seed + same spec = identical bytes.
    pub seed: u64,
    /// Photos per leaf folder.
    pub photos_per_folder: usize,
}

impl Default for LibrarySpec {
    fn default() -> Self {
        Self {
            count: 1_000,
            seed: 0x1E_0F7,
            photos_per_folder: 100,
        }
    }
}

/// What was written.
#[derive(Debug, PartialEq, Eq)]
pub struct GeneratedLibrary {
    /// Files written.
    pub photos: usize,
    /// Leaf folders created.
    pub folders: usize,
}

/// Generation failed.
#[derive(Debug, Error)]
pub enum TestkitError {
    /// Filesystem failure.
    #[error("cannot write {path}: {source}")]
    Io {
        /// The path involved.
        path: Utf8PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
    /// Image encoding failure.
    #[error("cannot encode synthetic image: {0}")]
    Encode(#[from] image::ImageError),
    /// `photos_per_folder` was zero.
    #[error("photos_per_folder must be at least 1")]
    EmptyFolders,
}

/// Side length of every generated image: small, so 100k files stay fast and light.
const SIZE: u32 = 16;

/// Generates `spec.count` files under `root`, laid out as
/// `year-NNNN/folder-NNNNN/IMG_NNNNNN.<ext>`.
///
/// Mix: ~80% JPEG, ~10% PNG, ~5% fake HEIC and ~5% fake RAW (placeholder bytes: the
/// scanner only needs to *detect* those, and Phase 0 has no decoder for them).
///
/// # Errors
/// Filesystem or encoding failure, `photos_per_folder == 0`, or a target file that already
/// exists (existing files are never overwritten: generate into a fresh folder).
pub fn generate_library(
    root: &Utf8Path,
    spec: &LibrarySpec,
) -> Result<GeneratedLibrary, TestkitError> {
    if spec.photos_per_folder == 0 {
        return Err(TestkitError::EmptyFolders);
    }
    for index in 0..spec.count {
        let folder = index / spec.photos_per_folder;
        let dir = root
            .join(format!("year-{}", 2000 + folder % 25))
            .join(format!("folder-{folder:05}"));
        // Each photo gets its own RNG stream so files are independent of generation order.
        let mut rng = SplitMix64::new(spec.seed ^ (index as u64).wrapping_mul(0x9E37_79B9));
        let (ext, bytes) = match rng.next_u64() % 20 {
            0..=15 => ("jpg", encode_jpeg(&mut rng)?),
            16..=17 => ("png", encode_png(&mut rng)?),
            18 => ("heic", placeholder(&mut rng)),
            _ => ("cr2", placeholder(&mut rng)),
        };
        fs::create_dir_all(&dir).map_err(|source| TestkitError::Io {
            path: dir.clone(),
            source,
        })?;
        let file = dir.join(format!("IMG_{index:06}.{ext}"));
        // `create_new` refuses to overwrite: generating into a folder that already holds
        // a file with this name must never destroy it (it could be someone's real photo).
        write_new(&file, &bytes).map_err(|source| TestkitError::Io { path: file, source })?;
    }
    Ok(GeneratedLibrary {
        photos: spec.count,
        folders: spec.count.div_ceil(spec.photos_per_folder),
    })
}

/// Writes `bytes` to a file that must not exist yet (atomic check, no race).
fn write_new(path: &Utf8Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)
}

/// A smooth gradient between seeded endpoints: cheap and deterministic
/// (it compresses unlike a real photo, which is fine for volume tests).
fn random_image(rng: &mut SplitMix64) -> RgbImage {
    // Six random bytes = a start and an end value for each of R, G, B.
    let ends = rng.next_u64().to_le_bytes();
    // Linear blend between `ends[2*ch]` and `ends[2*ch + 1]` at position `pos` in 0..SIZE.
    let blend = |channel: usize, pos: u32| {
        let (from, to) = (ends[2 * channel], ends[2 * channel + 1]);
        let span = u32::from(from.abs_diff(to));
        let base = u32::from(from.min(to));
        u8::try_from(base + span * pos / (SIZE - 1)).unwrap_or(u8::MAX)
    };
    RgbImage::from_fn(SIZE, SIZE, |x, y| {
        image::Rgb([blend(0, x), blend(1, y), blend(2, u32::midpoint(x, y))])
    })
}

fn encode_jpeg(rng: &mut SplitMix64) -> Result<Vec<u8>, TestkitError> {
    let img = random_image(rng);
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 80).write_image(
        img.as_raw(),
        SIZE,
        SIZE,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(out)
}

fn encode_png(rng: &mut SplitMix64) -> Result<Vec<u8>, TestkitError> {
    let img = random_image(rng);
    let mut out = Vec::new();
    PngEncoder::new(&mut out).write_image(
        img.as_raw(),
        SIZE,
        SIZE,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(out)
}

/// 64 bytes of seeded noise standing in for an undecodable HEIC/RAW file.
fn placeholder(rng: &mut SplitMix64) -> Vec<u8> {
    (0..8).flat_map(|_| rng.next_u64().to_le_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hash of every file's relative path and contents, in sorted order.
    fn fingerprint(root: &Utf8Path) -> String {
        let mut files = Vec::new();
        let mut stack = vec![root.to_owned()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = Utf8PathBuf::from_path_buf(entry.unwrap().path()).unwrap();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    files.push(path);
                }
            }
        }
        files.sort();
        let mut hasher = blake3::Hasher::new();
        for f in files {
            hasher.update(f.strip_prefix(root).unwrap().as_str().as_bytes());
            hasher.update(&fs::read(&f).unwrap());
        }
        hasher.finalize().to_hex().to_string()
    }

    fn generate(spec: &LibrarySpec) -> (tempfile::TempDir, Utf8PathBuf, GeneratedLibrary) {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let report = generate_library(&root, spec).unwrap();
        (dir, root, report)
    }

    #[test]
    fn same_spec_gives_identical_bytes_and_a_different_seed_does_not() {
        let spec = LibrarySpec {
            count: 60,
            photos_per_folder: 25,
            ..LibrarySpec::default()
        };
        let (_d1, r1, report) = generate(&spec);
        let (_d2, r2, _) = generate(&spec);
        let (_d3, r3, _) = generate(&LibrarySpec {
            seed: spec.seed + 1,
            ..spec.clone()
        });
        assert_eq!(
            report,
            GeneratedLibrary {
                photos: 60,
                folders: 3
            }
        );
        assert_eq!(fingerprint(&r1), fingerprint(&r2));
        assert_ne!(fingerprint(&r1), fingerprint(&r3));
    }

    #[test]
    fn zero_photos_per_folder_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let spec = LibrarySpec {
            photos_per_folder: 0,
            ..LibrarySpec::default()
        };
        assert!(matches!(
            generate_library(&root, &spec),
            Err(TestkitError::EmptyFolders)
        ));
    }

    #[test]
    fn existing_files_are_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let spec = LibrarySpec {
            count: 1,
            seed: 1,
            photos_per_folder: 100,
        };
        // Pre-create the exact file the generator would write first.
        let victim = root.join("year-2000/folder-00000").join("IMG_000000.jpg");
        fs::create_dir_all(victim.parent().unwrap()).unwrap();
        fs::write(&victim, b"someone's real photo").unwrap();
        let err = generate_library(&root, &spec).unwrap_err();
        assert!(matches!(err, TestkitError::Io { .. }), "got {err:?}");
        assert_eq!(fs::read(&victim).unwrap(), b"someone's real photo");
    }
}
