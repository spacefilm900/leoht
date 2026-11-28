//! Thumbnails: resizing, and where cached thumbnails live on disk.
//!
//! JPEG and PNG are decoded with the `image` crate. RAW files are never
//! decoded in Phase 1: their thumbnail comes from the JPEG preview the camera
//! embedded. HEIC is not decoded in trial builds at all (ADR-0004, SQ-1), so it
//! gets no thumbnail yet. Every decode runs under [`decode_limits`].

mod cache;
mod error;
mod render;
mod resize;

pub use cache::cache_path;
pub use error::ThumbError;
pub use render::{MAX_DECODE_BYTES, Request, ThumbOrigin, decode_limits, render, write_atomically};
pub use resize::{render_to_cache, resize_to_fit, thumbnail};
