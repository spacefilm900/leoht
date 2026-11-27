//! Content hashes: identity that survives renames and moves.

use serde::{Deserialize, Serialize};

use crate::CoreError;

const PREFIX: &str = "blake3:";
const HEX_LEN: usize = 64;

/// A BLAKE3 content hash, rendered as `blake3:<64 lowercase hex digits>`.
///
/// The algorithm is part of the string so a future change of hash can coexist
/// with old sidecars. Computing the hash lives in `leoht-scan`; this crate only
/// validates and carries it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ContentHash(String);

impl ContentHash {
    /// Builds a hash from the 64 lowercase hex digits of a BLAKE3 digest.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidHash`] if `hex` is not 64 lowercase hex digits.
    pub fn from_hex(hex: &str) -> Result<Self, CoreError> {
        let valid =
            hex.len() == HEX_LEN && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        if valid {
            Ok(Self(format!("{PREFIX}{hex}")))
        } else {
            Err(CoreError::InvalidHash(hex.to_owned()))
        }
    }

    /// The full `blake3:…` string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ContentHash {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.strip_prefix(PREFIX) {
            Some(hex) => Self::from_hex(hex),
            None => Err(CoreError::InvalidHash(value)),
        }
    }
}

impl From<ContentHash> for String {
    fn from(value: ContentHash) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_well_formed_and_round_trips() {
        let hex = "0".repeat(64);
        let hash = ContentHash::from_hex(&hex).unwrap();
        assert_eq!(hash.as_str(), format!("blake3:{hex}"));
        assert_eq!(
            ContentHash::try_from(hash.as_str().to_owned()).unwrap(),
            hash
        );
    }

    #[test]
    fn rejects_malformed() {
        assert!(ContentHash::from_hex("abc").is_err());
        assert!(ContentHash::from_hex(&"G".repeat(64)).is_err());
        assert!(
            ContentHash::from_hex(&"A".repeat(64)).is_err(),
            "uppercase is not canonical"
        );
        assert!(ContentHash::try_from("sha1:abc".to_owned()).is_err());
    }
}
