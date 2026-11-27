//! Source-file size policy (ADR-0003): small, focused files.

use std::fs;
use std::path::Path;

use anyhow::{Result, bail};

/// Above this many lines a file gets a warning: consider splitting it.
const SOFT_LIMIT: usize = 300;
/// Above this many lines the check fails: split the file.
const HARD_LIMIT: usize = 500;

/// Checks every `.rs` file under the workspace source roots.
pub fn check(workspace_root: &Path) -> Result<()> {
    let mut over_hard = Vec::new();
    for root in ["crates", "xtask", "tests"] {
        visit(&workspace_root.join(root), &mut over_hard)?;
    }
    if over_hard.is_empty() {
        println!("file sizes OK (soft limit {SOFT_LIMIT}, hard limit {HARD_LIMIT} lines)");
        Ok(())
    } else {
        bail!(
            "files over the {HARD_LIMIT}-line hard limit: {}",
            over_hard.join(", ")
        )
    }
}

fn visit(dir: &Path, over_hard: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            visit(&path, over_hard)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            let lines = fs::read_to_string(&path)?.lines().count();
            if lines > HARD_LIMIT {
                over_hard.push(format!("{} ({lines})", path.display()));
            } else if lines > SOFT_LIMIT {
                eprintln!(
                    "warning: {} has {lines} lines (soft limit {SOFT_LIMIT})",
                    path.display()
                );
            }
        }
    }
    Ok(())
}
