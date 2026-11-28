//! Library errors.

use camino::Utf8PathBuf;
use leoht_catalog::CatalogError;
use leoht_scan::ScanError;
use thiserror::Error;

/// A library operation failed.
#[derive(Debug, Error)]
pub enum LibraryError {
    /// Walking or reading the library failed as a whole.
    #[error(transparent)]
    Scan(#[from] ScanError),
    /// The catalog database failed.
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    /// A cache file or folder could not be created or removed.
    #[error("cannot use cache location {path}: {source}")]
    Cache {
        /// The path involved.
        path: Utf8PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
    /// No cache location could be determined (no home folder and no override).
    #[error("cannot find a cache folder: set LEOHT_CACHE_DIR or pass --cache-dir")]
    NoCacheDir,
    /// The chosen cache folder is inside the library, where Leoht must not write.
    #[error("cache folder {cache} is inside the library {library}; choose a folder outside it")]
    CacheInsideLibrary {
        /// The cache folder.
        cache: Utf8PathBuf,
        /// The library root.
        library: Utf8PathBuf,
    },
}
