# Spec 01: Foundations and guarantees (FR-FND)

Cross-cutting requirements. They are not a trial of their own: every trial (Gt) re-checks the ones marked "every trial".
Format and vocabulary are explained in [`functional-spec.md`](functional-spec.md#2-how-to-read-this-specification).

**Reference machines.** *CI Mac* means the `macos-15` Apple Silicon runner. *Keith's Mac* means his Apple Silicon Mac used
in trials. Performance numbers below are **proposed targets**: Phase 1 records baselines on the CI Mac, then Keith confirms
or changes them (SQ-2).

### FR-FND-01 Originals are never modified
*Phase 1 · Foundation · Traces: FUT-12, LIB-10, ADR-0002*
Leoht never changes, moves, renames or deletes an original photo file unless the user explicitly asks for that action in
the UI or CLI. Scanning, thumbnailing, viewing, editing, rating, albums and search only write sidecars, the catalog,
thumbnail caches and library-level Leoht files, all outside the original files' bytes.
- **Auto** (`integration`): an end-to-end test hashes every original file (bytes and modification time) of a fixture library
  before and after each operation in a matrix. Originals must be identical; only sidecars (new or updated) and Leoht
  library files may change. The matrix covers the operations that exist at that phase and grows with each phase (Phase 1:
  scan, thumbnails, rebuild; Phase 2: open and view; Phase 3: edit, save sidecar, save a copy, export; Phase 4: rate, album, search).
  Runs on every PR.
- **Auto** (`integration`): with the *original files* set read-only, every operation still leaves them identical. On a fully
  read-only library (for example a read-only volume), scan, view and search work with the catalog and caches stored
  elsewhere, and actions that must write a sidecar fail with a clear message instead of writing anywhere hidden.
- **Every trial** (T5): the trial runs the before/after hash check on the bundled samples and prints the result.

### FR-FND-02 The index is disposable
*Phase 1 · Foundation · Traces: LIB-10, LIB-11, META-04, EDIT-19, ADR-0001, ADR-0002*
Deleting the catalog database and thumbnail cache loses no user-authored data (edits, ratings, captions, tags, albums,
stack choices, hidden flags, custom order). Everything the user authored lives in sidecars or library files.
- **Auto** (`integration`, `property`): a test authors data of every kind, deletes the database, rescans, and asserts the
  authored data is identical. The test list grows with each phase; a PR adding a new user-authored field fails review
  without a case here.
- **Auto** (`perf`): rebuild time on 1k and 10k libraries is within 1.2x the cold-scan budget in FR-FND-07.

### FR-FND-03 Writes are atomic and forward compatible
*Phase 3 · Foundation · Traces: EDIT-19, META-04, ADR-0002*
Every write of user data is atomic (a crash leaves the old or the new file, never a partial one), and an older Leoht never
destroys data written by a newer one (it refuses the file with a clear message).
- **Auto** (`unit`, `property`): sidecar round-trip; fault-injection tests that kill the writer between steps; a newer
  `schema_version` is refused and the file is left byte-identical.
- **Auto** (`integration`): a kill-and-restart test during a 1,000-photo paste leaves only whole sidecars.

### FR-FND-04 No hidden data
*Phase 3 · Foundation · Traces: META-03, EDIT-18, EDIT-20*
Everything the user needs to back up is in visible, named files in the library, and the app documents each kind of file
(sidecar, album file, cache location). Nothing important lives in a hidden folder or an app-private location.
- **Auto** (`integration`): after a session using every feature, a script lists all files created in the library; each
  matches a documented pattern in `docs/` (a docs test fails on an undocumented file kind).
- **Judge** (Gt, once per trial): Keith copies the sample library to a new folder and finds that edits and ratings came along.

### FR-FND-05 Everything core works from the keyboard
*Phase 2 · Foundation · Traces: MOD-04, FUT-11*
Every core flow (browse folders, move through the grid, open and close the lightbox, rate, cull, crop and adjust, search,
albums) can be done without a pointer, with a visible focus indicator and no keyboard traps.
- **Auto** (`ui`): UI tests drive each core flow with key events only and assert the end state.
- **Judge** (Gt-BRW and each later trial): checklist item "do the task using only the keyboard"; a failure is a defect, not a nit.

### FR-FND-06 Screen reader, contrast, text size and motion
*Phase 2 · Foundation · Traces: MOD-04, FUT-11*
Controls expose accessible names and roles to the system screen reader (VoiceOver on Mac), text meets a contrast ratio of at
least 4.5:1 (our own target; the common AA level), text honours the system size setting, and animation honours "reduce motion".
- **Auto** (`ui`, `unit`): an accessibility-tree test asserts every interactive control has a name and role; a contrast test
  checks both themes' palettes; a test asserts animations are disabled when reduce-motion is set.
- **Judge** (Gt-MAC, and a first spike in Phase 2): VoiceOver walkthrough of open folder, browse, open photo, rate. **Unverified:** how far the
  chosen UI toolkit supports this today; the Phase 2 spike must report it, and Phase 5 reassesses the toolkit (inventory MOD-04).

### FR-FND-07 Performance budgets by library size
*Phase 1 · Foundation · Traces: MOD-12, FUT-05, FUT-06, ORG-10, VIEW-01*
Leoht meets these budgets (p95 unless stated) on the CI Mac, and feels right on Keith's Mac. They are proposed targets.

| Metric | 1k photos | 10k photos | 100k photos |
|---|---|---|---|
| Cold scan (walk, stat, EXIF), without thumbnails | 5 s | 50 s | 10 min |
| Warm start to a browsable grid | 1 s | 1.5 s | 3 s |
| Rescan with no changes | 1 s | 5 s | 60 s |
| Grid scroll, frame preparation | 8 ms | 8 ms | 8 ms |
| Jump to a position, visible thumbnails drawn (cached) | 200 ms | 300 ms | 500 ms |
| Live search, per keystroke to updated results | 50 ms | 100 ms | 250 ms |
| Memory while browsing (RSS) | 500 MB | 800 MB | 1.5 GB |
| Thumbnails for the whole library (background, informational) | 1 min | 10 min | 100 min |

Size-independent: app launch to window 2 s; lightbox next image (preloaded) 50 ms, cold open of a 12-megapixel JPEG 300 ms;
edit preview update while dragging a slider 50 ms; export of one 12-megapixel JPEG 2 s; paste of an edit onto 1,000 photos 10 s.
- **Auto** (`perf`, `volume`): `criterion` benchmarks and deterministic synthetic libraries (`cargo xtask gen-library`)
  enforce the 1k and 10k columns in CI from Phase 1 (rows for features that do not exist yet, such as live search from Phase 4, join the check when their phase delivers them). The 100k column runs in a scheduled volume job from Phase 6 (plan and
  ADR-0003). A regression of more than 25% against the recorded baseline fails the check.
- **Judge** (each trial): "does it feel fast?" answered fast, OK or slow; "slow" opens a performance issue even if the budget passes.

### FR-FND-08 Private and offline by default
*Phase 1 · Foundation · Traces: Core Principle 3, MOD-07*
Leoht makes no network connection and collects no telemetry in any build unless the user turns on a named feature that needs it.
- **Auto** (`integration`): the core runs a full scenario with all network traffic routed to a recording sink, and the test
  fails on any connection attempt (not merely on success when blocked); `cargo-deny` bans networking and telemetry crates
  from the default feature set.
- **Every trial:** the trial notes say that no data leaves the Mac.

### FR-FND-09 A scriptable core
*Phase 1 · Foundation · Traces: MOD-13*
Everything the UI does is available from a stable CLI (`scan`, `list`, `thumbs`, `verify`, later `edit` and `export`) with
machine-readable output and documented exit codes, so agents and tests can drive it.
- **Auto** (`integration`): CLI contract tests with snapshot output (`insta`); a changed output needs a reviewed snapshot update.
- **Judge** (Gt-LIB): Keith runs the one-line demo command on the samples.

### FR-FND-10 Unicode and long paths
*Phase 1 · Foundation · Traces: MOD-15*
Folder and file names with any Unicode, spaces, very long paths and case differences work everywhere (UTF-8 path types).
- **Auto** (`unit`, `integration`): fixture libraries with emoji, combining characters, right-to-left names and 400-character paths.

### FR-FND-11 Platform
*Phase 1 · Foundation · Traces: META-07*
macOS on Apple Silicon is the supported and required-CI platform; Linux is required in CI; Windows builds are non-blocking
until Phase 6 (see `docs/process/phase-gates.md`). The supported macOS versions are decided in Phase 5 (SQ-5).
- **Auto:** CI matrix as defined in `.github/workflows/ci.yml`; the required checks include `macos-15`.

### FR-FND-12 Strings ready for translation
*Phase 2 · Foundation · Traces: META-06*
All user-visible text lives in one string module (no literals in UI code), so translation can come later. English only ships.
- **Auto** (`unit`): a lint-style test fails on user-visible string literals outside the string module.

### FR-FND-13 Quality is measured and the story is honest
*Phase 1 · Foundation · Traces: Positioning (inventory)*
Each phase exit publishes a short quality report (test counts by level, benchmarks against the FR-FND-07 budgets,
golden-image changes, open defects, trial verdicts) and a "How we built X" narrative with a section on what went wrong.
- **Auto** (`integration`): a `cargo xtask` check fails the phase-exit PR if the report or the "what went wrong" heading is missing.
- **Judge** (Gn): Keith reviews the report and the narrative at the phase gate.
