# Cohesive Owned-Substrate Doctrine (idea-refine one-pager)

_Founder directive 2026-06-13. Refined via /agent-skills:idea-refine. Leader-state record; to be formalized as an ADR (returns for founder sign-off)._

## Problem Statement
How might we design oyatie's owned substrates (crypto, secure transport, HTTP, persistence) so the bespoke destination reads as if built **ground-up in-house** — cohesion where concepts belong, separation-of-concerns serving *maintainability* — rather than a pile of like-for-like swaps of transient crates (reqwest→hyper, ring→aws-lc, quinn→our-quinn)?

## Recommended Direction
**The unit of replacement is the SUBSTRATE (a cohesive whole), never the crate.** Transient OSS crates are *implementation details inside* a cohesively-designed owned substrate; they are purged at a whole-substrate cutover, never crate-by-crate.

Carve the owned stack into a **few coarse domain substrates** derived from OUR domain, not upstream crate boundaries:
1. **oya-crypto** — primitives, KMS-rooted signing, FIPS path (Tier-4 bespoke per ADR-0506/0482).
2. **secure-channel** — TLS + QUIC + mTLS + SVID/cert handshake as ONE cohesive plane (today smeared across rustls + rustls-webpki + ring/aws-lc-rs + x509-parser + rcgen).
3. **http-surface** — client + server + routing + middleware on the channel (today smeared across hyper + reqwest + tower + axum; partially re-cohered already in `libs/oya-http-*-kernel`).
4. **oya-data** — persistence (already substantially designed in `libs/oya-data-*`; gap is adoption).

Each is a cohesive whole with a clean external seam; transient crates absorbed *inside* each. **Phasing:** design the cohesive architecture + ADRs NOW; keep transient OSS absorbed behind the cohesive facades (Phase-1 OSS bridge, bespoke-over-oss-doctrine); do the bespoke in-house implementation at the gated **W5 cutover** (kubers Phase-B per ADR-0506). Keep the incremental hygiene (zero-ring activation gate, etc.) running alongside — it guards the facade's purity.

**Maintainability test:** a new engineer should understand each owned substrate *without knowing reqwest/ring/quinn/hyper ever existed*. If a seam only makes sense by reference to an upstream crate, the seam is wrong.

## Key Assumptions to Validate
- [ ] The 4-substrate carve is the right granularity (not 1 mega-plane, not N fine adapters) — validate by drafting each substrate's domain boundary and checking no concept is split across two substrates.
- [ ] The existing `libs/oya-http-*-kernel` + `oya-shared-*-transport` family can be RE-COHERED into the `http-surface`/`secure-channel` substrates without a rewrite — validate via an inventory of which existing kernels map where.
- [ ] A cohesion-enforcement gate can mechanically catch (a) transient types leaking across a substrate's public boundary and (b) a domain concept fragmented across upstream-mirroring adapter crates — validate against the existing `oya-check-cohesion` / `oya-check-client-stack-discipline` gates.

## MVP Scope
Author the governing **ADR(s)**: the owned-substrate MAP (the 4 coarse substrates, their domain boundaries, internal modules, external seams, transient-absorption facades, W5 bespoke destinations), extending bespoke-over-oss-doctrine + ADR-0482 + ADR-0506 + ADR-0090 + ADR-0510. Pick the **secure-channel** substrate as the first fully-designed cohesive substrate (most fragmented today + most active via G002 mTLS/SVID). Define the cohesion-enforcement gate. NO bespoke implementation yet (that's W5).

## Not Doing (and why)
- **Crate-by-crate swaps** ("remove reqwest, add hyper-rustls") — the founder's explicit rejection; produces lego-of-different-flavors, not an in-house design.
- **Building the bespoke impl now** — W5/kubers-Phase-B gated (ADR-0506); premature.
- **One mega comms+crypto plane** — rejected in favor of a few coarse substrates with clean seams (maintainability).
- **Dropping the incremental hygiene** — founder said keep both; the activation gate guards the facade while the cohesive design matures.

## Open Questions
- Exact home/naming of each substrate under ADR-0131/0512 (`libs/` for shared ports vs `cloud/`/`oya/` for service homes).
- How the secure-channel substrate composes oya-crypto (signing root) without a circular seam.
- W5 numeric cutover triggers (ties to ADR-0536 OQ-4 for data; analogous OQ for crypto/channel).
