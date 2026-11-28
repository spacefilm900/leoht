//! Library-level scanning rules: what is listed and what is never touched
//! (FR-LIB-01, FR-LIB-03, FR-LIB-05, FR-LIB-08, FR-FND-02).

mod common;

use common::TempLibrary;
use leoht_library::Library;
use leoht_scan::CONFIG_FILE;
use leoht_testkit::corpus::write_metadata_corpus;

fn is_raw(rel: &str) -> bool {
    camino::Utf8Path::new(rel)
        .extension()
        .and_then(leoht_core::PhotoFormat::from_extension)
        == Some(leoht_core::PhotoFormat::Raw)
}

fn listed(lib: &Library) -> Vec<String> {
    lib.catalog()
        .photos()
        .unwrap()
        .into_iter()
        .map(|p| p.path.to_string())
        .collect()
}

fn put(root: &camino::Utf8Path, rel: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        leoht_testkit::fixtures::jpeg(&leoht_testkit::fixtures::quadrants(4, 4), None, None)
            .unwrap(),
    )
    .unwrap();
}

#[test]
fn scan_lists_exactly_the_visible_photos() {
    let lib = TempLibrary::new();
    let cache = TempLibrary::new();
    for rel in [
        "2020/a.jpg",
        "2020/deeply/nested/folder/b.jpg",
        "2020/._a.jpg",     // macOS resource fork: hidden
        ".Trash/c.jpg",     // hidden folder
        "2020/.hidden.jpg", // hidden file
        "root.jpg",
    ] {
        put(&lib.root, rel);
    }
    std::fs::write(lib.root.join("2020/readme.txt"), b"x").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(lib.root.join("2020"), lib.root.join("link-to-2020")).unwrap();

    let mut library = Library::open(&lib.root, Some(&cache.root)).unwrap();
    let summary = library.scan(|_| {}).unwrap();
    assert_eq!(
        listed(&library),
        ["2020/a.jpg", "2020/deeply/nested/folder/b.jpg", "root.jpg"]
    );
    #[cfg(unix)]
    assert_eq!(
        summary.skipped.len(),
        1,
        "the symlink is recorded as skipped"
    );
    let _ = summary;
}

#[cfg(unix)]
#[test]
fn an_ignored_folder_is_never_opened() {
    use std::os::unix::fs::PermissionsExt;
    let lib = TempLibrary::new();
    let cache = TempLibrary::new();
    put(&lib.root, "keep/a.jpg");
    put(&lib.root, "Backups/old.jpg");
    std::fs::write(
        lib.root.join(CONFIG_FILE),
        r#"{"version": 1, "ignore": ["Backups/"]}"#,
    )
    .unwrap();
    // A folder nobody may list: any attempt to open it would be recorded as skipped.
    let backups = lib.root.join("Backups");
    std::fs::set_permissions(&backups, std::fs::Permissions::from_mode(0o000)).unwrap();
    let mut library = Library::open(&lib.root, Some(&cache.root)).unwrap();
    let summary = library.scan(|_| {}).unwrap();
    std::fs::set_permissions(&backups, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(summary.skipped.is_empty(), "{:?}", summary.skipped);
    assert_eq!(listed(&library), ["keep/a.jpg"]);
}

#[test]
fn switching_raw_off_and_on_changes_the_listing_and_touches_no_file() {
    let lib = TempLibrary::new();
    let cache = TempLibrary::new();
    write_metadata_corpus(&lib.root).unwrap();
    let before = common::tree_fingerprint(&lib.root);
    let mut library = Library::open(&lib.root, Some(&cache.root)).unwrap();
    library.scan(|_| {}).unwrap();
    let all = listed(&library);
    assert!(all.iter().any(|p| p.starts_with("raw/")));

    let config = lib.root.join(CONFIG_FILE);
    std::fs::write(&config, r#"{"version": 1, "formats": {"raw": false}}"#).unwrap();
    library.scan(|_| {}).unwrap();
    let without_raw = listed(&library);
    assert!(without_raw.iter().all(|p| !is_raw(p)));
    assert!(without_raw.len() < all.len());

    std::fs::remove_file(&config).unwrap();
    library.scan(|_| {}).unwrap();
    assert_eq!(listed(&library), all);
    assert_eq!(common::tree_fingerprint(&lib.root), before);
}

#[test]
fn deleting_the_catalog_loses_nothing_and_sidecars_are_untouched() {
    let lib = TempLibrary::new();
    let cache = TempLibrary::new();
    write_metadata_corpus(&lib.root).unwrap();
    let mut library = Library::open(&lib.root, Some(&cache.root)).unwrap();
    library.scan(|_| {}).unwrap();
    let before = library.catalog().photos().unwrap();
    let sidecar = lib.root.join("jpeg/full-exif.jpg.leoht.json");
    let sidecar_bytes = std::fs::read(&sidecar).unwrap();
    let db = library.paths().catalog.clone();
    drop(library);
    std::fs::remove_file(&db).unwrap();

    let mut library = Library::open(&lib.root, Some(&cache.root)).unwrap();
    library.scan(|_| {}).unwrap();
    assert_eq!(library.catalog().photos().unwrap(), before);
    // Phase 1 has no user-authored data of its own yet; the sidecar (the home
    // of edits from Phase 3) must be byte-identical after the rebuild.
    assert_eq!(std::fs::read(&sidecar).unwrap(), sidecar_bytes);
}
