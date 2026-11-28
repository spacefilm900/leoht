//! Volume tests: FR-FND-07 budgets at 1k and 10k photos (and FR-LIB-09, FR-FND-02).
//!
//! `#[ignore]`d so `cargo test --workspace` stays fast; `volume.yml` runs them:
//! `cargo test -p leoht-it --test volume --release -- --ignored --nocapture`.
//!
//! Each size gets a freshly generated library (`LibrarySpec::volume`: 640x480
//! JPEGs with EXIF and embedded previews, plus PNG, HEIC and RAW stand-ins)
//! and a fresh cache. Measured: cold scan, first 500 photos visible to a
//! second reader during that scan, warm start (open + full listing), rescan
//! with no change, rescan after changing 10 files (must read exactly 10),
//! rebuild, and thumbnails. Every number is printed and written out before
//! any budget is checked, so a failing run still reports everything.
//!
//! Environment:
//! * `LEOHT_VOLUME_COUNTS`: comma-separated sizes (default `1000,10000`).
//! * `LEOHT_PERF_PLATFORM`: baseline key in `perf/baselines.json` (e.g. `macos-15`).
//! * `LEOHT_PERF_OUT`: folder for `volume-<count>.json` and `volume.md`.
//! * `LEOHT_DROP_CACHES_CMD`: shell command run before the cold scan to empty the
//!   OS page cache (CI uses `sudo purge` / `drop_caches`); without it "cold"
//!   means a cold Leoht cache on a warm OS cache, and the report says so.

#![allow(clippy::print_stderr, clippy::unwrap_used)] // timings are the output; test code

mod common;
mod perf;

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use camino::Utf8Path;
use common::TempLibrary;
use leoht_catalog::Catalog;
use leoht_core::{DateSource, Photo, PhotoFormat};
use leoht_library::{Library, ThumbOptions};
use leoht_testkit::{LibrarySpec, generate_library};
use perf::{Measured, Metric, REPEATS, ms};
use serde_json::json;

fn drop_os_cache() -> bool {
    let Ok(cmd) = std::env::var("LEOHT_DROP_CACHES_CMD") else {
        return false;
    };
    let status = std::process::Command::new("sh")
        .args(["-c", &cmd])
        .status()
        .unwrap();
    assert!(status.success(), "LEOHT_DROP_CACHES_CMD failed: {cmd}");
    true
}

/// Polls a second catalog connection until it sees `want` photos; returns when it did.
fn time_until_visible(
    catalog: &Utf8Path,
    want: u64,
    start: Instant,
    done: &AtomicBool,
) -> Option<Duration> {
    let reader = Catalog::open(catalog).unwrap();
    loop {
        if reader.photo_count().unwrap_or(0) >= want {
            return Some(start.elapsed());
        }
        if done.load(Ordering::Relaxed) && reader.photo_count().unwrap_or(0) < want {
            return None;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn slowest(mut f: impl FnMut() -> Duration) -> f64 {
    (0..REPEATS).map(|_| ms(f())).fold(0.0, f64::max)
}

/// Cold scan of a fresh catalog, with a second reader watching for the first
/// 500 photos. Returns the scan time, when 500 became visible, and whether the
/// OS page cache was dropped first.
fn cold_scan(lib: &mut Library, count: usize) -> (Duration, Option<Duration>, bool) {
    let dropped = drop_os_cache();
    let done = AtomicBool::new(false);
    let catalog = lib.paths().catalog.clone();
    let want = u64::try_from(count.min(500)).unwrap();
    let start = Instant::now();
    let (summary, first) = std::thread::scope(|s| {
        let watcher = s.spawn(|| time_until_visible(&catalog, want, start, &done));
        let summary = lib.scan(|_| {}).unwrap();
        done.store(true, Ordering::Relaxed);
        (summary, watcher.join().unwrap())
    });
    let cold = start.elapsed();
    assert_eq!(summary.found, count, "every generated photo found");
    assert_eq!(summary.damaged, 0, "no generated photo is damaged");
    // The scan really read EXIF (a fast number from a scan that skipped it would be meaningless).
    let jpeg = jpegs(lib).into_iter().next().unwrap();
    let source = jpeg.meta.capture_time.map(|c| c.source);
    assert_eq!(source, Some(DateSource::ExifOriginal));
    assert!(jpeg.meta.camera_make.is_some() && jpeg.meta.width == Some(640));
    (cold, first, dropped)
}

fn jpegs(lib: &Library) -> Vec<Photo> {
    let photos = lib.catalog().photos().unwrap();
    photos
        .into_iter()
        .filter(|p| p.format == PhotoFormat::Jpeg)
        .collect()
}

/// Changes 10 JPEGs (bytes appended after the image: still valid, new size),
/// rescans, and checks exactly those 10 were read.
fn rescan_after_ten_changes(lib: &mut Library, root: &Utf8Path) -> Duration {
    let all = jpegs(lib);
    let changed: Vec<_> = all.iter().step_by(all.len() / 10).take(10).collect();
    assert_eq!(changed.len(), 10);
    for p in &changed {
        let path = root.join(p.path.as_str());
        let mut bytes = fs::read(&path).unwrap();
        bytes.extend_from_slice(b"changed");
        fs::write(&path, bytes).unwrap();
    }
    let ten = lib.scan(|_| {}).unwrap();
    let counted = (ten.probed, ten.updated);
    assert_eq!(
        counted,
        (10, 10),
        "a rescan after 10 changes reads exactly 10 files"
    );
    ten.elapsed
}

fn run_size(count: usize) -> (Vec<Metric>, serde_json::Value) {
    let lib_dir = TempLibrary::new();
    let cache_dir = tempfile::tempdir().unwrap();
    let cache = Utf8Path::from_path(cache_dir.path()).unwrap().to_owned();
    let t = Instant::now();
    generate_library(&lib_dir.root, &LibrarySpec::volume(count)).unwrap();
    eprintln!("generated {count} photos in {:?}", t.elapsed());

    let mut lib = Library::open(&lib_dir.root, Some(&cache)).unwrap();
    let (cold, first, dropped) = cold_scan(&mut lib, count);
    // Warm start: open the existing catalog and load the full listing.
    let warm = slowest(|| {
        let t = Instant::now();
        let reopened = Library::open(&lib_dir.root, Some(&cache)).unwrap();
        assert_eq!(reopened.catalog().photos().unwrap().len(), count);
        t.elapsed()
    });
    let rescan = slowest(|| {
        let s = lib.scan(|_| {}).unwrap();
        assert_eq!(s.probed, 0);
        s.elapsed
    });
    let ten = rescan_after_ten_changes(&mut lib, &lib_dir.root);
    let rebuild = lib.rebuild(|_| {}).unwrap().elapsed;
    let thumbs = lib
        .make_thumbnails(ThumbOptions::default(), |_| {})
        .unwrap();
    assert!(
        thumbs.write_errors.is_empty() && thumbs.newly_damaged == 0,
        "{thumbs:?}"
    );

    let measured = Measured {
        cold,
        first,
        warm,
        rescan,
        ten,
        rebuild,
        thumbs: thumbs.elapsed,
    };
    let env = json!({
        "os_page_cache_dropped_before_cold_scan": dropped,
        "library": "LibrarySpec::volume: 640x480 JPEG with EXIF + 160x120 preview (80%), PNG (10%), HEIC/RAW stand-ins (10%)",
        "thumbnails": { "generated": thumbs.generated, "from_preview": thumbs.from_preview, "unsupported": thumbs.unsupported, "no_preview": thumbs.no_preview },
        "repeats_for_warm_and_rescan": REPEATS,
    });
    (measured.metrics(count), env)
}

#[test]
#[ignore = "volume test: run via `cargo test -p leoht-it --test volume --release -- --ignored`"]
fn fr_fnd_07_budgets_at_1k_and_10k() {
    let counts: Vec<usize> = std::env::var("LEOHT_VOLUME_COUNTS")
        .unwrap_or_else(|_| "1000,10000".into())
        .split(',')
        .map(|c| c.trim().parse().unwrap())
        .collect();
    let baselines: serde_json::Value =
        serde_json::from_str(include_str!("../perf/baselines.json")).unwrap();
    let platform = std::env::var("LEOHT_PERF_PLATFORM").ok();
    let out = std::env::var("LEOHT_PERF_OUT").ok();
    let mut report = String::new();
    let mut failures = Vec::new();
    for count in counts {
        let (metrics, env) = run_size(count);
        let base = platform
            .as_deref()
            .and_then(|p| perf::baseline(&baselines, p, count));
        let title = format!(
            "{count} photos on {} (baseline: {})",
            platform.as_deref().unwrap_or("unknown platform"),
            if base.is_some() {
                "recorded"
            } else {
                "none yet"
            }
        );
        let table = perf::table(&title, &metrics);
        eprintln!("{table}\n{env:#}");
        report.push_str(&table);
        report.push('\n');
        if let Some(dir) = &out {
            fs::create_dir_all(dir).unwrap();
            let doc = perf::result_json(count, &env, &metrics);
            fs::write(format!("{dir}/volume-{count}.json"), format!("{doc:#}\n")).unwrap();
        }
        failures.extend(
            perf::check(&metrics, base.as_ref())
                .into_iter()
                .map(|f| format!("{count}: {f}")),
        );
    }
    if let Some(dir) = &out {
        fs::write(format!("{dir}/volume.md"), &report).unwrap();
    }
    assert!(
        failures.is_empty(),
        "FR-FND-07 check failed:\n{}",
        failures.join("\n")
    );
}
