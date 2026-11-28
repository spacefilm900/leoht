# How we built the scanner and index (Phase 1, in progress)

* Dates: 2026-11-28 → (open)
* Agents/roles involved: Leoht Master (coordinator), one executor agent working locally on a clone of the public repo
  (see "What went wrong" for the clone collision)
* PRs / issues: #4 scanner + index; #5 thumbnails, CLI, no-network and trial checks (stacked on #4); #6 volume tests and
  performance baselines (stacked on #5)

_Progress note, updated with each Phase 1 PR. It becomes the Phase 1 narrative at the phase exit (Gn)._

## Goal
Phase 1 delivers the folder scanner, metadata extraction, the disposable SQLite index, thumbnails for JPEG and PNG, a
scriptable CLI, and 1k/10k volume baselines (`docs/spec/01-foundations.md`, `docs/spec/02-library-and-viewing.md`).

## What we did
### PR: scanner, metadata and catalog index
* `leoht-core` gained the metadata model (`PhotoMeta`, `CaptureTime` with its source, exact EXIF `Ratio`s,
  `Orientation`, `ColorInfo`) and damage records (`Damage`, `DamageKind`), all I/O-free.
* `leoht-scan` was split into walk (directory listing and stat only, name-sorted), probe (signature check, EXIF,
  JPEG/PNG headers, ICC description) and config (`leoht-library.json`: ignore patterns, format switches).
* `leoht-catalog` schema v2 stores every metadata field and damage record; incremental-scan helpers.
* New crate `leoht-library`: cache location outside the library, incremental scan with batched commits, rebuild.
* Tests: a generated fixture corpus (every format, EXIF variants, ICC, RAF and HEIF containers, six kinds of damaged
  file) with a golden JSON snapshot; the originals-never-modified matrix (scan, rescan, rebuild, also on a fully read-only
  library); Unicode and 400-character paths; property tests feeding random and truncated bytes to the probe.
* ADR-0006 records the layout and scanning rules; Keith accepted it on 2026-11-28.

### PR: thumbnails, CLI, no-network and trial checks
* `leoht-thumb` chooses a source per photo: the embedded JPEG preview when it is at least the thumbnail size (and
  always for RAW, which is never decoded), otherwise a full decode under memory limits. Orientation is applied in both
  paths; a golden table of all eight EXIF orientations, worked out from the EXIF definitions rather than from the
  `image` crate, is checked for JPEG and PNG. Big sources are pre-shrunk with a cheap filter before Lanczos3.
  Thumbnails are written atomically (temp file and rename).
* `leoht-library` gained `make_thumbnails` (content-hash keyed cache, parallel workers with `std::thread::scope`,
  hashes and decode failures recorded in the catalog) and a read-only `verify`.
* The `leoht` CLI now has `scan`, `list`, `thumbs`, `verify`, `rebuild` and `info`, each with `--json`, a
  `schema_version`, and documented exit codes (`docs/cli.md`). Contract tests run the real binary and pin the
  output with insta snapshots.
* FR-FND-08: the binary runs a full scenario under `strace` (Linux) or a kill-on-network `sandbox-exec` profile
  (macOS), and a deliberate canary connection proves each recorder catches attempts. `deny.toml` bans HTTP, TLS and
  telemetry crates.
* FR-LIB-04: `cargo xtask check-trial` (CI job on macos-15, part of `CI OK`) fails if the trial build's dependency
  tree contains a HEIC decoder crate or enables a HEIC feature, from the list in `xtask/heic-decoding.txt`, or if a
  trial sample is a HEIC/HEIF file by extension or content.

### PR: volume tests and performance baselines
* `leoht-testkit` writes a camera-like synthetic library (`LibrarySpec::volume`): 640×480 JPEGs with camera EXIF and a
  160×120 embedded preview (80%), PNGs (10%) and HEIC/RAW stand-ins (10%).
* `tests/volume.rs` measures the FR-FND-07 metrics at 1k and 10k photos (cold scan after dropping the OS page cache,
  first 500 photos visible to a second reader, warm start, rescans, rebuild, thumbnails) and fails on a budget miss or
  on a regression of more than 25% and more than 100 ms against `perf/baselines.json`. It also checks that EXIF was
  actually read, so a fast number cannot come from skipped work.
* Criterion benchmarks (scan, listing, thumbnail) and a `Volume` workflow on macos-15 and ubuntu (pull requests that
  touch the hot paths, `main`, nightly, manual) that also records the CLI's peak memory. It is not part of `CI OK`.

## Prompts and agent patterns
The executor brief asked for the Phase 1 requirements from the spec, strict adherence to ADR-0003 and the clean-room
rules, PRs of a reviewable size, no merging, and honest performance numbers ("report actual numbers rather than tuning to
fake a pass").

## Decisions and human gates
* Cache outside the library (rejected: a hidden `.leoht/` folder inside it), see ADR-0006 (accepted by Keith 2026-11-28).
* `.leohtignore` from Phase 0 retired in favour of a visible `leoht-library.json` (FR-LIB-03 asks for a visible file).
* HEIC is detected and its EXIF read; nothing decodes HEIC (SQ-1, G-legal).

## What worked
* Generating every fixture from code: deterministic, nothing to license, and the golden JSON makes extraction drift visible.
* `kamadak-exif`'s own TIFF writer produced EXIF for fixtures, so tests and reader share no hand-rolled byte layouts.

## Surprises
* The `image` crate's JPEG decoder reads the whole file even to report dimensions, which would make a cold scan read
  every byte of every photo. A small marker walker reads only the header segments instead.
* The scaffold's synthetic HEIC and RAW placeholders were pure noise; once the scanner checks signatures they would all
  have been "damaged", so the generator now writes a valid signature before the noise.

## What went wrong
* The first fixture run showed the Fujifilm RAF test file with no metadata: the probe read only 64 header bytes, but the
  RAF preview pointer sits at bytes 84 to 92. The golden JSON caught it before review; the header read is now 128 bytes.
* Clippy (pedantic, `-D warnings`) rejected the first draft in several places (a collapsible `if`, `chunks_exact`, a
  test helper using `unwrap` outside a `#[test]`). Fixed before the first push; noted because it shows the lints doing their job.
* A first draft of a test helper used a home-made checksum "to avoid a dependency" with a misleading name; replaced with
  BLAKE3 before commit. Small, but exactly the kind of shortcut review should catch.

* CodeRabbit's review of the scanner PR found 15 issues, most of them real: impossible dates (31 April, 29 February
  2023) and out-of-range zone offsets were accepted; a rescan would have deleted the catalog rows of a folder that was
  only temporarily unreadable; a failed rebuild left the library writing to a throwaway in-memory database; the
  "cache inside the library" refusal created folders before refusing; ORF and RW2 files lost their metadata because the
  EXIF reader rejects their private TIFF magic numbers; the JPEG frame header and ICC chunk loop were not fully bounded;
  a PNG `sRGB` chunk was ignored. All fixed with tests. Lesson: the hostile-input property tests found none of these,
  because they are about *plausible* inputs handled wrongly, not malformed ones.
* PR #4's Windows job (non-blocking) failed: a Unicode fixture name contained a tab, which Windows forbids. Linux and
  macOS accept it, so the required checks were green. Fixed by leaving that one name out on Windows.
* PR #4's Windows job failed again after the read-only library test started denying writes with `icacls`: the
  simple `W` and `D` rights include SYNCHRONIZE, and denying that also blocks opening and listing the folder. Then
  removing the protection failed because the deny entry also blocked clearing the read-only attribute. Fixed by denying
  only the specific rights (`WD,AD,WEA,WA,DE,DC`) and lifting the entry before clearing the attribute.
* **Clone collision.** While the executor worked in its local clone (`p1`), two commits it had not made appeared
  there: `25843eb` ("wip: volume/perf", which swept up its uncommitted volume work plus additions) and `93fb24e`
  (fixes for most of CodeRabbit's PR #4 comments). The coordinator could not identify their source and confirmed no
  other agent was assigned. Nothing had been pushed by them. The executor moved to a fresh clone (`p1b`), treated both
  commits as untrusted input, reviewed them line by line, kept what was correct (all of `93fb24e`, after a fix to its
  Windows test helper; the volume work in `25843eb`, finished and split into PR #6), and checked for further unknown
  commits before every push. Lesson: one working copy per agent, and an agent should commit early, so stray changes
  show up as unexplained commits rather than silently mixing with its own work.
* The sandbox for the strace-based no-network test could not use the package manager (no root), so strace was
  unpacked from the Debian archive into the home folder to run the test locally before CI.

## Numbers
**Tests** (at PR #6, all Phase 0 and Phase 1 tests; `cargo test --workspace --all-features`):

| Level | Count | Where |
|---|---:|---|
| Unit | 112 | in-crate `#[cfg(test)]` modules (core 24, scan 27, library 16, thumb 15, catalog 13, edit 8, testkit 5, xtask 4) |
| Integration | 16 | `tests/`: golden fixture JSON, originals-never-modified matrix, library rules, corpus scan, Unicode paths, RAW variants, end to end, perf rule |
| CLI contract | 7 | the real `leoht` binary, output pinned with 7 insta snapshots |
| No-network | 2 | full CLI scenario under strace / `sandbox-exec`, plus the canary that must be caught |
| Property | 5 | hostile bytes into the probe (3), sidecar round trips (2, Phase 0) |
| Volume | 1 | 1k and 10k, run only by the `Volume` workflow (ignored in the normal run) |

**CI duration:** the required checks finish in about 1 min 10 s per push (slowest: Windows tests at about 1 min 20 s,
non-blocking; macOS tests about 50 s). The `Volume` workflow takes 21 min on macos-15 and 23 min on ubuntu, most of it
generating the 10k library and thumbnailing it for the memory measurement.

**FR-FND-07 at 1k and 10k photos**, `Volume` run
[46125712164](https://github.com/spacefilm900/leoht/actions/runs/46125712164) (2026-11-28, GitHub-hosted runners:
macos-15 Apple Silicon with 3 CPUs, ubuntu x86_64 with 4 CPUs). Slowest of 5 runs for warm start and rescans;
milliseconds; these are now the baselines in `perf/baselines.json`.

| Metric | Budget 1k | macOS 1k | Ubuntu 1k | Budget 10k | macOS 10k | Ubuntu 10k |
|---|---:|---:|---:|---:|---:|---:|
| Cold scan (walk, stat, EXIF) | 5 000 | 413 | 236 | 50 000 | 5 151 | 2 498 |
| First 500 visible | 2 000 | 295 | 123 | 2 000 | 715 | 143 |
| Warm start (open + full listing) | 1 000 | 2 | 2 | 1 500 | 22 | 17 |
| Rescan, no changes | 1 000 | 4 | 4 | 5 000 | 100 | 42 |
| Rescan, 10 changed (informational) | - | 5 | 5 | - | 41 | 42 |
| Rebuild | 6 000 | 27 | 21 | 60 000 | 449 | 213 |
| Thumbnails, 256 px (informational) | 60 000 | 4 955 | 3 019 | 600 000 | 28 029 | 30 199 |

Peak memory of the CLI on the 10k library: `scan` 8 MB (macOS and Ubuntu), `list --json` 105 MB / 116 MB, `thumbs`
248 MB / 28 MB. Criterion (macOS / Ubuntu): scan of 1k photos 33 / 24 ms, listing 10k rows 11.5 / 15 ms, one
thumbnail from a 640×480 JPEG 4.6 / 5.8 ms, from a 12-megapixel JPEG 47 / 73 ms.

**How far to trust these numbers.** Every budget is met with a wide margin, but the library is synthetic and small:
640×480 JPEGs of 12–26 KB (median 20 KB), against 3–15 MB camera JPEGs and 20–60 MB RAW files. The cold scan reads only header
segments, so it should grow mostly with file count, not size, but that is not yet shown. Thumbnailing real photos will
be far slower: the 12-megapixel benchmark is 10–13× the 640×480 one, so a real 10k library would take a few minutes
(still inside the informational 600 s at that rate, but untested). No thumbnail came from an embedded preview, because
the synthetic 160 px preview is smaller than the 256 px thumbnail. "Rebuild" runs with a warm page cache, so it is much
faster than the cold scan; FR-FND-02's "rebuild within 1.2× cold scan" holds trivially here. The baselines come from one
run per platform on shared runners. Before the Phase 1 exit the run should be repeated on a real photo folder (Keith's
samples) and on a slow external disk, and the baselines re-recorded from several runs. Open: why `thumbs` peaks at
248 MB on macOS against 28 MB on Ubuntu, and whether `list --json` should stream rather than hold 10k rows (105–116 MB).

## Lessons for the next phase
_To be written at the Phase 1 exit._
