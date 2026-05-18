---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-social + ops-sre-reliability
deciders: ops-finops, axis-social, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/social/capacity-model.md
  - microservices/social/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (social µservice)

## Purpose

Track the social µservice's monthly cloud cost across Postgres + Redis + S3 + Meilisearch + WebSocket gateway + observability sidecars + Layer-B compute (Rust BC services), per pack region. Surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (OKE node) | Layer-B Rust services (rest + worker + app per BC) | `oracle.com/cloud/compute/pricing/` |
| Postgres (managed or self-hosted on PV) | Profiles + posts + follow-graph + reactions + moderation + bookmarks + lists | `oracle.com/database/pricing/` |
| Redis (managed or self-hosted) | Feed cache + reactions + trending + notifications + ephemeral | `oracle.com/cloud/cache/pricing/` |
| WebSocket gateway pods | Envoy + custom Rust gateway crate | bundled into compute |
| Object storage (S3-compatible) | Media blobs + previews + quarantine + transcode variants | `oracle.com/cloud/storage/object-storage/pricing/` |
| Meilisearch | People + content + hashtag search | self-hosted on PV |
| Block storage (PV) | Postgres data + Meilisearch indexes + Redis AOF | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | WebSocket fanout to public clients; CDN egress for media | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-tenant DEK envelope; media SSE-KMS | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack ingress (Envoy / Cloudflare) | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Media scan | OPSWAT MetaDefender (SaaS) or ClamAV (self-hosted) | `metadefender.opswat.com/pricing` (SaaS path) |
| Media transcode | ImageMagick (CPU; bundled into worker) + ffmpeg (GPU optional for video) | OCI BM.GPU.A10 (when video volume large) |
| Foundry-runtime classifier inference | Content-moderation + ranking model | foundry-runtime cluster (separate; cost attributed via inference call) |
| Observability sidecar | Alloy sidecar pushing to observability cluster | bundled into compute |
| CDN (image + video delivery) | Public media tier | Cloudflare R2 (or OCI Object Storage + Cloudflare Workers) |

## Per-Component Monthly Cost (XS tier; pack-kr; M02 launch)

Per `capacity-model.md` "XS: 20 tenants, ~500k MAU, ~1k post/sec sustained".

| Component | Replicas × type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| WebSocket gateway | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| user-profile-rest | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| follow-graph-rest + worker | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| post-composition-rest | 6 × VM.Standard.E4 2-core | $216 | – | $216 |
| post-composition-worker (transcode) | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| feed-timeline-rest + worker | 8 × VM.Standard.E4 4-core | $580 | – | $580 |
| reactions-worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| mentions-worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| hashtags-worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| trending-topics-worker | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| notifications-worker | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| content-moderation-worker | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| search-worker (indexer) | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| Postgres primary | 1 × VM.Standard.E4 8-core | $145 | $300 PV (6 TB) | $445 |
| Postgres replicas (2) | 2 × VM.Standard.E4 8-core | $290 | $600 PV | $890 |
| Redis cluster (3 shards × primary+replica) | 6 × VM.Standard.E4 2-core | $216 | $40 PV | $256 |
| Meilisearch primary + replica | 2 × VM.Standard.E4 4-core | $145 | $400 PV (8 TB) | $545 |
| Media S3 bucket | – | – | $500 hot (20 TB) + $300 cold (150 TB archive) | $800 |
| OPSWAT scan SaaS | – | $400 (8k scans/day) | – | $400 |
| Foundry-runtime classifier (call attribution) | – | $300 (5M moderation classifications/day at $0.0002 batched) | – | $300 |
| KMS keyring | – | $5 | – | $5 |
| Load balancer (per-pack ingress) | – | $25 | – | $25 |
| CDN (Cloudflare R2 + Workers) | – | $150 (20 TB egress) | – | $150 |
| Alloy sidecars (per pod) | absorbed | – | – | $50 |
| **XS tier total per pack region** | | **~$5025** | **~$2140** | **~$7165 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15 % for OCI rate increases + 20 % for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Tier | MAU | Post/sec | Monthly per pack |
|---|---|---|---|
| XS (M02 launch; 20 tenants; 500k MAU) | 500k | 1k | ~$7200 |
| S (~100 tenants; 5M MAU) | 5M | 5k | ~$32k |
| M (~1k tenants; 50M MAU) | 50M | 50k | ~$200k |
| L (~10k tenants; 500M MAU) | 500M | 500k | ~$1.8M |

## Per-Tenant Unit Economics

| Tier | $/active user / month | $/post | $/media-GB-month |
|---|---|---|---|
| XS | $0.014 | $0.0000028 | $0.025 |
| S | $0.011 | $0.0000022 | $0.022 |
| M | $0.0070 | $0.0000016 | $0.018 |
| L | $0.0050 | $0.0000012 | $0.015 |

## Cost-Optimisation Levers

| Lever | Saving | Effort |
|---|---|---|
| Hot-tier feed cache TTL tune (Redis hot 24h → 12h) | ~5% | Low |
| Cold-media archive after 90d (S3 Standard → Archive) | ~30% S3 cost | Low |
| Postgres aggressive vacuuming on tombstoned posts | ~10% storage | Low |
| Meilisearch shard rebalance per-tenant | ~5% search-storage | Medium |
| Per-tenant ingest rate cap for free-tier abuse | varies | Medium |
| Foundry-runtime classifier batching (1000 / batch vs 100) | ~70% inference cost | Medium |
| CDN cache-hit-ratio optimisation (image variant strategy) | ~20% egress | Medium |
| Trending-topic compute interval (5min → 10min on low-traffic packs) | ~3% | Low |

## Budget Breach Alerting

| Alert | Threshold | Action |
|---|---|---|
| Pack monthly burn > 110% forecast | sustained 7 days | FinOps review |
| Pack monthly burn > 130% forecast | sustained 3 days | engagement of council-architecture |
| Pack monthly burn > 150% forecast | sustained 1 day | Sev-3 incident |

CI lane `oya-check-cost-budget --microservice social` evaluates against this matrix every 24h.

## References

- `microservices/social/capacity-model.md`.
- `microservices/observability/cost-budget.md` (shape reference).
- `microservices/messenger/cost-budget.md` (sibling reference).
- OCI pricing pages (verify at deploy).
- OPSWAT MetaDefender pricing (verify at deploy).
- Cloudflare R2 + Workers pricing (verify at deploy).
