//! `leoht list`.

use std::fmt::Write as _;

use anyhow::{Result, bail};
use camino::Utf8Path;
use leoht_core::{ContentHash, Photo, PhotoFormat, utc_timestamp};
use serde_json::{Value, json};

use crate::exit::{self, Exit};
use crate::output::Output;

pub fn run(
    out: &Output,
    root: &Utf8Path,
    cache: Option<&Utf8Path>,
    damaged_only: bool,
    format: Option<&str>,
) -> Result<Exit> {
    let format = match format {
        None => None,
        Some(name) => match PhotoFormat::from_name(&name.to_ascii_lowercase()) {
            Some(f) => Some(f),
            None => bail!("unknown format {name:?}; use one of jpeg, heic, png, raw, tiff, webp"),
        },
    };
    let lib = super::open(root, cache)?;
    super::require_scanned(&lib, root)?;
    let photos: Vec<Photo> = lib
        .catalog()
        .photos()?
        .into_iter()
        .filter(|p| !damaged_only || p.damage.is_some())
        .filter(|p| format.is_none_or(|f| p.format == f))
        .collect();
    let body = json!({
        "count": photos.len(),
        "photos": photos.iter().map(photo_json).collect::<Vec<_>>(),
    });
    out.result("list", body, || human(&photos));
    Ok(exit::OK)
}

/// One photo as the list contract shows it.
pub fn photo_json(p: &Photo) -> Value {
    json!({
        "path": p.path.as_str(),
        "format": p.format,
        "support": p.format.support(),
        "size_bytes": p.size_bytes,
        "modified": utc_timestamp(p.modified_unix_secs),
        "hash": p.hash.as_ref().map(ContentHash::as_str),
        "damage": p.damage,
        "meta": p.meta,
    })
}

fn human(photos: &[Photo]) -> String {
    let mut text = String::new();
    for p in photos {
        let when = p
            .meta
            .capture_time
            .as_ref()
            .map_or_else(|| "-".to_owned(), |c| c.local.clone());
        let damage = p.damage.as_ref().map_or_else(String::new, |d| {
            format!("\tDAMAGED {}: {}", d.kind.name(), d.detail)
        });
        let _ = writeln!(
            text,
            "{}\t{}\t{}\t{}{}",
            p.path,
            p.format.name(),
            p.size_bytes,
            when,
            damage
        );
    }
    let _ = writeln!(text, "{} photos", photos.len());
    text
}
