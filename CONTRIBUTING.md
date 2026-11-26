# Contributing

Leoht is built by humans and AI agents working from specs. Everything goes through GitHub Issues and Pull Requests.

1. **Pick or open an Issue** (use the "Agent task" template; see `docs/process/task-dispatch-template.md`).
2. **Branch**: `phase-N/short-description`. Keep PRs small and focused (one concern, ideally < 400 changed lines).
3. **Run `cargo xtask ci`** before pushing. It runs what CI runs.
4. **Open a PR** with a conventional-commit title (`feat(scan): detect HEIC files`) and complete the checklist in the template.
5. **CodeRabbit reviews every PR**; address or reply to each comment. A human (Keith) approves merges at phase gates.
6. **Standards** are in `docs/adr/0003-code-quality-standards.md`. Clean-room rules are in `CLEAN_ROOM.md`.

By contributing you agree your contribution is licensed under the MIT licence of this repository.
