# Picasa feature inventory (clean-room, public sources)

* Status: Draft for review (research input to the functional specification); revised 2026-11-27 with Leoht additions and positioning; OQ-1 resolved 2026-11-27 (Keith)
* Date: 2026-11-27
* Author: Leoht research agent, for Keith Haviland (owner)
* Scope: Google Picasa **desktop** application, final 3.x versions (3.9 line, last builds October 2015; support ended
  2016-03-15 [S15, S16])
* Companion: [`sources.md`](sources.md) (every `[S..]` reference below is numbered there, with archive dates)

## Positioning (Leoht original)

*Our own framing; no external sources needed.*

Leoht's primary purpose is a **public case study**: can a high-quality desktop app whose requirements are partly
subjective ("feels like a Mac app", "colour looks right", "fast") be built agentically, in the open? This differs from
spec-driven showcase projects such as the multi-agent C compiler and functional AWK interpreter efforts cited in
`LEOHT_PLAN.md`, where a precise specification and a large conformance suite exist from day one. A photo app has no
such oracle, so the project has to create its own.

Differentiators we intend to demonstrate and measure:

1. **Mac-native excellence** (Mac first; colour, Retina, HDR, platform conventions).
2. **HEIC and RAW done properly** (format matrix in ADR-0004; licensing in ADR-0005).
3. **Testable acceptance criteria derived from fuzzy requirements** (golden images, performance budgets, UI tests, labelled fixture sets).
4. **Recorded human judgement gates** where a person must decide (feel, best-shot quality, HDR on real displays).
5. **Measured quality**, published with each phase's "How we built X" narrative.

**Features are complex to visualise from a description.** So the first trial of each feature group (an early, rough build that
Keith tries hands-on before polish; see `docs/process/phase-gates.md`, gate Gt) is what validates the recommendations below.
Every trial must be very easy to run: a single download or command, no build tools, sample photos ready, a short
checklist, originals never modified, and easy feedback. Until a group has had its trial, treat its rows as hypotheses.

This is the lens for the recommendations below: Picasa's behaviours are the baseline for *what users expect*, and the
sections "Leoht additions (committed)" and "Future possibilities" record where we intend to go further.

## Clean-room statement

This document describes what Picasa *did*, in our own words, using only public descriptions: archived Google help
pages and blog posts, Wikipedia, and third-party reviews and tutorials. Nobody downloaded, installed, decompiled or
inspected Picasa binaries, code, libraries or assets. We reproduce no Google text beyond a few words, no icons or
screenshots, and we use "Picasa" only to say what we studied. We did **not** record Picasa's private file layouts (see
[open question OQ-1](#open-questions-for-the-human-gate)). Anything not backed by a source is marked **unsourced, verify**.

## How to read this

Each feature has a neutral description, source references, and a Leoht recommendation:

| Recommendation | Meaning |
|---|---|
| **Keep** | Build the behaviour essentially as Picasa did. |
| **Modernise** | Build the idea, but change how (safer, more modern, or to fit a Leoht principle). |
| **Defer** | Worth having, but not before the phase shown; revisit then. |
| **Drop** | Do not build (obsolete, cloud-bound, or out of scope). Phase shown as `-`. |
| **Add** | Leoht original, not a Picasa feature: commit to building it (section "Leoht additions (committed)"). |

The *Phase* column uses the numbering in `LEOHT_PLAN.md` (on `main`, as are ADR-0001 to ADR-0005): 0 foundation, 1 folder scanner and index, 2 viewer and grid UI,
3 non-destructive edit engine, 4 organisation features, 5 Mac polish and packaging, 6+ cross-platform and advanced
features (faces, geotag map, batch operations, large libraries). Where the plan has no home for a feature (import,
slideshow) we say so and propose one; those are suggestions for you to approve, not decisions.

## Summary


| Area | Keep | Modernise | Defer | Drop | Add | Total |
|---|---:|---:|---:|---:|---:|---:|
| 1. Library and folder management | 7 | 6 | 4 | 0 | 0 | 17 |
| 2. Organisation: albums, stars, captions, tags, timeline, search | 6 | 7 | 1 | 1 | 0 | 15 |
| 3. Viewing: grid, lightbox, slideshow | 4 | 2 | 0 | 2 | 0 | 8 |
| 4. Editing | 12 | 7 | 4 | 1 | 0 | 24 |
| 5. People and faces | 2 | 2 | 0 | 2 | 0 | 6 |
| 6. Places and geotagging | 1 | 2 | 0 | 1 | 0 | 4 |
| 7. Import | 2 | 2 | 1 | 1 | 0 | 6 |
| 8. Export and output | 2 | 2 | 2 | 4 | 0 | 10 |
| 9. Metadata, duplicates, hidden items and misc | 3 | 3 | 1 | 0 | 0 | 7 |
| **Subtotal: Picasa-derived features** | **39** | **33** | **13** | **12** | **0** | **97** |
| Leoht additions (committed) | 0 | 0 | 0 | 0 | 5 | 5 |
| **Total inventoried rows** | **39** | **33** | **13** | **12** | **5** | **102** |
| *Future possibilities (not committed; separate list)* | | | | | | *14* |

Counts are of the rows in sections 1 to 9 below, plus the five committed Leoht additions (`Add`: our own ideas, not Picasa's). A row sometimes groups several closely related Picasa functions, such as a pack of effects. Future possibilities are listed separately and are not counted as commitments. They are a planning aid, not a feature-point estimate.

### Most notable recommendations

1. **Keep the core model.** Watched folders, albums as virtual collections that point at files, and edits kept apart from the original are exactly what the plan and ADR-0001/0002 already say. The research confirms they are the features people valued [S01, S18, S22, S24].
2. **Do not hide data.** Picasa stored per-folder edit data and moved originals into a hidden dot-folder on "Save" [S02, S16, S24]. Leoht keeps one visible sidecar per image and defaults to export or "save a copy", with any in-place save keeping a visible, documented backup (EDIT-18 to EDIT-21, META-03).
3. **Principle conflict to settle.** Picasa wrote keywords, captions and map coordinates *into* the original files [S16, S22]. Principle 1 says originals are never changed unless asked. We recommend sidecar by default and an explicit "write metadata to files" option (META-01, GEO-02). This needs your decision (OQ-2).
4. **Drop everything cloud, disc or novelty.** Web-album upload and sync, collaborative albums, online contacts, movie maker, gift discs, burned slideshows, the screensaver and the online editor add-on (12 rows marked Drop). Novelty effect packs are deferred, not built early (EDIT-15).
5. **Import has no home in the plan.** Picasa's import (device detection, date-named folders, skip duplicates, safe card clearing) is where folder structure begins. We suggest pulling a basic import earlier than Phase 6+ (OQ-3).
6. **Phase 5 is the natural home** for slideshow, email via the share sheet, quick standalone viewer and file associations.
7. **Picasa interoperability is a live question.** Several active open-source projects work with Picasa's per-folder metadata: PicasaPy documents reading and writing it [S44], photon shares star ratings with Picasa's ini file [S48], and Picasso imports stars, captions and rotation [S46]; Picco describes itself as keeping a Picasa library alive [S45]. We did not verify read and write support for Picco or Dalica [S47]. Keith decided on 2026-11-27 that Leoht will offer a read-only importer in Phase 6 or later, gated on a legal review (OQ-1, resolved).
8. **Avoid Google's names.** Do not reuse "I'm Feeling Lucky" or effect names that echo third-party camera brands; use our own labels (EDIT-06, EDIT-15).
9. **Leoht goes beyond Picasa in five committed places** (geometry suite, culling mode, copy/paste edits, similar-photo stacks, wide-colour and HDR display; ADD-01 to ADD-05), and records 14 further ideas as future possibilities (FUT-01 to FUT-14). Accessibility, performance and the originals guarantee may be promoted to requirements in the functional spec.
10. **Our closest peers now exist.** Two Rust projects (Dalica, Picco) state a near-identical Picasa-faithful, local-first, sidecar-plus-cache goal; see the [Picasa-inspired projects](#picasa-inspired-projects) section for positioning.


## 1. Library and folder management

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| LIB-01 | Watched folders | The user picks folders to monitor; files added or removed on disk appear or vanish in the app without a manual refresh. | [S17] [S19] [S22] | **Keep** | 1 | Core to the plan's 'filesystem is the source of truth'; use OS file-change events with a periodic reconcile scan. |
| LIB-02 | Scan-once folders | A folder could be indexed a single time and then left alone, mainly so a disconnected drive stayed in the library. | [S19] [S21] | **Modernise** | 4 | Replace with remembered removable volumes shown as 'offline' rather than a separate scan mode. |
| LIB-03 | Ignore a folder | 'Remove from Picasa' made the app skip a folder or drive entirely without touching the files. | [S19] [S20] [S22] | **Keep** | 1 | Needed so users can exclude backups and system folders; store as a config list. |
| LIB-04 | Folder list mirrors disk | A left-hand list showed the real folders, as a tree or a flat list, sortable by name, date or size; empty folders were not shown. | [S01] [S22] | **Keep** | 2 | Direct plan item ('folder hierarchy mirroring disk'). |
| LIB-05 | Hide photos or folders | Individual photos or folders could be hidden from the library and un-hidden later, without deleting them. | [S01] [S22] | **Keep** | 4 | Cheap, useful for duplicates and private items; store in sidecar/config, never rename files. |
| LIB-06 | Reveal on disk | A context-menu action opened the photo's or folder's location in the system file manager. | [S22] | **Keep** | 2 | Reinforces that folders are real; use Finder 'Reveal' on Mac (Phase 5 polish). |
| LIB-07 | Folder operations inside the app | Rename, move and delete folders, and drag photos between folders, acted on the real files. | [S01] [S18] [S22] | **Modernise** | 4 | Keep only as explicit, confirmed actions (plan: do not relocate originals unless asked); undo where possible. |
| LIB-08 | Delete means delete | Deleting a photo from a folder in the app removed the real file (and thus its album entries). | [S18] [S22] | **Modernise** | 4 | Always send to the system Trash/Recycle Bin with a confirmation; never unlink permanently. |
| LIB-09 | Add a single file | A menu item added one file outside the watched folders to the library. | [S04] | **Defer** | 5 | Folder-first design; later handle via 'Open with Leoht' file association instead of adding to the index. |
| LIB-10 | Central index with cached thumbnails | A private database held paths, thumbnails and search words so large libraries browsed instantly without reading the files. | [S17] [S19] [S22] | **Keep** | 1 | Matches plan (SQLite + cached thumbnails); the index is a rebuildable cache, never the only copy of user data (ADR-0001/0002). |
| LIB-11 | Rebuild the index | If the index was damaged it could be rebuilt from the photos' own metadata plus per-folder data. | [S01] [S17] [S20] | **Modernise** | 1 | Make rebuild the normal recovery path and a tested one: delete the database, rescan, lose nothing. |
| LIB-12 | Move the index to another drive | An experimental menu item (3.9) moved the database to a different local drive. | [S11] | **Defer** | 6+ | A cache-location setting is enough; low priority. |
| LIB-13 | Choose which file types to show | Preferences let users tick which photo, video, audio and RAW types appear; the list covered JPEG, TIFF, BMP, GIF (first frame), PSD, PNG, TGA, major RAW makers, and later WebP. | [S03] [S11] [S16] | **Modernise** | 1 | Adopt the idea; set list from ADR-0004 (JPEG, HEIC, PNG first; RAW, TIFF, WebP staged). |
| LIB-14 | Video files in the library | Video files were listed and played in the app and could feed the movie maker. | [S03] [S22] | **Defer** | 6+ | Not in the plan; revisit after core photo workflow. |
| LIB-15 | Per-folder face-scan opt-out | A switch in the folder manager turned face detection off for a folder and its sub-folders. | [S21] [S11] | **Keep** | 6+ | Privacy default: faces are opt-in per library and per folder when we build them. |
| LIB-16 | Network drive support | Release notes mention improved behaviour on network drives. | [S11] | **Defer** | 6+ | Plan targets local-first; handle NAS later with the volume-identity work (LIB-02). |
| LIB-17 | Colour management switch | A 3.8 option applied colour profiles (off by default). | [S11] | **Modernise** | 2 | Be colour-managed by default on HiDPI wide-gamut Macs; see 'Modern expectations'. |

## 2. Organisation: albums, stars, captions, tags, timeline, search

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| ORG-01 | Folders versus albums | Folders were real directories; albums were app-only lists that pointed at photos. Photo lives in one folder, may appear in many albums. | [S01] [S18] [S22] | **Keep** | 4 | Exactly the plan's 'albums as virtual collections'; the distinction is the heart of the model (ADR-0001). |
| ORG-02 | Album create and edit | Select photos, choose 'add to album', name it; removing a photo from an album left the file alone. | [S01] [S18] | **Keep** | 4 | Core organisation feature. |
| ORG-03 | Photo tray | A holding area collected photos from any folders for batch actions (hold, clear), then drove album, export, email and print. | [S01] [S02] [S07] | **Modernise** | 4 | Keep as a persistent 'selection basket' that survives restarts and can be saved as an album. |
| ORG-04 | Manual order inside folders and albums | Drag and drop changed display order; batch rename followed that order. | [S22] | **Keep** | 4 | Store custom order in the album definition; for folders, as a durable sort key in sidecars (the index may cache it, but must not be its only home, so a rebuilt index keeps the order). |
| ORG-05 | Album data survives reinstall | From 3.9, album membership was written alongside other edit data so albums could be restored after reinstalling or moving machines. | [S11] | **Modernise** | 4 | Albums span folders, so they cannot live in a per-image sidecar alone; decide the portable album file in an ADR. |
| ORG-06 | Star favourite | A single yellow star toggled with the space bar or button; a button selected all starred photos. | [S18] | **Modernise** | 4 | Keep one-key starring; offer 0-5 ratings (modern norm) and map to XMP rating on export. |
| ORG-07 | Captions | One caption per photo, typed under the thumbnail, shown in thumbnails, slideshows and prints; stored inside the file as IPTC data. Not batch-editable. | [S22] [S16] [S26] | **Keep** | 4 | Keep captions; allow batch caption (the gap users noticed). Storage policy: see META-01. |
| ORG-08 | Tags (keywords) | Keywords added singly or in batch, with a Tags tab and quick-tag buttons (3.5); IPTC keywords were written only into JPEG files. | [S11] [S16] [S22] | **Modernise** | 4 | Support all formats; store in sidecar, optional XMP write (META-01). |
| ORG-09 | Timeline | A date-ordered view of the library (timeline view existed; details unsourced, verify). | [S11] [S16] | **Keep** | 4 | Date-grouped browsing is a core expectation; build on EXIF capture date from Phase 1. |
| ORG-10 | Live search box | Always-visible search, results filter as you type; matches folder names, file names, tags and captions; every term must match and partial words from the start of a word match. (A leading minus to exclude terms is reported only by an older Wikipedia mirror: unsourced, verify.) | [S22] [S16] | **Modernise** | 4 | Keep behaviour; add EXIF fields and saved searches; performance-test on 100k indexes (plan). |
| ORG-11 | Colour search | An experimental search operator found photos containing a given colour. | [S16] | **Defer** | 6+ | Novelty; revisit with on-device image embeddings. |
| ORG-12 | Selection shortcuts | Select all, deselect, first/last, hold-in-tray and new-album had keyboard shortcuts. | [S07] | **Keep** | 2 | Standard grid behaviour; use Mac conventions (Command key) first. |
| ORG-13 | Batch rename | Rename all photos in a folder with a numbering scheme, in display order. | [S22] | **Modernise** | 6+ | Pattern rename with preview and persistent undo; plan lists 'batch operations' for Phase 6+. |
| ORG-14 | Adjust date and time | A tool shifted the recorded date and time of photos (added in 3.5). | [S11] | **Modernise** | 6+ | Useful for camera clock errors; store as sidecar offset, write EXIF only on explicit consent. |
| ORG-15 | Collaborative albums | 3.6 let several people add to one online album. | [S11] | **Drop** | - | Cloud feature; Leoht is local-first. |

## 3. Viewing: grid, lightbox, slideshow

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| VIEW-01 | Thumbnail grid | Thumbnails of adjustable size with either caption or file name beneath (not both). | [S22] | **Modernise** | 2 | Keep size slider; allow caption and name together. |
| VIEW-02 | Lightbox (edit view) | Double-click opened a large view that was also the editor; Esc returned to the library; next photo with the mouse wheel; star from the viewer. | [S22] [S18] [S16] | **Keep** | 2 | Plan Phase 2; separate a read-only lightbox from the editor so viewing is instant. |
| VIEW-03 | Properties panel | A panel showed EXIF data and, from 3.8, better XMP support. | [S11] | **Keep** | 2 | EXIF extraction is Phase 1; the panel arrives with the viewer. |
| VIEW-04 | Fullscreen slideshow | Play an album or folder fullscreen, optional loop; captions can show. | [S06] [S22] | **Keep** | 5 | Not in plan phases explicitly; natural Mac-polish item; low effort once the viewer exists. |
| VIEW-05 | Side-by-side compare | Compare two photos (A vs B), or the original and the edited version of one photo (A vs A), above/below or left/right. | [S02] [S11] | **Keep** | 3 | Original-versus-edit (A vs A) is the best way to judge an edit; two-photo (A vs B) is useful for culling (Phase 4). |
| VIEW-06 | Fast standalone viewer | A separate lightweight viewer opened images from the file manager without starting the full app. | [S16] [S11] | **Modernise** | 5 | Mac file association and quick-look style viewer; spawned projects replicate this (see Picasa-inspired section). |
| VIEW-07 | Slideshow burned to disc | Slideshows could be written to autorun CD/DVD at reduced size. | [S22] [S25] | **Drop** | - | Optical media is obsolete. |
| VIEW-08 | Screensaver | Cropped pictures and captions in the screensaver were fixed in release notes, implying a screensaver existed. | [S11] | **Drop** | - | Not worth shipping. |

## 4. Editing

Picasa grouped edits into Basic fixes, Tuning and Effects tabs.

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| EDIT-01 | Crop with presets | Crop with preset proportions; custom sizes could be saved (3.5). | [S02] [S11] [S22] | **Keep** | 3 | Plan item. |
| EDIT-02 | Straighten | Rotate by a small variable angle to level horizons. | [S02] [S22] | **Keep** | 3 | Plan item. |
| EDIT-03 | Rotate and flip | Turn by 90 degree steps; flip horizontally or vertically. | [S02] | **Keep** | 3 | Store as orientation op in recipe; never rewrite the original's EXIF. |
| EDIT-04 | Red-eye correction | Manual correction and automatic correction (also in batch). | [S02] [S11] [S13] | **Keep** | 3 | Plan item (manual first; automatic detection after face work). |
| EDIT-05 | Auto contrast and auto colour | One-click fixes: set brightness/contrast to sensible levels, and remove colour cast. | [S02] | **Modernise** | 3 | Keep one-click; use smarter, tested algorithms (golden-image tests). |
| EDIT-06 | One-click 'lucky' fix | A single button adjusted colour and contrast together. | [S02] | **Modernise** | 3 | Keep as 'Auto-enhance' (our own name; the original phrase is Google's). |
| EDIT-07 | Fill light, highlights, shadows | Sliders to lift dark areas, tame or boost bright areas, deepen shadows. | [S02] [S26] [S22] | **Keep** | 3 | Maps to plan's exposure/contrast family; core tone controls. |
| EDIT-08 | Colour temperature and neutral picker | Warm/cool slider, and a picker to mark a neutral grey/white point. | [S26] [S22] | **Keep** | 3 | White balance is essential, especially for RAW later. |
| EDIT-09 | Retouch | A brush to remove spots and blemishes by clicking. | [S02] [S26] [S22] | **Defer** | 6+ | Heal/clone is a large feature; do after core recipe ops. |
| EDIT-10 | Text on image | Add and style text over a photo. | [S02] [S13] [S26] | **Defer** | 6+ | Low priority; overlap with export watermark (OUT-02). |
| EDIT-11 | Sharpen and saturation | A sharpen slider (also able to soften) and a colour-intensity slider from monochrome to vivid. | [S23] [S26] [S22] | **Keep** | 3 | Plan item (saturation); sharpen is standard. |
| EDIT-12 | Core look effects | Sepia, black-and-white, warm-up, colour tint with adjustable preserved colour. | [S23] [S26] | **Keep** | 3 | Cheap, popular; implement as parametric recipe ops. |
| EDIT-13 | Film grain, glow, soft focus | Grain, bright-area glow, and softening around an adjustable centre point. | [S23] [S26] | **Defer** | 6+ | Nice-to-have; implement after local masks. |
| EDIT-14 | Local-area effects | Focal black-and-white (colour kept in one region), graduated tint (tint feathered from the top), black-and-white through simulated colour filters. | [S23] [S26] | **Modernise** | 6+ | Modernise into general radial/graduated masks with any adjustment. |
| EDIT-15 | Third-tab effect packs (3.9) | About two dozen added looks: infrared, camera-toy looks, HDR-ish, cinemascope, cross-process, posterise, duo-tone, vignette, pixelate, pencil sketch, comic book, neon, borders, shadows, polaroid frame and similar. | [S11] | **Defer** | 6+ | Offer a few (vignette, duo-tone, cross-process) as shareable presets; skip novelty looks. Use our own preset names, not third-party brand-like names. Exact total unsourced. |
| EDIT-16 | Online editing add-ons | 3.8 added an online photo-editor entry in Basic fixes. | [S11] | **Drop** | - | Service retired; no cloud dependency. |
| EDIT-17 | Stackable, undoable edits | Effects could be applied repeatedly; undo/redo stepped back; the edit history persisted so changes could be undone in later sessions. | [S02] [S23] [S22] | **Keep** | 3 | Recipe is an ordered op list; history persists in the sidecar (ADR-0001/0002). |
| EDIT-18 | Edits as an overlay | Edits stayed outside the photo file; the app showed original plus edits; printing, emailing, exporting and uploading from the app used the edited version; other programs saw the unchanged original. | [S09] [S16] [S22] [S24] | **Modernise** | 3 | Keep the principle (plan principle 2). Add a visible 'edited' badge and an XMP export so other apps can see something. |
| EDIT-19 | Per-folder edit store | Edit history lived in one hidden file in each folder (also holding other per-photo data); the central index mirrored it. | [S16] [S17] [S22] [S24] | **Modernise** | 3 | Leoht chose one sidecar per image instead (ADR-0002) to avoid a hot shared file. |
| EDIT-20 | Save writes a new picture; original kept | 'Save' (single photo) or 'Save to disk' (whole folder/album) wrote an edited JPEG in place and moved the original to a hidden sub-folder; the save could be undone. | [S02] [S22] [S16] [S24] | **Modernise** | 3 | Default to export or 'save a copy'; if in-place save is offered, put the backup somewhere visible and documented, never hidden. |
| EDIT-21 | Save a Copy and Save As | 'Save a Copy' wrote an edited file beside the original with a '-1' suffix; 'Save As' chose name and place. | [S02] | **Keep** | 3 | Adopt a predictable suffix rule. |
| EDIT-22 | Output format | Saved edits were always JPEG. | [S02] | **Modernise** | 3 | Offer JPEG and PNG first, TIFF/HEIC later (ADR-0004). |
| EDIT-23 | RAW handled non-destructively | RAW files could be viewed, edited and saved out as JPEG without touching the RAW. | [S16] [S03] | **Keep** | 3 | Phase 1 detects RAW and shows embedded previews; editing in Phase 3+. |
| EDIT-24 | Edit many at once | Edits such as automatic red-eye could be run across photos held in the tray; a batch edit menu existed. | [S02] [S11] [S22] | **Keep** | 6+ | Plan lists batch operations for Phase 6+. |

## 5. People and faces

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| FACE-01 | Face detection and grouping | The app found faces and gathered similar ones into an 'unnamed people' collection. | [S01] [S14] [S16] | **Modernise** | 6+ | Plan: on-device recognition. Run models locally, opt-in, and keep the data deletable. |
| FACE-02 | Name tags | Type a name once under a face group and it applies to every photo in the group; names could also be added singly. | [S01] [S14] | **Keep** | 6+ | Core people workflow. |
| FACE-03 | People albums | A virtual album for each named person; better name suggestions arrived in 3.6. | [S01] [S11] | **Keep** | 6+ | Falls out of the album model (ORG-01). |
| FACE-04 | Where face data lived | Face tags were stored in the photo's own metadata area as well as in the index. | [S17] | **Modernise** | 6+ | Store in sidecar by default; embedding in the original only as an explicit opt-in because it exposes names. |
| FACE-05 | Face collage and Face Movie | Collage or time-aligned movie made from one person's photos. | [S14] [S11] | **Drop** | - | Novelty; skip. |
| FACE-06 | Contacts and online name sync | Names linked to a Google account's contacts and web albums. | [S11] [S14] | **Drop** | - | Cloud-linked; skip. |

## 6. Places and geotagging

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| GEO-01 | Geotag on a map | Select photos and drop them on a map (Google Maps in the Places panel) or search for a place and place a pin. | [S14] [S11] [S16] | **Modernise** | 6+ | Plan: geotagging map view. Use an open map source with offline option. |
| GEO-02 | Coordinates written into the file | Placing a photo wrote latitude and longitude into its EXIF data. | [S16] | **Modernise** | 6+ | Conflicts with 'never modify originals': store in sidecar; write EXIF only on explicit export or opt-in. |
| GEO-03 | View geotagged photos on a map | Already-tagged photos could be shown on the map; coordinates could be cleared. | [S14] [S11] | **Keep** | 6+ | Plan item. |
| GEO-04 | Album place (web) | Web albums had a 'place taken' field and map. | [S07] | **Drop** | - | Part of the retired web service. |

## 7. Import

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| IMP-01 | Import from camera, card or scanner | Connected devices were detected; user chose source, destination folder and which photos; already-imported photos were marked. | [S04] [S22] [S01] | **Keep** | 6+ | Not in the plan's phases; candidate to pull earlier because importing is where folder structure is born. |
| IMP-02 | Skip duplicates on import | A preference excluded photos already in the library, marking them with a red cross. | [S01] | **Modernise** | 6+ | Use content hashes (BLAKE3) rather than names. |
| IMP-03 | Auto-folder by date | Imports could be saved to folders by date taken, today's date, or a chosen name. | [S11] | **Keep** | 6+ | Matches the folder-first principle: make it configurable and previewable. |
| IMP-04 | Delete from card after copy | After importing, the user could delete copied photos or everything on the card. | [S04] | **Modernise** | 6+ | Only after verifying copies (hash check) and with confirmation. |
| IMP-05 | Star or upload while importing | Mark favourites and upload starred photos during import (3.5). | [S11] | **Defer** | 6+ | Rating at import is nice; upload is cloud, skip. |
| IMP-06 | Third-party library import | Release notes mention iPhoto import fixes. | [S11] | **Drop** | - | Obsolete. |

## 8. Export and output

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| OUT-01 | Export to folder | Selected photos were exported to a folder with a size slider and other export settings, baking in edits. | [S05] [S02] [S25] | **Keep** | 3 | Plan: export path that bakes edits. |
| OUT-02 | Resize and watermark on export | Resize to a chosen size; optional text watermark on export or upload. | [S02] | **Keep** | 6+ | Resize with export (Phase 3); watermark later. |
| OUT-03 | Email photos | Selected photos were resized and handed to Gmail or the default mail program. | [S05] [S02] | **Modernise** | 5 | Use the system share sheet on Mac; no account integration. |
| OUT-04 | Print | Print layouts and selectable print sizes (including 'full page'); links to print services. | [S07] [S16] | **Defer** | 6+ | Not in plan; use the OS print dialog later. |
| OUT-05 | Collage maker | Build collages (mosaic, framed mosaic, grid) with drag and drop. | [S06] [S11] | **Defer** | 6+ | Fun but non-core. |
| OUT-06 | Movie maker | Combine photos, video and music into a movie and upload to a video site. | [S06] [S13] | **Drop** | - | Out of scope. |
| OUT-07 | Gift disc | Write exported, edited copies to CD/DVD for giving. | [S22] [S25] | **Drop** | - | Obsolete. |
| OUT-08 | Backup pictures | A backup tool copied originals plus the per-folder data to disc or disk and could restore with edits intact. | [S25] [S11] | **Modernise** | 6+ | Because sidecars travel with photos, a plain folder copy backs up photos, edits and sidecars, but not album membership until the portable album file is decided (ORG-05); document that limit and offer a verify tool. |
| OUT-09 | Web album upload and sync | Upload, share with privacy levels, sync albums to the web, batch upload. | [S08] [S07] [S13] [S11] | **Drop** | - | Retired service (S15); plan says no mandatory cloud. |
| OUT-10 | Embed and link sharing | Links and embed code for web albums and slideshows. | [S07] | **Drop** | - | Retired service. |

## 9. Metadata, duplicates, hidden items and misc

| ID | Feature | What it did (our words) | Sources | Leoht | Phase | Reason |
|---|---|---|---|---|---|---|
| META-01 | Keywords and captions written into the file | Tags and captions were written into the image's IPTC fields (JPEG only) so other programs could read them; it also parsed XMP but could not search XMP keywords. | [S16] [S22] | **Modernise** | 4 | Principle-1 conflict: Picasa modified originals. Leoht: sidecar by default; optional, explicit 'write metadata to files' (XMP) with a clear warning. |
| META-02 | Duplicate handling | The help describes only manual methods: hide duplicates, drop folders from the library, or find and delete them by hand; import could skip known photos. | [S01] | **Modernise** | 4 | Plan marks duplicate detection optional; exact (hash) first, near-duplicates later. |
| META-03 | Hidden helper folders and files | The originals backup lived in a dot-named folder, hidden by default in file managers (easy to miss when copying; our inference). | [S16] [S24] | **Modernise** | 3 | Lesson: never hide data users need to back up; our sidecars are visible, named files. |
| META-04 | Per-folder data as a safety net | Per-folder data allowed rebuilding the index and restoring albums and edits. | [S17] [S20] | **Keep** | 3 | Same role as our sidecars (ADR-0002). |
| META-05 | Keyboard shortcuts and preferences | A set of shortcuts and preference pages (file types, slideshow, email, printing). | [S07] [S03] | **Keep** | 2 | Mac-native shortcuts first; consistent preferences window (Phase 5). |
| META-06 | Localisation | Release notes list 35 to 38 interface languages. | [S11] | **Defer** | 6+ | Design strings for translation early; ship English first. |
| META-07 | Platforms | Windows, Mac (Intel), and Linux via a Windows build under Wine; Mac functionality similar to Windows. | [S16] [S22] | **Keep** | 5 | Informational: confirms Mac-first is a reasonable strategy; Mac-only quirks in 3.9 are unsourced. |

## Modern expectations beyond Picasa

**These are Leoht additions. They are not features of Picasa and are not derived from Picasa sources.** They record what
users of a 2026 photo app reasonably expect, so the functional specification covers them. Where a source is shown it only
supports the rationale.

| ID | Expectation | Why it matters | Leoht approach | Phase |
|---|---|---|---|---|
| MOD-01 | HEIC/HEIF photos | Phones shoot HEIC by default; Picasa predates it. | First-class per ADR-0004 (pure-Rust decoder preferred; HEVC patent note). Mac can use system decoding for shipping. | 1 |
| MOD-02 | RAW without heavy dependencies | Picasa read RAW from many makers [S03]; today users also expect up-to-date cameras. | Detect, read metadata, show embedded preview first; decoding library choice waits on the licence decision in ADR-0005. | 1 (detect), 3+ (develop) |
| MOD-03 | Dark mode and system appearance | Expected on every platform. | Follow the OS setting; test both themes in snapshot tests. | 2 |
| MOD-04 | Accessibility | Keyboard-only use, screen-reader labels, contrast, reduced motion, text scaling. Immediate-mode toolkits are weaker here (verify current egui support). | Plan keyboard navigation and accessible names from Phase 2; reassess the UI toolkit against this in Phase 5. | 2, 5 |
| MOD-05 | HiDPI, wide-gamut and colour-managed display | Retina Macs and Display P3 screens are standard; Picasa's colour management was an off-by-default option [S11]. | Colour-managed by default; embed profiles on export. | 2, 3 |
| MOD-06 | Smarter auto-enhance | Picasa's one-click fixes [S02] were global and simple. | Histogram-aware, per-photo auto-enhance, deterministic and covered by golden-image tests; always shown as editable recipe values, never a black box. | 3 |
| MOD-07 | Private, on-device intelligence | Face grouping and natural-language search are now normal; users want them without uploads (compare Lap, Immich and PhotoPrism [S50, S42, S43]). | Opt-in, local models, data deletable, no network calls; per-folder opt-out as Picasa offered [S21]. | 6+ |
| MOD-08 | Safe file operations | Deleting from a photo app should be recoverable. | System Trash, confirmations, undo for moves and renames. | 4 |
| MOD-09 | Cloud-synced and shared folders | Libraries sit in iCloud/Dropbox/NAS; vendors warn about corruption risk [S37]. | Per-image sidecars and atomic writes (ADR-0002) reduce conflicts; detect and warn on sync-conflict copies. | 4 |
| MOD-10 | Offline and removable volumes | External drives come and go. | Remember volume identity; show offline items; never delete index data on disconnect. | 4 |
| MOD-11 | Standard metadata interchange | Other tools use XMP sidecars [S36, S39, S40]. | Read rating and keywords from XMP; export XMP; opt-in write-back (META-01). | 4 |
| MOD-12 | Very large libraries | Plan targets 10k to 100k+ photos. | Virtualised grid, incremental scans, volume benchmarks in CI. | 1, 2, 6+ |
| MOD-13 | Scriptable core | Agentic development and testing need a stable CLI. | CLI first (plan Phase 1), then a stable command surface the UI calls. | 1 |
| MOD-14 | Related-file groups | RAW+JPEG pairs and motion photos are common (Lap treats them as one item [S52], excerpt only). | Model "stacks" in the data model before Phase 4. | 4 |
| MOD-15 | Unicode and long paths | International folder names and cross-platform paths. | UTF-8 path types (already a workspace rule). | 1 |


## Leoht additions (committed)

**Leoht original: these are our own ideas, not Picasa's, and need no external sources.** Recommendation `Add` means
"commit to building it" (the equivalent of Keep for something Picasa did not have). Each has a rationale tag, a target
phase from `LEOHT_PLAN.md`, and a testability note, because the project's case-study goal depends on being able to verify
quality (see Positioning). Rationale tags: **Excellence** (makes the app clearly better) and **First ten minutes** (a user
notices it almost immediately).

| ID | Addition | Description | Rationale | Leoht | Phase | How we would verify it |
|---|---|---|---|---|---|---|
| ADD-01 | Geometry suite | One coherent set of geometry tools: straighten, horizontal perspective, vertical perspective and free crop; one-tap standard ratios (1:1, 4:3, 3:2, 16:9, 5:4, common print sizes, custom, and a lock to the original ratio); and auto-straighten that detects the horizon or strong vertical lines, only *suggests* a correction, and is fully undoable. | Excellence, and noticed in the first ten minutes: crop and straighten are the first edits anyone tries, so feel and correctness decide first impressions. Picasa had crop and straighten (EDIT-01, EDIT-02) but no perspective correction. | **Add** | 3 | Golden-image tests on synthetic grids and chart images with a tolerance; property tests for ratio arithmetic (rounding, lock, rotate-then-crop invariants); a labelled test set of tilted horizons and converging verticals with an accuracy target for auto-straighten; undo/redo round-trip test; UI test that a ratio tap changes the crop box as specified; recipe ops versioned under ADR-0002. |
| ADD-02 | Culling mode | A full-screen mode for sorting a shoot quickly: arrow keys to move, single-key star or reject, and optional auto-advance after a decision. Decisions are saved as sidecar data, never by touching or moving files. | Excellence: it turns the grid into a working tool for the most repetitive job in photography. Picasa had stars and a viewer (ORG-06, VIEW-02) but no dedicated pass-through workflow. | **Add** | 2 (full-screen viewer and keys), 4 (stars, reject flag, saved decisions) | UI tests with egui_kittest driving key sequences and asserting resulting ratings; a latency budget for next-image display (preload ahead; measured on the CI Macs); atomic-write test for sidecar updates; kill-and-restart test that decisions survive; manual Mac checklist for feel. |
| ADD-03 | Copy and paste edits across a selection | Copy the edit recipe from one photo and paste it (all of it, or chosen parts) onto many selected photos, with one undo for the whole paste. | Excellence, and it falls out of the design: because each edit is a portable recipe in a sidecar (ADR-0001, ADR-0002), pasting is just copying data. Picasa's batch edit (EDIT-24) was narrower. | **Add** | 3 | Property test: pasting a recipe onto N photos yields identical recipes (apart from per-photo ops such as crop that the user excludes); golden-image test for the rendered result; performance budget for a paste onto 1,000 photos; undo test; partial-paste semantics specified and tested. |
| ADD-04 | Burst and similar-photo stacks with best-shot pick | Group bursts and near-identical shots into a stack, show one cover, and suggest a best shot (for example by sharpness and exposure); the user can override. Files stay where they are. User choices (cover, best-shot override, manual stack edits) must be stored in durable sidecar data; the index may only cache the derived stack view, so it is never the sole copy (consistent with LIB-10 and LIB-11). | Excellence: modern cameras and phones produce many near-duplicates, and sorting them is where users spend time. Extends the stack idea (MOD-14) and optional duplicate detection (META-02). | **Add** | 4 (grouping on capture time and similarity), 6 (best-shot scoring) | Labelled fixture libraries of bursts and similar shots with precision/recall targets for grouping; deterministic scoring tests on fixtures for sharpness and exposure; override persistence test that deletes the database, rescans, and checks every override is still in effect; volume test that grouping a 100k synthetic library meets a time budget; a human-judgement gate for the best-shot heuristic, since 'best' is subjective. |
| ADD-05 | Correct wide-colour and HDR display | Photos display with correct colour on wide-gamut (Display P3) screens and Retina scaling, and HDR images and HDR-capable displays are handled properly, so edits look the same in the app as in an export. | Excellence, and noticed in the first ten minutes on a Mac: washed-out or oversaturated colour or soft Retina rendering makes an app feel wrong immediately. Picasa's colour management was an off-by-default option (LIB-17). | **Add** | 2 (colour-management baseline: ICC-aware, P3, Retina), 5 (HDR/XDR display on Mac) | Colour pipeline tests with known colour charts and ICC profiles against reference values; golden-image tolerances stated in a colour space; snapshot tests at 1x and 2x scale. CI cannot verify a real XDR display, so Phase 5 HDR needs a manual checklist on real Mac hardware and is recorded as a human gate. |

## Future possibilities (not yet committed)

**Leoht original: ideas we have recorded so they are not lost. None is a commitment.** Each has a candidate phase and the
reason it is deferred. Three of them (accessibility FUT-11, performance FUT-05, originals guarantee FUT-12) may be promoted
to requirements in the functional specification; the rest stay optional until a phase gate says otherwise.

| ID | Idea | One-line description | Candidate phase | Why deferred |
|---|---|---|---|---|
| FUT-01 | Hold-to-compare and split before/after | Hold a key to see the original, or drag a divider across the photo to compare before and after. | 3 | Needs the edit engine; the simple side-by-side compare (VIEW-05) covers the basic need first. |
| FUT-02 | Named edit history, snapshots and virtual copies | Name points in an edit history, snapshot them, and keep several versions of one photo without duplicating the file. | 3 to 4 | Needs a settled recipe format first; versions change the sidecar schema (ADR-0002). |
| FUT-03 | Explainable auto-enhance | Auto-enhance shows exactly which slider values it chose, so users can learn from and adjust it. | 3 | Depends on auto-enhance existing (EDIT-05, EDIT-06); cheap if recipes are the output, so a candidate to fold in. |
| FUT-04 | Highlight and shadow recovery from HEIC, RAW and HDR headroom | Use the extra range in HEIC, RAW and HDR files to recover bright and dark areas properly. | 3 to 6 | Needs the decode and colour work (ADD-05, RAW decoder licence choice in ADR-0005) before it is meaningful. |
| FUT-05 | Instant grid at 100k photos, 60 fps, with CI budgets | Smooth scrolling over very large libraries with performance budgets enforced in CI. | 1 to 2, 6+ | Partly covered already by the plan's volume tests and MOD-12; may be promoted to a requirement in the functional spec. |
| FUT-06 | Background indexing with no import wait | Open a folder and start browsing immediately while scanning and thumbnails continue in the background. | 1 to 2 | Follows from the Phase 1 scanner design; deferred only because the first scanner is a CLI. |
| FUT-07 | Live Photo and RAW+JPEG pairing | Treat a RAW with its JPEG, or a still with its motion clip, as one item. | 4 | Needs the stacks model (ADD-04, MOD-14); Live Photo video handling is outside the current plan. |
| FUT-08 | On-device content and colour search | Find photos by what is in them or by colour, using local models only. | 6+ | Needs the on-device AI work (MOD-07) and a model-licence review. |
| FUT-09 | Safe in-app move and rename that carries sidecars | Moving or renaming a photo or folder in the app moves its sidecar too and keeps albums intact. | 4 | Related to LIB-07 and MOD-08; needs the sidecar re-association design (source_hash) built first. |
| FUT-10 | Quick Look, Finder integration, share sheet and drag to Mail or Messages | Native Mac integration so Leoht photos behave like system photos. | 5 | Mac-polish phase; extends VIEW-06 and OUT-03. |
| FUT-11 | Accessibility | VoiceOver labels, keyboard-only operation, system dark and light, contrast, text scaling and reduced motion. | 2, 5 | Depends on the UI toolkit's accessibility support (MOD-04). Likely to be promoted to a requirement in the functional spec. |
| FUT-12 | Originals guarantee | A published, tested guarantee that Leoht never modifies, moves or deletes original files unless asked, with the test that proves it. | 1 onward | Already enforced by the end-to-end test in ADR-0002. Likely to be promoted to a requirement in the functional spec. |
| FUT-13 | Export-all-edits | One command exports every edit and metadata to open formats (XMP and flattened images) so users are never locked in. | 4 to 6 | Needs XMP export (META-01, MOD-11) first. |
| FUT-14 | Orphaned-sidecar health check | A scan that finds sidecars whose photo was renamed or removed outside the app and offers to re-attach or clean them. | 3 to 4 | Already noted as a Phase 3 task in ADR-0002; waits for the sidecar writer. |


## Lessons from comparable apps

Behaviour and user-experience lessons only. Licence column is what we verified on 2026-11-27 (sources in `sources.md`).
**GPL, LGPL and AGPL projects are studied for behaviour and public documentation only; under our clean-room rules
nobody reads or copies their code.** Proprietary products are studied from public help pages only.

| App | Licence (verified?) | Model | What to learn | Clean-room handling |
|---|---|---|---|---|
| digiKam [S27, S28] | GPL-2.0-or-later (licence file seen) | Album-based database, tags/ratings/labels, search over EXIF, IPTC and XMP | Powerful metadata search and tag management; import rules that organise on the fly; cautionary tale on feature sprawl. | Behaviour/docs only; never code (GPL). |
| XnView MP [S29, S30] | Proprietary freeware (private/educational/non-profit; paid for companies) | Folder browser, tabbed, many formats, batch convert | Format breadth and batch conversion expectations; folder-first browsing. | Public docs only; EULA forbids reverse engineering. |
| FastStone Image Viewer [S31, S32] | Proprietary freeware (personal/educational) | Fast browser, viewer, light editor | Speed-first browsing; compare, red-eye, resize, crop, colour adjustments in one small tool. | Public docs only. |
| Shotwell [S33] | LGPL-2.1-or-later | GNOME photo manager: import, events, keywords | Automatic date-based "events", simple import. | Behaviour/docs only (copyleft). |
| gThumb [S34] | GPL (v2 text; "or later" **not verified**) | Viewer/browser with importer and plug-ins | Lightweight browsing plus extension model. | Behaviour/docs only (GPL). |
| Apple Photos [S35] | Proprietary, bundled with macOS (**unsourced, verify**) | Managed library by default; optional referenced files | What Mac users expect by default; the referenced-files caveats (breaks on move/rename, not in iCloud or backup) show the cost of folder-first and why Leoht needs rename-proof re-association (`source_hash`, ADR-0002). | Public help only. |
| Lightroom Classic [S36] | Proprietary, subscription (**unsourced, verify**) | Catalogue plus optional XMP sidecars | Writing metadata to sidecars on every change costs performance; Adobe added a second sidecar for heavy edits to keep XMP and catalogue small. Lesson for sidecar size and recipe design. | Public help only. |
| Capture One [S37, S38] | Proprietary (**unsourced, verify**) | Sessions (portable folder) or catalogues | Portable "session" thinking: keep settings next to images; vendor warns against cloud-synced storage (MOD-09). | Public help only. |
| darktable [S39] | GPL-3.0 | Database plus XMP sidecars as redundant fail-safe | Choose when sidecars are written (on import or after edit); detect externally changed sidecars at startup; database snapshots. All directly relevant to ADR-0002 follow-ups. | Behaviour/docs only (GPL). |
| RawTherapee [S40] | GPL-3.0 | Plain-text `.pp3` sidecars beside images or in a cache | Embedded-JPEG thumbnails until first edit; neutral defaults for non-RAW; copy/paste partial settings; versioning pitfalls when tool behaviour changes (supports our `schema_version`). | Behaviour/docs only (GPL). |
| RapidRAW [S41] | AGPL-3.0 (GitHub licence field) | GPU-accelerated RAW editor (named in plan) | Modern UI expectations for RAW editing; shows RAW decoding via a separate library. | Behaviour only; network copyleft, never code. |
| Picco [S45] | Apache-2.0 | Rust + egui, Picasa-faithful | Closest UX reference to what egui can deliver; see next section. | Permissive, but we study behaviour only to keep clean-room provenance simple. |
| Immich [S42] | AGPL-3.0 | Self-hosted server with mobile backup | Timeline and ML search at 100k+ scale; server model is out of scope. | Behaviour only. |
| PhotoPrism [S43] | AGPL-3.0 licence file; GitHub shows "Other"; paid memberships exist (**verify**) | Self-hosted index, labels, faces, places | Local-first indexing, filters, privacy messaging. | Behaviour only. |

## Picasa-inspired projects

Survey of projects that describe themselves as Picasa replacements or Picasa-inspired. Method: GitHub repository search
("picasa replacement", "picasa alternative", "inspired by picasa", topic pages), AlternativeTo, and Reddit/blog posts
found through web search, on 2026-11-27. **Not exhaustive.** Only repositories we confirmed exist through the GitHub API
are listed. Dates are UTC from GitHub (release publish date, last commit date). "Maintained?" is our judgement: *active*
= commit within 30 days, *recent* = within 90 days, *stale* = older.

| Project | URL | Licence | Platform / stack | Latest release; last commit | Maintained? | What it borrows from Picasa | Src |
|---|---|---|---|---|---|---|---|
| Dalica | https://github.com/pkkid/dalica | Apache-2.0 | Linux, Windows, macOS; Rust | v1.1.0 2026-11-06; 2026-11-11 | Active | Re-creates Picasa's workflow: watched folders, one continuous library view, photo tray, star/caption/tag/album, effects, collage, export; sidecar files plus cache index; states clean-room intent; built for AI agents. README banner says "0.1 preview" while releases show v1.1.0, so treat its status text as stale. | [S47] |
| Picco | https://github.com/miamibc/picco | Apache-2.0 | Linux, Windows, macOS; Rust + egui | v0.7.0 2026-11-06; 2026-11-06 | Active | Aims to "keep your Picasa library alive": reads and writes Picasa's folder metadata; basic fixes, tuning, effects, albums, stars, collage, movie, slideshow, export. | [S45] |
| Picasso | https://github.com/kuopinghsu/Picasso | MIT | Windows, macOS, Linux; Rust | v0.9.5 2026-10-27; 2026-11-27 | Active | "Inspired by Picasa": folders, albums, captions, stars, watching; imports stars, captions and rotation from Picasa metadata; non-destructive edits, batch export. | [S46] |
| PicasaPy | https://github.com/sanchomuzax/PicasaPy | GPL-3.0 | Python, PySide6/Qt Quick (platforms not verified) | v1.0.184 2026-11-25; 2026-11-27 | Active | Self-described modern successor; shares Picasa's per-folder metadata format; folders, albums, stars, captions, tags, faces, geotags, effects, collage, movie, print. | [S44] |
| photon | https://github.com/bsg62/photon | MIT | Linux, macOS, Windows; Rust | v0.59.0 2026-11-26; 2026-11-26 | Active | "Spiritual successor to Picasa 3": watches folders in place; shares star ratings with Picasa's folder file; SQLite cache. | [S48] |
| PIC | https://github.com/froggy744/PIC | MIT | Linux only; Rust + GTK4 | v1.0.4 2026-11-24; 2026-11-27 | Active | "Inspired by classic Picasa and iPhoto": browse in place, albums, non-destructive editing, RAW, collages. | [S49] |
| Lap | https://github.com/julyx10/lap | GPL-3.0-or-later | macOS, Windows, Linux; Tauri, Vue, Rust | v0.4.1 2026-11-20; 2026-11-27 | Active | Author says he built it after Picasa [S51]; folder-first, no import step, local AI search and face clustering, duplicate finder; README does not mention Picasa. Largest by stars (about 4.5k). | [S50] [S51] [S52] |
| Lumio Photo | https://github.com/coldtrace00/LumioPhoto | MIT | Windows; C++ / Qt | v1.0.0 2026-11-11; 2026-11-11 | Recent (brand new) | "Open replacement for Google Picasa": folder library, quick viewer like Picasa Photo Viewer, non-destructive edits, albums, stars, slideshow; keeps data in its own database, not in photo folders. | [S53] |
| PicaCatalog | https://github.com/peterfixit/picacatalog | MIT | Linux (only platform the README mentions); beta | none; 2026-09-24 | Stale | Offline catalogue "inspired by Picasa 3": local search, albums, captions, tags, duplicates, removable-drive awareness. | [S54] |
| Picasera | https://github.com/goldcarib/picasera | GPL-3.0 | Windows/macOS/Linux; Tauri + React + Rust | v0.3.1 2026-01-10; 2026-01-11 | Stale | "Modern Picasa replacement" for 10k+ libraries; early. | [S55] |
| picasa (ganeshapp) | https://github.com/ganeshapp/picasa | MIT | Tauri 2 | v0.1.0 2026-08-01; 2026-08-01 | Stale | "In the spirit of" Picasa: library folders, albums, stars, timeline, non-destructive rotate/crop, duplicates, presets. | [S56] |
| FlyPhotos | https://github.com/riyasy/FlyPhotos | GitHub shows "Other" (verify) | Windows; C# WinUI 3 | v2.9.0 2026-11-19; 2026-11-27 | Active | Viewer-only replacement for Picasa Photo Viewer. About 1k stars. | [S57] |
| Krumassan Image Viewer | https://github.com/sergkrumas/image_viewer | GPL-3.0 | Windows; PyQt | none (alpha); 2026-11-19 | Recent | Aims to replace Picasa Photo Viewer; release repeatedly postponed. | [S63] |
| Spica Photo Viewer | https://github.com/hiz8/spica-photo-viewer | MIT | Windows; Tauri | none; 2026-11-21 | Recent | Fast viewer with thumbnail strip, inspired by Picasa Photo Viewer. | [S60] |
| Micasa | https://github.com/ImRIzo/micasa | BSD-2-Clause | macOS 14+ (Apple silicon); SwiftUI | release 2026-10-25; 2026-10-25 | Recent (brand new) | Mac viewer/editor "inspired by Picasa". Mac-native, so a useful comparison for Phase 5. | [S59] |
| Iris | https://github.com/eliocro/iris | none stated | macOS; Electron | none; 2026-04-03 | Stale | Library viewer for Mac "inspired by Picasa". | [S58] |
| AJANTA Media Suite | https://github.com/raaz2507/AJANTA-Image-Suite | none stated | C++ | none; 2026-11-10 | Recent | Tribute to Picasa's speed and non-destructive editing. | [S61] |
| picasa2digikam | https://github.com/Philipp91/picasa2digikam | GPL-3.0 | Python script | none; 2026-03-17 | Stale | Migration tool: shows what Picasa data users want to keep (stars, albums, faces, tags). | [S62] |
| papa | https://github.com/shaeberling/papa | none | n/a | none; 2017-04-27 | Dead | Empty stub ("Picasa Alternative Photo App"); listed only to show we checked. | [S65] |

Also seen but not Picasa-derived: PhotoManager by Hawkynt (LGPL-3.0; folders and XMP sidecars with no database; borrows
Picasa-style picks/rejects) [S66]; and the AlternativeTo list of general alternatives (digiKam, nomacs, ImageGlass, Shotwell,
gThumb) [S64].

### What this means for Leoht

* **A wave of Picasa-inspired apps appeared in 2026** (11 of the 19 repositories below were created between July and November 2026; Lap, FlyPhotos, Krumassan and Spica are older). Demand is real, and the space is no longer empty.
* **Closest to Leoht's goals:** Dalica (Rust, Picasa-faithful, sidecars plus disposable index, clean-room intent, agent-driven development) and Picco (Rust + egui, Apache-2.0, macOS build). Both are permissively licensed. Under our rules we still study them for behaviour only, and keep Leoht's own data model and name. Differentiators worth stating in the spec: Mac-first, the public "How we built X" case study, golden-image and volume testing, and strict provenance.
* **Several projects interoperate with Picasa's per-folder metadata** (Picco, PicasaPy, Picasso, photon). Users with old libraries may expect that. It is a deliberate human-gate decision (OQ-1) because the format is undocumented by Google; third-party descriptions come from reverse-engineering.
* **Most projects are weeks or months old.** Verify activity again before relying on any of them, and expect their README claims (such as Dalica's status banner) to lag behind.
* **GPL-licensed ones (PicasaPy, Lap, Picasera, Krumassan, picasa2digikam) are behaviour-only for us**; Lap is the most relevant of those for local AI features.

## Open questions for the human gate

| ID | Question | Recommendation |
|---|---|---|
| OQ-1 | Should Leoht ever read Picasa's per-folder metadata to help migrate old libraries? | **Resolved by Keith, 2026-11-27: yes, as a read-only importer in Phase 6 or later, gated on a legal review** (gate G-import in `docs/process/phase-gates.md`; requirement FR-LAT-01 in `docs/spec/`). Leoht never writes Picasa's files, and the importer is built only from public third-party descriptions after a clean-room and legal review. Today we deliberately recorded no format details. |
| OQ-2 | Principle 1 versus writing tags, captions and coordinates into originals (META-01, GEO-02). | Sidecar by default; explicit opt-in write-back to XMP; originals untouched otherwise. |
| OQ-3 | Where does import live? The plan has no import phase. | Add a minimal import (copy into dated folders, skip duplicates by hash, verify before clearing a card) to late Phase 4 or an early Phase 6. |
| OQ-4 | Slideshow and quick viewer: which phase? | Phase 5 with Mac polish. |
| OQ-5 | Effects and presets: which to ship? | Core tone controls and a handful of looks in Phase 3; presets as user-shareable files after; no novelty looks. |
| OQ-6 | Positioning against Dalica and Picco. | Decide before the functional specification whether Leoht differentiates on Mac-native polish, testing rigour and transparent process, or collaborates. |

## Next step: functional specification

This inventory is intended as the input to a functional specification. Suggested path: (1) you review the Keep/Modernise/
Defer/Drop calls and open questions; (2) we turn the accepted rows into numbered requirements per phase, each with an
acceptance test (done as a draft: see `docs/spec/functional-spec.md`); (3) each requirement keeps its row ID (for example `EDIT-07`) so traceability from research to test is
unbroken; (4) each feature group gets an early trial (gate Gt) that validates its requirements before polish. Re-verify the live-site sources (sections C and D of `sources.md`) when the specification is written, because
repositories and vendor pages change.

