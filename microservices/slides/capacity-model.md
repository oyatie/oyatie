---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-sre-reliability
doc_status: published
---

# Capacity model — slides µservice

## Dimensions

| Axis | Unit | Baseline per cell | XL per cell | Notes |
|---|---|---|---|---|
| Active editor sessions | session | 10,000 | 200,000 | per-tenant session cap default 50 |
| Concurrent WS connections | conn | 50,000 | 500,000 | including presenter-view + audience-view + collab |
| Active broadcast sessions | session | 100 | 5,000 | one session = one broadcasting deck |
| Broadcast viewers per session | viewer | 500 baseline | 5,000 max | LiveKit SFU cascade beyond 500 |
| Total broadcast viewers (cluster) | viewer | 50,000 | 5,000,000 | bounded by messenger LiveKit capacity |
| Decks per tenant | deck | 1,000 | 100,000 | Postgres shard fill |
| Slides per deck | slide | 50 typical | 1,000 max | per-deck max enforced at API |
| Save round-trips/sec | save | 1,000 | 100,000 | Postgres write capacity |
| Cursor sync events/sec (collab) | event | 100,000 | 5,000,000 | WS fan-out |
| CRDT ops/sec (per session) | op | 100 sustained, 1k burst | — | per-session rate limit |
| Chart-live-link refreshes/sec | refresh | 1,000 | 100,000 | sheets-side throttle |
| Export jobs/sec (PPTX+PDF+MP4) | job | 10 | 200 | gVisor worker pool |
| AI T1 invocations/sec | inv | 5 | 100 | foundry-runtime throughput |
| AI T2 invocations/sec | inv | 0.1 | 5 | T2 expensive; per-tenant + per-pack quota |
| Asset uploads/sec (image+video+audio) | upload | 50 | 5,000 | ClamAV+OPSWAT throughput |
| Import jobs/sec (PPTX+ODP+Keynote) | job | 5 | 100 | gVisor worker pool |

## Resource-to-throughput mapping

### editor-rest

- p99 save latency ≤ 100ms requires Postgres write p99 ≤ 60ms + Cedar evaluation p99 ≤ 30ms.
- 4 replicas × 2 vCPU baseline handles 1,000 save/sec.
- HPA target 70% CPU; min 4 / max 50.

### real-time-collaboration-worker (WS gateway)

- p99 cursor sync ≤ 150ms requires WS RTT ≤ 50ms + CRDT merge ≤ 30ms + dispatch ≤ 70ms.
- 3 replicas × 2 vCPU × 8 GiB baseline handles 50,000 WS connections.
- HPA on WS connection count > 70% of cap.
- Lease via Redis: single-writer per deck.

### broadcast-mode-worker

- p99 signaling round-trip ≤ 250ms requires messenger LiveKit p99 ≤ 200ms + slides bridge ≤ 50ms.
- 2 replicas × 2 vCPU × 4 GiB baseline handles 100 active broadcast sessions.
- HPA on session count.
- Bridge to messenger SDK; not LiveKit-pod-owner.

### Export workers (PPTX / PDF / MP4)

- 50-slide PDF in WeasyPrint p95 ≤ 3s on 4 vCPU.
- 50-slide PPTX in OOXML serializer p95 ≤ 5s on 2 vCPU.
- 50-slide MP4 in ffmpeg p95 ≤ 55s (slide_count × 1s + 5s) on 4 vCPU.
- 4 replicas × 4 vCPU baseline handles 10 jobs/sec mixed; HPA scales to 100.
- Per-job memory budget: import 2 GiB, MP4 export 4 GiB, others 1 GiB.

### Import workers (PPTX / ODP / Keynote)

- 50-slide PPTX import in Pandoc bridge p95 ≤ 5s on 2 vCPU + gVisor.
- ClamAV scan p99 ≤ 1s; OPSWAT scan p99 ≤ 2s.

### Postgres (Citus)

- 3-node Citus cluster: 8 vCPU + 32 GiB + 1 TB SSD.
- Write capacity: 10,000 IOPS sustained; 5,000 writes/sec mixed; shard by tenant_id.
- Linear scale: each added shard adds ~3,000 writes/sec.

### Redis

- 3-node sentinel cluster: 4 vCPU + 16 GiB.
- CRDT cache + lease coordination + presence: ~50k connections × 1 KiB session = 50 MiB working set.

### S3

- Per-tenant prefix; per-pack bucket.
- Tier transitions: hot (deck snapshots last 30d), warm (90d), archive (per-pack retention).

### CDN

- WASM chunks: immutable + max TTL + SRI; global.
- Theme/template gallery: signed Ed25519 + 1h TTL + revocation list polling.
- Deck-rendered preview (public-read decks): per-tenant CDN key + 60s TTL.

## Scaling triggers (per `iac/helm/*/templates/hpa.yaml`)

| Trigger | Component | Action |
|---|---|---|
| CPU > 70% | editor-rest | scale up by 25% (min 4, max 50) |
| WS conn > 70% capacity | real-time-collaboration-worker | scale up by 33% (min 3, max 100) |
| Cursor sync p99 > 150ms | real-time-collaboration-worker | scale up + alarm |
| Save p99 > 100ms | editor-rest | scale up + DB shard add if Postgres bottleneck |
| Export queue depth > 100 | export-workers | scale up by 50% (min 4, max 100) |
| Broadcast session count > 70 | broadcast-mode-worker | scale up by 33% |
| AI T2 backpressure signal from foundry-runtime | ai-content-generation-rest | throttle + queue + tenant notify |

## Saturation alarms

| Alarm | Source | Threshold | Severity |
|---|---|---|---|
| Postgres write IOPS saturation | Postgres metrics | > 80% sustained | Sev-2 |
| Redis connection-pool saturation | Redis metrics | > 80% sustained | Sev-2 |
| WS dispatch queue depth | metrics | > 1000 | Sev-2 |
| Export queue depth | metrics | > 500 | Sev-2 |
| gVisor worker OOM rate | metrics | > 3 / 5min | Sev-1 |
| LiveKit broadcast SFU saturation | messenger metrics | > 80% | Sev-2 |
| AI T1 latency p99 | foundry-runtime metrics | > 2s | Sev-3 |
| AI T2 latency p99 | foundry-runtime metrics | > 30s | Sev-3 |

## Cost-vs-capacity trade-offs

- Per-pack right-sizing weekly; us-healthcare maintains baseline 2× to absorb burst (HIPAA SLO).
- Off-hours scale-down except us-healthcare (clinical 24×7).
- gVisor worker pool warm-pool of 2 replicas during off-hours (cold-start cost).

## References

- ADR-0139 SLO-gated promotion (capacity verified before promotion).
- `cost-budget.md`.
- `multi-region.md`.
