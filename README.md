# Leoht

> **Working title.** "Leoht" (Old English for *light*) is a working title. **Full trademark clearance
> (USPTO, EUIPO, domains, common-law) is still pending**, so the project name and this repository's name
> may change. Please don't build branding around it yet.

A modern, **folder-first**, **non-destructive** photo organiser and light editor in Rust, inspired by the simplicity of
Google Picasa. Mac first, then Linux and Windows. Your folders are the source of truth; edits live in small
`.leoht.json` sidecars next to your photos; the database is only a disposable index.

* Plan: [LEOHT_PLAN.md](LEOHT_PLAN.md) · Status: [STATUS.md](STATUS.md) · Decisions: [docs/adr](docs/adr) · Process: [docs/process](docs/process) · Spec (draft): [docs/spec](docs/spec/functional-spec.md) · Research: [docs/research](docs/research)
* Legal: fully clean-room, see [CLEAN_ROOM.md](CLEAN_ROOM.md). Licence: [MIT](LICENSE).

## Workspace

| Crate | Purpose |
|-------|---------|
| `leoht-core` | Domain model (photos, folders, albums, edit recipes). No I/O, UI-agnostic. |
| `leoht-catalog` | SQLite index (rebuildable cache) via `rusqlite`. |
| `leoht-scan` | Folder walking, content checks, metadata (EXIF, header), damage records, hashing, optional fs watching. |
| `leoht-library` | A library on disk: config, cache location, incremental scan into the catalog, rebuild, thumbnails, verify (ADR-0006). |
| `leoht-edit` | Sidecar persistence now; pixel edit pipeline in Phase 3. |
| `leoht-thumb` | Thumbnail rendering (decode or embedded preview, orientation, decode limits) and the content-addressed cache layout. |
| `leoht-cli` | `leoht` CLI: `scan`, `list`, `thumbs`, `verify`, `rebuild`, `info`, with `--json` (contract in [`docs/cli.md`](docs/cli.md)). |
| `leoht-testkit` | Dev-only deterministic synthetic library generator and fixture builders. |
| `leoht-ui` | *Phase 2, not yet created:* egui front-end. |

## Develop

```sh
cargo xtask ci                      # fmt, clippy -D warnings, tests, docs, deny, size and trial checks
cargo xtask gen-library --count 1000 --out target/volume-library
cargo run -p leoht-cli -- scan <folder>      # then: list, thumbs, verify (see docs/cli.md)
cargo xtask bless                   # re-record golden images (review the diff!)
```

The toolchain is pinned in `rust-toolchain.toml`; `rustup` installs it automatically.
