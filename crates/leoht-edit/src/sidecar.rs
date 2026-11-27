//! The per-image `.leoht.json` sidecar.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

use camino::{Utf8Path, Utf8PathBuf};
use leoht_core::{ContentHash, EditRecipe};
use serde::{Deserialize, Serialize};

use crate::EditError;

/// Highest sidecar schema version this build can read and write.
pub const SCHEMA_VERSION: u32 = 1;

/// Appended to the *full* image file name: `IMG_0001.jpg` → `IMG_0001.jpg.leoht.json`.
pub const SIDECAR_SUFFIX: &str = ".leoht.json";

/// Returns where the sidecar for `image` lives (same folder, `<file name>.leoht.json`).
///
/// The full original file name is kept so `a.jpg` and `a.png` never collide.
#[must_use]
pub fn sidecar_path_for(image: &Utf8Path) -> Utf8PathBuf {
    let mut name = image.file_name().unwrap_or_default().to_owned();
    name.push_str(SIDECAR_SUFFIX);
    image.with_file_name(name)
}

/// The contents of one sidecar file.
///
/// Unknown top-level fields are preserved in [`Sidecar::extra`] so that a newer
/// or third-party writer's data survives a read-modify-write by this version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sidecar {
    /// Schema version of this file; see [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Hash of the original image the recipe was authored against (for re-association).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<ContentHash>,
    /// The edits to apply to the untouched original.
    #[serde(default)]
    pub recipe: EditRecipe,
    /// Fields this version does not know about, kept verbatim.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Sidecar {
    /// A sidecar at the current schema version with the given recipe.
    #[must_use]
    pub fn new(recipe: EditRecipe) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            source_hash: None,
            recipe,
            extra: BTreeMap::new(),
        }
    }

    /// Reads a sidecar. Returns `Ok(None)` when the file does not exist.
    ///
    /// # Errors
    /// I/O failure, malformed JSON, a newer schema than supported, or an invalid recipe.
    pub fn read(path: &Utf8Path) -> Result<Option<Self>, EditError> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(EditError::Io {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        let sidecar: Self = serde_json::from_slice(&bytes).map_err(|source| EditError::Parse {
            path: path.to_owned(),
            source,
        })?;
        if sidecar.schema_version > SCHEMA_VERSION {
            return Err(EditError::UnsupportedSchema {
                path: path.to_owned(),
                found: sidecar.schema_version,
                supported: SCHEMA_VERSION,
            });
        }
        sidecar
            .recipe
            .validate()
            .map_err(|source| EditError::InvalidRecipe {
                path: path.to_owned(),
                source,
            })?;
        Ok(Some(sidecar))
    }

    /// Writes the sidecar atomically and durably (ADR-0002).
    ///
    /// The JSON goes to a fresh, uniquely named temp file in the same folder
    /// (created with `create_new`, so it can never truncate or follow a link onto
    /// an existing file such as an original photo), is flushed to storage with
    /// `sync_all`, renamed over the sidecar, and then the folder is synced so the
    /// rename itself survives a power cut. A crash or failure therefore leaves
    /// either the old sidecar or the new one, never a half-written file.
    ///
    /// # Errors
    /// Invalid recipe, or I/O failure (including a failed sync, which is not ignored).
    pub fn write(&self, path: &Utf8Path) -> Result<(), EditError> {
        self.recipe
            .validate()
            .map_err(|source| EditError::InvalidRecipe {
                path: path.to_owned(),
                source,
            })?;
        let mut json = serde_json::to_string_pretty(self).map_err(|source| EditError::Parse {
            path: path.to_owned(),
            source,
        })?;
        json.push('\n');

        let io_err = |p: &Utf8Path| {
            let p = p.to_owned();
            move |source| EditError::Io { path: p, source }
        };
        let (tmp, mut file) = create_temp_beside(path).map_err(io_err(path))?;
        let published = file
            .write_all(json.as_bytes())
            .and_then(|()| file.sync_all()) // contents on disk *before* they replace the old sidecar
            .and_then(|()| {
                drop(file);
                fs::rename(&tmp, path)
            });
        if let Err(source) = published {
            // `tmp` is the file *we* just created exclusively, so removing it is safe.
            let _ = fs::remove_file(&tmp);
            return Err(EditError::Io {
                path: path.to_owned(),
                source,
            });
        }
        sync_parent_dir(path).map_err(io_err(path))
    }
}

/// Distinguishes temp files created by concurrent writers in one process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Creates `<path>.<pid>.<n>.tmp` next to `path` with exclusive creation.
/// A name collision (stale file from a crashed run) is skipped, never overwritten.
fn create_temp_beside(path: &Utf8Path) -> std::io::Result<(Utf8PathBuf, File)> {
    let name = path.file_name().unwrap_or("sidecar");
    let mut last_err = None;
    for _ in 0..16 {
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = path.with_file_name(format!("{name}.{}.{n}.tmp", std::process::id()));
        match OpenOptions::new().write(true).create_new(true).open(&tmp) {
            Ok(file) => return Ok((tmp, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => last_err = Some(e),
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or_else(|| std::io::Error::other("no free temp file name")))
}

/// Flushes the folder entry so a completed rename is durable. Unix only:
/// Windows cannot open a directory for syncing, and NTFS journals the rename.
#[cfg(unix)]
fn sync_parent_dir(path: &Utf8Path) -> std::io::Result<()> {
    let dir = match path.parent() {
        Some(p) if !p.as_str().is_empty() => p,
        _ => Utf8Path::new("."),
    };
    File::open(dir)?.sync_all()
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)] // keeps the signature identical to the Unix version
fn sync_parent_dir(_path: &Utf8Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use leoht_core::EditOp;

    #[test]
    fn sidecar_path_keeps_the_full_file_name() {
        let p = sidecar_path_for(Utf8Path::new("lib/2020/IMG_1.jpg"));
        // Compare components, not the string: Windows joins with a backslash, Unix with `/`.
        assert_eq!(p.file_name(), Some("IMG_1.jpg.leoht.json"));
        assert_eq!(p.parent(), Some(Utf8Path::new("lib/2020")));
    }

    fn utf8_tmp(dir: &tempfile::TempDir) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap()
    }

    #[test]
    fn missing_file_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            Sidecar::read(&utf8_tmp(&dir).join("nope.leoht.json"))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn write_then_read_round_trips_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = utf8_tmp(&dir).join("a.jpg.leoht.json");
        let sidecar = Sidecar::new(EditRecipe {
            ops: vec![EditOp::Rotate { quarter_turns: 2 }],
        });
        sidecar.write(&path).unwrap();
        assert_eq!(Sidecar::read(&path).unwrap().unwrap(), sidecar);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn newer_schema_is_refused_not_clobbered() {
        let dir = tempfile::tempdir().unwrap();
        let path = utf8_tmp(&dir).join("a.leoht.json");
        std::fs::write(&path, r#"{"schema_version": 99}"#).unwrap();
        assert!(matches!(
            Sidecar::read(&path),
            Err(EditError::UnsupportedSchema { found: 99, .. })
        ));
    }

    #[test]
    fn unknown_fields_survive_a_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = utf8_tmp(&dir).join("a.leoht.json");
        std::fs::write(
            &path,
            r#"{"schema_version":1,"rating":4,"recipe":{"ops":[]}}"#,
        )
        .unwrap();
        let sidecar = Sidecar::read(&path).unwrap().unwrap();
        sidecar.write(&path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("\"rating\": 4"), "unknown field lost: {raw}");
    }

    #[test]
    fn invalid_recipe_is_rejected_on_read_and_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = utf8_tmp(&dir).join("a.leoht.json");
        let bad = Sidecar::new(EditRecipe {
            ops: vec![EditOp::Rotate { quarter_turns: 9 }],
        });
        assert!(bad.write(&path).is_err());
        std::fs::write(
            &path,
            r#"{"schema_version":1,"recipe":{"ops":[{"op":"rotate","quarter_turns":9}]}}"#,
        )
        .unwrap();
        assert!(matches!(
            Sidecar::read(&path),
            Err(EditError::InvalidRecipe { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn write_never_touches_a_file_that_a_temp_name_could_alias() {
        // A symlink at the old fixed temp name must not redirect the write onto the photo.
        let dir = tempfile::tempdir().unwrap();
        let root = utf8_tmp(&dir);
        let photo = root.join("a.jpg");
        std::fs::write(&photo, b"ORIGINAL PHOTO BYTES").unwrap();
        std::os::unix::fs::symlink(&photo, root.join("a.jpg.leoht.json.tmp")).unwrap();
        Sidecar::new(EditRecipe::default())
            .write(&root.join("a.jpg.leoht.json"))
            .unwrap();
        assert_eq!(std::fs::read(&photo).unwrap(), b"ORIGINAL PHOTO BYTES");
    }

    #[test]
    fn concurrent_writers_each_publish_complete_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = utf8_tmp(&dir).join("a.jpg.leoht.json");
        std::thread::scope(|s| {
            for turns in 1..=3_u8 {
                let path = &path;
                s.spawn(move || {
                    let sc = Sidecar::new(EditRecipe {
                        ops: vec![EditOp::Rotate {
                            quarter_turns: turns,
                        }],
                    });
                    for _ in 0..25 {
                        sc.write(path).unwrap();
                    }
                });
            }
        });
        assert!(Sidecar::read(&path).unwrap().is_some());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
