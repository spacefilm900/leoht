//! Golden-image tests: render, then compare with a committed reference PNG.
//!
//! Comparison is *perceptual-ish*: each channel may differ by up to `TOLERANCE`,
//! because resampling floating-point maths can differ by ±1 between CPU
//! architectures (Apple Silicon vs x86). To intentionally update a reference:
//! `cargo xtask bless`, then review the image diff in the PR.

use std::path::PathBuf;

use image::{DynamicImage, RgbImage};
use leoht_thumb::resize_to_fit;

/// Maximum allowed per-channel difference (0–255).
const TOLERANCE: u8 = 2;

/// A deterministic input: horizontal/vertical gradient with a hard-edged square.
fn source_image() -> DynamicImage {
    DynamicImage::ImageRgb8(RgbImage::from_fn(256, 192, |x, y| {
        if (80..160).contains(&x) && (60..120).contains(&y) {
            image::Rgb([240, 20, 20])
        } else {
            image::Rgb([
                u8::try_from(x).unwrap_or(255),
                u8::try_from(y).unwrap_or(255),
                90,
            ])
        }
    }))
}

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("golden")
        .join(name)
}

/// Compares `actual` with the stored reference, or rewrites it when `LEOHT_BLESS` is set.
fn assert_matches_golden(name: &str, actual: &RgbImage) {
    let path = golden_path(name);
    if std::env::var_os("LEOHT_BLESS").is_some() {
        actual.save(&path).expect("write golden image");
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|e| panic!("missing golden {path:?} ({e}); run `cargo xtask bless`"))
        .to_rgb8();
    assert_eq!(
        actual.dimensions(),
        expected.dimensions(),
        "dimensions differ for {name}"
    );
    let worst = actual
        .pixels()
        .zip(expected.pixels())
        .flat_map(|(a, b)| a.0.iter().zip(b.0.iter()).map(|(x, y)| x.abs_diff(*y)))
        .max()
        .unwrap_or(0);
    assert!(
        worst <= TOLERANCE,
        "{name}: max channel difference {worst} exceeds {TOLERANCE}"
    );
}

#[test]
fn thumbnail_of_gradient_matches_golden() {
    let thumb = resize_to_fit(&source_image(), 64);
    assert_matches_golden("thumbnail_gradient_64.png", &thumb);
}
