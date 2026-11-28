//! Calendar conversion for file dates, without a date-time dependency.
//!
//! Only one direction is needed: Unix seconds to a proleptic-Gregorian UTC
//! timestamp, used when a photo has no EXIF capture date and Leoht falls back
//! to the file's modification time (FR-LIB-06). The day arithmetic is the
//! standard "days from civil" inverse (public-domain algorithm by H. Hinnant).

/// Formats `secs` since the Unix epoch as `YYYY-MM-DDTHH:MM:SSZ` (UTC).
#[must_use]
pub fn utc_timestamp(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60
    )
}

/// Year, month (1..=12) and day (1..=31) for a count of days since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097); // day of era, 0..=146096
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // year of era
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year, March-based
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_instants() {
        assert_eq!(utc_timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_timestamp(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(utc_timestamp(951_782_400), "2000-02-29T00:00:00Z"); // leap day
        assert_eq!(utc_timestamp(-1), "1969-12-31T23:59:59Z");
        assert_eq!(utc_timestamp(-86_400 * 365), "1969-01-01T00:00:00Z");
    }
}
