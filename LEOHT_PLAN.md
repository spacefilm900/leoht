# Leoht: A Modern, Folder-First Photo Management System

**Project Goal**: Build a production-grade, open-source photo organizer and light editor inspired by the simplicity of Google Picasa (discontinued ~2016). Prioritize natural filesystem folder structures, non-destructive per-picture enhancements, and a clean user experience.

**Platform Strategy**: Mac first (native feel where possible), then Linux and Windows.

**Development Approach**: Agentic software development — iterative, well-documented steps using AI agents. Each phase produces working code, tests, documentation, and a narrative of the process. All work lives in a public GitHub repository with GitHub Actions CI and CodeRabbit for automated, high-quality code review.

**Legal Stance**: Fully clean-room. No reverse engineering of proprietary binaries, no copying of Google code or trademarks. Working project name is "Leoht". Full trademark clearance (USPTO, EUIPO, domains, common-law) recommended before public branding or a permanent GitHub repository name.

## Core Principles
1. Filesystem is the source of truth.
2. Non-destructive edits by default (sidecar JSON or lightweight DB); export or explicit Save applies changes.
3. Simple, fast, local-first.
4. Modernized facilities (RAW, on-device face/object recognition, performance, accessibility, dark mode).
5. High-quality, maintainable code; small focused files; rigorous testing incl. performance and large-volume.

## Stack
- Rust, rustfmt, clippy strict.
- UI: Phase 1-2 egui; later Tauri or native SwiftUI via FFI.
- Image: image crate, vips bindings or pure Rust, rawloader/libraw; wgpu later.
- Index: SQLite (rusqlite or sqlx). Sidecars `.leoht.json` or XMP-compatible.
- Faces/AI: tract/ONNX or Apple Vision, optional.
- Cargo, GitHub, GitHub Actions (lint, test, build Mac/Linux/Windows), CodeRabbit every PR.

## Phases
- Phase 0: repo structure; data model (photos, folders, albums as virtual collections, edit recipes); initial ADRs; CI skeleton (fmt, clippy, unit tests) and testing harness; code quality standards (file size limits, comment conventions, Rust style).
- Phase 1: scanner & index (JPEG, PNG, HEIC, RAW detection; basic EXIF; cached thumbnails; SQLite with fs watch; CLI demo first; unit+integration; volume tests 1k-10k synthetic; basic benchmarks).
  - **Phase 1 format matrix (priority order, added at Keith's request 2026-11-26):**
    1. **JPEG** — full first-class: decode, EXIF, embedded thumbnail, sidecar edits, volume-test baseline.
    2. **HEIC/HEIF** — first-class via a pure-Rust crate (Keith prefers `heifer` or `heic-rs`; `libheif-rs` is an optional fallback behind a cargo feature). Patent note: HEVC is patent-encumbered; pure-Rust decoders still implement HEVC, and libheif links libde265/x265. Recommend legal review before distributing binaries. See ADR-0004.
    3. **PNG** — full support.
    4. **RAW (common formats)** — detection + basic metadata + thumbnail where possible (e.g. embedded preview); deeper decoding later.
    5. **Others (TIFF, WebP, etc.)** — detection only in Phase 1; full support later.
- Phase 2: viewer & grid UI (folder hierarchy, thumbnail grid, lightbox; UI and large-grid perf tests).
- Phase 3: non-destructive edit engine (recipe format: crop, rotate, straighten, exposure, contrast, saturation, red-eye; live preview; sidecar/DB persistence; export bakes edits; golden-image and batch perf tests).
- Phase 4: stars, captions, tags, virtual albums, search, optional duplicates; search perf on large indexes.
- Phase 5: Mac polish & packaging (native windowing, menus, DnD, notarization, prefs, shortcuts, macOS CI packaging).
- Phase 6+: Linux/Windows builds, faces, geotag map, batch ops, 10k-100k+ library support with volume/benchmark suites, optional sharing.

## Agentic techniques
Spec-driven (specs + ADRs before code); isolated agents with shared interface contracts; comprehensive testing harness enforced in CI; code quality standards reviewed by CodeRabbit; per-phase "How we built X" narratives in docs; incremental bootstrapping (CLI scanner + thumbnails first); human-in-the-loop gates via GitHub Issues/PRs.

## Immediate Next Steps
1. Scaffold the Rust project in a new GitHub repo incl. CI and CodeRabbit config.
2. Implement Phase 1 scanner with strong initial test suite incl. volume tests.
3. Capture the process in docs/process/.

(Condensed reference copy saved by Leoht Master from Keith's message of 2026-11-26; Keith's full text is canonical.)
