---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-009-vote-engine-bc
status: pending
execution_unit: ChangeSet
owner: axis-anonymous
acceptance_lanes: [cargo-check, cargo-test, oya-governance-vote-idempotency-property]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: Vote-engine BC (blinded vote tokens, Wilson ranking)

## Intent

Implement FR-04 (cast vote) with blinded vote tokens + Wilson lower-bound ranking per ADR-COMM-0002 inherited. Per PRD vote p99 ≤ 50ms.

## ChangeSet

- vote-engine kernel + domain + usecase + adapter-redis + adapter-postgres + rest + sdk
- Blinded vote token issuance and verification
- Wilson-bound + time-decay ranking computation

## Acceptance

- Idempotent vote-cast property test (same blinded token → same outcome)
- p99 ≤ 50ms under load
- Wilson ranking convergence test (low-vote regime falls back to Hacker News per ADR-COMM-0002)
