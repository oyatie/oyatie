---
doc_class: CapacityModel
title: Capacity Model
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-shorts
deciders: ops-sre-reliability, axis-shorts, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/shorts/cost-budget.md
  - microservices/shorts/multi-region.md
  - microservices/shorts/failure-modes.md
review_cadence: per capacity-tier transition (XS→S→M→L)
doc_status: published
---

# Capacity Model (shorts µservice)

## Purpose

Per-tier capacity envelope: dimensions, baseline, max, scale-out trigger. Drives Helm `replicas` + HPA `maxReplicas` + Postgres shard plan + Redis cluster sizing + ffmpeg worker pool autoscale + S3 + CDN sizing + DRM key-server HA + Meilisearch shard plan. Numbers cross-referenced with TikTok / Reels / Shorts / Snapchat Spotlight published telemetry estimates.

## Tiers

| Tier | Tenants | MAU | Upload/sec sustained | Plays/sec | Followers (largest) | Multi-region |
|---|---|---|---|---|---|---|
| XS (M03 launch) | 20 | 100k | 50 | 5k | 100k | single-region pack-kr |
| S | 100 | 1M | 500 | 50k | 1M | single pack + DR pair |
| M | 1k | 10M | 5k | 500k | 10M | multi-region per pack |
| L | 10k | 100M | 50k | 5M | 100M+ | multi-region per pack |

## Per-Component Capacity

### Postgres (video metadata + claims + ages + parental + analytics + audio-track-library + comments)

| Tier | Primary | Replicas | Storage | Shards | Burn rule |
|---|---|---|---|---|---|
| XS | 1 × 16-core 128GB | 2 | 12 TB | none | n/a |
| S | 1 × 32-core 256GB | 3 | 60 TB | none | when tenant shard > 1k upload/sec |
| M | 1 × 64-core 512GB primary + tenant-sharded | 3 per shard | 600 TB | 4 shards | when tenant shard > 1k upload/sec |
| L | tenant-sharded × 16 | 3 per shard | 6 PB | 16 shards | continuous monitoring |

### Redis (feed-cache + watch-position + like-counters + trending + notifications)

| Tier | Shards | Memory per shard | Total memory |
|---|---|---|---|
| XS | 3 | 16 GB | 48 GB |
| S | 6 | 32 GB | 192 GB |
| M | 16 | 64 GB | 1 TB |
| L | 64 | 128 GB | 8 TB |

Sharding key: `(tenant_id, video_id) mod N` for feed cache; `(tenant_id, viewer_ref) mod N` for watch-position.

### S3 (video blobs + transcode variants + thumbnails + captions + quarantine)

| Tier | Hot (last 30d) | Cold archive | Buckets |
|---|---|---|---|
| XS | 60 TB hot + 750 TB archive | – | per-tenant prefix; KMS SSE; Object Lock for Professional |
| S | 600 TB hot + 7.5 PB archive | – | per-tenant prefix |
| M | 6 PB hot + 75 PB archive | tiered (Standard→IA→Glacier) | per-tenant prefix |
| L | 60 PB hot + 750 PB archive | tiered | per-tenant prefix |

### CDN (Cloudflare R2 + Workers)

| Tier | Edge hot R2 | Workers requests/mo | Egress |
|---|---|---|---|
| XS | 30 TB | 1 B | $0 (zero-egress R2) |
| S | 300 TB | 10 B | $0 |
| M | 3 PB | 100 B | $0 |
| L | 30 PB | 1 T | $0 |

### ffmpeg Transcode Worker Pool (gVisor-sandboxed)

| Tier | Baseline workers | Max workers | Trigger | Storage scratch |
|---|---|---|---|---|
| XS | 16 × 8-core | 200 | queue-depth > 100 | 100 GB PV per worker |
| S | 50 | 1000 | queue-depth > 500 | 100 GB PV per worker |
| M | 200 | 5000 | queue-depth > 2000 | per-pack pool |
| L | 1000 | 20000 | queue-depth > 10000 | per-pack pool |

KEDA-style autoscaler tied to `oya_shorts_transcode_queue_depth`. Workers terminate at idle 5min.

### Meilisearch (hashtag + sound + creator search)

| Tier | Shards | Memory per shard | Total index size |
|---|---|---|---|
| XS | 2 | 8 GB | 8 TB |
| S | 4 | 16 GB | 40 TB |
| M | 16 | 32 GB | 400 TB |
| L | 64 | 64 GB | 4 PB |

### DRM key-server (Widevine + FairPlay + PlayReady)

| Tier | Active key-servers per pack | License throughput | Per-content keys cached |
|---|---|---|---|
| XS | 2 (active-active HA) | 5k licenses/sec | 100k keys hot |
| S | 4 | 50k licenses/sec | 1M keys hot |
| M | 8 per pack | 500k licenses/sec | 10M keys hot |
| L | 16 per pack | 5M licenses/sec | 100M keys hot |

### Fingerprint Matcher (Chromaprint + DCT perceptual-hash)

| Tier | Workers | Corpus size hot | Match latency p95 |
|---|---|---|---|
| XS | 4 × 4-core | 10M fingerprints | ≤ 2s |
| S | 16 | 100M | ≤ 2s |
| M | 64 | 1B | ≤ 2s |
| L | 256 | 10B | ≤ 2s |

### WebSocket Gateway

| Tier | Replicas | Max concurrent sessions per replica | Total concurrent sessions |
|---|---|---|---|
| XS | 6 × 4-core | 50k | 300k |
| S | 30 | 50k | 1.5M |
| M | 200 | 50k | 10M |
| L | 2000 | 50k | 100M |

## Per-Cell Capacity Envelope

| Dimension | Baseline | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active creators / cell | 100k | 1M | upload-queue depth > 70% |
| Concurrent viewers / cell | 500k | 5M | feed-load p95 > 250ms |
| Video uploads/sec sustained | 50 | 1000 | transcode worker pool saturation |
| Video plays/sec | 5k | 100k | CDN egress > 70% |
| Total videos per pack | 10M | 1B | S3 prefix cardinality |
| Audio-track corpus per pack | 100k | 10M | Meilisearch shard saturation |
| Fingerprint corpus | 10M | 1B | matcher latency p95 > 2s |
| Per-creator follower count | 100k | 100M | celebrity tier (separate fanout strategy) |

## Scale-Out Mechanics

- HPA on REST pods: CPU > 70 %, min 8, max 200 replicas at XS; max scales linearly with tier.
- ffmpeg transcode pool: KEDA queue-depth-based autoscale; min 16, max 200 at XS.
- Postgres shard-by-tenant once cell hits 1k upload/sec aggregate.
- Redis cluster sharding by `(tenant_id, video_id) mod N`; rebalance per 4x growth.
- CDN POP-presence per pack region; multi-region for high-fanout videos.
- DRM key-server HA cluster: rotation 90d; active-active across two AZ.
- Fingerprint corpus partitions by `(pack, fingerprint_prefix mod N)`.

## Notification Fanout Sharding

Celebrity tier (>1M followers): sharded notification workers; per-recipient idempotent processing; coalesce digest for low-priority notifications; backpressure-throttle.

- 10k followers: p99 ≤ 2s
- 100k followers: p99 ≤ 5s
- 1M followers: p99 ≤ 15s
- 100M followers (super-celebrity): p99 ≤ 5 min via batched fanout

## Burn Rules

| Burn rule | Action |
|---|---|
| Postgres CPU > 70 % sustained 10min | shard prep; capacity meeting |
| Redis memory > 75 % | shard add |
| Transcode queue-depth > 1000 sustained 5min | autoscale workers up; if cap hit, defer to lower-priority tier (free users delayed) |
| CDN cache-hit-ratio < 70 % | edge-tuning meeting |
| ffmpeg worker error-rate > 1 % | gVisor sandbox CVE check; pin to last-known-good ffmpeg |
| Fingerprint matcher latency p95 > 2s | shard partition; or batch-size tune |
| DRM license issuance rate > 80% of key-server capacity | HA cluster add |

## References

- `microservices/shorts/cost-budget.md`.
- `microservices/shorts/multi-region.md`.
- `microservices/shorts/failure-modes.md`.
- `microservices/social/capacity-model.md` (sibling pattern).
- TikTok / Reels / YouTube Shorts published telemetry estimates.
- OCI BM.GPU.A10 docs.
- KEDA docs.
- Cloudflare R2 + Workers pricing.
