//! Decoding and resizing.

use std::fs;

use camino::Utf8Path;
use image::{DynamicImage, ImageDecoder, ImageReader, RgbImage, imageops::FilterType};

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

/// Decodes `path` and applies its EXIF orientation, so a phone photo stored
/// sideways (orientation 6, say) is upright. The `image` crate does not do this
/// by itself: the caller must read the orientation from the decoder and apply it.
fn decode_upright(path: &Utf8Path) -> Result<DynamicImage, image::ImageError> {
    let mut decoder = ImageReader::open(path)?
        .with_guessed_format()?
        .into_decoder()?;
    // A missing or unreadable orientation tag means "as stored", not a failure.
    let orientation = decoder.orientation().ok();
    let mut img = DynamicImage::from_decoder(decoder)?;
    if let Some(orientation) = orientation {
        img.apply_orientation(orientation);
    }
    Ok(img)
}

/// Decodes the image at `path` and returns an upright thumbnail no larger than `max_edge`.
///
/// # Errors
/// Returns [`ThumbError::Decode`] if the file cannot be opened or decoded.
pub fn thumbnail(path: &Utf8Path, max_edge: u32) -> Result<RgbImage, ThumbError> {
    let img = decode_upright(path).map_err(|source| ThumbError::Decode {
        path: path.to_owned(),
        source,
    })?;
    Ok(resize_to_fit(&img, max_edge))
}

/// True when `a` and `b` are the same file: same canonical path, or (Unix) the
/// same device and inode, which also catches hard links. A destination that
/// does not exist yet cannot alias anything.
fn same_file(a: &Utf8Path, b: &Utf8Path) -> bool {
    let (Ok(ma), Ok(mb)) = (fs::metadata(a), fs::metadata(b)) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ma.dev() == mb.dev() && ma.ino() == mb.ino()
    }
    #[cfg(not(unix))]
    {
        let _ = (ma, mb);
        matches!((fs::canonicalize(a), fs::canonicalize(b)), (Ok(x), Ok(y)) if x == y)
    }
}

/// Renders a thumbnail of `source` and writes it as JPEG to `dest`
/// (typically from [`crate::cache_path`]), creating parent folders as needed.
///
/// # Errors
/// [`ThumbError::DestIsSource`] if `dest` is the source photo (checked before anything
/// is written), plus decode, directory-creation or encode/write failure.
pub fn render_to_cache(
    source: &Utf8Path,
    dest: &Utf8Path,
    max_edge: u32,
) -> Result<(), ThumbError> {
    // Originals are never overwritten, whatever the caller passes as `dest`.
    if same_file(source, dest) {
        return Err(ThumbError::DestIsSource {
            path: dest.to_owned(),
        });
    }
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

    /// Encodes a 40x20 two-colour JPEG (left half red, right half blue) carrying an
    /// EXIF orientation tag, built by hand so the fixture is deterministic.
    fn jpeg_with_orientation(orientation: u8) -> Vec<u8> {
        let img = RgbImage::from_fn(40, 20, |x, _| {
            if x < 20 {
                image::Rgb([255, 0, 0])
            } else {
                image::Rgb([0, 0, 255])
            }
        });
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 95)
            .encode_image(&img)
            .unwrap();
        // APP1 "Exif" segment: big-endian TIFF, one IFD entry (tag 0x0112 = Orientation).
        let mut exif: Vec<u8> = b"Exif\0\0MM\0*\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01".to_vec();
        exif.extend_from_slice(&[0, orientation, 0, 0, 0, 0, 0, 0]);
        let len = u16::try_from(exif.len() + 2).unwrap();
        let mut out = jpeg[..2].to_vec(); // SOI
        out.extend_from_slice(&[0xFF, 0xE1]);
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(&exif);
        out.extend_from_slice(&jpeg[2..]);
        out
    }

    #[test]
    fn exif_orientation_6_is_applied_before_resizing() {
        let dir = tempfile::tempdir().unwrap();
        let p = Utf8PathBuf::from_path_buf(dir.path().join("rot.jpg")).unwrap();
        std::fs::write(&p, jpeg_with_orientation(6)).unwrap();
        let t = thumbnail(&p, 100).unwrap();
        // Stored 40x20 landscape, rotated 90 degrees clockwise: 20x40 portrait,
        // with the red (left) half now on top.
        assert_eq!((t.width(), t.height()), (20, 40));
        assert!(t.get_pixel(10, 5)[0] > 200, "top should be red");
        assert!(t.get_pixel(10, 35)[2] > 200, "bottom should be blue");
    }

    #[test]
    fn orientation_1_is_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let p = Utf8PathBuf::from_path_buf(dir.path().join("upright.jpg")).unwrap();
        std::fs::write(&p, jpeg_with_orientation(1)).unwrap();
        let t = thumbnail(&p, 100).unwrap();
        assert_eq!((t.width(), t.height()), (40, 20));
        assert!(t.get_pixel(5, 10)[0] > 200, "left should be red");
    }

    #[test]
    fn destination_that_is_the_source_is_refused_and_original_survives() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let src = root.join("a.jpg");
        let original = jpeg_with_orientation(1);
        std::fs::write(&src, &original).unwrap();
        assert!(matches!(
            render_to_cache(&src, &src, 16),
            Err(ThumbError::DestIsSource { .. })
        ));
        // Also through a hard link (a filesystem alias of the same file); Unix only,
        // where we compare device+inode.
        #[cfg(unix)]
        {
            let link = root.join("alias.jpg");
            std::fs::hard_link(&src, &link).unwrap();
            assert!(matches!(
                render_to_cache(&src, &link, 16),
                Err(ThumbError::DestIsSource { .. })
            ));
        }
        assert_eq!(std::fs::read(&src).unwrap(), original);
    }
}
