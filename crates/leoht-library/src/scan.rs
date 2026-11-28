//! Incremental scanning into the catalog.

use std::time::{Duration, Instant};

use camino::Utf8Path;
use leoht_core::{Photo, RelPath};
use leoht_scan::{LibraryConfig, PROBE_VERSION, SkipReason, Skipped, probe_file, walk_library};

use crate::{Library, LibraryError};

/// Photos written per catalog transaction. Each commit makes that batch
/// visible to other readers, so a UI can browse while a big scan runs.
pub const BATCH_SIZE: usize = 500;

/// Catalog fact recording which probe rules built the rows.
const PROBE_VERSION_KEY: &str = "probe_version";

/// Reported after every committed batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Candidate photo files found so far.
    pub found: usize,
    /// New or changed photos committed to the catalog so far.
    pub committed: usize,
}

/// What a scan did. `probed` is the count of files actually opened and read:
/// on an unchanged library it is zero.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanSummary {
    /// Photo files found on disk.
    pub found: usize,
    /// Files new to the catalog.
    pub added: usize,
    /// Files whose size or modification time changed (re-probed).
    pub updated: usize,
    /// Catalog rows whose file has gone.
    pub removed: usize,
    /// Files skipped because nothing changed.
    pub unchanged: usize,
    /// Files opened and probed (`added + updated`).
    pub probed: usize,
    /// Damaged photos in the catalog after the scan.
    pub damaged: u64,
    /// Entries the walk passed over, with reasons.
    pub skipped: Vec<Skipped>,
    /// Wall-clock time of the scan.
    pub elapsed: Duration,
}

impl Library {
    /// Brings the catalog up to date with the disk, reading only what changed.
    ///
    /// If the catalog was built by different probe rules (an older Leoht),
    /// every file is re-probed. `progress` is called after each committed batch.
    ///
    /// # Errors
    /// The library config is unusable, the root vanished, or the catalog failed.
    /// Problems with single files never fail the scan; they become damage records.
    pub fn scan(
        &mut self,
        mut progress: impl FnMut(Progress),
    ) -> Result<ScanSummary, LibraryError> {
        let start = Instant::now();
        let config = LibraryConfig::load(&self.paths.root)?;
        let reprobe_all = self.catalog.info(PROBE_VERSION_KEY)?.as_deref()
            != Some(PROBE_VERSION.to_string().as_str());
        let mut known = self.catalog.file_states()?;
        let mut summary = ScanSummary::default();
        let mut batch: Vec<Photo> = Vec::with_capacity(BATCH_SIZE);
        let mut failure: Option<LibraryError> = None;
        let catalog = &mut self.catalog;

        let skipped = walk_library(&self.paths.root, &config, |found| {
            if failure.is_some() {
                return;
            }
            summary.found += 1;
            let previous = known.remove(&found.rel);
            let unchanged = previous.is_some_and(|s| {
                s.size_bytes == found.size_bytes
                    && s.modified_unix_secs == found.modified_unix_secs
                    && s.modified_nanos == found.modified_nanos
            });
            if unchanged && !reprobe_all {
                summary.unchanged += 1;
                return;
            }
            if previous.is_some() {
                summary.updated += 1;
            } else {
                summary.added += 1;
            }
            batch.push(probe_file(&found));
            if batch.len() >= BATCH_SIZE {
                if let Err(e) = catalog.upsert_photos(&batch) {
                    failure = Some(e.into());
                    return;
                }
                summary.probed += batch.len();
                batch.clear();
                progress(Progress {
                    found: summary.found,
                    committed: summary.probed,
                });
            }
        })?;
        if let Some(e) = failure {
            return Err(e);
        }
        self.catalog.upsert_photos(&batch)?;
        summary.probed += batch.len();
        progress(Progress {
            found: summary.found,
            committed: summary.probed,
        });

        // Rows under an entry the walk could not read are not "gone": the
        // folder may be briefly unreadable (permissions, a sleeping disk).
        // Keep them; a later complete scan reconciles them.
        let protected = unreadable_prefixes(&self.paths.root, &skipped);
        let gone: Vec<_> = known
            .keys()
            .filter(|rel| !is_protected(rel, protected.as_deref()))
            .cloned()
            .collect();
        let retained = known.len() - gone.len();
        self.catalog.remove_photos(&gone)?;
        // Retained rows were not re-probed. If the probe rules changed, keep the
        // old version recorded so the next complete scan re-probes everything.
        if retained == 0 {
            self.catalog
                .set_info(PROBE_VERSION_KEY, &PROBE_VERSION.to_string())?;
        }
        summary.removed = gone.len();
        summary.damaged = self.catalog.damaged_count()?;
        summary.skipped = skipped;
        summary.elapsed = start.elapsed();
        tracing::info!(
            found = summary.found,
            probed = summary.probed,
            removed = summary.removed,
            elapsed_ms = summary.elapsed.as_millis(),
            "scan finished"
        );
        Ok(summary)
    }

    /// Deletes the catalog and rebuilds it with a full scan (FR-LIB-08). This is
    /// the normal recovery path; user-authored data lives in sidecars and is
    /// unaffected (FR-FND-02).
    ///
    /// # Errors
    /// As [`Library::scan`], plus failure to delete the old database.
    pub fn rebuild(&mut self, progress: impl FnMut(Progress)) -> Result<ScanSummary, LibraryError> {
        self.reset_catalog()?;
        self.scan(progress)
    }
}

/// Library-relative prefixes of entries the walk could not read. `None` means
/// an unreadable entry with no usable path: nothing may be treated as removed.
fn unreadable_prefixes(root: &Utf8Path, skipped: &[Skipped]) -> Option<Vec<String>> {
    let mut prefixes = Vec::new();
    for s in skipped {
        if !matches!(s.reason, SkipReason::Unreadable(_)) {
            continue;
        }
        let rel = s
            .abs
            .as_deref()
            .and_then(|abs| abs.strip_prefix(root).ok())?;
        prefixes.push(rel.as_str().replace('\\', "/"));
    }
    Some(prefixes)
}

fn is_protected(rel: &RelPath, prefixes: Option<&[String]>) -> bool {
    let Some(prefixes) = prefixes else {
        return true;
    };
    let rel = rel.as_str();
    prefixes.iter().any(|p| {
        p.is_empty()
            || rel == p
            || rel
                .strip_prefix(p.as_str())
                .is_some_and(|r| r.starts_with('/'))
    })
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod tests;
