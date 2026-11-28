//! Pixels and EXIF for generated library files.

use image::{ImageEncoder, RgbImage, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};

use crate::fixtures::{self, ExifSpec};
use crate::rng::SplitMix64;
use crate::{LibrarySpec, TestkitError};

/// Size of the embedded preview in EXIF-carrying JPEGs (a typical camera IFD1 thumbnail).
const PREVIEW: (u32, u32) = (160, 120);

/// A smooth gradient between seeded endpoints: cheap and deterministic
/// (it compresses unlike a real photo, which is fine for volume tests).
fn random_image(rng: &mut SplitMix64, width: u32, height: u32) -> RgbImage {
    // Six random bytes = a start and an end value for each of R, G, B.
    let ends = rng.next_u64().to_le_bytes();
    // Linear blend between `ends[2*ch]` and `ends[2*ch + 1]` at position `pos` of `len`.
    let blend = |channel: usize, pos: u32, len: u32| {
        let (from, to) = (ends[2 * channel], ends[2 * channel + 1]);
        let span = u64::from(from.abs_diff(to));
        let base = u64::from(from.min(to));
        let step = span * u64::from(pos) / u64::from(len.max(2) - 1);
        u8::try_from(base + step).unwrap_or(u8::MAX)
    };
    RgbImage::from_fn(width, height, |x, y| {
        image::Rgb([
            blend(0, x, width),
            blend(1, y, height),
            blend(2, u32::midpoint(x, y), u32::midpoint(width, height)),
        ])
    })
}

/// Camera-like EXIF for photo number `index`: a capture date that advances
/// with the index, one of three cameras, mostly upright with some portrait
/// orientations, and an embedded preview.
fn camera_exif(rng: &mut SplitMix64, index: usize, preview: Vec<u8>) -> ExifSpec {
    let minutes = u64::try_from(index).unwrap_or(0) * 7;
    let (day, hour, minute) = (minutes / 1440 % 28 + 1, minutes / 60 % 24, minutes % 60);
    let month = minutes / (1440 * 28) % 12 + 1;
    let year = 2000 + minutes / (1440 * 28 * 12) % 25;
    let roll = rng.next_u64();
    let (make, model) = [
        ("Canon", "EOS R6"),
        ("Apple", "iPhone 15"),
        ("FUJIFILM", "X-T5"),
    ][usize::try_from(roll % 3).unwrap_or(0)];
    let orientation = match roll / 3 % 10 {
        0 => 6,
        1 => 8,
        _ => 1,
    };
    ExifSpec {
        date_original: Some(format!(
            "{year:04}:{month:02}:{day:02} {hour:02}:{minute:02}:00"
        )),
        make: Some(make.into()),
        model: Some(model.into()),
        exposure_time: Some((1, 250)),
        f_number: Some((28, 10)),
        iso: Some(200),
        orientation: Some(orientation),
        thumbnail_jpeg: Some(preview),
        ..ExifSpec::default()
    }
}

/// A JPEG of the spec's size, with EXIF if the spec asks for it.
pub fn encode_jpeg(
    rng: &mut SplitMix64,
    spec: &LibrarySpec,
    index: usize,
) -> Result<Vec<u8>, TestkitError> {
    let img = random_image(rng, spec.width, spec.height);
    if spec.exif {
        let small = image::imageops::thumbnail(&img, PREVIEW.0, PREVIEW.1);
        let preview = fixtures::jpeg(&small, None, None)?;
        let exif = camera_exif(rng, index, preview);
        return fixtures::jpeg(&img, Some(&exif), None);
    }
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 80).write_image(
        img.as_raw(),
        spec.width,
        spec.height,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(out)
}

/// A PNG of the spec's size.
pub fn encode_png(rng: &mut SplitMix64, spec: &LibrarySpec) -> Result<Vec<u8>, TestkitError> {
    let img = random_image(rng, spec.width, spec.height);
    let mut out = Vec::new();
    PngEncoder::new(&mut out).write_image(
        img.as_raw(),
        spec.width,
        spec.height,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(out)
}
