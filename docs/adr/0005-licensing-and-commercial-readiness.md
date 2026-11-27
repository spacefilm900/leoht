# ADR-0005: Licensing and commercial readiness

* Status: Proposed
* Date: 2026-11-27
* Deciders: Keith Haviland (owner), Leoht Master (coordinator)

> **Not legal advice.** This ADR records engineering decisions and the open legal questions behind them. It was
> written by an AI coordinator, not a lawyer. **Consult an IP lawyer before any commercial release** (paid build,
> App Store listing, or distribution of a binary that decodes HEIC or RAW).

## Context and problem statement
Leoht is public under the MIT licence, copyright holder Keith Haviland. Keith wants to keep a **paid edition possible**
(for example a signed, notarised Mac build) without closing the door on open-source collaboration. Four things decide
whether that stays possible: what MIT allows others to do, who owns (or may relicense) the code, which dependencies we
pull in, and which patent and trademark constraints apply to a *commercial* binary. Related: ADR-0004 (formats),
`CLEAN_ROOM.md`, `deny.toml`, and Appendix A of `LEOHT_PLAN.md` (dependency licence register).

## Decision drivers
* Keep every commercial option open: do not paint ourselves into a corner by accepting code or dependencies we cannot ship in a paid product.
* Stay a genuine open-source project that people want to contribute to.
* Keep the default build free of copyleft and, where possible, of patent-royalty exposure.
* Make the rules mechanical (CI, templates) so agents and contributors cannot break them by accident.

## Considered options
1. MIT only, no further measures.
2. MIT + owned code (CLA) + permissive-only dependencies + a patent-aware HEIC path. **(chosen as the proposal)**
3. Switch to a copyleft or source-available licence now (rejected for now: contradicts the plan's open-source goal and is hard to reverse).

## Decision

### Key decisions
* **The project licence stays MIT (© Keith Haviland).** Implication, stated plainly: *MIT lets anyone, including third parties,
  copy, modify, redistribute and **sell** Leoht, and Keith cannot stop them.* Past releases can never be withdrawn from the MIT terms.
  What remains ours to control is the **trademark/name** (once cleared), the official signed and notarised builds, the App Store
  listing, support, and future non-MIT additions we own outright. A paid edition therefore competes on trust, convenience and extras, not on exclusive access to the source.
* **Paid-edition options kept possible by owning all code** (decide later, with a lawyer): (a) *open core* — MIT core in this repo, proprietary add-ons in a
  separate private repo that depends on it; (b) *paid, signed and notarised Mac build* (and/or Mac App Store listing) of the same open code, sold for convenience, updates and support;
  (c) *relicensing* future versions under different terms. (a) and (b) need only that we may ship the code commercially, which MIT already permits; (c) needs that we hold the rights to do so
  for *all* contributed code — hence the next point.
* **Contributor agreement: accept no outside code until a simple CLA is in place; DCO alone is not enough.** A DCO sign-off only certifies the contributor's right to submit under the project's
  *existing* licence. Under MIT, a contribution can be included in a proprietary derivative, but only while preserving that contributor's MIT notice, with no express patent grant and no right to
  relicense on different terms. A short **CLA** (a licence grant, not a copyright assignment — e.g. via a CLA bot such as CLA Assistant) gives Keith a broad, sublicensable licence plus a patent
  grant and the contributor's confirmation that they own or may license the work (including employer rights). Until it is set up, outside pull requests are not merged (issues and discussion stay open). The CLA wording needs lawyer review.
* **Permissive-only dependency policy, enforced mechanically.** `deny.toml` allows only permissive licences (MIT, Apache-2.0, BSD, ISC, Zlib, CC0, Unicode, BSL-1.0 and similar) and CI runs `cargo-deny`.
  A GPL, LGPL, AGPL or MPL dependency needs an ADR first and a deliberate, commented exception. Native libraries linked at build time (libheif, LibRaw, libvips, x265, …) count too even though `cargo-deny` cannot see them: they are tracked by hand in Appendix A.
* **HEVC/HEIC patent risk and the commercial Mac path.** HEVC is covered by patent pools. As of 2026-11-27, per Access Advance's own publications: Access Advance (HEVC Advance) licenses HEVC *software* decoders and encoders, with a
  per-copy royalty for software sold separately from a device (July 2026 schedule: $0.555 per copy in Region 1 for in-compliance licensees, higher standard rates otherwise); in December 2025 Access Advance acquired the administrator of
  Via LA's HEVC/VVC programme, which continues as a separate pool ("VCL Advance"), and the pools are still not merged. Pure-Rust or C decoders grant no patent rights. **Recommended commercial Mac path:** decode HEIC through Apple's
  system frameworks (ImageIO / Core Graphics) behind a cargo feature (e.g. `heic-system`), so the OS's own codec is used; use a **pure-Rust decoder (`heifer` or `heic-rs`, ADR-0004) for Linux, Windows and tests**; keep `libheif-rs` optional
  and off by default. *Unverified, for the lawyer:* whether calling the system decoder from a paid third-party app is covered by Apple's licences, and whether free/open-source distribution of a pure-Rust decoder is exempt in any jurisdiction (the pool documents we read do not say so).
* **RAW licence gate.** `rawloader` and `rawler` are **LGPL-2.1** (crates.io metadata). **LibRaw** is dual-licensed **LGPL-2.1 *or* CDDL-1.0, at the user's choice** (verified from LibRaw's official repository `COPYRIGHT` file; its website adds that
  licensing may change in future versions). None passes the permissive-only policy as is. Adopting any of them requires a new ADR covering linking mode (Rust links statically by default), relinking/source-offer obligations for a closed paid build,
  and the CDDL's file-level terms, plus legal sign-off. Phase 1 therefore ships RAW detection, basic metadata and embedded-preview thumbnails only (ADR-0004).
* **Trademark clearance before commercial release.** "Leoht" is a working title; full clearance (USPTO, EUIPO, domains, common-law) is still pending. No paid product, App Store listing, or paid marketing under the name until it is cleared
  or the project is renamed. Because the trademark is our main lever against look-alike paid copies, clearing it and then registering it is part of commercial readiness.

## Consequences
* Good: every commercial route stays open; the default build is permissively licensed and mostly royalty-free except for the HEVC question, which has an explicit mitigation path.
* Good: contributors get clear rules; CI blocks accidental copyleft dependencies.
* Bad: a CLA adds friction and may deter some contributors; we accept that and delay outside PRs until it exists.
* Bad: the system-framework HEIC path is Mac-only, so Linux and Windows builds carry the pure-Rust decoder's patent exposure if distributed commercially.
* Bad: third parties may sell MIT-licensed copies; our protections are the trademark, signed builds and service, not the code.
* Follow-ups (owners in `STATUS.md`): choose and review the CLA; set up the CLA bot; lawyer review of HEVC, RAW, trademark and the paid-edition structure; add `heic-system` to the Phase 1 spike plan; keep Appendix A current on every dependency change.
