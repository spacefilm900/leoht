//! Process exit codes: part of the CLI contract (docs/cli.md). Never renumber.

use std::process::ExitCode;

/// A documented exit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exit(u8);

/// The command did what was asked (damaged photos are reported, not errors).
pub const OK: Exit = Exit(0);
/// The command failed (unreadable folder, bad config, catalog error, ...).
pub const ERROR: Exit = Exit(1);
/// Bad arguments. Produced by the argument parser itself; listed for completeness.
#[allow(dead_code)]
pub const USAGE: Exit = Exit(2);
/// `verify` found the catalog out of step with the disk.
pub const DRIFT: Exit = Exit(3);

impl From<Exit> for ExitCode {
    fn from(e: Exit) -> Self {
        ExitCode::from(e.0)
    }
}
