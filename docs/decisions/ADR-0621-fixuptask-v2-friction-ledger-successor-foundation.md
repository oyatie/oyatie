---
id: ADR-0621
title: "FixupTask v2 as the proposed durable successor foundation for friction-ledger accounting"
status: Proposed
planning_impact: false
deciders: []
date: 2026-07-21
door: two-way
owner: council-architecture
supersedes: [ADR-0544, ADR-0558]
superseded_by: []
amends: [ADR-0363, ADR-0515]
amended_by: []
depends_on: [ADR-0363, ADR-0515, ADR-0619]
related: [ADR-0544, ADR-0558]
related_specs:
  - /registry/fixuptasks.jsonl
milestone: W0
---

# ADR-0621: FixupTask v2 friction-ledger successor foundation

## Status

**Proposed — 2026-07-21.** This is an implementable proposal only. It neither records
qualified-human approval nor changes `HOLD(Planning)`: it must not dispatch roadmap work, curate
predecessor dispositions, or promote any completion claim.

## Context

ADR-0363 retains the friction-ledger bridge until a durable successor exists. The current
`registry/fixuptasks.jsonl` destination is append-only but has loose historical rows and no
machine-checkable accountability lifecycle. Proposed ADR-0544 and ADR-0558 describe a
friction-ledger-specific gate and structural merge driver, but neither is Accepted; retaining both
alongside a successor would leave competing future contracts.

ADR-0619 requires Git history, rather than a readable in-tree archive, for retired predecessor
context. Therefore a successor may carry only identities needed to prove a mapping, never copied
friction prose, status history, evidence, or human disposition.

## Proposed decision

If accepted, this ADR would amend ADR-0363 and ADR-0515 to replace the friction-ledger bridge with
the `registry/fixuptasks.jsonl` v2 contract enforced through the existing
`ci/facade/action-item-accounting` Rust gate and its existing `oya-ci-required` workflow lane. It
would supersede Proposed ADR-0544 and ADR-0558 rather than accepting either older proposal.

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
4. The legacy ledger is neither deleted nor rewritten by this proposal. The 189 predecessor IDs
   and all human disposition decisions remain a qualified-human migration responsibility.

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

### Integration order

1. A qualified human selects and records the protected predecessor source plus the 189
   identity-only mappings.
2. The same authority completes and reviews v2 fields for rows it chooses to create or modify.
3. CI supplies the actual protected merge-base snapshot to the existing pure evaluator; it must not
   accept a candidate baseline artifact.
4. Only after the preceding checks are live may ADR-0363's bridge retirement and the Proposed
   ADR-0544/0558 supersession be considered for acceptance.

## Alternatives considered

- **Copy the predecessor ledger into a successor archive.** Rejected by ADR-0619's history-only
  provenance boundary.
- **Normalize all legacy rows in this change.** Rejected: it would invent or curate human
  dispositions and exceed this automatable foundation.
- **Add a second CI workflow or generated face.** Rejected: ADR-0515 assigns the existing Rust
  gate lane to `oya-ci-required`; generated faces are not hand-edited.

## Verification

The implementation must demonstrate RED and GREEN fixtures for new/modified validation,
merge-base-only grandfathering, missing accountability, each constrained lifecycle state, omitted
and duplicate predecessor IDs, source mismatch, and missing target FixupTasks. Targeted Cargo and
Buck tests, formatting, clippy, and generated-face diff checks remain required before merge.
