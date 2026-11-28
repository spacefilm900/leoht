//! FR-FND-08: Leoht makes no network connection.
//!
//! The real binary runs a full scenario (scan, list, thumbs, verify, rebuild,
//! info) under an OS-level recorder, and the test fails on any *attempt*, not
//! only on a successful connection:
//!
//! * Linux: `strace -f -e trace=%network` records every network system call
//!   (`socket`, `connect`, `sendto`, ...) by the process and its threads. Any
//!   recorded call fails the test, including local (Unix-domain) sockets:
//!   Leoht has no reason to open one.
//! * macOS: `sandbox-exec` with a profile that kills the process with SIGKILL
//!   on any network operation, so an attempt cannot be swallowed by the code
//!   that made it.
//!
//! A canary (a deliberate connection attempt from this test binary) proves
//! each recorder actually catches attempts. Without the recorder installed
//! the test is skipped with a message, unless `LEOHT_REQUIRE_NETWORK_CHECK=1`
//! (set in CI), which makes a missing recorder a failure.

// Recorder helpers live outside `#[test]` functions, where the workspace
// allows unwrap; this file is test code throughout, and the skip notice is
// meant for the person running the tests.
#![allow(clippy::unwrap_used, clippy::print_stderr)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use camino::Utf8PathBuf;
use leoht_testkit::corpus::write_metadata_corpus;

/// Not a real test: the deliberate connection attempt the recorders must catch.
#[test]
#[ignore = "helper run by the no-network test as its canary"]
fn network_canary() {
    // Port 9 (discard) on loopback: refused or not, the attempt is a syscall.
    let _ = std::net::TcpStream::connect(("127.0.0.1", 9));
}

/// A command that runs `program args` under the platform's network recorder,
/// and a check that reports whether that run attempted network access.
trait Recorder {
    fn run(&self, program: &Path, args: &[&str]) -> Output;
    /// `Some(evidence)` if the run attempted network access.
    fn attempted(&self, output: &Output) -> Option<String>;
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    pub struct Strace {
        pub exe: PathBuf,
        pub log: tempfile::NamedTempFile,
    }

    impl Recorder for Strace {
        fn run(&self, program: &Path, args: &[&str]) -> Output {
            Command::new(&self.exe)
                .args(["-f", "-qq", "-e", "trace=%network", "-o"])
                .arg(self.log.path())
                .arg(program)
                .args(args)
                .output()
                .unwrap()
        }

        fn attempted(&self, _output: &Output) -> Option<String> {
            let log = std::fs::read_to_string(self.log.path()).unwrap();
            (!log.trim().is_empty()).then_some(log)
        }
    }

    pub fn recorder() -> Option<Box<dyn Recorder>> {
        let exe = std::env::var_os("LEOHT_STRACE")
            .map(PathBuf::from)
            .or_else(|| super::which("strace"))?;
        Some(Box::new(Strace {
            exe,
            log: tempfile::NamedTempFile::new().unwrap(),
        }))
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    /// Any network operation, inbound or outbound, local or remote, kills the process.
    const PROFILE: &str = "(version 1)(allow default)(deny network* (with send-signal SIGKILL))";

    pub struct Sandbox;

    impl Recorder for Sandbox {
        fn run(&self, program: &Path, args: &[&str]) -> Output {
            Command::new("/usr/bin/sandbox-exec")
                .args(["-p", PROFILE])
                .arg(program)
                .args(args)
                .output()
                .unwrap()
        }

        fn attempted(&self, output: &Output) -> Option<String> {
            use std::os::unix::process::ExitStatusExt;
            (output.status.signal() == Some(9)).then(|| "killed by the sandbox (SIGKILL)".into())
        }
    }

    pub fn recorder() -> Option<Box<dyn Recorder>> {
        Path::new("/usr/bin/sandbox-exec")
            .exists()
            .then(|| Box::new(Sandbox) as Box<dyn Recorder>)
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod platform {
    pub fn recorder() -> Option<Box<dyn super::Recorder>> {
        None
    }
}

#[allow(dead_code)] // unused on platforms without a recorder
fn which(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|p| p.is_file())
}

#[test]
fn full_scenario_makes_no_network_attempt() {
    let Some(recorder) = platform::recorder() else {
        assert!(
            std::env::var("LEOHT_REQUIRE_NETWORK_CHECK").as_deref() != Ok("1"),
            "LEOHT_REQUIRE_NETWORK_CHECK=1 but no network recorder (strace / sandbox-exec) found"
        );
        eprintln!("SKIPPED: no network recorder on this machine (install strace on Linux)");
        return;
    };

    // The recorder must catch a deliberate attempt, or a pass means nothing.
    let me = std::env::current_exe().unwrap();
    let canary = recorder.run(
        &me,
        &["network_canary", "--exact", "--ignored", "--nocapture"],
    );
    assert!(
        recorder.attempted(&canary).is_some(),
        "the recorder missed the canary's connection attempt: {canary:?}"
    );

    let (lib, cache) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let root = Utf8PathBuf::from_path_buf(lib.path().to_path_buf()).unwrap();
    write_metadata_corpus(&root).unwrap();
    let cache = cache.path().to_str().unwrap();
    let leoht = Path::new(env!("CARGO_BIN_EXE_leoht"));
    let scenario: [&[&str]; 6] = [
        &["scan", root.as_str()],
        &["list", root.as_str()],
        &["thumbs", root.as_str()],
        &["verify", "--hashes", root.as_str()],
        &["rebuild", root.as_str()],
        &["info", root.as_str()],
    ];
    for args in scenario {
        let mut full = args.to_vec();
        full.extend(["--json", "--cache-dir", cache]);
        let out = recorder.run(leoht, &full);
        if let Some(evidence) = recorder.attempted(&out) {
            panic!(
                "`leoht {}` attempted network access:\n{evidence}",
                args.join(" ")
            );
        }
        assert!(out.status.success(), "leoht {args:?} failed: {out:?}");
    }
}
