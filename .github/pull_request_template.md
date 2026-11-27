## What and why
<!-- One paragraph. Link the GitHub Issue: "Closes #123". -->

## Phase / task
<!-- e.g. Phase 1 — scanner. Link the task-dispatch issue. -->

## Checklist (see docs/process/review-checklist.md)
- [ ] `cargo xtask ci` passes locally
- [ ] New behaviour has tests (unit / integration / golden / volume as appropriate)
- [ ] No `unwrap`/`expect`/`panic` in library code; public items have doc comments
- [ ] No file over ~300 lines (hard limit 500)
- [ ] Clean-room: nothing copied from proprietary or unlicensed sources (`CLEAN_ROOM.md`)
- [ ] Golden images changed? Before/after attached and the reason explained
- [ ] ADR added or updated if an architectural decision changed
- [ ] `STATUS.md` updated if a phase gate moved
