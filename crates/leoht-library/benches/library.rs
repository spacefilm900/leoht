//! Micro-benchmarks behind the FR-FND-07 volume numbers. Run with
//! `cargo bench -p leoht-library` (the Volume workflow runs them with `--quick`).
//!
//! * `scan/1000`: cold-ish scan of a 1k volume library into a fresh catalog
//! * `listing/10000`: loading the full photo list from a 10k catalog (warm start)
//! * `thumbnail/640x480` and `thumbnail/12mp`: one thumbnail from a JPEG of each
//!   size (the 12 MP case is closer to a real camera photo than the volume library)

// Benchmarks are test-like code: failing loudly with `expect` is the right behaviour.
#![allow(clippy::expect_used)]

use std::hint::black_box;

use camino::{Utf8Path, Utf8PathBuf};
use criterion::{Criterion, criterion_group, criterion_main};
use leoht_core::Orientation;
use leoht_library::Library;
use leoht_testkit::{LibrarySpec, fixtures, generate_library};
use leoht_thumb::{Request, render};

fn temp() -> (tempfile::TempDir, Utf8PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf-8 temp path");
    (dir, path)
}

fn library(count: usize) -> (tempfile::TempDir, Utf8PathBuf) {
    let (dir, root) = temp();
    generate_library(&root, &LibrarySpec::volume(count)).expect("generate library");
    (dir, root)
}

fn bench_scan_and_listing(c: &mut Criterion) {
    let (_lib, root) = library(1_000);
    c.bench_function("scan/1000", |b| {
        b.iter_batched(
            temp,
            |(_cache_dir, cache)| {
                let mut lib = Library::open(&root, Some(&cache)).expect("open");
                black_box(lib.scan(|_| {}).expect("scan").found)
            },
            criterion::BatchSize::PerIteration,
        );
    });

    let (_lib10k, root10k) = library(10_000);
    let (_cache_dir, cache) = temp();
    Library::open(&root10k, Some(&cache))
        .expect("open")
        .scan(|_| {})
        .expect("scan");
    c.bench_function("listing/10000", |b| {
        b.iter(|| {
            let lib = Library::open(&root10k, Some(&cache)).expect("open");
            black_box(lib.catalog().photos().expect("photos").len())
        });
    });
}

fn thumbnail_of(c: &mut Criterion, name: &str, path: &Utf8Path) {
    c.bench_function(name, |b| {
        b.iter(|| {
            let request = Request {
                source: path,
                preview: None,
                orientation: Orientation::UPRIGHT,
                max_edge: 256,
                may_decode: true,
            };
            black_box(render(&request).expect("render").0.width())
        });
    });
}

fn bench_thumbnails(c: &mut Criterion) {
    let (_dir, root) = temp();
    for (name, (w, h)) in [("640x480", (640, 480)), ("12mp", (4000, 3000))] {
        let path = root.join(format!("{name}.jpg"));
        let bytes = fixtures::jpeg(&fixtures::quadrants(w, h), None, None).expect("encode");
        std::fs::write(&path, bytes).expect("write");
        thumbnail_of(c, &format!("thumbnail/{name}"), &path);
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_scan_and_listing, bench_thumbnails
}
criterion_main!(benches);
