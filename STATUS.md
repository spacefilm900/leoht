# Leoht status

_Last updated: 2026-11-28 by Leoht Master. Update this file in every PR that moves a gate._

## Now
**Phase 0 — scaffold.** The public repo `spacefilm900/leoht` exists and the scaffold PR #1 is merged to `main` (2026-11-27). CI is green on macOS (`macos-15`, Apple Silicon), Linux and
(non-blocking) Windows; CodeRabbit has reviewed. The clean-room Picasa feature inventory (`docs/research/`) and the early-feature-trial gate (Gt) are merged (PR #2). The functional specification (`docs/spec/functional-spec.md`, 107 requirements) was reviewed by Keith on 2026-11-27 (PR #3) and his decisions are applied.

**Phase 1 — scanner and index: in progress** (started 2026-11-28 at Keith's instruction, "Do start"). Work lands as a
series of PRs, none merged without Keith: (1) scanner, metadata and catalog index (`phase-1/scanner-index`, PR #4);
(2) thumbnails, the CLI (`docs/cli.md`), the no-network test and the HEIC trial check (`phase-1/thumbs-cli`, stacked on #4);
(3) volume and performance baselines. File watching (FR-LIB-02) is not done yet. Not every G1 entry criterion was met when Phase 1
started (G0a trademark is open; Phase 1 tasks are tracked in PRs rather than separate issues; the nightly `Volume`
workflow had not yet run on `main`); Keith chose to proceed. Progress notes: `docs/process/narratives/phase-1-scanner-and-index.md`.

## Phases
| Phase | Scope (from LEOHT_PLAN.md) | State |
|-------|----------------------------|-------|
| 0 | Repo structure, data model, ADRs, CI skeleton, test harness, quality standards | **Done** (PRs #1–#3 merged; G0a trademark still open) |
| 1 | Scanner & index, JPEG/PNG/HEIC/RAW, EXIF, thumbnails, SQLite + watch, CLI demo, volume tests 1k–10k | **In progress** (PR #4 scanner + index and PR #5 thumbnails + CLI open for review) |
| 2 | Viewer & grid UI (egui) | Not started |
| 3 | Non-destructive edit engine | Not started |
| 4 | Stars, captions, tags, albums, search | Not started |
| 5 | Mac polish & packaging | Not started |
| 6+ | Linux/Windows, faces, geotag, batch ops, 100k+ scale | Not started |

## Human gates (Keith)
| Gate | State |
|------|-------|
| Trademark clearance for "Leoht" (USPTO, EUIPO, domains, common-law) | **Open** — repo was made public anyway at Keith's direction; the name may still change |
| Create public repo `spacefilm900/leoht`, enable CodeRabbit, set branch protection | Repo created (public) with Keith's approval while clearance is pending; CodeRabbit active; branch protection requiring `CI OK` is set |
| `CLEAN_ROOM.md` rule 6 wording (recorded public-repo exception) | **Confirmed by Keith 2026-11-27** |
| Plan principle 2 and Phase 3 aligned with ADR-0002 (sidecar canonical, DB index only) | **Done**, approved by Keith 2026-11-27 |
| ADR-0006: library layout and scanning rules (cache outside the library, `leoht-library.json`, hidden files and symlinks skipped) | **Accepted by Keith 2026-11-28** |
| Approve ADR-0001 … ADR-0005 (and `LEOHT_PLAN.md`) | **Approved by Keith 2026-11-27, "for now"** (to be revisited as the spec and trials teach us more) |
| Functional specification (`docs/spec/`) | **Reviewed by Keith 2026-11-27**; decisions below applied |
| SQ-1: HEIC in trials | **Decided 2026-11-27:** trial builds exclude HEIC decoding; JPEG and PNG samples only; revisit after the G-legal review |
| SQ-2: performance targets (FR-FND-07) | **Accepted as starting points 2026-11-27**; calibrated from Phase 1 baselines and recalibrated at each phase exit |
| SQ-4: rating model | **Decided 2026-11-27:** 0 to 5 stars plus a reject flag |
| SQ-5: minimum macOS | **Decided 2026-11-27:** macOS 15 (Sequoia), Apple Silicon only |
| SQ-6: sample photos | **Decided 2026-11-27:** Keith supplies them; published under the repo licence (MIT); GPS and personal metadata stripped before commit. **Slot open until he provides them** (FR-FND-14) |
| OQ-2 to OQ-6, SQ-3 (defaults in the spec) | **Default applied, may revisit** (Keith did not object 2026-11-27); G-import gate stands |
| OQ-1: Picasa metadata import | **Decided by Keith 2026-11-27:** read-only importer, Phase 6 or later, gated on a legal review (gate G-import) |
| Early feature trials (Gt): each feature group ships as a rough trial Keith tries on his Apple Silicon Mac before polish; every trial must be very easy to run (`docs/process/phase-gates.md`) | **Adopted 2026-11-27** at Keith's request; first trial to be scheduled with the first feature group |
| Focused-core quality bar (Core Principle 6): define measurable bars for organise, view, crop and adjust before widening scope | Open (define during Phase 1–3 planning) |
| Choose and set up a CLA (and CLA bot); until then outside PRs are not merged (ADR-0005) | Open |
| IP-lawyer review before any commercial release: HEVC patents, RAW licences, trademark, CLA wording, paid-edition structure (ADR-0005) | Open (not needed for development) |
| Legal review of HEVC patent exposure before distributing binaries that decode HEIC | Open (needed before any release, not before dev) |

## Open questions
* HEIC metadata is tested only on a generated metadata-only HEIF; real iPhone HEIC files (Keith's samples) are needed to confirm. CR3 is detection-only in Phase 1.
* HEIC decoder: `heifer` vs `heic-rs` (Phase 1 spike; ADR-0004).
* RAW approach given LGPL-2.1 Rust RAW crates and LibRaw's LGPL-2.1-or-CDDL-1.0 licence (ADR-0004, ADR-0005).
* Commercial HEIC path: Apple system frameworks behind a `heic-system` feature vs the pure-Rust decoder (ADR-0005; include in the Phase 1 spike).
* Dependency licence register: Appendix A of `LEOHT_PLAN.md` (update on every dependency change).
* Where albums are persisted (decide in Phase 4; ADR-0001).
* Windows CI becomes required in Phase 6.
* GitHub retired the `macos-14` runner on 2026-11-02; CI uses `macos-15`. Revisit when `macos-15` is deprecated.

## Risks
* CodeRabbit's 28 first-review comments on PR #1 are triaged and fixed (Keith approved 2026-11-27). CodeRabbit skipped re-review of the later pushes, so run a manual review request if needed.
* HEIC pure-Rust crates are weeks old (maintenance/correctness risk).
* Name may fail trademark clearance → rename cost grows once the repo is public.

## Metrics (filled in from Phase 1)
Scan/hash/index timings at 1k and 10k photos from the nightly `Volume` workflow: _none yet_.
