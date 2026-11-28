//! Recursive library walk: find candidate photo files and stat them.
//!
//! The walk reads directory listings and file metadata only; it never opens a
//! photo. Opening and probing happen in [`crate::probe_file`].

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use camino::{Utf8Path, Utf8PathBuf};
use ignore::{WalkBuilder, gitignore::Gitignore, gitignore::GitignoreBuilder};
use leoht_core::{PhotoFormat, RelPath};

use crate::{LibraryConfig, ScanError};

/// A candidate photo found by the walk: file-system facts only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundFile {
    /// Absolute path on this machine.
    pub abs: Utf8PathBuf,
    /// Library-relative path (the photo's identity).
    pub rel: RelPath,
    /// Format claimed by the extension (the probe checks the content).
    pub format: PhotoFormat,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Modification time: whole seconds since the epoch, rounded down.
    pub modified_unix_secs: i64,
    /// Modification time: nanoseconds past `modified_unix_secs`.
    pub modified_nanos: u32,
}

/// Why the walk passed over an entry. Recorded so nothing disappears silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The name is not valid UTF-8, so it cannot be stored as a library path.
    NonUtf8Name,
    /// Symbolic links are not followed (no loops, nothing outside the library).
    Symlink,
    /// The directory entry or its metadata could not be read.
    Unreadable(String),
}

/// An entry the walk passed over, with the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// The path for display, without loss: UTF-8 paths verbatim, others in
    /// Rust's quoted, escaped form (`"caf\xE9.jpg"`), so distinct names stay distinct.
    pub path: String,
    /// The entry's path when it is known and valid UTF-8. A rescan uses it to
    /// keep catalog rows under an unreadable folder instead of treating them
    /// as deleted.
    pub abs: Option<Utf8PathBuf>,
    /// Why it was skipped.
    pub reason: SkipReason,
}

/// Walks `root` in a stable (name-sorted) order and calls `visit` for every
/// file whose extension is a recognised and enabled [`PhotoFormat`].
///
/// Rules (FR-LIB-01, FR-LIB-03, FR-LIB-05):
/// * hidden files and folders (name starts with `.`) are skipped, as Finder does,
///   which also skips macOS `._*` resource files and Leoht's own caches;
/// * `config.ignore` patterns are skipped *before* descending, so an ignored
///   folder is never opened;
/// * formats switched off in `config.formats` are not listed;
/// * symbolic links are not followed (recorded as skipped);
/// * `.gitignore` files are deliberately not honoured: they are meaningless for photos.
///
/// # Errors
/// [`ScanError::NotADirectory`] if `root` is not a directory, or
/// [`ScanError::Config`] if an ignore pattern is invalid.
pub fn walk_library(
    root: &Utf8Path,
    config: &LibraryConfig,
    mut visit: impl FnMut(FoundFile),
) -> Result<Vec<Skipped>, ScanError> {
    if !root.is_dir() {
        return Err(ScanError::NotADirectory(root.to_owned()));
    }
    let ignore = Arc::new(build_ignore(root, config)?);
    let walker = WalkBuilder::new(root)
        .standard_filters(false) // no .gitignore; hidden entries are handled below
        .follow_links(false)
        .sort_by_file_name(Ord::cmp)
        .filter_entry(move |entry| keep_entry(entry, &ignore))
        .build();

    let mut skipped = Vec::new();
    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                let native = error_path(&e);
                skipped.push(Skipped {
                    path: native.map_or_else(|| e.to_string(), display_path),
                    abs: native.and_then(|p| Utf8PathBuf::from_path_buf(p.to_path_buf()).ok()),
                    reason: SkipReason::Unreadable(e.to_string()),
                });
                continue;
            }
        };
        let Some(file_type) = entry.file_type() else {
            continue; // stdin entry; cannot happen for a directory walk
        };
        if file_type.is_symlink() {
            skipped.push(skip(&entry, SkipReason::Symlink));
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        match found_file(root, &entry, config) {
            Ok(Some(file)) => visit(file),
            Ok(None) => {} // not a photo, or a format switched off
            Err(reason) => skipped.push(skip(&entry, reason)),
        }
    }
    Ok(skipped)
}

fn skip(entry: &ignore::DirEntry, reason: SkipReason) -> Skipped {
    Skipped {
        path: display_path(entry.path()),
        abs: Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).ok(),
        reason,
    }
}

/// A path as text without loss (see [`Skipped::path`]).
// `Debug` is deliberate for non-UTF-8 paths: it escapes the invalid bytes
// instead of replacing them, so two different names never look the same.
#[allow(clippy::unnecessary_debug_formatting)]
fn display_path(path: &std::path::Path) -> String {
    path.to_str()
        .map_or_else(|| format!("{path:?}"), str::to_owned)
}

/// The path a walk error is about, if it names one.
fn error_path(e: &ignore::Error) -> Option<&std::path::Path> {
    match e {
        ignore::Error::WithPath { path, .. } => Some(path),
        ignore::Error::WithDepth { err, .. } | ignore::Error::WithLineNumber { err, .. } => {
            error_path(err)
        }
        ignore::Error::Loop { child, .. } => Some(child),
        _ => None,
    }
}

/// Compiles the config's ignore patterns, anchored at the library root.
fn build_ignore(root: &Utf8Path, config: &LibraryConfig) -> Result<Gitignore, ScanError> {
    let mut builder = GitignoreBuilder::new(root);
    for pattern in &config.ignore {
        builder
            .add_line(None, pattern)
            .map_err(|e| ScanError::Config {
                path: root.join(crate::CONFIG_FILE),
                message: format!("bad ignore pattern {pattern:?}: {e}"),
            })?;
    }
    builder.build().map_err(|e| ScanError::Config {
        path: root.join(crate::CONFIG_FILE),
        message: e.to_string(),
    })
}

/// The walk's entry filter: drop hidden and ignored entries (never the root).
fn keep_entry(entry: &ignore::DirEntry, ignore: &Gitignore) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    let hidden = entry.file_name().as_encoded_bytes().first() == Some(&b'.');
    let is_dir = entry.file_type().is_some_and(|t| t.is_dir());
    !hidden && !ignore.matched(entry.path(), is_dir).is_ignore()
}

/// Splits a modification time into whole seconds (rounded down, so times
/// before 1970 stay negative rather than becoming 0) and nanoseconds past them.
pub(crate) fn split_time(t: SystemTime) -> (i64, u32) {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => (
            i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
            d.subsec_nanos(),
        ),
        Err(e) => {
            let before = e.duration();
            let nanos = before.subsec_nanos();
            let secs = i64::try_from(before.as_secs()).unwrap_or(i64::MAX);
            if nanos == 0 {
                (-secs, 0)
            } else {
                (-secs - 1, 1_000_000_000 - nanos)
            }
        }
    }
}

/// `Ok(None)` = not a (wanted) photo; `Err` = looked like one but cannot be listed.
fn found_file(
    root: &Utf8Path,
    entry: &ignore::DirEntry,
    config: &LibraryConfig,
) -> Result<Option<FoundFile>, SkipReason> {
    let Some(path) = Utf8Path::from_path(entry.path()) else {
        tracing::warn!(path = ?entry.path(), "skipping non-UTF-8 path");
        return Err(SkipReason::NonUtf8Name);
    };
    let Some(format) = path.extension().and_then(PhotoFormat::from_extension) else {
        return Ok(None);
    };
    if !config.format_enabled(format) {
        return Ok(None);
    }
    let meta = entry
        .metadata()
        .map_err(|e| SkipReason::Unreadable(e.to_string()))?;
    let rel = path
        .strip_prefix(root)
        .map_err(|e| SkipReason::Unreadable(e.to_string()))?;
    let rel = RelPath::from_native(rel).map_err(|e| SkipReason::Unreadable(e.to_string()))?;
    let (secs, nanos) = meta.modified().map_or((0, 0), split_time);
    Ok(Some(FoundFile {
        abs: path.to_owned(),
        rel,
        format,
        size_bytes: meta.len(),
        modified_unix_secs: secs,
        modified_nanos: nanos,
    }))
}

#[cfg(test)]
#[path = "walk_tests.rs"]
mod tests;
