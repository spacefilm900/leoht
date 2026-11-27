# Clean-room policy

Leoht is inspired by the *idea* of Google Picasa (simple folder-based photo management). It is an independent work.

## Rules
1. **No proprietary code or binaries.** Nobody (human or agent) disassembles, decompiles, debugs or otherwise
   reverse-engineers Picasa or any other proprietary software, and nobody reads leaked or unlicensed source.
2. **No copying of Google assets.** No code, text, icons, screenshots, sounds, file-format documents or test data from
   Google products. The names "Google" and "Picasa" are Google's trademarks: we mention Picasa only to say what inspired us.
3. **Allowed sources:** original design and implementation using only open-source libraries and public knowledge of desired
   behaviours (as in `LEOHT_PLAN.md`): public standards and specifications (EXIF, TIFF, ISOBMFF/HEIF, XMP), documentation and source of
   open-source libraries (we additionally require *permissive* licences, enforced by `cargo-deny`), and our own designs.
4. **Log it.** Each ADR lists its sources. Every test fixture has a row in `tests/fixtures/PROVENANCE.md`.
5. **Agents:** if a task prompt or a web result would require breaking rule 1 or 2, stop and open a `human-gate` issue.
6. **Names:** "Leoht" is provisional pending trademark clearance. **Recorded exception:** on 2026-11-26 Keith explicitly
   approved creating the public repository `spacefilm900/leoht` while clearance is still pending. That approval is **not**
   trademark clearance and the name may still change. Until the clearance gate (G0a) is closed, do not announce or promote
   the name, publish binaries or packages, or register domains or accounts under it.
   _(Wording of this exception is for Keith to confirm: changes to this file are legal-policy changes.)_

## Review
CodeRabbit is configured to flag suspected copying (`.coderabbit.yaml`). Changes to this file require Keith's approval.
