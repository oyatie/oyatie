---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-tenancy
deciders: ops-sre-reliability, axis-tenancy, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - tenancy/cost-budget.md
  - tenancy/multi-region.md
  - tenancy/policy/rls-isolation.md (per-tenant limits)
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (tenancy µservice)

## Purpose

Sizing formulas + reference-architecture baseline numbers for Layer-A persistence (Postgres + Citus + Patroni + Valkey) and Layer-B oyatie-owned components. Drives `cost-budget.md` and `multi-region.md`. Numbers cite Citus + Postgres + Patroni reference architectures; verify-against-current-Citus-docs marker called out.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | OpenBao tenant-resolver |
| Validate-path QPS per tenant (avg) | `V_validate_qps_per_tenant` | observability metrics |
| Concurrent activations (peak) | `A_concurrent` | tenant-onboarding pipeline |
| DSR cascades in flight | `D_in_flight` | DSR worker queue |
| Tenant lifecycle write events / day | `L_lifecycle_writes_per_day` | Workflow events |
| Audit-chain seals / day | `S_seals_per_day` | audit-chain emission count |
| Citus shard size (max) | `K_shard_max_rows` | Citus operational guidance (~10 GB / shard) |

## Postgres + Citus Sizing

### Formulae

```
total_validate_qps = N_tenants × V_validate_qps_per_tenant

postgres_primary_cpu = max(8 cores, ceil(total_validate_qps × 0.01 ms_per_validate_at_postgres))
postgres_data_volume = N_tenants × 5 KB tenant_metadata + N_tenants × 1 KB rls_policy_index + audit-chain index
postgres_wal_volume = postgres_data_volume × 0.3 / day  (WAL turnover ratio per Patroni docs)

citus_coordinator_cpu = max(4 cores, ceil(L_lifecycle_writes_per_day / 86400 × 100ms_coord_latency))
citus_worker_count = ceil(N_tenants / K_shard_max_rows × 8 shards_per_worker)  // ~8 shards / worker baseline
```

References:
- Postgres tuning guide — `postgresql.org/docs/16/runtime-config.html`.
- Citus capacity planning — `docs.citusdata.com/en/stable/admin_guide/cluster_management.html`.
- Patroni HA topology — `patroni.readthedocs.io/en/latest/`.

### Per-component replica formulae

```
postgres_primary_replicas = 1  // always 1 primary
postgres_sync_replicas    = 2  // min 2 for quorum
postgres_async_replicas   = max(0, ceil(N_tenants / 10000))  // 0 for XS, scales with size

citus_coordinator_replicas = 1 + 1 sync_replica via patroni  // Patroni-managed HA
citus_worker_replicas     = max(4, ceil(N_tenants / K_shard_max_rows × 8))

valkey_validate_replicas  = 3  // Sentinel HA
valkey_cell_replicas       = 2  // smaller cache

patroni_dcs_etcd_replicas = 3  // quorum 2-of-3
```

### Reference-architecture baselines

| Scale tier | N_tenants | total_validate_qps | Replica counts |
|---|---|---|---|
| **XS** (oyatie M01-launch; ~20 tenants) | 20 | ~2k | postgres={primary=1, sync=2, async=0}, citus={coord=1, workers=4}, valkey-validate=3, valkey-cell=2, dcs-etcd=3 |
| **S** (~100 tenants; small SaaS) | 100 | ~10k | postgres={primary=1, sync=2, async=1}, citus={coord=1, workers=8}, valkey-validate=3, valkey-cell=2 |
| **M** (~1000 tenants; medium SaaS) | 1000 | ~100k | postgres={primary=1, sync=2, async=2}, citus={coord=1, workers=24}, valkey-validate=6, valkey-cell=3 |
| **L** (~10000 tenants; large SaaS / hyperscaler) | 10000 | ~1M | postgres={primary=1, sync=2, async=4}, citus={coord=1, workers=80}, valkey-validate=24, valkey-cell=8 |

Per-pack-region multiplier: each pack has its own cluster sized at the active-tenants-in-pack tier. DR pair (pack-eu, pack-us, etc.) sized 1.0× primary + 0.6× warm-standby (snapshot-restore in ≤ 1h).

## Valkey Cache Sizing

```
valkey_memory_per_node = N_tenants × 256 B (validate cache entry) × 2 (replication factor)
valkey_validate_memory = max(8 GB, valkey_memory_per_node × 1.5 buffer)

valkey_cell_memory = N_tenants × 128 B (cell-assignment entry) × 2
                   = max(1 GB, valkey_memory_per_node × 1.5)
```

For XS tier: 20 tenants × 256 B × 2 × 1.5 ≈ 16 KB; 8 GB is the floor (Valkey min pod sizing).

## Layer-B Sizing (oya-tenancy-*)

```
tenant_lifecycle_rest_replicas  = max(3, ceil(qps_rest / 100)) × 2 HA
tenant_lifecycle_worker_replicas = max(2, ceil(A_concurrent / 50)) × 2 HA
isolation_policy_rest_replicas  = max(3, ceil(jwt_issue_qps / 1000))
isolation_policy_worker_replicas = max(2, ceil(active_packs))  // rotation worker
cell_assignment_worker_replicas = max(2, ceil(N_tenants / 5000))
dsr_cascade_rest_replicas       = max(2, ceil(dsr_submit_qps / 10))
dsr_cascade_worker_replicas     = max(2, ceil(D_in_flight / 100))
```

## Headroom + Burst

- **Pre-warmed pool**: 3 standby validate-rest pods (cold-start budget ≤ 200ms per ADR-0020 inherited).
- **HPA**: scales on CPU > 60% OR p99 latency > 4ms; ratchet up 2 replicas per scale-out event.
- **VPA**: vertical-pod-autoscaler for non-critical (cell-assignment-worker, dsr-cascade-worker).
- **Postgres connection pooling**: PgBouncer in front of Postgres + Citus; pool size = 100 / app pod; per-tenant prepared-statement cache.

## Storage Costs (per pack region)

### Object-storage (Postgres WAL archive)

```
OCI object-storage standard tier: ~$0.0255 / GB / month
OCI archive tier: ~$0.0025 / GB / month

WAL archive policy:
- 0–7d: standard (hot)
- 7d–30d: standard (hot; for PITR window)
- Beyond 30d: deleted (or per-pack regulatory minimum if longer)
```

### Worked example: oyatie XS tier (M01 launch; 20 tenants pack-kr-only)

```
N_tenants = 20
validate_qps = 20 × 100 = 2k
postgres_data = 20 × 5 KB + audit_chain ≈ 100 MB (small)
postgres_wal/day = 30 MB/day; 30d retention = 900 MB
audit_seals/day ≈ 20 × 10 events/day × 256 B = 50 KB/day; 30d = 1.5 MB

Total persistent storage (XS, M01 launch):
  ~ 200 GB Postgres data (oversized; safety margin) + 50 GB WAL + 100 GB Citus worker
  ~ 350 GB total / pack region

Total monthly persistent storage cost:
  ~ 350 GB × $0.0255 = ~$9/month (negligible compared to compute)
  + 1.9 TB archive (30d × 30 MB/day × 30d retention; 10× buffer) ≈ $5/month archive
```

Cost projections per scale tier in `cost-budget.md`.

## Per-Tenant Limits

Per `policy/rls-isolation.md` and threat-model.md T-D-01:

| Limit | XS tier (default) | Trial scope | Production scope | Sandbox | Internal |
|---|---|---|---|---|---|
| Max validate RPS / tenant | 1000 | 100 | 10000 (scales per plan) | 500 | unlimited |
| Max concurrent activations / tenant | 1 (rare) | 1 | 1 | 1 | 10 |
| Max DSR submissions / day | 5 | 1 | 10 | 5 | unlimited |
| Max RLS-policy reads / day | 1000 | 100 | 10000 | 1000 | unlimited |

Excess returns HTTP 429 + emits `oya_tenancy_rate_limit_exceeded_total{tenant_id, dimension}`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-conformance --microservice tenancy` — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review: actual usage vs forecast; recalibrate `V_validate_qps_per_tenant` average.
- Annual reference-architecture refresh: re-verify against current Citus + Patroni published sizing guides.

## References

- Postgres tuning guide — `postgresql.org/docs/16/runtime-config.html`.
- Citus capacity planning — `docs.citusdata.com`.
- Patroni HA — `patroni.readthedocs.io`.
- Valkey docs — `valkey.io/docs/`.
- PgBouncer docs — `pgbouncer.github.io`.
- OCI compute + storage pricing — `oracle.com/cloud/pricing/`.
- `tenancy/cost-budget.md`.
- `tenancy/multi-region.md`.
- `tenancy/policy/rls-isolation.md` (per-tenant limits).
