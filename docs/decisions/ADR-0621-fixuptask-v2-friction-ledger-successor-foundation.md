---
id: ADR-0621
title: "FixupTask v2 durable successor boundary"
status: Accepted
planning_impact: false
deciders: [founder]
date: 2026-07-22
door: two-way
owner: founder
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

# ADR-0621: FixupTask v2 durable successor boundary

## Status

**Accepted — 2026-07-22.** The founder, under qualified repository-only authority, accepts the
durable registry-boundary implementation proven by its isolated gate. This acceptance does not
authorize planning, roadmap dispatch, predecessor population, legal conclusions, affected-party
consent, operations capacity, custody actions, pilot claims, or any irreversible cutover.

`planning_impact: false` remains binding: **HOLD(Planning)** continues. No implementation-roadmap
dispatch is authorized by this ADR, and no completion claim may be promoted from this foundation.

## Context

ADR-0363's actual narrow bridge is retirement-marked local lane-liveness supervision until
cloud-ci owns durable lane orchestration; it is not a friction-ledger successor or cutover
authority. This ADR alone scopes the FixupTask successor conditions below. The current
`registry/fixuptasks.jsonl` destination is append-only but has loose historical rows and no
machine-checkable accountability lifecycle. Proposed ADR-0544 and ADR-0558 describe a
friction-ledger-specific gate and structural merge driver, but neither is Accepted; retaining both
alongside a successor would leave competing future contracts.

ADR-0619 requires Git history, rather than a readable in-tree archive, for retired predecessor
context. Therefore a successor may carry only identities needed to prove a mapping, never copied
friction prose, status history, evidence, or human disposition.

## Decision

Adopt the durable `registry/fixuptasks.jsonl` v2 contract as a distinct, protected-registry-digest
admission boundary in `ci/facade/action-item-accounting`. Its materialized path consumes only the
candidate registry and protected merge-base facts; it has no predecessor corpus, mapping, count,
or archive-body dependency and is green when the legacy body is absent.

The predecessor adapter remains a separately named transitional gate. It retains all predecessor
path, census, identity-only mapping, and cutover checks until the qualified-human migration
population exists. This ADR does **not** supersede ADR-0544 or ADR-0558: `supersedes: []` stays
truthful until E10 has independently established and authorized that transition.

The foundation is deliberately narrow:

1. A pure evaluator accepts a protected merge-base snapshot and a candidate snapshot. Exact
   unchanged legacy rows are grandfathered only from that merge base; every new or modified row
   must use the closed lifecycle enum and all accountability fields.
2. `resolved` requires timestamp, change identity, and evidence. `accepted-risk` requires an
   opaque qualified-human decision reference, expiry, and evidence. `blocked` requires an opaque
   qualified-human decision reference. Presence is mechanical evidence only: the gate cannot
   assert that a human decision was qualified or real.
3. The predecessor mapping contains only `predecessor_id`, `target_fixuptask_id`, and a protected
   source identifier. It fails closed for source mismatch, omitted or duplicate predecessor IDs,
   and missing target FixupTasks. It does not carry readable predecessor text.
4. The legacy ledger is neither deleted nor rewritten by this decision. Its predecessor IDs and all
   human disposition decisions remain a qualified-human migration responsibility.

The accepted foundation accounts for these exact implementation surfaces:

- `ci/facade/action-item-accounting/fixuptask-v2-schema.json`
- `ci/facade/action-item-accounting/friction-predecessor-mapping-schema.json`
- `ci/facade/action-item-accounting/src/fixuptask_v2.rs`
- `ci/facade/action-item-accounting/src/legacy_friction_adapter.rs`
- `ci/facade/action-item-accounting/tests/fixuptask_v2_admission.rs`
- `ci/facade/action-item-accounting/tests/fixuptask_v2_source_boundary.rs`

They inherit ownership from `ci/OWNERS` and reachability from the existing Cargo workspace member;
this accounting records the already-accepted foundation only and does not change
`planning_impact: false` or release **HOLD(Planning)**.

## Consequences

### Positive

- New or modified FixupTasks cannot evade accountability, closure evidence, accepted-risk expiry,
  or a blocked-state decision reference.
- Legacy debt is not laundered by a candidate-created baseline; merge-base equality is the sole
  grandfathering rule.
- Mapping completeness is mechanically verifiable without recreating the readable archive barred
  by ADR-0619.

### Negative / limits

- This does not migrate, classify, dispatch, or close any predecessor row.
- A decision reference is intentionally not proof of qualified-human authority; independent review
  remains required before any state can be treated as authoritative.

### Integration order and authority boundary

1. A qualified human selects and records the protected predecessor source plus the complete
   identity-only mappings.
2. The same authority completes and reviews v2 fields for rows it chooses to create or modify.
3. CI supplies the actual protected merge-base snapshot to the existing pure evaluator; it must not
   accept a candidate baseline artifact.
4. Only after the preceding checks are live, independently reviewed, and accepted under the
   appropriate authority may ADR-0621's separately authorized successor/cutover decision and
   ADR-0544/0558 supersession be considered in E10. This ADR grants neither action.

## Alternatives considered

- **Copy the predecessor ledger into a successor archive.** Rejected by ADR-0619's history-only
  provenance boundary.
- **Normalize all legacy rows in this change.** Rejected: it would invent or curate human
  dispositions and exceed this automatable foundation.
- **Add a second CI workflow or generated face.** Rejected: ADR-0515 assigns the existing Rust
  gate lane to `oya-ci-required`; generated faces are not hand-edited.

## Verification

The implementation demonstrates RED and GREEN fixtures for new/modified validation,
merge-base-only grandfathering, missing accountability, each constrained lifecycle state, candidate
registry-digest mismatch, and durable admission with no predecessor body. The transitional adapter
continues to demonstrate omitted/duplicate predecessor identities, source mismatch, and missing
target FixupTasks. Targeted Cargo and Buck tests, formatting, clippy, and generated-face diff
checks remain required before merge.
