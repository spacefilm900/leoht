//! Unit tests for the catalog store (kept beside it to keep `store.rs` short).

use camino::Utf8PathBuf;
use leoht_core::{
    CaptureTime, ColorInfo, Damage, DamageKind, DateSource, Orientation, PhotoFormat, Ratio,
};

use super::*;
use crate::SCHEMA_VERSION;

fn photo(path: &str, format: PhotoFormat) -> Photo {
    let mut p = Photo::new(RelPath::new(path).unwrap(), format, 42, 1_700_000_000);
    p.hash = Some(ContentHash::from_hex(&"ab".repeat(32)).unwrap());
    p
}

/// A photo with every optional field set, to prove nothing is lost in a round trip.
fn rich_photo() -> Photo {
    let mut p = photo("2024/Ünïcødé 📷/IMG_0001.JPG", PhotoFormat::Jpeg);
    p.modified_nanos = 123_456_789;
    p.meta.capture_time = Some(CaptureTime {
        local: "2024-06-01T12:34:56".into(),
        offset: Some("+01:00".into()),
        source: DateSource::ExifOriginal,
    });
    p.meta.camera_make = Some("Maker".into());
    p.meta.camera_model = Some("Model X".into());
    p.meta.lens_model = Some("24-70mm".into());
    p.meta.exposure.exposure_time = Some(Ratio::new(1, 250).unwrap());
    p.meta.exposure.f_number = Some(Ratio::new(28, 10).unwrap());
    p.meta.exposure.iso = Some(400);
    p.meta.exposure.focal_length_mm = Some(Ratio::new(50, 1).unwrap());
    p.meta.width = Some(4032);
    p.meta.height = Some(3024);
    p.meta.orientation = Orientation::from_exif(6).unwrap();
    p.meta.color = ColorInfo::Icc {
        description: Some("Display P3".into()),
    };
    p
}

#[test]
fn every_field_survives_a_round_trip() {
    let mut cat = Catalog::open_in_memory().unwrap();
    let rich = rich_photo();
    let mut damaged = photo("bad.png", PhotoFormat::Png);
    damaged.damage = Some(Damage::new(DamageKind::ContentMismatch, "not a png"));
    damaged.meta.color = ColorInfo::Srgb;
    cat.upsert_photos(&[rich.clone(), damaged.clone()]).unwrap();
    assert_eq!(cat.photo(&rich.path).unwrap().unwrap(), rich);
    assert_eq!(cat.photo(&damaged.path).unwrap().unwrap(), damaged);
    assert_eq!(cat.damaged_count().unwrap(), 1);
    // `photos()` is path-ordered.
    let all: Vec<_> = cat.photos().unwrap().into_iter().map(|p| p.path).collect();
    assert_eq!(all, [rich.path.clone(), damaged.path.clone()]);
}

#[test]
fn file_states_remove_and_set_hash() {
    let mut cat = Catalog::open_in_memory().unwrap();
    let a = rich_photo();
    let b = photo("b.jpg", PhotoFormat::Jpeg);
    cat.upsert_photos(&[a.clone(), b.clone()]).unwrap();
    let states = cat.file_states().unwrap();
    assert_eq!(
        states[&a.path],
        FileState {
            size_bytes: 42,
            modified_unix_secs: 1_700_000_000,
            modified_nanos: 123_456_789
        }
    );
    let new_hash = ContentHash::from_hex(&"cd".repeat(32)).unwrap();
    cat.set_hash(&b.path, &new_hash).unwrap();
    assert_eq!(cat.photo(&b.path).unwrap().unwrap().hash, Some(new_hash));
    cat.remove_photos(std::slice::from_ref(&a.path)).unwrap();
    assert_eq!(cat.photo_count().unwrap(), 1);
}

#[test]
fn info_values_are_upserted() {
    let mut cat = Catalog::open_in_memory().unwrap();
    assert_eq!(cat.info("probe_version").unwrap(), None);
    cat.set_info("probe_version", "1").unwrap();
    cat.set_info("probe_version", "2").unwrap();
    assert_eq!(cat.info("probe_version").unwrap().as_deref(), Some("2"));
}

#[test]
fn a_version_1_catalog_is_migrated_to_the_current_schema() {
    let dir = tempfile::tempdir().unwrap();
    let db = Utf8PathBuf::from_path_buf(dir.path().join("catalog.sqlite")).unwrap();
    {
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TABLE photos (path TEXT PRIMARY KEY, folder TEXT NOT NULL, format TEXT NOT NULL,
             size_bytes INTEGER NOT NULL, modified_unix_secs INTEGER NOT NULL, content_hash TEXT)
             WITHOUT ROWID;
             INSERT INTO photos VALUES ('a.jpg', '', 'jpeg', 1, 0, NULL);
             PRAGMA user_version = 1;",
        )
        .unwrap();
    }
    let mut cat = Catalog::open(&db).unwrap();
    // The cache was rebuilt empty; the next scan refills it.
    assert_eq!(cat.photo_count().unwrap(), 0);
    cat.upsert_photos(&[rich_photo()]).unwrap();
    let version: u32 = cat
        .connection()
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION);
}

#[test]
fn upsert_get_and_count() {
    let mut cat = Catalog::open_in_memory().unwrap();
    let a = photo("2020/a.jpg", PhotoFormat::Jpeg);
    cat.upsert_photos(&[a.clone(), photo("2020/b.heic", PhotoFormat::Heic)])
        .unwrap();
    assert_eq!(cat.photo_count().unwrap(), 2);
    assert_eq!(cat.photo(&a.path).unwrap().unwrap(), a);
    assert!(
        cat.photo(&RelPath::new("nope.jpg").unwrap())
            .unwrap()
            .is_none()
    );
}

#[test]
fn upsert_updates_existing_rows_instead_of_duplicating() {
    let mut cat = Catalog::open_in_memory().unwrap();
    let mut a = photo("a.png", PhotoFormat::Png);
    cat.upsert_photos(std::slice::from_ref(&a)).unwrap();
    a.size_bytes = 99;
    cat.upsert_photos(std::slice::from_ref(&a)).unwrap();
    assert_eq!(cat.photo_count().unwrap(), 1);
    assert_eq!(cat.photo(&a.path).unwrap().unwrap().size_bytes, 99);
}

#[test]
fn folders_are_derived_from_photo_paths() {
    let mut cat = Catalog::open_in_memory().unwrap();
    cat.upsert_photos(&[
        photo("root.jpg", PhotoFormat::Jpeg),
        photo("x/1.jpg", PhotoFormat::Jpeg),
        photo("x/2.jpg", PhotoFormat::Jpeg),
    ])
    .unwrap();
    assert_eq!(
        cat.folder_counts().unwrap(),
        vec![(String::new(), 1), ("x".to_owned(), 2)]
    );
}

#[test]
fn every_format_survives_a_round_trip() {
    let mut cat = Catalog::open_in_memory().unwrap();
    let formats = [
        PhotoFormat::Jpeg,
        PhotoFormat::Heic,
        PhotoFormat::Png,
        PhotoFormat::Raw,
        PhotoFormat::Tiff,
        PhotoFormat::Webp,
    ];
    for (i, f) in formats.into_iter().enumerate() {
        let p = photo(&format!("f{i}.bin"), f);
        cat.upsert_photos(std::slice::from_ref(&p)).unwrap();
        assert_eq!(cat.photo(&p.path).unwrap().unwrap().format, f);
    }
}

#[test]
fn reopening_a_file_database_is_idempotent_and_keeps_data() {
    let dir = tempfile::tempdir().unwrap();
    let db = Utf8PathBuf::from_path_buf(dir.path().join("catalog.sqlite")).unwrap();
    {
        let mut cat = Catalog::open(&db).unwrap();
        cat.upsert_photos(&[photo("a.jpg", PhotoFormat::Jpeg)])
            .unwrap();
    }
    let cat = Catalog::open(&db).unwrap();
    assert_eq!(cat.photo_count().unwrap(), 1);
}

#[test]
fn newer_schema_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let db = Utf8PathBuf::from_path_buf(dir.path().join("catalog.sqlite")).unwrap();
    Connection::open(&db)
        .unwrap()
        .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
        .unwrap();
    assert!(matches!(
        Catalog::open(&db),
        Err(CatalogError::NewerSchema { .. })
    ));
}

#[test]
fn clear_empties_the_catalog_for_a_rebuild() {
    let mut cat = Catalog::open_in_memory().unwrap();
    cat.upsert_photos(&[photo("a.jpg", PhotoFormat::Jpeg)])
        .unwrap();
    cat.clear().unwrap();
    assert_eq!(cat.photo_count().unwrap(), 0);
}

#[test]
fn replace_photos_swaps_contents_in_one_transaction() {
    let mut cat = Catalog::open_in_memory().unwrap();
    cat.upsert_photos(&[photo("old.jpg", PhotoFormat::Jpeg)])
        .unwrap();
    cat.replace_photos(&[
        photo("new1.jpg", PhotoFormat::Jpeg),
        photo("new2.png", PhotoFormat::Png),
    ])
    .unwrap();
    assert_eq!(cat.photo_count().unwrap(), 2);
    assert!(
        cat.photo(&RelPath::new("old.jpg").unwrap())
            .unwrap()
            .is_none()
    );
}

#[test]
fn failed_replace_keeps_the_previous_catalog() {
    let mut cat = Catalog::open_in_memory().unwrap();
    cat.upsert_photos(&[photo("keep.jpg", PhotoFormat::Jpeg)])
        .unwrap();
    // Force the insert step to fail after the delete has run, inside the same transaction.
    cat.conn
        .execute_batch(
            "CREATE TRIGGER boom BEFORE INSERT ON photos BEGIN SELECT RAISE(ABORT, 'boom'); END;",
        )
        .unwrap();
    assert!(
        cat.replace_photos(&[photo("new.jpg", PhotoFormat::Jpeg)])
            .is_err()
    );
    assert_eq!(cat.photo_count().unwrap(), 1);
    assert!(
        cat.photo(&RelPath::new("keep.jpg").unwrap())
            .unwrap()
            .is_some()
    );
}
