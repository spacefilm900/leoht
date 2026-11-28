//! Byte-exact builders for small test images with chosen metadata.
//!
//! Every fixture is generated from code, so there is nothing to license and
//! nothing personal in it (`tests/fixtures/PROVENANCE.md`). EXIF blocks are
//! written with `kamadak-exif`'s TIFF writer; images with the `image` crate.

use std::io::Cursor;

use exif::experimental::Writer;
use exif::{Field, In, Rational, Tag, Value};
use image::{
    ExtendedColorType, ImageEncoder, RgbImage, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder,
};

use crate::TestkitError;

/// EXIF fields to write. `None` fields are left out.
#[derive(Debug, Clone, Default)]
pub struct ExifSpec {
    /// `DateTimeOriginal`, EXIF syntax `YYYY:MM:DD HH:MM:SS`.
    pub date_original: Option<String>,
    /// `OffsetTimeOriginal`, e.g. `+01:00`.
    pub offset_original: Option<String>,
    /// `DateTimeDigitized`.
    pub date_digitized: Option<String>,
    /// `Make`.
    pub make: Option<String>,
    /// `Model`.
    pub model: Option<String>,
    /// `LensModel`.
    pub lens: Option<String>,
    /// `ExposureTime` as (numerator, denominator).
    pub exposure_time: Option<(u32, u32)>,
    /// `FNumber` as (numerator, denominator).
    pub f_number: Option<(u32, u32)>,
    /// `PhotographicSensitivity`.
    pub iso: Option<u16>,
    /// `FocalLength` as (numerator, denominator).
    pub focal_length: Option<(u32, u32)>,
    /// `Orientation`, 1..=8 (or any value, to test rejection).
    pub orientation: Option<u16>,
    /// `ColorSpace` (1 = sRGB, 65535 = uncalibrated).
    pub color_space: Option<u16>,
    /// `PixelXDimension` and `PixelYDimension`.
    pub pixel_size: Option<(u32, u32)>,
    /// A JPEG thumbnail stored in IFD1 (the embedded preview).
    pub thumbnail_jpeg: Option<Vec<u8>>,
}

fn ascii(tag: Tag, text: &str) -> Field {
    Field {
        tag,
        ifd_num: In::PRIMARY,
        value: Value::Ascii(vec![text.as_bytes().to_vec()]),
    }
}

fn rational(tag: Tag, (num, denom): (u32, u32)) -> Field {
    Field {
        tag,
        ifd_num: In::PRIMARY,
        value: Value::Rational(vec![Rational { num, denom }]),
    }
}

fn short(tag: Tag, v: u16) -> Field {
    Field {
        tag,
        ifd_num: In::PRIMARY,
        value: Value::Short(vec![v]),
    }
}

fn long(tag: Tag, v: u32) -> Field {
    Field {
        tag,
        ifd_num: In::PRIMARY,
        value: Value::Long(vec![v]),
    }
}

impl ExifSpec {
    fn fields(&self) -> Vec<Field> {
        let mut f = Vec::new();
        let texts = [
            (Tag::DateTimeOriginal, &self.date_original),
            (Tag::OffsetTimeOriginal, &self.offset_original),
            (Tag::DateTimeDigitized, &self.date_digitized),
            (Tag::Make, &self.make),
            (Tag::Model, &self.model),
            (Tag::LensModel, &self.lens),
        ];
        for (tag, text) in texts {
            if let Some(t) = text {
                f.push(ascii(tag, t));
            }
        }
        let ratios = [
            (Tag::ExposureTime, self.exposure_time),
            (Tag::FNumber, self.f_number),
            (Tag::FocalLength, self.focal_length),
        ];
        for (tag, r) in ratios {
            if let Some(r) = r {
                f.push(rational(tag, r));
            }
        }
        let shorts = [
            (Tag::PhotographicSensitivity, self.iso),
            (Tag::Orientation, self.orientation),
            (Tag::ColorSpace, self.color_space),
        ];
        for (tag, v) in shorts {
            if let Some(v) = v {
                f.push(short(tag, v));
            }
        }
        if let Some((w, h)) = self.pixel_size {
            f.push(long(Tag::PixelXDimension, w));
            f.push(long(Tag::PixelYDimension, h));
        }
        f
    }
}

/// Encodes `spec` as a big-endian TIFF/EXIF block (what follows `Exif\0\0` in a JPEG).
///
/// # Errors
/// [`TestkitError::Exif`] if the writer rejects the fields (for example none at all).
pub fn exif_tiff(spec: &ExifSpec) -> Result<Vec<u8>, TestkitError> {
    exif_tiff_ordered(spec, false)
}

/// As [`exif_tiff`], little-endian (`II`) when `little_endian` is true.
///
/// # Errors
/// As [`exif_tiff`].
pub fn exif_tiff_ordered(spec: &ExifSpec, little_endian: bool) -> Result<Vec<u8>, TestkitError> {
    let fields = spec.fields();
    let mut writer = Writer::new();
    for field in &fields {
        writer.push_field(field);
    }
    // IFD1 needs at least one entry besides the JPEG pointer, or readers skip it.
    let thumb_marker = short(Tag::Compression, 6);
    let thumb_marker = Field {
        ifd_num: In::THUMBNAIL,
        ..thumb_marker
    };
    if let Some(jpeg) = &spec.thumbnail_jpeg {
        writer.push_field(&thumb_marker);
        writer.set_jpeg(jpeg, In::THUMBNAIL);
    }
    let mut out = Cursor::new(Vec::new());
    writer
        .write(&mut out, little_endian)
        .map_err(|e| TestkitError::Exif(e.to_string()))?;
    Ok(out.into_inner())
}

/// A `w` x `h` image in four solid quadrants: top-left red, top-right green,
/// bottom-left blue, bottom-right white. Any rotation or mirror is detectable.
#[must_use]
pub fn quadrants(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| match (x < w / 2, y < h / 2) {
        (true, true) => image::Rgb([255, 0, 0]),
        (false, true) => image::Rgb([0, 255, 0]),
        (true, false) => image::Rgb([0, 0, 255]),
        (false, false) => image::Rgb([255, 255, 255]),
    })
}

/// Encodes `img` as a JPEG with optional EXIF and ICC profile.
///
/// # Errors
/// Encoding or EXIF-writing failure.
pub fn jpeg(
    img: &RgbImage,
    exif: Option<&ExifSpec>,
    icc: Option<Vec<u8>>,
) -> Result<Vec<u8>, TestkitError> {
    let mut out = Vec::new();
    let mut enc = JpegEncoder::new_with_quality(&mut out, 90);
    if let Some(spec) = exif {
        enc.set_exif_metadata(exif_tiff(spec)?)
            .map_err(|e| TestkitError::Exif(e.to_string()))?;
    }
    if let Some(icc) = icc {
        enc.set_icc_profile(icc)
            .map_err(|e| TestkitError::Exif(e.to_string()))?;
    }
    enc.write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        ExtendedColorType::Rgb8,
    )?;
    Ok(out)
}

/// Encodes `img` as a PNG with optional EXIF (`eXIf` chunk) and ICC profile.
///
/// # Errors
/// Encoding or EXIF-writing failure.
pub fn png(
    img: &RgbImage,
    exif: Option<&ExifSpec>,
    icc: Option<Vec<u8>>,
) -> Result<Vec<u8>, TestkitError> {
    let mut out = Vec::new();
    let mut enc = PngEncoder::new(&mut out);
    if let Some(spec) = exif {
        enc.set_exif_metadata(exif_tiff(spec)?)
            .map_err(|e| TestkitError::Exif(e.to_string()))?;
    }
    if let Some(icc) = icc {
        enc.set_icc_profile(icc)
            .map_err(|e| TestkitError::Exif(e.to_string()))?;
    }
    enc.write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        ExtendedColorType::Rgb8,
    )?;
    Ok(out)
}

/// A minimal ICC v4 profile whose `desc` tag (`mluc`) says `description`.
/// It is not a usable colour profile, only enough to test description reading.
#[must_use]
pub fn icc_profile(description: &str) -> Vec<u8> {
    let text: Vec<u8> = description
        .encode_utf16()
        .flat_map(u16::to_be_bytes)
        .collect();
    let text_len = u32::try_from(text.len()).unwrap_or(0);
    let mut tag = b"mluc\0\0\0\0".to_vec();
    tag.extend_from_slice(&1u32.to_be_bytes());
    tag.extend_from_slice(&12u32.to_be_bytes());
    tag.extend_from_slice(b"enUS");
    tag.extend_from_slice(&text_len.to_be_bytes());
    tag.extend_from_slice(&28u32.to_be_bytes());
    tag.extend_from_slice(&text);
    let mut p = vec![0u8; 128];
    p[36..40].copy_from_slice(b"acsp"); // profile file signature
    p.extend_from_slice(&1u32.to_be_bytes());
    p.extend_from_slice(b"desc");
    p.extend_from_slice(&144u32.to_be_bytes());
    p.extend_from_slice(&u32::try_from(tag.len()).unwrap_or(0).to_be_bytes());
    p.extend_from_slice(&tag);
    let total = u32::try_from(p.len()).unwrap_or(0);
    p[..4].copy_from_slice(&total.to_be_bytes());
    p
}

/// A Fujifilm-RAF-shaped file: the 16-byte magic, a header whose bytes 84..92
/// point at `embedded_jpeg`, then that JPEG. No sensor data (detection and
/// metadata only).
#[must_use]
pub fn raf(embedded_jpeg: &[u8]) -> Vec<u8> {
    let mut out = b"FUJIFILMCCD-RAW 0201FF383501".to_vec();
    out.resize(100, 0);
    let len = u32::try_from(embedded_jpeg.len()).unwrap_or(0);
    out[84..88].copy_from_slice(&100u32.to_be_bytes());
    out[88..92].copy_from_slice(&len.to_be_bytes());
    out.extend_from_slice(embedded_jpeg);
    out
}

/// One ISOBMFF box: 32-bit size, four-character type, body.
fn iso_box(kind: [u8; 4], body: &[u8]) -> Vec<u8> {
    let size = u32::try_from(body.len() + 8).unwrap_or(u32::MAX);
    let mut out = size.to_be_bytes().to_vec();
    out.extend_from_slice(&kind);
    out.extend_from_slice(body);
    out
}

/// A HEIF file holding only an `Exif` item (ISO/IEC 23008-12 structure: `ftyp`,
/// then a `meta` box with `iinf`, `iloc` and `idat`). No image item, so it is
/// for metadata tests only; real HEIC pixel data is out of scope (SQ-1).
///
/// # Errors
/// EXIF-writing failure.
pub fn heic_with_exif(spec: &ExifSpec) -> Result<Vec<u8>, TestkitError> {
    let mut payload = 0u32.to_be_bytes().to_vec(); // offset to the TIFF header
    payload.extend_from_slice(&exif_tiff(spec)?);
    let len = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    // infe v2: item 1, no protection, type `Exif`, empty name.
    let infe = iso_box(*b"infe", b"\x02\0\0\0\0\x01\0\0Exif\0");
    let mut iinf = b"\0\0\0\0\0\x01".to_vec(); // v0, one entry
    iinf.extend_from_slice(&infe);
    // iloc v1: 4-byte offsets and lengths, item 1 stored in `idat` (method 1).
    let mut iloc = b"\x01\0\0\0\x44\x00\0\x01\0\x01\0\x01\0\0\0\x01".to_vec();
    iloc.extend_from_slice(&0u32.to_be_bytes());
    iloc.extend_from_slice(&len.to_be_bytes());
    let mut meta = vec![0u8; 4]; // full box v0
    meta.extend_from_slice(&iso_box(*b"iinf", &iinf));
    meta.extend_from_slice(&iso_box(*b"iloc", &iloc));
    meta.extend_from_slice(&iso_box(*b"idat", &payload));
    let mut out = iso_box(*b"ftyp", b"heic\0\0\0\0mif1heic");
    out.extend_from_slice(&iso_box(*b"meta", &meta));
    Ok(out)
}

/// The start of a HEIC file: an ISOBMFF `ftyp` box with brands `heic`/`mif1`,
/// then filler. Enough for detection; there is no image data or EXIF.
#[must_use]
pub fn heic_stub() -> Vec<u8> {
    let mut out = 24u32.to_be_bytes().to_vec();
    out.extend_from_slice(b"ftypheic\0\0\0\0mif1heic");
    out.resize(256, 0);
    out
}
