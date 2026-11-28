//! Turning parsed EXIF fields into [`PhotoMeta`] values.

use exif::{Exif, Field, In, Tag, Value};
use leoht_core::{CaptureTime, ColorInfo, DateSource, Orientation, PhotoMeta, Ratio};

/// Longest text value kept from a field (makers sometimes pad with garbage).
const MAX_TEXT: usize = 128;

/// Fills `meta` from `exif`. Missing or malformed fields are simply left empty.
pub fn apply_exif(exif: &Exif, meta: &mut PhotoMeta) {
    let field = |tag| exif.get_field(tag, In::PRIMARY);
    meta.capture_time = capture_time(exif);
    meta.camera_make = field(Tag::Make).and_then(text);
    meta.camera_model = field(Tag::Model).and_then(text);
    meta.lens_model = field(Tag::LensModel).and_then(text);
    meta.exposure.exposure_time = field(Tag::ExposureTime).and_then(ratio);
    meta.exposure.f_number = field(Tag::FNumber).and_then(ratio);
    meta.exposure.iso = field(Tag::PhotographicSensitivity).and_then(|f| f.value.get_uint(0));
    meta.exposure.focal_length_mm = field(Tag::FocalLength).and_then(ratio);
    meta.width = field(Tag::PixelXDimension).and_then(|f| f.value.get_uint(0));
    meta.height = field(Tag::PixelYDimension).and_then(|f| f.value.get_uint(0));
    if let Some(o) = field(Tag::Orientation)
        .and_then(|f| f.value.get_uint(0))
        .and_then(Orientation::from_exif)
    {
        meta.orientation = o;
    }
    if field(Tag::ColorSpace).and_then(|f| f.value.get_uint(0)) == Some(1) {
        meta.color = ColorInfo::Srgb;
    }
}

/// `DateTimeOriginal` (with `OffsetTimeOriginal`), else `DateTimeDigitized`.
fn capture_time(exif: &Exif) -> Option<CaptureTime> {
    let pairs = [
        (
            Tag::DateTimeOriginal,
            Tag::OffsetTimeOriginal,
            DateSource::ExifOriginal,
        ),
        (
            Tag::DateTimeDigitized,
            Tag::OffsetTimeDigitized,
            DateSource::ExifDigitized,
        ),
    ];
    pairs
        .into_iter()
        .find_map(|(date_tag, offset_tag, source)| {
            let local = exif
                .get_field(date_tag, In::PRIMARY)
                .and_then(text)
                .and_then(|t| iso_local(&t))?;
            let offset = exif
                .get_field(offset_tag, In::PRIMARY)
                .and_then(text)
                .filter(|o| valid_offset(o));
            Some(CaptureTime {
                local,
                offset,
                source,
            })
        })
}

/// First ASCII value, without NUL padding or surrounding spaces; `None` if empty.
fn text(field: &Field) -> Option<String> {
    let Value::Ascii(parts) = &field.value else {
        return None;
    };
    let raw = parts.first()?;
    let s = String::from_utf8_lossy(raw);
    let s = s.trim_matches(|c: char| c == '\0' || c.is_whitespace());
    (!s.is_empty()).then(|| s.chars().take(MAX_TEXT).collect())
}

fn ratio(field: &Field) -> Option<Ratio> {
    let Value::Rational(values) = &field.value else {
        return None;
    };
    let r = values.first()?;
    (r.denom != 0).then_some(Ratio {
        num: r.num,
        den: r.denom,
    })
}

/// EXIF `YYYY:MM:DD HH:MM:SS` to ISO 8601 `YYYY-MM-DDTHH:MM:SS`. Blank or
/// all-zero dates (written by cameras with no clock set) count as missing.
pub(crate) fn iso_local(exif_date: &str) -> Option<String> {
    let b = exif_date.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let digits = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18];
    let separators_ok =
        b[4] == b':' && b[7] == b':' && b[10] == b' ' && b[13] == b':' && b[16] == b':';
    if !separators_ok || !digits.iter().all(|&i| b[i].is_ascii_digit()) {
        return None;
    }
    let num = |r: std::ops::Range<usize>| exif_date[r].parse::<u32>().ok();
    let (month, day) = (num(5..7)?, num(8..10)?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let (hour, minute, second) = (num(11..13)?, num(14..16)?, num(17..19)?);
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    Some(format!(
        "{}-{}-{}T{}",
        &exif_date[0..4],
        &exif_date[5..7],
        &exif_date[8..10],
        &exif_date[11..19]
    ))
}

/// `+HH:MM` or `-HH:MM`, as EXIF 2.31 offset tags are written.
fn valid_offset(offset: &str) -> bool {
    let b = offset.as_bytes();
    b.len() == 6
        && matches!(b[0], b'+' | b'-')
        && b[3] == b':'
        && [1, 2, 4, 5].iter().all(|&i| b[i].is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exif_dates_become_iso_and_bad_ones_are_dropped() {
        assert_eq!(
            iso_local("2020:07:14 09:30:00").as_deref(),
            Some("2020-07-14T09:30:00")
        );
        assert_eq!(
            iso_local("2020:07:14 09:30:00\0\0").as_deref(),
            Some("2020-07-14T09:30:00")
        );
        for bad in [
            "0000:00:00 00:00:00",
            "    :  :     :  :  ",
            "2020-07-14 09:30:00",
            "2020:13:01 00:00:00",
            "2020:07:14 25:00:00",
            "2020:07:14",
            "",
        ] {
            assert_eq!(iso_local(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn offsets_must_be_signed_hours_and_minutes() {
        assert!(valid_offset("+01:00"));
        assert!(valid_offset("-07:30"));
        assert!(!valid_offset("01:00"));
        assert!(!valid_offset("+1:00"));
        assert!(!valid_offset("+01:00:00"));
    }
}
