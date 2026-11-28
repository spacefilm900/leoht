//! Metadata read from a photo file (FR-LIB-06).
//!
//! These values are *derived*: they are re-read from the file on every scan and
//! cached in the catalog. They are never user-authored, so they never go into
//! sidecars. Values are kept exactly as the file states them (EXIF rationals
//! stay rationals), which keeps them comparable, hashable and snapshot-stable.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Orientation;

/// Where a photo's capture time came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    /// EXIF `DateTimeOriginal` (the moment the shutter fired).
    ExifOriginal,
    /// EXIF `DateTimeDigitized`, used when the original time is missing.
    ExifDigitized,
    /// No usable EXIF date: the file's modification time, in UTC. Marked so the
    /// UI can say "date from file" rather than pretending it is a capture date.
    FileModified,
}

/// When the photo was taken.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CaptureTime {
    /// ISO 8601 local date and time, `YYYY-MM-DDTHH:MM:SS` (EXIF dates carry no
    /// zone), or a UTC timestamp ending in `Z` for [`DateSource::FileModified`].
    pub local: String,
    /// The EXIF `OffsetTimeOriginal` (or digitized) zone offset such as `+01:00`, if recorded.
    pub offset: Option<String>,
    /// Where the value came from.
    pub source: DateSource,
}

/// An unsigned EXIF rational such as an exposure time of `1/125` s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ratio {
    /// Numerator.
    pub num: u32,
    /// Denominator (never zero: a zero denominator is dropped when reading).
    pub den: u32,
}

impl Ratio {
    /// The value as a float, for display and sorting.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        f64::from(self.num) / f64::from(self.den)
    }

    /// Parses the `num/den` form written by [`fmt::Display`].
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (num, den) = text.split_once('/')?;
        let (num, den) = (num.parse().ok()?, den.parse().ok()?);
        (den != 0).then_some(Self { num, den })
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

/// Exposure settings, each optional because cameras record different subsets.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Exposure {
    /// Exposure time in seconds (EXIF `ExposureTime`).
    pub exposure_time: Option<Ratio>,
    /// Aperture as an f-number (EXIF `FNumber`).
    pub f_number: Option<Ratio>,
    /// ISO sensitivity (EXIF `PhotographicSensitivity`, formerly `ISOSpeedRatings`).
    pub iso: Option<u32>,
    /// Lens focal length in millimetres (EXIF `FocalLength`).
    pub focal_length_mm: Option<Ratio>,
}

/// What the file says about its colour space.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ColorInfo {
    /// Nothing recorded; displays treat this as sRGB (FR-BRW-06).
    #[default]
    Untagged,
    /// Declared sRGB without an embedded profile: EXIF `ColorSpace` = 1, or a
    /// PNG `sRGB` chunk.
    Srgb,
    /// An embedded ICC profile, with its description if one could be read
    /// (for example `"Display P3"`).
    Icc {
        /// The profile's description text.
        description: Option<String>,
    },
}

/// Everything Phase 1 reads from a photo besides its file-system facts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhotoMeta {
    /// Capture time, with its source. Always present after a scan (it falls
    /// back to the file date); `None` only before metadata has been read.
    pub capture_time: Option<CaptureTime>,
    /// Camera maker (EXIF `Make`), trimmed.
    pub camera_make: Option<String>,
    /// Camera model (EXIF `Model`), trimmed.
    pub camera_model: Option<String>,
    /// Lens model (EXIF `LensModel`), trimmed.
    pub lens_model: Option<String>,
    /// Exposure settings.
    pub exposure: Exposure,
    /// Stored pixel width (before orientation is applied), if known.
    pub width: Option<u32>,
    /// Stored pixel height (before orientation is applied), if known.
    pub height: Option<u32>,
    /// EXIF orientation; upright when absent.
    pub orientation: Orientation,
    /// Colour space information.
    pub color: ColorInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_display_and_parse_round_trip() {
        let r = Ratio { num: 1, den: 125 };
        assert_eq!(r.to_string(), "1/125");
        assert_eq!(Ratio::parse("1/125"), Some(r));
        assert_eq!(Ratio::parse("1/0"), None);
        assert_eq!(Ratio::parse("x/2"), None);
        assert!((Ratio { num: 28, den: 10 }.as_f64() - 2.8).abs() < 1e-9);
    }

    #[test]
    fn color_info_serialises_with_a_kind_tag() {
        let icc = ColorInfo::Icc {
            description: Some("Display P3".into()),
        };
        assert_eq!(
            serde_json::to_string(&icc).unwrap(),
            r#"{"kind":"icc","description":"Display P3"}"#
        );
        assert_eq!(
            serde_json::to_string(&ColorInfo::Untagged).unwrap(),
            r#"{"kind":"untagged"}"#
        );
    }
}
