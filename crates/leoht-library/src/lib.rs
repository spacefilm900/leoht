//! A Leoht library: a folder of photos plus Leoht's disposable index of it.
//!
//! This crate ties the scanner and the catalog together:
//! * [`cache`] decides *where* the catalog and thumbnails live. By default that
//!   is a clearly named per-user cache folder, **outside** the photo folder, so
//!   scanning never writes inside the library (FR-FND-01) and read-only
//!   libraries work.
//! * [`Library::scan`] is the incremental scan: unchanged files (same size and
//!   modification time) are not re-read, changed and new files are probed,
//!   vanished files are removed (FR-LIB-09). Results are committed in batches,
//!   so another reader sees the first photos while the scan is still running.
//! * [`Library::rebuild`] deletes the catalog and scans from scratch: the
//!   normal, tested recovery path (FR-LIB-08, FR-FND-02).
//! * [`Library::make_thumbnails`] fills the content-addressed thumbnail cache
//!   (FR-LIB-07), and [`Library::verify`] compares the catalog with the disk
//!   without changing either.

pub mod cache;
mod error;
mod library;
mod scan;
mod thumbs;
mod verify;

pub use error::LibraryError;
pub use library::{Library, LibraryPaths};
pub use scan::{BATCH_SIZE, Progress, ScanSummary};
pub use thumbs::{DEFAULT_THUMB_EDGE, MAX_WORKERS, ThumbOptions, ThumbProgress, ThumbSummary};
pub use verify::VerifyReport;
