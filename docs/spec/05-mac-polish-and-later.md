# Spec 05: Mac polish and sharing (FR-MAC) and Later phases (FR-LAT)

FR-MAC is Phase 5; FR-LAT is Phase 6 and later. Trial: **Gt-MAC**; later groups get their own trials at phase entry.
Items marked *Widening* start only after gate Gf confirms the focused core meets its quality bar (Core Principle 6).
Vocabulary: [`functional-spec.md`](functional-spec.md#2-how-to-read-this-specification).

## Mac polish and sharing (FR-MAC), Phase 5

### FR-MAC-01 Native app shell
*Phase 5 · Core · Traces: META-05, META-07*
A proper Mac app: menu bar with the standard items, native window behaviour (full screen, tabs of the system where relevant, restore state), drag and drop of folders onto the window and Dock icon, and standard shortcuts.
- **Auto** (`ui`, `unit`): the menu and shortcut table is tested for completeness against a checklist; dropped folders open as libraries.
- **Judge** (Gt-MAC): a "does it behave like a Mac app?" checklist of ten behaviours (undo menu, Command-comma, window restore and so on); pass is at least 9 of 10, with each miss an issue. (The inventory flags Mac quirks of Picasa itself as unsourced; we use this list, not those.)

### FR-MAC-02 Preferences
*Phase 5 · Core · Traces: META-05*
One preferences window (Command-comma) holds all settings (file types, cache location, appearance-related options, keyboard shortcuts view); changes apply immediately and are stored in one visible file.
- **Auto** (`integration`): every setting persists, resets to default, and the file is documented.

### FR-MAC-03 Packaging: one download, signed and notarised
*Phase 5 · Foundation · Traces: Plan Phase 5, META-07*
Releases and trials ship as a single signed and notarised disk image (or one install command) for Apple Silicon, with no toolchain needed. Any binary that leaves the project and decodes HEIC passes the G-legal review first (ADR-0004).
- **Auto** (`integration`): CI builds the image, verifies the signature and notarisation status, and launches the app headless to a smoke test; the image contains the bundled sample photos for trials.
- **Judge** (Gt-MAC): fresh-account install by following the one-line instructions (the T1 to T10 rehearsal).

### FR-MAC-04 HDR and wide-gamut on HDR displays
*Phase 5 · Core · Traces: ADD-05*
HDR images and HDR-capable (XDR) displays are handled properly: HDR photos show with correct brightness range, SDR photos are not distorted, and edits look the same in the app as in an export.
- **Auto** (`golden`, `unit`): reference values for HDR transfer functions and tone mapping on charts, within stated tolerances. CI cannot verify a real XDR display.
- **Judge** (Gt-MAC, on real hardware, recorded as a human gate): a manual checklist on an XDR display comparing against the system viewer for three HDR samples and three SDR samples.

### FR-MAC-05 Accessibility reassessment
*Phase 5 · Foundation · Traces: MOD-04, FUT-11*
The results of FR-FND-05 and FR-FND-06 on the full app are reviewed against the UI toolkit's limits; if a core flow cannot meet them, a decision on the toolkit (or a native layer) goes to the architecture gate.
- **Judge** (Gt-MAC): a full VoiceOver walkthrough of all core flows, with a pass or fail per flow; the report is attached to the Phase 5 exit.

### FR-MAC-06 Slideshow *(Widening; default pending OQ-4)*
*Phase 5 · Widening · Traces: VIEW-04, OQ-4*
A full-screen slideshow of a folder, album or selection with timing and transition options and keyboard control.
- **Auto** (`ui`): advance timing and key handling; no file is modified.
- **Judge** (Gt-MAC): "run it over the sample album: does it look right and stop when you press Escape?"

### FR-MAC-07 Quick viewer and "Open with Leoht" *(Widening; default pending OQ-4)*
*Phase 5 · Widening · Traces: VIEW-06, LIB-09, OQ-4*
Opening a single photo from Finder shows it instantly in a light viewer without adding it to any library; the app registers for supported file types.
- **Auto** (`perf`, `integration`): launch-to-image budget with a cold app; viewing writes nothing (FR-FND-01).

### FR-MAC-08 Share through the system share sheet *(Widening)*
*Phase 5 · Widening · Traces: OUT-03*
Selected photos can be sent through the macOS share sheet (Mail, Messages, AirDrop) resized to a chosen size; no account integration.
- **Auto** (`integration`): the files handed to the share sheet are correctly resized copies; originals untouched.

## Later phases (FR-LAT), Phase 6 and later

These are *outline* requirements. Each is detailed in its own spec task at phase entry, with a trial per group, and none starts before gate Gf. Acceptance below says only what must eventually be proven.

### FR-LAT-01 Picasa metadata importer (read-only) *(Keith decision: OQ-1, 2026-11-27)*
*Phase 6 · Widening · Traces: OQ-1 (resolved), sources S44-S48 for context only*
An optional importer reads the per-folder metadata that Picasa left in old libraries and copies what it can (for example ratings and captions) into Leoht sidecars. It is **read-only**: it never writes, changes or deletes Picasa's files. It is built only from public third-party descriptions, and **work on it must not start until a clean-room and legal review is complete and its gate (G-import) is closed**. It is Phase 6 or later and removable without affecting the rest of Leoht.
- **Auto** (`integration`): hash test shows every source file unchanged; import is idempotent; a dry run lists what would be imported; unknown fields are reported, not guessed.
- **Judge** (Gt-LAT-import): Keith runs it on a *copy* of an old library, if he has one, and checks a sample.
- **Note:** no Picasa file-format details are recorded in this repository today, deliberately (inventory clean-room statement).

### FR-LAT-02 Minimal import from camera or card *(default pending OQ-3)*
*Phase 6 · Widening · Traces: IMP-01, IMP-02, IMP-03, IMP-04, OQ-3*
Copy photos from a card or camera into dated folders (configurable and previewed), skip files already in the library by content hash, and offer to clear the card only after the copies are verified by hash and the user confirms.
- **Auto** (`integration`): hash-verified copy; duplicates skipped; the card-clearing step cannot run if any copy fails verification (fault injection).

### FR-LAT-03 Faces: on-device detection, opt-in
*Phase 6 · Widening · Traces: FACE-01, LIB-15, MOD-07*
Face detection and grouping run only on the device, only when the user opts in per library and per folder, with all data deletable and no network use. Needs gate Gx-privacy.
- **Auto** (`integration`): with the option off, no detection runs and no data exists; delete removes all face data; a full detection run with traffic routed to a recording sink makes no connection attempt (FR-FND-08).

### FR-LAT-04 Names and people albums
*Phase 6 · Widening · Traces: FACE-02, FACE-03*
Name faces, and see a person's photos as an album built on the album model.
- **Auto** (`integration`): naming updates the person's album; deleting a name removes it everywhere.

### FR-LAT-05 Where face data lives
*Phase 6 · Widening · Traces: FACE-04*
Face data is stored in sidecars by default; it is written into original files only on an explicit opt-in because it exposes names.
- **Auto** (`integration`): FR-FND-01 hash test passes with faces on.

### FR-LAT-06 Places: map view and coordinates
*Phase 6 · Widening · Traces: GEO-01, GEO-02, GEO-03*
Show geotagged photos on a map (open map source, with an offline option), set coordinates by hand or from a track, stored in sidecars; writing coordinates into files only through FR-RAT-07's opt-in.
- **Auto** (`integration`): coordinates round-trip; no network request unless the user opens the map and chooses an online source.

### FR-LAT-07 Batch edit
*Phase 6 · Widening · Traces: EDIT-24*
Apply a set of operations to many photos with progress, cancel and a single undo.
- **Auto** (`integration`, `perf`): same assertions as FR-ADJ-08 over larger sets.

### FR-LAT-08 Batch rename and adjust date and time
*Phase 6 · Widening · Traces: ORG-13, ORG-14*
Pattern rename with a preview and persistent undo; time-shift photos from a wrong camera clock, stored as a sidecar offset and written to files only on consent.
- **Auto** (`property`, `integration`): preview equals result; undo restores names exactly; offset arithmetic is exact across time zones and day boundaries.

### FR-LAT-09 Watermark on export
*Phase 6 · Widening · Traces: OUT-02*
Optional text or image watermark on export, with position, opacity and size.
- **Auto** (`golden`): watermark placement matches references.

### FR-LAT-10 Local adjustments and masks
*Phase 6 · Widening · Traces: EDIT-14*
General radial and graduated masks that carry any adjustment.
- **Auto** (`golden`): mask geometry and feathering match references.

### FR-LAT-11 Backup and verify
*Phase 6 · Widening · Traces: OUT-08*
A verify command that confirms a folder copy contains every photo, sidecar and library file, and reports what it cannot protect (such as album membership until FR-ALB-03's storage is final).
- **Auto** (`integration`): a deliberately incomplete copy is reported correctly.

### FR-LAT-12 Best-shot suggestion
*Phase 6 · Widening · Traces: ADD-04*
Suggest the best photo of a stack from measurable qualities (sharpness, exposure), with the user's choice always winning.
- **Auto** (`unit`): deterministic scoring on fixtures; override persists.
- **Judge** (Gt-LAT-stacks): "does the suggested pick match yours?" counted over 20 stacks (proposed pass: at least 70%).

### FR-LAT-13 Large-library proof
*Phase 6 · Foundation · Traces: MOD-12, FUT-05*
The 100k column of FR-FND-07 is met on the CI Mac in the scheduled volume job, and the grid stays at 60 fps in Keith's judgement on a real 100k-photo library.
- **Auto** (`volume`, `perf`): scheduled job on a deterministic 100k synthetic library.
- **Judge** (Gt-LAT-scale): Keith scrolls his own large library and answers fast, OK or slow. The trial build also records frame times during the session (a log the checklist asks him to send), so the 60 fps target is measured as well as judged.

### FR-LAT-14 Linux and Windows builds
*Phase 6 · Foundation · Traces: META-07, Plan Phase 6*
Linux and Windows builds are produced and tested in CI and become required checks once stable.
- **Auto:** CI matrix; smoke tests on both.
