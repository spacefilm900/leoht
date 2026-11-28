//! Test support: deterministic synthetic photo libraries and fixture builders.
//!
//! Given the same [`LibrarySpec`] it always writes byte-identical files, so
//! volume tests and benchmarks are reproducible across machines and CI runs.
//! No real photographs are used, so there is nothing to license or leak.
//!
//! Used by `cargo xtask gen-library`, the criterion benches, the volume tests and
//! the integration tests ([`corpus`] and [`fixtures`]).

pub mod corpus;
pub mod fixtures;
mod generate;
mod rng;

pub use generate::{GeneratedLibrary, LibrarySpec, TestkitError, generate_library};
