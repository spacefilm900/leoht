//! Volume tests: the scanner and catalog at library sizes that expose O(n²) mistakes.
//!
//! These are `#[ignore]`d so `cargo test --workspace` stays fast. CI's
//! `volume.yml` runs them with `-- --ignored`. Targets: 1k and 10k photos in
//! Phase 1; the 100k suite is Phase 6 (see docs/process/phase-gates.md).
//!
//! Environment:
//! * `LEOHT_VOLUME_LIBRARY` – ABSOLUTE path of an already generated library (from
//!   `cargo xtask gen-library`); cargo runs tests from the package folder, so relative paths break.
//! * `LEOHT_VOLUME_COUNT`   – otherwise generate this many photos (default 1000).

#![allow(clippy::print_stderr)] // timings are the output of these tests

mod common;

use std::time::Instant;

use camino::Utf8PathBuf;
use common::TempLibrary;
use leoht_catalog::Catalog;
use leoht_scan::{hash_file, scan_library};
use leoht_testkit::{LibrarySpec, generate_library};

#[test]
#[ignore = "volume test: run via `cargo test -p leoht-it --test volume --release -- --ignored`"]
fn scan_hash_and_index_a_large_library() {
    let temp;
    let (root, expected) = if let Ok(dir) = std::env::var("LEOHT_VOLUME_LIBRARY") {
        let report = scan_library(&Utf8PathBuf::from(&dir)).expect("scan pre-generated library");
        (Utf8PathBuf::from(dir), report.photos.len())
    } else {
        let count = std::env::var("LEOHT_VOLUME_COUNT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_000);
        temp = TempLibrary::new();
        generate_library(
            &temp.root,
            &LibrarySpec {
                count,
                ..LibrarySpec::default()
            },
        )
        .expect("generate");
        (temp.root.clone(), count)
    };

    let t = Instant::now();
    let mut report = scan_library(&root).expect("scan");
    eprintln!(
        "scan:    {:>7} photos in {:?}",
        report.photos.len(),
        t.elapsed()
    );
    assert_eq!(report.photos.len(), expected);
    assert_eq!(report.skipped, 0);

    let t = Instant::now();
    for photo in &mut report.photos {
        photo.hash = Some(hash_file(&photo.path.to_absolute(&root)).expect("hash"));
    }
    eprintln!(
        "hash:    {:>7} photos in {:?}",
        report.photos.len(),
        t.elapsed()
    );

    let t = Instant::now();
    let mut catalog = Catalog::open_in_memory().expect("catalog");
    catalog.upsert_photos(&report.photos).expect("upsert");
    eprintln!(
        "index:   {:>7} photos in {:?}",
        report.photos.len(),
        t.elapsed()
    );
    assert_eq!(
        catalog.photo_count().expect("count"),
        u64::try_from(expected).expect("fits")
    );

    let t = Instant::now();
    let folders = catalog.folder_counts().expect("folders");
    eprintln!("folders: {:>7} folders in {:?}", folders.len(), t.elapsed());
}
