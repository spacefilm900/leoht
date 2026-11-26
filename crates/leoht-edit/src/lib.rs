//! Non-destructive editing.
//!
//! Phase 0 ships the *persistence* half: the `.leoht.json` sidecar that stores an
//! [`leoht_core::EditRecipe`] next to each photo. The pixel pipeline that applies
//! a recipe arrives in Phase 3. See `docs/adr/0002-sidecar-format.md`.

mod error;
mod sidecar;

pub use error::EditError;
pub use sidecar::{SCHEMA_VERSION, SIDECAR_SUFFIX, Sidecar, sidecar_path_for};
