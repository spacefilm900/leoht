//! `leoht scan` and `leoht rebuild`.

use std::fmt::Write as _;

use anyhow::Result;
use camino::Utf8Path;
use leoht_library::{Progress, ScanSummary};
use leoht_scan::SkipReason;
use serde_json::{Value, json};

use crate::exit::{self, Exit};
use crate::output::{Output, millis};

pub fn run(out: &Output, root: &Utf8Path, cache: Option<&Utf8Path>, rebuild: bool) -> Result<Exit> {
    let mut lib = super::open(root, cache)?;
    let report = |p: Progress| out.progress(&format!("{} found, {} read", p.found, p.committed));
    let summary = if rebuild {
        lib.rebuild(report)?
    } else {
        lib.scan(report)?
    };
    out.progress_done();
    let command = if rebuild { "rebuild" } else { "scan" };
    let paths = lib.paths();
    let body = json!({
        "library": { "root": paths.root, "cache_dir": paths.cache_dir },
        "summary": summary_json(&summary),
    });
    out.result(command, body, || human(&summary));
    Ok(exit::OK)
}

fn reason(r: &SkipReason) -> Value {
    match r {
        SkipReason::NonUtf8Name => json!({ "kind": "non_utf8_name" }),
        SkipReason::Symlink => json!({ "kind": "symlink" }),
        SkipReason::Unreadable(detail) => json!({ "kind": "unreadable", "detail": detail }),
    }
}

pub fn summary_json(s: &ScanSummary) -> Value {
    let skipped: Vec<Value> = s
        .skipped
        .iter()
        .map(|k| json!({ "path": k.path, "reason": reason(&k.reason) }))
        .collect();
    json!({
        "found": s.found,
        "added": s.added,
        "updated": s.updated,
        "removed": s.removed,
        "unchanged": s.unchanged,
        "read": s.probed,
        "damaged": s.damaged,
        "skipped": skipped,
        "elapsed_ms": millis(s.elapsed),
    })
}

fn human(s: &ScanSummary) -> String {
    let mut text = format!(
        "{} photos: {} new, {} changed, {} removed, {} unchanged ({:.2} s)\n",
        s.found,
        s.added,
        s.updated,
        s.removed,
        s.unchanged,
        s.elapsed.as_secs_f64()
    );
    if s.damaged > 0 {
        let _ = writeln!(text, "{} damaged (see `leoht list --damaged`)", s.damaged);
    }
    for k in &s.skipped {
        let _ = writeln!(text, "skipped {}: {:?}", k.path, k.reason);
    }
    text
}
