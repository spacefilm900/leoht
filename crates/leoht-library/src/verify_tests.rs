//! Unit tests for `verify`.

use std::fs;

use camino::Utf8PathBuf;
use leoht_testkit::corpus::write_metadata_corpus;

use super::*;
use crate::ThumbOptions;

fn utf8(dir: &tempfile::TempDir) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap()
}

#[test]
fn verify_reports_drift_and_never_writes() {
    let (lib_dir, cache) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let root = utf8(&lib_dir);
    write_metadata_corpus(&root).unwrap();
    let mut lib = Library::open(&root, Some(&utf8(&cache))).unwrap();
    lib.scan(|_| {}).unwrap();
    lib.make_thumbnails(ThumbOptions::default(), |_| {})
        .unwrap();

    let clean = lib.verify(true).unwrap();
    assert!(clean.in_sync(), "{clean:?}");
    assert_eq!((clean.on_disk, clean.in_catalog), (20, 20));
    assert_eq!(clean.hashes_checked, 10, "every hashed photo re-checked");
    assert_eq!(clean.damaged.len(), 6);

    // One file removed, one added, one rewritten with a new size.
    fs::remove_file(root.join("png/plain.png")).unwrap();
    fs::write(root.join("jpeg/new.jpg"), b"x").unwrap();
    fs::write(root.join("jpeg/no-exif.jpg"), b"shorter").unwrap();
    let catalog_before = lib.catalog().photos().unwrap();
    let drift = lib.verify(false).unwrap();
    assert!(!drift.in_sync());
    assert_eq!(drift.missing, vec![RelPath::new("png/plain.png").unwrap()]);
    assert_eq!(drift.unindexed, vec![RelPath::new("jpeg/new.jpg").unwrap()]);
    assert_eq!(
        drift.changed,
        vec![RelPath::new("jpeg/no-exif.jpg").unwrap()]
    );
    assert_eq!(
        lib.catalog().photos().unwrap(),
        catalog_before,
        "catalog untouched"
    );
}

#[test]
fn hash_check_catches_content_changed_behind_the_same_mtime() {
    let (lib_dir, cache) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let root = utf8(&lib_dir);
    write_metadata_corpus(&root).unwrap();
    let mut lib = Library::open(&root, Some(&utf8(&cache))).unwrap();
    lib.scan(|_| {}).unwrap();
    lib.make_thumbnails(ThumbOptions::default(), |_| {})
        .unwrap();
    let path = root.join("png/plain.png");
    let mtime = fs::metadata(&path).unwrap().modified().unwrap();
    let mut bytes = fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 1; // same length
    fs::write(&path, bytes).unwrap();
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    assert!(
        lib.verify(false).unwrap().in_sync(),
        "size+mtime cannot see it"
    );
    let deep = lib.verify(true).unwrap();
    assert_eq!(
        deep.hash_mismatch,
        vec![RelPath::new("png/plain.png").unwrap()]
    );
}
