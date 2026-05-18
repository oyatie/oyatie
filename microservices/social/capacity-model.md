---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-social
deciders: ops-sre-reliability, axis-social, council-architecture
related_adrs: [ADR-0117, ADR-0126, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/social/cost-budget.md
  - microservices/social/multi-region.md
  - microservices/social/policy/dual-context-isolation.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (social µservice)

## Purpose

Sizing formulas + reference-architecture baselines for every social component: WebSocket gateway, Postgres profile + post + follow-graph store, Redis feed cache + reactions + trending + notifications, S3 media, Meilisearch search, Layer-B Rust services, foundry-runtime classifier calls. Drives `cost-budget.md` and `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | tenancy µservice |
| MAU per tenant | `U_users_per_tenant` | tenancy registry |
| Active concurrent users | `C_active` | gateway runtime metric |
| Posts/sec sustained | `P_post_per_sec` | post-composition metric |
| Posts/sec peak burst | `P_peak` | usually 5× sustained |
| Avg post size (text) | `B_text_bytes` | ~ 200 bytes |
| Posts with media ratio | `M_media_ratio` | ~ 0.15 (15 % of posts carry media) |
| Avg media size | `B_media_mb` | ~ 1.5 MB (image-heavy mix) |
| Avg followers per account | `F_follower_avg` | ~ 200 |
| Avg followees per account | `F_followee_avg` | ~ 150 |
| Reactions per post | `R_reactions_avg` | ~ 5 |
| Comments per post | `C_comments_avg` | ~ 2 |
| Mentions per post | `Mn_mention_avg` | ~ 1.2 |
| Hashtags per post | `H_hashtag_avg` | ~ 1.5 |
| Search QPS | `Q_search_per_sec` | ~ 0.005 × MAU |
| Feed-render QPS | `Fr_per_sec` | ~ 0.3 × C_active (every 3s while active) |

## WebSocket Gateway Sizing

```
gateway_replicas = ceil(C_active / 15_000) × 1.3 buffer
```

Each replica (VM.Standard.E4 4-core) handles ~ 15k stable WebSocket connections (Envoy + custom Rust gateway). 1.3× buffer for HPA headroom and burst.

| Tier | C_active | Replicas |
|---|---|---|
| XS | 100k | 8 |
| S | 500k | 44 |
| M | 5M | 434 |
| L | 50M | 4340 (shards across cells; not one cell) |

## Postgres Sizing

```
storage_post_per_day_GB    = P_post_per_sec × 86400 × B_text_bytes / 1e9 × 1.4 (index overhead)
storage_post_30d_hot       = storage_post_per_day_GB × 30
write_iops_baseline        = P_post_per_sec × 6 (post + comment + reaction + audit + follow-delta + indexer-source)
write_iops_peak            = P_peak × 6
follow_edges_added_per_sec = C_active × 0.0001 (one new follow per 10000 active sec; loose)
```

| Tier | P_post_per_sec | storage_post_30d_hot | write_iops_peak |
|---|---|---|---|
| XS | 1k | ~ 700 GB | 30k |
| S | 5k | ~ 3.5 TB | 150k |
| M | 50k | ~ 35 TB | 1.5M |
| L | 500k | ~ 350 TB (shard across cells) | 15M (shard across cells) |

Per-cell envelope: Postgres primary handles ≤ 25k post/sec at HA-RF=3; beyond this, shard by `(tenant_id mod N)`.

## Redis Sizing

```
feed_cache_ops_per_sec     = Fr_per_sec × 2 (read + write-back)
reaction_ops_per_sec       = P_post_per_sec × R_reactions_avg
trending_ops_per_sec       = P_post_per_sec × H_hashtag_avg × 0.5 (some hashtag-emit batched)
notification_ops_per_sec   = P_post_per_sec × F_follower_avg × 0.05 (only 5% of follower-count get real-time notif; rest digest)
total_redis_ops_per_sec    = feed_cache + reaction + trending + notification
redis_memory_bytes         = C_active × 8KB (feed cache slice per active user) + tenant overhead
redis_shard_count          = ceil(total_redis_ops_per_sec / 100_000)
```

| Tier | total_redis_ops_per_sec | Shards | Memory |
|---|---|---|---|
| XS | ~ 25k | 1 (3 nodes HA) | ~ 1 GB |
| S | ~ 130k | 2 | ~ 4 GB |
| M | ~ 1.3M | 14 | ~ 40 GB |
| L | ~ 13M | 130 | ~ 400 GB |

## S3 Media Sizing

```
media_per_day      = P_post_per_sec × M_media_ratio × 86400
storage_per_day_GB = media_per_day × B_media_mb / 1e3
storage_hot_30d    = storage_per_day_GB × 30
storage_cold_pack_retention = storage_per_day_GB × retention_days × 0.6 (compression + dedup)
```

| Tier | media_per_day | storage_hot_30d | storage_cold_180d |
|---|---|---|---|
| XS | ~ 13k | ~ 600 GB | ~ 2.2 TB |
| S | ~ 65k | ~ 3 TB | ~ 11 TB |
| M | ~ 648k | ~ 30 TB | ~ 110 TB |
| L | ~ 6.5M | ~ 300 TB | ~ 1100 TB |

## Meilisearch Sizing

```
index_doc_count_per_day = P_post_per_sec × 86400 + (profile-corpus growth which is small)
index_size_bytes        = index_doc_count_per_day × B_text_bytes × 1.5 (terms + positions)
index_30d_hot_GB        = index_size_bytes × 30 / 1e9
indexer_workers         = ceil(P_peak / 4000) (each worker handles 4k post/sec indexing throughput)
```

| Tier | index_30d_hot | Indexer workers |
|---|---|---|
| XS | ~ 30 GB | 4 |
| S | ~ 150 GB | 6 |
| M | ~ 1.5 TB | 14 (sharded) |
| L | ~ 15 TB | 130 (sharded) |

## Follow-Graph Sizing

```
total_edges = N_tenants × U_users_per_tenant × F_followee_avg
storage_per_edge = 64 bytes (ULID-pair + edge-meta)
graph_storage_GB = total_edges × 64 / 1e9
follow_write_iops = follow_edges_added_per_sec × 2 (forward + reverse-index)
```

| Tier | total_edges | Storage |
|---|---|---|
| XS | ~ 80M | ~ 5 GB |
| S | ~ 800M | ~ 50 GB |
| M | ~ 8B | ~ 500 GB |
| L | ~ 80B | ~ 5 TB (sharded) |

## Notification Fanout Sizing

```
notification_events_per_sec = P_post_per_sec × F_follower_avg × 1.0 (all followers get some notification; real-time vs digest split)
notification_workers        = ceil(notification_events_per_sec / 3000) (each worker handles 3k notif/sec dispatch)
```

| Tier | notification_events_per_sec | Notification workers |
|---|---|---|
| XS | ~ 200k | 67 |
| S | ~ 1M | 334 |
| M | ~ 10M | 3334 (sharded across cells) |
| L | ~ 100M | 33334 (sharded across cells) |

## Foundry-Runtime Classifier Call Sizing

```
classifier_calls_per_sec = (P_post_per_sec + C_comments_avg × P_post_per_sec) × 1.0 (every post + comment goes through moderation)
batch_size               = 100 (batched at foundry-runtime gateway)
batched_inference_per_sec = classifier_calls_per_sec / 100
ranking_calls_per_sec    = Fr_per_sec (every feed-render asks for ranking)
```

| Tier | moderation_calls/sec | ranking_calls/sec |
|---|---|---|
| XS | ~ 3k | ~ 30k |
| S | ~ 15k | ~ 150k |
| M | ~ 150k | ~ 1.5M |
| L | ~ 1.5M | ~ 15M |

(Inference cost attribution flows to `cost-budget.md` foundry-runtime line.)

## Per-Tenant Limits

(Set per tenant_scope at OpenBao onboarding; enforced at Postgres + gateway + worker layers.)

| Limit | trial | sandbox | production | internal |
|---|---|---|---|---|
| max active connections | 1k | 1k | 50k | 100k |
| max accounts per tenant | 1k | 10k | 1M | 50M |
| max post/sec per account | 5 | 10 | 100 | 1000 |
| max post/min per account | 50 | 100 | 1000 | 10000 |
| max follow/hr per account | 50 | 100 | 1000 | 10000 |
| max media size per post | 10 MB image / 200 MB video | 10/200 | 10/200 | 50/500 |
| max @mention recipients per post | 50 | 50 | 500 | 1000 |
| max #hashtag per post | 10 | 10 | 20 | 30 |
| max search QPS | 10 | 50 | 500 | 5000 |
| max followers per account | 100k | 1M | 10M | 100M |
| post retention max | 30 days | 1 year | 7 years | 10 years |

## Cell Scale-out Triggers

| Trigger | Action |
|---|---|
| Gateway CPU sustained > 70 % | HPA scale-up (≤ 200 replicas) |
| Postgres primary write-IOPS > 70 % | Shard by tenant_id |
| Redis shard CPU > 70 % | Add Redis shard |
| Meilisearch indexer lag > 60s sustained | Add indexer worker; enable Postgres-ILIKE fallback |
| S3 PUT rate > 70 % provisioned | Sharded bucket prefix per-tenant |
| Per-tenant max accounts > 1M | Shard tenant across cells |
| Notification worker queue depth > 100k | Add notification worker; coalesce more digest |

## Cross-Region Story

- M02 launch: single pack-kr region (OCI ap-seoul-1).
- Post-M02 expansion: pack-eu + pack-us + DR pairs; cross-pack replication forbidden (per data-residency.md); per-pack independent capacity.
- Federation egress (Professional-tier only) opt-in; doesn't add to in-pack capacity since it routes through `federation-gateway` workers attributed separately.

## References

- `microservices/social/cost-budget.md`.
- `microservices/social/multi-region.md`.
- `microservices/observability/capacity-model.md` (shape reference).
- `microservices/messenger/capacity-model.md` (sibling reference).
- Postgres tuning: PostgreSQL 16 ops docs.
- Meilisearch ops: `docs.meilisearch.com`.
- Redis Cluster ops: `redis.io/docs/management/scaling/`.
