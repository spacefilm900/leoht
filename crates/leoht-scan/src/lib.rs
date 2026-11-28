//! Reads a library folder: finds photos, checks and probes them, hashes them.
//!
//! Everything here only *reads* the filesystem. Nothing in this crate modifies,
//! moves or deletes a user's files (FR-FND-01).
//!
//! The pieces compose as: [`LibraryConfig::load`] → [`walk_library`] (directory
//! listings and stat only) → [`probe_file`] per file (content check, EXIF,
//! header) → a [`Photo`]. [`scan_library`] does all three.

mod config;
mod error;
mod hash;
mod probe;
mod walk;

#[cfg(feature = "watch")]
mod watch;

use camino::Utf8Path;
use leoht_core::Photo;

pub use config::{CONFIG_FILE, CONFIG_VERSION, LibraryConfig};
pub use error::ScanError;
pub use hash::hash_file;
pub use probe::{MAX_PIXELS, PROBE_VERSION, matches_format, probe_file};
pub use walk::{FoundFile, SkipReason, Skipped, walk_library};

#[cfg(feature = "watch")]
pub use watch::watch_library;

/// What a full scan found.
#[derive(Debug, Default)]
pub struct ScanReport {
    /// Every photo found, in walk (name-sorted) order, damaged ones included.
    pub photos: Vec<Photo>,
    /// Entries passed over, with reasons.
    pub skipped: Vec<Skipped>,
}

/// Loads the library config, walks `root` and probes every photo found.
///
/// This is the simple, all-in-memory path (tests, small libraries, the
/// benchmark). Incremental scanning into a catalog lives in `leoht-library`.
///
/// # Errors
/// [`ScanError::NotADirectory`] or [`ScanError::Config`].
pub fn scan_library(root: &Utf8Path) -> Result<ScanReport, ScanError> {
    let config = LibraryConfig::load(root)?;
    let mut photos = Vec::new();
    let skipped = walk_library(root, &config, |found| photos.push(probe_file(&found)))?;
    Ok(ScanReport { photos, skipped })
}
