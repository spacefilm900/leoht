//! Where Leoht keeps a library's catalog and thumbnails.
//!
//! Layout: `<cache root>/libraries/<folder name>-<16 hex digits>/`, holding
//! `catalog.sqlite` (plus SQLite's `-wal`/`-shm` files), `thumbs/` and a
//! `library-root.txt` naming the photo folder it belongs to. The hex digits are
//! a BLAKE3 hash of the library's canonical path, so two libraries never share
//! a cache and the folder name says which is which. Everything here is
//! disposable (FR-FND-02): deleting the folder costs only rescan time.
//!
//! The cache root is, in order: an explicit override (CLI `--cache-dir`), the
//! `LEOHT_CACHE_DIR` environment variable, or the platform's per-user cache
//! folder (`~/Library/Caches/Leoht` on macOS, `$XDG_CACHE_HOME/leoht` or
//! `~/.cache/leoht` elsewhere, `%LOCALAPPDATA%\Leoht` on Windows).

use camino::{Utf8Path, Utf8PathBuf};

/// Environment variable that overrides the cache root.
pub const CACHE_ENV: &str = "LEOHT_CACHE_DIR";

/// The platform default cache root, ignoring overrides; `None` without a home folder.
#[must_use]
pub fn platform_cache_root() -> Option<Utf8PathBuf> {
    let env = |name| {
        std::env::var(name)
            .ok()
            .filter(|v| !v.is_empty())
            .map(Utf8PathBuf::from)
    };
    if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Caches/Leoht"))
    } else if cfg!(windows) {
        env("LOCALAPPDATA").map(|d| d.join("Leoht"))
    } else {
        env("XDG_CACHE_HOME")
            .map(|d| d.join("leoht"))
            .or_else(|| env("HOME").map(|h| h.join(".cache/leoht")))
    }
}

/// The cache root to use: `explicit`, else `LEOHT_CACHE_DIR`, else the platform default.
#[must_use]
pub fn cache_root(explicit: Option<&Utf8Path>) -> Option<Utf8PathBuf> {
    explicit
        .map(Utf8Path::to_owned)
        .or_else(|| {
            std::env::var(CACHE_ENV)
                .ok()
                .filter(|v| !v.is_empty())
                .map(Utf8PathBuf::from)
        })
        .or_else(platform_cache_root)
}

/// The per-library cache folder for the (canonical) `library_root`.
#[must_use]
pub fn library_cache_dir(cache_root: &Utf8Path, library_root: &Utf8Path) -> Utf8PathBuf {
    let hash = blake3::hash(library_root.as_str().as_bytes());
    let hex = hash.to_hex();
    let name: String = library_root
        .file_name()
        .unwrap_or("library")
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(40)
        .collect();
    cache_root
        .join("libraries")
        .join(format!("{name}-{}", &hex.as_str()[..16]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_dir_is_named_after_the_library_and_unique_per_path() {
        let root = Utf8Path::new("/cache");
        let a = library_cache_dir(root, Utf8Path::new("/photos/Family Trip"));
        let b = library_cache_dir(root, Utf8Path::new("/other/Family Trip"));
        assert!(a.starts_with("/cache/libraries"));
        let name = a.file_name().unwrap();
        assert!(name.starts_with("Family_Trip-"), "{name}");
        assert_eq!(name.len(), "Family_Trip-".len() + 16);
        assert_ne!(a, b);
        assert_eq!(
            a,
            library_cache_dir(root, Utf8Path::new("/photos/Family Trip"))
        );
    }

    #[test]
    fn explicit_cache_root_wins() {
        assert_eq!(
            cache_root(Some(Utf8Path::new("/x"))),
            Some(Utf8PathBuf::from("/x"))
        );
    }
}
