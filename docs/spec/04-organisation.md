# Spec 04: Rate, cull, caption and tag (FR-RAT), Albums, timeline and search (FR-ALB), Files, folders, duplicates and stacks (FR-FIL)

Phase 4 (the culling viewer starts in Phase 2). Trials: **Gt-RAT**, **Gt-ALB**, **Gt-FIL**. This is the "organise" half of the
focused core. Vocabulary: [`functional-spec.md`](functional-spec.md#2-how-to-read-this-specification).

## Rate, cull, caption and tag (FR-RAT)

### FR-RAT-01 Star ratings, 0 to 5
*Phase 4 · Core · Traces: ORG-06, SQ-4 (Keith decision, 2026-11-27)*
The rating model is 0 to 5 stars plus a reject flag (FR-RAT-02), set independently. One key sets each rating; a filter shows photos at or above a rating. Ratings are stored in the sidecar.
- **Auto** (`ui`, `integration`): key sequences set the expected ratings; a rating survives deleting the database (FR-FND-02).

### FR-RAT-02 Culling mode
*Phase 4 · Core · Traces: ADD-02*
In the full-screen viewer (FR-BRW-09) single keys set a star rating, reject or clear a photo, with optional auto-advance after a decision. Decisions are saved to sidecar data, never by touching or moving files, and survive a crash.
- **Auto** (`ui`, `perf`, `integration`): `egui_kittest` key sequences assert the resulting ratings and flags; next-image latency stays within budget; atomic-write test; kill-and-restart test that decisions survive.
- **Judge** (Gt-RAT): cull the 60-photo sample shoot to a 10-photo selection; record time taken, any missed or wrong keypress, and "would you use this for a real shoot?" (yes, maybe, no).

### FR-RAT-03 Captions, single and batch
*Phase 4 · Core · Traces: ORG-07*
Type a caption under a photo, or apply one to a whole selection in one step (the gap users noticed in the original [S22]). Captions show in the grid when chosen.
- **Auto** (`ui`, `integration`): single and batch captions persist in sidecars; one undo reverts a batch; Unicode captions round-trip.

### FR-RAT-04 Tags, single and batch
*Phase 4 · Core · Traces: ORG-08*
Add and remove tags singly or on a selection, with quick-tag buttons and autocomplete from existing tags, for every supported format.
- **Auto** (`ui`, `integration`, `property`): tag set operations (add, remove, merge) obey set semantics; tags persist in sidecars for JPEG, HEIC and PNG alike.

### FR-RAT-05 Sidecar is the default home for metadata
*Phase 4 · Foundation · Traces: META-01, OQ-2*
Ratings, flags, captions and tags are stored in the sidecar by default; original files are not changed to hold them. This is the recommended default for OQ-2 and is applied until Keith decides.
- **Auto** (`integration`): the FR-FND-01 hash test passes through every metadata operation.

### FR-RAT-06 Interchange with other tools (XMP)
*Phase 4 · Foundation · Traces: MOD-11, EDIT-18*
Leoht reads rating and keywords from existing XMP sidecars and exports its own rating, caption and keywords as XMP for other tools. Export creates new `.xmp` files only.
- **Auto** (`integration`, `snapshot`): fixture XMP files from several tools import correctly; exported XMP matches reference snapshots and is accepted by a reference parser.

### FR-RAT-07 Optional write-back into the file (opt-in)
*Phase 6 · Widening · Traces: META-01, GEO-02, OQ-2*
An explicit, warned action writes selected metadata into the original file's own metadata, never by default, with a confirmation that names the files and a backup. Under the recommended default for OQ-2 this ships after the focused core, and it is not part of the Gt-RAT trial: it gets its own trial after Gf.
- **Auto** (`integration`): without the action no original changes; with it, only the chosen fields change and a backup exists; the file stays valid.

### FR-RAT-08 Selection basket
*Phase 4 · Core · Traces: ORG-03*
A persistent basket collects photos from any folders for batch actions (rate, tag, export, make an album); it survives restarts and can be saved as an album.
- **Auto** (`integration`, `ui`): basket contents persist across restart; save-as-album creates an album with the same members in order.

## Albums, timeline and search (FR-ALB)

### FR-ALB-01 Albums are virtual
*Phase 4 · Core · Traces: ORG-01, ADR-0001*
A photo lives in one folder and can be in many albums; albums never move or copy files.
- **Auto** (`property`, `integration`): adding and removing members never changes any file (FR-FND-01); a photo can be in N albums.

### FR-ALB-02 Create, edit and remove albums
*Phase 4 · Core · Traces: ORG-02*
Create an album from a selection, rename it, add and remove photos, delete the album; deleting an album or removing a photo from it leaves the files alone.
- **Auto** (`ui`, `integration`): operations produce the expected membership with files untouched.
- **Judge** (Gt-ALB): make an album from three folders; "was it obvious how?"

### FR-ALB-03 Albums survive and travel
*Phase 4 · Foundation · Traces: ORG-05, ADR-0001*
Albums are stored in visible library-level files (the shape from ADR-0001; final location per ADR-0001's Phase 4 decision), so they survive deleting the database and move with the library; members are re-found by path, then by hash after renames.
- **Auto** (`integration`): delete the database and albums are intact; rename a member outside the app and re-association by hash finds it.

### FR-ALB-04 Manual order
*Phase 4 · Core · Traces: ORG-04*
Drag to reorder photos in an album, and optionally in a folder; custom order is stored durably (album file, or a sort key in sidecars) and the index may only cache it.
- **Auto** (`integration`): order survives deleting the database and rescanning.

### FR-ALB-05 Timeline
*Phase 4 · Core · Traces: ORG-09*
A date-ordered view groups the whole library by year, month and day from capture date, scrolling smoothly at any size.
- **Auto** (`integration`, `perf`, `volume`): grouping on fixtures with known dates (including missing and time-zone edge cases); scroll budgets from FR-FND-07.
- **Judge** (Gt-ALB): "can you find photos from last summer without folders?"

### FR-ALB-06 Live search
*Phase 4 · Core · Traces: ORG-10, MOD-12*
A search box filters as you type across folder names, file names, tags, captions and metadata fields (camera, lens, date); every term must match and partial words match from the start of a word.
- **Auto** (`unit`, `perf`, `volume`): query semantics on a labelled fixture set; per-keystroke budget on 1k and 10k (FR-FND-07); 100k in the scheduled volume job.
- **Judge** (Gt-ALB): five search tasks ("photos of Anna with a 50mm lens in 2024"); record found or not found and time.

### FR-ALB-07 Saved searches
*Phase 4 · Core · Traces: ORG-10*
A search can be saved as a smart album that updates itself and is stored in a library file.
- **Auto** (`integration`): saved search results track added and removed photos; it survives deleting the database.

## Files, folders, duplicates and stacks (FR-FIL)

### FR-FIL-01 Hide photos and folders
*Phase 4 · Core · Traces: LIB-05*
Hide photos or folders from the library and un-hide them later; nothing is renamed or deleted. The flag lives in sidecars or library config.
- **Auto** (`integration`): hidden items vanish from grid, search and timeline and return on un-hide; files untouched.

### FR-FIL-02 Folder operations are explicit and keep sidecars together
*Phase 4 · Core · Traces: LIB-07, FUT-09*
Renaming, moving and deleting folders, and moving photos between folders, are explicit and confirmed; a moved or renamed photo takes its sidecar with it, and album references stay valid. Undo is offered where the filesystem allows it. (This promotes inventory item FUT-09 because without it a move would orphan edits.)
- **Auto** (`integration`, `property`): after any move or rename, sidecar, albums and ratings are intact; failure mid-move leaves a consistent state (test with injected failure).
- **Judge** (Gt-FIL): rename a folder and move five photos; "did anything go missing?" (no is the pass).

### FR-FIL-03 Delete means Trash, with confirmation
*Phase 4 · Core · Traces: LIB-08, MOD-08*
Deleting a photo sends it and its sidecar to the system Trash after a confirmation; the app never unlinks permanently.
- **Auto** (`integration`): no code path calls permanent delete (checked by a lint-style test); Trash restore returns both files and the library reattaches them.

### FR-FIL-04 Removable and offline volumes
*Phase 4 · Foundation · Traces: LIB-02, MOD-10*
Removable and network volumes are remembered by identity; when absent their photos show as offline and nothing is removed from the index or sidecars.
- **Auto** (`integration`): simulate a volume disappearing and returning: catalog entries persist, offline state is shown, no data loss.

### FR-FIL-05 Cloud-synced folders
*Phase 4 · Foundation · Traces: MOD-09*
Leoht detects sync-conflict copies of sidecars and library files (as created by common sync tools), warns, and never merges them silently.
- **Auto** (`integration`): fixtures with conflict-copy names are detected and listed; originals and the current sidecar stay untouched.

### FR-FIL-06 Exact duplicate detection
*Phase 4 · Core · Traces: META-02*
Photos with identical content (BLAKE3 hash) are listed as duplicate sets; the user chooses what to hide or move to Trash. Near-duplicates are handled by stacks (FR-FIL-08).
- **Auto** (`integration`, `volume`): known duplicate sets in a fixture are found with no false positives; hashing 10k files meets a time budget set from the Phase 1 baseline.

### FR-FIL-07 Related-file stacks
*Phase 4 · Core · Traces: MOD-14, ADD-04*
The data model supports stacks: a group of items shown as one cover with the rest beneath (RAW plus JPEG pair, burst). Files stay where they are. The user's choices (cover, manual stack changes) are stored in sidecar data; the index only caches the derived stack view.
- **Auto** (`integration`): override persistence test: delete the database, rescan, and every override is still in effect (FR-FND-02).

### FR-FIL-08 Burst and similar-photo grouping
*Phase 4 · Core · Traces: ADD-04*
Bursts and near-identical shots are grouped automatically from capture time and image similarity, with a cover shown and the user able to split, merge and override.
- **Auto** (`unit`, `volume`): labelled fixture libraries meet precision and recall targets (proposed: 95% precision, 85% recall; fixed in the Phase 4 spec task); grouping a 100k synthetic library meets a time budget in the volume job.
- **Judge** (Gt-FIL): on a real-looking shoot of 200 frames, "do the groups make sense?" (yes, mostly, no) and a count of obvious mistakes.
