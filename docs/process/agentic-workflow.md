# Agentic workflow

## Principles (from LEOHT_PLAN.md)
1. **Spec before code.** Specs and ADRs land first; implementation PRs reference them.
2. **Isolated agents, shared contracts.** Each agent owns one crate or concern and talks to others only through the
   public interfaces in `leoht-core` and the crate APIs. No agent edits another's crate without an issue.
3. **CI is the referee.** fmt, clippy, tests (incl. golden, volume) and CodeRabbit decide whether work is acceptable, not the agent's say-so.
4. **Incremental bootstrapping.** CLI scanner and thumbnails first; UI comes after the engine is trustworthy.
5. **Humans at the gates.** Keith decides at phase gates and for legal/naming questions (see phase-gates.md).

## Roles
* **Leoht Master** — plans, writes task briefs, dispatches, integrates, keeps `STATUS.md` and the narratives current.
* **Worker agents** — implement one task each on a branch and open a PR.
* **CodeRabbit** — automated review on every PR against `.coderabbit.yaml`.
* **Keith Haviland** — owner; approves gates, ADRs, legal and naming decisions.

## Loop for one task
1. Master opens a GitHub Issue from the task-dispatch template (scope, files, acceptance tests, out-of-scope).
2. Worker branches `phase-N/...`, writes tests first where practical, implements, runs `cargo xtask ci`.
3. Worker opens a PR; CodeRabbit reviews; worker fixes; CI green.
4. Master reviews against `review-checklist.md`, merges (squash), updates `STATUS.md`, adds a line to the phase narrative.
5. Record the prompt/brief used, any iterations and any surprises in the phase narrative's working notes (the plan: "document every decision, prompt pattern, failure, and lesson").
6. Anything needing a human decision becomes a `human-gate` issue and work that depends on it stops.
