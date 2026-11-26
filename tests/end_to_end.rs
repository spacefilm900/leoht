//! Generate → scan → hash → catalog → thumbnail → sidecar, across all crates.

mod common;

use common::TempLibrary;
use leoht_catalog::Catalog;
use leoht_core::{EditOp, EditRecipe, PhotoFormat};
use leoht_edit::{Sidecar, sidecar_path_for};
use leoht_scan::{hash_file, scan_library};
use leoht_testkit::{LibrarySpec, generate_library};
use leoht_thumb::{cache_path, render_to_cache};

#[test]
fn a_small_library_flows_through_every_crate() {
    let lib = TempLibrary::new();
    generate_library(
        &lib.root,
        &LibrarySpec {
            count: 40,
            photos_per_folder: 10,
            ..Default::default()
        },
    )
    .unwrap();

    // Scan: every generated file is detected; folders come out as expected.
    let mut report = scan_library(&lib.root).unwrap();
    assert_eq!(report.photos.len(), 40);
    assert_eq!(report.skipped, 0);

    // Hash, then index into the catalog.
    for photo in &mut report.photos {
        photo.hash = Some(hash_file(&photo.path.to_absolute(&lib.root)).unwrap());
    }
    let mut catalog = Catalog::open_in_memory().unwrap();
    catalog.upsert_photos(&report.photos).unwrap();
    assert_eq!(catalog.photo_count().unwrap(), 40);
    assert_eq!(catalog.folder_counts().unwrap().len(), 4);

    // Thumbnail the first JPEG into the content-addressed cache.
    let jpeg = report
        .photos
        .iter()
        .find(|p| p.format == PhotoFormat::Jpeg)
        .unwrap();
    let dest = cache_path(
        &lib.root.join(".leoht/cache"),
        jpeg.hash.as_ref().unwrap(),
        8,
    );
    render_to_cache(&jpeg.path.to_absolute(&lib.root), &dest, 8).unwrap();
    assert!(dest.is_file());

    // Sidecar: write an edit next to the photo; the original bytes must be untouched.
    let image_path = jpeg.path.to_absolute(&lib.root);
    let before = hash_file(&image_path).unwrap();
    let sidecar_path = sidecar_path_for(&image_path);
    Sidecar::new(EditRecipe {
        ops: vec![EditOp::Rotate { quarter_turns: 1 }],
    })
    .write(&sidecar_path)
    .unwrap();
    assert_eq!(
        hash_file(&image_path).unwrap(),
        before,
        "editing must never touch the original"
    );

    // The catalog is a cache: wipe it, rescan, and the same state comes back.
    catalog.clear().unwrap();
    catalog
        .upsert_photos(&scan_library(&lib.root).unwrap().photos)
        .unwrap();
    assert_eq!(catalog.photo_count().unwrap(), 40);
    // ...and the sidecar file (not a photo) is not indexed.
    assert!(sidecar_path.is_file());
}
