# Leoht: A Modern, Folder-First Photo Management System

**Project Goal**: Build a production-grade, open-source photo organizer and light editor inspired by the simplicity of Google Picasa (discontinued ~2016). Prioritize natural filesystem folder structures, non-destructive per-picture enhancements, and a clean user experience.

**Platform Strategy**: Mac first (native feel where possible), then Linux and Windows.

**Development Approach**: Agentic software development — iterative, well-documented steps using AI agents (similar in spirit to multi-agent C compiler efforts, functional AWK interpreters built with Claude, and spec-driven AI workflow kits). Each phase produces working code, tests, documentation, and a narrative of the process. All work lives in a public GitHub repository with GitHub Actions CI and CodeRabbit for automated, high-quality code review.

**Legal Stance**: Fully clean-room. No reverse engineering of proprietary binaries, no copying of Google code or trademarks. Original design and implementation using only open-source libraries and public knowledge of desired behaviors. Working project name is "Leoht" (Old English for "light"). A full trademark clearance (USPTO, EUIPO, domains, common-law) is recommended before public branding or a permanent GitHub repository name.

## Core Principles (from Picasa lessons)

1. **Filesystem is the source of truth** — Folders on disk are the primary organization. The app indexes them; it does not own or relocate originals unless explicitly asked.
2. **Non-destructive edits by default** — Edits stored as instructions (sidecar JSON or in a lightweight database). Originals remain untouched. Export or explicit "Save" applies changes.
3. **Simple, fast, local-first** — No mandatory cloud. Optional later features.
4. **Modernized facilities** — Better RAW support, modern face/object recognition (on-device), improved performance, accessibility, dark mode, etc.
5. **High-quality, maintainable code** — Small, focused files; highly readable Rust that follows community standards; clear comments and documentation; rigorous testing including performance and large-volume scenarios.
6. **Excellence at a focused core first** — Make organise, view, crop and adjust clearly excellent before widening scope. New features are added only once the core meets its quality bar.

## Recommended Technology Stack (for agentic velocity + production path)

- **Core language**: Rust (performance, safety, excellent cross-platform story, strong AI agent familiarity with the language). Strict adherence to Rust idioms, `rustfmt`, and `clippy`.
- **UI**:
  - Phase 1–2: egui (immediate mode, pure Rust, fast to prototype — used by projects like Picco).
  - Later / production polish: Tauri (Rust backend + modern web frontend) for richer UI while keeping small footprint, or native SwiftUI on Mac with shared Rust core via FFI.
- **Image processing**: `image` crate, `vips` bindings or pure-Rust where possible, `rawloader` / `libraw` for RAW. GPU acceleration later (wgpu).
- **Indexing / metadata**: SQLite (via `rusqlite` or `sqlx`). Sidecar files (`.leoht.json` or XMP-compatible) for edit recipes so folders remain portable.
- **Face / AI features**: On-device models (e.g., via `tract` or ONNX, or Apple Vision on Mac). Optional.
- **Build / packaging / quality**: Cargo. GitHub as the canonical repository. GitHub Actions for CI (lint, test, build on Mac/Linux/Windows). CodeRabbit for AI-assisted code review on every PR. Emphasis on small file sizes, readable modules, and excellent comments.

This stack draws from successful open projects (Picco, PIC, RapidRAW, etc.) while remaining original.

## Phased Development Plan (Agentic)

Each phase is a self-contained increment. Agents can work in parallel on isolated components with clear specs, tests, and integration points. Document every decision, prompt pattern, failure, and lesson. All changes go through GitHub PRs with CodeRabbit review and passing CI.

### Phase 0: Foundation & Documentation (this document + repo scaffold)
- Create GitHub repository structure.
- Define data model (photos, folders, albums as virtual collections, edit recipes).
- Write initial architecture decision records (ADRs).
- Set up GitHub Actions CI skeleton (fmt, clippy, unit tests) and testing harness.
- Establish code quality standards (file size limits, comment conventions, Rust style).

### Phase 1: Folder Scanner & Index
- Walk directories, detect image files (JPEG, PNG, HEIC, RAW, etc.).
- Extract basic EXIF.
- Generate thumbnails (cached).
- Store in SQLite with watch for filesystem changes.
- CLI demo first.
- Unit + integration tests; volume tests with synthetic libraries of 1k–10k images; basic performance benchmarks.

#### Phase 1 format matrix (added 2026-11-26, at Keith's request)

Priority order and support level for the initial scanner + thumbnail + index. Rationale, HEIC crate findings and patent note: [ADR-0004](docs/adr/0004-image-format-support-strategy.md).

| # | Format | Phase 1 support level |
|---|--------|-----------------------|
| 1 | **JPEG** | Full first-class: decode, EXIF, embedded thumbnail, sidecar edits, volume-test baseline. |
| 2 | **HEIC/HEIF** | First-class via a pure-Rust crate (`heifer` or `heic-rs` preferred; `libheif-rs` as optional fallback behind a cargo feature). Choice and patent note documented in ADR-0004. |
| 3 | **PNG** | Full support. |
| 4 | **RAW** (common formats) | Detection + basic metadata + thumbnail where possible (e.g. embedded preview); deeper decoding can follow. |
| 5 | **Others** (TIFF, WebP, etc.) | Detection only in Phase 1; full support later. |

### Phase 2: Basic Viewer & Grid UI
- Display folder hierarchy mirroring disk.
- Thumbnail grid, lightbox viewer.
- Simple navigation.
- UI tests and performance checks on large grids.

### Phase 3: Non-Destructive Edit Engine
- Define edit recipe format (crop, rotate, straighten, exposure, contrast, saturation, red-eye, etc.).
- Apply on-the-fly for preview.
- Persist in sidecar or DB.
- Export path that bakes edits.
- Golden-image tests; performance tests on batches of edits.

### Phase 4: Organization Features
- Stars, captions, tags.
- Virtual albums.
- Search (filename, EXIF, tags).
- Duplicate detection (optional).
- Search performance tests on large indexes.

### Phase 5: Mac Polish & Packaging
- Native windowing, menus, drag-and-drop.
- Notarization path.
- Preferences, keyboard shortcuts.
- Full CI builds and packaging on macOS.

### Phase 6+: Cross-platform, advanced features
- Linux / Windows builds via GitHub Actions.
- Face recognition / clustering (on-device).
- Geotagging map view.
- Batch operations.
- Performance optimization, large library support (10k–100k+ photos) with dedicated volume and benchmark suites.
- Optional sharing / export galleries.

## Agentic Techniques to Apply

- **Spec-driven**: Write precise Markdown specs and ADRs before coding each component.
- **Isolated agents**: Different agents (or sequential sessions) own scanner, UI, edit engine, packaging. Shared interface contracts.
- **Testing harness**: Comprehensive suite including unit tests, integration tests, golden-image tests for edits, filesystem fixture libraries, regression suite that runs on every change, plus dedicated performance benchmarks and volume/scale tests (synthetic large libraries). All enforced in GitHub Actions CI.
- **Code quality standards**: Small, focused source files; highly readable code following Rust community standards (`rustfmt`, `clippy` with high strictness); clear, useful comments and module-level documentation. CodeRabbit reviews every PR for these qualities.
- **Documentation as first-class**: Each phase ends with a narrative "How we built X" that records prompts, iterations, surprises, and lessons — turning the project into a public case study of agentic application development. All docs live in the GitHub repo.
- **Incremental bootstrapping**: Start with the smallest useful system (CLI scanner + thumbnail gen), then layer UI and features. Never big-bang.
- **Human-in-the-loop gates**: Clear checkpoints for review of architecture, UX, and legal/privacy decisions. GitHub Issues and PRs as the coordination surface.

## Immediate Next Steps

1. Scaffold the Rust project with basic structure in a new GitHub repository, including CI and CodeRabbit configuration.
2. Implement Phase 1 scanner with a strong initial test suite (including volume tests).
3. Capture the process in `docs/process/`.

This project is deliberately designed to be a transparent, reproducible example of building a real desktop application with AI agents, emphasizing production-grade quality, rigorous testing, and maintainable Rust code from the start.

---
*Initial plan drafted as the first artifact of the Leoht agentic development process. Working title pending formal name clearance.*

## Appendix A: Dependencies and licences

*Added 2026-11-27 at Keith's request. Not part of Keith's original plan text. **Not legal advice**; consult an IP lawyer before any commercial release (see [ADR-0005](docs/adr/0005-licensing-and-commercial-readiness.md)).
Policy: only permissive licences in the default build, enforced by `cargo-deny` (`deny.toml`); anything else needs an ADR.*

**Verification legend** (all checked 2026-11-27): **[M]** `cargo metadata` of this workspace with `--all-features` (the licence field of the version in `Cargo.lock`);
**[C]** crates.io API metadata for the latest stable version shown; **[R]** a licence/README file in the project's official repository; **[W]** a web source that is not the project itself.
"Unverified" means we could not confirm it from an official source.

**Status:** *in use* = a dependency of this workspace today · *planned* = intended in a later phase of the plan · *candidate* = under evaluation, not committed · *rejected* = decided against.

### A.1 In use (Phase 0 workspace)

| Library (version) | Role | Phase | Licence (source) | Commercial notes | Status |
|---|---|---|---|---|---|
| rusqlite 0.43.0 (`bundled`) | SQLite binding for the catalog | 0–1 | MIT [M] | Fine. Chosen over sqlx (ADR-0001). | in use |
| libsqlite3-sys 0.40.0 + bundled SQLite C source | compiles SQLite into the binary | 0–1 | sys crate MIT [C]; SQLite itself public domain [R: sqlite.org/copyright.html] | Fine; no attribution needed for SQLite. | in use |
| image 0.25.10 (`jpeg`, `png` only) | decode/encode JPEG & PNG, resize | 0–1 | MIT OR Apache-2.0 [M] | Fine. HEIC/RAW are *not* part of it. | in use |
| kamadak-exif 0.6.2 | EXIF reading | 0–1 | BSD-2-Clause [M] | Fine; keep the copyright notice in distributions. | in use |
| ignore 0.4.35 | directory walking, `.leohtignore` | 0–1 | Unlicense OR MIT [M] | Fine (choose MIT). | in use |
| notify 9.0.0 | filesystem watching (`watch` feature) | 1 | CC0-1.0 [M] | Fine (public-domain dedication). | in use (optional feature) |
| blake3 1.8.8 | content hashing | 0–1 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception [M] | Fine. | in use |
| camino 1.2.6 | UTF-8 paths | 0 | MIT OR Apache-2.0 [M] | Fine. | in use |
| serde 1.0.229 / serde_json 1.0.154 | (de)serialisation, JSON sidecars | 0 | MIT OR Apache-2.0 [M] | Fine. | in use |
| thiserror 2.0.21 / anyhow 1.0.105 | error types (libs / binaries) | 0 | MIT OR Apache-2.0 [M] | Fine. | in use |
| tracing 0.1.44 / tracing-subscriber 0.3.23 | logging | 0 | MIT [M] | Fine. | in use |
| clap 4.6.10 | CLI parsing (`leoht-cli`, `xtask`) | 0–1 | MIT OR Apache-2.0 [M] | Fine. | in use |
| tempfile 3.27.0 | temp dirs in tests | 0 | MIT OR Apache-2.0 [M] | Dev-only; not shipped. | in use (dev) |
| proptest 1.12.0 | property tests | 0 | MIT OR Apache-2.0 [M] | Dev-only; not shipped. | in use (dev) |
| insta 1.49.0 | snapshot tests | 0 | Apache-2.0 [M] | Dev-only; not shipped. | in use (dev) |
| criterion 0.8.2 | benchmarks | 0 | Apache-2.0 OR MIT [M] | Dev-only; not shipped. | in use (dev) |

The whole resolved tree (178 packages, including transitive ones) passes `cargo deny check licenses` against the allow-list in `deny.toml` (permissive licences only).

### A.2 Planned and candidate libraries named in the plan or its ADRs

| Library (version) | Role | Phase | Licence (source) | Commercial notes | Status |
|---|---|---|---|---|---|
| egui / eframe 0.37.0 | immediate-mode UI (`leoht-ui`) | 2 | MIT OR Apache-2.0 [C][R] | Fine. | planned |
| wgpu 30.0.2 | GPU acceleration | later | MIT OR Apache-2.0 [C] | Fine. | planned |
| tract (`tract-onnx`) 0.23.10 | pure-Rust ONNX inference for faces/objects | 6+ | MIT OR Apache-2.0 [C] | Fine. Model files carry their own licences: review each before shipping. | planned |
| ONNX Runtime (C++ library) | alternative ML runtime | 6+ | MIT [W: PyPI and a cached copy of the repo page] | Fine if confirmed. The official GitHub repo page returned 404 on 2026-11-27 (a news item reports it vanished), so confirm the licence from a release archive before use. Unverified from the official repo today. | candidate |
| ort 2.0.0-rc.15 | Rust bindings to ONNX Runtime | 6+ | MIT OR Apache-2.0 [C] | Pre-release only (no stable 2.x yet); needs the ONNX Runtime binaries. | candidate |
| Tauri 2.12.7 | later UI shell (Rust + web front-end) | 5+ | Apache-2.0 OR MIT [C]; README: "MIT or MIT/Apache 2.0 where applicable" [R] | Fine. The Tauri logo is CC-BY-NC-ND: do not use it in a paid product. | candidate |
| SwiftUI (via FFI to the Rust core) | native Mac UI | 5+ | Apple proprietary framework; SDK licence terms not reviewed | Fine for Mac App Store apps in practice, but we did not review Apple's developer terms. Unverified. | candidate |
| Apple ImageIO / Core Graphics (system frameworks) | HEIC decode on macOS behind a `heic-system` feature | 1 (spike) | Apple proprietary system API; terms not reviewed | ADR-0005's recommended commercial Mac HEIC path. Whether it covers HEVC patents for a paid third-party app is unverified; ask the lawyer. | candidate |
| Apple Vision (system framework) | on-device faces/objects on Mac | 6+ | Apple proprietary system API; terms not reviewed | Mac-only; avoids shipping a model. Unverified terms. | candidate |
| heifer 0.8.0 (0.7.3 when ADR-0004 was written) | pure-Rust HEIC/HEIF decoder + encoder | 1 | MIT OR Apache-2.0 [C]; GitHub detects Apache-2.0 [R] | Licence is fine, but HEVC patents still apply: pure-Rust code grants no patent rights (its README says so). Weeks old. | candidate |
| heic-rs 0.1.5 | pure-Rust HEIC/HEIF decoder (decode only) | 1 | MIT OR Apache-2.0 [C]; GitHub detects Apache-2.0 [R] | As heifer: patents not granted. Weeks old. | candidate |
| heic-decoder 0.1.1 | another pure-Rust HEIC decoder | 1 | MIT OR Apache-2.0 [C] | Very new; not evaluated beyond metadata. | candidate |
| libheif-rs 3.0.0 / libheif-sys 5.3.2+1.23.7 | Rust bindings to libheif (optional fallback) | 1 | both MIT [C] | The bindings are MIT, **but they link libheif, which is LGPL-3.0** (see next row). | candidate (optional, off by default) |
| libheif (C library) | HEIF container + codec plugins | 1 | LGPL-3.0 [R: `COPYING` and README of strukturag/libheif] | LGPL: for a closed paid build, dynamic linking and relinking/source-offer obligations apply; static linking is harder. Sample apps are MIT. Needs legal review. | candidate (fallback only) |
| libde265 (C library) | HEVC decoder used by libheif | 1 | LGPL-3.0 [R: `COPYING` and README of strukturag/libde265] | Same LGPL considerations as libheif, plus HEVC patents. | candidate (via libheif) |
| x265 (C library) | HEVC encoder used by libheif for encoding | – | GPL-2.0 [R: GitHub licence of videolan/x265] | **GPL: incompatible with a closed paid build.** We do not need encoding; never enable it. A separate commercial licence may exist: not verified. | rejected (do not link) |
| heic 0.1.6 (imazen) | pure-Rust HEIC decoder | – | AGPL-3.0-only OR LicenseRef-Imazen-Commercial [C] | AGPL is incompatible with our policy; the commercial licence would have to be bought. | rejected |
| rawloader 0.37.2 | pure-Rust RAW loader (named in the plan) | 1 spike / later | LGPL-2.1 [C] | LGPL, and Rust links statically by default: a licence gate (ADR-0005) before any use. | candidate (blocked by licence gate) |
| rawler 0.8.0 (dnglab) | pure-Rust RAW library | 1 spike / later | LGPL-2.1 [C] | Same as rawloader. | candidate (blocked by licence gate) |
| LibRaw (C library, named in the plan) | RAW decode, metadata, embedded previews | 1 spike / later | LGPL-2.1 **or** CDDL-1.0, at the user's choice [R: `COPYRIGHT` in LibRaw/LibRaw; the website agrees and says licensing may change in future versions] | CDDL is file-level copyleft and may suit a closed binary better than LGPL; both need legal review. Bundles other code (BSD-style, MIT-style) listed in its `COPYRIGHT`. | candidate (blocked by licence gate) |
| libvips (C library; "vips bindings" in the plan) | fast image processing pipeline | 3+ | LGPL-2.1-or-later [R: README of libvips/libvips] | LGPL: usually dynamically linked, needing a shippable shared library and relinking rights: awkward for notarised Mac apps. Prefer pure Rust unless a lawyer approves. | candidate (unlikely) |
| libvips crate 2.3.0 (Rust bindings) | binding to libvips | 3+ | MIT [C] | The binding is MIT but links LGPL libvips. | candidate (unlikely) |
| sqlx 0.9.1 | async SQL toolkit | – | MIT OR Apache-2.0 [C] | Licence fine; rejected for fit: async runtime and tooling not needed for a local embedded DB (ADR-0001). | rejected |

**Not verified in this appendix:** Apple's developer/SDK terms (SwiftUI, ImageIO, Vision); x265's commercial licence; ONNX Runtime from its official repo (404 on the day); licences of any ML model files; versions of native libraries beyond what is stated.

---
**Changelog**
- 2026-11-26: Phase 1 format matrix added under Phase 1 (Keith's request); see ADR-0004. Replaced the earlier condensed working copy with the full plan text; no other wording changed.
- 2026-11-26: Public repository `spacefilm900/leoht` created with Keith's approval while "Leoht" is still a working title; full trademark clearance (USPTO, EUIPO, domains, common-law) is still pending and the name/repo may change.
- 2026-11-27: Appendix A (Dependencies and licences) added, with a verification legend; see ADR-0005 (licensing and commercial readiness). Keith's plan text above is otherwise unchanged.
- 2026-11-27: Core Principle 6 added (Excellence at a focused core first), at Keith's request. Keith's other plan text is unchanged.
