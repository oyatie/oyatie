---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-ontology
deciders: ops-sre-reliability, axis-ontology, council-architecture
related_adrs: [ADR-0006, ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/ontology/cost-budget.md
  - microservices/ontology/multi-region.md
  - microservices/ontology/policy/type-isolation.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (ontology µservice)

## Purpose

Sizing formulas + reference-architecture baselines for every Layer-A component (Postgres + Citus / ClickHouse / Valkey / Kafka KRaft) and Layer-B component (`oya-ontology-*`). Drives `cost-budget.md` and `multi-region.md`. Numbers cite Postgres + Citus + ClickHouse published reference architectures; verify-at-deploy markers where upstream may have moved.

## Inputs

The model is parameterised by:

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | OpenBao tenant-resolver |
| Object Type instances per tenant | `K_objects_per_tenant` | `tenant_scope` per-tenant limits (see type-isolation.md TI-06) |
| Object Types in catalog | `M_object_types` | schema registry |
| Action invocations / sec / tenant | `A_actions_per_sec_per_tenant` | `tenant_scope` limits |
| Function reads / sec / tenant | `F_reads_per_sec_per_tenant` | `tenant_scope` limits |
| Avg properties per Object Type | `P_avg_properties` | empirical ≈ 12 |
| Avg bytes per Object Type instance | `B_avg_bytes_per_object` | empirical ≈ 1.5 KB |
| LLM agent sessions concurrent / tenant | `L_sessions_per_tenant` | `tenant_scope` limits |
| Citus shard count per cluster | `S_shards` | 32 per pack (default) |
| Citus replication factor | `RF` | 3 |

## Postgres + Citus Sizing

### Formulae

```
total_objects              = N_tenants × K_objects_per_tenant
total_object_size          = total_objects × B_avg_bytes_per_object
storage_per_day            = (writes/day × B_avg_bytes_per_object) × (1 + index_overhead 0.4) × RF
storage_30d_hot            = storage_per_day × 30
storage_24mo_cold          = (storage_per_day × 730) × cold_compression_ratio 0.5

total_actions_per_sec      = N_tenants × A_actions_per_sec_per_tenant
total_function_reads_per_sec = N_tenants × F_reads_per_sec_per_tenant
```

### Per-component replica formulae

```
postgres_coordinator_replicas = 2 (HA) × 1.0 buffer
                              = 2

citus_worker_replicas       = ceil(total_objects / (1B objects per worker)) × RF
                            = max(2, ceil(N_tenants × K_objects_per_tenant / 1B)) × 3

postgres_streaming_replicas = citus_worker_replicas × 1.0 (1:1 streaming)

action_engine_replicas      = max(2, ceil(total_actions_per_sec / 5_000)) × 1.3 buffer
function_engine_replicas    = max(4, ceil(total_function_reads_per_sec / 2_500)) × 1.3 buffer
agent_gateway_replicas      = max(2, ceil(total_concurrent_llm_sessions / 100)) × 1.3 buffer
```

References: Citus reference architectures — `docs.citusdata.com/en/v12.1/admin_guide/cluster_management.html`. Verify-at-deploy: 2026-05-17.

### Reference-architecture baselines (cites Postgres + Citus)

| Scale tier | N_tenants | total_objects | total_function_reads/s | Postgres replicas |
|---|---|---|---|---|
| **XS** (M02b-launch; ~5–20 tenants) | 20 | 20M | 20k | coordinator=2, worker=8 (+ replicas=8), action-engine=2, function-engine=4, agent-gateway=2 |
| **S** (~100 tenants; small SaaS) | 100 | 100M | 100k | coordinator=2, worker=16 (+ replicas=16), action-engine=4, function-engine=8, agent-gateway=4 |
| **M** (~1k tenants; medium SaaS) | 1000 | 1B | 1M | coordinator=4, worker=64 (+ replicas=64), action-engine=20, function-engine=80, agent-gateway=20 |
| **L** (~10k tenants; large SaaS / hyperscaler) | 10000 | 10B | 10M | coordinator=8, worker=256 (+ replicas=256), action-engine=200, function-engine=800, agent-gateway=200 |

Per-pack-region multiplier: each pack has its own cluster sized at active-tenants-in-pack tier. DR pair (pack-eu eu-frankfurt + eu-amsterdam; pack-us us-ashburn + us-phoenix) sized 1.0× primary + 0.6× warm-standby (snapshot-restore in ≤ 1h).

## ClickHouse Sizing

### Formulae

```
total_clickhouse_storage_per_day = total_object_writes_per_day × B_avg_bytes_per_object × 0.25 (zstd compression)
clickhouse_storage_24mo          = total_clickhouse_storage_per_day × 730 × 0.5 (additional cold-tier compression)

clickhouse_shard_replicas        = max(2, ceil(total_function_reads_olap_per_sec / 1000)) × RF (2 for ClickHouse)
clickhouse_query_replicas        = same as shard_replicas (collocated)
```

### Reference baselines

| Tier | N_tenants | clickhouse_storage_24mo | ClickHouse replicas |
|---|---|---|---|
| XS | 20 | ~6 TB | shard=4 |
| S | 100 | ~30 TB | shard=8 |
| M | 1000 | ~300 TB | shard=32 |
| L | 10000 | ~3 PB | shard=128 |

References: `clickhouse.com/docs/en/operations/tips`.

## Valkey Schema-Registry Cache Sizing

```
valkey_memory_per_pack = M_object_types × P_avg_properties × 0.5 KB (per schema entry) × 1.3 buffer
                       ≈ a few MB per pack region (small)

valkey_replicas        = 3 (HA cluster)
```

## Kafka KRaft Outbox Sizing

```
kafka_storage_per_day = total_outbox_events_per_day × avg_outbox_size 2 KB × replication_factor 3
                      = (total_actions + total_writes) × 2 KB × 3

kafka_broker_replicas = 3 (HA broker cluster; outbox is the only critical traffic)
```

For M02b launch (XS tier; ~10⁶ events/day): Kafka storage ≈ 6 GB/day; 3 brokers each with 100 GB PV; PITR retention 7 days.

## Layer-B Sizing (oya-ontology-*)

```
oya-ontology-object-type-registry-app-replicas = 2 (HA)
oya-ontology-link-type-registry-app-replicas   = 2
oya-ontology-action-type-registry-app-replicas = 2
oya-ontology-function-type-registry-app-replicas = 2
oya-ontology-entity-store-app-replicas         = 2 (writes routed through Postgres adapter; mostly pass-through)
oya-ontology-link-store-app-replicas           = 2
oya-ontology-function-engine-app-replicas      = max(4, ...)  (per formula above)
oya-ontology-action-engine-app-replicas        = max(2, ...)
oya-ontology-query-engine-app-replicas         = max(2, ceil(olap_qps / 500)) × 1.3
oya-ontology-agent-gateway-app-replicas        = max(2, ...)  (per formula above)
oya-ontology-audit-chain-app-replicas          = 2 (worker + REST)
oya-ontology-*-rest-replicas                   = 2 per BC × 1.3 buffer
```

For M02b launch (M_microservices ≈ 36), replicas computed per the per-tier baselines.

## Headroom + Burst

All component-replica counts include buffer multipliers (1.2–1.5×). Additionally:

- **Pre-warmed pool**: 2 standby pods per critical component (Postgres coordinator, action-engine, function-engine, audit-chain worker). Cold-start budget ≤ 500 ms per ADR-0020.
- **HPA**: scales on CPU > 70 % OR queue-depth thresholds; ratchet up 2 replicas per scale-out event.
- **VPA**: vertical-pod-autoscaler for non-critical components (schema-registry, registry workers) sized to recommended memory.

## Storage Costs (per pack region)

### Block storage (Postgres + ClickHouse + Kafka + Valkey) at OCI rates (cites Oracle public pricing, 2026-05-17)

```
OCI block storage standard: ~$0.0255 / GB / month
OCI block storage performance: ~$0.05 / GB / month (high-perf SSD; used for Postgres + ClickHouse hot data)
OCI object-storage standard: ~$0.0255 / GB / month
OCI object-storage infrequent-access: ~$0.01 / GB / month
OCI object-storage archive: ~$0.0025 / GB / month
```

Storage tier policy:
- 0–30d Postgres: standard high-perf block PV
- 0–90d ClickHouse: standard block PV
- 30d–6mo Postgres backups: infrequent-access object storage
- 90d–24mo ClickHouse: cold-tier object storage
- Beyond retention: deleted per `data-residency.md` matrix

### Worked example: oyatie XS tier (M02b launch; 20 tenants pack-kr-only)

```
total_objects = 20 × 1_000_000 = 20M
B_avg_bytes_per_object = 1.5 KB
total_object_size = 30 GB

writes/day = 20 × 100_000 = 2M ops/day
storage_per_day = 2M × 1.5 KB × 1.4 (index overhead) × 3 (RF) ≈ 12.6 GB/day Postgres
storage_30d_hot = 378 GB Postgres hot (high-perf PV)
storage_PITR_3d_cold = 38 GB cold

ClickHouse:
  daily_writes_mirrored = 2M × 1.5 KB × 0.25 (zstd) = 750 MB/day
  storage_90d_hot = 67 GB
  storage_24mo_cold = 750 MB × 730 × 0.5 = 274 GB cold

Kafka:
  daily_outbox = 2M × 2 KB × 3 (RF) ≈ 12 GB/day; 7d retention = 84 GB

Valkey:
  ~50 MB cache

Total ontology storage (XS, M02b launch):
  ~1.6 TB / pack region all-tiers
  ~$80/month per pack region storage cost (mix of hot block + cold object)
```

Cost projections per scale tier in `cost-budget.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-conformance --microservice ontology` — exit 0; deployed replica counts ≥ formula minimums.
- Quarterly capacity review: actual usage vs forecast; recalibrate `K_objects_per_tenant` averages.
- Annual reference-architecture refresh: re-verify against current Postgres + Citus + ClickHouse published sizing guides.

## References

- Postgres + Citus reference architectures — `docs.citusdata.com`.
- ClickHouse operations — `clickhouse.com/docs/en/operations/`.
- Kafka KRaft sizing — `kafka.apache.org/documentation/`.
- Valkey docs — `valkey.io`.
- OCI block + object-storage pricing — `oracle.com/cloud/storage/pricing/`.
- `microservices/ontology/cost-budget.md`.
- `microservices/ontology/multi-region.md`.
- `microservices/ontology/policy/type-isolation.md` (per-tenant limits).
