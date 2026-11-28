//! `leoht thumbs`.

use std::fmt::Write as _;

use anyhow::Result;
use camino::Utf8Path;
use leoht_library::{ThumbOptions, ThumbSummary};
use serde_json::json;

use crate::exit::{self, Exit};
use crate::output::{Output, millis};

pub fn run(
    out: &Output,
    root: &Utf8Path,
    cache: Option<&Utf8Path>,
    size: u32,
    workers: Option<usize>,
) -> Result<Exit> {
    let mut lib = super::open(root, cache)?;
    super::require_scanned(&lib, root)?;
    let options = ThumbOptions {
        max_edge: size,
        workers,
    };
    let s = lib.make_thumbnails(options, |p| {
        out.progress(&format!("{}/{} thumbnails", p.done, p.total));
    })?;
    out.progress_done();
    let body = json!({
        "size": size,
        "thumbs_dir": lib.paths().thumbs,
        "summary": {
            "eligible": s.eligible,
            "generated": s.generated,
            "from_preview": s.from_preview,
            "cached": s.cached,
            "unsupported": s.unsupported,
            "no_preview": s.no_preview,
            "skipped_damaged": s.skipped_damaged,
            "newly_damaged": s.newly_damaged,
            "write_errors": s.write_errors,
            "elapsed_ms": millis(s.elapsed),
        },
    });
    out.result("thumbs", body, || human(&s));
    // Unwritable thumbnails are a cache problem worth failing on; damaged
    // photos are not (they are recorded and listed).
    Ok(if s.write_errors.is_empty() {
        exit::OK
    } else {
        exit::ERROR
    })
}

fn human(s: &ThumbSummary) -> String {
    let mut text = format!(
        "{} thumbnails made ({} from embedded previews), {} already cached ({:.2} s)\n",
        s.generated,
        s.from_preview,
        s.cached,
        s.elapsed.as_secs_f64()
    );
    if s.unsupported + s.no_preview > 0 {
        let _ = writeln!(
            text,
            "{} photos have no thumbnail yet (format not decoded, or RAW without a preview)",
            s.unsupported + s.no_preview
        );
    }
    if s.newly_damaged > 0 {
        let _ = writeln!(
            text,
            "{} photos would not decode and are now marked damaged",
            s.newly_damaged
        );
    }
    for e in &s.write_errors {
        let _ = writeln!(text, "error: {e}");
    }
    text
}
