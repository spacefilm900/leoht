//! `leoht verify`: read-only; exit 3 when the catalog and disk disagree.

use std::fmt::Write as _;

use anyhow::Result;
use camino::Utf8Path;
use leoht_core::RelPath;
use leoht_library::VerifyReport;
use serde_json::json;

use crate::exit::{self, Exit};
use crate::output::{Output, millis};

pub fn run(out: &Output, root: &Utf8Path, cache: Option<&Utf8Path>, hashes: bool) -> Result<Exit> {
    let lib = super::open(root, cache)?;
    super::require_scanned(&lib, root)?;
    let r = lib.verify(hashes)?;
    let damaged: Vec<_> = r
        .damaged
        .iter()
        .map(|(p, d)| json!({ "path": p.as_str(), "damage": d }))
        .collect();
    let body = json!({
        "in_sync": r.in_sync(),
        "on_disk": r.on_disk,
        "in_catalog": r.in_catalog,
        "missing": paths(&r.missing),
        "unindexed": paths(&r.unindexed),
        "changed": paths(&r.changed),
        "hash_mismatch": paths(&r.hash_mismatch),
        "hashes_checked": r.hashes_checked,
        "damaged": damaged,
        "elapsed_ms": millis(r.elapsed),
    });
    out.result("verify", body, || human(&r));
    Ok(if r.in_sync() { exit::OK } else { exit::DRIFT })
}

fn paths(list: &[RelPath]) -> Vec<&str> {
    list.iter().map(RelPath::as_str).collect()
}

fn human(r: &VerifyReport) -> String {
    let mut text = String::new();
    for (label, list) in [
        ("missing", &r.missing),
        ("not indexed", &r.unindexed),
        ("changed", &r.changed),
        ("content changed", &r.hash_mismatch),
    ] {
        for p in list {
            let _ = writeln!(text, "{label}: {p}");
        }
    }
    if r.in_sync() {
        let _ = writeln!(
            text,
            "in sync: {} photos on disk match the catalog ({} hashes checked, {} damaged)",
            r.on_disk,
            r.hashes_checked,
            r.damaged.len()
        );
    } else {
        text.push_str("out of sync: run `leoht scan` to update the catalog\n");
    }
    text
}
