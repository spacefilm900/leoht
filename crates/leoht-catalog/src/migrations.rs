//! Schema migrations, tracked with SQLite's built-in `PRAGMA user_version`.
//!
//! To change the schema: append a new entry to [`MIGRATIONS`]. Never edit an
//! existing entry — released databases have already run it. Because the catalog
//! is a rebuildable cache, a breaking change may also simply bump the version
//! and ask the user to re-scan.

use rusqlite::Connection;

use crate::CatalogError;

/// Ordered migrations; entry `n` upgrades the database from version `n` to `n + 1`.
const MIGRATIONS: &[&str] = &[
    // v1: one row per photo. Folders are derived (`GROUP BY folder`), not stored.
    "CREATE TABLE photos (
         path                TEXT PRIMARY KEY,   -- library-relative, forward slashes
         folder              TEXT NOT NULL,      -- parent path; '' for the library root
         format              TEXT NOT NULL,
         size_bytes          INTEGER NOT NULL,
         modified_unix_secs  INTEGER NOT NULL,
         content_hash        TEXT                -- 'blake3:<hex>' or NULL if not yet computed
     ) WITHOUT ROWID;
     CREATE INDEX photos_by_folder ON photos(folder);
     CREATE INDEX photos_by_hash ON photos(content_hash) WHERE content_hash IS NOT NULL;",
    // v2 (Phase 1): metadata, damage records and sub-second modification times.
    // The catalog is a cache, so the table is rebuilt rather than altered: the
    // next scan refills it (a v1 catalog only ever held scaffold test data).
    "DROP TABLE photos;
     CREATE TABLE photos (
         path                TEXT PRIMARY KEY,   -- library-relative, forward slashes
         folder              TEXT NOT NULL,      -- parent path; '' for the library root
         format              TEXT NOT NULL,      -- PhotoFormat::name()
         size_bytes          INTEGER NOT NULL,
         modified_unix_secs  INTEGER NOT NULL,
         modified_nanos      INTEGER NOT NULL,
         content_hash        TEXT,               -- 'blake3:<hex>' or NULL if not yet computed
         capture_local       TEXT,               -- ISO 8601 local time, or UTC ending in Z
         capture_offset      TEXT,               -- '+01:00' etc. when the file records it
         capture_source      TEXT,               -- exif_original | exif_digitized | file_modified
         camera_make         TEXT,
         camera_model        TEXT,
         lens_model          TEXT,
         exposure_time       TEXT,               -- EXIF rational 'num/den'
         f_number            TEXT,               -- EXIF rational 'num/den'
         iso                 INTEGER,
         focal_length_mm     TEXT,               -- EXIF rational 'num/den'
         width               INTEGER,            -- stored (pre-orientation) pixels
         height              INTEGER,
         orientation         INTEGER NOT NULL,   -- EXIF 1..=8
         color_kind          TEXT NOT NULL,      -- untagged | srgb | icc
         icc_description     TEXT,
         damage_kind         TEXT,               -- DamageKind::name() or NULL if readable
         damage_detail       TEXT
     ) WITHOUT ROWID;
     CREATE INDEX photos_by_folder ON photos(folder);
     CREATE INDEX photos_by_hash ON photos(content_hash) WHERE content_hash IS NOT NULL;
     CREATE INDEX photos_by_capture ON photos(capture_local);
     -- Small key/value facts about how the catalog was built (probe version, root).
     CREATE TABLE catalog_info (
         key    TEXT PRIMARY KEY,
         value  TEXT NOT NULL
     ) WITHOUT ROWID;",
];

/// Highest schema version this build understands.
#[allow(clippy::cast_possible_truncation)] // the migration list is tiny
pub const SCHEMA_VERSION: u32 = MIGRATIONS.len() as u32;

/// Brings `conn` up to [`SCHEMA_VERSION`], running only the migrations it lacks.
pub(crate) fn migrate(conn: &mut Connection) -> Result<(), CatalogError> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current > SCHEMA_VERSION {
        return Err(CatalogError::NewerSchema {
            found: current,
            supported: SCHEMA_VERSION,
        });
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let version = u32::try_from(index + 1).unwrap_or(u32::MAX);
        tracing::debug!(to_version = version, "applying catalog migration");
        // Each migration and its version bump commit together or not at all.
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}
