//! Scanner errors.

use camino::Utf8PathBuf;
use thiserror::Error;

/// A scan or hash operation failed.
#[derive(Debug, Error)]
pub enum ScanError {
    /// A file or folder could not be read.
    #[error("cannot read {path}: {source}")]
    Io {
        /// The path involved.
        path: Utf8PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
    /// The library root does not exist or is not a directory.
    #[error("library root {0} is not a directory")]
    NotADirectory(Utf8PathBuf),
}
