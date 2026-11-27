//! Repo automation, run as `cargo xtask <command>`.
//!
//! Commands:
//! * `gen-library`  – write a deterministic synthetic photo library (for volume tests).
//! * `bless`        – regenerate golden-image references after an intended change.
//! * `check-size`   – enforce the source-file size limits from ADR-0003.
//! * `ci`           – run the same required checks as CI, locally (needs `cargo-deny`).
//!
//! Every command works from the workspace root, whatever folder it is invoked from.

// Automation prints progress; the workspace print lints target library crates.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod size;

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use leoht_testkit::{LibrarySpec, generate_library};

#[derive(Parser)]
#[command(name = "xtask", about = "Leoht repo automation")]
struct Cli {
    #[command(subcommand)]
    command: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Generate a deterministic synthetic library.
    GenLibrary {
        /// Number of photos.
        #[arg(long, default_value_t = 1_000)]
        count: usize,
        /// Output folder (created if missing).
        #[arg(long, default_value = "target/volume-library")]
        out: Utf8PathBuf,
        /// Seed; the same seed always produces the same bytes.
        #[arg(long, default_value_t = LibrarySpec::default().seed)]
        seed: u64,
        /// Photos per leaf folder.
        #[arg(long, default_value_t = LibrarySpec::default().photos_per_folder)]
        per_folder: usize,
    },
    /// Re-record golden-image references (review the diff before committing!).
    Bless,
    /// Fail if any Rust file exceeds the hard size limit; warn above the soft limit.
    CheckSize,
    /// fmt, clippy, tests, docs, cargo-deny and size check: the required CI checks.
    Ci,
}

/// The workspace root. `cargo xtask` runs from the caller's folder (the alias does not
/// `cd`), so repo-relative work must anchor here: xtask lives one level below the root.
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map_or(manifest_dir.clone(), PathBuf::from)
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Task::GenLibrary {
            count,
            out,
            seed,
            per_folder,
        } => {
            let spec = LibrarySpec {
                count,
                seed,
                photos_per_folder: per_folder,
            };
            let report = generate_library(&out, &spec).context("generating library")?;
            println!(
                "wrote {} photos in {} folders to {out}",
                report.photos, report.folders
            );
            Ok(())
        }
        Task::Bless => run(
            "cargo",
            &["test", "-p", "leoht-it", "--test", "golden"],
            &[("LEOHT_BLESS", "1")],
        ),
        Task::CheckSize => size::check(&workspace_root()),
        Task::Ci => {
            run("cargo", &["fmt", "--all", "--check"], &[])?;
            run(
                "cargo",
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
                &[],
            )?;
            // Same flags as the CI `test` job (`--no-fail-fast` shows every failure at once).
            run(
                "cargo",
                &["test", "--workspace", "--all-features", "--no-fail-fast"],
                &[],
            )?;
            // Same as the CI `docs` job: broken intra-doc links fail the build.
            run(
                "cargo",
                &["doc", "--workspace", "--no-deps", "--all-features"],
                &[("RUSTDOCFLAGS", "-D warnings")],
            )?;
            // Same as the CI `cargo-deny` job (licences, advisories, bans, sources).
            if !Command::new("cargo")
                .args(["deny", "--version"])
                .output()
                .is_ok_and(|o| o.status.success())
            {
                bail!(
                    "cargo-deny is required by CI: install it with `cargo install cargo-deny --locked`"
                );
            }
            run("cargo", &["deny", "--all-features", "check"], &[])?;
            size::check(&workspace_root())
        }
    }
}

/// Runs a command, inheriting stdio, and fails if it exits non-zero.
fn run(program: &str, args: &[&str], env: &[(&str, &str)]) -> Result<()> {
    println!("$ {program} {}", args.join(" "));
    let status = Command::new(program)
        .current_dir(workspace_root())
        .args(args)
        .envs(env.iter().copied())
        .status()
        .with_context(|| format!("failed to start `{program}`"))?;
    if !status.success() {
        bail!("`{program} {}` failed with {status}", args.join(" "));
    }
    Ok(())
}
