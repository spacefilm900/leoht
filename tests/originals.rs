//! FR-FND-01: originals are never modified.
//!
//! Every Phase 1 operation runs against a fixture library (all formats, damaged
//! files, awkward names, a generated batch), and the whole tree, every file's
//! bytes and modification time and every folder, must be identical afterwards.
//! Each operation also runs with the entire library read-only (files and
//! folders, like a read-only volume), with the catalog stored elsewhere.
//!
//! The matrix grows each phase (Phase 1: scan, rescan, rebuild; thumbnails join
//! with the thumbnail cache; later phases add view, edit, rate and so on).

mod common;

use camino::Utf8Path;
use common::{TempLibrary, folder_rejects_writes, set_tree_readonly, tree_fingerprint};
use leoht_library::Library;
use leoht_testkit::corpus::{write_metadata_corpus, write_unicode_corpus};
use leoht_testkit::{LibrarySpec, generate_library};

/// One operation from the matrix, run against an open library.
type Operation = fn(&Utf8Path, &Utf8Path);

fn scan(root: &Utf8Path, cache: &Utf8Path) {
    Library::open(root, Some(cache))
        .unwrap()
        .scan(|_| {})
        .unwrap();
}

fn rescan(root: &Utf8Path, cache: &Utf8Path) {
    let mut lib = Library::open(root, Some(cache)).unwrap();
    lib.scan(|_| {}).unwrap();
    lib.scan(|_| {}).unwrap();
}

fn rebuild(root: &Utf8Path, cache: &Utf8Path) {
    let mut lib = Library::open(root, Some(cache)).unwrap();
    lib.scan(|_| {}).unwrap();
    lib.rebuild(|_| {}).unwrap();
}

const MATRIX: &[(&str, Operation)] = &[("scan", scan), ("rescan", rescan), ("rebuild", rebuild)];

fn fixture_library() -> TempLibrary {
    let lib = TempLibrary::new();
    write_metadata_corpus(&lib.root.join("corpus")).unwrap();
    write_unicode_corpus(&lib.root.join("names")).unwrap();
    generate_library(
        &lib.root.join("generated"),
        &LibrarySpec {
            count: 60,
            photos_per_folder: 20,
            ..LibrarySpec::default()
        },
    )
    .unwrap();
    lib
}

#[test]
fn no_operation_changes_any_original_file_or_folder() {
    for (name, op) in MATRIX {
        let lib = fixture_library();
        let cache = TempLibrary::new();
        let before = tree_fingerprint(&lib.root);
        op(&lib.root, &cache.root);
        assert_eq!(
            tree_fingerprint(&lib.root),
            before,
            "operation `{name}` changed the library"
        );
    }
}

#[test]
fn operations_work_on_a_fully_read_only_library() {
    for (name, op) in MATRIX {
        let lib = fixture_library();
        let cache = TempLibrary::new();
        let before = tree_fingerprint(&lib.root);
        set_tree_readonly(&lib.root, true);
        let protected = folder_rejects_writes(&lib.root);
        if !protected {
            set_tree_readonly(&lib.root, false);
        }
        // Running as root (or a filesystem ignoring permissions) would make
        // this test vacuous: fail loudly rather than pass.
        assert!(protected, "could not write-protect the library folder");
        let outcome = std::panic::catch_unwind(|| op(&lib.root, &cache.root));
        set_tree_readonly(&lib.root, false); // so the temp dir can be deleted
        assert!(
            outcome.is_ok(),
            "operation `{name}` failed on a read-only library"
        );
        assert_eq!(
            tree_fingerprint(&lib.root),
            before,
            "operation `{name}` changed the library"
        );
    }
}
