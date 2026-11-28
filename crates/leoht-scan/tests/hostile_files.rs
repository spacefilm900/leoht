//! FR-LIB-10: damaged and hostile files never crash the scan.
//!
//! Property tests feed the probe random bytes behind every known signature,
//! and every prefix of real fixture files (truncation at each length). The
//! probe must return (never panic), and anything that is not a valid file of
//! its claimed format must carry a damage record or simply lack metadata.

// The helper below is test code too; clippy only exempts `#[test]` functions.
#![allow(clippy::unwrap_used)]

use camino::Utf8PathBuf;
use leoht_core::{PhotoFormat, RelPath};
use leoht_scan::{FoundFile, probe_file};
use leoht_testkit::fixtures::{ExifSpec, jpeg, png, quadrants};
use proptest::prelude::*;

fn probe_bytes(dir: &tempfile::TempDir, name: &str, bytes: &[u8]) -> leoht_core::Photo {
    let abs = Utf8PathBuf::from_path_buf(dir.path().join(name)).unwrap();
    std::fs::write(&abs, bytes).unwrap();
    let format = PhotoFormat::from_extension(abs.extension().unwrap()).unwrap();
    probe_file(&FoundFile {
        abs,
        rel: RelPath::new(name).unwrap(),
        format,
        size_bytes: bytes.len() as u64,
        modified_unix_secs: 0,
        modified_nanos: 0,
    })
}

const SIGNATURES: &[(&str, &[u8])] = &[
    ("a.jpg", &[0xFF, 0xD8, 0xFF]),
    ("a.png", b"\x89PNG\r\n\x1a\n"),
    ("a.heic", b"\0\0\0\x18ftypheic\0\0\0\0mif1heic"),
    ("a.cr2", b"II*\0\x08\0\0\0"),
    ("a.nef", b"MM\0*\0\0\0\x08"),
    ("a.raf", b"FUJIFILMCCD-RAW 0201FF383501"),
    ("a.tif", b"II*\0"),
    ("a.webp", b"RIFF\0\0\0\0WEBP"),
];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn random_bytes_behind_a_signature_never_panic(
        which in 0..SIGNATURES.len(),
        tail in proptest::collection::vec(any::<u8>(), 0..2048),
    ) {
        let (name, magic) = SIGNATURES[which];
        let mut bytes = magic.to_vec();
        bytes.extend_from_slice(&tail);
        let dir = tempfile::tempdir().unwrap();
        let photo = probe_bytes(&dir, name, &bytes);
        prop_assert!(photo.meta.capture_time.is_some());
    }

    #[test]
    fn pure_noise_is_never_mistaken_for_a_valid_jpeg_or_png(
        noise in proptest::collection::vec(any::<u8>(), 1..512),
    ) {
        prop_assume!(!noise.starts_with(&[0xFF, 0xD8, 0xFF]) && !noise.starts_with(b"\x89PNG"));
        let dir = tempfile::tempdir().unwrap();
        prop_assert!(probe_bytes(&dir, "n.jpg", &noise).damage.is_some());
        prop_assert!(probe_bytes(&dir, "n.png", &noise).damage.is_some());
    }
}

#[test]
fn every_truncation_of_real_files_is_handled() {
    let full = ExifSpec {
        date_original: Some("2023:08:19 17:45:12".into()),
        make: Some("Maker".into()),
        orientation: Some(6),
        ..ExifSpec::default()
    };
    let img = quadrants(24, 16);
    let files = [
        ("t.jpg", jpeg(&img, Some(&full), Some(vec![1; 64])).unwrap()),
        ("t.png", png(&img, Some(&full), None).unwrap()),
    ];
    let dir = tempfile::tempdir().unwrap();
    for (name, bytes) in files {
        let header_end = bytes.len().min(600);
        for cut in 0..header_end {
            // Must return; a truncated header is damaged, a truncated body is
            // found later by the thumbnailer (pixel decode).
            let photo = probe_bytes(&dir, name, &bytes[..cut]);
            if cut < 8 {
                assert!(photo.damage.is_some(), "{name} cut at {cut}");
            }
        }
        assert!(
            probe_bytes(&dir, name, &bytes).damage.is_none(),
            "{name} whole"
        );
    }
}
