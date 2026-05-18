---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-network
deciders: ops-sre-reliability, axis-network, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/network/cost-budget.md
  - microservices/network/multi-region.md
  - microservices/network/policy/professional-context-isolation.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (network µservice)

## Purpose

Sizing formulas + reference-architecture baselines for every `network` component: WebSocket gateway, Postgres profile + post + connection-graph + endorsement-chain + jobs + groups + pages + events store, Redis feed cache + reactions + trending + notifications + InMail queue, S3 media + document, Meilisearch search (multi-index), Layer-B Rust services, foundry-runtime classifier calls (caption assist T1 + ranker T2 + people-you-may-know T2 + recruiter-stub ranker T2 when activated). Drives `cost-budget.md` and `multi-region.md`.

Difference vs sibling `social`: lower posts/sec (Professional users post less), higher search-people QPS (network is the most-searched surface industry-wide), higher endorsement + connection-action volume, additional indexes (skills, jobs, companies, events).

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | tenancy µservice |
| Professional MAU per tenant | `U_users_per_tenant` | tenancy registry |
| Active concurrent users | `C_active` | gateway runtime metric |
| Posts/sec sustained | `P_post_per_sec` | post-composition metric |
| Posts/sec peak burst | `P_peak` | usually 5× sustained |
| Avg post size (text) | `B_text_bytes` | ~ 320 bytes (longer Professional articles) |
| Posts with media ratio | `M_media_ratio` | ~ 0.10 (10 % of posts carry media in Professional context) |
| Posts with document ratio | `M_doc_ratio` | ~ 0.05 (5 % of posts attach a document) |
| Avg media size | `B_media_mb` | ~ 1.5 MB |
| Avg document size | `B_doc_mb` | ~ 3.5 MB (PDF / Office) |
| Avg followers per account | `F_follower_avg` | ~ 500 (Professional graphs trend larger) |
| Avg followees per account | `F_followee_avg` | ~ 300 |
| Avg 1st-degree connections | `K_1deg_avg` | ~ 250 |
| Connection-requests/sec | `CR_per_sec` | ~ 0.0005 × MAU |
| Reactions per post | `R_reactions_avg` | ~ 8 (extended reaction set + Professional engagement) |
| Comments per post | `C_comments_avg` | ~ 3 |
| Mentions per post | `Mn_mention_avg` | ~ 1.4 |
| Hashtags per post | `H_hashtag_avg` | ~ 1.8 |
| Search-people QPS | `Q_search_people_per_sec` | ~ 0.015 × MAU (most-searched surface) |
| Search-content QPS | `Q_search_content_per_sec` | ~ 0.004 × MAU |
| Search-jobs QPS | `Q_search_jobs_per_sec` | ~ 0.005 × MAU |
| Feed-render QPS | `Fr_per_sec` | ~ 0.25 × C_active |
| InMail send rate | `IM_send_per_sec` | ~ 0.0001 × MAU |
| Endorsement add rate | `EN_add_per_sec` | ~ 0.0008 × MAU |
| Recommendation publish rate | `REC_pub_per_sec` | ~ 0.00002 × MAU |
| Recruiter searches (when activated) | `RS_search_per_sec` | per-tenant configured (default cap 0.0001 × MAU) |

## WebSocket Gateway Sizing

```
gateway_replicas = ceil(C_active / 15_000) × 1.3 buffer
```

Each replica (VM.Standard.E4 4-core) handles ~ 15k stable WebSocket connections. 1.3× buffer for HPA headroom and burst.

| Tier | C_active | Replicas |
|---|---|---|
| XS | 100k | 8 |
| S | 500k | 44 |
| M | 5M | 434 |
| L | 50M | 4340 (shards across cells; not one cell) |

## Postgres Sizing

```
storage_post_per_day_GB    = P_post_per_sec × 86400 × B_text_bytes / 1e9 × 1.4 (index overhead)
storage_connection_edges   = N_tenants × U_users_per_tenant × K_1deg_avg × 64 bytes / 1e9 (forward + reverse index)
storage_endorsement_per_day_GB = EN_add_per_sec × 86400 × 512 bytes / 1e9 (endorsement record + signature)
write_iops_baseline        = P_post_per_sec × 6 (post + audit + ranker-source + content-mod + indexer-source + fanout-source) + CR_per_sec × 4 (connection-request + audit + reverse + indexer) + EN_add_per_sec × 5 (endorsement + signature-seal + audit + indexer + recipient-notify)
write_iops_peak            = baseline × 5
```

| Tier | P_post_per_sec | storage_post_30d_hot | storage_connections | write_iops_peak |
|---|---|---|---|---|
| XS | 500 | ~ 600 GB | ~ 100 GB | 25k |
| S | 2.5k | ~ 3 TB | ~ 800 GB | 125k |
| M | 25k | ~ 30 TB | ~ 8 TB | 1.25M |
| L | 250k | ~ 300 TB (shard) | ~ 80 TB (shard) | 12.5M (shard) |

Per-cell envelope: Postgres primary handles ≤ 25k post/sec at HA-RF=3; beyond this, shard by `(tenant_id mod N)`.

## Redis Sizing

```
feed_cache_ops_per_sec     = Fr_per_sec × 2 (read + write-back)
reaction_ops_per_sec       = P_post_per_sec × R_reactions_avg
trending_ops_per_sec       = P_post_per_sec × H_hashtag_avg × 0.5
notification_ops_per_sec   = P_post_per_sec × F_follower_avg × 0.05 (real-time) + EN_add_per_sec × 1.0 + CR_per_sec × 1.0
inmail_queue_ops_per_sec   = IM_send_per_sec × 3 (send + receipt + spam-check signal)
total_redis_ops_per_sec    = feed_cache + reaction + trending + notification + inmail
redis_memory_bytes         = C_active × 10KB (Professional feed cache slice per active user larger than Personal) + tenant overhead
redis_shard_count          = ceil(total_redis_ops_per_sec / 100_000)
```

| Tier | total_redis_ops_per_sec | Shards | Memory |
|---|---|---|---|
| XS | ~ 25k | 1 (3 nodes HA) | ~ 2 GB |
| S | ~ 130k | 2 | ~ 8 GB |
| M | ~ 1.3M | 14 | ~ 80 GB |
| L | ~ 13M | 130 | ~ 800 GB |

## S3 Media + Document Sizing

```
media_per_day      = P_post_per_sec × M_media_ratio × 86400
doc_per_day        = P_post_per_sec × M_doc_ratio × 86400
storage_per_day_GB = media_per_day × B_media_mb / 1e3 + doc_per_day × B_doc_mb / 1e3
storage_hot_30d    = storage_per_day_GB × 30
storage_cold_pack_retention = storage_per_day_GB × retention_days × 0.6 (compression + dedup)
```

| Tier | media+doc_per_day | storage_hot_30d | storage_cold_180d |
|---|---|---|---|
| XS | ~ 6.5k | ~ 350 GB | ~ 1.4 TB |
| S | ~ 32k | ~ 1.7 TB | ~ 7 TB |
| M | ~ 320k | ~ 17 TB | ~ 70 TB |
| L | ~ 3.2M | ~ 170 TB | ~ 700 TB |

## Meilisearch Sizing (multi-index)

Six indexes: `people`, `content`, `skills`, `jobs`, `companies`, `events`.

```
people_index_doc_count   = N_tenants × U_users_per_tenant
content_index_doc_count_per_day = P_post_per_sec × 86400
skills_index_doc_count   = N_tenants × 800 (canonical skill taxonomy; pack-localised)
jobs_index_doc_count     = N_tenants × 200 active job posts on average
companies_index_doc_count = N_tenants × 50 average pages per tenant
events_index_doc_count_per_day = 50 events × N_tenants

index_size_bytes_total   = sum(doc_count × avg_doc_size × 1.5 (terms + positions))
indexer_workers          = ceil(P_peak / 4000) (each worker handles 4k post/sec indexing throughput)
```

| Tier | index_30d_hot_total | Indexer workers |
|---|---|---|
| XS | ~ 60 GB | 4 |
| S | ~ 300 GB | 6 |
| M | ~ 3 TB | 16 (sharded per index) |
| L | ~ 30 TB | 150 (sharded per index) |

## Connection-Graph Sizing

```
total_edges          = N_tenants × U_users_per_tenant × K_1deg_avg
storage_per_edge     = 72 bytes (ULID-pair + edge-meta + degree-of-separation cache)
graph_storage_GB     = total_edges × 72 / 1e9
connection_write_iops = CR_per_sec × 6 (connection-request + accept + forward + reverse + audit + indexer + notification)
```

| Tier | total_edges | Storage |
|---|---|---|
| XS | ~ 250M | ~ 18 GB |
| S | ~ 2.5B | ~ 180 GB |
| M | ~ 25B | ~ 1.8 TB |
| L | ~ 250B | ~ 18 TB (sharded) |

## Notification Fanout Sizing

```
notification_events_per_sec = P_post_per_sec × F_follower_avg × 1.0 + EN_add_per_sec × 1.0 + CR_per_sec × 1.0
notification_workers        = ceil(notification_events_per_sec / 3000)
```

| Tier | notification_events_per_sec | Notification workers |
|---|---|---|
| XS | ~ 250k | 84 |
| S | ~ 1.25M | 417 |
| M | ~ 12.5M | 4167 (sharded across cells) |
| L | ~ 125M | 41667 (sharded across cells) |

## Foundry-Runtime Classifier Call Sizing

```
moderation_calls_per_sec = (P_post_per_sec + C_comments_avg × P_post_per_sec) × 1.0
ranker_calls_per_sec     = Fr_per_sec (every feed-render asks for ranker; heuristic in P01, ML in P03)
people_you_may_know_per_sec = C_active × 0.005 (PYMK panel refresh on profile view)
recruiter_search_per_sec = RS_search_per_sec (recruiter-stub OFF by default; 0 in default deployment)
batch_size               = 100
```

| Tier | moderation/sec | ranker/sec | PYMK/sec | recruiter/sec (when activated) |
|---|---|---|---|---|
| XS | ~ 2k | ~ 25k | ~ 500 | ~ 0.1 |
| S | ~ 10k | ~ 125k | ~ 2.5k | ~ 1 |
| M | ~ 100k | ~ 1.25M | ~ 25k | ~ 10 |
| L | ~ 1M | ~ 12.5M | ~ 250k | ~ 100 |

(Inference cost attribution flows to `cost-budget.md` foundry-runtime line.)

## Per-Tenant Limits

(Set per tenant_scope at OpenBao onboarding; enforced at Postgres + gateway + worker layers.)

| Limit | trial | sandbox | production | internal |
|---|---|---|---|---|
| max active connections | 1k | 1k | 50k | 100k |
| max accounts per tenant | 1k | 10k | 1M | 100M |
| max post/sec per account | 5 | 10 | 50 | 500 |
| max post/min per account | 30 | 60 | 500 | 5000 |
| max connection-request/week per account | 100 | 100 | 500 | 5000 |
| max follow/hr per account | 50 | 100 | 1000 | 10000 |
| max media size per post | 10 MB image / 200 MB video | 10/200 | 10/200 | 50/500 |
| max document size per post | 25 MB | 25 MB | 100 MB | 250 MB |
| max @mention recipients per post | 50 | 50 | 500 | 1000 |
| max #hashtag per post | 10 | 10 | 30 | 50 |
| max search-people QPS | 10 | 50 | 500 | 5000 |
| max followers per account | 100k | 1M | 30M | 300M |
| max connections per account (LinkedIn-parity bound) | 5k | 30k | 30k | 100k |
| max InMail/day per account | 5 | 25 | 250 | 2500 |
| max endorsements granted/day per account | 50 | 100 | 500 | 5000 |
| post retention max | 30 days | 1 year | 7 years | 10 years |

## Cell Scale-out Triggers

| Trigger | Action |
|---|---|
| Gateway CPU sustained > 70 % | HPA scale-up (≤ 200 replicas) |
| Postgres primary write-IOPS > 70 % | Shard by tenant_id |
| Redis shard CPU > 70 % | Add Redis shard |
| Meilisearch indexer lag > 60s sustained | Add indexer worker; per-index sharding |
| S3 PUT rate > 70 % provisioned | Sharded bucket prefix per-tenant |
| Per-tenant max accounts > 1M | Shard tenant across cells |
| Notification worker queue depth > 100k | Add notification worker; coalesce more digest |
| InMail-bridge queue > 100k | Add inmail-bridge worker; throttle per-tenant |
| Endorsement-chain seal worker lag > 30s | Add seal-batcher worker |
| Connection-graph degree-of-separation cache miss rate > 30% | Expand Redis cache slice; recompute warm tier |

## Cross-Region Story

- M02 launch: single pack-kr region (OCI ap-seoul-1).
- Post-M02 expansion: pack-eu + pack-us + DR pairs; cross-pack replication forbidden (per data-residency.md); per-pack independent capacity.
- No federation in P01.

## References

- `microservices/network/cost-budget.md`.
- `microservices/network/multi-region.md`.
- `microservices/observability/capacity-model.md` (shape reference).
- `microservices/social/capacity-model.md` (sibling reference).
- Postgres tuning: PostgreSQL 16 ops docs.
- Meilisearch ops: `docs.meilisearch.com`.
- Redis Cluster ops: `redis.io/docs/management/scaling/`.
- LinkedIn engineering blog (FollowGraph + Identity Service public posts) — `engineering.linkedin.com`.
