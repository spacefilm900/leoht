//! One module per subcommand.

pub mod info;
pub mod list;
pub mod scan;
pub mod thumbs;
pub mod verify;

use anyhow::{Context, Result};
use camino::Utf8Path;
use leoht_library::Library;

/// Opens the library at `root` with an optional cache-root override.
pub fn open(root: &Utf8Path, cache: Option<&Utf8Path>) -> Result<Library> {
    Library::open(root, cache).with_context(|| format!("opening library {root}"))
}

/// Fails with a helpful message if `scan` has never run for this library.
pub fn require_scanned(lib: &Library, root: &Utf8Path) -> Result<()> {
    if !lib.is_scanned()? {
        anyhow::bail!("{root} has not been scanned yet; run `leoht scan {root}` first");
    }
    Ok(())
}
