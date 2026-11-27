//! Errors from reading and writing sidecars.

use camino::Utf8PathBuf;
use leoht_core::CoreError;
use thiserror::Error;

/// A sidecar could not be read, written or understood.
#[derive(Debug, Error)]
pub enum EditError {
    /// Filesystem failure.
    #[error("sidecar I/O error at {path}: {source}")]
    Io {
        /// The sidecar (or temp) path involved.
        path: Utf8PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
    /// The file is not valid sidecar JSON.
    #[error("sidecar {path} is not valid JSON: {source}")]
    Parse {
        /// The sidecar path.
        path: Utf8PathBuf,
        /// Underlying error.
        source: serde_json::Error,
    },
    /// The sidecar was written by a newer Leoht than this one.
    #[error("sidecar {path} has schema_version {found}, newer than supported {supported}")]
    UnsupportedSchema {
        /// The sidecar path.
        path: Utf8PathBuf,
        /// Version found in the file.
        found: u32,
        /// Highest version this build understands.
        supported: u32,
    },
    /// The recipe inside the sidecar violates a domain invariant.
    #[error("sidecar {path} contains an invalid recipe: {source}")]
    InvalidRecipe {
        /// The sidecar path.
        path: Utf8PathBuf,
        /// What was wrong.
        source: CoreError,
    },
}
