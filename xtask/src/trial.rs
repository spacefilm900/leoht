//! `cargo xtask check-trial`: trial builds never decode HEIC (FR-LIB-04, SQ-1).
//!
//! Two checks on the build Keith receives:
//! 1. the dependency tree of the trial package (all targets, its default
//!    features plus any passed with `--features`) contains no crate, and
//!    enables no cargo feature, named in `xtask/heic-decoding.txt`;
//! 2. no sample folder shipped with the trial holds a HEIC/HEIF file, judged
//!    by extension *and* by content (the ISO-BMFF `ftyp` brand).

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

/// HEIF-family `ftyp` brands (ISO/IEC 23008-12), including image sequences.
const HEIF_BRANDS: [&[u8; 4]; 8] = [
    b"heic", b"heix", b"heim", b"heis", b"hevc", b"hevx", b"mif1", b"msf1",
];
/// HEIF-family extensions.
const HEIF_EXTENSIONS: [&str; 5] = ["heic", "heif", "hif", "heics", "avci"];

/// What the list file forbids.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Forbidden {
    pub crates: BTreeSet<String>,
    pub features: BTreeSet<String>,
}

/// Parses `heic-decoding.txt`.
pub fn parse_list(text: &str) -> Result<Forbidden> {
    let mut list = Forbidden::default();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.split_once(' ') {
            Some(("crate", name)) => list.crates.insert(name.trim().to_owned()),
            Some(("feature", name)) => list.features.insert(name.trim().to_owned()),
            _ => bail!(
                "heic-decoding.txt line {}: expected `crate <name>` or `feature <name>`",
                n + 1
            ),
        };
    }
    Ok(list)
}

/// Finds forbidden crates and features in `cargo tree -e features --prefix none` output.
/// Lines look like `image v0.25.10` or `image feature "png"`.
pub fn violations(tree: &str, list: &Forbidden) -> Vec<String> {
    let mut found = BTreeSet::new();
    for line in tree.lines() {
        let mut parts = line.split_whitespace();
        let Some(name) = parts.next() else { continue };
        if list.crates.contains(name) {
            found.insert(format!("crate `{name}` is in the trial build"));
        }
        if parts.next() == Some("feature")
            && let Some(feature) = parts.next().map(|f| f.trim_matches('"'))
            && list.features.contains(feature)
        {
            found.insert(format!("feature `{feature}` is enabled on `{name}`"));
        }
    }
    found.into_iter().collect()
}

/// True if `path` looks like a HEIF-family file by extension or content.
pub fn is_heif(path: &Path) -> bool {
    let by_ext = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| HEIF_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()));
    let by_content = fs::read(path).is_ok_and(|bytes| {
        bytes.get(4..8) == Some(b"ftyp")
            && bytes.get(8..bytes.len().min(64)).is_some_and(|brands| {
                brands
                    .chunks(4)
                    .any(|b| HEIF_BRANDS.iter().any(|h| *h == b))
            })
    });
    by_ext || by_content
}

fn heif_files(dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            heif_files(&path, out)?;
        } else if is_heif(&path) {
            out.push(path.display().to_string());
        }
    }
    Ok(())
}

/// Runs both checks.
pub fn check(root: &Path, package: &str, features: &[String], samples: &[String]) -> Result<()> {
    let list = parse_list(&fs::read_to_string(root.join("xtask/heic-decoding.txt"))?)?;
    let mut args = vec![
        "tree",
        "-p",
        package,
        "-e",
        "normal,build,features",
        "--target",
        "all",
        "--prefix",
        "none",
    ];
    let joined = features.join(",");
    if !features.is_empty() {
        args.extend(["--features", &joined]);
    }
    let out = Command::new("cargo")
        .current_dir(root)
        .args(&args)
        .output()
        .context("running cargo tree")?;
    if !out.status.success() {
        bail!(
            "cargo tree failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let mut problems = violations(&String::from_utf8_lossy(&out.stdout), &list);
    for dir in samples {
        let dir = root.join(dir);
        if dir.exists() {
            let mut files = Vec::new();
            heif_files(&dir, &mut files)?;
            problems.extend(
                files
                    .into_iter()
                    .map(|f| format!("HEIC/HEIF sample file {f}")),
            );
        }
    }
    if problems.is_empty() {
        println!("trial check: no HEIC decoding in `{package}`, no HEIC samples");
        return Ok(());
    }
    for p in &problems {
        eprintln!("trial check: {p}");
    }
    bail!("the trial build must not decode HEIC (FR-LIB-04, SQ-1)")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TREE: &str = "leoht-cli v0.0.0 (/x)\nimage v0.25.10\nimage feature \"png\"\n";

    #[test]
    fn list_file_in_the_repo_parses_and_names_the_adr_candidates() {
        let list = parse_list(include_str!("../heic-decoding.txt")).unwrap();
        for name in ["heifer", "heic-rs", "libheif-rs"] {
            assert!(list.crates.contains(name), "{name}");
        }
        assert!(list.features.contains("heic-system"));
    }

    #[test]
    fn clean_tree_passes_and_dirty_trees_fail() {
        let list = parse_list("crate heifer\nfeature heic").unwrap();
        assert_eq!(violations(TREE, &list), Vec::<String>::new());
        let dirty = format!("{TREE}heifer v0.7.3\nleoht-thumb feature \"heic\"\n");
        assert_eq!(violations(&dirty, &list).len(), 2);
    }

    #[test]
    fn bad_lines_are_rejected() {
        assert!(parse_list("heifer").is_err());
    }

    #[test]
    fn heif_files_are_found_by_extension_and_by_content() {
        let dir = tempfile::tempdir().unwrap();
        let disguised = dir.path().join("photo.jpg");
        fs::write(&disguised, b"\0\0\0\x18ftypmif1\0\0\0\0mif1heic").unwrap();
        let named = dir.path().join("IMG.HEIC");
        fs::write(&named, b"x").unwrap();
        let jpeg = dir.path().join("ok.jpg");
        fs::write(&jpeg, [0xFF, 0xD8, 0xFF]).unwrap();
        assert!(is_heif(&disguised) && is_heif(&named) && !is_heif(&jpeg));
    }
}
