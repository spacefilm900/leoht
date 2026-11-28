//! TIFF-structured RAW files with a private magic number (Olympus ORF,
//! Panasonic RW2) still yield their EXIF (FR-LIB-04: RAW detected with metadata).

#![allow(clippy::unwrap_used)] // test helpers

use camino::Utf8PathBuf;
use leoht_core::{DateSource, PhotoFormat, RelPath};
use leoht_scan::{FoundFile, probe_file};
use leoht_testkit::fixtures::{ExifSpec, exif_tiff_ordered};

fn probe(name: &str, bytes: &[u8]) -> leoht_core::Photo {
    let dir = tempfile::tempdir().unwrap();
    let abs = Utf8PathBuf::from_path_buf(dir.path().join(name)).unwrap();
    std::fs::write(&abs, bytes).unwrap();
    probe_file(&FoundFile {
        abs,
        rel: RelPath::new(name).unwrap(),
        format: PhotoFormat::Raw,
        size_bytes: bytes.len() as u64,
        modified_unix_secs: 0,
        modified_nanos: 0,
    })
}

#[test]
fn orf_and_rw2_metadata_is_read() {
    let spec = ExifSpec {
        make: Some("OLYMPUS".into()),
        date_original: Some("2021:05:06 07:08:09".into()),
        ..ExifSpec::default()
    };
    // (file, byte order, the maker's private magic in place of TIFF's 42)
    for (name, little_endian, magic) in [
        ("a.orf", true, b"RO"),
        ("b.orf", true, b"RS"),
        ("c.orf", false, b"OR"),
        ("d.rw2", true, b"U\0"),
    ] {
        let mut bytes = exif_tiff_ordered(&spec, little_endian).unwrap();
        bytes[2..4].copy_from_slice(magic);
        let photo = probe(name, &bytes);
        assert!(photo.damage.is_none(), "{name}: {:?}", photo.damage);
        assert_eq!(photo.meta.camera_make.as_deref(), Some("OLYMPUS"), "{name}");
        let source = photo.meta.capture_time.map(|c| c.source);
        assert_eq!(source, Some(DateSource::ExifOriginal), "{name}");
    }
}
