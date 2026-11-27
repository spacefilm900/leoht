# Leoht status

_Last updated: 2026-11-26 by Leoht Master. Update this file in every PR that moves a gate._

## Now
**Phase 0 — scaffold.** The public repo `spacefilm900/leoht` exists and PR #1 carries the scaffold. CI is green on macOS (`macos-15`, Apple Silicon), Linux and
(non-blocking) Windows; CodeRabbit has reviewed. Awaiting Keith's review and ADR approval; PR #1 is not merged.

## Phases
| Phase | Scope (from LEOHT_PLAN.md) | State |
|-------|----------------------------|-------|
| 0 | Repo structure, data model, ADRs, CI skeleton, test harness, quality standards | **In progress** (scaffold built locally; awaiting human gates) |
| 1 | Scanner & index, JPEG/PNG/HEIC/RAW, EXIF, thumbnails, SQLite + watch, CLI demo, volume tests 1k–10k | Not started |
| 2 | Viewer & grid UI (egui) | Not started |
| 3 | Non-destructive edit engine | Not started |
| 4 | Stars, captions, tags, albums, search | Not started |
| 5 | Mac polish & packaging | Not started |
| 6+ | Linux/Windows, faces, geotag, batch ops, 100k+ scale | Not started |

## Human gates (Keith)
| Gate | State |
|------|-------|
| Trademark clearance for "Leoht" (USPTO, EUIPO, domains, common-law) | **Open** — repo was made public anyway at Keith's direction; the name may still change |
| Create public repo `spacefilm900/leoht`, enable CodeRabbit, set branch protection | Repo created (public) with Keith's approval while clearance is pending; CodeRabbit active; **branch protection not yet set** (recommend requiring `CI OK`) |
| Approve ADR-0001 … ADR-0004 | Open |
| Legal review of HEVC patent exposure before distributing binaries that decode HEIC | Open (needed before any release, not before dev) |

## Open questions
* HEIC decoder: `heifer` vs `heic-rs` (Phase 1 spike; ADR-0004).
* RAW approach given LGPL-2.1 Rust RAW crates (ADR-0004).
* Where albums are persisted (decide in Phase 4; ADR-0001).
* Windows CI becomes required in Phase 6.
* GitHub retired the `macos-14` runner on 2026-11-02; CI uses `macos-15`. Revisit when `macos-15` is deprecated.

## Risks
* Triage CodeRabbit's 28 first-review comments on PR #1.
* HEIC pure-Rust crates are weeks old (maintenance/correctness risk).
* Name may fail trademark clearance → rename cost grows once the repo is public.

## Metrics (filled in from Phase 1)
Scan/hash/index timings at 1k and 10k photos from the nightly `Volume` workflow: _none yet_.
