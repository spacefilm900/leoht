//! The Leoht catalog: a SQLite *index* of a photo library.
//!
//! The catalog is a cache. Everything in it can be rebuilt by re-scanning the
//! library folder, and user-authored data (edits, later ratings and tags) lives
//! in sidecars, not here. Deleting the database file loses nothing but time.
//! See `docs/adr/0001-data-model.md` and `docs/adr/0002-sidecar-format.md`.
//!
//! The API is deliberately synchronous: SQLite is embedded and fast, so no async
//! runtime is needed.

mod error;
mod migrations;
mod row;
mod store;

pub use error::CatalogError;
pub use migrations::SCHEMA_VERSION;
pub use store::{Catalog, FileState};
