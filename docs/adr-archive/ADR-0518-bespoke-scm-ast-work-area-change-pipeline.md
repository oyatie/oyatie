---
id: ADR-0518
title: "Bespoke SCM = the 10-stage AST work-area change pipeline (native-only, leases-not-locks); defines the deferred ADR-0510 destination"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-0709]
depends_on: [ADR-0516, ADR-0517]
amends: [ADR-0510]
related: [ADR-0510, ADR-0516, ADR-0517, ADR-0521, ADR-0526]
related_specs:
  - /specs/masterplan.json
  - /specs/gitops-vcs-replacement.json
  - /specs/bespoke-scm-declare-observe-contract.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W4
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0518: Bespoke SCM = the 10-stage AST work-area change pipeline

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Decomposes ADR-0516 Component 4. **Amends ADR-0510** by defining its deferred destination concretely;
ADR-0510's numeric cutover-trigger discipline is preserved unchanged — W4 is cutover-gated per
ADR-0510.

## Context

ADR-0510 recorded the SCM destination as a bespoke hyperscaler monorepo-VCS deferred behind numeric
cutover triggers, with GitHub + git as the transitory host — but left the destination's concrete shape
implicit. The Agentic Delivery Fabric (ADR-0516) needs that destination defined so the work-area
content-address (ADR-0517) and the lifecycle hash unify, and so a fleet of agents can edit one codebase
concurrently with near-zero wasted work.

## Decision

The bespoke SCM destination is a **10-stage hyperscaler change pipeline**:

DECLARE → ADMIT → LEASE → ISOLATE(virtual) → AUTHOR → GATE(buck2 + AST gates + auto-remediate) →
ATTEST → INTEGRATE → PROPAGATE(CD) → OBSERVE.

It is Sapling / Mononoke / EdenFS / CommitCloud-inspired, owned in Rust, and **native-only**. The
grit-essence claim/work/done model is re-framed as these native pipeline stages (no git-overlay).
Concurrency is **leases-not-locks**, sharded, with no single leader. Work-area identity is the
content-addressed AST hash (ADR-0517).

The current W4 metadata-only contract projection for these stages is
`specs/bespoke-scm-declare-observe-contract.json`.

## Drivers

- A fleet of agents editing one codebase concurrently with near-zero wasted work demands native
  leases and sharded, no-single-leader concurrency.
- Hyperscaler-scale fan-out beyond single-node forge limits — the load-bearing destination capability
  ADR-0510 named as the most-plausible cutover-forcing function.

## Alternatives considered

- **A standalone grit-style git-overlay tool** — rejected (Non-Goal): locking is native to the
  pipeline, not a bolt-on overlay.
- **A GritQL codemod product** — rejected (Non-Goal): codemods exist only as the owned auto-fix tier
  (ADR-0530/0531).
- **Building it now** — rejected: deferred to W4, cutover-gated per ADR-0510's numeric triggers.

## Consequences

**Amends ADR-0510** — that ADR recorded the SCM destination as a bespoke hyperscaler monorepo-VCS
deferred behind numeric cutover triggers, with GitHub + git as the transitory host; this ADR DEFINES
that destination concretely as the AST work-area change pipeline and re-frames the claim/work/done
locking model as native pipeline stages. ADR-0510's numeric cutover-trigger discipline and its
"GitHub transitory until trigger fires" stance are PRESERVED unchanged — W4 is cutover-gated per
ADR-0510, and the scm-facts seam (ADR-0526) de-risks the cutover to a single adapter impl-swap.
door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: settled-vision spec (PASSED). Decomposes
ADR-0516; amends ADR-0510 (defines the deferred destination; cutover discipline preserved).*
