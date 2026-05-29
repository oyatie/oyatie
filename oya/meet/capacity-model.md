---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-meet
deciders: ops-sre-reliability, axis-meet, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/meet/cost-budget.md
  - microservices/meet/multi-region.md
  - microservices/meet/policy/data-residency.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (meet µservice)

## Purpose

Sizing formulas + reference-architecture baselines for every meet component: LiveKit SFU cluster, coturn TURN cluster, Postgres meeting/recording metadata, Valkey lobby/presence/signaling, S3 recordings + transcripts, Whisper GPU transcription pool, ffmpeg recording mux pool (gVisor), SRS RTMP egress, Meilisearch transcript search, Layer-B Rust services. Drives `cost-budget.md` and `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | tenancy µservice |
| Concurrent meetings (peak) | `M_concurrent_meetings` | meeting-instance metric |
| Concurrent participants (peak) | `P_concurrent_participants` | participant metric |
| Avg participants per meeting | `P_per_meeting` | typically 6–8; webinar 100–10 000 |
| Recordings/day | `R_recordings_per_day` | recording metric |
| Avg recording duration | `T_recording_min` | ~ 35 min typical; webinars 60 min |
| Avg recording bitrate | `B_recording_kbps` | ~ 2 Mbps composite (audio 64kbps + video 1.5 Mbps + screen 400 kbps) |
| Live caption sessions concurrent | `C_caption_sessions` | live transcription metric |
| Webinar attendees (peak event) | `W_webinar_attendees` | per-event |
| RTMP egress streams concurrent | `E_egress_streams` | per-event |
| Meeting-create rate | `MC_create_per_sec` | typical 5–50 / sec / pack at peak |

## LiveKit SFU Sizing

Each LiveKit SFU pod (VM.Standard.E4 8-core + 32 GB RAM) handles approximately:
- 500 publishers (people with active video) per pod.
- 1 500 subscribers (people receiving video) per pod.
- ≤ 50 rooms per pod at typical-6-participant sizing.
- CPU-bound at simulcast transcoding; HPA scales out at 70%.

```
livekit_pods = ceil((P_concurrent_publishers / 500) + (P_concurrent_subscribers / 1500)) × 1.3 buffer
livekit_node_count = ceil(livekit_pods / pods_per_node)
```

| Tier | M_concurrent_meetings | P_concurrent_participants | LiveKit pods |
|---|---|---|---|
| XS | 1k | 6k | 12 |
| S | 5k | 30k | 60 |
| M | 50k | 300k | 600 |
| L | 500k | 3M | 6000 (multi-cluster) |

## coturn TURN Sizing

```
coturn_bandwidth_required_gbps = P_concurrent_participants × 1.5 Mbps × turn_relay_ratio (0.2 typical) / 1000
coturn_pods = ceil(coturn_bandwidth_required_gbps / 2.5) × 1.3 buffer  // 2.5 Gbps per VM
```

| Tier | P_concurrent_participants | coturn pods |
|---|---|---|
| XS | 6k | 2 |
| S | 30k | 6 |
| M | 300k | 60 |
| L | 3M | 600 (multi-cluster) |

## Postgres Meeting Store Sizing

```
meeting_rows_per_day  = MC_create_per_sec × 86400
participant_rows_per_day = meeting_rows_per_day × P_per_meeting
recording_rows_per_day = R_recordings_per_day
storage_per_day_GB    = ~ 50 bytes/meeting + ~ 200 bytes/participant + ~ 500 bytes/recording-manifest
storage_30d_hot       = storage_per_day_GB × 30 × 1.4 (index overhead)
write_iops_baseline   = MC_create_per_sec × 6 (meeting + participants + audit derivative)
```

| Tier | MC_create_per_sec | storage_30d_hot | write_iops_peak |
|---|---|---|---|
| XS | 5 | ~ 1 GB | 2k |
| S | 50 | ~ 10 GB | 20k |
| M | 500 | ~ 100 GB | 200k |
| L | 5000 | ~ 1 TB | 2M (shard across cells) |

Per-cell envelope: Postgres primary handles ≤ 5000 meeting-creates/sec at HA-RF=3.

## Valkey Sizing (lobby + presence + signaling session state)

```
valkey_ops_per_sec   = P_concurrent_participants / 5 (1 heartbeat per 5s during active session)
valkey_memory_bytes  = P_concurrent_participants × 600 (per-participant keyset)
valkey_shard_count   = ceil(valkey_ops_per_sec / 100_000)
```

| Tier | valkey_ops_per_sec | Shards | Memory |
|---|---|---|---|
| XS | ~ 1.2k | 1 | ~ 4 MB |
| S | ~ 6k | 1 | ~ 20 MB |
| M | ~ 60k | 1 | ~ 200 MB |
| L | ~ 600k | 6 | ~ 2 GB |

## S3 Recording + Transcript Sizing

```
recording_bytes_per_meeting = T_recording_min × 60 × B_recording_kbps × 1000 / 8 = T × 60 × 250 KB
                            ≈ 15 MB per minute composite (audio + video + screen)
recording_per_day_GB   = R_recordings_per_day × T_recording_min × 15 / 1024
storage_hot_30d        = recording_per_day_GB × 30
storage_cold_pack_retention = recording_per_day_GB × retention_days × 0.7 (HEVC-recompressed cold tier)
transcript_per_day_MB  = R_recordings_per_day × T_recording_min × 200 (200 KB / min as JSON)
summary_per_day_MB     = R_recordings_per_day × 5 (~ 5 KB per summary)
```

| Tier | R_recordings_per_day | T_recording_min | storage_hot_30d (recordings) |
|---|---|---|---|
| XS | 500 | 35 | ~ 8 TB |
| S | 5k | 35 | ~ 80 TB |
| M | 50k | 35 | ~ 800 TB |
| L | 500k | 35 | ~ 8 PB (multi-bucket sharded) |

## Whisper Transcription GPU Pool

Whisper-large (1.5B params) on a single A10 / L4 GPU achieves real-time-x1 for streaming + real-time-x5 for batch.

```
gpu_count_live_caption = ceil(C_caption_sessions × 1.0) // real-time-x1; one GPU per concurrent caption session
gpu_count_batch_transcribe = ceil(R_recordings_per_day × T_recording_min / (5 × 60 × 24)) // batch can lag up to 24h
```

| Tier | C_caption_sessions | Live-caption GPUs | Batch-transcribe GPUs |
|---|---|---|---|
| XS | 200 | 4 | 1 |
| S | 2k | 40 | 8 |
| M | 20k | 400 | 60 |
| L | 200k | 4000 | 600 |

GPU node selector required: `nvidia.com/gpu=true`; per-pack reserved capacity; Burst-pool capacity for live-caption spikes.

## ffmpeg Recording Mux Pool (gVisor)

```
ffmpeg_pods = ceil(R_recordings_concurrent / 8) × 1.3 buffer  // each pod handles 8 concurrent records (CPU-bound)
```

| Tier | R_recordings_concurrent | ffmpeg pods |
|---|---|---|
| XS | ~ 100 | 16 |
| S | ~ 1k | 160 |
| M | ~ 10k | 1600 |
| L | ~ 100k | 16000 (multi-cluster) |

All ffmpeg pods run under gVisor sandbox per ADR-MEET-0002.

## SRS RTMP Egress Pool

```
srs_pods = ceil(E_egress_streams / 50)  // each SRS pod handles 50 outbound RTMP streams
```

| Tier | E_egress_streams | SRS pods |
|---|---|---|
| XS | ~ 10 | 1 |
| S | ~ 100 | 2 |
| M | ~ 1k | 20 |
| L | ~ 10k | 200 |

## Meilisearch Transcript Search Sizing

```
transcript_docs_per_day  = R_recordings_per_day
index_size_bytes         = transcript_docs_per_day × 50 KB (sparse text + segments)
index_30d_hot_GB         = index_size_bytes × 30 / 1e9
indexer_workers          = ceil(transcript_docs_per_day / 5000)
```

| Tier | transcript_docs_per_day | index_30d_hot | Indexer workers |
|---|---|---|---|
| XS | 500 | ~ 750 MB | 1 |
| S | 5k | ~ 7.5 GB | 1 |
| M | 50k | ~ 75 GB | 10 |
| L | 500k | ~ 750 GB | 100 |

## Per-Tenant Limits

(Set per tenant_scope at OpenBao onboarding; enforced at Postgres + meet-rest + worker layers.)

| Limit | trial | sandbox | production | internal |
|---|---|---|---|---|
| max concurrent meetings | 5 | 5 | 1k | 10k |
| max concurrent participants per tenant | 50 | 50 | 5k | 50k |
| max participants per meeting | 50 | 50 | 1000 (interactive) | 1000 |
| max webinar broadcast attendees | 100 | 100 | 10 000 | 100 000 |
| max recordings/day | 10 | 50 | 5k | 50k |
| max recording duration (per meeting) | 30 min | 2h | 12h | 24h |
| max RTMP egress streams concurrent | 0 | 1 | 10 | 100 |
| max breakout rooms per meeting | 5 | 10 | 50 | 100 |
| recording retention max | 30 days | 90 days | 7 years | 10 years |

## Cell Scale-out Triggers

| Trigger | Action |
|---|---|
| LiveKit SFU pod CPU sustained > 70 % | HPA scale-up (≤ 6000 replicas; multi-cluster past that) |
| coturn bandwidth > 70 % provisioned | HPA scale-up |
| Whisper GPU pool depth > 5 sustained | Burst-pool scale-up; downgrade Whisper-large → Whisper-medium |
| Postgres write-IOPS > 70 % | Shard by tenant_id |
| Valkey shard CPU > 70 % | Add Valkey shard |
| Meilisearch indexer lag > 60s sustained | Add indexer worker |
| S3 PUT rate > 70 % provisioned | Sharded bucket prefix per-tenant |
| Per-tenant max meetings > 50k concurrent | Shard tenant across cells |

## Cross-Region Story

- M02 launch: single pack-kr region (OCI ap-seoul-1) + single pack-us region for pack-us-healthcare pilot.
- Post-M02 expansion: pack-eu + pack-us + DR pairs; cross-pack replication forbidden (per `policy/data-residency.md`); per-pack independent capacity.
- Cross-pack media routing (pack-eu attendee in pack-us meeting): inter-region SFU mesh; latency budget +100ms.

## References

- `microservices/meet/cost-budget.md`.
- `microservices/meet/multi-region.md`.
- `microservices/messenger/capacity-model.md` (shape reference; substrate-sharing pattern).
- Postgres tuning: PostgreSQL 16 ops docs.
- LiveKit ops: `docs.livekit.io/realtime/server/`.
- Whisper.cpp ops: `github.com/ggerganov/whisper.cpp`.
- faster-whisper: `github.com/SYSTRAN/faster-whisper`.
- Meilisearch ops: `docs.meilisearch.com`.
- Valkey Cluster ops: `valkey.io/docs/management/scaling/`.
- gVisor ops: `gvisor.dev/docs/user_guide/`.
- SRS RTMP ops: `github.com/ossrs/srs/wiki`.
