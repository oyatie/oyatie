---
doc_class: CapacityModel
template_id: TPL-CAPACITY
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-recordings
related_adrs: [ADR-0139, ADR-RECORDINGS-0001, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005]
doc_status: published
---

# Capacity Model: recordings µservice

## Demand Drivers

| Demand input | Unit | Baseline | Peak |
|---|---|---|---|
| Recordings ingested per day | recordings | 50,000 | 1,000,000 |
| Hours of media per day | h | 10,000 | 200,000 |
| Concurrent active playbacks | sessions | 5,000 | 100,000 |
| Transcription queue depth | hours pending | 1,000 | 50,000 |
| Diarization queue depth | hours pending | 1,000 | 50,000 |
| Search QPS | qps | 100 | 5,000 |
| Active legal holds | holds | 100 | 50,000 |
| Active share-links | links | 10,000 | 2,000,000 |
| eDiscovery exports per day | exports | 1 | 100 |
| Daily new transcripts | transcripts | 50,000 | 1,000,000 |

## Component Sizing

### Postgres 16 (metadata + transcript + redaction + retention + legal-hold)

| Metric | Baseline | Peak |
|---|---|---|
| Data size | 500 GB | 50 TB |
| Write IOPS | 500 | 25,000 |
| Read IOPS | 5,000 | 100,000 |
| CPU | 4 vCPU | 32 vCPU |
| Connections | 200 | 4,000 (via PgBouncer) |

Partition by `(tenant_id, year-month)` for `recording`, `(tenant_id, recording_id)` for `transcript`, `(tenant_id, recording_id)` for `redaction`.

### S3 hot tier

| Metric | Baseline | Peak |
|---|---|---|
| Stored bytes | 500 TB | 50 PB |
| PUT/s | 50 | 5,000 |
| GET/s | 500 | 100,000 |
| Average object size | 5 MB (HLS segment) | 5 MB |

### S3 cold (Glacier-class) tier

| Metric | Baseline | Peak |
|---|---|---|
| Stored bytes | 5 PB | 500 PB |
| Restore SLA | 12h (Standard) | 5 min (Expedited) |

Tiering per ADR-RECORDINGS-0005: hot → cold after 90 days of inactivity per
pack default (pack-us-financial: hot for SEC 17a-4 36mo; pack-us-healthcare:
hot for HIPAA 6y).

### Redis 7.2 (share-link + playback session)

| Metric | Baseline | Peak |
|---|---|---|
| RAM | 8 GB | 256 GB |
| Connections | 1,000 | 50,000 |
| QPS | 10,000 | 500,000 |

### Meilisearch 0.10.0 (search index)

| Metric | Baseline | Peak |
|---|---|---|
| Index size | 50 GB (transcript fragments) | 5 TB |
| Indexing QPS | 50 | 5,000 |
| Search QPS | 100 | 5,000 |
| CPU | 4 vCPU | 32 vCPU |

Sharded per `tenant_id`.

### foundry-runtime (Whisper-large + pyannote 3.x)

| Metric | Baseline | Peak |
|---|---|---|
| GPU count (Whisper) | 4 × A10 / L4 | 200 × A10 / L4 |
| GPU count (pyannote) | 2 × A10 | 100 × A10 |
| Hours/day transcribed | 10,000 | 200,000 |
| Latency budget per hour audio | ≤ 6 min Whisper-large + ≤ 3 min pyannote | same |

Per ADR-RECORDINGS-0001: Whisper-large default; falls back to Whisper-medium
under sustained queue pressure (`runbooks/transcript-pipeline-degraded-whisper.md`).

### ffmpeg 7.x in gVisor sandbox

| Metric | Baseline | Peak |
|---|---|---|
| Concurrent transcode jobs | 50 | 5,000 |
| CPU per job | 2 vCPU | 8 vCPU |
| Encode-time per minute of source | ≤ 15s (HLS multi-bitrate) | same |

### CDN (CloudFront primary)

| Metric | Baseline | Peak |
|---|---|---|
| Cache hit rate (warm) | ≥ 90 % | ≥ 80 % under storm |
| Edge throughput | 5 Gbps | 200 Gbps |

## Cost Profile (per 1k hour ingest, US-East)

| Component | Unit cost | Cost per 1k h |
|---|---|---|
| Whisper-large transcription (foundry-runtime GPU) | $0.30/h audio | $300 |
| pyannote diarization | $0.15/h audio | $150 |
| ffmpeg transcode (HLS ladder, 4 bitrates) | $0.05/h | $50 |
| S3 hot storage (5 MB segments × 1k hours ≈ 5 TB; @ $0.023/GB/mo) | — | $115/mo |
| S3 cold storage (after 90d; @ $0.004/GB/mo) | — | $20/mo |
| Meilisearch indexing | $0.02/h | $20 |
| CDN egress (CloudFront, @ $0.085/GB) | — | varies w/ playback hours |

## Scale-Out Triggers

| Trigger | Action |
|---|---|
| Postgres write IOPS > 70 % | shard by tenant; new shard cell |
| Whisper queue > 60 min | scale GPU pool ×2; activate Whisper-medium fallback |
| ffmpeg job queue > 1k | scale transcoder pool |
| CDN cache hit < 70 % | engage `runbooks/playback-cdn-cache-cascade.md` |
| Search QPS > 80 % of provisioned | add Meilisearch shard |
| Legal holds > 10k per cell | activate cell-isolation per ADR-0139 |

## References

- ADR-RECORDINGS-0001, ADR-RECORDINGS-0004, ADR-RECORDINGS-0005.
- `multi-region.md`, `cost-budget.md`, `failure-modes.md`.
