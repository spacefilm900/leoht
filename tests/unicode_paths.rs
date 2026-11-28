//! FR-FND-10: Unicode names, whitespace, punctuation, very long paths and
//! case differences work through scan, catalog and rebuild.

mod common;

use common::TempLibrary;
use leoht_core::RelPath;
use leoht_library::Library;
use leoht_testkit::corpus::write_unicode_corpus;

/// True if the filesystem under `dir` treats `a` and `A` as different names
/// (Linux usually; macOS APFS and Windows NTFS usually not).
fn case_sensitive(dir: &camino::Utf8Path) -> bool {
    std::fs::write(dir.join("case-probe"), b"").unwrap();
    let sensitive = !dir.join("CASE-PROBE").exists();
    std::fs::remove_file(dir.join("case-probe")).unwrap();
    sensitive
}

#[test]
fn awkward_names_are_listed_exactly_and_survive_the_catalog() {
    let lib = TempLibrary::new();
    let cache = TempLibrary::new();
    let mut expected = write_unicode_corpus(&lib.root).unwrap();
    if case_sensitive(&lib.root) {
        for name in ["Case/photo.jpg", "Case/PHOTO.jpg", "case/photo.jpg"] {
            let path = lib.root.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::copy(lib.root.join(&expected[0]), &path).unwrap();
            expected.push(name.to_owned());
        }
    }
    expected.sort();
    let longest = expected.iter().map(|p| p.chars().count()).max().unwrap();
    assert!(
        longest > 400,
        "the corpus must include a 400+ character path"
    );

    let mut library = Library::open(&lib.root, Some(&cache.root)).unwrap();
    let summary = library.scan(|_| {}).unwrap();
    assert!(summary.skipped.is_empty(), "{:?}", summary.skipped);
    assert_eq!(summary.damaged, 0);
    let listed: Vec<String> = library
        .catalog()
        .photos()
        .unwrap()
        .into_iter()
        .map(|p| p.path.to_string())
        .collect();
    assert_eq!(listed, expected);

    // Every stored path points back at the real file, and a rebuild agrees.
    for rel in &listed {
        let rel = RelPath::new(rel).unwrap();
        assert!(rel.to_absolute(&library.paths().root).is_file(), "{rel}");
        let photo = library.catalog().photo(&rel).unwrap().unwrap();
        assert_eq!(photo.path, rel);
    }
    let before = library.catalog().photos().unwrap();
    library.rebuild(|_| {}).unwrap();
    assert_eq!(library.catalog().photos().unwrap(), before);
}
