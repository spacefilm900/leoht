//! Reading and writing photo rows.

use camino::Utf8Path;
use leoht_core::{ContentHash, Photo, PhotoFormat, RelPath};
use rusqlite::{Connection, params};

use crate::{CatalogError, migrations};

/// An open catalog database.
pub struct Catalog {
    conn: Connection,
}

impl Catalog {
    /// Opens (creating if needed) the catalog file at `path` and migrates it.
    ///
    /// # Errors
    /// SQLite failure, or a database from a newer Leoht.
    pub fn open(path: &Utf8Path) -> Result<Self, CatalogError> {
        Self::from_connection(Connection::open(path)?)
    }

    /// Opens a throw-away in-memory catalog (tests, benchmarks).
    ///
    /// # Errors
    /// SQLite failure.
    pub fn open_in_memory() -> Result<Self, CatalogError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(mut conn: Connection) -> Result<Self, CatalogError> {
        // WAL lets a UI read while a scan writes; harmless for in-memory databases.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        migrations::migrate(&mut conn)?;
        Ok(Self { conn })
    }

    /// Inserts or updates photos in a single transaction (fast for large batches).
    ///
    /// # Errors
    /// SQLite failure.
    pub fn upsert_photos(&mut self, photos: &[Photo]) -> Result<(), CatalogError> {
        let tx = self.conn.transaction()?;
        upsert_in(&tx, photos)?;
        tx.commit()?;
        Ok(())
    }

    /// Replaces the whole catalog with `photos` in **one transaction**: delete and
    /// insert commit together or not at all, so a failed rebuild leaves the previous
    /// index intact (unlike calling [`Catalog::clear`] then [`Catalog::upsert_photos`]).
    ///
    /// # Errors
    /// SQLite failure; the previous contents are kept.
    pub fn replace_photos(&mut self, photos: &[Photo]) -> Result<(), CatalogError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM photos", [])?;
        upsert_in(&tx, photos)?;
        tx.commit()?;
        Ok(())
    }

    /// Looks up one photo by its library-relative path.
    ///
    /// # Errors
    /// SQLite failure, or a corrupt row.
    pub fn photo(&self, path: &RelPath) -> Result<Option<Photo>, CatalogError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT format, size_bytes, modified_unix_secs, content_hash FROM photos WHERE path = ?1",
        )?;
        let mut rows = stmt.query(params![path.as_str()])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let format: String = row.get(0)?;
        let size: i64 = row.get(1)?;
        let hash: Option<String> = row.get(3)?;
        Ok(Some(Photo {
            path: path.clone(),
            format: format_from_str(&format)?,
            size_bytes: u64::try_from(size).unwrap_or(0),
            modified_unix_secs: row.get(2)?,
            hash: hash.map(ContentHash::try_from).transpose()?,
        }))
    }

    /// Total number of indexed photos.
    ///
    /// # Errors
    /// SQLite failure.
    pub fn photo_count(&self) -> Result<u64, CatalogError> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM photos", [], |r| r.get(0))?;
        Ok(u64::try_from(n).unwrap_or(0))
    }

    /// Photos per folder, sorted by folder path (`""` is the library root).
    ///
    /// # Errors
    /// SQLite failure.
    pub fn folder_counts(&self) -> Result<Vec<(String, u64)>, CatalogError> {
        let mut stmt = self
            .conn
            .prepare("SELECT folder, COUNT(*) FROM photos GROUP BY folder ORDER BY folder")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        rows.map(|r| {
            let (folder, n) = r?;
            Ok((folder, u64::try_from(n).unwrap_or(0)))
        })
        .collect()
    }

    /// Removes every photo row (an explicit "forget everything"). To rebuild from a
    /// fresh scan use [`Catalog::replace_photos`], which cannot lose the old data on failure.
    ///
    /// # Errors
    /// SQLite failure.
    pub fn clear(&mut self) -> Result<(), CatalogError> {
        self.conn.execute("DELETE FROM photos", [])?;
        Ok(())
    }
}

/// Upserts `photos` using an already-open transaction (the caller commits).
fn upsert_in(tx: &rusqlite::Transaction<'_>, photos: &[Photo]) -> Result<(), CatalogError> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO photos (path, folder, format, size_bytes, modified_unix_secs, content_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(path) DO UPDATE SET
             folder = excluded.folder,
             format = excluded.format,
             size_bytes = excluded.size_bytes,
             modified_unix_secs = excluded.modified_unix_secs,
             content_hash = excluded.content_hash",
    )?;
    for photo in photos {
        let folder = photo.folder().map_or_else(String::new, |f| f.to_string());
        stmt.execute(params![
            photo.path.as_str(),
            folder,
            format_to_str(photo.format),
            // SQLite integers are i64; photo files are nowhere near 8 EiB.
            i64::try_from(photo.size_bytes).unwrap_or(i64::MAX),
            photo.modified_unix_secs,
            photo.hash.as_ref().map(ContentHash::as_str),
        ])?;
    }
    Ok(())
}

/// Stable on-disk names for formats (decoupled from the Rust enum's spelling).
fn format_to_str(format: PhotoFormat) -> &'static str {
    match format {
        PhotoFormat::Jpeg => "jpeg",
        PhotoFormat::Heic => "heic",
        PhotoFormat::Png => "png",
        PhotoFormat::Raw => "raw",
        PhotoFormat::Tiff => "tiff",
        PhotoFormat::Webp => "webp",
    }
}

fn format_from_str(s: &str) -> Result<PhotoFormat, CatalogError> {
    match s {
        "jpeg" => Ok(PhotoFormat::Jpeg),
        "heic" => Ok(PhotoFormat::Heic),
        "png" => Ok(PhotoFormat::Png),
        "raw" => Ok(PhotoFormat::Raw),
        "tiff" => Ok(PhotoFormat::Tiff),
        "webp" => Ok(PhotoFormat::Webp),
        other => Err(CatalogError::UnknownFormat(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SCHEMA_VERSION;
    use camino::Utf8PathBuf;

    fn photo(path: &str, format: PhotoFormat) -> Photo {
        Photo {
            path: RelPath::new(path).unwrap(),
            format,
            size_bytes: 42,
            modified_unix_secs: 1_700_000_000,
            hash: Some(ContentHash::from_hex(&"ab".repeat(32)).unwrap()),
        }
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
}
