//! Leoht domain model.
//!
//! This crate defines *what things are* (photos, folders, albums, edit
//! recipes) and nothing about *how they are stored or shown*. It performs no
//! file or database I/O and depends on no UI toolkit, so any front-end — egui
//! today, Tauri or SwiftUI via FFI later — can reuse it unchanged.
//!
//! See `docs/adr/0001-data-model.md` for the reasoning behind these types.

mod album;
mod error;
mod format;
mod hash;
mod path;
mod photo;
mod recipe;

pub use album::{Album, AlbumId, AlbumMember};
pub use error::CoreError;
pub use format::{FormatSupport, PhotoFormat};
pub use hash::ContentHash;
pub use path::RelPath;
pub use photo::Photo;
pub use recipe::{EditOp, EditRecipe, RedEyeRegion};
