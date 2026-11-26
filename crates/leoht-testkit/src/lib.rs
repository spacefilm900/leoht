//! Test support: a deterministic synthetic photo-library generator.
//!
//! Given the same [`LibrarySpec`] it always writes byte-identical files, so
//! volume tests and benchmarks are reproducible across machines and CI runs.
//! No real photographs are used, so there is nothing to license or leak.
//!
//! Used by `cargo xtask gen-library`, the criterion benches and the volume tests.

mod generate;
mod rng;

pub use generate::{GeneratedLibrary, LibrarySpec, TestkitError, generate_library};
