//! Recursive library walk.

use std::time::{SystemTime, UNIX_EPOCH};

use camino::Utf8Path;
use ignore::WalkBuilder;
use leoht_core::{Photo, PhotoFormat, RelPath};

use crate::ScanError;

/// Name of the optional per-library ignore file (gitignore syntax).
const IGNORE_FILE: &str = ".leohtignore";

/// What a scan found.
#[derive(Debug, Default)]
pub struct ScanReport {
    /// Recognised image files, in directory-walk order (not sorted).
    pub photos: Vec<Photo>,
    /// Entries skipped because they could not be read or had non-UTF-8 names.
    pub skipped: usize,
}

/// Walks `root` and returns every file whose extension is a known [`PhotoFormat`].
///
/// * Dot-folders (e.g. `.leoht`, `.git`) are skipped; `.leohtignore` files are honoured.
/// * `.gitignore` is deliberately *not* honoured — it is meaningless for photos.
/// * Hashes are not computed here (see [`crate::hash_file`]); `Photo::hash` is `None`.
/// * Unreadable or non-UTF-8 entries are counted in `skipped`, not fatal.
///
/// # Errors
/// [`ScanError::NotADirectory`] if `root` is not a directory.
pub fn scan_library(root: &Utf8Path) -> Result<ScanReport, ScanError> {
    if !root.is_dir() {
        return Err(ScanError::NotADirectory(root.to_owned()));
    }
    let walker = WalkBuilder::new(root)
        .standard_filters(false) // no .gitignore, no hidden-file filtering by the crate itself
        .add_custom_ignore_filename(IGNORE_FILE)
        .filter_entry(|entry| !is_dot_dir(entry))
        .build();

    let mut report = ScanReport::default();
    for entry in walker {
        let Ok(entry) = entry else {
            report.skipped += 1;
            continue;
        };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        match photo_from_entry(root, &entry) {
            Ok(Some(photo)) => report.photos.push(photo),
            Ok(None) => {} // not an image: ignore silently
            Err(()) => report.skipped += 1,
        }
    }
    Ok(report)
}

/// True for directories whose name starts with `.` (but never the root itself).
fn is_dot_dir(entry: &ignore::DirEntry) -> bool {
    entry.depth() > 0
        && entry.file_type().is_some_and(|t| t.is_dir())
        && entry
            .file_name()
            .to_str()
            .is_some_and(|n| n.starts_with('.'))
}

/// Whole seconds since the Unix epoch, negative before it (rounded down), so a
/// pre-1970 modification time is kept rather than silently becoming `0`.
fn unix_secs(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
        Err(e) => {
            let before = e.duration();
            // Round away from zero: 0.5 s before the epoch is second -1.
            let secs = before.as_secs() + u64::from(before.subsec_nanos() > 0);
            i64::try_from(secs).map_or(i64::MIN, |s| -s)
        }
    }
}

/// `Ok(None)` = not a photo; `Err(())` = looked like one but could not be read.
fn photo_from_entry(root: &Utf8Path, entry: &ignore::DirEntry) -> Result<Option<Photo>, ()> {
    let path = Utf8Path::from_path(entry.path()).ok_or_else(|| {
        tracing::warn!(path = ?entry.path(), "skipping non-UTF-8 path");
    })?;
    let Some(format) = path.extension().and_then(PhotoFormat::from_extension) else {
        return Ok(None);
    };
    let meta = entry
        .metadata()
        .map_err(|e| tracing::warn!(%path, error = %e, "cannot stat"))?;
    let rel = path.strip_prefix(root).map_err(|_| ())?;
    let rel =
        RelPath::from_native(rel).map_err(|e| tracing::warn!(%path, error = %e, "bad path"))?;
    let modified_unix_secs = meta.modified().ok().map_or(0, unix_secs);
    Ok(Some(Photo {
        path: rel,
        format,
        size_bytes: meta.len(),
        modified_unix_secs,
        hash: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use std::time::Duration;

    #[test]
    fn unix_secs_handles_both_sides_of_the_epoch() {
        assert_eq!(unix_secs(UNIX_EPOCH), 0);
        assert_eq!(unix_secs(UNIX_EPOCH + Duration::from_secs(5)), 5);
        assert_eq!(unix_secs(UNIX_EPOCH - Duration::from_secs(1)), -1);
        assert_eq!(unix_secs(UNIX_EPOCH - Duration::from_millis(500)), -1);
    }

    fn touch(root: &Utf8Path, rel: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"x").unwrap();
    }

    fn root(dir: &tempfile::TempDir) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap()
    }

    #[test]
    fn finds_images_recursively_and_ignores_other_files() {
        let dir = tempfile::tempdir().unwrap();
        let r = root(&dir);
        for f in [
            "a.jpg",
            "2020/b.PNG",
            "2020/trip/c.heic",
            "2020/notes.txt",
            "d.jpg.leoht.json",
        ] {
            touch(&r, f);
        }
        let mut found: Vec<_> = scan_library(&r)
            .unwrap()
            .photos
            .into_iter()
            .map(|p| p.path.to_string())
            .collect();
        found.sort();
        assert_eq!(found, ["2020/b.PNG", "2020/trip/c.heic", "a.jpg"]);
    }

    #[test]
    fn skips_dot_folders_and_honours_leohtignore() {
        let dir = tempfile::tempdir().unwrap();
        let r = root(&dir);
        for f in [".leoht/thumb.jpg", "keep/a.jpg", "private/b.jpg"] {
            touch(&r, f);
        }
        std::fs::write(r.join(".leohtignore"), "private/\n").unwrap();
        let found: Vec<_> = scan_library(&r)
            .unwrap()
            .photos
            .into_iter()
            .map(|p| p.path.to_string())
            .collect();
        assert_eq!(found, ["keep/a.jpg"]);
    }

    #[test]
    fn records_format_and_size() {
        let dir = tempfile::tempdir().unwrap();
        let r = root(&dir);
        touch(&r, "x.CR3");
        let report = scan_library(&r).unwrap();
        assert_eq!(report.photos[0].format, PhotoFormat::Raw);
        assert_eq!(report.photos[0].size_bytes, 1);
    }

    #[test]
    fn missing_root_is_an_error() {
        assert!(matches!(
            scan_library(Utf8Path::new("/definitely/not/here")),
            Err(ScanError::NotADirectory(_))
        ));
    }
}
