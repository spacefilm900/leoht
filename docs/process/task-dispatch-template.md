# Task dispatch template

Copy into a GitHub Issue (label `task`). One task = one PR.

```markdown
## Task: <imperative title>
**Phase:** N   **Crate(s):** leoht-…   **Branch:** phase-N/<slug>   **Estimate:** S / M / L

### Why
<one paragraph; link ADRs / plan section>

### Scope (do)
- …

### Out of scope (do not)
- …

### Interfaces / contracts
<public types or functions this task must provide or must not change>

### Acceptance
- [ ] Tests: <unit / integration / golden / volume — be specific>
- [ ] `cargo xtask ci` passes
- [ ] Docs: public items documented; ADR/STATUS updates if needed

### Constraints
Clean-room (CLEAN_ROOM.md); ADR-0003 standards; files < 300 lines; no unwrap in lib code.

### Stop and open a `human-gate` issue if
- A licence, patent, naming or legal question comes up
- The task needs a new dependency not in `[workspace.dependencies]`
```
