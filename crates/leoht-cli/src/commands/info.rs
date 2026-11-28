//! `leoht info`: what a bug report or trial note needs to identify the build.

use std::fmt::Write as _;

use anyhow::Result;
use camino::Utf8Path;
use leoht_core::PhotoFormat;
use leoht_library::cache;
use serde_json::{Value, json};

use crate::exit::{self, Exit};
use crate::output::Output;

pub fn run(out: &Output, root: Option<&Utf8Path>, cache_dir: Option<&Utf8Path>) -> Result<Exit> {
    let formats: Vec<Value> = PhotoFormat::ALL
        .iter()
        .map(|f| json!({ "format": f, "support": f.support(), "decoded": decoded(*f) }))
        .collect();
    let mut body = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "cache_root": cache::cache_root(cache_dir),
        "network": "none: Leoht makes no network connections",
        "formats": formats,
    });
    if let Some(root) = root {
        let lib = super::open(root, cache_dir)?;
        let library = json!({
            "root": lib.paths().root,
            "cache_dir": lib.paths().cache_dir,
            "scanned": lib.is_scanned()?,
            "photos": lib.catalog().photo_count()?,
            "damaged": lib.catalog().damaged_count()?,
        });
        body["library"] = library;
    }
    let human = || {
        let mut text = format!(
            "leoht {} ({}-{})\ncache: {}\nnetwork: none\n",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            body["cache_root"].as_str().unwrap_or("(none)"),
        );
        if let Some(lib) = body.get("library") {
            let _ = writeln!(
                text,
                "library: {}\n  cache folder: {}\n  photos: {} ({} damaged)",
                lib["root"].as_str().unwrap_or_default(),
                lib["cache_dir"].as_str().unwrap_or_default(),
                lib["photos"],
                lib["damaged"],
            );
        }
        text
    };
    let text = human();
    out.result("info", body, || text);
    Ok(exit::OK)
}

/// Whether this build decodes pixels of `format` (thumbnails), as opposed to
/// detecting it. HEIC is never decoded in Phase 1 builds (ADR-0004, SQ-1);
/// RAW thumbnails come from embedded previews only.
fn decoded(format: PhotoFormat) -> &'static str {
    match format {
        PhotoFormat::Jpeg | PhotoFormat::Png => "yes",
        PhotoFormat::Raw => "embedded preview only",
        _ => "no",
    }
}
