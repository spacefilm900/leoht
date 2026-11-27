# ADR-0003: Code-quality standards

* Status: Proposed
* Date: 2026-11-26
* Deciders: Keith Haviland (owner), Leoht Master (coordinator)

## Context and problem statement
Leoht is built largely by AI agents working in parallel, reviewed by CodeRabbit and a human owner. Quality has to be
**mechanically enforced**, not remembered: the plan calls for strict Rust, small focused files, readable code and
good comments, and a rigorous test pyramid including performance and volume tests.

## Decision drivers
* Agents drift without hard, automated rules; reviewers (human or bot) must have something objective to cite.
* Code must stay readable to a newcomer and to the next agent that touches it.
* The same rules must hold on a MacBook Air and in CI.

## Considered options
1. Written guidelines only.
2. Guidelines plus automated gates (compiler, lints, CI scripts, CodeRabbit instructions). **(chosen)**

## Decision

### Key decisions
* **Strict toolchain, pinned.** `rust-toolchain.toml` pins the compiler (edition 2024); `rustfmt.toml` uses stable options only;
  `[workspace.lints]` enables clippy `all` + `pedantic`, forbids `unsafe`, denies `unwrap`/`todo`/`dbg!`, warns on `expect`
  and on missing docs. CI runs clippy with `-D warnings`. Tests are exempt from the unwrap/expect rules (clippy.toml).
* **Small, focused files.** Soft cap ~300 lines per Rust file, hard cap 500, enforced by `cargo xtask check-size` in CI.
  One responsibility per module; crate boundaries follow the architecture (`core` has no I/O, `catalog` is only SQLite, etc.).
* **Errors and comments.** Library crates return typed errors (`thiserror`); only binaries (`leoht-cli`, `xtask`) use `anyhow`.
  Every public item has a `///` comment (with `# Errors` where fallible). Comments explain *why*, not *what*; invariants,
  platform quirks and non-obvious maths are always commented.
* **Test pyramid, enforced in CI.** Unit tests beside the code; integration tests across crates (`tests/`); `proptest` for
  invariants (e.g. sidecar round-trip); `insta` snapshots for formats; golden-image tests with a stated tolerance and
  `cargo xtask bless`; `criterion` benchmarks; deterministic synthetic-library **volume tests** (1k/10k in Phase 1, 100k in
  Phase 6). A change that adds behaviour without a test is rejected in review.
* **Dependency and review hygiene.** New dependencies go in `[workspace.dependencies]` with a justification and must pass
  `cargo-deny` (permissive licences only unless an ADR says otherwise). CodeRabbit reviews every PR against these rules
  (`.coderabbit.yaml`); conventional-commit titles; squash-merge to `main`; `leoht-core` must stay UI-agnostic.

## Consequences
* Good: objective review criteria; fast onboarding for new agents; consistent code across parallel workstreams.
* Bad: pedantic clippy and strict docs slow the first pass; mitigated by targeted `allow`s with a comment explaining why.
* Bad: line caps are blunt; the 300-line figure is a soft prompt to split, not a goal in itself.
* Follow-up: revisit thresholds (`too-many-lines-threshold`, file caps) at the Phase 1 exit review with real data.
