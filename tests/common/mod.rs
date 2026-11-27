//! Helpers shared by the integration test files.

// Test helpers: panicking with a message is the right failure mode here.
#![allow(clippy::expect_used)]

use camino::Utf8PathBuf;

/// A temporary directory plus its UTF-8 path. Deleted when dropped.
pub struct TempLibrary {
    _dir: tempfile::TempDir,
    pub root: Utf8PathBuf,
}

impl TempLibrary {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("create temp dir");
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf-8 temp path");
        Self { _dir: dir, root }
    }
}
