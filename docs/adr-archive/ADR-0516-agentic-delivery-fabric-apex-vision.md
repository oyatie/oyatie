---
id: ADR-0516
title: "Agentic Delivery Fabric — the owned, cloud-native, productized unified delivery platform (apex vision + 5-component topology)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-705]
amended_by: [ADR-0617]
depends_on: [ADR-0515, ADR-0482]
amends: [ADR-0515]
related: [ADR-0482, ADR-0512, ADR-0510, ADR-0364, ADR-0517, ADR-0518, ADR-0519, ADR-0520, ADR-0521, ADR-0617]
related_specs:
  - /specs/masterplan.json
  - /specs/bespoke-cloud-toolchain-services.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0516: Agentic Delivery Fabric — the apex product north-star (5-component topology)

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

This is the umbrella keystone of the WAVE-1 convergence. It ratifies the founder-settled vision
(`.omc/specs/deep-interview-agentic-delivery-fabric.md`, "Status: PASSED — vision settled, roadmap
ratified by founder") as canon. ADR-0517 through ADR-0521 decompose this umbrella into the AST
substrate, bespoke-SCM destination, safety governor, owned-substrate stack, and the W0–W6 roadmap.
The five sub-architecture clusters ADR-0522..0535 carry `related: [ADR-0516]` and topo-sort under it.

## Context

oyatie's destination is a single owned platform — not an assembly of third-party SCM + CI + CD +
dev-env + doc tooling, but one productized system the founder named the **Agentic Delivery Fabric**.
AI agents are the primary producers; quality is built in from project genesis (poka-yoke) and
continuously auto-remediated. The grounding doctrines are already canon: the bespoke-everything
hyperscaler-convergence roadmap (ADR-0482), the maintainable-by-enforcement firewall (ADR-0515), the
canonical-monorepo pattern (ADR-0512), and the founder hermetic-just-works apex doctrine. What was
missing was the single ratified umbrella that names the system, its invariants, and its component
topology so every sub-architecture is reachable from one keystone instead of emerging as a scatter of
CI facts.

## Decision

Adopt the **Agentic Delivery Fabric** as the apex product north-star: an owned, cloud-native,
infinite-scale, productized platform that lets anyone automatically create and maintain
hyperscaler-grade, well-architected, well-documented, well-maintained projects, with AI agents as the
primary producers and quality built-in from project genesis and continuously auto-remediated.

The fabric unifies SCM + CI + CD + dev-env + doc-SSOT as ONE system over FIVE confirmed-active
components:

1. **Hermetic lifecycle** — build · CI · CD · dev-env; zero-shell; one buck2 graph, many runners
   (build + CI half LIVE, fresh-checkout verified). Detailed by ADR-0522/0523/0524.
2. **Automated quality enforcement + auto-remediation** — firewall gates over the AST tree + the
   AUTO/ADVISE/GATE governor. Detailed by ADR-0528/0529/0530/0531. ADR-0515's firewall +
   one-canonical-CI is the **W0 floor** of this component (not the whole of it).
3. **Platform productization + project genesis** — canonical, not-repo-specific, third-party-adoptable;
   the `oya new` scaffolder. Detailed by ADR-0532/0533/0534/0535.
4. **Bespoke SCM** — the AST work-area change pipeline (ADR-0518), cutover-gated by ADR-0510.
5. **Doc / knowledge SSOT** — Diátaxis; the JSON stores; ADRs-are-SSOT, masterplan-is-generated.

Every fabric capability MUST satisfy the universal invariants: **hermetic · canonical · productized ·
automated-foremost · poka-yoke**.

## Drivers

- Agentic-dev-primary economics: a fleet of AI agents is the primary producer, so machine-legible
  structural truth and built-in quality are first-order requirements.
- Hyperscaler convergence on bespoke-everything (ADR-0482): every FAANG-scale operator owns its full
  stack; the fabric is the top of that owned ladder.
- The maintainable-by-enforcement doctrine and the founder hermetic-just-works apex doctrine: a clean
  checkout must build and run reproducibly, with no external blobs and no manual steps.

## Alternatives considered

- **Assemble third-party SCM + CI + CD as the destination** (git / GitHub Actions / SeaweedFS / Ceph
  as permanent) — rejected: these are transitional bridges only (Non-Goal); the destination is owned.
- **A standalone grit-style git-overlay tool** — rejected (Non-Goal): locking is native to the
  bespoke SCM (ADR-0518), not a bolt-on overlay.
- **Big-bang substrate build** — rejected (Non-Goal): the substrate stack is interface-decoupled and
  cutover-gated (ADR-0520/0521), so no layer blocks delivery.

## Consequences

This ADR is the umbrella that ADR-0517–0521 decompose and that the sub-architecture clusters
ADR-0522–0535 hang under. It **amends ADR-0515** by naming ADR-0515's firewall + one-canonical-CI
(`oya-ci-required`) as the W0 floor of fabric Component 2 — ADR-0515 remains the governing floor, in
force, not superseded. Reachability is by construction: ADR-0516 is the masterplan keystone node, and
because every fabric Wave references it, the masterplan-reachability-wiring gate (ADR-0515) goes RED
if any Accepted fabric ADR fails to propagate. door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: the founder-settled vision spec
`.omc/specs/deep-interview-agentic-delivery-fabric.md` (PASSED). Amends ADR-0515 (W0 floor of
Component 2). Umbrella for ADR-0517..0535.*
