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
    /// Parses the *stored* syntax: components separated by `/` (the same on every OS).
    /// Empty and `.` components are dropped, so `./2020//a.jpg` becomes `2020/a.jpg`.
    ///
    /// A backslash is **rejected**, not converted: `trip\a.jpg` means a folder on Windows
    /// but a single odd file name on Unix, so accepting it would give one string two
    /// identities. To turn a native OS path into a `RelPath`, use [`RelPath::from_native`].
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidPath`] if the path is empty, absolute (leading `/` or a
    /// drive prefix like `C:`), contains `..` (which could escape the library root) or
    /// contains `\`.
    pub fn new(input: impl AsRef<str>) -> Result<Self, CoreError> {
        let raw = input.as_ref();
        let reject = |reason| CoreError::InvalidPath {
            path: raw.to_owned(),
            reason,
        };

        if raw.starts_with('/') {
            return Err(reject("must be relative to the library root"));
        }
        let mut parts: Vec<&str> = Vec::new();
        for name in raw.split('/') {
            match name {
                "" | "." => {}
                ".." => return Err(reject("`..` is not allowed")),
                _ if name.contains('\\') => {
                    return Err(reject("backslash is ambiguous across platforms"));
                }
                _ if is_drive_prefix(name) => {
                    return Err(reject("must be relative to the library root"));
                }
                _ => parts.push(name),
            }
        }
        if parts.is_empty() {
            return Err(reject("must not be empty"));
        }
        Ok(Self(parts.join("/")))
    }

    /// Converts a native, already root-relative path (as produced by `strip_prefix` on
    /// this OS) to the stored syntax by joining its components with `/`.
    ///
    /// # Errors
    /// Same as [`RelPath::new`]; additionally a file name that itself contains a
    /// backslash on Unix is rejected as ambiguous rather than renamed.
    pub fn from_native(path: &Utf8Path) -> Result<Self, CoreError> {
        let mut parts: Vec<&str> = Vec::new();
        for component in path.components() {
            match component {
                Utf8Component::Normal(name) => parts.push(name),
                Utf8Component::CurDir => {}
                // Let `new` produce the proper error for these.
                Utf8Component::ParentDir => parts.push(".."),
                Utf8Component::RootDir | Utf8Component::Prefix(_) => parts.push("/"),
            }
        }
        if parts.contains(&"/") {
            return Self::new(format!("/{}", parts.join("/")));
        }
        Self::new(parts.join("/"))
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

/// True for a Windows drive prefix such as `C:` (never a valid first component of a
/// library-relative path).
fn is_drive_prefix(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
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
        assert!(RelPath::new("C:/Users/a.jpg").is_err());
        assert!(RelPath::new("C:\\a.jpg").is_err());
    }

    #[test]
    fn backslash_is_rejected_not_converted() {
        // Same string must mean the same thing on every OS: reject, never reinterpret.
        assert!(RelPath::new("trip\\a.jpg").is_err());
        assert_eq!(
            RelPath::new("trip/a.jpg")
                .unwrap()
                .parent()
                .unwrap()
                .as_str(),
            "trip"
        );
    }

    #[test]
    fn from_native_joins_components_with_forward_slashes() {
        let native = Utf8Path::new("2020").join("trip").join("a.jpg");
        assert_eq!(
            RelPath::from_native(&native).unwrap().as_str(),
            "2020/trip/a.jpg"
        );
        assert!(RelPath::from_native(Utf8Path::new("../x.jpg")).is_err());
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
