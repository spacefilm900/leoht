//! A photo as the filesystem presents it.

use crate::{ContentHash, PhotoFormat, RelPath};

/// One image file in a library.
///
/// This is a *derived* view of a file on disk (path, size, mtime, hash). It
/// carries no user data: stars, tags and edits live in sidecars, never here.
/// Its folder is simply the parent of [`Photo::path`] — folders are not a
/// separate stored entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Photo {
    /// Library-relative path; the photo's primary identity.
    pub path: RelPath,
    /// Detected format.
    pub format: PhotoFormat,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last-modified time, seconds since the Unix epoch.
    pub modified_unix_secs: i64,
    /// Content hash, if it has been computed (it is optional because hashing is slower than scanning).
    pub hash: Option<ContentHash>,
}

impl Photo {
    /// The folder containing this photo (`None` = the library root itself).
    #[must_use]
    pub fn folder(&self) -> Option<RelPath> {
        self.path.parent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_is_the_parent_path() {
        let photo = Photo {
            path: RelPath::new("2020/trip/a.jpg").unwrap(),
            format: PhotoFormat::Jpeg,
            size_bytes: 1,
            modified_unix_secs: 0,
            hash: None,
        };
        assert_eq!(photo.folder().unwrap().as_str(), "2020/trip");
    }
}
