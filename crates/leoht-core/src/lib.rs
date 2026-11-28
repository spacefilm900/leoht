//! Leoht domain model.
//!
//! This crate defines *what things are* (photos, folders, albums, edit
//! recipes) and nothing about *how they are stored or shown*. It performs no
//! file or database I/O and depends on no UI toolkit, so any front-end — egui
//! today, Tauri or SwiftUI via FFI later — can reuse it unchanged.
//!
//! See `docs/adr/0001-data-model.md` for the reasoning behind these types.

mod album;
mod damage;
mod error;
mod format;
mod hash;
mod meta;
mod orientation;
mod path;
mod photo;
mod recipe;
mod time;

pub use album::{Album, AlbumId, AlbumMember};
pub use damage::{Damage, DamageKind};
pub use error::CoreError;
pub use format::{FormatSupport, PhotoFormat};
pub use hash::ContentHash;
pub use meta::{CaptureTime, ColorInfo, DateSource, Exposure, PhotoMeta, Ratio};
pub use orientation::Orientation;
pub use path::RelPath;
pub use photo::Photo;
pub use recipe::{EditOp, EditRecipe, RedEyeRegion};
pub use time::utc_timestamp;
