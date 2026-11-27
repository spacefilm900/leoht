//! Errors raised while constructing or validating domain values.

use thiserror::Error;

/// A domain value was rejected because it violates an invariant.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    /// A library-relative path was absolute, empty, or tried to escape the library.
    #[error("invalid library path {path:?}: {reason}")]
    InvalidPath {
        /// The offending input, as given.
        path: String,
        /// Why it was rejected.
        reason: &'static str,
    },
    /// A content hash was not `blake3:` followed by 64 lowercase hex digits.
    #[error("invalid content hash {0:?}: expected `blake3:` + 64 lowercase hex digits")]
    InvalidHash(String),
    /// An edit operation had an out-of-range or non-finite parameter.
    #[error("invalid edit operation `{op}`: {reason}")]
    InvalidEdit {
        /// Name of the operation (matches its serialised `op` tag).
        op: &'static str,
        /// Why it was rejected.
        reason: &'static str,
    },
}
