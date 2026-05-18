---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-shorts + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-shorts, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/shorts/policy/data-residency.md
  - microservices/shorts/capacity-model.md
  - microservices/shorts/cost-budget.md
  - microservices/shorts/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (shorts µservice)

## Purpose

Define multi-region topology for shorts across the 11 oyatie packs: pack-pinning, in-pack DR pair, cross-pack replication-forbidden policy, BCDR posture, RPO/RTO targets, failover procedures, CDN POP topology, federation peer topology (Professional-tier only, metadata-only).

## Topology Per Pack

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES | YES (M03 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | DR pair; isolated | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional |

CDN POPs (Cloudflare R2 + Workers): per-pack regional POPs; cross-pack edge replication is metadata-only (manifest references); blob fetch is from in-pack S3 origin via in-pack POP.

## In-Pack DR-Pair Architecture

```text
┌─ Pack <X> ────────────────────────────────────────────────────────────┐
│                                                                       │
│  Primary region                          DR-pair region               │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ WebSocket gateway pool   │            │ Gateway warm-standby      │ │
│  │ Active (handling traffic)│            │ 0.5× capacity             │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐ logical    ┌──────────────────────────┐ │
│  │ Postgres primary + 2 RR  │◀──────────▶│ Postgres logical-replica  │ │
│  │ HA-RF=3                  │ replic     │ async; ≤ 5 s lag          │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ Valkey cluster (3 shards) │            │ Valkey cluster warm        │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐  CRR       ┌──────────────────────────┐ │
│  │ S3 video bucket          │◀──────────▶│ S3 replica                │ │
│  │ + transcode variants     │            │                           │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ ffmpeg transcode pool    │            │ ffmpeg warm pool          │ │
│  │ (KEDA queue-depth scale) │            │ 0.3× capacity             │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ Meilisearch primary      │            │ Meilisearch warm (replays │ │
│  │ (rebuilt from Postgres)  │            │ post-stream events)       │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│  ┌──────────────────────────┐            ┌──────────────────────────┐ │
│  │ DRM key-server (Widevine │            │ DRM key-server warm        │ │
│  │ + FairPlay + PlayReady)  │            │                           │ │
│  └──────────────────────────┘            └──────────────────────────┘ │
│                                                                       │
│  Global Traffic Manager (per-pack DNS):                               │
│  - Health check on primary's gateway + Postgres write path            │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                     │
│                                                                       │
│  CDN (Cloudflare R2 + Workers):                                       │
│  - Per-pack edge POP; serves from in-pack S3 origin                   │
│  - On primary-region failure: edge auto-serves from DR-pair S3 replica │
│                                                                       │
│  ActivityPub federation gateway (metadata-only):                      │
│  - Egress only from Primary (Professional-tier only)                  │
│  - Inbox accepted by either region (peer allowlist enforced both)     │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region |
|---|---|---|---|
| Postgres (metadata + claims + ages + parental + analytics) | Logical replication; async to DR-pair | ≤ 5s | within-pack DR-pair only |
| Valkey (feed cache + watch + counters) | Cluster replicas + cross-AZ; DR-pair rebuild from Postgres | ≤ 60s | within-pack DR-pair only |
| S3 (video blobs + transcode variants + thumbnails + captions) | Cross-region replication (CRR) | ≤ 15min | within-pack DR-pair only |
| Meilisearch | Rebuilt from Postgres in DR-pair | depends on rebuild SLA | within-pack DR-pair only |
| DRM key system | Active-active HSM cluster per pack | 0 | within-pack DR-pair only |
| Audit-chain seals | Cross-pack-replicable (no PII) | – | YES |
| CDN cache | Edge-distributed naturally | – | YES (metadata-only) |

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres logical replication: within-pack only.
- Valkey cluster replication: within-pack only.
- S3 cross-region replication: within-pack only.
- Meilisearch index replication: within-pack only.
- ffmpeg transcode jobs: within-pack only.
- DRM key system: within-pack only.

### Allowed exceptions

- DR-pair packs replicate primary → warm-standby within the pack.
- Audit-chain seals are cross-pack-replicable because they contain no PII (just commit-hash + signature).
- ActivityPub federation egress (Professional-tier only, metadata-only) crosses pack borders to external peers under per-tenant opt-in + SCC; **video blob never crosses pack boundary** — federation is metadata + signed CDN URL pointing to source-pack CDN POP.
- CDN edge POPs serve from in-pack S3 origin; cross-POP cache propagation is permitted (metadata-only).

### Disallowed everywhere

- Search index sharing across packs (rebuilt per-pack).
- Mention-resolution caches sharing across packs.
- Fingerprint-corpus sharing across packs unless explicitly licensed cross-pack (rare; ops-legal sign-off).
- DRM per-content keys sharing across packs.
- Age-attestation table sharing across packs.

## RPO + RTO Targets

| Component | RPO | RTO |
|---|---|---|
| Video metadata (Postgres) | ≤ 5s | ≤ 5 min failover; ≤ 30 min full recovery |
| Video blobs (S3 originals) | ≤ 15 min | provider-dependent (≤ 1h DR pair; ≤ 4h single-region) |
| Transcode variants (S3 + CDN) | regeneratable from originals | ≤ 30 min rebuild |
| Feed cache (Valkey) | regeneratable from Postgres | ≤ 15 min rebuild |
| Audit-chain seals | 0 (synchronous within-pack) | 0 |
| DRM key system | active-active | 0 |
| Search index (Meilisearch) | rebuilt from Postgres | ≤ 1h rebuild |

## Failover Procedures

### Postgres primary failure → failover to in-pack DR-pair

1. Detection: `shorts_video_metadata_primary_alive == 0` for ≥ 1min.
2. Operator-confirmed failover via runbook `cell/runbooks/postgres-primary-failover.md` (in cell µservice).
3. DNS GTM points gateway → DR-pair Postgres logical-replica (promoted to primary).
4. Write-block window ≤ 5min; reads continue from replicas.
5. Audit-chain replay verifies no missing seals during write-block window.
6. Postmortem within 5 business days.

### Pack-wide DR failover (rare; pack-wide outage)

1. Detection: pack-primary unreachable for ≥ 15 min via two independent checks.
2. Decision: council-architecture chair + ops-sre-reliability lead + ExecSponsor on Sev-1 conference call.
3. DNS failover to DR-pair region (≤ 60s TTL).
4. CDN edge auto-serves from DR-pair S3 replica.
5. Resume operations on DR-pair primary; degraded for ≤ 4h until full capacity.
6. Pen-test + postmortem; cross-region replication audit.

### CDN POP failure → degrade-to-origin

1. Detection: CDN POP unavailability via Cloudflare health-check.
2. Cloudflare auto-routes to nearest healthy POP.
3. If all in-pack POPs down: serve from S3 origin (degraded latency but functional).
4. Sev-2 if sustained > 30 min.

## Cross-Region Story

- M03 launch: single pack-kr region (OCI ap-seoul-1).
- Post-M03 expansion: pack-eu + pack-us + DR pairs; cross-pack replication forbidden (per data-residency.md); per-pack independent capacity.
- Federation egress (Professional-tier only, metadata-only) opt-in; doesn't add to in-pack capacity since it routes through `federation-gateway` workers attributed separately.

## Drills

| Drill | Cadence | Last drill |
|---|---|---|
| Postgres primary failover | Quarterly | 2026-Q3 (scheduled) |
| Pack-wide DR failover | Annually (DR-pair packs) | 2026-Q4 (scheduled) |
| CDN cache invalidation cascade | Quarterly | 2026-Q3 (scheduled) |
| Transcode queue backup | Quarterly | 2026-Q3 (scheduled) |
| Copyright-claim storm tabletop | Quarterly | 2026-Q3 (scheduled) |
| DRM key rotation | Per rotation cadence (90d) | rolling |
| Moderation classifier rollback | Quarterly | 2026-Q3 (scheduled) |

## References

- `microservices/shorts/policy/data-residency.md`.
- `microservices/shorts/capacity-model.md`.
- `microservices/shorts/cost-budget.md`.
- `microservices/shorts/failure-modes.md`.
- `microservices/observability/multi-region.md` (shape reference).
- `microservices/social/multi-region.md` (sibling reference).
- ADR-0117 (single-cloud-substrate; primary OCI).
- ADR-0135 (parallel; dual-context).
- ADR-0139 (agentic SLO-gated promotion).
- ADR-0131 (per-microservice flat layout).
- ADR-SHORTS-0001 (video-transcode pipeline; CDN choice).
- OCI multi-region docs `docs.oracle.com/iaas/Content/Cloud-Adoption-Framework`.
- Cloudflare R2 + Workers multi-region.
