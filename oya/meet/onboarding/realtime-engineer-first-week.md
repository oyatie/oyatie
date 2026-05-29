---
doc_class: Onboarding
microservice: meet
persona: realtime-engineer + webrtc-platform-engineer
related_adrs: [ADR-0316, ADR-0131, ADR-0251, ADR-0254]
date: 2026-05-20
doc_status: published
---

# Realtime Engineer onboarding — first 5 working days

Audience: a new realtime engineer or WebRTC platform engineer joining the `meet` rotation. By Day-5 they will have: provisioned a room, walked an SFU failover drill, debugged a high-packet-loss participant, shadowed a transcription accuracy review, and walked a recording retention drill.

## Day 1 — Tour the substrate

1. Read `PRD.md` § Tenant Outcomes 1-5 + `decisions/ADR-MEET-0001-sfu-vs-mcu.md` + `decisions/ADR-MEET-0002-recording-substrate.md` + `decisions/ADR-MEET-0003-transcription-vendor-selection.md`. Skim W3C WebRTC + W3C WebRTC-SVC drafts.
2. Open the Grafana folder `meet`. Identify boards: `meet-join-latency`, `meet-rtp-loss`, `meet-jitter`, `meet-mos-score`, `meet-sfu-cpu-pct`, `meet-recording-lag`, `meet-transcription-accuracy`, `meet-translation-latency`.
3. Walk the runbook index. On-call runbooks: `sfu-overloaded.md`, `participant-packet-loss-spike.md`, `recording-failure.md`, `transcription-stall.md`, `breakout-room-split-error.md`, `cross-region-failover.md`, `co-host-permit-stuck.md`.
4. Sit in on Wednesday's meet handoff.

Acceptance: you can sketch the join path: client → signalling → ICE candidate exchange → DTLS-SRTP handshake → SFU forwarding setup → first-frame delivery.

## Day 2 — Provision a room + join + first-media

```sh
oya meet room create \
    --tenant drill-acme \
    --name design-review-2026-05-20 \
    --capacity 50 \
    --recording-enabled true \
    --transcription-enabled true \
    --transcription-language en-US \
    --pack-overlay public
```

Output:

```
[room] room-id=rm-7f3a9b2c
[join-url] https://meet.drill-syd-1.oyatie.local/rm-7f3a9b2c
[recording-enabled] true
[transcription-enabled] true
[co-host-permits] []
```

Join + watch first-media:

```sh
oya meet room join \
    --room rm-7f3a9b2c \
    --participant drill-modeller-a \
    --media audio,video \
    --measure-join-latency
```

Expected output:

```
[ICE] candidate-gathering complete in 240 ms
[DTLS] handshake complete in 380 ms
[SRTP] forwarding established
[first-media] audio at 420 ms; video at 560 ms; total join-to-first-media: 980 ms
```

Acceptance: room joined; first-media within ≤ 1.4 s p99 budget; you can read the join-stage timing.

## Day 3 — SFU failover drill

Read `runbooks/cross-region-failover.md` + `decisions/ADR-MEET-0004-sfu-failover.md`.

Force a failover:

```sh
oya meet drill sfu-failover \
    --tenant drill-acme \
    --room rm-failover-drill \
    --participants 20 \
    --primary-sfu sfu-syd-1-az-a \
    --target-sfu sfu-syd-1-az-b
```

The drill:

1. Provisions a 20-participant room on `sfu-syd-1-az-a`.
2. After 60 s, marks `sfu-syd-1-az-a` as offline.
3. Watches for failover within RTO ≤ 30 s.

Expected sequence (visible in `oya meet events tail --room rm-failover-drill`):

1. `sfu_health_degraded` at t=60s.
2. `sfu_failover_started` at t=60.5s.
3. `sfu_signalling_reconnect` × 20 (each participant reconnects to new SFU) at t=61-63s.
4. `sfu_failover_completed` at t=66s.

Each participant sees a 3-6 s gap in audio/video but no full disconnect.

Acceptance: drill completed; RTO confirmed; no orphaned recordings.

## Day 4 — Debug a high-packet-loss participant

A participant reports: "audio is garbled; video freezes every 5 seconds."

```sh
oya meet participant inspect --room rm-test --participant drill-stress
```

Expected output:

```
[network]
- ICE selected pair: srflx ↔ host (NAT traversal)
- Reported RTT: 280 ms (high; typical < 80 ms)
- Reported jitter: 45 ms (high; typical < 15 ms)
- Packet loss (inbound): 6.2 % (high; typical < 1 %)
- Packet loss (outbound): 0.4 % (low; OK)
- DCSP marking: best-effort (not EF)
[media]
- Sending: 720p VP9 at 1.4 Mbit/s
- Receiving: 720p VP9 (downscaled by SFU from 1080p)
- Codec switches: 3 in last 5m (high; codec instability)
```

Diagnosis: high inbound loss + jitter = the participant's downlink is congested. Recommendation:

1. Tell participant to reduce other LAN traffic (or use wired vs WiFi).
2. SFU can downgrade their feed: switch from 1080p to 480p or 360p simulcast layer.

```sh
oya meet participant downgrade \
    --room rm-test \
    --participant drill-stress \
    --simulcast-layer low
```

Expected: jitter + loss recover within ~ 10 s.

Acceptance: triage path walked; runbook reviewed; you can articulate the simulcast layer model.

## Day 5 — Transcription accuracy + recording retention drill

Read `decisions/ADR-MEET-0003-transcription-vendor-selection.md` + `runbooks/transcription-stall.md`.

Pull recent transcription accuracy:

```sh
oya meet transcription accuracy --tenant drill-acme --window 24h
```

Expected output by language:

```
en-US: 94.3 % (WER, measured on labelled test set)
ko-KR: 91.2 % (Whisper Large v3 + tenant glossary)
es-ES: 92.8 %
fr-FR: 92.1 %
ja-JP: 89.5 % (some loss on technical jargon)
zh-CN: 88.2 % (homophones + dialect detection)
```

Audit a low-accuracy session:

```sh
oya meet transcription audit \
    --tenant drill-acme \
    --session-id sess-low-accuracy \
    --include-audio-snippets
```

The audit shows the transcript vs ground-truth; identifies error types (dictation, mis-segmentation, named-entity drops).

For recording retention drill:

```sh
oya meet recording retention drill \
    --tenant drill-acme \
    --pack pack-us-healthcare \
    --simulated-age 6y
```

The drill confirms: PHI-class recordings retained 7 years per HIPAA; non-PHI retained 30 days; non-tenant participants' recordings retained 1 year per BAA.

Acceptance: transcription audit walked; retention drill verified.

## What you've learned

- The WebRTC join path + DTLS-SRTP setup.
- The SFU failover + RTO budget.
- The participant-network-debug + simulcast downgrade.
- The transcription accuracy review per language.
- The recording retention pack-bound policy.

Next week: video-quality benchmarks (MOS / VMAF), provider-credential BYOK transcription rollout (ADR-0255 §D-4), AV1 + SVC migration.
