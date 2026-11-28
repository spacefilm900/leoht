//! Reading and writing photo rows.

use std::collections::HashMap;
use std::time::Duration;

use camino::Utf8Path;
use leoht_core::{ContentHash, Photo, RelPath};
use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

use crate::row::{COLUMN_COUNT, COLUMNS, photo_from_row, photo_params};
use crate::{CatalogError, migrations};

/// How long a connection waits for another one's write lock (a scan in progress)
/// before giving up. Readers in WAL mode never wait for writers.
const BUSY_TIMEOUT: Duration = Duration::from_secs(10);

/// The file-system facts a rescan compares to decide whether a file changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileState {
    /// Size in bytes.
    pub size_bytes: u64,
    /// Whole seconds of the modification time.
    pub modified_unix_secs: i64,
    /// Nanoseconds of the modification time.
    pub modified_nanos: u32,
}

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
        conn.busy_timeout(BUSY_TIMEOUT)?;
        // WAL lets a UI (or `leoht list`) read while a scan writes; harmless in memory.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // NORMAL is durable enough for a rebuildable cache and much faster than FULL.
        conn.pragma_update(None, "synchronous", "NORMAL")?;
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

    /// Deletes the rows for `paths` (files that disappeared), in one transaction.
    ///
    /// # Errors
    /// SQLite failure.
    pub fn remove_photos(&mut self, paths: &[RelPath]) -> Result<(), CatalogError> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached("DELETE FROM photos WHERE path = ?1")?;
            for path in paths {
                stmt.execute(params![path.as_str()])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Records the content hash of one photo (computed lazily, e.g. for thumbnails).
    ///
    /// # Errors
    /// SQLite failure.
    pub fn set_hash(&mut self, path: &RelPath, hash: &ContentHash) -> Result<(), CatalogError> {
        self.conn.execute(
            "UPDATE photos SET content_hash = ?2 WHERE path = ?1",
            params![path.as_str(), hash.as_str()],
        )?;
        Ok(())
    }

    /// Looks up one photo by its library-relative path.
    ///
    /// # Errors
    /// SQLite failure, or a corrupt row.
    pub fn photo(&self, path: &RelPath) -> Result<Option<Photo>, CatalogError> {
        let sql = format!("SELECT {COLUMNS} FROM photos WHERE path = ?1");
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let mut rows = stmt.query(params![path.as_str()])?;
        rows.next()?.map(photo_from_row).transpose()
    }

    /// Every photo, ordered by path (byte order of the UTF-8 path).
    ///
    /// # Errors
    /// SQLite failure, or a corrupt row.
    pub fn photos(&self) -> Result<Vec<Photo>, CatalogError> {
        let sql = format!("SELECT {COLUMNS} FROM photos ORDER BY path");
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(photo_from_row(row)?);
        }
        Ok(out)
    }

    /// Size and modification time of every indexed file, keyed by path: what an
    /// incremental rescan compares against the disk.
    ///
    /// # Errors
    /// SQLite failure, or a corrupt row.
    pub fn file_states(&self) -> Result<HashMap<RelPath, FileState>, CatalogError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, size_bytes, modified_unix_secs, modified_nanos FROM photos")?;
        let mut rows = stmt.query([])?;
        let mut out = HashMap::new();
        while let Some(row) = rows.next()? {
            let path: String = row.get(0)?;
            let size: i64 = row.get(1)?;
            let state = FileState {
                size_bytes: u64::try_from(size).unwrap_or(0),
                modified_unix_secs: row.get(2)?,
                modified_nanos: row.get(3)?,
            };
            out.insert(RelPath::new(path)?, state);
        }
        Ok(out)
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

    /// Number of photos recorded as damaged.
    ///
    /// # Errors
    /// SQLite failure.
    pub fn damaged_count(&self) -> Result<u64, CatalogError> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM photos WHERE damage_kind IS NOT NULL",
            [],
            |r| r.get(0),
        )?;
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

    /// Reads a catalog fact (for example the probe version it was built with).
    ///
    /// # Errors
    /// SQLite failure.
    pub fn info(&self, key: &str) -> Result<Option<String>, CatalogError> {
        Ok(self
            .conn
            .query_row(
                "SELECT value FROM catalog_info WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?)
    }

    /// Writes a catalog fact.
    ///
    /// # Errors
    /// SQLite failure.
    pub fn set_info(&mut self, key: &str, value: &str) -> Result<(), CatalogError> {
        self.conn.execute(
            "INSERT INTO catalog_info (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
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

    #[cfg(test)]
    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// Upserts `photos` using an already-open transaction (the caller commits).
fn upsert_in(tx: &rusqlite::Transaction<'_>, photos: &[Photo]) -> Result<(), CatalogError> {
    let placeholders: Vec<String> = (1..=COLUMN_COUNT).map(|i| format!("?{i}")).collect();
    let updates: Vec<String> = COLUMNS
        .split(", ")
        .skip(1) // the primary key
        .map(|c| format!("{c} = excluded.{c}"))
        .collect();
    let sql = format!(
        "INSERT INTO photos ({COLUMNS}) VALUES ({}) ON CONFLICT(path) DO UPDATE SET {}",
        placeholders.join(", "),
        updates.join(", ")
    );
    let mut stmt = tx.prepare_cached(&sql)?;
    for photo in photos {
        stmt.execute(params_from_iter(photo_params(photo)))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
