---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-workflow
deciders: ops-sre-reliability, axis-workflow, council-architecture
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/cost-budget.md
  - microservices/workflow-engine/multi-region.md
  - microservices/workflow-engine/policy/data-residency.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (workflow-engine µservice)

## Purpose

Sizing formulas + reference-architecture baseline numbers for every Layer-A (Postgres + Citus / Valkey / ClickHouse) and Layer-B (engine workers / REST / SDK) component. Drives `cost-budget.md` and `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | OpenBao tenant-resolver |
| Per-tenant active runs | `R_active_per_tenant` | engine REST metrics |
| Per-tenant runs-per-day | `R_runs_per_day_per_tenant` | per-tenant SDK telemetry |
| Per-tenant avg steps per run | `S_steps_per_run` | per-run telemetry |
| Per-tenant avg event rate | `E_events_per_sec_per_tenant` | event-bus metrics |
| Per-tenant avg subscription count | `Sub_per_tenant` | event-bus subscription registry |
| Engine evaluator cadence | `E_cycle_seconds` | n/a (engine is event-driven, not cadence-driven) |

## Core Formulae

### Throughput

```
total_active_runs              = N_tenants × R_active_per_tenant
total_runs_per_day             = N_tenants × R_runs_per_day_per_tenant
total_steps_per_sec            = total_runs_per_day × S_steps_per_run / 86400
total_events_per_sec_inbound   = N_tenants × E_events_per_sec_per_tenant
total_events_per_sec_outbound  = total_events_per_sec_inbound × avg_subscribers_per_event
total_audit_seals_per_sec      = total_steps_per_sec × 2  (one seal at step-start + step-complete)
```

### Storage

```
run_state_per_row              ≈ 2KB (compressed) per Postgres
step_execution_per_row         ≈ 1.5KB
event_log_per_row              ≈ 1KB
audit_seal_per_row             ≈ 256B

postgres_storage_per_day       = (total_runs_per_day × run_state_per_row)
                              + (total_steps_per_day × step_execution_per_row)
                              + (total_events_per_day × event_log_per_row)

postgres_storage_90d_hot       = postgres_storage_per_day × 90
clickhouse_storage_24mo_cold   = postgres_storage_per_day × 730 × 0.5  (ClickHouse compression)

valkey_state_per_run            ≈ 200B (lease + ephemeral)
valkey_total                    = total_active_runs × valkey_state_per_run × 2 (HA replication)
```

## Per-Component Replica Formulae

```
postgres_coordinator_replicas      = 2  (HA primary + standby; always)
postgres_worker_replicas           = ceil(total_active_runs / 200_000) × 1.2 buffer
postgres_read_replica_replicas     = postgres_worker_replicas × 1  (1:1 ratio)
valkey_sentinel_replicas            = 3  (quorum)
clickhouse_replicas                = ceil(total_audit_seals_per_sec / 1000) × 1.2

execution_engine_worker_replicas   = max(3, ceil(total_steps_per_sec / 2000)) × 1.5 buffer
execution_engine_rest_replicas     = max(2, ceil(qps_rest / 200)) × 1.2
event_bus_worker_replicas          = max(2, ceil(total_events_per_sec_outbound / 5000)) × 1.2
event_bus_rest_replicas            = max(2, ceil(qps_pubsub / 1000)) × 1.2
spec_store_rest_replicas           = max(2, ceil(qps_spec_read / 500)) × 1.2
replay_debugger_backend_rest_replicas = max(2, ceil(qps_debug / 100)) × 1.2

execution_engine_app_replicas      = 2  (HA composition root)
event_bus_app_replicas             = 2
spec_store_app_replicas            = 2
replay_debugger_backend_app_replicas = 2
```

## Reference-Architecture Baselines

| Scale tier | N_tenants | total_active_runs | total_steps_per_sec | Postgres+Citus replica counts | Engine worker count |
|---|---|---|---|---|---|
| **XS** (M02b-launch; ~5–20 tenants) | 20 | 200 | 100 | coord=2, worker=4, replica=4 | 3 |
| **S** (~100 tenants) | 100 | 1,000 | 500 | coord=2, worker=4, replica=4 | 3 |
| **M** (~1k tenants) | 1000 | 10,000 | 5,000 | coord=2, worker=8, replica=8 | 6 |
| **L** (~10k tenants) | 10000 | 100,000 | 50,000 | coord=2, worker=80, replica=80 | 60 |
| **XL** (~100k tenants; hyperscaler) | 100000 | 1,000,000 | 500,000 | coord=2, worker=800, replica=800 | 600 |

Per-pack-region multiplier: each pack has own cluster sized at active-tenants-in-pack tier. DR pair (pack-eu, pack-us, etc.) sized 1.0× primary + 0.6× warm-standby (snapshot-restore in ≤ 1h).

## Headroom + Burst

All replica counts include buffer multipliers (1.2–1.5×). In addition:

- **Pre-warmed pool**: 10 standby engine-worker pods per cell; cold-start budget ≤ 500ms.
- **HPA**: scales on CPU > 70% OR step queue depth > 5k; ratchets 2 replicas per scale-out event.
- **VPA**: vertical-pod-autoscaler for ClickHouse + Postgres workers; sized to recommended memory.
- **Burst absorbing**: 30s of step backlog absorbed by ephemeral Valkey lease queue before back-pressure.

## Postgres + Citus Sizing

```
total_postgres_tables          = (run_state + step_execution + event_log + spec_versions + audit_seals + outbox_lookup) × N_tenants_per_shard
citus_distributed_tables       = run_state, step_execution, event_log, outbox (all sharded on tenant_id)
citus_reference_tables         = spec_versions, audit_seal_metadata (replicated to all workers)

per_shard_size_target          = ≤ 500 GB (Citus reference)
shard_count                    = ceil(total_postgres_storage / per_shard_size_target)
```

References: Citus docs — `docs.citusdata.com/`. Verify-at-deploy.

## Valkey Sizing

```
total_valkey_keys               = total_active_runs × 3 (lease + step-claim + heartbeat)
                              + total_active_subscriptions
valkey_memory                   = total_valkey_keys × 200B + 1GB Sentinel overhead
valkey_replicas                 = 3 (quorum)
```

References: Valkey Sentinel — `valkey.io/topics/sentinel`.

## ClickHouse Sizing

```
clickhouse_rows_per_day        = total_steps_per_day × 2 (each step writes start + end)
                              + total_audit_seals_per_day
clickhouse_storage_per_day     ≈ clickhouse_rows_per_day × 200B (after compression)
clickhouse_replicas            = ceil(insert_qps / 1000) × 1.2
```

References: ClickHouse capacity — `clickhouse.com/docs/en/operations/sizing`.

## Worked Example: oyatie XS Tier (M02b launch; 20 tenants pack-kr-only)

```
N_tenants                  = 20
R_active_per_tenant        = 10  (average active runs per tenant)
R_runs_per_day_per_tenant  = 500
S_steps_per_run            = 10
E_events_per_sec_per_tenant = 0.5
Sub_per_tenant             = 5

total_active_runs          = 20 × 10 = 200
total_runs_per_day         = 20 × 500 = 10,000
total_steps_per_day        = 100,000
total_steps_per_sec        ≈ 1.2 (avg)
total_events_per_sec       ≈ 10
total_audit_seals_per_sec  ≈ 2.4

postgres_storage_per_day   ≈ (10k × 2KB) + (100k × 1.5KB) + (10×86400 × 1KB)
                           ≈ 20MB + 150MB + 864MB ≈ 1GB/day
postgres_storage_90d_hot   ≈ 90GB
clickhouse_storage_24mo    ≈ 1GB × 730 × 0.5 ≈ 365GB

Replica counts:
  postgres_coordinator     = 2
  postgres_worker          = max(ceil(200 / 200_000), 4) → 4 (HA minimum)
  postgres_read_replica    = 4
  valkey_sentinel           = 3
  clickhouse_replica       = 2
  execution_engine_worker  = max(3, ceil(1.2 / 2000)) = 3
  execution_engine_rest    = 2 (HA min)
  event_bus_worker         = 2
  event_bus_rest           = 2
  spec_store_rest          = 2
  replay_debugger_rest     = 2

Total engine storage (XS, M02b launch):
  ~90 TB Postgres hot + 365 GB ClickHouse cold + 50 GB object storage
  ~$1980/month per pack region storage cost

(Note: XS tier sizing dominated by HA minimums; actual usage ~1% of provisioned at M02b launch.)
```

Cost projections per scale tier in `cost-budget.md`.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=capacity-conformance` — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review: actual usage vs forecast; recalibrate per-tenant averages.
- Annual reference-architecture refresh: re-verify against current Postgres / Citus / ClickHouse sizing guides.

## References

- Postgres + Citus docs — `docs.citusdata.com/`.
- Valkey Sentinel — `valkey.io/topics/sentinel`.
- ClickHouse operations — `clickhouse.com/docs/en/operations/`.
- OCI pricing — `oracle.com/cloud/pricing/`.
- `microservices/workflow-engine/cost-budget.md`.
- `microservices/workflow-engine/multi-region.md`.
- `microservices/workflow-engine/policy/data-residency.md`.
