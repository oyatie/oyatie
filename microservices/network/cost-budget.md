---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-network + ops-sre-reliability
deciders: ops-finops, axis-network, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/network/capacity-model.md
  - microservices/network/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (network µservice)

## Purpose

Track the `network` µservice's monthly cloud cost across Postgres + Redis + S3 + Meilisearch + WebSocket gateway + observability sidecars + Layer-B compute (Rust BC services) + foundry-runtime classifier inference + cross-µservice bridges (messenger / mail / calendar / ATS), per pack region. Surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (OKE node) | Layer-B Rust services (rest + worker + app per BC) | `oracle.com/cloud/compute/pricing/` |
| Postgres (managed or self-hosted on PV) | Profiles + posts + connection-graph + endorsement-chain + jobs + groups + pages + audit | `oracle.com/database/pricing/` |
| Redis (managed or self-hosted) | Feed cache + reactions + trending + notifications + InMail queue + ephemeral | `oracle.com/cloud/cache/pricing/` |
| WebSocket gateway pods | Envoy + custom Rust gateway crate | bundled into compute |
| Object storage (S3-compatible) | Media blobs + document attachments + previews + quarantine + transcode variants + profile-export | `oracle.com/cloud/storage/object-storage/pricing/` |
| Meilisearch | People + content + skills + jobs + companies + events search | self-hosted on PV |
| Block storage (PV) | Postgres data + Meilisearch indexes + Redis AOF | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | WebSocket fanout + CDN egress for media | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-tenant DEK envelope; media SSE-KMS; endorsement Ed25519 keystore | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack ingress (Envoy / Cloudflare) | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Media scan | OPSWAT MetaDefender (SaaS) or ClamAV (self-hosted) | `metadefender.opswat.com/pricing` (SaaS path) |
| Media transcode | ImageMagick (CPU; bundled into worker) + ffmpeg (GPU optional for video) | OCI BM.GPU.A10 (when video volume large) |
| Foundry-runtime classifier inference | Caption/article assist + ranker + people-you-may-know + recruiter ranker (EU AI Act high-risk) | foundry-runtime cluster (separate; cost attributed via inference call) |
| Observability sidecar | Alloy sidecar pushing to observability cluster | bundled into compute |
| CDN | Public media tier | Cloudflare R2 (or OCI Object Storage + Cloudflare Workers) |
| Bias-audit pipeline | Per-release golden-set eval + 4/5-rule statistical compute | bundled into foundry-runtime |

## Per-Component Monthly Cost (XS tier; pack-kr; M02 launch)

Per `capacity-model.md` "XS: 20 tenants, ~1M Professional MAU, ~500 post/sec sustained".

| Component | Replicas × type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| WebSocket gateway | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| professional-profile-rest | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| professional-graph-rest + worker | 6 × VM.Standard.E4 2-core | $216 | – | $216 |
| connection-request-rest + worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| post-composition-rest | 6 × VM.Standard.E4 2-core | $216 | – | $216 |
| post-composition-worker (transcode) | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| feed-timeline-rest + worker | 8 × VM.Standard.E4 4-core | $580 | – | $580 |
| reactions-worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| mentions-worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| hashtags-worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| trending-topics-worker | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| notifications-worker | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| endorsement-engine-rest + worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| skill-assessments-rest + worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| profile-verification-rest | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| pages-rest + groups-rest + events-bridge | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| inmail-bridge-worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| jobs-handoff-worker | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| search-worker (indexer; multi-index people/content/skills/jobs/companies/events) | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| abuse-reporting-rest | 1 × VM.Standard.E4 2-core | $36 | – | $36 |
| Postgres primary | 1 × VM.Standard.E4 8-core | $145 | $500 PV (10 TB) | $645 |
| Postgres replicas (2) | 2 × VM.Standard.E4 8-core | $290 | $1000 PV | $1290 |
| Redis cluster (3 shards × primary+replica) | 6 × VM.Standard.E4 2-core | $216 | $80 PV | $296 |
| Meilisearch primary + replica (multi-index) | 2 × VM.Standard.E4 8-core | $290 | $700 PV (14 TB) | $990 |
| Media + document S3 bucket | – | – | $700 hot (28 TB) + $400 cold (200 TB archive) | $1100 |
| OPSWAT scan SaaS | – | $400 (8k scans/day) | – | $400 |
| Foundry-runtime classifier (call attribution) | – | $550 (T1 caption/article + T2 ranker + people-you-may-know + recruiter ranker batched) | – | $550 |
| KMS keyring (incl. endorsement Ed25519 store) | – | $10 | – | $10 |
| Load balancer (per-pack ingress) | – | $25 | – | $25 |
| CDN (Cloudflare R2 + Workers) | – | $150 (20 TB egress) | – | $150 |
| Alloy sidecars (per pod) | absorbed | – | – | $80 |
| **XS tier total per pack region** | | **~$6225** | **~$2680** | **~$9305 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15 % for OCI rate increases + 20 % for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Tier | Professional MAU | Post/sec | Monthly per pack |
|---|---|---|---|
| XS (M02 launch; 20 tenants; 1M MAU) | 1M | 500 | ~$9300 |
| S (~100 tenants; 10M MAU) | 10M | 2.5k | ~$42k |
| M (~1k tenants; 100M MAU) | 100M | 25k | ~$260k |
| L (~10k tenants; 1B MAU; hyperscaler) | 1B | 250k | ~$2.3M |

## Per-Tenant Unit Economics

| Tier | $/Professional active user / month | $/post | $/connection-action | $/recruiter-search |
|---|---|---|---|---|
| XS | $0.0093 | $0.0000072 | $0.0000004 | $0.04 (recruiter-stub OFF in P01; XS approximation when activated) |
| S | $0.0042 | $0.0000054 | $0.0000003 | $0.03 |
| M | $0.0026 | $0.0000040 | $0.0000002 | $0.022 |
| L | $0.0023 | $0.0000030 | $0.0000002 | $0.018 |

## Cost-Optimisation Levers

| Lever | Saving | Effort |
|---|---|---|
| Hot-tier feed cache TTL tune (Redis hot 24h → 12h) | ~5% | Low |
| Cold-media archive after 90d (S3 Standard → Archive) | ~30% S3 cost | Low |
| Postgres aggressive vacuuming on tombstoned posts + revoked endorsements | ~8% storage | Low |
| Meilisearch shard rebalance per-tenant + per-index | ~7% search-storage | Medium |
| Per-tenant ingest rate cap for free-tier abuse | varies | Medium |
| Foundry-runtime classifier batching (1000 / batch vs 100) | ~70% inference cost | Medium |
| CDN cache-hit-ratio optimisation (image variant strategy) | ~20% egress | Medium |
| Trending-topic compute interval (5min → 10min on low-traffic packs) | ~3% | Low |
| Profile-export pre-cache for serial vCard exports | ~15% export latency cost | Low |
| Endorsement-chain re-batching at audit-chain seal time | ~10% audit-chain cost | Low |

## Budget Breach Alerting

| Alert | Threshold | Action |
|---|---|---|
| Pack monthly burn > 110% forecast | sustained 7 days | FinOps review |
| Pack monthly burn > 130% forecast | sustained 3 days | engagement of council-architecture |
| Pack monthly burn > 150% forecast | sustained 1 day | Sev-3 incident |

CI lane `oya-check-cost-budget --microservice network` evaluates against this matrix every 24h.

## References

- `microservices/network/capacity-model.md`.
- `microservices/observability/cost-budget.md` (shape reference).
- `microservices/social/cost-budget.md` (sibling reference).
- OCI pricing pages (verify at deploy).
- OPSWAT MetaDefender pricing (verify at deploy).
- Cloudflare R2 + Workers pricing (verify at deploy).
