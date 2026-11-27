# Spec 03: Edit engine (FR-ENG), Crop and geometry (FR-CRP), Adjust (FR-ADJ)

Phase 3, the focused core's "crop and adjust". Trials: **Gt-CRP** (ships the engine with crop, geometry and compare, the
first edits anyone tries) and **Gt-ADJ**. Vocabulary: [`functional-spec.md`](functional-spec.md#2-how-to-read-this-specification).

Note for the architecture gate: ADR-0001 fixes the Phase 3 operation set at crop, rotate, straighten, exposure, contrast,
saturation and red-eye. Perspective, flip, highlights and shadows, temperature, sharpen and the look presets below need
new operations, and under ADR-0002 each addition bumps `schema_version`. This spec does not change the ADRs; see
[implications](functional-spec.md#11-implications-for-the-adrs-not-applied).

## Edit engine (FR-ENG)

### FR-ENG-01 Edits are an ordered recipe applied to the untouched original
*Phase 3 · Core · Traces: EDIT-17, EDIT-18*
Every edit is an operation in an ordered recipe. The original is never changed (FR-FND-01); the recipe plus the original reproduces the edited image.
- **Auto** (`unit`, `property`): recipe validation (finite, bounded parameters); applying a recipe twice to the same input gives identical output; reordering operations changes output only where it should.

### FR-ENG-02 Recipes persist in the sidecar
*Phase 3 · Foundation · Traces: EDIT-19, META-04*
Recipes are saved in the photo's `.leoht.json` sidecar (ADR-0002), the single source of truth for edits; the catalog holds only an index.
- **Auto** (`property`, `integration`): sidecar round-trip; delete-the-database test (FR-FND-02); unknown top-level fields preserved on rewrite.

### FR-ENG-03 Unlimited undo and redo, kept across sessions
*Phase 3 · Core · Traces: EDIT-17*
Every change can be undone and redone, in order, and the history of an edited photo survives closing the app.
- **Auto** (`unit`, `integration`): apply N random operations, undo all, and the recipe equals the start; redo restores the end; quit and relaunch keeps the history.
- **Judge** (Gt-CRP): "can you get back to where you started from any point?" (yes or no, with the step where it failed).

### FR-ENG-04 Responsive preview
*Phase 3 · Core · Traces: EDIT-17, Plan Phase 3*
Edits preview on the fly at screen resolution; dragging a control updates within 50 ms (FR-FND-07) and never blocks the interface. A full-resolution render replaces the preview when the interaction stops.
- **Auto** (`perf`): preview update time on a 12-megapixel JPEG on the CI Mac, per operation type.
- **Judge** (Gt-CRP, Gt-ADJ): "does dragging feel direct?" (direct, slightly laggy, laggy).

### FR-ENG-05 Edited badge and reset
*Phase 3 · Core · Traces: EDIT-18*
Edited photos show a badge in the grid and lightbox; "Revert to original" clears the recipe (undoable) without touching the file.
- **Auto** (`ui`, `integration`): badge appears and disappears with the recipe; revert then undo restores the recipe.

### FR-ENG-06 Save a copy; never overwrite
*Phase 3 · Core · Traces: EDIT-20, EDIT-21*
"Save a copy" writes a new file next to the original with a predictable suffix (`name-edit.jpg`, then `-edit-2`) and never overwrites an existing file. v1 offers no in-place overwrite of an original.
- **Auto** (`integration`): suffix rule on collisions; the original is byte-identical afterwards (FR-FND-01); a copy opens correctly in a reference decoder.
- **Judge** (Gt-CRP): "is it obvious where the saved copy went?"

### FR-ENG-07 Export to a folder, with resize
*Phase 3 · Core · Traces: OUT-01, OUT-02*
Export bakes the edits of the selected photos into new files in a chosen folder, optionally resized to a chosen long edge or percentage, with a progress indicator and a cancel.
- **Auto** (`golden`, `integration`): exports of fixture recipes match goldens within stated tolerance; resize preserves aspect ratio and orientation; cancel leaves no partial files.
- **Auto** (`perf`): single export budget (FR-FND-07).

### FR-ENG-08 Output formats and colour profile
*Phase 3 · Core · Traces: EDIT-22, MOD-05*
Export writes JPEG (with a quality setting) and PNG first; TIFF and HEIC output follow later (ADR-0004). The output carries the correct embedded colour profile.
- **Auto** (`golden`, `unit`): the profile in each output matches the working space; colours of a chart image survive a decode round trip within tolerance.

### FR-ENG-09 Deterministic rendering
*Phase 3 · Foundation · Traces: MOD-06, ADR-0003*
The same recipe on the same original gives the same pixels on every machine and run, within a stated tolerance (zero for integer-only operations).
- **Auto** (`golden`): goldens render on the macOS and Linux CI runners from the same references; `cargo xtask bless` is the only way to change a golden, with before and after images in the PR.

### FR-ENG-10 RAW editing (gated on the RAW decoder licence)
*Phase 3 · Widening · Traces: EDIT-23, MOD-02*
Once a RAW decoder is chosen under ADR-0005, RAW files edit non-destructively like any other photo, with the camera's embedded preview as a first display. If no licence-compatible decoder is accepted, this requirement moves to Phase 6 and RAW stays detect-and-preview only.
- **Auto** (`golden`, `integration`): licensed sample RAW files from several makers decode to references within tolerance; the sidecar and original are untouched.
- **Gate:** the decoder choice is an architecture review (Gx-arch) because the obvious Rust RAW crates are LGPL (ADR-0004).

### FR-ENG-11 Orphaned sidecar check
*Phase 3 · Foundation · Traces: FUT-14, ADR-0002*
A command finds sidecars whose photo was renamed or removed outside the app and offers to re-attach (by `source_hash`) or remove them, never silently.
- **Auto** (`integration`): rename a photo outside the app and the check proposes the right match; ambiguous matches are listed, not guessed.

## Crop and geometry (FR-CRP)

### FR-CRP-01 Crop with presets and a custom box
*Phase 3 · Core · Traces: EDIT-01, ADD-01*
Crop by dragging a box, with one-tap ratios (1:1, 4:3, 3:2, 16:9, 5:4, common print sizes, custom) and a lock to the original ratio; the ratio can be rotated between landscape and portrait.
- **Auto** (`property`, `ui`, `golden`): ratio arithmetic (rounding, lock, rotate-then-crop invariants) holds for random inputs; a ratio tap changes the crop box as specified; cropped output matches goldens.
- **Judge** (Gt-CRP): "does cropping feel natural?" (natural, odd, frustrating) and notes on handles and snapping.

### FR-CRP-02 Straighten
*Phase 3 · Core · Traces: EDIT-02, ADD-01*
An angle control with a grid overlay straightens the photo; the crop adjusts automatically so no empty corners appear.
- **Auto** (`golden`, `property`): synthetic grid images straighten to reference within tolerance; no output pixel is outside the source.

### FR-CRP-03 Rotate and flip
*Phase 3 · Core · Traces: EDIT-03*
Rotate in quarter turns and flip horizontally or vertically as recipe operations; the file's own orientation tag is never rewritten.
- **Auto** (`unit`, `golden`): all eight orientation cases render upright; four quarter turns return the original recipe state.

### FR-CRP-04 Perspective correction
*Phase 3 · Core · Traces: ADD-01*
Horizontal and vertical perspective sliders correct converging lines; the result stays inside the source.
- **Auto** (`golden`): chart images with known keystone give the expected result within tolerance.
- **Judge** (Gt-CRP): on the bundled architecture sample, "do vertical lines become vertical?" (yes, nearly, no).

### FR-CRP-05 Auto-straighten suggests, never forces
*Phase 3 · Core · Traces: ADD-01*
Auto-straighten finds the horizon or strong verticals and *proposes* a correction the user accepts, tweaks or rejects; accepting is one undoable step.
- **Auto** (`unit`): a labelled set of tilted horizons and converging verticals reaches an accuracy target (proposed: angle within 0.5 degrees on at least 90% of the set; set and target are fixed in the Phase 3 spec task); undo round trip.
- **Judge** (Gt-CRP): on 10 photos Keith counts "good suggestion", "harmless" or "wrong"; pass is no more than 1 wrong in 10.

### FR-CRP-06 Geometry maths is exact and resolution-independent
*Phase 3 · Foundation · Traces: ADD-01, ADR-0001*
Geometry is stored in normalised 0 to 1 coordinates so a recipe renders the same framing at any resolution; crops never produce zero-size or out-of-bounds results.
- **Auto** (`property`): random recipes render at two resolutions with matching framing within one pixel at the smaller size.

### FR-CRP-07 Compare with the original
*Phase 3 · Core · Traces: VIEW-05*
A key toggles or splits the view between the original and the edit (A versus A) at the same zoom and position.
- **Auto** (`ui`): toggle keeps zoom and position; the displayed original equals the unedited render.
- **Judge** (Gt-CRP): "does the comparison help you judge the edit?"

### FR-CRP-08 Geometry tools feel right in the first ten minutes
*Phase 3 · Core · Traces: ADD-01*
Crop and straighten are the first edits a new user tries, so the whole interaction (handles, snapping, keyboard nudging, undo) is judged as a whole, not only by calculation.
- **Judge** (Gt-CRP): a 10-minute task list ("crop to 16:9, straighten the horizon, undo twice, save a copy") with the question "what annoyed you?"; every annoyance becomes an issue, and pass is no blocker issues left open.

## Adjust (FR-ADJ)

### FR-ADJ-01 Tone controls
*Phase 3 · Core · Traces: EDIT-07*
Exposure, contrast, highlights and shadows (fill light) sliders with a reset on each.
- **Auto** (`golden`, `property`): each operation at known values gives reference output within tolerance; zero value is an exact no-op.

### FR-ADJ-02 Colour: temperature and saturation
*Phase 3 · Core · Traces: EDIT-08, EDIT-11*
A temperature slider and a neutral-colour picker set white balance; a saturation slider adjusts colour strength.
- **Auto** (`golden`): a grey-card fixture becomes neutral after picking it, within tolerance.
- **Judge** (Gt-ADJ): "can you fix the yellow indoor photo in under a minute?" (yes, with difficulty, no).

### FR-ADJ-03 Sharpen
*Phase 3 · Core · Traces: EDIT-11*
A sharpen control with a sensible default amount and a visible 100% preview.
- **Auto** (`golden`): edge fixtures sharpen to reference; flat areas gain no noise beyond tolerance.

### FR-ADJ-04 One-click auto contrast and auto colour
*Phase 3 · Core · Traces: EDIT-05, MOD-06*
One click applies a deterministic, histogram-aware correction expressed as ordinary recipe values the user can see and edit.
- **Auto** (`golden`): a fixture set of under-exposed, over-exposed and colour-cast photos improves a stated metric (proposed: clipped pixels and cast distance reduced) and matches references.
- **Judge** (Gt-ADJ): on 10 photos, "better", "same", "worse"; pass is at least 7 better and no more than 1 worse.

### FR-ADJ-05 Auto-enhance
*Phase 3 · Core · Traces: EDIT-06, MOD-06*
A single "Auto-enhance" action (our own name) combines the one-click fixes; it is never a black box: the resulting values appear as recipe operations and can be edited or undone in one step.
- **Auto** (`golden`, `unit`): deterministic; undo in one step; values visible in the recipe.

### FR-ADJ-06 Red-eye correction
*Phase 3 · Core · Traces: EDIT-04*
The user marks an eye region and red-eye is removed; automatic detection waits for the faces work (Phase 6).
- **Auto** (`golden`): fixture eyes lose the red cast within tolerance; surrounding pixels change by no more than a threshold.

### FR-ADJ-07 A small set of looks
*Phase 3 · Core · Traces: EDIT-12*
A handful of parametric looks (for example black and white, sepia, warm, cool, fade, vignette) with our own names and an intensity slider, each stored as ordinary recipe operations. The set is a default pending OQ-5.
- **Auto** (`golden`): each look at intensity 0 is a no-op and at 100% matches its reference.

### FR-ADJ-08 Copy and paste edits across a selection
*Phase 3 · Core · Traces: ADD-03*
Copy the recipe of one photo and paste all of it, or chosen parts, onto many selected photos in one undoable step. Per-photo operations (crop, straighten) can be excluded.
- **Auto** (`property`, `golden`, `perf`, `integration`): pasting onto N photos yields identical recipes apart from excluded operations; partial-paste semantics are specified and tested; one undo reverts the whole paste; pasting onto 1,000 photos meets the budget; kill-and-restart leaves only whole sidecars.
- **Judge** (Gt-ADJ): "edit one photo, paste onto the 20 from the same shoot: did it behave as expected?"
