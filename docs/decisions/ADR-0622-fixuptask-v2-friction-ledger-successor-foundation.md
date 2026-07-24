---
id: ADR-0622
title: "FixupTask v2 durable successor foundation"
status: Proposed
planning_impact: false
deciders: []
date: 2026-07-24
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: []
amended_by: []
depends_on: [ADR-0363, ADR-0515, ADR-0619]
related: [ADR-0544, ADR-0558]
related_specs:
  - /registry/fixuptasks.jsonl
milestone: W0
---

# ADR-0622: FixupTask v2 durable successor foundation

## Status

**Proposed — 2026-07-24.** This is an automatable foundation proposal, not a
qualified-human decision. `planning_impact: false` is binding: **HOLD(Planning)**
continues, no implementation-roadmap dispatch is authorized, and no completion
claim may be promoted from this work.

## Context

The append-only `registry/fixuptasks.jsonl` has historical rows but lacks a
machine-checkable lifecycle for new or modified work. ADR-0619 requires retired
predecessor context to remain in Git history rather than a readable in-tree
archive. A successor can therefore carry only identity-level mapping facts; it
must not copy predecessor prose, evidence, status history, or human disposition.

## Proposed decision

If separately accepted under qualified authority, the existing cloud-CI Rust lane
could enforce a durable FixupTask v2 contract. This proposal does not amend or
supersede ADR-0363, ADR-0515, ADR-0544, or ADR-0558 and does not create a binding
lifecycle edge.

The bounded design is:

1. A pure evaluator compares a protected merge-base snapshot with the candidate;
   only byte-identical legacy rows are grandfathered.
2. New or modified rows require the closed lifecycle enum and accountability
   fields. `resolved`, `accepted-risk`, and `blocked` require their mechanical
   evidence fields, but a decision reference never proves qualified authority.
3. A separately named legacy adapter owns any predecessor source, identity-only
   mapping, or qualified-human population work. The durable target has none of
   those source dependencies.
4. The existing required workflow dispatches independently named unit, materialized,
   source-boundary, and legacy-adapter targets. No new workflow and no hand-edited
   generated face are introduced.

## Limits and next authority boundary

This proposal neither migrates nor classifies predecessor rows; it neither creates
human decisions nor judges their qualification. The first non-automatable join is
the qualified-human selection and review of any protected predecessor source and
identity-only mapping. Until that evidence exists, the truthful terminal state is
`BLOCKED_QUALIFIED_HUMAN_INPUT`, not a synthetic approval.

## Verification

The implementation must preserve RED and GREEN coverage for protected-input
absence/staleness, new and modified lifecycle validation, merge-base-only
grandfathering, and the explicit Buck source boundary. ADR projections are emitted
only through the sanctioned Buck2 producer. Targeted Buck2 tests, formatting,
clippy, generated-face policy checks, and protected admission remain required
before any future merge.
