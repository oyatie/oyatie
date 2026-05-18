---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-messenger
deciders: ops-sre-reliability, axis-messenger, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/messenger/cost-budget.md
  - microservices/messenger/multi-region.md
  - microservices/messenger/policy/dual-context-isolation.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (messenger µservice)

## Purpose

Sizing formulas + reference-architecture baselines for every messenger component: WebSocket gateway, Postgres message store, Redis presence/read-receipt, S3 attachments, Tantivy search index, Layer-B Rust services. Drives `cost-budget.md` and `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | tenancy µservice |
| Active users (MAU) per tenant | `U_users_per_tenant` | tenancy registry |
| Active connections (peak concurrent) | `C_active_conn` | gateway runtime metric |
| Messages/sec sustained | `M_msg_per_sec` | message-stream metric |
| Messages/sec peak burst | `M_peak` | usually 5× sustained |
| Avg message size | `B_msg_bytes` | ~ 200 bytes professional, ~ 800 bytes personal-DM (E2E ciphertext is larger) |
| Channels per tenant (avg) | `K_channels` | ~ 50 small tenant; 500 medium; 5000 large |
| Avg channel members | `M_members` | ~ 20 |
| Attachment rate per msg | `A_attach_ratio` | ~ 0.05 (5 % of messages carry attachments) |
| Avg attachment size | `B_attach_mb` | ~ 2 MB |
| Read-receipt fanout per message | `R_recipients` | ~ M_members |
| @mention fanout per message | `Mn_mention_avg` | ~ 1.5 per message |
| Search query rate | `Q_search_per_sec` | ~ 0.001 × MAU |

## WebSocket Gateway Sizing

```
gateway_replicas = ceil(C_active_conn / 15_000) × 1.3 buffer
```

Each replica (VM.Standard.E4 4-core) handles ~ 15k stable WebSocket connections (Envoy + custom Rust gateway). 1.3× buffer for HPA headroom and burst.

| Tier | C_active_conn | Replicas |
|---|---|---|
| XS | 100k | 8 |
| S | 500k | 44 |
| M | 5M | 434 |
| L | 50M | 4340 (shards across cells; not one cell) |

## Postgres Message Store Sizing

```
storage_per_day_GB = M_msg_per_sec × 86400 × B_msg_bytes / 1e9 × 1.4 (index overhead)
storage_30d_hot    = storage_per_day_GB × 30
write_iops_baseline = M_msg_per_sec × 4 (msg + thread + receipt + audit derivative)
write_iops_peak     = M_peak × 4
```

| Tier | M_msg_per_sec | storage_30d_hot | write_iops_peak |
|---|---|---|---|
| XS | 1k | ~ 700 GB | 20k |
| S | 5k | ~ 3.5 TB | 100k |
| M | 50k | ~ 35 TB | 1M |
| L | 500k | ~ 350 TB (shard across cells) | 10M (shard across cells) |

Per-cell envelope: Postgres primary handles ≤ 50k msg/sec at HA-RF=3; beyond this, shard by `(tenant_id mod N)`.

## Redis Presence + Read-Receipt Sizing

```
presence_ops_per_sec     = C_active_conn / 30 (1 heartbeat per 30s)
read_receipt_ops_per_sec = M_msg_per_sec × R_recipients × 0.3 (30 % seen rate within hot window)
total_redis_ops_per_sec  = presence + read_receipt
redis_memory_bytes       = C_active_conn × 400 (per-user keyset)
redis_shard_count        = ceil(total_redis_ops_per_sec / 100_000)
```

| Tier | total_redis_ops_per_sec | Shards | Memory |
|---|---|---|---|
| XS | ~ 9k | 1 (3 nodes HA) | ~ 40 MB |
| S | ~ 50k | 1 | ~ 200 MB |
| M | ~ 500k | 5 | ~ 2 GB |
| L | ~ 5M | 50 | ~ 20 GB |

## S3 Attachment Sizing

```
attachments_per_day = M_msg_per_sec × A_attach_ratio × 86400
storage_per_day_GB  = attachments_per_day × B_attach_mb / 1e3
storage_hot_30d     = storage_per_day_GB × 30
storage_cold_pack_retention = storage_per_day_GB × retention_days × 0.6 (compression)
```

| Tier | attachments_per_day | storage_hot_30d | storage_cold_30d |
|---|---|---|---|
| XS | ~ 4.3k | ~ 256 GB | ~ 154 GB |
| S | ~ 21.5k | ~ 1.3 TB | ~ 770 GB |
| M | ~ 216k | ~ 13 TB | ~ 7.7 TB |
| L | ~ 2.16M | ~ 130 TB | ~ 77 TB |

## Tantivy / Elasticsearch Search Sizing

```
index_doc_count_per_day = M_msg_per_sec × 86400
index_size_bytes        = index_doc_count_per_day × B_msg_bytes × 1.5 (terms + positions)
index_30d_hot_GB        = index_size_bytes × 30 / 1e9
indexer_workers         = ceil(M_peak / 5000) (each worker handles 5k msg/sec indexing throughput)
```

| Tier | index_30d_hot | Indexer workers |
|---|---|---|
| XS | ~ 30 GB | 4 |
| S | ~ 150 GB | 6 |
| M | ~ 1.5 TB | 12 (sharded) |
| L | ~ 15 TB | 100 (sharded) |

## Mention-Router + Notification Sizing

```
mention_events_per_sec = M_msg_per_sec × Mn_mention_avg
mention_workers        = ceil(mention_events_per_sec / 2000) (each handles 2k mentions/sec)
```

| Tier | mention_events_per_sec | Mention workers |
|---|---|---|
| XS | ~ 1.5k | 4 |
| S | ~ 7.5k | 6 |
| M | ~ 75k | 38 |
| L | ~ 750k | 380 |

## Per-Tenant Limits

(Set per tenant_scope at OpenBao onboarding; enforced at Postgres + gateway + worker layers.)

| Limit | trial | sandbox | production | internal |
|---|---|---|---|---|
| max active connections | 1k | 1k | 50k | 100k |
| max channels per tenant | 100 | 100 | 10k | 50k |
| max msg/sec ingest | 100 | 100 | 5k | 50k |
| max attachment size | 100 MB | 100 MB | 5 GB | 5 GB |
| max attachment rate per tenant | 100/min | 100/min | 1000/min | 5000/min |
| max @mention recipients per message | 50 | 50 | 500 | 500 |
| max search QPS | 10 | 10 | 100 | 1000 |
| max channel members per channel | 1000 | 1000 | 10k | 100k |
| message retention max | 7 days | 30 days | 7 years | 10 years |

## Cell Scale-out Triggers

| Trigger | Action |
|---|---|
| Gateway CPU sustained > 70 % | HPA scale-up (≤ 200 replicas) |
| Postgres primary write-IOPS > 70 % | Shard by tenant_id |
| Redis shard CPU > 70 % | Add Redis shard |
| Tantivy indexer lag > 60s sustained | Add indexer worker; enable Postgres-LIKE fallback |
| S3 PUT rate > 70 % provisioned | Sharded bucket prefix per-tenant |
| Per-tenant max channels > 50k | Shard tenant across cells |

## Cross-Region Story

- M02 launch: single pack-kr region (OCI ap-seoul-1).
- Post-M02 expansion: pack-eu + pack-us + DR pairs; cross-pack replication forbidden (per data-residency.md); per-pack independent capacity.

## References

- `microservices/messenger/cost-budget.md`.
- `microservices/messenger/multi-region.md`.
- `microservices/observability/capacity-model.md` (shape reference).
- Postgres tuning: PostgreSQL 16 ops docs.
- Tantivy ops: `github.com/quickwit-oss/tantivy/wiki/Operations`.
- Redis Cluster ops: `redis.io/docs/management/scaling/`.
