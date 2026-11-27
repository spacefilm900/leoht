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
| G0b Create public repo, enable CodeRabbit, branch protection | After G0a | Keith (Master prepares the exact steps) |
| G0c ADR approval | Before Phase 1 starts | Keith approves ADR-0001…0005 |
| G1 Phase 1 entry | After scaffold CI is green on `main` | See "Phase 1 entry criteria" in the proposal |
| G1x Phase 1 exit | CLI demo + 1k/10k volume baselines recorded | Keith |
| G-commercial Commercial release readiness | Before any paid build, App Store listing or paid marketing | Trademark cleared; IP-lawyer review (ADR-0005); CLA in place; Appendix A reviewed |
| G-legal HEIC/HEVC patent review | Before distributing any binary that decodes HEIC | Keith / counsel |
| Gf Focused-core quality bar | Before any new feature area or scope widening beyond organise, view, crop and adjust (Core Principle 6) | Keith confirms the focused core meets its quality bar |
| Gn Phase N exit | Each phase | Keith reviews narrative, tests and demo |

## Phase exit checklist (every phase)
- [ ] Working code merged; CI green on macOS and Linux
- [ ] Core Principle 6: does the focused core (organise, view, crop, adjust) meet its quality bar before this phase widens scope?
- [ ] Tests at the right levels (unit, integration, golden/perf/volume where the phase calls for them)
- [ ] ADRs written/updated; `STATUS.md` current
- [ ] "How we built X" narrative in `narratives/`
- [ ] Keith's sign-off recorded on the exit issue

## CI policy by platform
macOS (macos-15, Apple Silicon) and Linux are required checks. Windows runs but is non-blocking until Phase 6.
Volume tests: 1k and 10k synthetic photos from Phase 1; the 100k suite is Phase 6.
