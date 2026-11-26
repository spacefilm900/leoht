//! Content hashing.

use std::fs::File;

use camino::Utf8Path;
use leoht_core::ContentHash;

use crate::ScanError;

/// Streams `path` through BLAKE3 and returns its [`ContentHash`].
///
/// BLAKE3 hashes at several GB/s, so hashing is I/O-bound on typical disks.
///
/// # Errors
/// Returns [`ScanError::Io`] if the file cannot be opened or read.
pub fn hash_file(path: &Utf8Path) -> Result<ContentHash, ScanError> {
    let io = |source| ScanError::Io {
        path: path.to_owned(),
        source,
    };
    let file = File::open(path).map_err(io)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(file).map_err(io)?;
    // A BLAKE3 digest is always 64 hex digits, so this cannot fail.
    ContentHash::from_hex(hasher.finalize().to_hex().as_str()).map_err(|e| ScanError::Io {
        path: path.to_owned(),
        source: std::io::Error::other(e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_content_sensitive() {
        let dir = tempfile::tempdir().unwrap();
        let p = |n: &str| camino::Utf8PathBuf::from_path_buf(dir.path().join(n)).unwrap();
        std::fs::write(p("a"), b"hello").unwrap();
        std::fs::write(p("b"), b"hello").unwrap();
        std::fs::write(p("c"), b"hellO").unwrap();
        assert_eq!(hash_file(&p("a")).unwrap(), hash_file(&p("b")).unwrap());
        assert_ne!(hash_file(&p("a")).unwrap(), hash_file(&p("c")).unwrap());
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(hash_file(Utf8Path::new("/definitely/not/here")).is_err());
    }
}
