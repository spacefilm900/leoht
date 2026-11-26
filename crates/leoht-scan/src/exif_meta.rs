//! Basic EXIF reading (feature `exif`).

use std::fs::File;
use std::io::BufReader;

use camino::Utf8Path;
use exif::{In, Reader, Tag};

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
    Some(field.display_value().to_string())
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
}
