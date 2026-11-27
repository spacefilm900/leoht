//! Filesystem watching (feature `watch`).
//!
//! Phase 0 fixes the API surface only; debouncing, rename detection and
//! behaviour tests land in Phase 1 (see STATUS.md).

use std::sync::mpsc::Sender;

use camino::Utf8Path;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

/// Starts watching `root` recursively and forwards raw events to `tx`.
/// Dropping the returned watcher stops watching.
///
/// # Errors
/// Returns the platform watcher's error if it cannot start.
pub fn watch_library(
    root: &Utf8Path,
    tx: Sender<notify::Result<Event>>,
) -> notify::Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |event| {
        // The receiver may be gone during shutdown; that is not an error.
        let _ = tx.send(event);
    })?;
    watcher.watch(root.as_std_path(), RecursiveMode::Recursive)?;
    Ok(watcher)
}
