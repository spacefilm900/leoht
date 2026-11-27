# Spec 02: Library, scan and index (FR-LIB) and Browse and view (FR-BRW)

Phases 1 and 2. Trials: **Gt-LIB** (CLI, at Phase 1 exit) and **Gt-BRW** (first graphical trial).
Vocabulary: [`functional-spec.md`](functional-spec.md#2-how-to-read-this-specification).

## Library, scan and index (FR-LIB), Phase 1

### FR-LIB-01 Open a folder as a library and scan it
*Phase 1 · Foundation · Traces: LIB-01, LIB-10*
The user points Leoht at a folder; it walks it recursively, lists every supported photo with path, size and dates, and
stores that in the catalog. Folders on disk remain the organisation (Core Principle 1).
- **Auto** (`integration`): scan of fixture libraries lists exactly the expected files (nested folders, symlinks, hidden files, odd names).
- **Judge** (Gt-LIB): Keith scans the bundled sample folder with one command and the output matches what he sees in Finder.

### FR-LIB-02 Changes on disk appear without a manual refresh
*Phase 1 · Foundation · Traces: LIB-01*
Added, removed and renamed files appear or vanish within 5 seconds using file-change events, with a periodic reconcile scan
as a safety net (events can be missed).
- **Auto** (`integration`): tests add, delete and rename files in a watched fixture and assert catalog state within 5 s; a test drops events and asserts the reconcile scan repairs the catalog.

### FR-LIB-03 Ignore folders
*Phase 1 · Foundation · Traces: LIB-03*
The user can exclude folders or whole drives (backups, system folders). Excluded folders are skipped by scans and are never touched. The list is a visible config file in the library.
- **Auto** (`integration`): an ignored folder never appears in the catalog and is never opened for reading after the rule is added.

### FR-LIB-04 Supported formats and support levels
*Phase 1 · Foundation · Traces: LIB-13, MOD-01, MOD-02, EDIT-23*
Support follows the format matrix in ADR-0004: JPEG and PNG full; HEIC/HEIF first-class through a swappable decoder; RAW (common makers) detected with metadata and embedded preview, full decode later (FR-ENG-10); other types (TIFF, WebP and so on) detected only, shown as a placeholder until supported. The support level of each format is machine-readable and tested.
- **Auto** (`unit`, `integration`): `PhotoFormat::support()` is pinned by a test; a corpus of small, licensed sample files per format is decoded or detected as the matrix says.
- **Gate:** any shipped binary that decodes HEIC needs the G-legal review first (ADR-0004). **Trial builds (Gt) exclude HEIC decoding and ship JPEG and PNG samples only** (Keith, 2026-11-27, SQ-1; revisit after the review).
- **Auto** (`integration`): the trial artifact is built without the HEIC feature and contains no HEIC sample files (checked by a CI step on the trial build).

### FR-LIB-05 Choose which file types to show
*Phase 1 · Foundation · Traces: LIB-13*
The user can switch file types on or off; the choice is stored in the library config and applies to scan and grid.
- **Auto** (`integration`): turning RAW off removes RAW items from the listing and turning it on restores them, with no file touched.

### FR-LIB-06 Metadata extraction
*Phase 1 · Foundation · Traces: VIEW-03, ORG-09*
Capture date, camera, lens, exposure settings, dimensions, orientation and colour profile are read from the file (EXIF and equivalents) and stored in the catalog. Capture date falls back to file date, marked as such.
- **Auto** (`unit`, `integration`): fixture files with known EXIF give the expected values; missing or broken EXIF never fails a scan; a golden JSON of extracted fields guards against drift (`insta`).

### FR-LIB-07 Cached thumbnails
*Phase 1 · Foundation · Traces: LIB-10*
Thumbnails are generated lazily and in the background, cached outside the photo folders, correctly oriented, and regenerated when the original changes. A photo with an embedded preview uses it first.
- **Auto** (`golden`, `integration`): thumbnails of fixtures match goldens within tolerance; orientation cases (all eight EXIF values) are correct; changing a file invalidates its thumbnail.
- **Auto** (`perf`, `volume`): FR-FND-07 thumbnail throughput on 1k and 10k libraries.

### FR-LIB-08 Rebuild the index
*Phase 1 · Foundation · Traces: LIB-11*
A single command (and later a menu item) deletes and rebuilds the catalog from the photos and sidecars. It is the normal recovery path and is tested, not a last resort.
- **Auto** (`integration`, `perf`): see FR-FND-02; rebuild time budget on 1k and 10k.

### FR-LIB-09 Browse while indexing; incremental scans
*Phase 1 · Foundation · Traces: MOD-12, FUT-06*
Opening a library never blocks on a full scan: results appear as they are found, a progress indicator shows state, and later scans only process changes.
- **Auto** (`integration`, `volume`): on a 10k library the first 500 items are listable within 2 s of start; a rescan after changing 10 files processes about 10 files (counted).

### FR-LIB-10 Damaged and hostile files never crash the scan
*Phase 1 · Foundation · Traces: ADR-0004 (untrusted input)*
Truncated, corrupt, zero-byte, enormous or malformed files are recorded in the catalog as damaged entries with the reason (never silently skipped), so the viewer can show them (FR-BRW-10); the scan finishes and lists the errors.
- **Auto** (`integration`, `property`): a fuzz corpus (using `cargo-fuzz`) and fixture set of broken files; the scan always terminates with no panic; memory stays bounded; every broken fixture appears as a damaged catalog entry with a reason.

## Browse and view (FR-BRW), Phase 2

### FR-BRW-01 Folder tree mirrors the disk
*Phase 2 · Core · Traces: LIB-04*
A folder list mirrors the real folders (tree or flat), sortable by name, date or size, hiding empty folders, expanding and collapsing from the keyboard.
- **Auto** (`ui`, `integration`): fixture tree renders as expected; sorting orders are correct; empty folders are not listed.
- **Judge** (Gt-BRW): "do the folders look like what you have in Finder?"

### FR-BRW-02 Thumbnail grid
*Phase 2 · Core · Traces: VIEW-01, MOD-12*
A virtualised grid shows thumbnails with a size slider, optional file name and caption under each, and stays smooth at any library size.
- **Auto** (`perf`, `volume`, `ui`): frame and jump budgets from FR-FND-07 on 1k and 10k; only visible items are loaded (counted); snapshot tests at 1x and 2x scale.
- **Judge** (Gt-BRW): scroll fast through the sample library; answer fast, OK or slow, and whether any thumbnail looked wrong.

### FR-BRW-03 Selection and shortcuts
*Phase 2 · Core · Traces: ORG-12, META-05*
Click, Shift-click and Command-click, select all, deselect, first and last, using Mac conventions; the selection is visible and countable. A shortcut sheet lists every key.
- **Auto** (`ui`): scripted selection sequences produce the expected selections; the shortcut sheet is generated from the same table the app uses, so it cannot drift.

### FR-BRW-04 Lightbox
*Phase 2 · Core · Traces: VIEW-02*
Double-click or Return opens a read-only, instant full-size view with next/previous, zoom to fit and 100%, and Escape to close. Viewing never creates edit data.
- **Auto** (`perf`, `ui`): cold-open and preloaded-next budgets from FR-FND-07; viewing leaves no sidecar (FR-FND-01).
- **Judge** (Gt-BRW): "does the photo appear instantly?"; "does zoom feel natural?"

### FR-BRW-05 Properties panel
*Phase 2 · Core · Traces: VIEW-03*
A panel shows file name, folder, size, dimensions, capture date and camera details from FR-LIB-06, and can be shown or hidden.
- **Auto** (`ui`, `snapshot`): panel content for fixtures matches expected fields.

### FR-BRW-06 Correct colour on screen
*Phase 2 · Core · Traces: LIB-17, MOD-05, ADD-05*
Photos with an embedded colour profile, and untagged ones (treated as sRGB), display correctly on the main display, including Display P3 screens and Retina scaling, with no soft or doubled rendering. Colour management is on by default.
- **Auto** (`golden`, `unit`): known colour charts and ICC profiles render to reference values with tolerances stated in a colour space; snapshot tests at 1x and 2x.
- **Judge** (Gt-BRW, on Keith's Mac): side-by-side with the same sample photos in macOS Preview; "does colour look the same?" (same, slightly different, clearly different). Evidence: screenshots in the trial issue.

### FR-BRW-07 Light and dark appearance
*Phase 2 · Core · Traces: MOD-03*
The app follows the system appearance and updates live when it changes.
- **Auto** (`snapshot`): core views are snapshot-tested in both themes; contrast is covered by FR-FND-06.

### FR-BRW-08 Reveal in Finder
*Phase 2 · Core · Traces: LIB-06*
A context action shows the photo or folder in Finder (or the system file manager). Nothing else happens.
- **Auto** (`unit`): the command built for each platform is correct (the launch itself is not run in CI).
- **Judge** (Gt-BRW): "does it open the right Finder window?"

### FR-BRW-09 Full-screen viewer with preloading
*Phase 2 · Core · Traces: ADD-02*
A full-screen mode moves through photos with arrow keys; the next and previous images are preloaded so there is no visible wait. (Rating and rejecting inside it is FR-RAT-02.)
- **Auto** (`perf`, `ui`): preloaded next-image display within the budget on the CI Mac; key sequences land on the expected photo.
- **Judge** (Gt-BRW): hold the right arrow through 100 photos; answer "never waited", "sometimes waited" or "often waited".

### FR-BRW-10 Unsupported and damaged files
*Phase 2 · Foundation · Traces: ADR-0004*
Files of unsupported or damaged type appear as a clear placeholder with the reason, never as a blank tile or a crash.
- **Auto** (`ui`, `snapshot`, `integration`): placeholders for each failure kind; every damaged entry recorded by FR-LIB-10 is shown with its reason.

### FR-BRW-11 Remembered view state
*Phase 2 · Core · Traces: VIEW-01, META-05*
The app restores the last folder, scroll position, thumbnail size and panel state at launch. These are app preferences only.
- **Auto** (`integration`): restart test restores the state; deleting preferences gives sane defaults.
