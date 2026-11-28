//! Helpers shared by the integration test files.

// Test helpers: panicking with a message is the right failure mode here.
#![allow(clippy::expect_used, dead_code)]

use std::collections::BTreeMap;
use std::time::SystemTime;

use camino::{Utf8Path, Utf8PathBuf};

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

/// The state of one file for before/after comparisons: bytes and mtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFingerprint {
    pub blake3: String,
    pub len: u64,
    pub modified: SystemTime,
}

/// Every file and folder under `root` (relative path → fingerprint; folders map
/// to `None`), so a test can prove nothing was added, removed or changed.
pub fn tree_fingerprint(root: &Utf8Path) -> BTreeMap<String, Option<FileFingerprint>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_owned()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read dir") {
            let path = Utf8PathBuf::from_path_buf(entry.expect("entry").path()).expect("utf-8");
            let rel = path.strip_prefix(root).expect("under root").to_string();
            let meta = std::fs::symlink_metadata(&path).expect("stat");
            if meta.is_dir() {
                stack.push(path);
                out.insert(rel, None);
            } else {
                let bytes = std::fs::read(&path).expect("read");
                out.insert(
                    rel,
                    Some(FileFingerprint {
                        blake3: blake3_hex(&bytes),
                        len: meta.len(),
                        modified: meta.modified().expect("mtime"),
                    }),
                );
            }
        }
    }
    out
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Recursively sets `root` and everything in it read-only (or back to writable).
pub fn set_tree_readonly(root: &Utf8Path, readonly: bool) {
    let mut paths = vec![root.to_owned()];
    let mut i = 0;
    while i < paths.len() {
        let p = paths[i].clone();
        if p.is_dir() {
            for entry in std::fs::read_dir(&p).expect("read dir") {
                paths
                    .push(Utf8PathBuf::from_path_buf(entry.expect("entry").path()).expect("utf-8"));
            }
        }
        i += 1;
    }
    // Order does not matter: changing permissions needs ownership, not write access
    // to the parent folder, and listing a read-only folder still works.
    for p in &paths {
        let mut perms = std::fs::metadata(p).expect("stat").permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(readonly);
        std::fs::set_permissions(p, perms).expect("chmod");
    }
    // On Windows the read-only attribute does not stop changes to a folder's
    // contents, so deny the specific modify rights (write data / add file,
    // append data / add folder, write attributes and extended attributes,
    // delete, delete child) to Everyone with an inherited access-control entry,
    // and remove it again afterwards. Only specific rights (`DE`, not the
    // simple `D` or `W`): the simple rights include SYNCHRONIZE, and denying
    // that also blocks opening and listing the folder.
    #[cfg(windows)]
    {
        let args: &[&str] = if readonly {
            &["/deny", "*S-1-1-0:(OI)(CI)(WD,AD,WEA,WA,DE,DC)"]
        } else {
            &["/remove:d", "*S-1-1-0", "/T", "/C"]
        };
        let status = std::process::Command::new("icacls")
            .arg(root.as_str())
            .args(args)
            .arg("/Q")
            .status()
            .expect("run icacls");
        assert!(status.success(), "icacls {args:?} failed");
    }
}

/// True if creating a file directly in `dir` fails, i.e. the folder really is
/// write-protected. A probe file that does get created is removed again.
pub fn folder_rejects_writes(dir: &Utf8Path) -> bool {
    let probe = dir.join(".leoht-write-probe");
    match std::fs::File::create(&probe) {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe);
            false
        }
        Err(_) => true,
    }
}
