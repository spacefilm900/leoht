//! Basic EXIF reading (feature `exif`).

use std::fs::File;
use std::io::BufReader;

use camino::Utf8Path;
use exif::{In, Reader, Tag, Value};

/// Returns the EXIF `DateTimeOriginal` string (e.g. `"2020:07:14 09:30:00"`), if present.
///
/// Returns `None` for files with no EXIF, unsupported files, or unreadable files:
/// missing metadata is normal for photos and must never abort a scan.
/// The raw EXIF string is returned unparsed; time-zone-aware parsing is Phase 1 work.
#[must_use]
pub fn read_capture_time(path: &Utf8Path) -> Option<String> {
    let file = File::open(path).ok()?;
    let exif = Reader::new()
        .read_from_container(&mut BufReader::new(file))
        .ok()?;
    let field = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)?;
    // Take the raw ASCII bytes. `display_value()` would reformat the date as
    // `2020-07-14 09:30:00`, which breaks the documented contract above.
    let Value::Ascii(parts) = &field.value else {
        return None;
    };
    let text = std::str::from_utf8(parts.first()?).ok()?;
    // EXIF strings may carry trailing NUL padding.
    Some(text.trim_end_matches('\0').to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_without_exif_yields_none() {
        let dir = tempfile::tempdir().unwrap();
        let p = camino::Utf8PathBuf::from_path_buf(dir.path().join("plain.jpg")).unwrap();
        std::fs::write(&p, b"not really a jpeg").unwrap();
        assert_eq!(read_capture_time(&p), None);
        assert_eq!(
            read_capture_time(Utf8Path::new("/definitely/not/here.jpg")),
            None
        );
    }

    /// A bare big-endian TIFF whose Exif sub-IFD holds `DateTimeOriginal`.
    fn tiff_with_capture_time(text: &str) -> Vec<u8> {
        let mut t = Vec::new();
        t.extend_from_slice(b"MM\0*\0\0\0\x08"); // header, IFD0 at offset 8
        t.extend_from_slice(&[
            0, 1, 0x87, 0x69, 0, 4, 0, 0, 0, 1, 0, 0, 0, 0x1A, 0, 0, 0, 0,
        ]); // -> Exif IFD
        let len = u8::try_from(text.len() + 1).unwrap();
        t.extend_from_slice(&[
            0, 1, 0x90, 0x03, 0, 2, 0, 0, 0, len, 0, 0, 0, 0x2C, 0, 0, 0, 0,
        ]);
        t.extend_from_slice(text.as_bytes());
        t.push(0); // NUL terminator, as EXIF writers emit
        t
    }

    #[test]
    fn capture_time_is_returned_raw_not_display_formatted() {
        let dir = tempfile::tempdir().unwrap();
        let p = camino::Utf8PathBuf::from_path_buf(dir.path().join("t.tif")).unwrap();
        std::fs::write(&p, tiff_with_capture_time("2020:07:14 09:30:00")).unwrap();
        assert_eq!(
            read_capture_time(&p).as_deref(),
            Some("2020:07:14 09:30:00")
        );
    }
}
