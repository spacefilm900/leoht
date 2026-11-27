# ADR-0002: Sidecar format: `.leoht.json`

* Status: Proposed
* Date: 2026-11-26
* Deciders: Keith Haviland (owner), Leoht Master (coordinator)

## Context and problem statement
Edits must be non-destructive and must travel with the photos. `LEOHT_PLAN.md` allows "sidecar JSON or
lightweight DB" and "`.leoht.json` or XMP-compatible". We need to pick the canonical store and the file layout
before Phase 3 builds the edit engine, because the format becomes a public, long-lived contract.

## Decision drivers
* Folders stay portable: copy a folder and its edits come along.
* A crash must never corrupt an edit or an original.
* Human-readable and diff-friendly (users may sync with git, Dropbox, iCloud, Syncthing).
* Forward compatibility: an older Leoht must not destroy data written by a newer one.
* Interoperability with other software is desirable but must not constrain our model.

## Considered options
1. Database only (edits in SQLite).
2. XMP sidecars as the primary format (`photo.xmp`).
3. One JSON sidecar per image, with XMP as an export/interop path. **(chosen)**
4. One JSON file per folder (Picasa `.picasa.ini` style).

## Decision

### Key decisions
* **One sidecar per image, named `<full original file name>.leoht.json`** in the same folder
  (`IMG_0001.jpg` → `IMG_0001.jpg.leoht.json`). The full name avoids `a.jpg`/`a.png` collisions. A sidecar exists
  only when there is something to store. Per-image (not per-folder) files move and sync with their photo and
  never create a single hot file that conflicts on merge.
* **The sidecar is canonical for edits; the database stores an index only.** If they disagree the sidecar wins and
  the catalog is rebuilt. Deleting the database never loses an edit.
* **Versioned schema.** Every file has an integer `schema_version` (currently 1). A reader refuses a *newer* version
  with a clear error (never overwrites it), preserves unknown top-level fields on write, and treats
  additive optional *top-level* fields as non-breaking. Adding a field **inside** `recipe.ops` (or any nested object) is
  *not* non-breaking: this version cannot preserve unknown nested fields, so such an addition must bump
  `schema_version`, and an older reader then refuses the file instead of silently dropping data. Breaking changes bump the version and add a migrate-on-read step.
  Contents v1: `schema_version`, optional `source_hash` (BLAKE3 of the original, for re-association after a rename),
  and `recipe.ops` (see ADR-0001).
* **Safe and durable writes.** Validate the recipe before writing and after reading. Create a *uniquely named* temp
  file in the same folder with exclusive creation (`create_new`; never reuse a fixed name, never overwrite or follow a
  link onto an existing file, which also keeps concurrent writers apart). Write it, `sync_all` the file, rename it over
  the target, then sync the parent directory so the rename survives a power cut (Unix; Windows cannot open a directory
  for syncing, and NTFS journals the rename). Sync failures are errors, not ignored. Remove only the temp file we
  created if a step fails. The original image bytes are never touched (asserted by the end-to-end test).
* **XMP is an export/interop path, not the primary store.** An `xmp` export (and later import of basic
  XMP fields such as rating and keywords) will be provided for other tools, but XMP cannot express Leoht's recipe
  faithfully, is XML-heavy, and would tie our evolution to someone else's schema.

## Consequences
* Good: portable, inspectable, diffable edits; DB is disposable; forward-compatible by construction.
* Good: JSON is trivial to read from any front-end (egui now; Tauri/SwiftUI later).
* Bad: more small files in photo folders (mitigated: only for edited photos; cloud-sync clients handle small files well).
* Bad: sidecars can be orphaned by external renames/deletes; a "repair" scan using `source_hash` is a Phase 3 task.
* Note: the plan's edit list is open-ended ("crop, rotate, …, red-eye, etc."). Adding a new operation changes what a reader must understand, so it bumps `schema_version`; an older reader then refuses the file instead of silently dropping an edit.
* Follow-up: Phase 4 decides whether stars/captions/tags live in this same sidecar (likely, as optional fields) —
  the unknown-field preservation rule already makes that safe to add.
