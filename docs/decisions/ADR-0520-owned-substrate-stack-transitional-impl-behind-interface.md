---
id: ADR-0520
title: "Owned substrate stack (fabric->DB->object-store->k8s->OS->kernel): transitional-impl-behind-a-stable-interface, none blocking, infinite-scale locked into W1 interfaces now"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: []
amended_by: [ADR-0537]
depends_on: [ADR-0516]
amends: [ADR-0280, ADR-0482, ADR-0196]
related: [ADR-0280, ADR-0482, ADR-0196, ADR-0510, ADR-0516, ADR-0517, ADR-0521]
related_specs:
  - /specs/masterplan.json
  - /specs/hyperscaler-architecture-invariants.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W1
---

# ADR-0520: Owned substrate stack — transitional-impl-behind-a-stable-interface

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Decomposes ADR-0516 (the fabric's foundation). **Amends ADR-0280, ADR-0482, and ADR-0196.**

## Context

The Agentic Delivery Fabric (ADR-0516) sits on top of an owned substrate stack. The grounding
doctrines exist — the substrate-of-substrate dependency doctrine (ADR-0280), the bespoke-substrate
roadmap (ADR-0482), and the object-storage choice (ADR-0196) — but they predate the fabric and do not
name the fabric/AST layer above them, nor lock infinite-scale into the W1 interfaces so a
non-scalable interface cannot calcify.

## Decision

Ratify the kernel-to-fabric owned-substrate stack as the fabric's foundation:

Agentic Delivery Fabric → bespoke distributed-SQL DB (metadata) + bespoke infinite-scale object-store
(content) → k8s + containerd → Talos-style OS → kuberos-kernel.

EVERY layer is: **owned · cloud-native · infinite-scale · productized · transitional-impl-behind-a-stable-interface**
→ owned-bespoke on its own timeline, with **NONE blocking another**. The infinite-scale constraints
(sharded / no-single-leader / content-addressed) are LOCKED into the interfaces NOW; swaps are
event-gated by cutover triggers, not hard deadlines.

The **W1 interfaces to lock**: `WorkAreaTree`, `scm-facts`, `object-store-kernel`, the DB trait, the
gate contract, and the content-address. The concrete W1 `WorkAreaTree` trait/vocabulary seam is
accounted by `governance/corpus/work-area-tree-kernel/OWNERS`,
`governance/corpus/work-area-tree-kernel/BUCK`,
`governance/corpus/work-area-tree-kernel/Cargo.toml`,
`governance/corpus/work-area-tree-kernel/src/lib.rs`,
`governance/corpus/work-area-tree-kernel/tests/work_area_tree_contract.rs`, and
`registry/catalog/work-area-tree-kernel.yaml`; it remains a seam only and does not implement the
W2 parser or W4 SCM pipeline.

## Drivers

- Hyperscaler convergence on bespoke-everything (ADR-0482).
- Interface-decoupling so no substrate blocks delivery.
- Locking infinite-scale into the interfaces now prevents a non-scalable interface from calcifying.

## Alternatives considered

- **Adopting external substrates as the destination** — rejected (Non-Goal): they are transitional
  bridges only.
- **Big-bang building the substrates** — rejected (Non-Goal): each lands on its own timeline behind a
  stable interface, none blocking.

## Consequences

- **Amends ADR-0280** — hardens its substrate-of-substrate dependency doctrine into the
  "transitional-impl-behind-a-stable-interface, none blocking, infinite-scale-locked-into-interfaces-now"
  sequencing rule and names the W1 interface set above. (ADR-0280's own Proposed→resolved status is
  the founder's separate ratify/drop call per the resolve-every-Proposed rule.)
- **Amends ADR-0482** — inserts the fabric + owned AST substrate as the top layer above its tiered
  bespoke-component roadmap and reaffirms its bridge-discipline (parallel-run, per-feature parity
  gates, quality-gated cutover, no hard-deadline); ADR-0521 sequences it as W0–W6.
- **Amends ADR-0196** — qualifies SeaweedFS/Ceph as explicitly TRANSITIONAL behind the
  `object-store-kernel` interface; the W5 bespoke infinite-scale object-store supersedes it at a
  parity-gated cutover (consistent with ADR-0482 bridge-discipline).

door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: settled-vision spec (PASSED). Decomposes
ADR-0516; amends ADR-0280 / ADR-0482 / ADR-0196.*
