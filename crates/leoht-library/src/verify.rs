//! Read-only consistency check between the catalog and the disk.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use leoht_core::{Damage, Photo, RelPath};
use leoht_scan::{LibraryConfig, hash_file, walk_library};

use crate::{Library, LibraryError};

/// What [`Library::verify`] found. Lists are sorted by path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VerifyReport {
    /// Photo files found on disk.
    pub on_disk: usize,
    /// Photos in the catalog.
    pub in_catalog: usize,
    /// In the catalog, gone from disk.
    pub missing: Vec<RelPath>,
    /// On disk, not in the catalog.
    pub unindexed: Vec<RelPath>,
    /// Size or modification time differs from the catalog.
    pub changed: Vec<RelPath>,
    /// Same size and time, but the content hash no longer matches (only with hashing on).
    pub hash_mismatch: Vec<RelPath>,
    /// Files whose hash was re-checked.
    pub hashes_checked: usize,
    /// Photos the catalog records as damaged (reported, not drift).
    pub damaged: Vec<(RelPath, Damage)>,
    /// Wall-clock time.
    pub elapsed: Duration,
}

impl VerifyReport {
    /// True when the catalog matches the disk: nothing missing, unindexed,
    /// changed or mismatched. Damaged photos do not count against this.
    #[must_use]
    pub fn in_sync(&self) -> bool {
        self.missing.is_empty()
            && self.unindexed.is_empty()
            && self.changed.is_empty()
            && self.hash_mismatch.is_empty()
    }
}

impl Library {
    /// Compares the catalog with the disk without writing to either. With
    /// `check_hashes`, every unchanged file that has a recorded hash is read
    /// and re-hashed, which catches content changed behind an unchanged mtime.
    ///
    /// # Errors
    /// The config is unusable, the root vanished, or the catalog failed.
    pub fn verify(&self, check_hashes: bool) -> Result<VerifyReport, LibraryError> {
        let start = Instant::now();
        let config = LibraryConfig::load(&self.paths.root)?;
        let mut known: HashMap<RelPath, Photo> = self
            .catalog
            .photos()?
            .into_iter()
            .map(|p| (p.path.clone(), p))
            .collect();
        let mut report = VerifyReport {
            in_catalog: known.len(),
            ..VerifyReport::default()
        };
        let mut io_failure = None;
        walk_library(&self.paths.root, &config, |found| {
            report.on_disk += 1;
            let Some(photo) = known.remove(&found.rel) else {
                report.unindexed.push(found.rel);
                return;
            };
            if !photo.same_file_state(
                found.size_bytes,
                found.modified_unix_secs,
                found.modified_nanos,
            ) {
                report.changed.push(found.rel);
                return;
            }
            if let Some(damage) = photo.damage {
                report.damaged.push((found.rel.clone(), damage));
            }
            if let (true, Some(stored)) = (check_hashes, photo.hash) {
                report.hashes_checked += 1;
                match hash_file(&found.abs) {
                    Ok(now) if now == stored => {}
                    Ok(_) => report.hash_mismatch.push(found.rel),
                    Err(e) => io_failure = Some(e),
                }
            }
        })?;
        if let Some(e) = io_failure {
            return Err(e.into());
        }
        report.missing = known.into_keys().collect();
        report.missing.sort();
        for list in [
            &mut report.unindexed,
            &mut report.changed,
            &mut report.hash_mismatch,
        ] {
            list.sort();
        }
        report.damaged.sort_by(|a, b| a.0.cmp(&b.0));
        report.elapsed = start.elapsed();
        Ok(report)
    }
}

#[cfg(test)]
#[path = "verify_tests.rs"]
mod tests;
