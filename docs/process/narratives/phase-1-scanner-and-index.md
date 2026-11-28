# How we built the scanner and index (Phase 1, in progress)

* Dates: 2026-11-28 → (open)
* Agents/roles involved: Leoht Master (coordinator), one executor agent working locally on a clone of the public repo
* PRs / issues: scanner + index (this note's first PR); thumbnails + CLI and volume/perf to follow

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

## Numbers
_Filled in as PRs land: test counts by level, CI duration, and the 1k/10k measurements against FR-FND-07._

## Lessons for the next phase
_To be written at the Phase 1 exit._
