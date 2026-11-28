//! Fixture libraries: small folders of known files for integration tests.
//!
//! * [`write_metadata_corpus`]: one file per format and metadata case, plus a
//!   set of damaged and hostile files (FR-LIB-04, FR-LIB-06, FR-LIB-10).
//! * [`write_unicode_corpus`]: awkward names and very long paths (FR-FND-10).
//!
//! Modification times are fixed so outputs (including the file-date fallback)
//! are reproducible.

use std::fs::{self, File};
use std::time::{Duration, SystemTime};

use camino::Utf8Path;

use crate::TestkitError;
use crate::fixtures::{
    ExifSpec, exif_tiff, heic_stub, heic_with_exif, icc_profile, jpeg, png, quadrants, raf,
};

/// Modification time given to every corpus file: 2021-03-04T05:06:07Z.
pub const CORPUS_MTIME_SECS: u64 = 1_614_834_367;

/// The EXIF block of a "fully tagged" camera photo.
#[must_use]
pub fn full_exif() -> ExifSpec {
    ExifSpec {
        date_original: Some("2023:08:19 17:45:12".into()),
        offset_original: Some("+01:00".into()),
        date_digitized: Some("2023:08:19 17:45:12".into()),
        make: Some("Leoht Test Camera Co.".into()),
        model: Some("Model T1".into()),
        lens: Some("Fixture 35mm F1.8".into()),
        exposure_time: Some((1, 250)),
        f_number: Some((18, 10)),
        iso: Some(200),
        focal_length: Some((35, 1)),
        orientation: Some(6),
        color_space: Some(1),
        pixel_size: Some((48, 32)),
        thumbnail_jpeg: None,
    }
}

fn write(root: &Utf8Path, rel: &str, bytes: &[u8]) -> Result<(), TestkitError> {
    let path = root.join(rel);
    let io = |source| TestkitError::Io {
        path: path.clone(),
        source,
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io)?;
    }
    fs::write(&path, bytes).map_err(io)?;
    let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(CORPUS_MTIME_SECS);
    File::options()
        .write(true)
        .open(&path)
        .and_then(|f| f.set_modified(mtime))
        .map_err(io)
}

/// Writes the metadata and damage corpus under `root` and returns the relative
/// paths of every photo file written, sorted.
///
/// # Errors
/// Filesystem or encoding failure.
pub fn write_metadata_corpus(root: &Utf8Path) -> Result<Vec<String>, TestkitError> {
    let img = quadrants(48, 32);
    let full = full_exif();
    let digitized_only = ExifSpec {
        date_digitized: Some("2019:01:02 03:04:05".into()),
        make: Some("  Padded Maker  ".into()),
        orientation: Some(9), // out of range: must be ignored, not trusted
        ..ExifSpec::default()
    };
    let blank_date = ExifSpec {
        date_original: Some("0000:00:00 00:00:00".into()),
        model: Some("No Clock".into()),
        ..ExifSpec::default()
    };
    let p3 = Some(icc_profile("Display P3"));
    let thumb = jpeg(&quadrants(16, 8), None, None)?;
    let with_preview = ExifSpec {
        thumbnail_jpeg: Some(thumb),
        ..full.clone()
    };
    let mut huge = jpeg(&quadrants(16, 16), None, None)?;
    patch_sof_size(&mut huge, 65_000, 65_000);

    let files: Vec<(&str, Vec<u8>)> = vec![
        ("jpeg/full-exif.jpg", jpeg(&img, Some(&full), p3.clone())?),
        (
            "jpeg/digitized-only.JPG",
            jpeg(&img, Some(&digitized_only), None)?,
        ),
        ("jpeg/blank-date.jpeg", jpeg(&img, Some(&blank_date), None)?),
        ("jpeg/no-exif.jpg", jpeg(&img, None, None)?),
        (
            "jpeg/with-preview.jpg",
            jpeg(&img, Some(&with_preview), None)?,
        ),
        ("png/plain.png", png(&img, None, None)?),
        ("png/icc-p3.png", png(&img, None, p3)?),
        ("png/exif.png", png(&img, Some(&full), None)?),
        ("heic/stub.heic", heic_stub()),
        ("heic/exif-only.HEIC", heic_with_exif(&full)?),
        ("raw/tiff-based.cr2", exif_tiff(&full)?),
        ("raw/fuji.raf", raf(&jpeg(&img, Some(&full), None)?)),
        ("other/scan.tif", exif_tiff(&full)?),
        (
            "other/web.webp",
            b"RIFF\x1a\0\0\0WEBPVP8L\x0d\0\0\0/\0\0\0\x10\x07\x10\x11\x11\x88\x88\xfe\x07\0"
                .to_vec(),
        ),
        ("damaged/empty.jpg", Vec::new()),
        ("damaged/text-file.jpg", b"this is not a photo\n".to_vec()),
        (
            "damaged/truncated.jpg",
            jpeg(&img, None, None)?[..20].to_vec(),
        ),
        (
            "damaged/garbled.png",
            b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR\0\0".to_vec(),
        ),
        ("damaged/huge-dimensions.jpg", huge),
        ("damaged/noise.nef", (0u8..=255).cycle().take(512).collect()),
    ];
    let mut written = Vec::with_capacity(files.len());
    for (rel, bytes) in files {
        write(root, rel, &bytes)?;
        written.push(rel.to_owned());
    }
    // Not photos: must never be listed.
    write(root, "notes.txt", b"not a photo")?;
    write(root, "jpeg/full-exif.jpg.leoht.json", b"{}")?;
    written.sort();
    Ok(written)
}

/// Rewrites the frame size in a baseline JPEG's SOF0 segment.
fn patch_sof_size(jpeg: &mut [u8], w: u16, h: u16) {
    if let Some(at) = jpeg.windows(2).position(|m| m == [0xFF, 0xC0]) {
        jpeg[at + 5..at + 7].copy_from_slice(&h.to_be_bytes());
        jpeg[at + 7..at + 9].copy_from_slice(&w.to_be_bytes());
    }
}

/// Names that break naive path handling.
pub const UNICODE_NAMES: &[&str] = &[
    "📷 Holiday 🏖️/sunset 🌅.jpg",
    "Cafe\u{301} (combining accent)/IMG 0001.jpg",
    "Café (precomposed)/IMG 0002.jpg",
    "صور العائلة/صورة.jpg",
    "תמונות/תמונה.png",
    "日本語のフォルダ/写真.jpeg",
    "spaces   and\ttabs/file with  spaces .jpg",
    "punctuation !#$%&'()+,;=@[]^`{}~/a.jpg",
];

/// Writes small JPEG/PNG files at awkward paths: emoji, combining characters,
/// right-to-left scripts, whitespace, punctuation, and one file whose path
/// relative to `root` is over 400 characters. Returns the relative paths, sorted.
///
/// # Errors
/// Filesystem or encoding failure.
pub fn write_unicode_corpus(root: &Utf8Path) -> Result<Vec<String>, TestkitError> {
    let jpg = jpeg(&quadrants(8, 8), None, None)?;
    let pngb = png(&quadrants(8, 8), None, None)?;
    // Windows forbids control characters (the tab) in names; everything else is portable.
    let mut rels: Vec<String> = UNICODE_NAMES
        .iter()
        .filter(|s| !(cfg!(windows) && s.chars().any(char::is_control)))
        .map(|s| (*s).to_owned())
        .collect();
    // Five 80-character folder names: a 400+ character relative path, while
    // each component stays under the common 255-byte name limit.
    let segment = "long-folder-name-".repeat(5);
    let deep: Vec<String> = (0..5).map(|i| format!("{i}-{}", &segment[..78])).collect();
    rels.push(format!("{}/deep photo.jpg", deep.join("/")));
    for rel in &rels {
        let is_png = camino::Utf8Path::new(rel).extension() == Some("png");
        let bytes = if is_png { &pngb } else { &jpg };
        write(root, rel, bytes)?;
    }
    rels.sort();
    Ok(rels)
}
