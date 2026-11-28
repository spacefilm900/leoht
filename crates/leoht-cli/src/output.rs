//! Printing: human text or one JSON document per command.
//!
//! Every JSON document is an object with `"schema_version"` (bumped on any
//! incompatible change) and `"command"`. Errors in JSON mode are also JSON,
//! on stdout, so a script can always parse what it gets.

use std::io::IsTerminal;

use serde_json::{Value, json};

/// JSON output schema version: part of the CLI contract.
pub const SCHEMA_VERSION: u32 = 1;

/// Where and how a command reports.
pub struct Output {
    json: bool,
    progress: bool,
}

impl Output {
    pub fn new(json: bool) -> Self {
        Self {
            json,
            // Progress lines only for a person watching a terminal.
            progress: !json && std::io::stderr().is_terminal(),
        }
    }

    /// Prints the command's result: `human` as text, or `body` (an object)
    /// wrapped with the schema version and command name.
    pub fn result(&self, command: &str, body: Value, human: impl FnOnce() -> String) {
        if self.json {
            let mut doc = json!({ "schema_version": SCHEMA_VERSION, "command": command });
            if let (Some(doc), Value::Object(body)) = (doc.as_object_mut(), body) {
                doc.extend(body);
            }
            println!("{doc:#}");
        } else {
            print!("{}", human());
        }
    }

    /// A transient progress line on stderr (terminal only).
    pub fn progress(&self, line: &str) {
        if self.progress {
            eprint!("\r{line}\x1b[K");
        }
    }

    /// Ends a progress line.
    pub fn progress_done(&self) {
        if self.progress {
            eprint!("\r\x1b[K");
        }
    }

    /// Reports a failure.
    pub fn error(&self, e: &anyhow::Error) {
        if self.json {
            let doc = json!({ "schema_version": SCHEMA_VERSION, "error": format!("{e:#}") });
            println!("{doc:#}");
        }
        eprintln!("leoht: error: {e:#}");
    }
}

/// Milliseconds, for JSON.
pub fn millis(d: std::time::Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}
