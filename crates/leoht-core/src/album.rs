//! Albums: virtual collections that reference photos without moving them.
//!
//! Folders are real (they exist on disk); albums are virtual (they exist only as
//! a list of references). Persistence arrives in Phase 4; Phase 0 fixes the shape.

use serde::{Deserialize, Serialize};

use crate::{ContentHash, RelPath};

/// Stable identifier of an album (an opaque string, e.g. a UUID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AlbumId(pub String);

/// A reference to a photo from inside an album.
///
/// The path is the fast lookup; the optional hash lets a reader re-find the
/// photo if the file was renamed or moved since the album was saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumMember {
    /// Where the photo was when it was added.
    pub path: RelPath,
    /// Content hash used to re-locate the photo after a move.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<ContentHash>,
}

/// A named, ordered, virtual collection of photos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Album {
    /// Stable identity.
    pub id: AlbumId,
    /// Display name chosen by the user.
    pub name: String,
    /// Members in display order. A photo may be in many albums.
    pub members: Vec<AlbumMember>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn album_round_trips_through_json() {
        let album = Album {
            id: AlbumId("a1".into()),
            name: "Best of 2020".into(),
            members: vec![AlbumMember {
                path: RelPath::new("2020/a.jpg").unwrap(),
                hash: None,
            }],
        };
        let json = serde_json::to_string(&album).unwrap();
        assert_eq!(serde_json::from_str::<Album>(&json).unwrap(), album);
        assert!(!json.contains("hash"), "absent hash is omitted, not null");
    }
}
