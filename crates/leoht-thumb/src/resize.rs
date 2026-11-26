//! Decoding and resizing.

use std::fs;

use camino::Utf8Path;
use image::{DynamicImage, RgbImage, imageops::FilterType};

use crate::ThumbError;

/// Shrinks `img` so its longest edge is at most `max_edge`, preserving aspect
/// ratio. Images already small enough are returned as RGB without upscaling.
#[must_use]
pub fn resize_to_fit(img: &DynamicImage, max_edge: u32) -> RgbImage {
    if img.width().max(img.height()) <= max_edge {
        return img.to_rgb8();
    }
    // Lanczos3 is the sharpest of the standard filters; fine for thumbnails.
    img.resize(max_edge, max_edge, FilterType::Lanczos3)
        .to_rgb8()
}

/// Decodes the image at `path` and returns a thumbnail no larger than `max_edge`.
///
/// # Errors
/// Returns [`ThumbError::Decode`] if the file cannot be opened or decoded.
pub fn thumbnail(path: &Utf8Path, max_edge: u32) -> Result<RgbImage, ThumbError> {
    let img = image::open(path).map_err(|source| ThumbError::Decode {
        path: path.to_owned(),
        source,
    })?;
    Ok(resize_to_fit(&img, max_edge))
}

/// Renders a thumbnail of `source` and writes it as JPEG to `dest`
/// (typically from [`crate::cache_path`]), creating parent folders as needed.
///
/// # Errors
/// Decode, directory-creation or encode/write failure.
pub fn render_to_cache(
    source: &Utf8Path,
    dest: &Utf8Path,
    max_edge: u32,
) -> Result<(), ThumbError> {
    let thumb = thumbnail(source, max_edge)?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|source| ThumbError::CacheDir {
            path: parent.to_owned(),
            source,
        })?;
    }
    thumb.save(dest).map_err(|source| ThumbError::Write {
        path: dest.to_owned(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    fn gradient(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_fn(w, h, |x, y| {
            image::Rgb([
                u8::try_from(x % 256).unwrap_or(0),
                u8::try_from(y % 256).unwrap_or(0),
                128,
            ])
        }))
    }

    #[test]
    fn landscape_is_limited_by_width_and_keeps_aspect() {
        let t = resize_to_fit(&gradient(400, 200), 100);
        assert_eq!((t.width(), t.height()), (100, 50));
    }

    #[test]
    fn portrait_is_limited_by_height() {
        let t = resize_to_fit(&gradient(200, 400), 100);
        assert_eq!((t.width(), t.height()), (50, 100));
    }

    #[test]
    fn small_images_are_not_upscaled() {
        let t = resize_to_fit(&gradient(40, 30), 100);
        assert_eq!((t.width(), t.height()), (40, 30));
    }

    #[test]
    fn render_to_cache_writes_a_decodable_jpeg() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let src = root.join("src.png");
        gradient(300, 200).save(&src).unwrap();
        let dest = root.join("cache/deep/t.jpg");
        render_to_cache(&src, &dest, 64).unwrap();
        let back = image::open(&dest).unwrap();
        assert_eq!((back.width(), back.height()), (64, 43));
    }

    #[test]
    fn undecodable_file_is_a_decode_error() {
        let dir = tempfile::tempdir().unwrap();
        let p = Utf8PathBuf::from_path_buf(dir.path().join("bad.jpg")).unwrap();
        std::fs::write(&p, b"nope").unwrap();
        assert!(matches!(thumbnail(&p, 64), Err(ThumbError::Decode { .. })));
    }
}
