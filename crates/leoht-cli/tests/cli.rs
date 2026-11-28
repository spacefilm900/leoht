//! CLI contract tests (FR-FND-09): the real `leoht` binary on the metadata
//! corpus. JSON output is pinned by insta snapshots in `tests/snapshots/`;
//! any change to the contract shows up as a snapshot diff to review.
//! Machine-specific values (temp paths, timings, version) are normalised.

// Helpers outside `#[test]` functions unwrap too; this file is all test code.
#![allow(clippy::unwrap_used)]

use std::process::Command;

use camino::Utf8PathBuf;
use leoht_testkit::corpus::write_metadata_corpus;
use serde_json::Value;

struct Env {
    _dirs: (tempfile::TempDir, tempfile::TempDir),
    root: Utf8PathBuf,
    cache: Utf8PathBuf,
}

impl Env {
    fn corpus() -> Self {
        let (lib, cache) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let root = Utf8PathBuf::from_path_buf(lib.path().canonicalize().unwrap()).unwrap();
        let cache_path = Utf8PathBuf::from_path_buf(cache.path().to_path_buf()).unwrap();
        write_metadata_corpus(&root).unwrap();
        Self {
            _dirs: (lib, cache),
            root,
            cache: cache_path,
        }
    }

    /// Runs `leoht <args> <root> --cache-dir <cache>`; returns exit code, stdout, stderr.
    fn run(&self, args: &[&str]) -> (i32, String, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_leoht"))
            .args(args)
            .arg(self.root.as_str())
            .args(["--cache-dir", self.cache.as_str()])
            .env_remove("LEOHT_CACHE_DIR")
            .env_remove("RUST_LOG")
            .output()
            .unwrap();
        let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
        (
            out.status.code().unwrap_or(-1),
            text(out.stdout),
            text(out.stderr),
        )
    }

    /// Runs with `--json`, asserts the exit code, returns the normalised document.
    fn json(&self, args: &[&str], code: i32) -> Value {
        let mut full = vec!["--json"];
        full.extend_from_slice(args);
        let (got, stdout, stderr) = self.run(&full);
        assert_eq!(
            got, code,
            "leoht {args:?}\nstdout: {stdout}\nstderr: {stderr}"
        );
        let mut doc: Value = serde_json::from_str(&stdout).unwrap();
        self.normalise(&mut doc);
        doc
    }

    fn normalise(&self, v: &mut Value) {
        match v {
            Value::String(s) => {
                // Longest first: the cache folder may sit beside the library.
                for (from, to) in [
                    (self.root.as_str(), "[ROOT]"),
                    (self.cache.as_str(), "[CACHE]"),
                ] {
                    *s = s.replace(from, to);
                }
                *s = s.replace('\\', "/");
            }
            Value::Array(items) => items.iter_mut().for_each(|i| self.normalise(i)),
            Value::Object(map) => {
                for (key, value) in map.iter_mut() {
                    if key == "elapsed_ms" {
                        *value = Value::from(0);
                    } else if key == "cache_dir" || key == "thumbs_dir" {
                        // Contains a per-path hash that depends on the temp folder.
                        *value = Value::from("[LIBRARY CACHE]");
                    } else {
                        self.normalise(value);
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn scan_then_rescan() {
    let env = Env::corpus();
    insta::assert_json_snapshot!("scan_first", env.json(&["scan"], 0));
    let again = env.json(&["scan"], 0);
    assert_eq!(again["summary"]["unchanged"], 20);
    assert_eq!(again["summary"]["read"], 0);
}

#[test]
fn list_json_and_filters() {
    let env = Env::corpus();
    env.json(&["scan"], 0);
    insta::assert_json_snapshot!("list_all", env.json(&["list"], 0));
    let raw = env.json(&["list", "--format", "RAW"], 0);
    assert_eq!(raw["count"], 3);
    let (code, text, _) = env.run(&["list", "--damaged"]);
    assert_eq!(code, 0);
    insta::assert_snapshot!("list_damaged_text", text);
}

#[test]
fn thumbs_then_cached() {
    let env = Env::corpus();
    env.json(&["scan"], 0);
    insta::assert_json_snapshot!("thumbs_first", env.json(&["thumbs", "--size", "64"], 0));
    let again = env.json(&["thumbs", "--size", "64"], 0);
    assert_eq!(again["summary"]["generated"], 0);
    assert_eq!(again["summary"]["cached"], 9);
}

#[test]
fn verify_in_sync_then_drift_exits_3() {
    let env = Env::corpus();
    env.json(&["scan"], 0);
    env.json(&["thumbs"], 0);
    let clean = env.json(&["verify", "--hashes"], 0);
    assert_eq!(clean["in_sync"], true);
    assert_eq!(clean["hashes_checked"], 10);
    std::fs::remove_file(env.root.join("png/plain.png")).unwrap();
    std::fs::write(env.root.join("new.jpg"), b"x").unwrap();
    insta::assert_json_snapshot!("verify_drift", env.json(&["verify"], 3));
    let (code, text, _) = env.run(&["verify"]);
    assert_eq!(code, 3);
    insta::assert_snapshot!("verify_drift_text", text);
}

#[test]
fn rebuild_matches_the_scan() {
    let env = Env::corpus();
    let scan = env.json(&["scan"], 0);
    let listed = env.json(&["list"], 0);
    let rebuilt = env.json(&["rebuild"], 0);
    assert_eq!(rebuilt["command"], "rebuild");
    assert_eq!(rebuilt["summary"]["added"], scan["summary"]["added"]);
    assert_eq!(env.json(&["list"], 0), listed);
}

#[test]
fn errors_are_json_with_exit_1_and_usage_errors_exit_2() {
    let env = Env::corpus();
    let not_scanned = env.json(&["list"], 1);
    let message = not_scanned["error"].as_str().unwrap();
    assert!(message.contains("has not been scanned yet"), "{message}");
    assert_eq!(
        env.json(&["list", "--format", "gif"], 1)["schema_version"],
        1
    );
    let (code, _, stderr) = env.run(&["thumbs", "--size", "3"]);
    assert_eq!(code, 2, "{stderr}");
}

#[test]
fn info_names_the_build_and_the_library() {
    let env = Env::corpus();
    env.json(&["scan"], 0);
    let mut info = env.json(&["info"], 0);
    for key in ["version", "os", "arch", "cache_root"] {
        assert!(info[key].is_string(), "{key}");
        info[key] = Value::from(format!("[{key}]"));
    }
    insta::assert_json_snapshot!("info", info);
}
