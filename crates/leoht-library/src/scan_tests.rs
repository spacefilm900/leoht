//! Unit tests for the incremental scan (kept beside it to keep `scan.rs` short).

use std::fs;
use std::time::{Duration, SystemTime};

use camino::{Utf8Path, Utf8PathBuf};
use leoht_testkit::{LibrarySpec, generate_library};

use super::*;

struct Fixture {
    _dirs: (tempfile::TempDir, tempfile::TempDir),
    root: Utf8PathBuf,
    cache: Utf8PathBuf,
}

fn utf8(dir: &tempfile::TempDir) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap()
}

fn fixture(count: usize) -> Fixture {
    let (lib, cache) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let root = utf8(&lib);
    generate_library(
        &root,
        &LibrarySpec {
            count,
            photos_per_folder: 10,
            ..LibrarySpec::default()
        },
    )
    .unwrap();
    let cache_root = utf8(&cache);
    Fixture {
        _dirs: (lib, cache),
        root,
        cache: cache_root,
    }
}

/// Rewrites a file with different bytes and a clearly different mtime.
fn touch_changed(path: &Utf8Path) {
    let mut bytes = fs::read(path).unwrap();
    bytes.push(0);
    fs::write(path, bytes).unwrap();
    let later = SystemTime::now() + Duration::from_secs(10);
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(later)
        .unwrap();
}

#[test]
fn second_scan_of_an_unchanged_library_reads_nothing() {
    let f = fixture(30);
    let mut lib = Library::open(&f.root, Some(&f.cache)).unwrap();
    let first = lib.scan(|_| {}).unwrap();
    assert_eq!((first.found, first.added, first.probed), (30, 30, 30));
    let second = lib.scan(|_| {}).unwrap();
    assert_eq!(
        (second.found, second.unchanged, second.probed, second.added),
        (30, 30, 0, 0)
    );
    assert_eq!(lib.catalog().photo_count().unwrap(), 30);
}

#[test]
fn rescan_processes_only_added_changed_and_removed_files() {
    let f = fixture(30);
    let mut lib = Library::open(&f.root, Some(&f.cache)).unwrap();
    lib.scan(|_| {}).unwrap();
    let photos = lib.catalog().photos().unwrap();
    touch_changed(&photos[0].path.to_absolute(&f.root));
    touch_changed(&photos[1].path.to_absolute(&f.root));
    fs::remove_file(photos[2].path.to_absolute(&f.root)).unwrap();
    fs::copy(
        photos[3].path.to_absolute(&f.root),
        f.root.join("new-photo.jpg"),
    )
    .unwrap();
    let s = lib.scan(|_| {}).unwrap();
    assert_eq!((s.added, s.updated, s.removed, s.probed), (1, 2, 1, 3));
    assert_eq!(s.unchanged, 27);
    assert_eq!(lib.catalog().photo_count().unwrap(), 30);
    let changed = lib.catalog().photo(&photos[0].path).unwrap().unwrap();
    assert_eq!(changed.size_bytes, photos[0].size_bytes + 1);
}

#[test]
fn a_catalog_from_other_probe_rules_is_fully_reprobed() {
    let f = fixture(12);
    let mut lib = Library::open(&f.root, Some(&f.cache)).unwrap();
    lib.scan(|_| {}).unwrap();
    lib.catalog_mut().set_info("probe_version", "0").unwrap();
    let s = lib.scan(|_| {}).unwrap();
    assert_eq!((s.probed, s.updated, s.unchanged), (12, 12, 0));
}

#[test]
fn progress_is_reported_per_committed_batch() {
    let f = fixture(BATCH_SIZE * 2 + 7);
    let mut lib = Library::open(&f.root, Some(&f.cache)).unwrap();
    let mut seen = Vec::new();
    lib.scan(|p| seen.push(p.committed)).unwrap();
    assert_eq!(seen, [BATCH_SIZE, BATCH_SIZE * 2, BATCH_SIZE * 2 + 7]);
}

#[test]
fn rebuild_after_deleting_the_catalog_restores_the_same_rows() {
    let f = fixture(25);
    let mut lib = Library::open(&f.root, Some(&f.cache)).unwrap();
    lib.scan(|_| {}).unwrap();
    let before = lib.catalog().photos().unwrap();
    let catalog_file = lib.paths().catalog.clone();
    drop(lib);
    fs::remove_file(&catalog_file).unwrap();
    let mut lib = Library::open(&f.root, Some(&f.cache)).unwrap();
    assert_eq!(lib.catalog().photo_count().unwrap(), 0);
    lib.scan(|_| {}).unwrap();
    assert_eq!(lib.catalog().photos().unwrap(), before);
    let rebuilt = lib.rebuild(|_| {}).unwrap();
    assert_eq!(rebuilt.added, 25);
    assert_eq!(lib.catalog().photos().unwrap(), before);
}

#[test]
fn cache_inside_the_library_is_refused_and_nothing_is_left_behind() {
    let f = fixture(1);
    let inside = f.root.join("cache-here");
    let err = Library::open(&f.root, Some(&inside)).err().unwrap();
    assert!(
        matches!(err, LibraryError::CacheInsideLibrary { .. }),
        "{err}"
    );
    // Only the empty parent folders may remain; no catalog was created.
    assert!(!inside.join("libraries").read_dir().unwrap().any(|_| true));
}

#[test]
fn cache_folder_says_which_library_it_belongs_to() {
    let f = fixture(1);
    let lib = Library::open(&f.root, Some(&f.cache)).unwrap();
    let note = fs::read_to_string(lib.paths().cache_dir.join("library-root.txt")).unwrap();
    assert!(note.contains(lib.paths().root.as_str()));
    assert!(
        lib.paths()
            .cache_dir
            .starts_with(fs::canonicalize(&f.cache).unwrap())
    );
}
