# Leoht

> **Working title.** "Leoht" is Old English for *light*. Trademark clearance (USPTO, EUIPO, domains, common-law)
> is **pending**; the name and the repository name `leoht` are provisional until a human gate clears them.

A modern, **folder-first**, **non-destructive** photo organiser and light editor in Rust, inspired by the simplicity of
Google Picasa. Mac first, then Linux and Windows. Your folders are the source of truth; edits live in small
`.leoht.json` sidecars next to your photos; the database is only a disposable index.

* Plan: [LEOHT_PLAN.md](LEOHT_PLAN.md) · Status: [STATUS.md](STATUS.md) · Decisions: [docs/adr](docs/adr) · Process: [docs/process](docs/process)
* Legal: fully clean-room, see [CLEAN_ROOM.md](CLEAN_ROOM.md). Licence: [MIT](LICENSE).

## Workspace

| Crate | Purpose |
|-------|---------|
| `leoht-core` | Domain model (photos, folders, albums, edit recipes). No I/O, UI-agnostic. |
| `leoht-catalog` | SQLite index (rebuildable cache) via `rusqlite`. |
| `leoht-scan` | Folder walking, format detection, hashing, EXIF, optional fs watching. |
| `leoht-edit` | Sidecar persistence now; pixel edit pipeline in Phase 3. |
| `leoht-thumb` | Thumbnails and the content-addressed cache layout. |
| `leoht-cli` | `leoht` command-line driver (the Phase 1 demo front-end). |
| `leoht-testkit` | Dev-only deterministic synthetic library generator. |
| `leoht-ui` | *Phase 2, not yet created:* egui front-end. |

## Develop

```sh
cargo xtask ci                      # fmt, clippy -D warnings, tests, file-size check
cargo xtask gen-library --count 1000 --out target/volume-library
cargo run -p leoht-cli -- scan <folder>
cargo xtask bless                   # re-record golden images (review the diff!)
```

The toolchain is pinned in `rust-toolchain.toml`; `rustup` installs it automatically.
