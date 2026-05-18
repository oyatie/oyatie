---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-005-redis-cache
status: pending
execution_unit: ChangeSet
owner: axis-anonymous
acceptance_lanes: [cargo-check, cargo-test]
---

# IP-005: Redis cache (feed + vote-token dedupe + handle-rotation pool)

## Intent

Author Redis 7.2 LTS adapters for feed-timeline (hot-feed cache), vote-engine (HyperLogLog dedupe + blinded vote-token dedupe), and pseudonymous-identity (per-thread handle-rotation pool). Cluster mode required per PRD §"Horizontal Scalability".

## ChangeSet

- `src/feed-timeline-adapter-redis/*`
- `src/vote-engine-adapter-redis/*`
- `src/pseudonymous-identity-adapter-redis/*`

## Key invariants

- Cache keys are per-affinity-cluster, NOT per-user
- TTL aligned with retention tier; cache expires before retention boundary
- Cluster sharding by `(tenant_id, affinity_cluster_id) mod N`

## Acceptance

- `cargo check` passes
- Cache key inspection: no Redis key contains `user_id` (LEAN lane)
- Tier-aware TTL test passes
