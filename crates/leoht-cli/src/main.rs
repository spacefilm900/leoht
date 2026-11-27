//! `leoht`: command-line driver. Phase 0 offers `scan` and `index`; the
//! Phase 1 demo grows from here before any GUI exists.

// A CLI exists to print; the workspace-wide print lints are for library crates.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use leoht_catalog::Catalog;

#[derive(Parser)]
#[command(name = "leoht", version, about = "Folder-first photo management")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List the photos found under a folder (read-only).
    Scan {
        /// Library root folder.
        root: Utf8PathBuf,
    },
    /// Scan a folder and (re)build its catalog database.
    Index {
        /// Library root folder.
        root: Utf8PathBuf,
        /// Catalog file to write.
        #[arg(long, default_value = "leoht-catalog.sqlite")]
        db: Utf8PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    match Cli::parse().command {
        Command::Scan { root } => {
            let report =
                leoht_scan::scan_library(&root).with_context(|| format!("scanning {root}"))?;
            for photo in &report.photos {
                println!(
                    "{}\t{:?}\t{} bytes",
                    photo.path, photo.format, photo.size_bytes
                );
            }
            eprintln!("{} photos, {} skipped", report.photos.len(), report.skipped);
        }
        Command::Index { root, db } => {
            let report =
                leoht_scan::scan_library(&root).with_context(|| format!("scanning {root}"))?;
            let mut catalog = Catalog::open(&db).context("opening catalog")?;
            // The catalog is a cache, so a full rebuild is safe, and `replace_photos`
            // does the swap in one transaction: a failure keeps the previous index.
            catalog.replace_photos(&report.photos)?;
            println!("indexed {} photos into {}", catalog.photo_count()?, db);
        }
    }
    Ok(())
}
