# Research source log

Every source relied on by the documents in `docs/research/`. Companion to
[`picasa-feature-inventory.md`](picasa-feature-inventory.md) (cites sources as `[S01]`, `[S16]`, and so on).

**Clean-room statement.** Nobody downloaded, installed, decompiled or inspected any Picasa binary, library, code or
asset in producing these documents. Only *public descriptions of behaviour* were read: archived vendor help pages,
Google blog posts, Wikipedia, and third-party reviews and tutorials. Where a source is a third-party open-source
project, only its published README/description was read for *behaviour and positioning*; no source code was read or
copied. No Google icons, screenshots or trademarks are reproduced. Quotes are limited to a few words.
See [`CLEAN_ROOM.md`](../../CLEAN_ROOM.md) (on the Phase 0 branch).

**Dates.** *Archive date* is the Wayback Machine snapshot timestamp (UTC date) actually read; the URL to open is
`https://web.archive.org/web/<snapshot>/<URL>`. *Accessed* is when we read it. All accessed dates are 2026-11-27
unless noted. "Live" means the current public page was read with no archive copy (for pages that still exist).
"Excerpt only" means we saw only a search-engine excerpt, not the whole page; it supports little and is flagged.

**Archive-format note.** The Google help pages are cited at their `support.google.com/picasa/answer/<id>` URLs;
the exact Wayback snapshot is in the table. Google replaced these pages after the product was retired.

## A. Google-published (archived)

| ID | Source (URL) | Archive date | Supported |
|---|---|---|---|
| S01 | Help: *Organize photos in Picasa* - `https://support.google.com/picasa/answer/156339` | 2016-04-27 | Folders mirror disk vs albums app-only; photo tray; create album; drag to move; face grouping; duplicate handling (hide, exclude on import, rebuild database). |
| S02 | Help: *Edit photos in Picasa* - `https://support.google.com/picasa/answer/156342` | 2016-01-30 | Basic fixes list; tuning; effects tabs; rotate/flip; resize via export/email/upload; watermark; side-by-side compare (A\|B, A\|A); Save vs Save a Copy vs Save As; originals moved to a hidden subfolder; JPEG output. |
| S03 | Help: *Choose which files to see in Picasa* - `https://support.google.com/picasa/answer/15625` | 2016-04-27 | Supported photo, video, audio and RAW file types; file-type options dialog. |
| S04 | Help: *Add or delete photos* - `https://support.google.com/picasa/answer/156341` | 2016-04-27 | Add one file; Folder Manager; import from camera/card (choose destination, delete after copy); web-album limits (context only). |
| S05 | Help: *Download, email, or move photos* - `https://support.google.com/picasa/answer/39513` | 2016-04-27 | Export to folder; email via Gmail or default mail client. |
| S06 | Help: *Slideshows, photo collages, and movies* - `https://support.google.com/picasa/answer/19539` | 2016-04-27 | Fullscreen slideshow from an album; loop option; collage creation; movie maker (photos, video, music); output folders under a Picasa projects folder. |
| S07 | Help: *Do more with Picasa & Picasa Web Albums* - `https://support.google.com/picasa/answer/66969` | 2016-04-24 | Sync-to-web switch; print-size options; keyboard shortcuts (select all, deselect, hold in tray, new album); web captions/location (context). |
| S08 | Help: *Share photos & albums* - `https://support.google.com/picasa/answer/39551` | 2016-01-26 | Upload to web albums/Google+; privacy levels; image size choice (online sharing, context). |
| S09 | Help: *Welcome to Picasa & Picasa Web Albums* - `https://support.google.com/picasa/answer/156347` | 2016-04-25 | Product summary: finds photos on the computer, originals preserved, edits visible only in Picasa until saved. |
| S10 | Help centre index - `https://support.google.com/picasa/?hl=en` | 2016-02-04 | Which help topics existed (used to find S01-S09). |
| S11 | *Picasa and Picasa Web Albums Release Notes* (Mac + Windows, 3.0-3.9) - `http://support.google.com/picasa/bin/picasa.google.com/support/bin/static.py?hl=en&page=release_notes.cs` | 2012-05-26 | Feature additions by version: name tags, Places/geotag, Tags tab and quick tags, import options, date/time adjust, Properties (EXIF) panel, colour management, Face Movie, batch upload, backup fixes, collage drag-and-drop, custom crop sizes, 3.9 effects list, side-by-side editing, WebP, database migration, album data written to ini. Covers builds up to March 2012; later 3.9.x builds (to Oct 2015, per S16) are not itemised here. |
| S12 | *picasa.google.com* home page - `https://picasa.google.com/` | 2016-03-01 | Final-era pitch: "Picasa 3.9"; manage photos locally, effects, upload and share. |
| S13 | Official Google Blog: *Introducing Picasa 3.0* (posted 2008-09-02) - `https://googleblog.blogspot.com/2008/09/introducing-picasa-30-and-big-changes.html` | 2016-03-25 | Sync-to-web; retouch/restore tools; automatic red-eye; text on images; overhauled collages/slideshows; movie maker. |
| S14 | Official Google Blog: *Picasa 3.5, now with name tags and more* (posted 2009-09-22) - `https://googleblog.blogspot.com/2009/09/picasa-35-now-with-name-tags-and-more.html` | 2016-02-12 | Desktop name tags via face grouping ("Unnamed People"); face collage; Google Maps geotagging; redesigned import. |
| S15 | Picasa Blog: *Moving on from Picasa* (posted 2016-02-12) - `https://googlephotos.blogspot.com/2016/02/moving-on-from-picasa.html` | 2016-03-13 | Retirement: desktop app unsupported from 2016-03-15; still worked locally with no updates. Later update notes (2018) say the app no longer works online. |

## B. Third-party descriptions of Picasa behaviour

| ID | Source (URL) | Archive date | Supported |
|---|---|---|---|
| S16 | Wikipedia: *Picasa* - `https://en.wikipedia.org/wiki/Picasa` | live (revision current on 2026-11-27) | Final release 3.9.141.259 (Windows) / 3.9.141.306 (Mac), 2015-10-09; per-folder ini and hidden originals folder; IPTC keywords in JPEG only; XMP parsing but no XMP keyword search; live search bar and operators; viewing and fullscreen; RAW/WebP; face recognition history; geotag writes EXIF; Linux via Wine. *Wikipedia is a tertiary source; where it matters we also have a primary or review source.* |
| S17 | Picasa Resources: *How Picasa Works* - `https://sites.google.com/site/picasaresources/picasa/how-picasa-works` | 2022-06-23 | Central database vs per-folder ini redundancy; database rebuild from metadata + ini; watching of Scan-Always folders; face tags in XMP. (Community-run Google Sites knowledge base, not official Google documentation.) |
| S18 | Picasa Resources: *Getting started with Picasa* - `https://sites.google.com/site/picasaresources/picasa/getting-started` | 2024-04-22 | Folders vs albums semantics; star with the space bar; "Select Starred Photos" then add to album; slideshow of an album. |
| S19 | Picasa Resources: *Missing photos troubleshooter* - `https://sites.google.com/site/picasaresources/picasa/how-to-find-missing-photos` | 2022-06-23 | Scan Always vs Scan Once vs Remove from Picasa semantics; thumbnails cached so Library need not read files; advice not to watch whole drives. |
| S20 | Picasa Resources: *How to rebuild the Picasa database* - `https://sites.google.com/site/picasaresources/picasa/how-to-rebuild-database` | 2022-06-23 | Rebuild procedure; Remove from Picasa does not delete files; sub-folder scan-state quirks. |
| S21 | Picasa Resources: *Quick Start Guide* - `https://sites.google.com/site/picasaresources/picasa/quick-start-guide` | 2024-04-22 | Folder Manager states shown by icons; per-folder switch to turn face detection off; scanner-output folder tip. |
| S22 | M. Boesen: *Review of Picasa 3 Picture Management Application* (content dated 2010-09-13, Picasa 3.8.0.240 Mac) - `http://www.pcug.org.au/boesen/picasa3/picasa3.htm` | 2016-04-07 | Watched folders; folder tree vs flat list; hide/unhide; locate on disk; batch rename; albums as pointers with drag-reorder; import behaviour; editing tabs; history persists in a hidden file; Save-to-disk workflow with undo; search matches folder names, file names, tags and captions by word prefix; IPTC tags and captions; export vs saved copies. |
| S23 | M. Henderson, MakeUseOf: *How To Use Special Effects on Images in Google Picasa* (2009-04-08) - `https://www.makeuseof.com/tag/picasa-101-how-to-master-the-special-effects-in-picasa/` | 2016-03-18 | Behaviour of the first effects tab (sharpen, sepia, B&W, warmify, film grain, tint, saturation, soft focus, glow, filtered B&W, focal B&W, graduated tint); effects can be undone and stacked; apply/cancel flow. |
| S24 | PicasaGeeks: *Original vs Edited Photo in Picasa* (2016-01-18) - `https://picasageeks.com/2016/01/original-vs-edited-photo-in-picasa/` | 2016-02-19 | Edits overlay the untouched original; other programs see the original; why edits are not auto-saved; save-to-disk makes a backup of originals. |
| S25 | PicasaGeeks: *Backup or Gift CD? What's the Difference?* (2011-08-02) - `https://picasageeks.com/2011/08/picasa-tip-backup-or-gift-cd-whats-the-difference/` | 2016-06-14 | Backup Pictures keeps originals plus ini files; Gift CD contains edited exports only. |
| S26 | Northbrook (Illinois) library handout: *Picasa 3.9 Photo Editing and Web Albums* (PDF) - `https://northbrook.info/sites/default/files/PICASA%203.9%20and%20WEB%20ALBUMS.pdf` | 2015-09-26 | Plain-language definitions of retouch, text, fill light, highlights, shadows, colour temperature, neutral colour picker, and the first effects tab. |

## C. Comparable apps (licence and model verification)

| ID | Source (URL) | Archive date | Supported |
|---|---|---|---|
| S27 | digiKam, KDE Applications page - `https://apps.kde.org/digikam/` | live | Feature summary (albums, tags/ratings/labels, search by EXIF/IPTC/XMP, LibRaw, FFmpeg). *Seen as a search-engine excerpt of the live page, not read in full.* |
| S28 | digiKam source tree (licence directory only) - `https://invent.kde.org/graphics/digikam` (`LICENSES/GPL-2.0-or-later.txt` present) | live | Licence: GPL-2.0-or-later. Only the licence file name was checked; no code read. |
| S29 | XnView MP download page and FAQ - `https://www.xnview.com/en/xnview-mp/`, `https://www.xnview.com/en/faq/` | live | Freeware for private/educational/non-profit use; paid licence for companies; Windows/macOS/Linux. |
| S30 | XnSoft end-user licence text (as shipped in the Chocolatey package page) - `https://community.chocolatey.org/packages/xnviewmp.install/1.5.5` | live (2023 package) | Proprietary EULA: no reverse engineering, no sub-licensing. |
| S31 | FastStone order page - `https://www.faststone.org/order.htm` and home page `https://www.faststone.org/` | live | Image Viewer free for personal/educational use; paid for commercial; version 8.5 (2026-06-24); Windows-only claims not verified here. |
| S32 | FastStone Image Viewer licence text - `https://documentation.help/FastStone-Image-Viewer/License.htm` | live | Freeware terms; no decompiling or reverse engineering. |
| S33 | Shotwell project - `https://gitlab.gnome.org/GNOME/shotwell` (README, `COPYING`) | live | Licence LGPL-2.1-or-later; GNOME photo manager; import, keywords/events, full-window and fullscreen viewing. |
| S34 | gThumb - `https://wiki.gnome.org/Apps/Gthumb` and `https://gitlab.gnome.org/GNOME/gthumb` (`COPYING`) | live | GPL (COPYING is GPL v2 text; "or later" wording per file headers **not verified**); viewer/browser for GNOME with camera importer and plug-ins. |
| S35 | Apple Support: *Change where photos and videos are stored on Mac* - `https://support.apple.com/guide/photos/change-where-photos-and-videos-are-stored-pht1ed9b966d/mac` | live | Library is a copy-in by default; referenced files option; caveats (no iCloud, not backed up, breaks on move/rename); Consolidate. |
| S36 | Adobe: *Save metadata to external sidecar files* (Lightroom Classic; updated 2025-10-27) - `https://helpx.adobe.com/lightroom-classic/desktop/organize-photos-in-lightroom-classic/create-xmp-acr-files.html` | live | Catalogue stores metadata; optional automatic write to XMP sidecars; manual save; ACR sidecar for heavy edits from LrC 15.0. |
| S37 | Capture One Support: *Sessions vs Catalogs* (2025-09-21) - `https://support.captureone.com/hc/en-us/articles/30041173920029-Sessions-vs-Catalogs-in-Capture-One-how-they-work-and-where-to-store-them` | live | Session (portable folder structure) vs Catalog (library model); referenced files; cloud-sync caution. |
| S38 | Capture One Support: *Session sidecar files explained* - `https://support.captureone.com/hc/en-us/articles/360002862137-Session-sidecar-files-explained-What-s-inside-the-CaptureOne-folder` | live | Per-session sidecar settings in a CaptureOne folder. **Excerpt only** (not read in full). |
| S39 | darktable user manual, *storage* - `https://docs.darktable.org/usermanual/development/en/preferences-settings/storage/`; repo `https://github.com/darktable-org/darktable` | live | XMP sidecars as redundant fail-safe next to the database; write on import vs after edit; detect externally edited XMP; database snapshots. Licence GPL-3.0 (GitHub licence field, 2026-11-27). |
| S40 | RawPedia: *Sidecar Files - Processing Profiles* - `https://rawpedia.pixls.us/sidecar_files_-_processing_profiles/`; repo `https://github.com/RawTherapee/RawTherapee` | live | `.pp3` text sidecars next to photos or in a cache; thumbnails from embedded JPEG until edited; neutral defaults for non-RAW; partial-profile paste; version-compatibility caveats. Licence GPL-3.0 (GitHub licence field). |
| S41 | RapidRAW - `https://github.com/CyberTimon/RapidRAW` | live | Licence AGPL-3.0 (GitHub licence field and README badge); RAW editor; README notes RAW decoding via `rawler`/dnglab. Last push 2026-11-27. |
| S42 | Immich - `https://github.com/immich-app/immich` | live | Licence AGPL-3.0 (GitHub licence field and README badge); self-hosted server model. |
| S43 | PhotoPrism - `https://github.com/photoprism/photoprism` (`LICENSE`, README) | live | Licence text is AGPL-3.0; GitHub reports it as "Other/NOASSERTION", and README describes paid membership features, so treat as AGPL-3.0 **with open-core extras, verify**. Local-first index, labels, faces, places. |

## D. Picasa-inspired and Picasa-replacement projects

Metadata (licence, last push, latest release, language) is from the GitHub API on 2026-11-27 and the project's own README;
nothing here was installed or run.

| ID | Source (URL) | Archive date | Supported |
|---|---|---|---|
| S44 | PicasaPy - `https://github.com/sanchomuzax/PicasaPy` | live | GPL-3.0; Python/Qt Quick; reads and writes Picasa's per-folder metadata format; features list. |
| S45 | Picco - `https://github.com/miamibc/picco` | live | Apache-2.0; Rust + egui; "keeps your Picasa library alive"; features list; macOS build exists. |
| S46 | Picasso - `https://github.com/kuopinghsu/Picasso` | live | MIT; Rust; "Inspired by Picasa"; imports stars/captions/rotation from Picasa metadata. |
| S47 | Dalica - `https://github.com/pkkid/dalica` | live | Apache-2.0; Rust; "faithfully re-creates" Picasa workflows; docs describe clean-room intent; sidecar + cache index; agent-drivable. |
| S48 | photon - `https://github.com/bsg62/photon` | live | MIT; Rust (Tauri); "spiritual successor to Picasa 3"; shares stars with Picasa's ini. |
| S49 | PIC (Personal Image Catalogue) - `https://github.com/froggy744/PIC` | live | MIT; Rust + GTK4/libadwaita; Linux only; "inspired by classic Picasa and iPhoto". |
| S50 | Lap - `https://github.com/julyx10/lap` | live | GPL-3.0-or-later; Tauri + Vue + Rust; folder-first, local AI search; not stated in README to be Picasa-derived. |
| S51 | Reddit r/Windows11 post by Lap's author (2026-04-21) - `https://www.reddit.com/r/Windows11/comments/1srasfg/i_built_a_modern_opensource_photo_manager_for/` | live (search excerpt) | Author states he used Picasa for years and built Lap as a replacement. **Excerpt only.** |
| S52 | Lap blog: *Best Picasa Replacement in 2026* - `https://lapphotos.com/blog/best-picasa-replacement/` | live (search excerpt) | A vendor-written list of what Picasa users valued. **Excerpt only; vendor blog, treat as opinion.** |
| S53 | Lumio Photo - `https://github.com/coldtrace00/LumioPhoto` | live | MIT; C++/Qt; Windows; "open replacement for Google Picasa"; keeps stars/tags/edits in its own database, not in the photo folders. |
| S54 | PicaCatalog - `https://github.com/peterfixit/picacatalog` | live | MIT; offline catalogue "inspired by Picasa 3"; beta; Linux. |
| S55 | Picasera - `https://github.com/goldcarib/picasera` | live | GPL-3.0; Tauri + React + Rust; "modern Picasa replacement"; early, last push 2026-01-11. |
| S56 | picasa (ganeshapp) - `https://github.com/ganeshapp/picasa` | live | MIT; Tauri 2; "in the spirit of Google's old Picasa"; v0.1.0 only. |
| S57 | FlyPhotos - `https://github.com/riyasy/FlyPhotos` | live | Windows viewer, described as a replacement for Picasa Photo Viewer; GitHub reports licence as "Other/NOASSERTION" (verify). |
| S58 | Iris - `https://github.com/eliocro/iris` | live | No licence stated; Electron; macOS viewer "inspired by Picasa". |
| S59 | Micasa - `https://github.com/ImRIzo/micasa` | live | BSD-2-Clause; SwiftUI; macOS viewer/editor "inspired by Picasa". |
| S60 | Spica Photo Viewer - `https://github.com/hiz8/spica-photo-viewer` | live | MIT; Tauri; Windows viewer inspired by Picasa Photo Viewer. |
| S61 | AJANTA Media Suite - `https://github.com/raaz2507/AJANTA-Image-Suite` | live | No licence stated; C++; tribute to Picasa's speed and non-destructive editing; README in Hindi (read via translation of headings only). |
| S62 | picasa2digikam - `https://github.com/Philipp91/picasa2digikam` | live | GPL-3.0; migration script from Picasa metadata to digiKam; shows what Picasa data users want to keep (stars, albums, faces). |
| S63 | Krumassan Image Viewer - `https://github.com/sergkrumas/image_viewer` | live | GPL-3.0; PyQt; Windows; alpha; "replace Google Picasa Photo Viewer". |
| S64 | AlternativeTo: Picasa alternatives (open source filter; updated 2025-10-01) - `https://alternativeto.net/software/picasa/?license=opensource` | live (search excerpt) | Lists digiKam, ImageGlass, nomacs, Shotwell, gThumb, Pix, FlyPhotos as alternatives. **Excerpt only.** |
| S65 | papa - `https://github.com/shaeberling/papa` | live | "Picasa Alternative Photo App"; no licence, no README, one commit in 2017. Listed only to say we found and dismissed it. |
| S66 | PhotoManager (Hawkynt) - `https://github.com/Hawkynt/PhotoManager` | live | LGPL-3.0; C#; folders + XMP sidecars, no database; borrows Picasa-style picks/rejects. *Adjacent, not a Picasa replacement.* |

## E. Leoht-internal references

| ID | Source | Supported |
|---|---|---|
| S67 | `LEOHT_PLAN.md` (phases 0-6+, principles) and `docs/adr/0001`-`0005` on branch `phase0/scaffold` (PR #1) | Phase numbers used for recommendations; sidecar and data-model decisions that the inventory is consistent with. |

## Unsourced or uncertain statements (verify before relying on them)

Marked **unsourced, verify** in the inventory too.

1. *Licence/pricing of proprietary comparables* (Apple Photos, Lightroom Classic, Capture One): "proprietary, commercial" is general knowledge; none of S35-S38 states the licence. **Unsourced, verify.**
2. *Exact count of Picasa effects.* The 12 first-tab effects (S23, S26) plus the 26 names in the 3.9 release notes (S11) suggest about 38, but we did not find an official total. **Unsourced, verify.**
3. *Timeline view details.* S11 and S16 confirm a Timeline existed; its exact behaviour (grouping, scrubbing) is from general knowledge. **Unsourced, verify.**
4. *Red-eye "auto" vs manual behaviour and Retouch algorithm.* S02, S11, S13 confirm both exist; how they worked internally is not documented in our sources and is irrelevant to us.
5. *Keyboard shortcuts beyond those in S07* (for example Ctrl+Alt fullscreen from S16, flip shortcuts from S02) were not cross-checked.
6. *The exact on-disk layout of Picasa's per-folder file and its database* is deliberately **not** recorded. Third-party write-ups exist (they come from reverse-engineering), so we did not open them. A web search incidentally displayed a few field names from such write-ups; we did not follow those links, and nothing from them is used here. See open question OQ-1 in the inventory.
7. *Whether Picasa preserved edits when files were moved outside Picasa* is only partly documented (S01, S22 say outside changes are mostly recognised; edits tied to the ini file in the folder per S16/S17).
8. *Mac vs Windows differences.* S22 says functionality was virtually identical on Mac 3.8; Mac-only behaviour in 3.9 is otherwise unsourced.
9. *Leading-minus exclusion in Picasa's search.* Reported only by an older Wikipedia mirror (en-academic.com, excerpt seen via search); the current Wikipedia text does not mention it. **Unsourced, verify.**
10. *Dates of the last 3.9.x builds* beyond S11 (March 2012) rely on S16 only.
