# How we built the scanner and index (Phase 1, in progress)

* Dates: 2026-11-28 → (open)
* Agents/roles involved: Leoht Master (coordinator), one executor agent working locally on a clone of the public repo
* PRs / issues: #4 scanner + index; #5 thumbnails, CLI, no-network and trial checks (stacked on #4); volume/perf to follow

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
* The sandbox for the strace-based no-network test could not use the package manager (no root), so strace was
  unpacked from the Debian archive into the home folder to run the test locally before CI.

## Numbers
_Filled in as PRs land: test counts by level, CI duration, and the 1k/10k measurements against FR-FND-07._

## Lessons for the next phase
_To be written at the Phase 1 exit._
