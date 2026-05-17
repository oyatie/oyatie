---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-runtime
deciders: ops-sre-reliability, axis-foundry-runtime, council-architecture
related_adrs: [ADR-0025, ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/foundry-runtime/cost-budget.md
  - microservices/foundry-runtime/multi-region.md
  - microservices/foundry-runtime/policy/runtime-isolation.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (foundry-runtime µservice)

## Purpose

Sizing formulas + reference-architecture baseline numbers for every layer-A component (Redis 7.4 LTS cluster + Postgres 16 LTS) and layer-B component (`oya-foundry-runtime-*`). Drives `cost-budget.md` and `multi-region.md`. Numbers cite Redis Enterprise + Postgres reference architectures; verify-against-current-docs marker where upstream may have moved on.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | tenancy µservice |
| Avg concurrent invocations per tenant | `C_concurrent_per_tenant` | per `tenant_scope` per runtime-isolation.md TI-06 |
| Avg dispatch rate per tenant | `R_dispatch_per_sec_per_tenant` | as above |
| Avg session ops per invocation | `S_ops_per_invocation` | empirical ≈ 5 (turn read + scratchpad write + completion seal) |
| Avg session conversation size (KB) | `K_session_bytes_avg` | empirical ≈ 50 KB after 10 turns |
| Avg session lifetime (hours) | `H_session_lifetime_hours` | empirical ≈ 24 |
| Avg capability cold-cache miss rate | `M_cache_miss_rate` | empirical ≈ 0.01 (1%) |
| Capabilities mirrored per tenant | `Q_capabilities_per_tenant` | empirical ≈ 10 |

## Capability-Executor Sizing

### Formulae

```
total_dispatch_rate_per_sec = N_tenants × R_dispatch_per_sec_per_tenant
total_concurrent_invocations = N_tenants × C_concurrent_per_tenant

executor_replicas = max(3, ceil(total_dispatch_rate_per_sec / 1000)) × 1.3 buffer
                  (1000/sec per executor pod baseline; HA quorum min 3)
```

References: empirical baseline from oyatie M0 prototype + Bedrock public benchmarks (`docs.aws.amazon.com/bedrock/latest/userguide/limits.html`). Verify-at-deploy.

## Session-State Sizing (Redis 7.4 LTS)

### Formulae

```
total_active_sessions = N_tenants × C_concurrent_per_tenant × 10
  (10× concurrency to account for idle-but-recent sessions)
total_session_bytes_hot = total_active_sessions × K_session_bytes_avg
total_session_ops_per_sec = N_tenants × R_dispatch_per_sec_per_tenant × S_ops_per_invocation

redis_shards = ceil(total_session_bytes_hot / 4 GB per shard)  (Redis 7.4 recommended per-shard memory cap)
redis_primary_replicas_per_shard = 1
redis_replica_replicas_per_shard = 1  (replication factor 2 for HA)

redis_ops_capacity_per_shard = 100_000 ops/sec  (Redis 7.4 benchmark, single-node)
redis_total_ops_capacity = redis_shards × redis_ops_capacity_per_shard
assert total_session_ops_per_sec ≤ redis_total_ops_capacity × 0.7  (headroom)
```

References: Redis 7.4 LTS release notes (`redis.io/docs/about/releases/7-4-0/`); Redis OSS sizing guide (`redis.io/docs/management/sizing/`).

### Reference baselines

| Tier | N_tenants | total_active_sessions | total_session_bytes_hot | Redis shards (primary + replica) |
|---|---|---|---|---|
| XS (M01 launch) | 20 | 50,000 | 2.5 GB | 6 shards (1 primary + 1 replica each = 12 nodes) |
| S | 100 | 500,000 | 25 GB | 8 shards |
| M | 1,000 | 5,000,000 | 250 GB | 64 shards |
| L | 10,000 | 50,000,000 | 2.5 TB | 640 shards |

Per-pack-region multiplier: each pack sized at active-tenants-in-pack tier. DR pair: 1.0× primary + 0.6× warm-standby.

## Postgres 16 LTS Sizing (Mirror + Cold Restore + Lifecycle)

### Formulae

```
capability_mirror_rows = N_tenants × Q_capabilities_per_tenant
invocation_lifecycle_rows_per_day = total_dispatch_rate_per_sec × 86400
session_cold_archive_bytes_per_day = N_tenants × C_concurrent_per_tenant × K_session_bytes_avg × (24/H_session_lifetime_hours)

postgres_data_gb = (
    capability_mirror_rows × avg_row_bytes (~5 KB) +
    invocation_lifecycle_rows_per_day × 90 × avg_row_bytes (~2 KB) +  // 90d retention hot
    session_cold_archive_bytes_per_day × 90
) / 1e9

postgres_primary_cores = max(4, ceil(total_dispatch_rate_per_sec / 200))
postgres_replica_count = ceil(read_qps / 5000)  (per-replica capacity)
postgres_replica_count = max(1, postgres_replica_count)  (HA min 1 replica)
```

References: Postgres 16 LTS docs (`postgresql.org/docs/16/`); RDS Postgres benchmarks for sizing comparison.

### Reference baselines

| Tier | postgres_data_gb | Primary cores × cores | Replicas |
|---|---|---|---|
| XS | 50 GB | 8 cores | 1 |
| S | 500 GB | 16 cores | 2 |
| M | 5 TB | 32 cores | 4 |
| L | 50 TB | 64 cores | 8 |

## Invocation-Orchestrator + Runtime-Pool + Cache Sizing

```
orchestrator_replicas = max(2, ceil(total_dispatch_rate_per_sec / 500)) × 1.3 buffer
pool_warm_pods = max(8, ceil(total_concurrent_invocations × 0.1))  (10% pre-warm)
cache_app_replicas = 2 (HA min; mostly mirror reader; low CPU)
```

For M01 XS tier (N_tenants=20; C=50; R=10 dispatch/sec/tenant):
- total_dispatch_rate_per_sec = 200/sec
- total_concurrent_invocations = 1000
- executor_replicas = 4
- orchestrator_replicas = 2
- pool_warm_pods = 100  (covers burst headroom; baseline 8 active + 92 reserve)

Adjusting for cost realism, M01 launch uses pool_warm_pods=8 with HPA up to 100 — capacity-model is the ceiling, not the always-on count.

## Layer-B Sizing Summary (XS tier; M01 launch)

| Component | Replicas | Notes |
|---|---|---|
| capability-executor-app | 4 | per dispatch formula |
| invocation-orchestrator-app | 2 | per orchestrator formula |
| invocation-orchestrator-worker | 2 | timeout monitoring + re-emit |
| runtime-pool-app | 2 | HA min |
| runtime-pool-worker | 2 | health-probe + autoscale-trigger |
| runtime-pool warm pods | 8 (baseline) | HPA up to 100 |
| session-state-app | 2 | HA min |
| capability-registry-cache-app | 2 | HA min |
| capability-registry-cache-worker | 2 | hot-reload subscriber |

## Headroom + Burst

All replica counts include 1.2–1.5× buffers per formulae. In addition:
- **Pre-warmed pool**: per `runtime-pool-stack.md`; cold-start ≤500ms per ADR-0020.
- **HPA**: scales on CPU > 70% OR queue-depth thresholds; ratchets up 2 replicas per event.
- **VPA**: vertical-pod-autoscaler for non-critical components.

## Worked Example: XS Tier (M01 Launch; 20 Tenants pack-kr-only)

```
N_tenants = 20
C_concurrent_per_tenant = 50  (production tier per runtime-isolation.md TI-06)
R_dispatch_per_sec_per_tenant = 10
K_session_bytes_avg = 50 KB

total_dispatch_rate_per_sec = 200/sec
total_concurrent_invocations = 1000
total_active_sessions = 20 × 50 × 10 = 10000
total_session_bytes_hot = 10000 × 50KB = 500 MB
total_session_ops_per_sec = 200 × 5 = 1000 ops/sec

Redis shards: ceil(500MB / 4GB) → minimum 6 shards (rounded up to provide ops capacity 100k × 6 = 600k ops/sec headroom; latency target dominates over memory)
Postgres data: 50 GB; 8-core primary; 1 replica
Executor pods: 4; orchestrator pods: 2; pool warm: 8
```

Storage cost at OCI rates → ~$2,415 compute + $630 storage per pack region per `cost-budget.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-conformance --microservice foundry-runtime` — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review: actual vs forecast; recalibrate inputs.
- Annual reference-architecture refresh.

## References

- Redis 7.4 LTS — `redis.io/docs/about/releases/7-4-0/`; sizing guide — `redis.io/docs/management/sizing/`.
- Postgres 16 LTS — `postgresql.org/docs/16/`.
- AWS Bedrock Agent runtime limits — `docs.aws.amazon.com/bedrock/latest/userguide/limits.html`.
- GCP Vertex AI Agent Builder limits — `cloud.google.com/vertex-ai/docs/quotas`.
- `microservices/foundry-runtime/cost-budget.md`.
- `microservices/foundry-runtime/multi-region.md`.
- `microservices/foundry-runtime/policy/runtime-isolation.md` (per-tenant limits).
