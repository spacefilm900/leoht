//! The library config file: which folders to ignore and which formats to show
//! (FR-LIB-03, FR-LIB-05).
//!
//! It is a visible, hand-editable JSON file at the library root, so it travels
//! with the library and is backed up with it:
//!
//! ```json
//! {
//!   "version": 1,
//!   "ignore": ["Backups/", "**/Thumbs/", "*.tmp.jpg"],
//!   "formats": { "raw": false }
//! }
//! ```
//!
//! `ignore` uses `.gitignore` pattern syntax relative to the library root.
//! `formats` switches formats off by their stable name; anything not listed is on.

use std::collections::BTreeMap;

use camino::{Utf8Path, Utf8PathBuf};
use leoht_core::PhotoFormat;
use serde::{Deserialize, Serialize};

use crate::ScanError;

/// File name of the library config, at the library root.
pub const CONFIG_FILE: &str = "leoht-library.json";

/// Highest config `version` this build understands.
pub const CONFIG_VERSION: u32 = 1;

/// Parsed library config. The default (no file) scans everything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryConfig {
    /// Format version of this file.
    pub version: u32,
    /// Folder and file patterns to skip (`.gitignore` syntax, relative to the root).
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Formats switched on (`true`) or off (`false`) by stable name; unlisted = on.
    #[serde(default)]
    pub formats: BTreeMap<String, bool>,
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            ignore: Vec::new(),
            formats: BTreeMap::new(),
        }
    }
}

impl LibraryConfig {
    /// Reads `<root>/leoht-library.json`, or returns the default if there is none.
    ///
    /// A config that exists but cannot be understood is an error rather than
    /// being ignored: silently scanning folders the user excluded is worse than
    /// stopping with a clear message.
    ///
    /// # Errors
    /// [`ScanError::Config`] for unreadable, malformed, newer-version or
    /// unknown-format configs.
    pub fn load(root: &Utf8Path) -> Result<Self, ScanError> {
        let path = root.join(CONFIG_FILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(config_error(&path, e.to_string())),
        };
        let config: Self =
            serde_json::from_str(&text).map_err(|e| config_error(&path, e.to_string()))?;
        config.validate().map_err(|msg| config_error(&path, msg))?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.version > CONFIG_VERSION {
            return Err(format!(
                "version {} is newer than this Leoht understands ({CONFIG_VERSION})",
                self.version
            ));
        }
        if let Some(bad) = self
            .formats
            .keys()
            .find(|k| PhotoFormat::from_name(k).is_none())
        {
            let known: Vec<_> = PhotoFormat::ALL.iter().map(|f| f.name()).collect();
            return Err(format!(
                "unknown format {bad:?} (known: {})",
                known.join(", ")
            ));
        }
        Ok(())
    }

    /// True if photos of `format` should be listed.
    #[must_use]
    pub fn format_enabled(&self, format: PhotoFormat) -> bool {
        self.formats.get(format.name()).copied().unwrap_or(true)
    }
}

fn config_error(path: &Utf8PathBuf, message: String) -> ScanError {
    ScanError::Config {
        path: path.clone(),
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &tempfile::TempDir, text: &str) -> Utf8PathBuf {
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        std::fs::write(root.join(CONFIG_FILE), text).unwrap();
        root
    }

    #[test]
    fn missing_file_means_scan_everything() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let config = LibraryConfig::load(&root).unwrap();
        assert_eq!(config, LibraryConfig::default());
        assert!(PhotoFormat::ALL.iter().all(|f| config.format_enabled(*f)));
    }

    #[test]
    fn formats_can_be_switched_off_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = write(&dir, r#"{"version":1,"formats":{"raw":false,"png":true}}"#);
        let config = LibraryConfig::load(&root).unwrap();
        assert!(!config.format_enabled(PhotoFormat::Raw));
        assert!(config.format_enabled(PhotoFormat::Png));
        assert!(config.format_enabled(PhotoFormat::Jpeg));
    }

    #[test]
    fn bad_configs_are_errors_not_silently_ignored() {
        for text in [
            "{not json",
            r#"{"version":2}"#,
            r#"{"version":1,"formats":{"gif":false}}"#,
            r#"{"version":1,"ignor":["typo/"]}"#,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let root = write(&dir, text);
            assert!(
                matches!(LibraryConfig::load(&root), Err(ScanError::Config { .. })),
                "{text} should be rejected"
            );
        }
    }
}
