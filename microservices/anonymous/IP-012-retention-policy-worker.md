---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-012-retention-policy-worker
status: pending
execution_unit: ChangeSet
owner: axis-anonymous + ops-data
acceptance_lanes: [cargo-check, cargo-test, oya-governance-retention-default-short]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: Retention-policy worker (30/60/90-day tiers, hard-delete propagation)

## Intent

Author retention-policy BC worker that hard-deletes expired posts. Per ADR-ANON-0004 + I3 invariant. Tier defaults to 30d; tenant-selectable up to 90d (60d for pack-eu, pack-us-healthcare, pack-jp per regulatory bound).

## Acceptance

- Worker hard-deletes expired records within p99 ≤ 5s of expiry
- Tombstone seal recorded for every deletion
- LEAN lane `oya-check-retention-default-short` verifies default is 30d
- Per-pack overlay test: pack-eu max retention 60d (GDPR Art. 5(1)(e))
