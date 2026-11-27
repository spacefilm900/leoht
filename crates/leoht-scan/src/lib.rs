//! Reads a library folder: finds photos, hashes them, extracts metadata.
//!
//! Everything here only *reads* the filesystem. Nothing in this crate modifies,
//! moves or deletes a user's files.

mod error;
mod hash;
mod walk;

#[cfg(feature = "exif")]
mod exif_meta;
#[cfg(feature = "watch")]
mod watch;

pub use error::ScanError;
pub use hash::hash_file;
pub use walk::{ScanReport, scan_library};

#[cfg(feature = "exif")]
pub use exif_meta::read_capture_time;
#[cfg(feature = "watch")]
pub use watch::watch_library;
