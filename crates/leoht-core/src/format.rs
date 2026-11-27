//! Image formats Leoht recognises, and how well it supports each in Phase 1.
//!
//! The support levels mirror the Phase 1 format matrix in `LEOHT_PLAN.md` and
//! ADR-0004. Recognising a format here does not mean we can decode it.

use serde::{Deserialize, Serialize};

/// How far Leoht goes with a format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatSupport {
    /// Decode, metadata, thumbnails, sidecar edits.
    Full,
    /// Detection, basic metadata and a thumbnail where possible (embedded preview).
    Partial,
    /// Recognised and indexed, but not decoded.
    DetectOnly,
}

/// A recognised image file format, in Phase 1 priority order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhotoFormat {
    /// JPEG (`.jpg`, `.jpeg`).
    Jpeg,
    /// HEIC / HEIF (`.heic`, `.heif`).
    Heic,
    /// PNG.
    Png,
    /// Camera RAW (CR2, CR3, NEF, ARW, DNG, ORF, RW2, RAF, ...).
    Raw,
    /// TIFF.
    Tiff,
    /// WebP.
    Webp,
}

impl PhotoFormat {
    /// Classifies a file by its extension (case-insensitive, without the dot).
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "heic" | "heif" => Some(Self::Heic),
            "png" => Some(Self::Png),
            "cr2" | "cr3" | "nef" | "nrw" | "arw" | "srf" | "sr2" | "dng" | "orf" | "rw2"
            | "raf" | "pef" | "srw" | "3fr" | "erf" | "mef" | "mrw" | "x3f" => Some(Self::Raw),
            "tif" | "tiff" => Some(Self::Tiff),
            "webp" => Some(Self::Webp),
            _ => None,
        }
    }

    /// The Phase 1 support level for this format.
    #[must_use]
    pub fn support(self) -> FormatSupport {
        match self {
            Self::Jpeg | Self::Heic | Self::Png => FormatSupport::Full,
            Self::Raw => FormatSupport::Partial,
            Self::Tiff | Self::Webp => FormatSupport::DetectOnly,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_extensions_case_insensitively() {
        assert_eq!(PhotoFormat::from_extension("JPG"), Some(PhotoFormat::Jpeg));
        assert_eq!(PhotoFormat::from_extension("HeIc"), Some(PhotoFormat::Heic));
        assert_eq!(PhotoFormat::from_extension("cr3"), Some(PhotoFormat::Raw));
        assert_eq!(PhotoFormat::from_extension("txt"), None);
        assert_eq!(PhotoFormat::from_extension("json"), None);
    }

    #[test]
    fn support_levels_match_the_phase1_matrix() {
        assert_eq!(PhotoFormat::Jpeg.support(), FormatSupport::Full);
        assert_eq!(PhotoFormat::Heic.support(), FormatSupport::Full);
        assert_eq!(PhotoFormat::Png.support(), FormatSupport::Full);
        assert_eq!(PhotoFormat::Raw.support(), FormatSupport::Partial);
        assert_eq!(PhotoFormat::Tiff.support(), FormatSupport::DetectOnly);
    }
}
