//! Choosing a thumbnail source (embedded preview or full decode) and writing
//! thumbnails safely.

use std::fs;
use std::io::Cursor;

use camino::Utf8Path;
use image::{DynamicImage, ImageReader, Limits, RgbImage, codecs::jpeg::JpegEncoder};
use leoht_core::Orientation;

use crate::{ThumbError, resize_to_fit, thumbnail};

/// Largest decoded image, in bytes, one decode may allocate. A 100 MP RGB
/// photo needs about 300 MB; anything past this is refused, not attempted,
/// so a hostile file cannot exhaust memory (the scanner already flags photos
/// over 400 MP as damaged before they get here).
pub const MAX_DECODE_BYTES: u64 = 768 * 1024 * 1024;

/// JPEG quality of written thumbnails.
const THUMB_QUALITY: u8 = 85;

/// The decoder limits every thumbnail decode uses.
#[must_use]
pub fn decode_limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    limits
}

/// Where a thumbnail's pixels came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbOrigin {
    /// The photo itself was decoded.
    Decoded,
    /// The JPEG preview embedded in the file was used (RAW files, and photos
    /// whose EXIF preview is already big enough).
    EmbeddedPreview,
}

/// What to make a thumbnail from.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    /// The photo file.
    pub source: &'a Utf8Path,
    /// Its embedded JPEG preview, if it has one.
    pub preview: Option<&'a [u8]>,
    /// The photo's EXIF orientation. Previews rarely carry their own, so it is
    /// applied to them from here; full decodes read it from the file.
    pub orientation: Orientation,
    /// Longest edge of the thumbnail, in pixels.
    pub max_edge: u32,
    /// Whether the photo itself may be decoded. False for formats Leoht cannot
    /// decode (RAW in Phase 1): then only the preview can be used, at any size.
    pub may_decode: bool,
}

/// Renders a thumbnail for `req`, preferring the embedded preview when it is
/// at least `max_edge` on its long side (or when decoding is not allowed).
///
/// # Errors
/// [`ThumbError::NoSource`] when decoding is not allowed and there is no usable
/// preview; [`ThumbError::Decode`] when the chosen source cannot be decoded.
pub fn render(req: &Request<'_>) -> Result<(RgbImage, ThumbOrigin), ThumbError> {
    if let Some(bytes) = req.preview
        && let Ok(preview) = decode_preview(bytes)
    {
        let (w, h) = (preview.width(), preview.height());
        if !req.may_decode || w.max(h) >= req.max_edge {
            let mut img = preview;
            if let Some(o) = image::metadata::Orientation::from_exif(req.orientation.value()) {
                img.apply_orientation(o);
            }
            return Ok((
                resize_to_fit(&img, req.max_edge),
                ThumbOrigin::EmbeddedPreview,
            ));
        }
    }
    if !req.may_decode {
        return Err(ThumbError::NoSource {
            path: req.source.to_owned(),
        });
    }
    Ok((thumbnail(req.source, req.max_edge)?, ThumbOrigin::Decoded))
}

fn decode_preview(bytes: &[u8]) -> Result<DynamicImage, image::ImageError> {
    let mut reader = ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Jpeg);
    reader.limits(decode_limits());
    reader.decode()
}

/// Writes `thumb` as a JPEG at `dest`, creating parent folders. The bytes go
/// to a temporary file beside `dest` first and are then renamed into place, so
/// a crash or a concurrent reader never sees a half-written thumbnail.
///
/// # Errors
/// The folder cannot be created, or encoding or writing fails.
pub fn write_atomically(thumb: &RgbImage, dest: &Utf8Path) -> Result<(), ThumbError> {
    let parent = dest.parent().unwrap_or(Utf8Path::new("."));
    fs::create_dir_all(parent).map_err(|source| ThumbError::CacheDir {
        path: parent.to_owned(),
        source,
    })?;
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, THUMB_QUALITY)
        .encode_image(thumb)
        .map_err(|source| ThumbError::Write {
            path: dest.to_owned(),
            source,
        })?;
    // Unique per thread and process, so parallel workers never share a temp file.
    let tmp = parent.join(format!(
        ".{}.{}-{:?}.tmp",
        dest.file_name().unwrap_or("thumb"),
        std::process::id(),
        std::thread::current().id()
    ));
    let io_err = |source: std::io::Error| ThumbError::Write {
        path: dest.to_owned(),
        source: image::ImageError::IoError(source),
    };
    fs::write(&tmp, &bytes).map_err(io_err)?;
    fs::rename(&tmp, dest).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        io_err(e)
    })
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
