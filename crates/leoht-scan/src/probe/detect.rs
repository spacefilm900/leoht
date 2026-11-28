//! Content sniffing: does a file's first bytes match the format its extension claims?
//!
//! Signatures come from the public format specifications (JPEG/JFIF, PNG,
//! TIFF 6.0, ISO/IEC 14496-12 ISOBMFF and HEIF brands, RIFF/WebP) and the
//! widely documented headers of camera RAW containers.

use leoht_core::PhotoFormat;

/// How many leading bytes the probe reads before deciding (enough for every
/// signature below and for the RAF header fields at bytes 84..92).
pub const HEAD_LEN: usize = 128;

const JPEG_SOI: &[u8] = &[0xFF, 0xD8, 0xFF];
const PNG_SIG: &[u8] = b"\x89PNG\r\n\x1a\n";

/// HEIF brands that indicate an image file (HEIC, HEIF image and sequence brands).
const HEIF_BRANDS: &[&[u8; 4]] = &[
    b"heic", b"heix", b"hevc", b"hevx", b"heim", b"heis", b"hevm", b"hevs", b"mif1", b"msf1",
    b"mif2", b"avif",
];

/// True if `head` (the first bytes of the file) is plausible for `format`.
#[must_use]
pub fn matches_format(head: &[u8], format: PhotoFormat) -> bool {
    match format {
        PhotoFormat::Jpeg => head.starts_with(JPEG_SOI),
        PhotoFormat::Png => head.starts_with(PNG_SIG),
        PhotoFormat::Heic => has_ftyp_brand(head, HEIF_BRANDS),
        PhotoFormat::Tiff => is_tiff(head),
        PhotoFormat::Webp => head.len() >= 12 && &head[..4] == b"RIFF" && &head[8..12] == b"WEBP",
        PhotoFormat::Raw => is_raw(head),
    }
}

/// Classic TIFF (`II*\0` / `MM\0*`) or `BigTIFF` (`II+\0` / `MM\0+`) header.
#[must_use]
pub fn is_tiff(head: &[u8]) -> bool {
    matches!(
        head.get(..4),
        Some(b"II*\0" | b"MM\0*" | b"II+\0" | b"MM\0+")
    )
}

/// RAW containers: TIFF-based (CR2, NEF, ARW, DNG, PEF, SRW, ...), Olympus ORF,
/// Panasonic RW2, Canon CR3 (ISOBMFF brand `crx `), Fujifilm RAF, Sigma X3F
/// and Minolta MRW.
fn is_raw(head: &[u8]) -> bool {
    is_tiff(head)
        || matches!(
            head.get(..4),
            Some(b"IIRO" | b"IIRS" | b"MMOR" | b"IIU\0" | b"FOVb" | b"\0MRM")
        )
        || head.starts_with(b"FUJIFILMCCD-RAW")
        || has_ftyp_brand(head, &[b"crx "])
}

/// True if `head` starts with an ISOBMFF `ftyp` box whose major or one of whose
/// compatible brands is in `brands`.
fn has_ftyp_brand(head: &[u8], brands: &[&[u8; 4]]) -> bool {
    if head.len() < 12 || &head[4..8] != b"ftyp" {
        return false;
    }
    let box_len = u32::from_be_bytes([head[0], head[1], head[2], head[3]]) as usize;
    // Major brand at 8..12, minor version at 12..16, compatible brands after.
    let end = box_len.clamp(12, head.len());
    let major = std::iter::once(8);
    let compatible = (16..end).step_by(4);
    major
        .chain(compatible)
        .filter_map(|at| head.get(at..at + 4))
        .any(|brand| brands.iter().any(|b| b.as_slice() == brand))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ftyp(major: [u8; 4], compatible: &[&[u8; 4]]) -> Vec<u8> {
        let len = u32::try_from(16 + 4 * compatible.len()).unwrap();
        let mut b = len.to_be_bytes().to_vec();
        b.extend_from_slice(b"ftyp");
        b.extend_from_slice(&major);
        b.extend_from_slice(&[0, 0, 0, 0]);
        for c in compatible {
            b.extend_from_slice(*c);
        }
        b
    }

    #[test]
    fn recognises_each_signature() {
        assert!(matches_format(&[0xFF, 0xD8, 0xFF, 0xE0], PhotoFormat::Jpeg));
        assert!(matches_format(b"\x89PNG\r\n\x1a\n....", PhotoFormat::Png));
        assert!(matches_format(
            &ftyp(*b"heic", &[b"mif1"]),
            PhotoFormat::Heic
        ));
        assert!(matches_format(
            &ftyp(*b"mif1", &[b"heic"]),
            PhotoFormat::Heic
        ));
        assert!(matches_format(b"II*\0\x08\0\0\0", PhotoFormat::Tiff));
        assert!(matches_format(b"MM\0*\0\0\0\x08", PhotoFormat::Raw)); // e.g. NEF, CR2
        assert!(matches_format(b"IIRO\x08\0\0\0", PhotoFormat::Raw)); // ORF
        assert!(matches_format(b"FUJIFILMCCD-RAW 0201", PhotoFormat::Raw));
        assert!(matches_format(
            &ftyp(*b"crx ", &[b"isom"]),
            PhotoFormat::Raw
        )); // CR3
        assert!(matches_format(b"RIFF\0\0\0\0WEBPVP8 ", PhotoFormat::Webp));
    }

    #[test]
    fn rejects_mismatches() {
        assert!(!matches_format(b"hello world", PhotoFormat::Jpeg));
        assert!(!matches_format(&[0xFF, 0xD8], PhotoFormat::Jpeg)); // truncated
        assert!(!matches_format(&[0xFF, 0xD8, 0xFF], PhotoFormat::Png));
        assert!(!matches_format(
            &ftyp(*b"isom", &[b"mp41"]),
            PhotoFormat::Heic
        )); // an MP4
        assert!(!matches_format(&ftyp(*b"crx ", &[]), PhotoFormat::Heic));
        assert!(!matches_format(b"RIFF\0\0\0\0WAVEfmt ", PhotoFormat::Webp));
        assert!(!matches_format(b"", PhotoFormat::Raw));
    }
}
