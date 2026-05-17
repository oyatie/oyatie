---
doc_class: ImplementationPlan
ip_id: IP-008
title: function-engine (OLTP Postgres reads; tier-filter projection)
microservice: ontology
phase: P01-typed-entity-substrate
status: pending
owner_team: axis-ontology
date: 2026-05-17
depends_on: [IP-004]
acceptance_lanes:
  - cargo-check
  - cargo-clippy
  - cargo-nextest
  - oya-foundry-fitness-perf-budget
  - oya-foundry-fitness-ontology-tier-enforcement
related_artifacts:
  - microservices/ontology/src/crates/oya-ontology-function-engine-{kernel,domain,usecase,adapter,worker}/
doc_status: published
---

# IP-008: function-engine (OLTP)

## Intent

Author the Function (read-projection) evaluator over Postgres OLTP. Hits the Function Type schema's EXPLAIN pre-check; tier-filters the result against the caller's `max_tier`; honours per-tenant rate-limit + cache TTL.

## Scope

In-scope:
- `oya-ontology-function-engine-{kernel,domain,usecase,adapter,worker}` crates.
- EXPLAIN pre-check: refuses Functions whose projected memory > `max_memory_projection_mb`.
- Tier-filter projection: result rows hide properties whose `property_tier > caller.max_tier`.
- Valkey hot cache; TTL per Function Type schema.
- Per-tenant rate limit (HTTP 429 with Retry-After).
- Worker: async background pre-warm + cache refresh.

## Implementation

| Step | Action |
|---|---|
| 1 | Scaffold 5 crates |
| 2 | Author EXPLAIN pre-check (rejects unbounded scans + memory projections > limit) |
| 3 | Author tier-filter projection (compile-time + runtime checks) |
| 4 | Wire Valkey cache adapter |
| 5 | Author per-tenant rate limiter (token bucket) |
| 6 | Worker: pre-warm + refresh; HPA on CPU |
| 7 | Tests: 10k QPS p99 ≤ 50 ms (perf bench); tier escape attempt refused |

## Verification

- `cargo bench -p oya-ontology-function-engine-domain -- function_read_p99` — p99 ≤ 50 ms at 10k QPS.
- LEAN lane `oya-foundry-fitness-perf-budget` exit 0.
- LEAN lane `oya-foundry-fitness-ontology-tier-enforcement` exit 0.

## References

- ADR-0006; ADR-0107 (Bominal — Function p99 ≤ 50 ms mandate).
- `microservices/ontology/PRD.md` §"Performance Targets".
