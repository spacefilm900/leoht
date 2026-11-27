//! Thumbnails: resizing, and where cached thumbnails live on disk.
//!
//! Phase 0 covers JPEG and PNG through the `image` crate. HEIC and RAW
//! thumbnails arrive in Phase 1 once the decoder choices in ADR-0004 are made.

mod cache;
mod error;
mod resize;

pub use cache::cache_path;
pub use error::ThumbError;
pub use resize::{render_to_cache, resize_to_fit, thumbnail};
