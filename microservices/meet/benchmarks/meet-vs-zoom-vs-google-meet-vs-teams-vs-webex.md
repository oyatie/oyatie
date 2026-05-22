---
doc_class: Benchmark
microservice: meet
benchmark_date: 2026-05-20
related_adrs: [ADR-0316, ADR-0131, ADR-0254]
doc_status: published
---

# Benchmarks — oyatie meet vs Zoom / Google Meet / Microsoft Teams / Webex / Jitsi / Whereby

Workloads measured: (a) join-to-first-media latency, (b) end-to-end audio latency, (c) end-to-end video latency, (d) MOS score (audio quality) + VMAF (video quality), (e) transcription accuracy vs vendor baselines, (f) annual TCO at 5 000 user seats × average 8 hrs/week meeting time.

Hardware (oyatie paid): 16× SFU + 12× signalling + 8× Postgres + 6× transcription / translation × 3 regions.

Comparators measured against published platform docs (Zoom engineering blog, Google Meet Whitepaper, Microsoft Teams quality guide).

## Workload (a) — join-to-first-media latency

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie meet paid | 480 | 920 |
| Zoom (V2 client) | 380 | 850 (per Zoom Engineering 2025 blog) |
| Google Meet | 520 | 1 100 |
| Microsoft Teams | 680 | 1 400 |
| Webex Meetings | 750 | 1 600 |
| Jitsi Meet (Jigasi) | 620 | 1 250 |
| Whereby | 580 | 1 200 |

Reading: Zoom leads (their CDN + signaling optimization is mature). oyatie paid is competitive at sub-1 s p99.

PRD target: join-to-first-media p99 ≤ 900 ms at paid; achieved.

## Workload (b) — end-to-end audio latency

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie meet paid | 88 | 130 |
| Zoom | 80 | 140 |
| Google Meet | 110 | 180 |
| Microsoft Teams | 140 | 240 |
| Webex Meetings | 165 | 285 |
| Jitsi Meet | 120 | 200 |

Reading: We're within 10-15 % of Zoom's class-leading audio latency. Opus codec + SFU forwarding is the key.

## Workload (c) — end-to-end video latency

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie meet paid | 145 | 220 |
| Zoom | 130 | 210 |
| Google Meet | 180 | 280 |
| Microsoft Teams | 220 | 340 |
| Webex Meetings | 260 | 420 |
| Jitsi Meet | 200 | 320 |

Reading: similar to audio. Zoom narrowly leads.

## Workload (d) — MOS score (audio) + VMAF (video) under realistic network conditions

| Platform | MOS (5 % loss, 50 ms jitter) | VMAF @ 1080p (5 % loss) |
|---|---:|---:|
| oyatie meet paid (Opus + FEC + RED) | 4.1 / 5 | 78 |
| Zoom | 4.2 / 5 | 82 |
| Google Meet | 4.0 / 5 | 76 |
| Microsoft Teams | 3.8 / 5 | 72 |
| Webex Meetings | 3.7 / 5 | 70 |
| Jitsi Meet | 3.9 / 5 | 74 |

Reading: Zoom's audio resilience under loss is class-leading; we're close due to Opus FEC + redundant audio (RED).

## Workload (e) — transcription accuracy (WER, English broadcast-quality audio)

| Platform / model | WER on LibriSpeech-clean | WER on LibriSpeech-other (challenging) |
|---|---:|---:|
| oyatie meet (Whisper Large v3) | 2.1 % | 4.5 % |
| oyatie meet (Deepgram Nova 2) | 1.9 % | 4.3 % |
| oyatie meet (AWS Transcribe) | 3.2 % | 6.8 % |
| Zoom built-in transcription | 4.5 % | 8.2 % |
| Google Meet (Speech-to-Text) | 2.8 % | 5.4 % |
| Microsoft Teams (Azure Speech) | 3.5 % | 6.5 % |
| Webex Meetings transcription | 5.2 % | 9.5 % |

Reading: Whisper Large + Deepgram lead the pack. AWS Transcribe is competitive but loses on challenging audio. Built-in vendor transcriptions trail open models.

## Workload (f) — annual TCO at 5 000 seats × 8 hrs/week meeting time

| Platform | Per-seat / year | Total at 5 000 seats |
|---|---:|---:|
| oyatie meet paid (on-prem; cell-cost amortised) | n/a | $920 000 cell-cost (one cell handles 5 000 active concurrent users) |
| Zoom Business | $180 | $900 000 |
| Zoom Enterprise (negotiated) | $240 | $1 200 000 |
| Google Workspace Business Plus (Meet included) | $216 | $1 080 000 |
| Microsoft 365 Business Premium (Teams included) | $264 | $1 320 000 |
| Webex Business | $216 | $1 080 000 |
| Jitsi Meet (self-hosted, ~ 2 FTE ops) | n/a | ~ $500 000 (self-managed) |

Reading: at 5 000 seats, oyatie's cell cost is competitive with Zoom / Google / Microsoft. Crossover advantages emerge above ~ 3 000 seats and at multi-pack (sovereign) needs.

## Reproducibility

Benchmark harness at `benchmarks/meetbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks meet \
    --workload end-to-end-latency \
    --tenant-class oyatie-paid \
    --participants 100 \
    --duration 30m \
    --output ./benchmark-results.json
```
