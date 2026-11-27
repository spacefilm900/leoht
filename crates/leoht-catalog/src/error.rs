//! Catalog errors.

use leoht_core::CoreError;
use thiserror::Error;

/// A catalog operation failed.
#[derive(Debug, Error)]
pub enum CatalogError {
    /// SQLite reported an error.
    #[error("catalog database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// The database was written by a newer Leoht. Delete it and re-scan to rebuild.
    #[error(
        "catalog schema version {found} is newer than supported {supported}; rebuild the catalog"
    )]
    NewerSchema {
        /// Version stored in the database.
        found: u32,
        /// Highest version this build understands.
        supported: u32,
    },
    /// A row named a format this build does not know (catalog from a different version).
    #[error("catalog contains unknown format name {0:?}")]
    UnknownFormat(String),
    /// A row held a value that is not a valid domain value (corrupt or hand-edited database).
    #[error("catalog contains an invalid value: {0}")]
    Corrupt(#[from] CoreError),
}
