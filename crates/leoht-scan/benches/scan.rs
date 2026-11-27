//! Scan and hash throughput on a small deterministic synthetic library.
//! Run with `cargo bench -p leoht-scan`. Larger sizes live in the volume job.

// Benchmarks are test-like code: failing loudly with `expect` is the right behaviour.
#![allow(clippy::expect_used)]

use std::hint::black_box;

use camino::Utf8PathBuf;
use criterion::{Criterion, criterion_group, criterion_main};
use leoht_testkit::{LibrarySpec, generate_library};

fn bench_scan(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf-8 temp path");
    generate_library(
        &root,
        &LibrarySpec {
            count: 500,
            ..LibrarySpec::default()
        },
    )
    .expect("generate library");

    c.bench_function("scan_library/500", |b| {
        b.iter(|| {
            leoht_scan::scan_library(black_box(&root))
                .expect("scan")
                .photos
                .len()
        });
    });
}

criterion_group!(benches, bench_scan);
criterion_main!(benches);
