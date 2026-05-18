---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-008-post-store-bc
status: pending
execution_unit: ChangeSet
owner: axis-anonymous
acceptance_lanes: [cargo-check, cargo-test, oya-governance-blinding-column-isolation]
---

# IP-008: Post-store BC end-to-end

## Intent

Author the full post-thread BC vertical: kernel + domain + usecase + adapter-postgres + rest + worker + sdk. Implements FR-02 (publish post) + FR-03 (reply) + FR-09 (hard-delete).

## Critical invariants

- Posts table contains `blinded_commitment`, NOT `user_id` (I1)
- Hard-delete propagates within p99 ≤ 5s (I3)
- Audit-chain seal on every create/delete

## Acceptance

- Post-create roundtrip < 100ms p95
- Hard-delete propagation < 5s p99
- Blinding-column lint passes
- Audit-chain seal verified on every event
