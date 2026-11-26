//! Content-addressed thumbnail cache layout.

use camino::{Utf8Path, Utf8PathBuf};
use leoht_core::ContentHash;

/// Where the thumbnail for `hash` at `max_edge` pixels lives under `cache_root`:
/// `<root>/<max_edge>/<first 2 hex digits>/<hash>.jpg`.
///
/// Keying by *content* means renaming or moving a photo never invalidates its
/// thumbnail, and identical files share one. The two-digit fan-out keeps any one
/// directory small even with 100k+ photos. The cache is disposable.
#[must_use]
pub fn cache_path(cache_root: &Utf8Path, hash: &ContentHash, max_edge: u32) -> Utf8PathBuf {
    let hex = hash
        .as_str()
        .strip_prefix("blake3:")
        .unwrap_or(hash.as_str());
    cache_root
        .join(max_edge.to_string())
        .join(&hex[..2])
        .join(format!("{hex}.jpg"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_is_size_then_fanout_then_hash() {
        let hash = ContentHash::from_hex(&format!("ab{}", "0".repeat(62))).unwrap();
        let p = cache_path(Utf8Path::new("/cache"), &hash, 256);
        assert_eq!(
            p.as_str(),
            format!("/cache/256/ab/ab{}.jpg", "0".repeat(62))
        );
    }
}
