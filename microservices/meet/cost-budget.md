---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-meet + ops-sre-reliability
deciders: ops-finops, axis-meet, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/meet/capacity-model.md
  - microservices/meet/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (meet µservice)

## Purpose

Track the meet µservice's monthly cloud cost across LiveKit SFU + coturn + Postgres + Redis + S3 + Whisper GPU pool + ffmpeg gVisor pool + SRS RTMP egress + Meilisearch + observability sidecars + Layer-B compute, per pack region. Surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (OKE node) | Layer-B Rust services + LiveKit SFU + coturn + SRS | `oracle.com/cloud/compute/pricing/` |
| GPU compute (A10 / L4 node) | Whisper transcription pool | `oracle.com/cloud/compute/gpu/pricing/` |
| Postgres (managed or self-hosted on PV) | Meeting + participant + recording manifest store | `oracle.com/database/pricing/` |
| Redis (managed or self-hosted) | Lobby + presence + signaling session | `oracle.com/cloud/cache/pricing/` |
| Object storage (S3-compatible) | Recordings + transcripts + summaries + quarantine | `oracle.com/cloud/storage/object-storage/pricing/` |
| Search backend (Meilisearch on PV) | Transcript search | self-hosted (PV cost) |
| Block storage (PV) | Postgres data + Meilisearch indexes + ffmpeg scratch + Whisper model weights | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | WebRTC media to clients; RTMP outbound to external streaming | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-tenant DEK envelope; recording SSE-KMS | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack ingress | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Observability sidecar | Alloy sidecar pushing to observability cluster | bundled into compute |

## Per-Component Monthly Cost (XS tier; pack-kr; M02 launch)

Per `capacity-model.md` "XS: 20 tenants, ~6k concurrent participants, 500 recordings/day".

| Component | Replicas × type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| LiveKit SFU StatefulSet | 12 × VM.Standard.E4 8-core | $870 | $200 PV scratch | $1070 |
| coturn cluster | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| meet-rest (meeting-instance-rest) | 6 × VM.Standard.E4 4-core | $435 | – | $435 |
| meet-rest (meeting-room-rest) | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| participant-rest + worker | 6 × VM.Standard.E4 2-core | $216 | – | $216 |
| recording-worker (ffmpeg gVisor) | 16 × VM.Standard.E4 4-core | $1160 | $400 PV scratch | $1560 |
| transcription-worker (Whisper streaming live-caption) | 4 × GPU A10 | $4400 (4 × $1100/mo on-demand) | $200 PV (model weights) | $4600 |
| transcription-worker (Whisper batch) | 1 × GPU L4 | $700 | – | $700 |
| webinar-rest + worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| live-stream-egress (SRS) | 1 × VM.Standard.E4 4-core | $73 | – | $73 |
| Postgres primary | 1 × VM.Standard.E4 8-core | $145 | $100 PV (1 TB) | $245 |
| Postgres replicas (2) | 2 × VM.Standard.E4 8-core | $290 | $200 PV | $490 |
| Redis cluster (3 nodes HA) | 3 × VM.Standard.E4 2-core | $108 | $10 PV | $118 |
| Meilisearch | 1 × VM.Standard.E4 4-core | $73 | $50 PV (1 TB) | $123 |
| Recording S3 bucket | – | – | $200 hot (8 TB) + $400 cold (200 TB archive) | $600 |
| Transcripts + summaries S3 | – | – | $20 hot + $30 cold | $50 |
| KMS keyring | – | $5 | – | $5 |
| Load balancer (per-pack ingress) | – | $25 | – | $25 |
| Alloy sidecars (per pod) | absorbed | – | – | $50 |
| **XS tier total per pack region** | | **~$8800** | **~$1900** | **~$10 700 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15 % for OCI rate increases + 25 % for actual-vs-forecast (recording + GPU are high-variance).

## Per-Scale-Tier Forecast

| Tier | Concurrent meetings | Concurrent participants | Recordings/day | Monthly per pack |
|---|---|---|---|---|
| XS (M02 launch; 20 tenants) | 1k | 6k | 500 | ~$11k |
| S (~100 tenants) | 5k | 30k | 5k | ~$60k |
| M (~1k tenants) | 50k | 300k | 50k | ~$500k |
| L (~10k tenants) | 500k | 3M | 500k | ~$5M |

## Per-Tenant Unit Economics

| Tier | $/active user / month | $/meeting | $/recording-min |
|---|---|---|---|
| XS | $0.020 | $0.020 | $0.0010 |
| S | $0.018 | $0.018 | $0.0008 |
| M | $0.013 | $0.013 | $0.0006 |
| L | $0.010 | $0.010 | $0.0005 |

## Cost-Optimisation Levers

| Lever | Impact | Trade-off |
|---|---|---|
| Whisper-medium for live-caption (instead of large) | -40 % GPU cost | -3 BLEU vs Whisper-large |
| HEVC re-encode on cold tier (24h+ old recordings) | -30 % storage | +CPU at re-encode time |
| Faster-whisper batched transcription | -50 % batch GPU cost | n/a; quality identical |
| LiveKit publisher simulcast off for 1:1 calls | -20 % SFU cost | -accessibility for slow links |
| Recording lifecycle tiering to OCI Archive after 30d | -60 % archive cost | retrieval delay 12h |

## Budget Breach Alerting

| Alert | Threshold | Action |
|---|---|---|
| Pack monthly burn > 110% forecast | sustained 7 days | FinOps review |
| Pack monthly burn > 130% forecast | sustained 3 days | engagement of council-architecture |
| Pack monthly burn > 150% forecast | sustained 1 day | Sev-3 incident |
| GPU pool burn > 200% forecast | sustained 1 day | switch to Whisper-medium default |
| RTMP egress bandwidth > 150% forecast | sustained 1 day | tenant FinOps notification |

CI lane `oya-check-cost-budget --microservice meet` evaluates against this matrix every 24h.

## References

- `microservices/meet/capacity-model.md`.
- `microservices/messenger/cost-budget.md` (shape reference).
- OCI pricing pages (verify at deploy).
- Whisper.cpp + faster-whisper benchmarks.
- LiveKit OSS pricing (self-hosted; bandwidth as primary cost).
