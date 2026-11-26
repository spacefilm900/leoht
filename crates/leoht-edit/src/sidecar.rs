//! The per-image `.leoht.json` sidecar.

use std::collections::BTreeMap;
use std::fs;

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

    /// Writes the sidecar atomically: temp file in the same folder, then rename.
    /// A crash mid-write therefore never leaves a half-written sidecar.
    ///
    /// # Errors
    /// Invalid recipe, or I/O failure.
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

        let tmp = path.with_extension("json.tmp");
        let io_err = |p: &Utf8Path| {
            let p = p.to_owned();
            move |source| EditError::Io { path: p, source }
        };
        fs::write(&tmp, json).map_err(io_err(&tmp))?;
        fs::rename(&tmp, path).map_err(io_err(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leoht_core::EditOp;

    #[test]
    fn sidecar_path_keeps_the_full_file_name() {
        let p = sidecar_path_for(Utf8Path::new("lib/2020/IMG_1.jpg"));
        assert_eq!(p.as_str(), "lib/2020/IMG_1.jpg.leoht.json");
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
}
