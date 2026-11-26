# Leoht status

_Last updated: 2026-11-26 by Leoht Master. Update this file in every PR that moves a gate._

## Now
**Phase 0 — scaffold.** A local, unpushed scaffold passes `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`
and `cargo test --workspace`. Nothing has been pushed; no GitHub repository exists yet.

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
| Trademark clearance for "Leoht" (USPTO, EUIPO, domains, common-law) before the public repo is created/named | **Open** |
| Create public repo `spacefilm900/leoht` and enable CodeRabbit + branch protection | **Blocked on the gate above** |
| Approve ADR-0001 … ADR-0004 | Open |
| Legal review of HEVC patent exposure before distributing binaries that decode HEIC | Open (needed before any release, not before dev) |

## Open questions
* HEIC decoder: `heifer` vs `heic-rs` (Phase 1 spike; ADR-0004).
* RAW approach given LGPL-2.1 Rust RAW crates (ADR-0004).
* Where albums are persisted (decide in Phase 4; ADR-0001).
* Windows CI becomes required in Phase 6.

## Risks
* HEIC pure-Rust crates are weeks old (maintenance/correctness risk).
* Name may fail trademark clearance → rename cost grows once the repo is public.

## Metrics (filled in from Phase 1)
Scan/hash/index timings at 1k and 10k photos from the nightly `Volume` workflow: _none yet_.
