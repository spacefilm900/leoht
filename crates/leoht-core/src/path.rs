//! Library-relative paths: the identity of a photo or folder.

use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A path *relative to the library root*, always UTF-8 and always written with
/// forward slashes, regardless of operating system.
///
/// Storing relative paths (never absolute ones) is what lets a library folder be
/// moved, synced or opened on another machine without rewriting anything.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RelPath(String);

impl RelPath {
    /// Validates and normalises `input` into a [`RelPath`].
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidPath`] if the path is empty, absolute, or
    /// contains `..` (which could escape the library root).
    pub fn new(input: impl AsRef<str>) -> Result<Self, CoreError> {
        let raw = input.as_ref();
        let reject = |reason| CoreError::InvalidPath {
            path: raw.to_owned(),
            reason,
        };

        let mut parts: Vec<&str> = Vec::new();
        for component in Utf8Path::new(raw).components() {
            match component {
                Utf8Component::Normal(name) => parts.push(name),
                Utf8Component::CurDir => {}
                Utf8Component::ParentDir => return Err(reject("`..` is not allowed")),
                Utf8Component::RootDir | Utf8Component::Prefix(_) => {
                    return Err(reject("must be relative to the library root"));
                }
            }
        }
        if parts.is_empty() {
            return Err(reject("must not be empty"));
        }
        Ok(Self(parts.join("/")))
    }

    /// The path as a forward-slash string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The containing folder, or `None` for a file directly in the library root.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        self.0
            .rsplit_once('/')
            .map(|(parent, _)| Self(parent.to_owned()))
    }

    /// The last path component (file or folder name).
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.0
            .rsplit_once('/')
            .map_or(self.0.as_str(), |(_, name)| name)
    }

    /// Resolves this path against an absolute library root.
    #[must_use]
    pub fn to_absolute(&self, root: &Utf8Path) -> Utf8PathBuf {
        root.join(&self.0)
    }
}

impl TryFrom<String> for RelPath {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<RelPath> for String {
    fn from(value: RelPath) -> Self {
        value.0
    }
}

impl std::fmt::Display for RelPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_separators_and_dots() {
        assert_eq!(
            RelPath::new("./2020//trip/a.jpg").unwrap().as_str(),
            "2020/trip/a.jpg"
        );
    }

    #[test]
    fn rejects_escape_absolute_and_empty() {
        assert!(RelPath::new("../x.jpg").is_err());
        assert!(RelPath::new("a/../../x.jpg").is_err());
        assert!(RelPath::new("/etc/passwd").is_err());
        assert!(RelPath::new("").is_err());
        assert!(RelPath::new(".").is_err());
    }

    #[test]
    fn parent_and_file_name() {
        let p = RelPath::new("2020/trip/a.jpg").unwrap();
        assert_eq!(p.file_name(), "a.jpg");
        assert_eq!(p.parent().unwrap().as_str(), "2020/trip");
        assert!(RelPath::new("a.jpg").unwrap().parent().is_none());
    }

    #[test]
    fn serde_validates_on_deserialise() {
        assert!(serde_json::from_str::<RelPath>("\"../bad\"").is_err());
        let ok: RelPath = serde_json::from_str("\"a/b.jpg\"").unwrap();
        assert_eq!(ok.as_str(), "a/b.jpg");
    }
}
