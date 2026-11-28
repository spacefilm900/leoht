//! Image headers: pixel dimensions and embedded colour profiles, read without
//! decoding any pixels.
//!
//! JPEG is walked marker by marker (ITU-T T.81 / JFIF) up to the first frame
//! header, collecting `ICC_PROFILE` APP2 chunks on the way; only those few
//! header segments are read, never the compressed image data. PNG uses the
//! `image` crate's decoder, which stops before the first image-data chunk.

use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

use image::ImageDecoder as _;

/// Largest ICC profile we keep (real profiles are a few KB to about 1 MB).
const MAX_ICC: usize = 4 * 1024 * 1024;
/// Most ICC chunks: the sequence number is one byte, so 255 is the format's limit.
const MAX_ICC_CHUNKS: usize = 255;
/// Most PNG chunks inspected before the image data (real files have a handful).
const MAX_PNG_CHUNKS: usize = 4096;

/// What a header told us.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HeaderInfo {
    /// Stored width and height in pixels.
    pub size: Option<(u32, u32)>,
    /// The embedded ICC profile, if any.
    pub icc: Option<Vec<u8>>,
    /// The file declares itself sRGB without a profile (PNG `sRGB` chunk).
    pub srgb: bool,
}

/// Reads a JPEG's frame size and ICC profile.
///
/// # Errors
/// A message when the marker structure is broken before a frame header is found
/// (the file is then recorded as damaged).
pub fn jpeg_header<R: Read + Seek>(reader: R) -> Result<HeaderInfo, String> {
    let mut r = BufReader::new(reader);
    r.seek(SeekFrom::Start(2)).map_err(|e| e.to_string())?; // past SOI
    let mut chunks: Vec<(u8, Vec<u8>)> = Vec::new();
    let mut icc_total = 0usize;
    loop {
        let marker = next_marker(&mut r)?;
        match marker {
            // Standalone markers (no length): TEM, RSTn, SOI.
            0x01 | 0xD0..=0xD8 => continue,
            0xD9 | 0xDA => return Err("no frame header before image data".into()),
            _ => {}
        }
        let len = usize::from(read_u16(&mut r)?);
        if len < 2 {
            return Err(format!("bad segment length {len}"));
        }
        let body_len = len - 2;
        if is_sof(marker) {
            // Precision, height, width, component count, then 3 bytes per component.
            let mut sof = vec![0u8; body_len];
            r.read_exact(&mut sof)
                .map_err(|_| "truncated frame header")?;
            let components = usize::from(*sof.get(5).ok_or("frame header too short")?);
            if components == 0 || body_len < 6 + 3 * components {
                return Err(format!(
                    "frame header of {body_len} bytes cannot hold {components} components"
                ));
            }
            let height = u32::from(u16::from_be_bytes([sof[1], sof[2]]));
            let width = u32::from(u16::from_be_bytes([sof[3], sof[4]]));
            let size = (width > 0 && height > 0).then_some((width, height));
            return Ok(HeaderInfo {
                size,
                icc: assemble_icc(chunks),
                srgb: false,
            });
        }
        if marker == 0xE2 && icc_total < MAX_ICC && chunks.len() < MAX_ICC_CHUNKS {
            let mut body = vec![0u8; body_len];
            r.read_exact(&mut body)
                .map_err(|_| "truncated APP2 segment")?;
            // Body: signature, sequence number (1-based), chunk count, profile bytes.
            if let Some(rest) = body.strip_prefix(b"ICC_PROFILE\0")
                && rest.len() >= 2
            {
                icc_total += rest.len() - 2;
                chunks.push((rest[0], rest[2..].to_vec()));
            }
        } else {
            let skip = i64::try_from(body_len).map_err(|e| e.to_string())?;
            r.seek_relative(skip).map_err(|e| e.to_string())?;
        }
    }
}

/// Reads the next marker code, skipping `0xFF` fill bytes.
fn next_marker<R: BufRead>(r: &mut R) -> Result<u8, String> {
    let mut byte = [0u8; 1];
    r.read_exact(&mut byte)
        .map_err(|_| "truncated before frame header")?;
    if byte[0] != 0xFF {
        return Err(format!("expected a marker, found byte {:#04x}", byte[0]));
    }
    loop {
        r.read_exact(&mut byte).map_err(|_| "truncated marker")?;
        if byte[0] != 0xFF {
            return Ok(byte[0]);
        }
    }
}

fn read_u16<R: Read>(r: &mut R) -> Result<u16, String> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b)
        .map_err(|_| "truncated segment length")?;
    Ok(u16::from_be_bytes(b))
}

/// Start-of-frame markers: SOF0..SOF15 except DHT (C4), JPG (C8) and DAC (CC).
fn is_sof(marker: u8) -> bool {
    matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC)
}

/// Joins ICC chunks in sequence order; `None` if there are none.
fn assemble_icc(mut chunks: Vec<(u8, Vec<u8>)>) -> Option<Vec<u8>> {
    if chunks.is_empty() {
        return None;
    }
    chunks.sort_by_key(|c| c.0);
    Some(chunks.into_iter().flat_map(|c| c.1).collect())
}

/// Reads a PNG's size and ICC profile.
///
/// # Errors
/// The decoder's message when the header chunks are malformed.
pub fn png_header<R: BufRead + Seek>(mut reader: R) -> Result<HeaderInfo, String> {
    let srgb = png_declares_srgb(&mut reader);
    reader.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut decoder = image::codecs::png::PngDecoder::new(reader).map_err(|e| e.to_string())?;
    let icc = decoder.icc_profile().ok().flatten();
    Ok(HeaderInfo {
        size: Some(decoder.dimensions()),
        icc,
        srgb,
    })
}

/// True if a PNG has an `sRGB` chunk before its image data. Walks chunk
/// headers only (length and type), skipping bodies; any oddity means "no".
fn png_declares_srgb<R: Read + Seek>(r: &mut R) -> bool {
    if r.seek(SeekFrom::Start(8)).is_err() {
        return false;
    }
    for _ in 0..MAX_PNG_CHUNKS {
        let mut head = [0u8; 8];
        if r.read_exact(&mut head).is_err() {
            return false;
        }
        let len = i64::from(u32::from_be_bytes([head[0], head[1], head[2], head[3]]));
        match &head[4..8] {
            b"sRGB" => return true,
            b"IDAT" | b"IEND" => return false,
            _ => {}
        }
        // Body plus the 4-byte CRC.
        if r.seek(SeekFrom::Current(len + 4)).is_err() {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use image::{ImageEncoder, RgbImage, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};

    use super::*;

    fn jpeg(w: u32, h: u32, icc: Option<Vec<u8>>) -> Vec<u8> {
        let img = RgbImage::new(w, h);
        let mut out = Vec::new();
        let mut enc = JpegEncoder::new(&mut out);
        if let Some(icc) = icc {
            enc.set_icc_profile(icc).unwrap();
        }
        enc.write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgb8)
            .unwrap();
        out
    }

    #[test]
    fn jpeg_size_and_icc_are_read_from_headers() {
        let info = jpeg_header(Cursor::new(jpeg(37, 21, Some(vec![7u8; 300])))).unwrap();
        assert_eq!(info.size, Some((37, 21)));
        assert_eq!(info.icc, Some(vec![7u8; 300]));
        let plain = jpeg_header(Cursor::new(jpeg(8, 8, None))).unwrap();
        assert_eq!(plain.icc, None);
    }

    #[test]
    fn truncated_or_garbled_jpeg_headers_are_errors() {
        let good = jpeg(16, 16, None);
        for cut in [3, 6, 20] {
            assert!(
                jpeg_header(Cursor::new(good[..cut].to_vec())).is_err(),
                "cut {cut}"
            );
        }
        let mut garbled = good.clone();
        garbled[2] = 0x00; // first marker byte destroyed
        assert!(jpeg_header(Cursor::new(garbled)).is_err());
    }

    #[test]
    fn png_size_and_icc_are_read() {
        let mut out = Vec::new();
        let mut enc = PngEncoder::new(&mut out);
        enc.set_icc_profile(vec![3u8; 200]).unwrap();
        enc.write_image(&[0u8; 5 * 4 * 3], 5, 4, image::ExtendedColorType::Rgb8)
            .unwrap();
        let info = png_header(Cursor::new(out)).unwrap();
        assert_eq!(info.size, Some((5, 4)));
        assert_eq!(info.icc, Some(vec![3u8; 200]));
        assert!(png_header(Cursor::new(b"\x89PNG\r\n\x1a\n".to_vec())).is_err());
    }

    /// PNG chunk CRC (ISO 3309 CRC-32), bitwise; test-only.
    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = !0u32;
        for &b in bytes {
            crc ^= u32::from(b);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    #[test]
    fn png_srgb_chunk_is_reported() {
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(&[0u8; 2 * 2 * 3], 2, 2, image::ExtendedColorType::Rgb8)
            .unwrap();
        assert!(!png_header(Cursor::new(png.clone())).unwrap().srgb);
        // Insert `sRGB` (rendering intent 0) right after IHDR (8 + 25 bytes).
        let mut chunk = 1u32.to_be_bytes().to_vec();
        chunk.extend_from_slice(b"sRGB\0");
        chunk.extend_from_slice(&crc32(&chunk[4..]).to_be_bytes());
        png.splice(33..33, chunk);
        let info = png_header(Cursor::new(png)).unwrap();
        assert!(info.srgb);
        assert_eq!(info.size, Some((2, 2)));
    }

    #[test]
    fn frame_headers_must_hold_their_components() {
        let good = jpeg(16, 16, None);
        let sof = good.windows(2).position(|w| w == [0xFF, 0xC0]).unwrap();
        // Claim 3 components but shorten the segment length to 8 (6 + 0 component bytes).
        let mut short = good.clone();
        short[sof + 2..sof + 4].copy_from_slice(&8u16.to_be_bytes());
        assert!(jpeg_header(Cursor::new(short)).is_err());
        // Zero components.
        let mut none = good;
        none[sof + 9] = 0;
        assert!(jpeg_header(Cursor::new(none)).is_err());
    }

    #[test]
    fn empty_icc_chunks_are_bounded() {
        let good = jpeg(8, 8, None);
        let mut evil = good[..2].to_vec();
        for _ in 0..1000 {
            evil.extend_from_slice(&[0xFF, 0xE2, 0x00, 0x10]);
            evil.extend_from_slice(b"ICC_PROFILE\0\x01\x01");
        }
        evil.extend_from_slice(&good[2..]);
        let info = jpeg_header(Cursor::new(evil)).unwrap();
        assert_eq!(
            info.icc,
            Some(Vec::new()),
            "chunks capped, header still read"
        );
        assert_eq!(info.size, Some((8, 8)));
    }
}
