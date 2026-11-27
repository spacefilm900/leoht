# Review checklist (CodeRabbit and humans)

**Correctness & scope**
- [ ] Does what the linked issue asks, nothing more; no unrelated refactors
- [ ] Originals are never modified; edits go to sidecars; the catalog stays rebuildable

**Standards (ADR-0003)**
- [ ] fmt/clippy clean; no `unwrap`/`expect`/`panic`/`todo` in library code; no `unsafe`
- [ ] Public items documented (`# Errors` where fallible); comments explain *why*
- [ ] Files < ~300 lines, one responsibility each; crate boundaries respected (`leoht-core` has no I/O or UI deps)

**Tests**
- [ ] New behaviour has tests at the right level; tests are deterministic
- [ ] Golden-image changes include before/after and a reason; snapshot changes are intentional
- [ ] Performance-sensitive code has a benchmark or volume test

**Legal & safety**
- [ ] Clean-room: no copied code/text/assets; fixtures have provenance rows
- [ ] New dependencies are in `[workspace.dependencies]`, permissively licensed, justified
- [ ] Untrusted input (file contents, paths) cannot panic or escape the library root

**Process**
- [ ] ADR added/updated if a decision changed; `STATUS.md` updated if a gate moved
