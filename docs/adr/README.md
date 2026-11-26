# Architecture Decision Records

One short Markdown file per significant decision, in [MADR](https://adr.github.io/madr/) style.
Spec first, code second: a decision that shapes more than one crate gets an ADR **before** the code lands.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-data-model.md) | Data model: photos, folders, albums, edit recipes | Proposed |
| [0002](0002-sidecar-format.md) | Sidecar format: `.leoht.json` | Proposed |
| [0003](0003-code-quality-standards.md) | Code-quality standards | Proposed |
| [0004](0004-image-format-support-strategy.md) | Image format support strategy (Phase 1) | Proposed |

Statuses: **Proposed** → **Accepted** (Keith approves via PR; see `docs/process/phase-gates.md`)
→ optionally **Superseded by ADR-NNNN**. Never edit an Accepted ADR's decision; write a new one that supersedes it.
Start from [0000-template.md](0000-template.md).
