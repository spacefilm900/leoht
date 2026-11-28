//! Making thumbnails for a scanned library (FR-LIB-07).
//!
//! Thumbnails are keyed by content hash under the library's cache folder, so
//! an unchanged photo is never re-rendered, a changed one gets a new
//! thumbnail (its hash changes), and renames cost nothing. Work is spread over
//! a few threads; only the calling thread touches the catalog.

use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use camino::{Utf8Path, Utf8PathBuf};
use leoht_core::{ContentHash, Damage, DamageKind, Photo, PhotoFormat};
use leoht_scan::{embedded_preview, hash_file};
use leoht_thumb::{Request, ThumbError, ThumbOrigin, cache_path, render, write_atomically};

use crate::{BATCH_SIZE, Library, LibraryError};

/// Default longest edge of a thumbnail, in pixels.
pub const DEFAULT_THUMB_EDGE: u32 = 256;
/// Most worker threads used, whatever the machine has: decoding is
/// memory-hungry, and more threads stop helping once the disk is the limit.
pub const MAX_WORKERS: usize = 8;

/// How to make thumbnails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThumbOptions {
    /// Longest edge, in pixels.
    pub max_edge: u32,
    /// Worker threads; `None` picks one per core, up to [`MAX_WORKERS`].
    pub workers: Option<usize>,
}

impl Default for ThumbOptions {
    fn default() -> Self {
        Self {
            max_edge: DEFAULT_THUMB_EDGE,
            workers: None,
        }
    }
}

/// Reported as thumbnails finish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThumbProgress {
    /// Photos handled so far.
    pub done: usize,
    /// Photos to handle in this run.
    pub total: usize,
}

/// What a thumbnail run did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThumbSummary {
    /// Photos that can have a thumbnail (JPEG, PNG and RAW, not damaged).
    pub eligible: usize,
    /// Thumbnails rendered in this run.
    pub generated: usize,
    /// Of those, how many came from an embedded preview.
    pub from_preview: usize,
    /// Thumbnails already in the cache (nothing rendered).
    pub cached: usize,
    /// Photos in formats with no thumbnail yet (HEIC, TIFF, WebP and others).
    pub unsupported: usize,
    /// RAW files with no usable embedded preview.
    pub no_preview: usize,
    /// Photos already marked damaged, not retried until they change.
    pub skipped_damaged: usize,
    /// Photos newly marked damaged because they would not decode or read.
    pub newly_damaged: usize,
    /// Thumbnails that could not be written (a cache problem, not the photo's).
    pub write_errors: Vec<String>,
    /// Wall-clock time.
    pub elapsed: Duration,
}

/// One worker's verdict on one photo.
enum Outcome {
    Generated(ThumbOrigin),
    Cached,
    NoPreview,
    Damaged(Damage),
    WriteError(String),
}

struct Done {
    index: usize,
    new_hash: Option<ContentHash>,
    outcome: Outcome,
}

impl Library {
    /// Where the thumbnail of content `hash` at `max_edge` lives (it may not exist yet).
    #[must_use]
    pub fn thumbnail_path(&self, hash: &ContentHash, max_edge: u32) -> Utf8PathBuf {
        cache_path(&self.paths.thumbs, hash, max_edge)
    }

    /// Makes any missing thumbnails for the photos in the catalog, recording
    /// content hashes and decode failures there. Run [`Library::scan`] first.
    ///
    /// # Errors
    /// Only catalog failures; problems with single photos are counted in the summary.
    pub fn make_thumbnails(
        &mut self,
        options: ThumbOptions,
        mut progress: impl FnMut(ThumbProgress),
    ) -> Result<ThumbSummary, LibraryError> {
        let start = Instant::now();
        let mut summary = ThumbSummary::default();
        let mut jobs = Vec::new();
        for photo in self.catalog.photos()? {
            if photo.damage.is_some() {
                summary.skipped_damaged += 1;
            } else if matches!(photo.format, PhotoFormat::Jpeg | PhotoFormat::Png)
                || photo.format == PhotoFormat::Raw
            {
                jobs.push(photo);
            } else {
                summary.unsupported += 1;
            }
        }
        summary.eligible = jobs.len();
        let workers = options
            .workers
            .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, NonZero::get))
            .clamp(1, MAX_WORKERS)
            .min(jobs.len().max(1));

        let next = AtomicUsize::new(0);
        let (tx, rx) = mpsc::channel::<Done>();
        let (root, thumbs) = (&self.paths.root, &self.paths.thumbs);
        let catalog = &mut self.catalog;
        let jobs_ref = &jobs;
        let mut updates: Vec<Photo> = Vec::new();
        let mut failure = None;
        std::thread::scope(|scope| {
            for _ in 0..workers {
                let tx = tx.clone();
                let next = &next;
                scope.spawn(move || {
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(photo) = jobs_ref.get(index) else {
                            break;
                        };
                        let (new_hash, outcome) = make_one(root, thumbs, photo, options.max_edge);
                        if tx
                            .send(Done {
                                index,
                                new_hash,
                                outcome,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                });
            }
            drop(tx);
            for (handled, done) in rx.into_iter().enumerate() {
                let photo = &jobs_ref[done.index];
                updates.extend(record(&mut summary, photo, done));
                if updates.len() >= BATCH_SIZE && failure.is_none() {
                    failure = catalog.upsert_photos(&updates).err();
                    updates.clear();
                }
                progress(ThumbProgress {
                    done: handled + 1,
                    total: jobs_ref.len(),
                });
            }
        });
        if let Some(e) = failure {
            return Err(e.into());
        }
        self.catalog.upsert_photos(&updates)?;
        summary.elapsed = start.elapsed();
        tracing::info!(
            generated = summary.generated,
            cached = summary.cached,
            elapsed_ms = summary.elapsed.as_millis(),
            "thumbnails finished"
        );
        Ok(summary)
    }
}

/// Counts one worker result in `summary`; returns the photo's updated catalog
/// row if its hash or damage changed.
fn record(summary: &mut ThumbSummary, photo: &Photo, done: Done) -> Option<Photo> {
    let mut updated = None;
    if let Some(hash) = done.new_hash {
        updated.get_or_insert_with(|| photo.clone()).hash = Some(hash);
    }
    match done.outcome {
        Outcome::Generated(origin) => {
            summary.generated += 1;
            summary.from_preview += usize::from(origin == ThumbOrigin::EmbeddedPreview);
        }
        Outcome::Cached => summary.cached += 1,
        Outcome::NoPreview => summary.no_preview += 1,
        Outcome::Damaged(damage) => {
            summary.newly_damaged += 1;
            updated.get_or_insert_with(|| photo.clone()).damage = Some(damage);
        }
        Outcome::WriteError(e) => summary.write_errors.push(e),
    }
    updated
}

/// Hashes (if needed) and renders one photo. Returns the hash if it was computed here.
fn make_one(
    root: &Utf8Path,
    thumbs: &Utf8Path,
    photo: &Photo,
    max_edge: u32,
) -> (Option<ContentHash>, Outcome) {
    let abs = root.join(photo.path.as_str());
    let (hash, new_hash) = match &photo.hash {
        Some(h) => (h.clone(), None),
        None => match hash_file(&abs) {
            Ok(h) => (h.clone(), Some(h)),
            Err(e) => {
                let damage = Damage::new(DamageKind::Unreadable, e.to_string());
                return (None, Outcome::Damaged(damage));
            }
        },
    };
    let dest = cache_path(thumbs, &hash, max_edge);
    if dest.is_file() {
        return (new_hash, Outcome::Cached);
    }
    let is_raw = photo.format == PhotoFormat::Raw;
    let preview = if is_raw || photo.format == PhotoFormat::Jpeg {
        embedded_preview(&abs)
    } else {
        None
    };
    let request = Request {
        source: &abs,
        preview: preview.as_deref(),
        orientation: photo.meta.orientation,
        max_edge,
        may_decode: !is_raw,
    };
    let outcome = match render(&request) {
        Ok((img, origin)) => match write_atomically(&img, &dest) {
            Ok(()) => Outcome::Generated(origin),
            Err(e) => Outcome::WriteError(e.to_string()),
        },
        Err(ThumbError::NoSource { .. }) => Outcome::NoPreview,
        Err(e) => Outcome::Damaged(Damage::new(DamageKind::DecodeFailed, e.to_string())),
    };
    (new_hash, outcome)
}

#[cfg(test)]
#[path = "thumbs_tests.rs"]
mod tests;
