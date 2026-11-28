//! Why a file that looks like a photo could not be read.
//!
//! Damaged files are *recorded*, never silently skipped (FR-LIB-10), so the
//! viewer can show a placeholder with the reason (FR-BRW-10).

use serde::{Deserialize, Serialize};

/// The category of a problem with a photo file. The names are stable: they are
/// stored in the catalog and printed by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageKind {
    /// The file is zero bytes long.
    Empty,
    /// The bytes do not match the format the extension claims (for example a
    /// `.jpg` that is really a text file, or a truncated header).
    ContentMismatch,
    /// The header was recognised but could not be parsed.
    UnreadableHeader,
    /// The image declares a size beyond the safety limits.
    TooLarge,
    /// The file could not be opened or read (permissions, I/O error).
    Unreadable,
    /// Pixel decoding failed (found while making a thumbnail).
    DecodeFailed,
}

impl DamageKind {
    /// Every kind, for exhaustive tests and documentation.
    pub const ALL: [Self; 6] = [
        Self::Empty,
        Self::ContentMismatch,
        Self::UnreadableHeader,
        Self::TooLarge,
        Self::Unreadable,
        Self::DecodeFailed,
    ];

    /// Stable lower-case name (`"content_mismatch"`), as stored in the catalog.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::ContentMismatch => "content_mismatch",
            Self::UnreadableHeader => "unreadable_header",
            Self::TooLarge => "too_large",
            Self::Unreadable => "unreadable",
            Self::DecodeFailed => "decode_failed",
        }
    }

    /// Inverse of [`DamageKind::name`].
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// A damaged-file record: what went wrong, in a category and in words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Damage {
    /// Category of the problem.
    pub kind: DamageKind,
    /// Human-readable detail, e.g. the decoder's message.
    pub detail: String,
}

impl Damage {
    /// Convenience constructor.
    #[must_use]
    pub fn new(kind: DamageKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip_and_match_serde() {
        for kind in DamageKind::ALL {
            assert_eq!(DamageKind::from_name(kind.name()), Some(kind));
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(json, format!("\"{}\"", kind.name()));
        }
        assert_eq!(DamageKind::from_name("nope"), None);
    }
}
