---
doc_class: Runbook
title: Real-time caption translation stream — stall / latency budget breach
microservice: translate
severity: "Sev-2 (stream stall — partial users) / Sev-1 (all caption streams stalled)"
status: Accepted
owner_team: axis-translate + ops-sre-reliability + axis-meet
date: 2026-05-18
related_artifacts:
  - microservices/translate/failure-modes.md (FM-30..FM-34)
  - microservices/translate/decisions/ADR-TRANSLATE-0006-real-time-translation-stream-architecture.md
  - microservices/translate/dashboards/real-time-stream-quality.json
  - microservices/translate/slos/real-time-caption-latency.openslo.yaml
doc_status: published
---

# Runbook: Real-time caption translation stream — stall

## Trigger

Any of:

- FM-30 (caption-stream p99 latency > 400 ms for ≥ 5 min).
- FM-31 (sentence-piece chunker stall; backpressure builds in `stream-gateway`).
- FM-32 (correction-replay window exceeded; corrections drop on the floor per ADR-TRANSLATE-0006 §"correction-replay").
- FM-33 (Whisper STT input from `meet` regresses; translate sees gap in chunk stream).
- FM-34 (foundry-runtime in-house streaming endpoint cold-start cascade).
- Tenant escalation: meet participants report "captions frozen for ≥ 10 s".

## Severity

| Symptom | Severity |
|---|---|
| Single tenant stream stalled | Sev-2 |
| Multi-tenant single-pack streams stalled | Sev-1 |
| All packs all streams stalled | Sev-1 (P0) |
| Latency budget breach < 5 min recoverable | Sev-3 |

## Symptoms

- `oya_translate_caption_stream_latency_ms{quantile="0.99"} > 400` for ≥ 5 min.
- `oya_translate_stream_gateway_queue_depth{pod=...} > 10_000` (frames queued).
- `oya_translate_stream_correction_replay_drops_total` rate increase.
- `oya_translate_stream_session_active_total` drops without corresponding session-close events (sessions hung).
- meet PRD dashboards show caption-translate egress dropping.

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm trigger via `dashboards/real-time-stream-quality.json` | ≤ 2 min |
| 2 | Check upstream Whisper STT availability via meet's dashboard (`oya_meet_stt_caption_egress_rate`) | ≤ 3 min |
| 3 | Check in-house translate streaming endpoint: `kubectl get pods -n translate -l app.kubernetes.io/component=real-time-stream-gateway` | ≤ 2 min |
| 4 | If gateway pods CPU-saturated: scale `cargo run -p oya-dev-cli -- translate scale-gateway --replicas 12` | ≤ 3 min |
| 5 | If foundry-runtime cold-start: scale `cargo run -p oya-dev-cli -- foundry-runtime scale --capability translate-stream-v1 --replicas 16` | ≤ 5 min |
| 6 | If sentence-piece chunker stalled: restart deployment `kubectl rollout restart deploy/translate-real-time-stream-gateway -n translate` | ≤ 5 min |
| 7 | Notify affected tenants via status page if Sev-1 | ≤ 15 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Upstream STT gap | meet `caption_egress_rate` drops simultaneously | engage axis-meet runbook |
| Gateway pod backpressure | `frame_queue_depth` > 10k; CPU 90 %+ | scale + restart |
| Foundry-runtime cold-start | new pod startup probe failures coinciding | scale ahead; pin replica count |
| Sentence-piece chunker bug | corrections-applied-per-chunk drops; `oya_translate_chunker_panic_total > 0` | adapter logs; rollback chunker version |
| Per-tenant runaway | one tenant ID dominates `frame_queue_depth` | per-tenant rate-limit `cargo run -p oya-dev-cli -- translate throttle-tenant --tenant <t>` |
| Network partition (multi-region) | `oya_istio_proxy_5xx_total{src_namespace="meet"}` jumps | engage cell µservice |

## Resolution Path

### Path A — Scale + drain

1. Increase gateway replicas (`replicas: 12 → 24`).
2. Increase foundry-runtime in-house streaming endpoint replicas (`16 → 32`).
3. Drain hot pods (rolling restart).
4. Resume.

### Path B — Rollback chunker version

1. Identify chunker version: `helm get values translate -n translate | grep streamGateway.image.tag`.
2. Rollback: `helm rollback translate-stream <prior> -n translate`.
3. Verify `oya_translate_chunker_panic_total == 0` for 15 min.
4. Emit `StreamChunkerRollback` audit event.

### Path C — Correction-replay window expansion (emergency)

If corrections are dropping due to insufficient replay window per ADR-TRANSLATE-0006:

1. Edit `values.yaml`: `realTimeStreamGateway.correctionReplayWindowMs: 800 → 1200`.
2. Reapply Helm chart.
3. Monitor `oya_translate_stream_correction_replay_drops_total` until zero for 30 min.
4. File ADR amendment if 1200 ms must remain.

## Verification Commands

```bash
# p99 latency back within budget
cargo run -p oya-dev-cli -- translate verify-slo \
  --slo real-time-caption-latency --window 30m
# expects: p99 < 400ms sustained

# Frame queue drained
kubectl exec -n translate deploy/translate-real-time-stream-gateway -- \
  curl -s localhost:9091/metrics | grep oya_translate_stream_gateway_queue_depth
# expects: < 1_000 per pod

# Correction drops zero
cargo run -p oya-dev-cli -- translate verify-correction-replay-no-drop \
  --window 30m

# Cross-pack consistency
cargo run -p oya-dev-cli -- translate verify-stream-slo-per-pack
```

## Rollback Path

If primary path fails:

1. Per ADR-TRANSLATE-0006 §"degraded-mode": fall back to non-streaming batch-segment translation (latency p99 ≤ 800 ms; degraded but serving).
2. Surface "captions in batch-degraded mode" banner to meet participants via meet's UI.
3. Engineering investigation continues; tenant comms.

## Post-Incident

- Postmortem within 5 business days.
- If Sev-1 P0 and accessibility-impact (deaf/hard-of-hearing users affected): WCAG 2.2 AA + ITU-T G.107 audio-quality MOS chain review; document remediation.
- If pattern recurs: gateway replica floor raised; chunker buffer-size tuned.

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-eu | accessibility-impact triggers EAA Directive (EU) 2019/882 + national DDA review |
| pack-us | ADA Title III + Section 508 accessibility-impact review |
| pack-kr | KR DDA (장애인차별금지법) Article 21 + KCC accessibility guidance |
| pack-us-healthcare | HIPAA — captions over PHI: ensure no leakage to non-resident translation endpoint |

## Named Industry Sources

- ITU-T G.107 (E-model for audio quality).
- WCAG 2.2 AA — `www.w3.org/TR/WCAG22/`.
- EAA (Directive (EU) 2019/882) — accessibility.
- ADA Title III (US) + Section 508.
- Whisper large-v3 — `github.com/openai/whisper`.
- SentencePiece — `github.com/google/sentencepiece`.

## References

- ADR-TRANSLATE-0006 (real-time translation stream architecture).
- `microservices/translate/dashboards/real-time-stream-quality.json`.
- `microservices/translate/slos/real-time-caption-latency.openslo.yaml`.
- `microservices/meet/runbooks/caption-stream-stall.md` (sibling).
