//! Unit tests for library thumbnailing.

use std::fs;

use camino::Utf8PathBuf;
use leoht_core::RelPath;
use leoht_testkit::corpus::write_metadata_corpus;
use leoht_testkit::fixtures;

use super::*;

struct Fixture {
    _dirs: (tempfile::TempDir, tempfile::TempDir),
    root: Utf8PathBuf,
    lib: Library,
}

fn utf8(dir: &tempfile::TempDir) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap()
}

fn scanned_corpus() -> Fixture {
    let (lib_dir, cache) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let root = utf8(&lib_dir);
    write_metadata_corpus(&root).unwrap();
    let mut lib = Library::open(&root, Some(&utf8(&cache))).unwrap();
    lib.scan(|_| {}).unwrap();
    Fixture {
        _dirs: (lib_dir, cache),
        root,
        lib,
    }
}

fn photo(lib: &Library, rel: &str) -> Photo {
    lib.catalog()
        .photo(&RelPath::new(rel).unwrap())
        .unwrap()
        .unwrap()
}

#[test]
fn corpus_thumbnails_by_format_and_cached_on_the_second_run() {
    let mut f = scanned_corpus();
    let mut ticks = 0;
    let first = f
        .lib
        .make_thumbnails(ThumbOptions::default(), |_| ticks += 1)
        .unwrap();
    // 5 JPEG + 3 PNG decoded, plus the RAF's embedded preview. The CR2 fixture
    // has no preview; HEIC, TIFF and WebP get none in Phase 1.
    assert_eq!(first.eligible, 10);
    assert_eq!(first.generated, 9, "{first:?}");
    assert_eq!(first.from_preview, 1);
    assert_eq!(first.no_preview, 1);
    assert_eq!(first.unsupported, 4);
    assert_eq!(first.skipped_damaged, 6);
    assert_eq!((first.newly_damaged, first.write_errors.len()), (0, 0));
    assert_eq!(ticks, 10);

    let raf = photo(&f.lib, "raw/fuji.raf");
    let thumb = f
        .lib
        .thumbnail_path(raf.hash.as_ref().unwrap(), DEFAULT_THUMB_EDGE);
    assert!(thumb.is_file(), "hash recorded and thumbnail written");

    let second = f
        .lib
        .make_thumbnails(ThumbOptions::default(), |_| {})
        .unwrap();
    assert_eq!((second.generated, second.cached), (0, 9));
}

#[test]
fn a_changed_photo_gets_a_new_thumbnail() {
    let mut f = scanned_corpus();
    f.lib
        .make_thumbnails(ThumbOptions::default(), |_| {})
        .unwrap();
    let rel = "png/plain.png";
    let before = photo(&f.lib, rel).hash.unwrap();
    let other = fixtures::png(&fixtures::quadrants(30, 20), None, None).unwrap();
    fs::write(f.root.join(rel), other).unwrap();
    f.lib.scan(|_| {}).unwrap();
    let again = f
        .lib
        .make_thumbnails(ThumbOptions::default(), |_| {})
        .unwrap();
    assert_eq!(again.generated, 1);
    let after = photo(&f.lib, rel).hash.unwrap();
    assert_ne!(before, after);
    let img = image::open(f.lib.thumbnail_path(&after, DEFAULT_THUMB_EDGE)).unwrap();
    assert_eq!((img.width(), img.height()), (30, 20));
}

#[test]
fn a_photo_that_will_not_decode_is_marked_damaged_once() {
    let mut f = scanned_corpus();
    // A PNG whose header is fine (so the scan accepts it) but whose image data is corrupt.
    let mut png = fixtures::png(&fixtures::quadrants(40, 40), None, None).unwrap();
    let idat = png.windows(4).position(|w| w == b"IDAT").unwrap();
    for b in &mut png[idat + 4..idat + 24] {
        *b ^= 0x5A;
    }
    fs::write(f.root.join("png/corrupt.png"), png).unwrap();
    let scan = f.lib.scan(|_| {}).unwrap();
    assert_eq!(scan.added, 1);
    let one_worker = ThumbOptions {
        workers: Some(1),
        ..ThumbOptions::default()
    };
    let run = f.lib.make_thumbnails(one_worker, |_| {}).unwrap();
    assert_eq!(run.newly_damaged, 1, "{run:?}");
    let damage = photo(&f.lib, "png/corrupt.png").damage.unwrap();
    assert_eq!(damage.kind, DamageKind::DecodeFailed);
    let rerun = f.lib.make_thumbnails(one_worker, |_| {}).unwrap();
    assert_eq!((rerun.newly_damaged, rerun.skipped_damaged), (0, 7));
}
