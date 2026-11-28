//! Opening a library and locating its cache files.

use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use leoht_catalog::Catalog;

use crate::{LibraryError, cache};

/// Name of the catalog database inside the per-library cache folder.
const CATALOG_FILE: &str = "catalog.sqlite";
/// Name of the human-readable note saying which folder a cache belongs to.
const ROOT_NOTE: &str = "library-root.txt";

/// Every location a library uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryPaths {
    /// The photo folder, canonicalised (absolute, symlinks resolved).
    pub root: Utf8PathBuf,
    /// This library's cache folder (outside `root`).
    pub cache_dir: Utf8PathBuf,
    /// The catalog database file.
    pub catalog: Utf8PathBuf,
    /// The thumbnail cache folder.
    pub thumbs: Utf8PathBuf,
}

/// An open library: its paths and its catalog.
pub struct Library {
    pub(crate) paths: LibraryPaths,
    pub(crate) catalog: Catalog,
}

impl Library {
    /// Opens the library at `root`, creating its cache folder and catalog if needed.
    /// `cache_root` overrides where caches live (see [`cache::cache_root`]).
    ///
    /// Nothing is written inside `root`.
    ///
    /// # Errors
    /// The root is not a folder, no cache folder can be found or created, the
    /// cache would be inside the library, or the catalog cannot be opened.
    pub fn open(root: &Utf8Path, cache_root: Option<&Utf8Path>) -> Result<Self, LibraryError> {
        let paths = resolve_paths(root, cache_root)?;
        let catalog = Catalog::open(&paths.catalog)?;
        Ok(Self { paths, catalog })
    }

    /// The library's locations.
    #[must_use]
    pub fn paths(&self) -> &LibraryPaths {
        &self.paths
    }

    /// The catalog (read access, e.g. for listing).
    #[must_use]
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// The catalog (write access, e.g. to record content hashes).
    pub fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }

    /// Deletes the catalog database and opens a fresh, empty one. Callers then
    /// scan to refill it ([`Library::rebuild`] does both).
    pub(crate) fn reset_catalog(&mut self) -> Result<(), LibraryError> {
        // Close the old connection first so its WAL is checkpointed and released.
        self.catalog = Catalog::open_in_memory()?;
        for suffix in ["", "-wal", "-shm"] {
            let path = Utf8PathBuf::from(format!("{}{suffix}", self.paths.catalog));
            match fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(source) => return Err(LibraryError::Cache { path, source }),
            }
        }
        self.catalog = Catalog::open(&self.paths.catalog)?;
        Ok(())
    }
}

fn resolve_paths(
    root: &Utf8Path,
    cache_root: Option<&Utf8Path>,
) -> Result<LibraryPaths, LibraryError> {
    if !root.is_dir() {
        return Err(leoht_scan::ScanError::NotADirectory(root.to_owned()).into());
    }
    let root = canonical(root)?;
    let cache_root = cache::cache_root(cache_root).ok_or(LibraryError::NoCacheDir)?;
    let cache_dir = cache::library_cache_dir(&cache_root, &root);
    fs::create_dir_all(&cache_dir).map_err(|source| LibraryError::Cache {
        path: cache_dir.clone(),
        source,
    })?;
    let cache_dir = canonical(&cache_dir)?;
    if cache_dir.starts_with(&root) {
        // Clean up the folder we just made: it must not stay inside the library.
        let _ = fs::remove_dir(&cache_dir);
        return Err(LibraryError::CacheInsideLibrary {
            cache: cache_dir,
            library: root,
        });
    }
    let note = cache_dir.join(ROOT_NOTE);
    if !note.exists() {
        let text = format!(
            "This Leoht cache belongs to the photo library at:\n{root}\nIt is safe to delete: Leoht rebuilds it.\n"
        );
        fs::write(&note, text).map_err(|source| LibraryError::Cache {
            path: note.clone(),
            source,
        })?;
    }
    Ok(LibraryPaths {
        catalog: cache_dir.join(CATALOG_FILE),
        thumbs: cache_dir.join("thumbs"),
        cache_dir,
        root,
    })
}

fn canonical(path: &Utf8Path) -> Result<Utf8PathBuf, LibraryError> {
    let cache_err = |source| LibraryError::Cache {
        path: path.to_owned(),
        source,
    };
    let canon = fs::canonicalize(path).map_err(cache_err)?;
    Utf8PathBuf::from_path_buf(canon).map_err(|p| {
        cache_err(std::io::Error::other(format!(
            "path is not valid UTF-8: {}",
            p.display()
        )))
    })
}
