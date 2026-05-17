---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-workflow
deciders: ops-sre-reliability, axis-workflow, council-architecture
related_adrs: [ADR-0065, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-studio/cost-budget.md
  - microservices/workflow-studio/multi-region.md
  - microservices/workflow-studio/policy/data-residency.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (workflow-studio µservice)

## Purpose

Sizing formulas + reference-architecture baseline numbers for every Layer-A (CDN + WAF + Postgres + Redis + WebSocket gateway) and Layer-B (visual-canvas-rest + collab-crdt-worker + node-library-registry) component. Drives `cost-budget.md` and `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | OpenBao tenant-resolver |
| Per-tenant active editor sessions | `S_active_per_tenant` | Studio REST metrics |
| Per-tenant editor sessions opened per day | `S_opens_per_day_per_tenant` | telemetry |
| Per-tenant avg saves per day | `Save_per_day_per_tenant` | telemetry |
| Per-tenant avg CRDT ops per second | `Op_per_sec_per_tenant` | WS gateway metrics |
| Per-tenant avg LLM-assist invocations per day | `LLM_per_day_per_tenant` | telemetry |
| Per-tenant avg seats | `Seats_per_tenant` | tenancy SDK |
| Per-tenant avg definitions | `Def_per_tenant` | telemetry |

## Core Formulae

### Throughput

```
total_active_sessions          = N_tenants × S_active_per_tenant
total_sessions_per_day         = N_tenants × S_opens_per_day_per_tenant
total_saves_per_sec            = N_tenants × Save_per_day_per_tenant / 86400
total_crdt_ops_per_sec         = N_tenants × Op_per_sec_per_tenant
total_ws_connections           = total_active_sessions × 1.2  (some users with multi-tab)
total_llm_invocations_per_day  = N_tenants × LLM_per_day_per_tenant
```

### Storage

```
editor_session_per_row         ≈ 4KB (compressed; viewport + cursor + draft summary) per Postgres
draft_per_row                  ≈ 10KB (full canonical spec snapshot)
license_attribution_per_row    ≈ 256B
audit_seal_per_row             ≈ 256B
llm_assist_per_row             ≈ 5KB (prompt + completion + metadata)

postgres_storage_per_day       = (total_sessions_per_day × editor_session_per_row)
                              + (total_saves_per_sec × 86400 × draft_per_row)
                              + (total_saves_per_sec × 86400 × audit_seal_per_row)
                              + (total_llm_invocations_per_day × llm_assist_per_row)

postgres_storage_30d_hot       = postgres_storage_per_day × 30
postgres_storage_90d_llm       = total_llm_invocations_per_day × 90 × llm_assist_per_row

redis_state_per_session        ≈ 50KB (CRDT state + cursor + presence)
redis_total                    = total_active_sessions × redis_state_per_session × 2 (HA replication)
```

### CDN

```
wasm_bundle_size_per_release   ≈ 8MB (gzip; Leptos WASM + design-system primitives)
asset_chunks_per_release       ≈ 50 chunks (route-split bundles)
cdn_egress_per_session_open    ≈ 12MB (first load + node library descriptors)
cdn_egress_per_session_resume  ≈ 200KB (HMR-style delta)
total_cdn_egress_per_day       = (total_sessions_per_day × cdn_egress_per_session_open)
                              + (total_active_sessions × 24 × cdn_egress_per_session_resume / 4)
```

## Per-Component Replica Formulae

```
postgres_coordinator_replicas       = 2  (HA primary + standby; always)
postgres_worker_replicas            = ceil(total_active_sessions / 50_000) × 1.2 buffer
postgres_read_replica_replicas      = postgres_worker_replicas × 1
redis_sentinel_replicas             = 3  (quorum)

visual_canvas_rest_replicas         = max(2, ceil(qps_rest / 500)) × 1.2
collab_crdt_worker_replicas         = max(3, ceil(total_ws_connections / 30_000)) × 1.5 buffer  (WS-stateful; lease-bound)
node_library_registry_rest_replicas = max(2, ceil(qps_library / 200)) × 1.2
node_library_registry_app_replicas  = 2  (HA composition root)
visual_canvas_app_replicas          = 2  (HA composition root)
```

## Reference-Architecture Baselines

| Scale tier | N_tenants | total_active_sessions | total_ws_connections | Postgres replica counts | WS gateway count |
|---|---|---|---|---|---|
| **XS** (M03-launch; ~5-20 tenants) | 20 | 100 | 120 | coord=2, worker=4, replica=4 | 3 |
| **S** (~100 tenants) | 100 | 1,000 | 1,200 | coord=2, worker=4, replica=4 | 3 |
| **M** (~1k tenants) | 1000 | 10,000 | 12,000 | coord=2, worker=8, replica=8 | 6 |
| **L** (~10k tenants) | 10000 | 100,000 | 120,000 | coord=2, worker=40, replica=40 | 40 |
| **XL** (~100k tenants; hyperscaler) | 100000 | 1,000,000 | 1,200,000 | coord=2, worker=400, replica=400 | 400 |

Per-pack-region multiplier: each pack has own cluster sized at active-tenants-in-pack tier. DR pair sized 1.0× primary + 0.6× warm-standby.

## Headroom + Burst

All replica counts include buffer multipliers (1.2-1.5×). In addition:

- **Pre-warmed pool**: 5 standby WS gateway pods per cell; cold-start budget ≤ 1s.
- **HPA**: scales on CPU > 70% OR WS connection count > 70% pod cap; ratchets 2 replicas per scale-out event.
- **VPA**: vertical-pod-autoscaler for Postgres workers + Redis; sized to recommended memory.
- **Burst absorbing**: 30s of session-open backlog absorbed by Redis ephemeral queue before back-pressure.

## Postgres + Citus Sizing

```
total_postgres_tables          = (editor_sessions + drafts + license_attributions + audit_seals + llm_assists) × N_tenants_per_shard
citus_distributed_tables       = editor_sessions, drafts, license_attributions, llm_assists (sharded on tenant_id)
citus_reference_tables         = node_library_descriptors_lookup (replicated to all workers)

per_shard_size_target          = ≤ 500 GB (Citus reference)
shard_count                    = ceil(total_postgres_storage / per_shard_size_target)
```

## Redis Sizing

```
total_redis_keys               = total_active_sessions × 4 (CRDT state + cursor + presence + lease)
                              + total_active_subscriptions
redis_memory                   = total_redis_keys × 50KB + 1GB Sentinel overhead
redis_replicas                 = 3 (quorum)
```

## WebSocket Gateway Sizing

```
ws_connections_per_pod         = 10,000 (axum-ws benchmark on E4 4-core)
ws_pod_replicas                = ceil(total_ws_connections / ws_connections_per_pod) × 1.5 buffer
```

Per-pod memory budget: 4GB; CPU budget: 2-4 cores.

## CDN Sizing

```
cdn_pop_count_per_pack         = OCI CDN PoP count per pack (KR: 1, EU: 3, US: 6, etc.)
cdn_origin_egress              = total_cdn_egress_per_day × 0.2 (assumes 80% cache hit)
cdn_egress_to_origin_budget    = $X per pack region per month
```

## Worked Example: oyatie XS Tier (M03 launch; 20 tenants pack-kr-only)

```
N_tenants                  = 20
S_active_per_tenant        = 5
S_opens_per_day_per_tenant = 30
Save_per_day_per_tenant    = 100
Op_per_sec_per_tenant      = 2
LLM_per_day_per_tenant     = 5
Seats_per_tenant           = 10
Def_per_tenant             = 50

total_active_sessions      = 20 × 5 = 100
total_sessions_per_day     = 20 × 30 = 600
total_saves_per_sec        ≈ 0.023  (avg)
total_crdt_ops_per_sec     ≈ 40
total_ws_connections       = 120
total_llm_invocations/day  = 100

postgres_storage_per_day   ≈ (600 × 4KB) + (2000 × 10KB) + (2000 × 256B) + (100 × 5KB)
                           ≈ 2.4MB + 20MB + 0.5MB + 0.5MB ≈ 23 MB/day
postgres_storage_30d       ≈ 690 MB

Replica counts:
  postgres_coordinator     = 2
  postgres_worker          = max(ceil(100 / 50_000), 4) → 4 (HA minimum)
  postgres_read_replica    = 4
  redis_sentinel           = 3
  visual_canvas_rest       = 2 (HA min)
  collab_crdt_worker       = max(3, ceil(120 / 30_000)) = 3
  node_library_rest        = 2
  node_library_app         = 2
  visual_canvas_app        = 2

CDN egress (XS):
  cdn_egress_per_day       ≈ (600 × 12MB) + (100 × 24 × 200KB / 4) ≈ 7.2 GB + 0.12 GB ≈ 7.3 GB/day
  cdn_egress_per_month     ≈ 220 GB

Total Studio storage (XS, M03 launch):
  ~21 GB Postgres hot + 10 GB Redis + 220 GB CDN egress
  ~$2600/month per pack region (per cost-budget.md)
```

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-conformance --microservice workflow-studio` — exit 0.
- Quarterly capacity review: actual usage vs forecast; recalibrate per-tenant averages.
- Annual reference-architecture refresh.

## References

- Postgres + Citus docs — `docs.citusdata.com/`.
- Redis Sentinel — `redis.io/topics/sentinel`.
- axum WebSocket — `docs.rs/axum/latest/axum/extract/ws/`.
- OCI CDN — `oracle.com/cloud/cdn/`.
- `microservices/workflow-studio/cost-budget.md`.
- `microservices/workflow-studio/multi-region.md`.
- `microservices/workflow-studio/policy/data-residency.md`.
