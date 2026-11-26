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

---
**Changelog**
- 2026-11-26: Phase 1 format matrix added under Phase 1 (Keith's request); see ADR-0004. Replaced the earlier condensed working copy with the full plan text; no other wording changed.
- 2026-11-26: Public repository `spacefilm900/leoht` created with Keith's approval while "Leoht" is still a working title; full trademark clearance (USPTO, EUIPO, domains, common-law) is still pending and the name/repo may change.
