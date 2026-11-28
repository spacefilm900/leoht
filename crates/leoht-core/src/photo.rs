//! A photo as the filesystem presents it.

use crate::{ContentHash, Damage, PhotoFormat, PhotoMeta, RelPath};

/// One image file in a library.
///
/// This is a *derived* view of a file on disk (path, size, mtime, metadata,
/// hash). It carries no user data: stars, tags and edits live in sidecars,
/// never here. Its folder is simply the parent of [`Photo::path`]; folders are
/// not a separate stored entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Photo {
    /// Library-relative path; the photo's primary identity.
    pub path: RelPath,
    /// Detected format.
    pub format: PhotoFormat,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last-modified time, whole seconds since the Unix epoch (rounded down).
    pub modified_unix_secs: i64,
    /// Sub-second part of the modification time, in nanoseconds (0 where the
    /// filesystem does not record it). Together with the size it lets a rescan
    /// tell an unchanged file from a changed one without reading it.
    pub modified_nanos: u32,
    /// Content hash, if it has been computed (it is optional because hashing is slower than scanning).
    pub hash: Option<ContentHash>,
    /// Metadata read from the file.
    pub meta: PhotoMeta,
    /// Set when the file looks like a photo but could not be read (FR-LIB-10).
    pub damage: Option<Damage>,
}

impl Photo {
    /// A photo with file-system facts only: no hash, default metadata, no damage.
    #[must_use]
    pub fn new(
        path: RelPath,
        format: PhotoFormat,
        size_bytes: u64,
        modified_unix_secs: i64,
    ) -> Self {
        Self {
            path,
            format,
            size_bytes,
            modified_unix_secs,
            modified_nanos: 0,
            hash: None,
            meta: PhotoMeta::default(),
            damage: None,
        }
    }

    /// The folder containing this photo (`None` = the library root itself).
    #[must_use]
    pub fn folder(&self) -> Option<RelPath> {
        self.path.parent()
    }

    /// True if a file with this size and modification time is unchanged since
    /// this record was made (the rescan shortcut; content is not re-read).
    #[must_use]
    pub fn same_file_state(&self, size_bytes: u64, secs: i64, nanos: u32) -> bool {
        self.size_bytes == size_bytes
            && self.modified_unix_secs == secs
            && self.modified_nanos == nanos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_is_the_parent_path() {
        let photo = Photo::new(
            RelPath::new("2020/trip/a.jpg").unwrap(),
            PhotoFormat::Jpeg,
            1,
            0,
        );
        assert_eq!(photo.folder().unwrap().as_str(), "2020/trip");
    }

    #[test]
    fn file_state_compares_size_and_both_time_parts() {
        let mut photo = Photo::new(RelPath::new("a.jpg").unwrap(), PhotoFormat::Jpeg, 10, 100);
        photo.modified_nanos = 5;
        assert!(photo.same_file_state(10, 100, 5));
        assert!(!photo.same_file_state(11, 100, 5));
        assert!(!photo.same_file_state(10, 101, 5));
        assert!(!photo.same_file_state(10, 100, 6));
    }
}
