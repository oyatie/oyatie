---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-shorts + ops-sre-reliability
deciders: ops-finops, axis-shorts, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/shorts/capacity-model.md
  - microservices/shorts/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (shorts µservice)

## Purpose

Track the shorts µservice's monthly cloud cost across Postgres + Redis + S3 + CDN + Meilisearch + WebSocket gateway + ffmpeg transcode pool + DRM key system + observability sidecars + Layer-B compute (Rust BC services), per pack region. Surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17) + Cloudflare R2 (per ADR-SHORTS-0001's CDN choice); verify-at-deploy markers called out.

## Cost Categories

| Category | What | Public pricing reference |
|---|---|---|
| Compute (OKE node) | Layer-B Rust services (rest + worker + app per BC) | `oracle.com/cloud/compute/pricing/` |
| Postgres (managed or self-hosted on PV) | Video metadata + upload sessions + claims + ages + parental + analytics | `oracle.com/database/pricing/` |
| Redis (managed or self-hosted) | Feed cache + watch-position + like-counters + trending + notifications + ephemeral | `oracle.com/cloud/cache/pricing/` |
| WebSocket gateway pods | Envoy + custom Rust gateway crate | bundled into compute |
| Object storage (S3-compatible) | Video blobs + transcode variants + thumbnails + captions + quarantine | `oracle.com/cloud/storage/object-storage/pricing/` |
| Meilisearch | Hashtag + sound + creator search | self-hosted on PV |
| Block storage (PV) | Postgres data + Meilisearch indexes + Redis AOF | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | WebSocket fanout to public clients; CDN egress for video | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-tenant DEK envelope; media SSE-KMS; DRM key rotation | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack ingress (Envoy / Cloudflare) | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Upload scan | OPSWAT MetaDefender (SaaS) or ClamAV (self-hosted) | `metadefender.opswat.com/pricing` (SaaS path) |
| Transcode | ffmpeg 7.x (CPU primary; GPU OCI BM.GPU.A10 for large-video tier) | OCI BM.GPU.A10 for video-tier batches |
| Foundry-runtime classifier + ASR + ranking | Content-moderation + auto-caption + ranking | foundry-runtime cluster (separate; cost attributed via inference call) |
| Observability sidecar | Alloy sidecar pushing to observability cluster | bundled into compute |
| CDN (video delivery) | Cloudflare R2 + Workers — primary CDN tier (zero egress + global POPs) | `developers.cloudflare.com/r2/pricing/` + `cloudflare.com/plans/workers/` |
| DRM key system | Widevine SecureStop + FairPlay key-server + PlayReady DRM-server (HSM-bound) | per-vendor (Google + Apple + Microsoft) |
| Copyright fingerprint corpus | Postgres + per-pack storage | bundled |

## Per-Component Monthly Cost (XS tier; pack-kr; M03 launch)

Per `capacity-model.md` "XS: 20 tenants, ~100k MAU, ~50 video-upload/sec sustained, ~5k video-plays/sec".

| Component | Replicas × type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| WebSocket gateway | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| video-upload-rest | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| video-transcode-worker (ffmpeg sandboxed) | 16 × VM.Standard.E4 8-core | $2320 | – | $2320 |
| video-storage-rest | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| thumbnail-generation-worker | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| audio-track-library-rest | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| audio-attribution-worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| video-composition-worker | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| feed-timeline-rest + worker | 8 × VM.Standard.E4 4-core | $580 | – | $580 |
| watch-time-tracking-worker | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| like-share-comment-worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| repost-stitch-duet-worker (ffmpeg subset) | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| hashtag-worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| trending-worker | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| notifications-worker | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| content-moderation-worker | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| copyright-claim-worker (Chromaprint + DCT fingerprint matcher) | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| accessibility-captions-worker (orchestrator; ASR is foundry-runtime) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| creator-analytics-worker | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| drm-key-server | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| age-gate-rest | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| parental-controls-rest | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Postgres primary | 1 × VM.Standard.E4 16-core | $290 | $600 PV (12 TB) | $890 |
| Postgres replicas (2) | 2 × VM.Standard.E4 16-core | $580 | $1200 PV | $1780 |
| Redis cluster (3 shards × primary+replica) | 6 × VM.Standard.E4 2-core | $216 | $80 PV | $296 |
| Meilisearch primary + replica | 2 × VM.Standard.E4 4-core | $145 | $400 PV (8 TB) | $545 |
| Video S3 bucket (hot) | – | – | $1500 hot (60 TB) | $1500 |
| Video S3 archive | – | – | $1500 archive (750 TB cold) | $1500 |
| Cloudflare R2 storage (CDN tier) | – | – | $450 (30 TB hot R2) | $450 |
| Cloudflare R2 egress | – | $0 (zero egress on R2) | – | $0 |
| Cloudflare Workers requests | – | $400 (1B requests/mo) | – | $400 |
| OPSWAT scan SaaS | – | $400 (8k scans/day) | – | $400 |
| Foundry-runtime classifier (NSFW + violence + minor-protection) | – | $500 (15M classifications/day at $0.0001 batched) | – | $500 |
| Foundry-runtime ASR auto-caption | – | $300 (10M video-minutes/mo at $0.00003/min batched on-prem) | – | $300 |
| Foundry-runtime ranking | – | $300 (50M feed-renders/day at $0.0001 batched) | – | $300 |
| KMS keyring + DRM key rotation | – | $80 | – | $80 |
| Load balancer (per-pack ingress) | – | $25 | – | $25 |
| Alloy sidecars (per pod) | absorbed | – | – | $80 |
| **XS tier total per pack region** | | **~$8650** | **~$5230** | **~$13880 / month** |

Verify-at-deploy: OCI + Cloudflare pricing changes; reconfirm against `oracle.com/cloud/pricing/` + `developers.cloudflare.com/r2/pricing/` at deploy time. Buffer 15 % for rate increases + 20 % for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Tier | MAU | Upload/sec | Plays/sec | Monthly per pack |
|---|---|---|---|---|
| XS (M03 launch; 20 tenants; 100k MAU) | 100k | 50 | 5k | ~$14k |
| S (~100 tenants; 1M MAU) | 1M | 500 | 50k | ~$70k |
| M (~1k tenants; 10M MAU) | 10M | 5k | 500k | ~$400k |
| L (~10k tenants; 100M MAU) | 100M | 50k | 5M | ~$3.2M |

Cost is transcode-heavy + CDN-heavy. ffmpeg worker pool + S3 + CDN dominate ~60% of variable cost.

## Per-Tenant Unit Economics

| Tier | $/active user / month | $/video-upload | $/video-play | $/video-GB-hot-month |
|---|---|---|---|---|
| XS | $0.140 | $0.011 | $0.0001 | $0.025 |
| S | $0.070 | $0.0054 | $0.00008 | $0.018 |
| M | $0.040 | $0.0031 | $0.00005 | $0.015 |
| L | $0.032 | $0.0025 | $0.00003 | $0.012 |

## Cost-Optimisation Levers

| Lever | Saving | Effort |
|---|---|---|
| Hot-tier video archive after 30d (S3 Standard → Archive) | ~30% storage | Low |
| HLS bitrate-ladder bottom-tier elision for pack-low-bandwidth (drop 1080p tier) | ~20% storage | Low |
| AV1 codec adoption for 1080p + 1440p tiers (better compression than H.264) | ~30% storage; ~5% transcode-CPU | Medium |
| CDN cache-hit-ratio optimisation (per-pack edge tuning) | ~25% egress | Medium |
| Transcode worker spot-instance fallback (queue-depth-aware) | ~40% transcode-CPU during off-peak | High |
| Postgres aggressive vacuuming on tombstoned videos | ~10% storage | Low |
| Meilisearch shard rebalance per-tenant | ~5% search-storage | Medium |
| Per-tenant upload rate cap for free-tier abuse | varies | Medium |
| Foundry-runtime classifier batching (1000 / batch vs 100) | ~70% inference cost | Medium |
| Auto-caption only on plays > 1s (skip never-viewed videos) | ~40% ASR cost | Low |
| Trending compute interval (5min → 10min on low-traffic packs) | ~3% | Low |

## Budget Breach Alerting

| Alert | Threshold | Action |
|---|---|---|
| Pack monthly burn > 110% forecast | sustained 7 days | FinOps review |
| Pack monthly burn > 130% forecast | sustained 3 days | engagement of council-architecture |
| Pack monthly burn > 150% forecast | sustained 1 day | Sev-3 incident |
| Transcode-worker pool sustained > 80% utilisation | sustained 1h | autoscale + capacity review |

CI lane `oya-check-cost-budget --microservice shorts` evaluates against this matrix every 24h.

## References

- `microservices/shorts/capacity-model.md`.
- `microservices/observability/cost-budget.md` (shape reference).
- `microservices/social/cost-budget.md` (sibling reference).
- `microservices/messenger/cost-budget.md` (sibling reference).
- OCI pricing pages (verify at deploy).
- OPSWAT MetaDefender pricing (verify at deploy).
- Cloudflare R2 + Workers pricing (verify at deploy).
- Widevine + FairPlay + PlayReady per-vendor pricing.
