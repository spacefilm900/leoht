//! The metadata and damage corpus through the scanner (FR-LIB-04, FR-LIB-06, FR-LIB-10).
//!
//! The golden JSON (`snapshots/scan_corpus__metadata_corpus.snap`) pins every
//! extracted field for every fixture; a change in extraction shows up as a
//! snapshot diff to review (`cargo insta review`).

mod common;

use common::TempLibrary;
use leoht_core::{DamageKind, DateSource, FormatSupport, PhotoFormat};
use leoht_scan::scan_library;
use leoht_testkit::corpus::write_metadata_corpus;
use serde_json::json;

#[test]
fn metadata_corpus_matches_the_golden_json() {
    let lib = TempLibrary::new();
    let written = write_metadata_corpus(&lib.root).unwrap();
    let report = scan_library(&lib.root).unwrap();
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);

    // Exactly the photo files are listed: no sidecars, no text files.
    let listed: Vec<String> = report.photos.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(listed, written);

    let golden: Vec<_> = report
        .photos
        .iter()
        .map(|p| {
            json!({
                "path": p.path.as_str(),
                "format": p.format,
                "support": p.format.support(),
                "damage": p.damage,
                "meta": p.meta,
            })
        })
        .collect();
    insta::assert_json_snapshot!("metadata_corpus", golden);
}

#[test]
fn every_damaged_fixture_is_recorded_with_a_reason_and_nothing_else_is() {
    let lib = TempLibrary::new();
    write_metadata_corpus(&lib.root).unwrap();
    let report = scan_library(&lib.root).unwrap();
    for photo in &report.photos {
        let in_damaged_folder = photo.path.as_str().starts_with("damaged/");
        assert_eq!(
            photo.damage.is_some(),
            in_damaged_folder,
            "{}: {:?}",
            photo.path,
            photo.damage
        );
        if let Some(d) = &photo.damage {
            assert!(!d.detail.is_empty(), "{} has an empty reason", photo.path);
        }
    }
    let kind = |name: &str| {
        report
            .photos
            .iter()
            .find(|p| p.path.as_str() == name)
            .and_then(|p| p.damage.as_ref())
            .map(|d| d.kind)
    };
    assert_eq!(kind("damaged/empty.jpg"), Some(DamageKind::Empty));
    assert_eq!(
        kind("damaged/text-file.jpg"),
        Some(DamageKind::ContentMismatch)
    );
    assert_eq!(
        kind("damaged/truncated.jpg"),
        Some(DamageKind::UnreadableHeader)
    );
    assert_eq!(
        kind("damaged/garbled.png"),
        Some(DamageKind::UnreadableHeader)
    );
    assert_eq!(
        kind("damaged/huge-dimensions.jpg"),
        Some(DamageKind::TooLarge)
    );
    assert_eq!(kind("damaged/noise.nef"), Some(DamageKind::ContentMismatch));
}

#[test]
fn support_levels_follow_the_format_matrix() {
    let lib = TempLibrary::new();
    write_metadata_corpus(&lib.root).unwrap();
    let report = scan_library(&lib.root).unwrap();
    for photo in report.photos.iter().filter(|p| p.damage.is_none()) {
        let expected = match photo.format {
            PhotoFormat::Jpeg | PhotoFormat::Png | PhotoFormat::Heic => FormatSupport::Full,
            PhotoFormat::Raw => FormatSupport::Partial,
            PhotoFormat::Tiff | PhotoFormat::Webp => FormatSupport::DetectOnly,
        };
        assert_eq!(photo.format.support(), expected, "{}", photo.path);
        // Every photo has a capture time: from EXIF or, failing that, the file date.
        let time = photo.meta.capture_time.as_ref().unwrap();
        if photo.path.as_str() == "jpeg/no-exif.jpg" {
            assert_eq!(time.source, DateSource::FileModified);
            assert_eq!(time.local, "2021-03-04T05:06:07Z");
        }
    }
}
