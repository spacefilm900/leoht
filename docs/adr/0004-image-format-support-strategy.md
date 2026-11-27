# ADR-0004: Image format support strategy (Phase 1)

* Status: Proposed
* Date: 2026-11-26
* Deciders: Keith Haviland (owner), Leoht Master (coordinator)

## Context and problem statement
Phase 1 must index and thumbnail the formats real libraries contain: JPEG, PNG, HEIC/HEIF and camera RAW.
Not all can be supported equally, and HEIC and RAW decoders carry **licence and patent** questions that must
be settled before binaries are distributed. Keith prefers a **pure-Rust HEIC decoder** (`heifer` or `heic-rs`).

## Decision drivers
* Mac first: iPhone HEIC photos are the most important non-JPEG format for the primary user.
* Permissive licensing (the project is MIT; `cargo-deny` rejects copyleft by default).
* No C toolchain or system library for the default build (simple CI, simple notarised Mac packaging).
* Untrusted input: safe-Rust decoders reduce memory-safety risk.

## Considered options (HEIC)
1. `heifer` — pure-Rust decoder **and encoder**.
2. `heic-rs` — pure-Rust, decoder-only.
3. `libheif-rs` (bindings to the C library libheif) behind an optional cargo feature.
4. `heic` (imazen) — pure-Rust but AGPL-3.0-only (or commercial): rejected for licence.
5. Platform decoders (e.g. Apple ImageIO via FFI on macOS): not evaluated; noted as a Phase 5 possibility.

## Decision

### Phase 1 format matrix (priority order)
| # | Format | Support level in Phase 1 |
|---|--------|--------------------------|
| 1 | **JPEG** | Full first-class: decode, EXIF, embedded thumbnail, sidecar edits, volume-test baseline. |
| 2 | **HEIC/HEIF** | First-class via a pure-Rust crate; `libheif-rs` optional fallback behind a cargo feature. |
| 3 | **PNG** | Full support. |
| 4 | **RAW** (common formats) | Detection + basic metadata + thumbnail where possible (e.g. embedded preview); deeper decoding later. |
| 5 | **Others** (TIFF, WebP, …) | Detection only in Phase 1; full support later. |

`leoht_core::PhotoFormat::support()` encodes this matrix in code (Full / Partial / DetectOnly) and has a test pinning it.

### Key decisions
* **HEIC: prefer a pure-Rust decoder behind a cargo feature; the choice between `heifer` and `heic-rs` is made by a
  Phase 1 spike**, not by this ADR (see findings). Neither is a Phase 0 dependency. Hide the choice behind a small
  internal trait/module in `leoht-thumb`/`leoht-scan` so it can be swapped.
* **`libheif-rs` is an optional fallback** behind a separate feature (e.g. `heic-libheif`), off by default, because it needs the
  C library and pulls LGPL/GPL-licensed codec libraries into the build.
* **Patent note.** HEVC (the codec inside HEIC) is covered by patents. A pure-Rust implementation avoids C dependencies and
  copyleft code but **does not grant patent rights**; both candidate crates say so in their own documentation. libheif
  links libde265 (decoding) and x265 (encoding) which carry their own licences. **Obtain legal review before distributing
  binaries that decode HEIC**, particularly outside personal use. Phase 1 development builds are not a distribution.
* **RAW: Phase 1 does detection, basic EXIF and embedded-preview thumbnails only.** The obvious Rust RAW crates
  (`rawloader`, `rawler`) are **LGPL-2.1** — in conflict with the MIT/`cargo-deny` policy (Rust links statically) — so adopting
  one requires a licence decision first (or a separate-process boundary, or a permissive alternative).
* **Others (TIFF, WebP, …) are detected and indexed but not decoded** in Phase 1; they appear in the grid with a placeholder.

## Verified findings (crates.io and GitHub, checked 2026-11-26)
Facts below come from crates.io metadata and each project's README; **no HEIC file has been decoded by us yet** — that is the spike's job.

| | `heifer` 0.7.3 | `heic-rs` 0.1.5 |
|---|---|---|
| Exists / what | Yes. Pure-Rust HEIF/HEIC **decoder and encoder** (own ISOBMFF + HEVC implementation; also `.heics` sequences) | Yes. Pure-Rust HEIC/HEIF **decoder** (own ISOBMFF container + HEVC *still-picture* decoder); explicitly not an encoder |
| Decodes HEVC pixels? | Yes, per README: own HEVC decoder; claims identical-to-ffmpeg on 62/62 comparable conformance streams, iPhone 48-tile grids, 8–12 bit, alpha, EXIF/XMP/ICC, HDR gain maps, `image`-crate integration | Yes, per README: HEVC still pictures, grid tiling, 8/10-bit, rotation/mirror/crop, EXIF + ICC extraction; no image sequences, overlays or inter prediction |
| Licence | MIT OR Apache-2.0 | MIT OR Apache-2.0 |
| `unsafe` / C deps | `forbid(unsafe_code)`, no C | `forbid(unsafe_code)`, no C; `no_std`-friendly; optional `rayon` |
| MSRV | 1.85 | 1.85 |
| Maturity / maintenance | **Very new**: first published 2026-10-08, 42 releases in 7 weeks, last release 2026-11-26; GitHub repo ~6 stars; fuzz CI claimed | **Very new**: first published 2026-09-12, 6 releases, last 2026-11-21; GitHub repo ~9 stars; ~313k crates.io downloads |
| Patent statement | Says users must comply with HEVC patent licensing | Says its licence grants no patent rights in HEVC |
| Repos | github.com/HarpeLm/Heifer | github.com/tbraun96/heic-rs |

Other verified alternatives: `heic-decoder` 0.1.1 (MIT OR Apache-2.0, pure Rust, created 2026-10-10, ~3 GitHub stars — also very new);
`heic` 0.1.6 (AGPL-3.0-only OR commercial — rejected); `libheif-rs` 3.0.0 (MIT wrapper, but links libheif; mature, 1.28M downloads).

Assessment: both preferred crates exist, are permissively licensed and claim real HEVC pixel decoding, so Keith's preference is
viable. Both are only weeks old with small communities: that is a **maintenance and correctness risk**, which is why the spike
and the swappable boundary matter. Provisional lean: **`heifer`** for breadth (colour management, metadata, `image` integration) and
an active release cadence, with **`heic-rs`** as the smaller, decoder-only alternative; do not treat the lean as a decision.

## Open questions (resolve in the Phase 1 spike / human gates)
1. **Spike**: decode a corpus of real iPhone/Samsung HEIC photos (we hold none yet; needs Keith's consent and a provenance row) with both crates. Compare
   pixel agreement vs. macOS `sips`/Preview, wall-clock and memory for 12 MP and 48 MP images, orientation and colour (Display P3), failure behaviour on corrupt files.
2. **Maintainer risk**: is either crate still maintained at Phase 1 exit? Is there a second maintainer? Vendoring/forking plan if abandoned.
3. **Legal**: HEVC patent exposure for distributing a macOS app that decodes HEIC (Keith to arrange review before any binary release).
4. **RAW**: which crate/approach, given LGPL-2.1 for `rawloader`/`rawler`? Candidate permissive approach to evaluate: read the
   embedded JPEG preview from TIFF-based RAWs via EXIF structures (`kamadak-exif`); CR3/RAF containers need separate handling.
5. **macOS-native decoding** (ImageIO) as an optional Mac-only path: ADR-0005 recommends it as the commercial Mac route (behind a `heic-system` feature); include it in the Phase 1 spike for speed and correctness. Whether it resolves patent exposure for a paid app is a question for the lawyer.

## Consequences
* Good: clear, testable scope for Phase 1; the default build stays pure Rust and permissively licensed.
* Good: HEIC decoder can be swapped without touching the rest of the system.
* Bad: HEIC support depends on very young crates; schedule risk if the spike finds gaps.
* Bad: RAW will initially show embedded previews only, which some users will find limited.
* Follow-up: add `deny.toml` exceptions only by explicit ADR; add HEIC/RAW provenance rows for any fixtures.
* See also [ADR-0005](0005-licensing-and-commercial-readiness.md): the commercial HEIC path (Apple system frameworks behind a feature, pure-Rust decoder elsewhere), the HEVC patent pools, and the RAW licence gate (including LibRaw's LGPL-2.1-or-CDDL-1.0 dual licence). Licence register: Appendix A of `LEOHT_PLAN.md`.
