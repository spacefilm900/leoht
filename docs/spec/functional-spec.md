# Leoht functional specification

* Status: **Reviewed by Keith on 2026-11-27 (PR #3); his decisions are applied in section 8.** Requirements are the agreed starting point, to be refined by trials and phase-exit calibration. Open items are listed in section 8 (OQ-2 to OQ-6 and SQ-3 stay as defaults that may be revisited). Nothing here changes `LEOHT_PLAN.md` or ADR-0001 to ADR-0005 (approved "for now"); it links to them.
* Date: 2026-11-27
* Author: Leoht Master (agent), for Keith Haviland (owner)
* Input: the merged [Picasa feature inventory](../research/picasa-feature-inventory.md) and its [source log](../research/sources.md)

## 1. Purpose and positioning

Leoht is a Picasa-inspired, folder-first photo manager for the Mac (Apple Silicon first), built in the open in Rust. But
its **primary purpose is to be a public case study**: it shows that a high-quality desktop app whose requirements are
partly subjective ("feels like a Mac app", "colour looks right", "fast") can be built agentically.

That sets it apart from Keith's earlier spec-driven repositories, the AWK interpreter and the C compiler. Those had a precise
specification and a large conformance suite from day one: the answer to "is it correct?" was mechanical. A photo app has no
such oracle, so this project has to build its own. Its differentiators, and what this specification does about each:

| Differentiator | What this specification does |
|---|---|
| Fuzzy requirements turned into testable acceptance criteria | Every requirement has automated checks (unit, integration, golden-image, performance, volume) wherever a number or a reference output can exist, and an explicit **judgement check** with a written rubric where it cannot (section 3). |
| Recorded human judgement gates | Judgement checks run through Keith's early-trial gate (Gt) in [`phase-gates.md`](../process/phase-gates.md); the verdicts are recorded on trial issues. |
| Quality measured over time | Performance budgets by library size (FR-FND-07), baselines recorded each phase, and a quality report at every phase exit (FR-FND-13). |
| Honest narratives | Each phase's "How we built X" must include what went wrong (FR-FND-13). |

**Focused core first (Core Principle 6).** Organise, view, crop and adjust come first and must meet their quality bar
before scope widens. Requirements are therefore tagged **Core**, **Foundation** (guarantees and enabling work the core
needs) or **Widening** (anything beyond the core, which starts only after gate Gf). Core and Foundation items make up Phases 1 to 4 and most of Phase 5.

## 2. How to read this specification

**Requirement IDs.** `FR-<GROUP>-<NN>`, stable and never reused (a dropped requirement is marked *Withdrawn*, not renumbered).
`FR-` avoids confusion with inventory IDs such as `EDIT-07`. Each requirement states: phase, priority, **Traces**
(the inventory rows, plan items or ADRs it comes from) and its acceptance checks.

**Phase.** The numbering in [`LEOHT_PLAN.md`](../../LEOHT_PLAN.md): 0 foundation, 1 scanner and index, 2 viewer and grid,
3 edit engine, 4 organisation, 5 Mac polish and packaging, 6+ later. One requirement has one phase; where the inventory
splits a feature across phases it becomes two requirements.

**Priority.** Core, Foundation or Widening, as above. Keith may rebalance them (SQ-3; defaults applied, may revisit).

**Checks.** `Auto` means a machine check that runs in CI; the tag in brackets names its kind. `Judge` means a human
judgement check, run by Keith in the named trial.

| Kind | Meaning |
|---|---|
| `unit`, `property`, `snapshot` | Unit tests; `proptest` invariants; `insta` snapshots of formats and text output |
| `integration` | Cross-crate or end-to-end tests on fixture libraries, including the originals-unchanged hash check |
| `golden` | Golden-image tests with a stated tolerance, changed only by `cargo xtask bless` (ADR-0003) |
| `perf`, `volume` | `criterion` benchmarks against budgets; deterministic synthetic libraries of 1k, 10k and (from Phase 6) 100k photos |
| `ui` | UI tests that drive key and pointer sequences (for example with `egui_kittest`) |
| `judge` | Human judgement in a Gt trial; never a CI check |

**Where the evidence lives.** CI results are on the PR; trial verdicts are on the trial issue
(`.github/ISSUE_TEMPLATE/trial.md`), which also holds the readiness record that proves the trial met T1 to T10.

## 3. From fuzzy requirement to testable criterion

Every requirement must have at least one `Auto` check, or be explicitly judgement-only with a rubric. The rules:

1. **Put a number on it when a number exists.** "Fast" becomes a budget (FR-FND-07) that CI enforces, *and* a judgement
   question ("fast, OK or slow") because a passing number can still feel wrong.
2. **Use a reference when a number does not exist.** "Colour looks right" becomes charts and ICC profiles rendered to
   reference values (`golden`), *and* a side-by-side with the system viewer on Keith's Mac (FR-BRW-06).
3. **Write the rubric before the trial.** Each `Judge` check states the question, the answer scale and the pass rule
   (for example "at least 7 better of 10, no more than 1 worse", FR-ADJ-04). Rubrics are in this spec so a verdict
   cannot be bent after the fact.
4. **Judgement results are data.** Keith's answers go on the trial issue and into the phase quality report, so quality
   can be tracked over time. A failed judgement check opens an issue; it does not block CI but it does block that
   feature group's polish and exit (gate Gt).
5. **Labelled fixture sets for "smart" features.** Auto-straighten, stack grouping and best-shot suggestions get labelled
   sets with accuracy targets, plus a judgement check on realistic photos (FR-CRP-05, FR-FIL-08, FR-LAT-12).

Worked example, "the crop tool feels good" (FR-CRP-01, FR-CRP-08): ratio maths by `property` tests; the crop box
behaviour by `ui` tests; the output by `golden`; the feel by a timed 10-minute task list in Gt-CRP where each annoyance
becomes an issue and the pass rule is "no blocker left open".

## 4. Feature groups and early trials

Each feature group maps to one early trial (gate Gt), shipped as a rough build before polishing and meeting the
ease-of-trial requirement **T1 to T10** in [`phase-gates.md`](../process/phase-gates.md#early-feature-trials-gt): one
download or command, no toolchain, photos ready, Apple Silicon, originals never modified, a short checklist, easy
feedback, a first result within 5 minutes, easy removal, feature-flagged where appropriate. The checklist template is
[`trial-checklist-template.md`](../process/trial-checklist-template.md). The requirement files below are the source for each trial's checklist.

<!-- groups:start -->
| Group | Name | Trial | Phase | Reqs | Core | Foundation | Widening | File |
|---|---|---|---|---:|---:|---:|---:|---|
| FR-FND | Foundations and guarantees | Checked in every trial (no trial of its own) | 1, 2, 3 | 14 | 0 | 14 | 0 | [01-foundations.md](01-foundations.md) |
| FR-LIB | Library, scan and index | Gt-LIB (CLI, at Phase 1 exit) | 1 | 10 | 0 | 10 | 0 | [02-library-and-viewing.md](02-library-and-viewing.md) |
| FR-BRW | Browse and view | Gt-BRW (**proposed first trial**) | 2 | 11 | 10 | 1 | 0 | [02-library-and-viewing.md](02-library-and-viewing.md) |
| FR-ENG | Edit engine, save and export | Gt-CRP (ships with crop; FR-ENG-10 excluded until after Gf) | 3 | 11 | 7 | 3 | 1 | [03-editing.md](03-editing.md) |
| FR-CRP | Crop and geometry | Gt-CRP | 3 | 8 | 7 | 1 | 0 | [03-editing.md](03-editing.md) |
| FR-ADJ | Adjust | Gt-ADJ | 3 | 8 | 8 | 0 | 0 | [03-editing.md](03-editing.md) |
| FR-RAT | Rate, cull, caption and tag | Gt-RAT (FR-RAT-07 excluded until after Gf) | 4, 6 | 8 | 5 | 2 | 1 | [04-organisation.md](04-organisation.md) |
| FR-ALB | Albums, timeline and search | Gt-ALB | 4 | 7 | 6 | 1 | 0 | [04-organisation.md](04-organisation.md) |
| FR-FIL | Files, folders, duplicates and stacks | Gt-FIL | 4 | 8 | 6 | 2 | 0 | [04-organisation.md](04-organisation.md) |
| FR-MAC | Mac polish and sharing | Gt-MAC | 5 | 8 | 3 | 2 | 3 | [05-mac-polish-and-later.md](05-mac-polish-and-later.md) |
| FR-LAT | Later phases (outline) | One trial per later group at phase entry | 6 | 14 | 0 | 2 | 12 | [05-mac-polish-and-later.md](05-mac-polish-and-later.md) |
| **Total** | | | | **107** | **52** | **38** | **17** | |
<!-- groups:end -->

### Requirements per phase

<!-- phases:start -->
| Group | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Phase 5 | Phase 6 | Total |
|---|---:|---:|---:|---:|---:|---:|---:|
| FR-FND | 9 | 3 | 2 |  |  |  | 14 |
| FR-LIB | 10 |  |  |  |  |  | 10 |
| FR-BRW |  | 11 |  |  |  |  | 11 |
| FR-ENG |  |  | 11 |  |  |  | 11 |
| FR-CRP |  |  | 8 |  |  |  | 8 |
| FR-ADJ |  |  | 8 |  |  |  | 8 |
| FR-RAT |  |  |  | 7 |  | 1 | 8 |
| FR-ALB |  |  |  | 7 |  |  | 7 |
| FR-FIL |  |  |  | 8 |  |  | 8 |
| FR-MAC |  |  |  |  | 8 |  | 8 |
| FR-LAT |  |  |  |  |  | 14 | 14 |
| **Total** | **19** | **14** | **29** | **22** | **8** | **15** | **107** |
<!-- phases:end -->

### Proposed first trial: Gt-BRW (browse and view)

Recommended as the first graphical trial, and the first one Keith feels hands-on. Reasons:

* It is the first place judgement matters (speed, colour, keyboard feel), and everything else builds on it.
* It exercises the whole T1 to T10 pipeline (packaging, bundled samples, read-only default, feedback), so the pipeline is
  proven on a small feature before the edit engine needs it.
* It checks the foundations in a place Keith can see them: FR-FND-01 (hash check on the samples), -05 (keyboard),
  -07 (speed) and -08 (offline).

What Keith would try (about 10 minutes): open the bundled sample library; scroll the grid fast; open a photo; hold the right
arrow through 100 photos in full screen; compare colour with macOS Preview; switch dark and light; reveal a photo in Finder.
Pass rule: each `Judge` check in FR-BRW-01 to -09 answered at least "OK", and no blocker issue left open.

Suggested dry run: at the Phase 1 exit, a CLI-only **Gt-LIB** trial (one command, bundled samples) rehearses the
packaging and feedback pipeline before the first graphical trial depends on it.

Constraints: a trial binary that decodes HEIC would need the G-legal review (ADR-0004), so the first trials
ship JPEG and PNG samples with HEIC decoding compiled out (SQ-1, decided by Keith 2026-11-27, to be revisited after G-legal); the sample photos come from Keith, who will supply them, and the slot stays open until he does (SQ-6);
an unsigned build needs the one-line Gatekeeper step in the checklist (T4).

### Trial sequence and the focused core

Core Principle 6 sets the order: Gt-BRW, Gt-CRP (engine, crop and geometry; FR-ENG-10 RAW editing is excluded because it is Widening), Gt-ADJ, Gt-RAT (FR-RAT-07, the write-back into files, is excluded because it is Widening and waits for Gf and OQ-2), Gt-ALB, Gt-FIL, then gate Gf
(quality bar) before any Widening item, then Gt-MAC (which holds the Mac polish plus three Widening items, FR-MAC-06 to -08, that wait for Gf).

## 5. Requirement files

| File | Groups |
|---|---|
| [`01-foundations.md`](01-foundations.md) | FR-FND: originals guarantee, disposable index, atomic writes, accessibility, performance budgets, privacy, CLI, platform |
| [`02-library-and-viewing.md`](02-library-and-viewing.md) | FR-LIB (scan, index, formats, thumbnails), FR-BRW (folders, grid, lightbox, colour) |
| [`03-editing.md`](03-editing.md) | FR-ENG (recipes, undo, save, export), FR-CRP (crop, geometry), FR-ADJ (adjust, copy and paste edits) |
| [`04-organisation.md`](04-organisation.md) | FR-RAT (ratings, culling, captions, tags), FR-ALB (albums, timeline, search), FR-FIL (folder operations, duplicates, stacks) |
| [`05-mac-polish-and-later.md`](05-mac-polish-and-later.md) | FR-MAC (Phase 5), FR-LAT (Phase 6+, outline only) |
| [`traceability.md`](traceability.md) | Every inventory row mapped to its requirements or disposition |

## 6. Scope: what is out, and what is later

* **Dropped** (12 inventory rows): cloud and web albums, collaborative albums, online contacts, movie maker, gift disc,
  burned slideshows, screensaver, online editor add-on, face collage and movie, third-party library import, embed and
  link sharing. Not specified; see [`traceability.md`](traceability.md).
* **Deferred** (13 rows): single-file add (covered by FR-MAC-07), moving the index, video, network drives, colour search,
  retouch, text on image, film grain and glow, effect packs, upload while importing, print, collage, localisation
  (strings are prepared, FR-FND-12). Revisit after Gf.
* **Future possibilities** (FUT-01 to FUT-14): not committed. Six are promoted into requirements because the inventory
  said they likely would be or because correctness needs them: FUT-05 and FUT-06 (FR-FND-07, FR-LIB-09), FUT-09 (FR-FIL-02),
  FUT-11 (FR-FND-05, -06), FUT-12 (FR-FND-01) and FUT-14 (FR-ENG-11). The rest stay future.
* **Non-goals:** a mandatory cloud, accounts, telemetry, an in-place overwrite of originals in v1, and any use of Picasa's
  code, binaries, assets or private file layouts.

## 7. Decisions applied

* **OQ-1, decided by Keith on 2026-11-27:** Picasa metadata import is a **read-only importer, Phase 6 or later, gated on a legal review**.
  Recorded in the inventory (OQ-1 marked resolved) and as FR-LAT-01. A new gate, G-import, is in
  [`phase-gates.md`](../process/phase-gates.md): no work starts until the clean-room and legal review is complete.
* **Plan and ADRs:** Keith approved [`LEOHT_PLAN.md`](../../LEOHT_PLAN.md) and ADR-0001 to ADR-0005 "for now". They are
  unchanged; this specification builds on them ([ADR-0001](../adr/0001-data-model.md), [ADR-0002](../adr/0002-sidecar-format.md),
  [ADR-0003](../adr/0003-code-quality-standards.md), [ADR-0004](../adr/0004-image-format-support-strategy.md),
  [ADR-0005](../adr/0005-licensing-and-commercial-readiness.md)).

## 8. Decisions and open questions

### Resolved by Keith, 2026-11-27

| ID | Decision | Applied in |
|---|---|---|
| OQ-1 | Picasa metadata import is a read-only importer, Phase 6 or later, gated on a legal review (gate G-import). | FR-LAT-01; inventory OQ-1 |
| SQ-1 | Accepted for now: trial builds exclude HEIC decoding, and trials use JPEG and PNG samples only. To be revisited later (after the G-legal review). | FR-LIB-04, section 4 |
| SQ-2 | Performance targets in FR-FND-07 are **accepted as starting points** (still framed as proposed targets) and are calibrated against reality: baselines from Phase 1, recalibration proposed at every phase exit. | FR-FND-07, FR-FND-13 |
| SQ-4 | Rating model: 0 to 5 stars plus a reject flag. | FR-RAT-01, FR-RAT-02 |
| SQ-5 | Minimum macOS 15 (Sequoia), Apple Silicon only. This matches CI (`macos-15`). | FR-FND-11 |
| SQ-6 | Keith will supply the sample photos himself. They will be published under the repository licence (MIT), and GPS and personal metadata must be stripped before commit. The slot stays open until he provides them. | FR-FND-14 |

### Open: default applied, may revisit

Keith did not object to these on 2026-11-27, so the recommended default stays applied and marked in the spec. Any of them can still be changed.

| ID | Question | Default applied | Requirements affected |
|---|---|---|---|
| OQ-2 | Principle 1 versus writing tags, captions and coordinates into originals. | Sidecar by default; explicit, warned opt-in write-back to files, shipped after the focused core. | FR-RAT-05, FR-RAT-07, FR-LAT-06 |
| OQ-3 | Where does import live? The plan has no import phase. | A minimal, hash-verified import at the start of Phase 6, right after Gf, because import is not part of the focused core. Keith may pull it into late Phase 4. | FR-LAT-02 |
| OQ-4 | Slideshow and quick viewer: which phase? | Phase 5 with Mac polish, tagged Widening, so they wait for Gf. | FR-MAC-06, FR-MAC-07 |
| OQ-5 | Effects and presets: which to ship? | Core tone controls and a handful of looks in Phase 3; shareable preset files later; no novelty looks. | FR-ADJ-07 |
| OQ-6 | Positioning against the peer projects Dalica and Picco. | The case-study positioning in section 1 stands as the project's purpose. Toward the peers: differentiate on testing rigour, a transparent process and Mac-native polish; study their public behaviour only within licence terms, copy no code. | Section 1 |
| SQ-3 | Are the Core, Foundation and Widening tags right? In particular duplicates and stacks (FR-FIL-06 to -08) are tagged Core. | As tagged; Keith may move items behind Gf. | All priority tags |

The G-import gate in [`phase-gates.md`](../process/phase-gates.md) stands as written.

## 9. Clean-room discipline

This specification is original work. It cites only public sources already in [`sources.md`](../research/sources.md) (as
`[S..]`, rarely) and the inventory rows derived from them (`Traces`). It copies no code, text or assets, uses our own names
for features (for example "Auto-enhance"), and records no Picasa file-format details (FR-LAT-01 starts only after a
clean-room and legal review). Targets, thresholds and rubrics are Leoht's own proposals. Where a statement about a third
party is not in the source log it is marked **unverified**.

## 10. Maintenance

Requirements change only by PR, citing the inventory or a decision. A requirement changed after a trial keeps its ID and
gets a dated note. Re-verify the live-site sources in `sources.md` when a group's detailed spec task is written, because
repositories and vendor pages change.

## 11. Implications for the ADRs (not applied)

Recorded so Keith and the architecture gate can see them. None are changed here.

1. **ADR-0001 operation set.** Flip, perspective, highlights and shadows, temperature, sharpen and the looks are new operations beyond the fixed Phase 3 list. Under ADR-0002 each bumps `schema_version`. Needs an ADR update or a follow-up ADR before Phase 3.
2. **ADR-0002 optional fields.** Ratings, flags, captions, tags, hidden flag, stack overrides and custom folder order need sidecar fields (the ADR already notes the follow-up). A new ADR should fix them, including the folder sort key.
3. **ADR-0001 album location** is finalised in Phase 4 as the ADR says (FR-ALB-03).
4. **ADR-0004 and ADR-0005:** the RAW decoder licence choice gates FR-ENG-10; HEIC distribution gates trials, so trial builds exclude HEIC decoding (SQ-1, decided).
5. **New ADRs likely needed:** colour pipeline (ICC, Display P3, HDR; FR-BRW-06, FR-MAC-04); UI toolkit accessibility, after the Phase 2 spike (FR-FND-06); the stacks and duplicates model (FR-FIL-07); the Picasa importer (FR-LAT-01), only after its review.
6. **ADR-0003:** the performance budgets (FR-FND-07) and the phase-exit quality-report check (FR-FND-13) are new automation for `cargo xtask`.
