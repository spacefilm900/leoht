# ADR-0001: Data model: photos, folders, albums, edit recipes

* Status: Proposed
* Date: 2026-11-26
* Deciders: Keith Haviland (owner), Leoht Master (coordinator)

## Context and problem statement
`LEOHT_PLAN.md` fixes the principle "the filesystem is the source of truth" and asks Phase 0 to define the
data model for **photos, folders, albums (as virtual collections) and edit recipes**. Everything else — scanner,
catalog, thumbnails, UI, edit engine — builds on these shapes, so they must be small, stable and free of any
UI or storage assumption (the UI starts as egui in Phases 1–2 and may later become Tauri or native SwiftUI over FFI).

## Decision drivers
* A library folder must be movable, syncable and usable on another machine with nothing lost.
* The database must be disposable: deleting it may cost time, never data.
* The model must work equally for a 1k and a 100k photo library.
* `leoht-core` must be reusable from any front-end.

## Considered options
1. Database as the source of truth (Lightroom-style catalogue).
2. Opaque per-photo IDs (UUIDs) assigned by Leoht.
3. Folder-first: identity is the path; the hash is an optional re-association aid. **(chosen)**

## Decision

### Key decisions
* **Photos and folders are derived from the filesystem.** A `Photo`'s identity is its *library-relative path*
  (`RelPath`: UTF-8, forward slashes, no `..`, never absolute), with an optional BLAKE3 `ContentHash`
  (`blake3:<64 hex>`) used to re-find a photo that was renamed or moved. A folder is just the parent path of
  photos — it is not a separately stored entity.
* **Albums are virtual collections.** An `Album` is an id, a name and an ordered list of `AlbumMember`s
  (`path` + optional `hash`). Albums never move or copy files; a photo can be in many albums. Where they are
  persisted (planned: library-level `.leoht/albums/<id>.json`, so they travel with the library) is finalised in
  Phase 4; Phase 0 fixes only the shape.
* **Edit recipes are ordered lists of non-destructive operations** applied to the untouched original. The Phase 3 operation
  set is fixed now: `crop`, `rotate` (quarter turns), `straighten`, `exposure`, `contrast`, `saturation`,
  `red_eye`. Geometry uses normalised 0–1 coordinates so a recipe survives re-export at another resolution.
  Parameters are range-validated (finite, bounded); invalid recipes are rejected on read and write.
* **The catalog is a rebuildable cache, accessed with `rusqlite` (bundled SQLite).** It stores one row per photo
  plus derived data (thumbnail keys, EXIF summary later). It holds no user-authored data that is not also in a
  sidecar or library file (see ADR-0002). `rusqlite` is chosen over `sqlx` because the database is embedded and
  local: a synchronous API is simpler and fast enough, no async runtime is needed, there is no server or
  compile-time-checked-query tooling to maintain, and `bundled` gives identical SQLite behaviour on macOS,
  Linux and Windows.
* **`leoht-core` is I/O-free and UI-agnostic.** It defines the types above and their validation only: no
  filesystem, database, async runtime or UI-toolkit dependency. The egui front-end (a `leoht-ui` crate, created in
  Phase 2 and *not* part of the Phase 0 build) and any later Tauri/SwiftUI-over-FFI front-end consume it unchanged.

## Consequences
* Good: libraries are portable; catalog loss is harmless; the model is tiny and easy to test (property tests, no mocks).
* Good: no migration burden for user data — only the cache has migrations.
* Bad: path identity means an external rename breaks the link until a re-scan re-associates by hash;
  hashing costs I/O, so it is optional and computed lazily.
* Bad: albums-as-files need conflict handling if two machines edit them (deferred to Phase 4).
* Follow-up: ADR for face/AI metadata (Phase 6+) must state where it lives relative to this model.
