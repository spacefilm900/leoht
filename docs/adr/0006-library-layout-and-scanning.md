# ADR-0006: Library layout and scanning rules (Phase 1)

* Status: Proposed
* Date: 2026-11-28
* Deciders: Keith Haviland (owner), Leoht Master (coordinator)

## Context and problem statement
Phase 1 builds the scanner and the catalog index (FR-LIB-01 to FR-LIB-10, FR-FND-01, FR-FND-02, FR-FND-10).
Several choices shape every later phase and the trials: where the catalog and thumbnails live, how a user excludes
folders and file types, which entries a scan lists, how a rescan decides that a file changed, and how damaged files
are recorded. ADR-0001 fixed that the catalog is a rebuildable cache; it did not fix where that cache lives.

## Decision drivers
* FR-FND-01: scanning and thumbnailing must never write inside the photo folders (and must work on read-only libraries).
* FR-FND-02 / FR-LIB-08: deleting the cache loses nothing; rebuild is the normal recovery path.
* T9 (ease of trial): removing a trial leaves at most one clearly named cache folder.
* FR-LIB-01 judge check: the listing should match what Keith sees in Finder.
* FR-LIB-03 / FR-LIB-05: exclusions and file-type choices are a *visible* config file that travels with the library.

## Considered options
1. Cache inside the library (a hidden `.leoht/` folder), config in `.leohtignore`.
2. Cache in the per-user cache folder, config as a visible JSON file at the library root. **(chosen)**
3. Cache next to the application.

## Decision

### Key decisions
* **Cache location.** Each library's catalog (`catalog.sqlite`) and thumbnails (`thumbs/`) live in
  `<cache root>/libraries/<folder name>-<16 hex>/`, where the hex is a BLAKE3 hash of the library's canonical path. The
  cache root is the CLI `--cache-dir`, else `LEOHT_CACHE_DIR`, else `~/Library/Caches/Leoht` (macOS),
  `$XDG_CACHE_HOME/leoht` or `~/.cache/leoht` (Linux), `%LOCALAPPDATA%\Leoht` (Windows). Each cache folder holds a
  `library-root.txt` naming its library. A cache inside the library is refused. Option 1 was rejected because it writes
  into the user's folders (FR-FND-01), fails on read-only volumes, and syncs a disposable database through cloud folders.
* **Library config.** `leoht-library.json` at the library root, with `version`, `ignore` (gitignore syntax, relative to
  the root) and `formats` (stable format names switched on or off). A config that exists but cannot be understood stops the
  scan with a clear message instead of silently scanning excluded folders. The Phase 0 `.leohtignore` file is retired.
  Leoht only ever reads this file; the user (or, from Phase 5, the preferences window) writes it.
* **What a scan lists.** Files with a recognised extension whose format is switched on. Hidden files and folders (name
  starting with `.`) are skipped, as Finder does; this also drops macOS `._*` resource files. Symbolic links are not
  followed and are reported as skipped (no loops, nothing outside the library). Ignored folders are pruned before they are
  opened. Non-UTF-8 names are reported as skipped. `.gitignore` files are not honoured.
* **Content check and damage.** Each file's first 128 bytes must match its extension's signature; JPEG and PNG headers
  are parsed (size, ICC profile) without decoding pixels. Empty files, mismatched content, unreadable headers and images
  over 400 megapixels are stored as damaged entries with a reason (`DamageKind`), never skipped. Pixel-decode failures are
  recorded by the thumbnailer.
* **Incremental rescan.** A file is unchanged when its size and modification time (seconds and nanoseconds) match the
  catalog; unchanged files are not opened. A `probe_version` fact in the catalog forces a full re-probe when the probing
  rules change. Rows for vanished files are removed. New and changed rows are committed in batches of 500 so readers see
  results while a scan runs (FR-LIB-09).
* **Metadata.** EXIF via `kamadak-exif` (bounded reads: TIFF-based files read at most 16 MiB; RAF via its embedded JPEG);
  capture date falls back to the file modification time (UTC), marked `file_modified`. Values are kept as the file
  states them (EXIF rationals stay rationals). CR3 and HEIC metadata depend on the container: HEIC EXIF is read by
  `kamadak-exif`'s HEIF parser; CR3 is detection-only in Phase 1.

## Consequences
* Good: the library folder is never written by scanning; read-only and network libraries work; one cache folder to delete.
* Good: the config travels with the library and is readable by people and agents.
* Bad: moving a library to a new path starts a new cache (rescan cost only; nothing user-authored is lost).
* Bad: a user who wants hidden folders scanned cannot do so yet (add a config switch if anyone asks).
* Follow-ups: FR-FND-04 (Phase 3) must document `leoht-library.json` and sidecars as the library's own files; the
  Phase 5 preferences window edits `leoht-library.json`; revisit symlink policy if users keep libraries behind links.
