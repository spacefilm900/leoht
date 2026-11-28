//! Locating and parsing the EXIF block of a file, with bounded reads.
//!
//! JPEG, PNG, HEIF and WebP containers are walked by `kamadak-exif`, which
//! reads only the segments it needs. TIFF-based files (TIFF and most RAW
//! formats) keep EXIF in their first IFDs, so only the first
//! [`MAX_TIFF_PREFIX`] bytes are read rather than the whole (possibly huge)
//! file. Fujifilm RAF stores EXIF inside an embedded JPEG whose offset is in
//! the RAF header.

use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};

use exif::{Exif, Reader};

use super::detect::is_tiff;

/// Bytes read from the start of a TIFF-based file when looking for EXIF.
pub const MAX_TIFF_PREFIX: u64 = 16 * 1024 * 1024;

/// Largest embedded RAF preview we read to find its EXIF.
const MAX_RAF_PREVIEW: u32 = 32 * 1024 * 1024;

/// Parses the file's EXIF, keeping whatever could be read when part of it is
/// malformed. `None` means "no EXIF", which is normal and never an error.
pub fn read_exif(file: &mut File, head: &[u8]) -> Option<Exif> {
    let mut reader = Reader::new();
    reader.continue_on_error(true);
    let result = if is_tiff(head) {
        read_tiff_prefix(file).and_then(|buf| reader.read_raw(buf).map_err(Some))
    } else if let Some(magic) = tiff_variant_magic(head) {
        // Olympus ORF and Panasonic RW2 are TIFF with a private magic number
        // that kamadak-exif rejects; the IFD layout after it is standard TIFF.
        read_tiff_prefix(file).and_then(|mut buf| {
            let Some(slot) = buf.get_mut(2..4) else {
                return Err(None);
            };
            slot.copy_from_slice(&magic);
            reader.read_raw(buf).map_err(Some)
        })
    } else if head.starts_with(b"FUJIFILMCCD-RAW") {
        read_raf_preview(file, head).and_then(|jpeg| {
            reader
                .read_from_container(&mut Cursor::new(jpeg))
                .map_err(Some)
        })
    } else {
        file.seek(SeekFrom::Start(0))
            .map_err(|_| None)
            .and_then(|_| {
                reader
                    .read_from_container(&mut BufReader::new(&mut *file))
                    .map_err(Some)
            })
    };
    match result {
        Ok(exif) => Some(exif),
        // Malformed but partly readable: keep the readable fields.
        Err(Some(e)) => e
            .distill_partial_result(|errors| {
                tracing::debug!(count = errors.len(), "partial EXIF");
            })
            .ok(),
        Err(None) => None,
    }
}

/// For TIFF-structured RAW files with a non-standard magic number (Olympus
/// ORF `IIRO`/`IIRS`/`MMOR`, Panasonic RW2 `IIU\0`), the standard TIFF magic
/// in the file's byte order, to substitute before parsing.
fn tiff_variant_magic(head: &[u8]) -> Option<[u8; 2]> {
    match head.get(..4)? {
        b"IIRO" | b"IIRS" | b"IIU\0" => Some([0x2A, 0x00]),
        b"MMOR" => Some([0x00, 0x2A]),
        _ => None,
    }
}

fn read_tiff_prefix(file: &mut File) -> Result<Vec<u8>, Option<exif::Error>> {
    file.seek(SeekFrom::Start(0)).map_err(|_| None)?;
    let mut buf = Vec::new();
    file.take(MAX_TIFF_PREFIX)
        .read_to_end(&mut buf)
        .map_err(|_| None)?;
    Ok(buf)
}

/// The embedded JPEG of a RAF file: big-endian offset and length at bytes 84 and 88.
fn read_raf_preview(file: &mut File, head: &[u8]) -> Result<Vec<u8>, Option<exif::Error>> {
    let be = |at: usize| {
        head.get(at..at + 4)
            .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    };
    let (Some(offset), Some(len)) = (be(84), be(88)) else {
        return Err(None);
    };
    if len == 0 || len > MAX_RAF_PREVIEW {
        return Err(None);
    }
    file.seek(SeekFrom::Start(u64::from(offset)))
        .map_err(|_| None)?;
    let mut buf = Vec::new();
    file.take(u64::from(len))
        .read_to_end(&mut buf)
        .map_err(|_| None)?;
    Ok(buf)
}

/// The embedded JPEG preview of a file, if it has one: the EXIF IFD1 thumbnail
/// (JPEG, PNG, HEIF and TIFF-based RAW files), or the large preview a Fujifilm
/// RAF carries in its header. Bounded reads, as for [`read_exif`].
pub fn embedded_preview(file: &mut File, head: &[u8]) -> Option<Vec<u8>> {
    if head.starts_with(b"FUJIFILMCCD-RAW") {
        return read_raf_preview(file, head).ok();
    }
    let exif = read_exif(file, head)?;
    let field = |tag| {
        exif.get_field(tag, exif::In::THUMBNAIL)
            .and_then(|f| f.value.get_uint(0))
            .map(|v| v as usize)
    };
    let offset = field(exif::Tag::JPEGInterchangeFormat)?;
    let len = field(exif::Tag::JPEGInterchangeFormatLength)?;
    let jpeg = exif.buf().get(offset..offset.checked_add(len)?)?;
    jpeg.starts_with(&[0xFF, 0xD8]).then(|| jpeg.to_vec())
}
