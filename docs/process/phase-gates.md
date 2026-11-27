# Phase gates and human-in-the-loop

**GitHub Issues and Pull Requests are the coordination surface.** Tasks are issues, work is PRs, decisions needing Keith are
issues labelled `human-gate`. Chat messages are not the record; issues and PRs are.

## How a gate works
1. Master opens a `human-gate` issue (template provided) stating the decision, options and a recommendation.
2. Dependent work stops. Agents do not work around a gate.
3. Keith closes the issue (or approves the PR) with his decision; Master records it in `STATUS.md` and, if architectural, in an ADR.

## Gates
| Gate | When | Needs |
|------|------|-------|
| Gx-arch Architecture review | Whenever an ADR is proposed | Keith approves the ADR PR |
| Gx-ux UX review | From Phase 2 (viewer/grid) onwards, and before Mac polish (Phase 5) | Keith |
| Gx-privacy Legal/privacy review | Before face recognition or any sharing/export feature (Phase 6+) | Keith |
| G0a Trademark / name clearance | Before the public repo is created or named | Keith (USPTO, EUIPO, domains, common-law searches; possibly counsel) |
| G0b Create public repo, enable CodeRabbit, branch protection | Normally after G0a. **Approved exception:** Keith approved creating the public repo on 2026-11-26 while G0a is still open (not a clearance; the name may change). Still open: branch protection requiring `CI OK` | Keith (Master prepares the exact steps) |
| G0c ADR approval | Before Phase 1 starts | Keith approves ADR-0001…0005 |
| G1 Phase 1 entry | After scaffold CI is green on `main` | The criteria listed under "Phase 1 entry criteria" below |
| G1x Phase 1 exit | CLI demo + 1k/10k volume baselines recorded | Keith |
| G-commercial Commercial release readiness | Before any paid build, App Store listing or paid marketing | Trademark cleared; IP-lawyer review (ADR-0005); CLA in place; Appendix A reviewed |
| G-legal HEIC/HEVC patent review | Before distributing any binary that decodes HEIC | Keith / counsel |
| Gt Early feature trial | For each feature group, **before** it is polished: a rough trial build that Keith tries hands-on on his Apple Silicon Mac (see "Early feature trials" below) | Keith's trial sign-off recorded on the trial issue; the trial meets the ease-of-trial requirement |
| G-import Picasa metadata importer | Before any work on the read-only Picasa importer (Keith decided 2026-11-27: Phase 6 or later, read-only, gated on a legal review) | A completed clean-room and legal review, recorded and approved by Keith |
| Gf Focused-core quality bar | Before any new feature area or scope widening beyond organise, view, crop and adjust (Core Principle 6) | Keith confirms the focused core meets its quality bar |
| Gn Phase N exit | Each phase | Keith reviews narrative, tests and demo |

## Early feature trials (Gt)
Features are hard to judge from a description (see `docs/research/picasa-feature-inventory.md`). So **each feature group first
ships as a rough early trial build**, behind a feature flag where appropriate, which Keith tries hands-on on his Apple Silicon
Mac **before** the group is polished. His trial sign-off is an explicit gate step: polishing, a phase exit (Gn) and any
UX review (Gx-ux) for that group do not start until the trial issue records Keith's verdict (accepted, or changes
requested, which means another trial).

Flow: Master opens a trial issue (`.github/ISSUE_TEMPLATE/trial.md`) with the build link and the checklist; Keith tries it
and records his verdict there; Master records it in `STATUS.md`; only then does polish begin.

### REQUIREMENT: every trial must be very easy for users to run
A trial that fails any item below is not ready to hand to Keith. Master checks every item before opening the trial issue.

| # | Requirement | How it is checked |
|---|-------------|-------------------|
| T1 | **One step to get it:** a single download, or a single copy-paste command. | The get-it instructions in the trial issue are exactly one download link or one command (the feedback link in T7 is separate). |
| T2 | **No toolchain:** no Rust, Xcode tools or build steps for the tester. | Cold-start rehearsal (below) uses a machine without a Rust toolchain. |
| T3 | **Photos ready:** sample photos are bundled (with provenance recorded in `tests/fixtures/PROVENANCE.md`), or the tester points at a folder in one step. | Rehearsal reaches the feature using only the bundled samples, or, where there are none, a sample folder chosen in one step (the rehearsal supplies it). |
| T4 | **Runs on macOS Apple Silicon** (arm64, current macOS), tested before hand-off. | Built on the `macos-15` runner; the rehearsal is run on Apple Silicon. First-launch Gatekeeper steps (unsigned builds) are written out in one line in the checklist. |
| T5 | **Originals are never modified:** the trial is read-only by default, or works on copies. | Automated test: hash the sample library before and after a trial session; they must match. |
| T6 | **A short "what to try / what to tell us" checklist:** at most 10 items, doable in about 10 minutes. | The checklist is in the trial issue and `docs/process/trial-checklist-template.md` is followed. |
| T7 | **Easy feedback:** one link to a prefilled feedback form (the trial issue template) and a one-step way to copy version and environment details (command or menu item). | Rehearsal files a test feedback issue using only that link. |
| T8 | **Quick to first result:** the feature is visible within 5 minutes of starting the download. | Timed in the rehearsal. |
| T9 | **Safe to throw away:** quitting and deleting the app leaves nothing behind except a clearly named cache folder, whose removal is described in the checklist. | Rehearsal removes the trial following the checklist. |
| T10 | **Flagged where appropriate:** the feature is behind a feature flag that is on in the trial build and off in normal builds. | Normal build run shows no trial feature. |

**Cold-start rehearsal.** Before opening a trial issue, Master follows the checklist exactly as written, from a clean
macOS user account or fresh runner and with no prior setup, and records the result (pass/fail per item and the time taken)
in the issue. The rehearsal is the evidence that T1 to T10 hold.

## Phase 1 entry criteria (G1)
- [ ] G0a closed (name cleared, or renamed) and G0b complete (branch protection requiring `CI OK`, CodeRabbit active and has reviewed a PR)
- [ ] Scaffold PR merged; `CI OK` green on `main` (macOS `macos-15` and Linux required)
- [ ] ADR-0001…0005 **Accepted** by Keith
- [ ] `STATUS.md` current; Phase 1 task issues written with acceptance tests
- [ ] A small corpus plan for HEIC/RAW spike fixtures (sources + provenance), or Keith's consent to supply sample photos
- [ ] Nightly `Volume` workflow has run successfully at least once on `main`

## Phase exit checklist (every phase)
- [ ] Working code merged; CI green on macOS and Linux
- [ ] Core Principle 6: does the focused core (organise, view, crop, adjust) meet its quality bar before this phase widens scope?
- [ ] Gt: every feature group shipped this phase had an early trial, with Keith's trial sign-off recorded (or a waiver that Keith recorded on the issue)
- [ ] Tests at the right levels (unit, integration, golden/perf/volume where the phase calls for them)
- [ ] ADRs written/updated; `STATUS.md` current
- [ ] "How we built X" narrative in `narratives/`
- [ ] Keith's sign-off recorded on the exit issue

## CI policy by platform
macOS (macos-15, Apple Silicon) and Linux are required checks. Windows runs but is non-blocking until Phase 6.
Volume tests: 1k and 10k synthetic photos from Phase 1; the 100k suite is Phase 6.
