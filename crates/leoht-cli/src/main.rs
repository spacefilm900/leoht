//! `leoht`: the scriptable core (FR-FND-09). Every command prints a human
//! summary by default and one JSON document with `--json`; exit codes are
//! documented in `docs/cli.md` and in [`exit`].

// A CLI exists to print; the workspace-wide print lints are for library crates.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod commands;
mod exit;
mod output;

use std::process::ExitCode;

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

use crate::output::Output;

#[derive(Parser)]
#[command(
    name = "leoht",
    version,
    about = "Folder-first photo management: scan, list, thumbnail and verify a photo folder.",
    after_help = "Leoht never changes, moves or deletes your photos, and never uses the network.\n\
                  Exit codes: 0 ok, 1 error, 2 usage error, 3 verify found drift."
)]
struct Cli {
    /// Print one JSON document on stdout instead of human-readable text.
    #[arg(long, global = true)]
    json: bool,
    /// Where Leoht keeps catalogs and thumbnails (default: `LEOHT_CACHE_DIR`, else
    /// the per-user cache folder). Never inside the library.
    #[arg(long, global = true, value_name = "DIR")]
    cache_dir: Option<Utf8PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan a folder into its catalog, reading only new and changed files.
    Scan {
        /// Library root folder.
        root: Utf8PathBuf,
    },
    /// List the photos in the catalog (run `scan` first).
    List {
        /// Library root folder.
        root: Utf8PathBuf,
        /// Only photos that could not be read, with the reason.
        #[arg(long)]
        damaged: bool,
        /// Only photos of this format (jpeg, heic, png, raw, tiff, webp).
        #[arg(long, value_name = "FORMAT")]
        format: Option<String>,
    },
    /// Make missing thumbnails (JPEG, PNG, and RAW embedded previews).
    Thumbs {
        /// Library root folder.
        root: Utf8PathBuf,
        /// Longest edge in pixels.
        #[arg(long, default_value_t = leoht_library::DEFAULT_THUMB_EDGE,
              value_parser = clap::value_parser!(u32).range(16..=4096))]
        size: u32,
        /// Worker threads (default: one per core, at most 8).
        #[arg(long, value_parser = clap::value_parser!(u16).range(1..=64))]
        workers: Option<u16>,
    },
    /// Compare the catalog with the disk without changing anything; exit 3 on drift.
    Verify {
        /// Library root folder.
        root: Utf8PathBuf,
        /// Also re-read every hashed photo and compare content hashes.
        #[arg(long)]
        hashes: bool,
    },
    /// Delete the catalog and scan from scratch (the normal recovery path).
    Rebuild {
        /// Library root folder.
        root: Utf8PathBuf,
    },
    /// Version, platform and cache locations (for bug reports and trial notes).
    Info {
        /// Optional library root: also show its cache folder and counts.
        root: Option<Utf8PathBuf>,
    },
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let out = Output::new(cli.json);
    let cache = cli.cache_dir.as_deref();
    let result = match &cli.command {
        Command::Scan { root } => commands::scan::run(&out, root, cache, false),
        Command::Rebuild { root } => commands::scan::run(&out, root, cache, true),
        Command::List {
            root,
            damaged,
            format,
        } => commands::list::run(&out, root, cache, *damaged, format.as_deref()),
        Command::Thumbs {
            root,
            size,
            workers,
        } => commands::thumbs::run(&out, root, cache, *size, workers.map(usize::from)),
        Command::Verify { root, hashes } => commands::verify::run(&out, root, cache, *hashes),
        Command::Info { root } => commands::info::run(&out, root.as_deref(), cache),
    };
    match result {
        Ok(code) => code.into(),
        Err(e) => {
            out.error(&e);
            exit::ERROR.into()
        }
    }
}
