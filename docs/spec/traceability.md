# Traceability: inventory to requirements

Every row of the [Picasa feature inventory](../research/picasa-feature-inventory.md) and its Leoht additions, mapped to the requirements in [`functional-spec.md`](functional-spec.md) or to a disposition. Kept in step with the requirement headers; when written, every Keep, Modernise and Add row and every `MOD` item had at least one requirement.

| Inventory ID | Feature | Inventory call | Requirements or disposition |
|---|---|---|---|
| LIB-01 | Watched folders | Keep | FR-LIB-01, FR-LIB-02 |
| LIB-02 | Scan-once folders | Modernise | FR-FIL-04 |
| LIB-03 | Ignore a folder | Keep | FR-LIB-03 |
| LIB-04 | Folder list mirrors disk | Keep | FR-BRW-01 |
| LIB-05 | Hide photos or folders | Keep | FR-FIL-01 |
| LIB-06 | Reveal on disk | Keep | FR-BRW-08 |
| LIB-07 | Folder operations inside the app | Modernise | FR-FIL-02 |
| LIB-08 | Delete means delete | Modernise | FR-FIL-03 |
| LIB-09 | Add a single file | Defer | FR-MAC-07 (partly: row is deferred; only the stated aspect is specified) |
| LIB-10 | Central index with cached thumbnails | Keep | FR-FND-01, FR-FND-02, FR-LIB-01, FR-LIB-07 |
| LIB-11 | Rebuild the index | Modernise | FR-FND-02, FR-LIB-08 |
| LIB-12 | Move the index to another drive | Defer | Out of scope for now (deferred); revisit after gate Gf |
| LIB-13 | Choose which file types to show | Modernise | FR-LIB-04, FR-LIB-05 |
| LIB-14 | Video files in the library | Defer | Out of scope for now (deferred); revisit after gate Gf |
| LIB-15 | Per-folder face-scan opt-out | Keep | FR-LAT-03 |
| LIB-16 | Network drive support | Defer | Out of scope for now (deferred); revisit after gate Gf |
| LIB-17 | Colour management switch | Modernise | FR-BRW-06 |
| ORG-01 | Folders versus albums | Keep | FR-ALB-01 |
| ORG-02 | Album create and edit | Keep | FR-ALB-02 |
| ORG-03 | Photo tray | Modernise | FR-RAT-08 |
| ORG-04 | Manual order inside folders and albums | Keep | FR-ALB-04 |
| ORG-05 | Album data survives reinstall | Modernise | FR-ALB-03 |
| ORG-06 | Star favourite | Modernise | FR-RAT-01 |
| ORG-07 | Captions | Keep | FR-RAT-03 |
| ORG-08 | Tags (keywords) | Modernise | FR-RAT-04 |
| ORG-09 | Timeline | Keep | FR-LIB-06, FR-ALB-05 |
| ORG-10 | Live search box | Modernise | FR-FND-07, FR-ALB-06, FR-ALB-07 |
| ORG-11 | Colour search | Defer | Out of scope for now (deferred); revisit after gate Gf |
| ORG-12 | Selection shortcuts | Keep | FR-BRW-03 |
| ORG-13 | Batch rename | Modernise | FR-LAT-08 |
| ORG-14 | Adjust date and time | Modernise | FR-LAT-08 |
| ORG-15 | Collaborative albums | Drop | Out of scope (dropped) |
| VIEW-01 | Thumbnail grid | Modernise | FR-FND-07, FR-BRW-02, FR-BRW-11 |
| VIEW-02 | Lightbox (edit view) | Keep | FR-BRW-04 |
| VIEW-03 | Properties panel | Keep | FR-LIB-06, FR-BRW-05 |
| VIEW-04 | Fullscreen slideshow | Keep | FR-MAC-06 |
| VIEW-05 | Side-by-side compare | Keep | FR-CRP-07 |
| VIEW-06 | Fast standalone viewer | Modernise | FR-MAC-07 |
| VIEW-07 | Slideshow burned to disc | Drop | Out of scope (dropped) |
| VIEW-08 | Screensaver | Drop | Out of scope (dropped) |
| EDIT-01 | Crop with presets | Keep | FR-CRP-01 |
| EDIT-02 | Straighten | Keep | FR-CRP-02 |
| EDIT-03 | Rotate and flip | Keep | FR-CRP-03 |
| EDIT-04 | Red-eye correction | Keep | FR-ADJ-06 |
| EDIT-05 | Auto contrast and auto colour | Modernise | FR-ADJ-04 |
| EDIT-06 | One-click 'lucky' fix | Modernise | FR-ADJ-05 |
| EDIT-07 | Fill light, highlights, shadows | Keep | FR-ADJ-01 |
| EDIT-08 | Colour temperature and neutral picker | Keep | FR-ADJ-02 |
| EDIT-09 | Retouch | Defer | Out of scope for now (deferred); revisit after gate Gf |
| EDIT-10 | Text on image | Defer | Out of scope for now (deferred); revisit after gate Gf |
| EDIT-11 | Sharpen and saturation | Keep | FR-ADJ-02, FR-ADJ-03 |
| EDIT-12 | Core look effects | Keep | FR-ADJ-07 |
| EDIT-13 | Film grain, glow, soft focus | Defer | Out of scope for now (deferred); revisit after gate Gf |
| EDIT-14 | Local-area effects | Modernise | FR-LAT-10 |
| EDIT-15 | Third-tab effect packs (3.9) | Defer | Out of scope for now (deferred); revisit after gate Gf |
| EDIT-16 | Online editing add-ons | Drop | Out of scope (dropped) |
| EDIT-17 | Stackable, undoable edits | Keep | FR-ENG-01, FR-ENG-03, FR-ENG-04 |
| EDIT-18 | Edits as an overlay | Modernise | FR-FND-04, FR-ENG-01, FR-ENG-05, FR-RAT-06 |
| EDIT-19 | Per-folder edit store | Modernise | FR-FND-02, FR-FND-03, FR-ENG-02 |
| EDIT-20 | Save writes a new picture; original kept | Modernise | FR-FND-04, FR-ENG-06 |
| EDIT-21 | Save a Copy and Save As | Keep | FR-ENG-06 |
| EDIT-22 | Output format | Modernise | FR-ENG-08 |
| EDIT-23 | RAW handled non-destructively | Keep | FR-LIB-04, FR-ENG-10 |
| EDIT-24 | Edit many at once | Keep | FR-LAT-07 |
| FACE-01 | Face detection and grouping | Modernise | FR-LAT-03 |
| FACE-02 | Name tags | Keep | FR-LAT-04 |
| FACE-03 | People albums | Keep | FR-LAT-04 |
| FACE-04 | Where face data lived | Modernise | FR-LAT-05 |
| FACE-05 | Face collage and Face Movie | Drop | Out of scope (dropped) |
| FACE-06 | Contacts and online name sync | Drop | Out of scope (dropped) |
| GEO-01 | Geotag on a map | Modernise | FR-LAT-06 |
| GEO-02 | Coordinates written into the file | Modernise | FR-RAT-07, FR-LAT-06 |
| GEO-03 | View geotagged photos on a map | Keep | FR-LAT-06 |
| GEO-04 | Album place (web) | Drop | Out of scope (dropped) |
| IMP-01 | Import from camera, card or scanner | Keep | FR-LAT-02 |
| IMP-02 | Skip duplicates on import | Modernise | FR-LAT-02 |
| IMP-03 | Auto-folder by date | Keep | FR-LAT-02 |
| IMP-04 | Delete from card after copy | Modernise | FR-LAT-02 |
| IMP-05 | Star or upload while importing | Defer | Out of scope for now (deferred); revisit after gate Gf |
| IMP-06 | Third-party library import | Drop | Out of scope (dropped) |
| OUT-01 | Export to folder | Keep | FR-ENG-07 |
| OUT-02 | Resize and watermark on export | Keep | FR-ENG-07, FR-LAT-09 |
| OUT-03 | Email photos | Modernise | FR-MAC-08 |
| OUT-04 | Print | Defer | Out of scope for now (deferred); revisit after gate Gf |
| OUT-05 | Collage maker | Defer | Out of scope for now (deferred); revisit after gate Gf |
| OUT-06 | Movie maker | Drop | Out of scope (dropped) |
| OUT-07 | Gift disc | Drop | Out of scope (dropped) |
| OUT-08 | Backup pictures | Modernise | FR-LAT-11 |
| OUT-09 | Web album upload and sync | Drop | Out of scope (dropped) |
| OUT-10 | Embed and link sharing | Drop | Out of scope (dropped) |
| META-01 | Keywords and captions written into the file | Modernise | FR-RAT-05, FR-RAT-07 |
| META-02 | Duplicate handling | Modernise | FR-FIL-06 |
| META-03 | Hidden helper folders and files | Modernise | FR-FND-04 |
| META-04 | Per-folder data as a safety net | Keep | FR-FND-02, FR-FND-03, FR-ENG-02 |
| META-05 | Keyboard shortcuts and preferences | Keep | FR-BRW-03, FR-BRW-11, FR-MAC-01, FR-MAC-02 |
| META-06 | Localisation | Defer | FR-FND-12 (partly: row is deferred; only the stated aspect is specified) |
| META-07 | Platforms | Keep | FR-FND-11, FR-MAC-01, FR-MAC-03, FR-LAT-14 |
| MOD-01 | HEIC/HEIF photos | Modern expectation | FR-LIB-04 |
| MOD-02 | RAW without heavy dependencies | Modern expectation | FR-LIB-04, FR-ENG-10 |
| MOD-03 | Dark mode and system appearance | Modern expectation | FR-BRW-07 |
| MOD-04 | Accessibility | Modern expectation | FR-FND-05, FR-FND-06, FR-MAC-05 |
| MOD-05 | HiDPI, wide-gamut and colour-managed display | Modern expectation | FR-BRW-06, FR-ENG-08 |
| MOD-06 | Smarter auto-enhance | Modern expectation | FR-ENG-09, FR-ADJ-04, FR-ADJ-05 |
| MOD-07 | Private, on-device intelligence | Modern expectation | FR-FND-08, FR-LAT-03 |
| MOD-08 | Safe file operations | Modern expectation | FR-FIL-03 |
| MOD-09 | Cloud-synced and shared folders | Modern expectation | FR-FIL-05 |
| MOD-10 | Offline and removable volumes | Modern expectation | FR-FIL-04 |
| MOD-11 | Standard metadata interchange | Modern expectation | FR-RAT-06 |
| MOD-12 | Very large libraries | Modern expectation | FR-FND-07, FR-LIB-09, FR-BRW-02, FR-ALB-06, FR-LAT-13 |
| MOD-13 | Scriptable core | Modern expectation | FR-FND-09 |
| MOD-14 | Related-file groups | Modern expectation | FR-FIL-07 |
| MOD-15 | Unicode and long paths | Modern expectation | FR-FND-10 |
| ADD-01 | Geometry suite | Add | FR-CRP-01, FR-CRP-02, FR-CRP-04, FR-CRP-05, FR-CRP-06, FR-CRP-08 |
| ADD-02 | Culling mode | Add | FR-BRW-09, FR-RAT-02 |
| ADD-03 | Copy and paste edits across a selection | Add | FR-ADJ-08 |
| ADD-04 | Burst and similar-photo stacks with best-shot pick | Add | FR-FIL-07, FR-FIL-08, FR-LAT-12 |
| ADD-05 | Correct wide-colour and HDR display | Add | FR-BRW-06, FR-MAC-04 |
| FUT-01 | Hold-to-compare and split before/after | Future possibility | Not committed (future) |
| FUT-02 | Named edit history, snapshots and virtual copies | Future possibility | Not committed (future) |
| FUT-03 | Explainable auto-enhance | Future possibility | Not committed (future) |
| FUT-04 | Highlight and shadow recovery from HEIC, RAW and HDR headroom | Future possibility | Not committed (future) |
| FUT-05 | Instant grid at 100k photos, 60 fps, with CI budgets | Future possibility | FR-FND-07, FR-LAT-13 (promoted from future) |
| FUT-06 | Background indexing with no import wait | Future possibility | FR-FND-07, FR-LIB-09 (promoted from future) |
| FUT-07 | Live Photo and RAW+JPEG pairing | Future possibility | Not committed (future) |
| FUT-08 | On-device content and colour search | Future possibility | Not committed (future) |
| FUT-09 | Safe in-app move and rename that carries sidecars | Future possibility | FR-FIL-02 (promoted from future) |
| FUT-10 | Quick Look, Finder integration, share sheet and drag to Mail or Messages | Future possibility | Not committed (future) |
| FUT-11 | Accessibility | Future possibility | FR-FND-05, FR-FND-06, FR-MAC-05 (promoted from future) |
| FUT-12 | Originals guarantee | Future possibility | FR-FND-01 (promoted from future) |
| FUT-13 | Export-all-edits | Future possibility | Not committed (future) |
| FUT-14 | Orphaned-sidecar health check | Future possibility | FR-ENG-11 (promoted from future) |

Other traces (plan, ADR, decision or principle), alongside any inventory rows above:

| Requirement | Other trace |
|---|---|
| FR-FND-01 | ADR-0002 |
| FR-FND-02 | ADR-0001, ADR-0002 |
| FR-FND-03 | ADR-0002 |
| FR-FND-08 | Core Principle 3 |
| FR-FND-13 | Positioning (inventory) |
| FR-FND-14 | SQ-6 (Keith decision, 2026-11-27) |
| FR-LIB-10 | ADR-0004 (untrusted input) |
| FR-BRW-10 | ADR-0004 |
| FR-ENG-04 | Plan Phase 3 |
| FR-ENG-09 | ADR-0003 |
| FR-ENG-11 | ADR-0002 |
| FR-CRP-06 | ADR-0001 |
| FR-RAT-01 | SQ-4 (Keith decision, 2026-11-27) |
| FR-RAT-05 | OQ-2 |
| FR-RAT-07 | OQ-2 |
| FR-ALB-01 | ADR-0001 |
| FR-ALB-03 | ADR-0001 |
| FR-MAC-03 | Plan Phase 5 |
| FR-MAC-06 | OQ-4 |
| FR-MAC-07 | OQ-4 |
| FR-LAT-01 | OQ-1 (resolved), sources S44-S48 for context only |
| FR-LAT-02 | OQ-3 |
| FR-LAT-14 | Plan Phase 6 |
