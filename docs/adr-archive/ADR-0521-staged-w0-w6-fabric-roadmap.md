---
id: ADR-0521
title: "Staged W0-W6 fabric roadmap: convergence-first, interface-locking, cutover-gated; W4 bespoke-SCM gated by ADR-0510"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0516, ADR-0517, ADR-0518, ADR-0519, ADR-0520]
amends: []
related: [ADR-0510, ADR-0515, ADR-0516, ADR-0517, ADR-0518, ADR-0519, ADR-0520]
related_specs:
  - /specs/masterplan.json
  - /specs/master-plan-sequencing.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0521: Staged W0–W6 fabric roadmap (roadmap root)

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

The roadmap root of the Agentic Delivery Fabric (ADR-0516). Every fabric Wave references this ADR;
W4 bespoke-SCM is explicitly gated by ADR-0510's numeric cutover triggers.

## Context

The fabric umbrella (ADR-0516) and its decomposed decisions (ADR-0517–0520) need a single staged
sequencing authority so the convergence is convergence-first, interface-locking, and cutover-gated —
not a scatter of parallel campaigns each minting its own order.

## Decision

Ratify the founder-chosen staged roadmap.

- **W0 (DONE):** hermetic buck2 build + de-cargo gates (fresh-checkout verified); firewall LIVE on
  `dev`; the JSON SSOT stores; kernel S4b/S4c/WAVE1 + carriers; transitional substrates (git,
  GitHub Actions, buck2, SeaweedFS/Ceph, cargo).
- **W1 (NEXT):** convergence (ratify the design-corpus + the fabric / AST / owned-stack / safety
  decisions into clean ADRs + the stores; A-delete sprawl; C-enforce); dev-advance (buck2 live on
  `dev`, paired); migration L2→L11; zero-shell refactors; LOCK the interfaces with infinite-scale
  baked in.
- **W2 (high priority, agentic-dev-primary):** owned AST parser behind `WorkAreaTree`; AST
  practice / anti-pattern gates + behavior-preserving auto-fix; AST doc-tracking; work-area
  affected-set; the auto-remediation bot fleet.
- **W3:** de-oyatie config + gate / plugin SDK + marketplace; the `oya new` scaffolder; generated
  forge adapters; buck2 RBE / NativeLink.
- **W4 (cutover-gated):** the bespoke SCM work-area change pipeline + virtual materialization +
  Mononoke-essence Rust server + cloud-backed ChangeUnits + segmented changelog + leases.
- **W5 (far-term, behind interfaces):** bespoke distributed-SQL + bespoke object-store swap-in.
- **W6 (far-term):** bespoke CD + full fabric assembly + kernel-stack convergence.

## Drivers

- Converge-now (the WAVE-1 capture) and pair the dev-advance.
- No tier blocks another (ADR-0482 / ADR-0280 / ADR-0520).
- Cutover-gated swaps (ADR-0510).

## Alternatives considered

- **Build-substrates-first** — rejected: no forcing function, fails the honest-cost test.
- **Skip convergence** — rejected: the sprawl regrows.

## Consequences

W4 is explicitly GATED by ADR-0510's numeric cutover triggers; the W5 object-store swap is gated per
ADR-0520 / ADR-0196 parity. Acceptance criterion: an external project adopts the platform without
forking oyatie; `oya new` births a project already hermetic + gate-conformant + CI/CD-wired +
doc-SSOT'd + work-area-lock-enabled; every finding-code declares a governor tier (ADR-0519); the
agent-fleet edits one codebase with near-zero wasted work. door:one-way.

## Open questions (capstone backlog — not blockers to this sign-off)

These ten capstone open-questions are recorded here for later door adjudication. None blocks
ratification of this roadmap or any fabric ADR; each is carried as a backlog item.

1. **OQ-1 — config format:** `oya-ci.toml` (TOML) vs `.oya-ci/config.json` (JSON) for the
   conformance-floor loader (ADR-0527). One canonical format; schema identical either way.
2. **OQ-2 — gate authoring:** Rust-engine-with-any-language-input vs first-class non-Rust gate
   authoring for the gate/plugin SDK (ADR-0534).
3. **OQ-3 — distribution channel:** confirm OCI + pinned-git as primary with crates.io as an
   optional convenience mirror only (ADR-0535).
4. **OQ-4 — gate-contract crate name:** RESOLVED at this door to `oya-ci-gate-contract` (the
   semver'd gate-trait crate); the third-party SDK is `oya-ci-gate-sdk` (ADR-0528/0534).
5. **OQ-5 — adopter floor lanes:** whether the bespoke `registry-drift` / `cloud-ci-firewall`
   workflow lanes become config-driven for adopters vs stay composite-action boilerplate
   (ADR-0527/0533).
6. **OQ-6 — `oya new` template scope:** which products a freshly-scaffolded project pins by default
   (ADR-0532/0535).
7. **OQ-7 — application portfolio:** whether the `oya/` SaaS verticals are in or out of the
   lifecycle-tooling product line by default (ADR-0532).
8. **OQ-8 — govbot shape:** one `oya-govbot` vs three sub-products (release-train / deps / repo-bots)
   (ADR-0535).
9. **OQ-9 — absent-source disposition:** disable-vs-advisory-empty for absent config sources
   (ADR-0533).
10. **OQ-10 — kernel hermeticity fallback:** whether to accept a minimal Nix flake for QEMU/musl if
    `download_file` pinning proves non-reproducible across host classes, or DEFER ADR-0524 and keep
    the kernel on cargo+bash (ADR-0523/0524 reproducibility-outranks-shell-count tension).

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: settled-vision spec (PASSED). Roadmap
root for ADR-0516; W4 gated by ADR-0510.*
