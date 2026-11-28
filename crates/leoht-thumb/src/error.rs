//! Thumbnail errors.

use camino::Utf8PathBuf;
use thiserror::Error;

/// A thumbnail could not be produced.
#[derive(Debug, Error)]
pub enum ThumbError {
    /// The source image could not be opened or decoded.
    #[error("cannot decode {path}: {source}")]
    Decode {
        /// The source path.
        path: Utf8PathBuf,
        /// Underlying error.
        source: image::ImageError,
    },
    /// The thumbnail could not be encoded or written.
    #[error("cannot write thumbnail {path}: {source}")]
    Write {
        /// The destination path.
        path: Utf8PathBuf,
        /// Underlying error.
        source: image::ImageError,
    },
    /// The destination is the source photo itself (same path, link or hard link);
    /// writing would overwrite the original, which Leoht never does.
    #[error("refusing to write thumbnail over the source photo {path}")]
    DestIsSource {
        /// The destination path.
        path: Utf8PathBuf,
    },
    /// The format cannot be decoded and the file has no usable embedded preview.
    #[error("no thumbnail source for {path}: not decodable and no embedded preview")]
    NoSource {
        /// The source path.
        path: Utf8PathBuf,
    },
    /// The cache directory could not be created.
    #[error("cannot create cache directory {path}: {source}")]
    CacheDir {
        /// The directory path.
        path: Utf8PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
}
