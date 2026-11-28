# The `leoht` command line

The CLI is Leoht's scriptable core (FR-FND-09): everything the app does is
available here, for Keith, for agents and for tests. It is a **contract**:
output fields and exit codes change only in a reviewed PR that updates the
snapshot tests in `crates/leoht-cli/tests/snapshots/`.

Leoht never changes, moves or deletes photos, and makes no network
connections (FR-FND-01, FR-FND-08). Its catalog and thumbnails live in a
cache folder **outside** the library (ADR-0006); deleting that folder loses
nothing.

## Commands

| Command | What it does | Writes |
|---|---|---|
| `leoht scan <ROOT>` | Brings the catalog up to date with the folder, reading only new and changed files. | catalog |
| `leoht list <ROOT> [--damaged] [--format F]` | Lists the photos in the catalog with their metadata. Needs a prior `scan`. | nothing |
| `leoht thumbs <ROOT> [--size N] [--workers N]` | Makes missing thumbnails (default 256 px long edge). | thumbnail cache, catalog (hashes, damage) |
| `leoht verify <ROOT> [--hashes]` | Compares the catalog with the disk; `--hashes` also re-reads every hashed photo. | nothing |
| `leoht rebuild <ROOT>` | Deletes the catalog and scans from scratch (the normal recovery path). | catalog |
| `leoht info [ROOT]` | Version, platform, cache location, format support; with a root, its cache folder and counts. | nothing (creates the cache folder if missing) |

Global options, accepted anywhere on the line:

* `--json`: print exactly one JSON document on stdout (see below). Without it,
  output is human-readable text and may change freely.
* `--cache-dir DIR`: where catalogs and thumbnails live. Default: the
  `LEOHT_CACHE_DIR` environment variable, else `~/Library/Caches/Leoht` on
  macOS (`$XDG_CACHE_HOME/leoht` or `~/.cache/leoht` on Linux). Refused if it
  is inside the library.

`RUST_LOG=info` (or `debug`) prints diagnostic logs on stderr.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. Damaged photos are reported in the output, not treated as failure. |
| 1 | Error: the folder is missing or unreadable, the library config is invalid, the catalog failed, the library has not been scanned (`list`, `thumbs`, `verify`), or a thumbnail could not be written. |
| 2 | Usage error: unknown command or option, or an out-of-range value. |
| 3 | `verify` only: the catalog is out of step with the disk (files missing, not indexed, changed, or with a different content hash). Run `leoht scan`. |

## JSON output

Every document is an object with:

* `schema_version`: currently `1`. Incremented on any incompatible change
  (renamed or removed field, changed meaning). New fields may be added
  without a bump, so consumers must ignore fields they do not know.
* `command`: the command that produced it (`scan`, `rebuild`, `list`,
  `thumbs`, `verify`, `info`).

On error (exit 1) the document is `{"schema_version": 1, "error": "<message>"}`,
also on stdout; the same message goes to stderr.

Times are milliseconds (`elapsed_ms`). Paths inside the library are
library-relative with `/` separators. Dates follow the catalog: `modified` is
UTC (`...Z`); `meta.capture_time.local` is the camera's local time as recorded
(EXIF has no zone), with `offset` when the camera recorded one, and `source`
saying where it came from (`exif_original`, `exif_digitized`, or
`file_modified` as a fallback).

### `scan` / `rebuild`

```json
{
  "schema_version": 1, "command": "scan",
  "library": { "root": "/Users/keith/Pictures", "cache_dir": "/Users/keith/Library/Caches/Leoht/libraries/Pictures-0123456789abcdef" },
  "summary": {
    "found": 20, "added": 20, "updated": 0, "removed": 0, "unchanged": 0,
    "read": 20, "damaged": 6, "skipped": [], "elapsed_ms": 41
  }
}
```

`read` is the number of files actually opened: 0 on a rescan with no changes.
`skipped` lists entries the walk passed over, each `{"path", "reason": {"kind": "symlink" | "non_utf8_name" | "unreadable", "detail"?}}`.

### `list`

`{"count": N, "photos": [...]}`, sorted by path. Each photo:

```json
{
  "path": "jpeg/full-exif.jpg", "format": "jpeg", "support": "full",
  "size_bytes": 2345, "modified": "2021-03-04T05:06:07Z",
  "hash": "blake3:...", "damage": null,
  "meta": {
    "capture_time": { "local": "2021-03-04T05:06:07", "offset": "+01:00", "source": "exif_original" },
    "camera_make": "...", "camera_model": "...", "lens_model": "...",
    "exposure": { "exposure_time": "1/250", "f_number": "28/10", "iso": 200, "focal_length_mm": "50/1" },
    "width": 64, "height": 48, "orientation": 6,
    "color": { "kind": "icc", "description": "Display P3" }
  }
}
```

* `format`: `jpeg`, `heic`, `png`, `raw`, `tiff`, `webp`; `support` is the
  ADR-0004 level (`full`, `partial`, `detect_only`).
* `hash` is `null` until `thumbs` (or a later feature) has read the file.
* `damage` is `null` or `{"kind", "detail"}` with `kind` one of `empty`,
  `content_mismatch`, `unreadable_header`, `too_large`, `unreadable`,
  `decode_failed`.
* `width`/`height` are as stored; `orientation` (1 to 8) says how to show them.
* `color.kind` is `untagged`, `srgb` or `icc` (with the profile's `description`).

### `thumbs`

```json
{
  "schema_version": 1, "command": "thumbs", "size": 256, "thumbs_dir": "...",
  "summary": {
    "eligible": 10, "generated": 9, "from_preview": 1, "cached": 0,
    "unsupported": 4, "no_preview": 1, "skipped_damaged": 6,
    "newly_damaged": 0, "write_errors": [], "elapsed_ms": 120
  }
}
```

Thumbnails exist for JPEG and PNG (decoded, or from the embedded EXIF preview
when it is at least `size` pixels) and RAW (embedded preview only). HEIC,
TIFF and WebP have none yet; HEIC is never decoded in Phase 1 or trial builds
(ADR-0004, SQ-1). Thumbnails are keyed by content hash
(`<thumbs_dir>/<size>/<2 hex>/<hash>.jpg`), so a changed photo gets a new one.

### `verify`

```json
{
  "schema_version": 1, "command": "verify", "in_sync": false,
  "on_disk": 20, "in_catalog": 20,
  "missing": ["png/plain.png"], "unindexed": ["new.jpg"], "changed": [], "hash_mismatch": [],
  "hashes_checked": 0, "damaged": [{ "path": "damaged/empty.jpg", "damage": { "kind": "empty", "detail": "the file is empty" } }],
  "elapsed_ms": 3
}
```

### `info`

`version`, `os`, `arch`, `cache_root`, `network` (always "none ..."),
`formats` (each `{format, support, decoded}`), and with a root, `library`
(`root`, `cache_dir`, `scanned`, `photos`, `damaged`).
