---
doc_class: Runbook
title: Live caption stalled (Whisper GPU pool exhaustion)
microservice: meet
severity: "Sev-2"
status: Accepted
owner_team: axis-meet + axis-foundry-runtime
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/meet/failure-modes.md (FM-06)
  - comms/meet/dashboards/ai-features-quality.json
  - microservices/meet/slos/live-caption-latency.openslo.yaml
  - comms/meet/capabilities/T1-assist.yaml
doc_status: published
---

# Runbook: Live caption stalled (meet)

## Trigger

Any of:
- `oya_meet_live_caption_lag_seconds` p99 > 1.0 s for ≥ 2 min (vs SLO target ≤ 500ms p99).
- Whisper streaming GPU pool queue depth > 5 sustained.
- GPU node failure (NVIDIA driver crash; CUDA OOM).
- Whisper model load error (e.g., model file corrupt).

## Severity

Sev-2 default. Sev-1 if multi-pack outage OR pack-us-healthcare clinical-meeting captions stall (potential patient-safety impact for telemedicine).

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect `dashboards/ai-features-quality.json` panel "Whisper GPU pool depth" + "live caption lag p99" | ≤ 2 min |
| 2 | Trigger burst-pool GPU spin-up (cold-spare → warm in ≤ 5 min) | ≤ 5 min |
| 3 | If burst-pool unavailable: degrade Whisper-large → Whisper-medium for streaming (acceptable temporary quality drop) | ≤ 2 min |
| 4 | If GPUs entirely unavailable: degrade to "Captions unavailable" banner; emit `oya_meet_live_caption_degraded_total++` | ≤ 5 min |
| 5 | Surface in-meeting host banner: "Live captions are temporarily unavailable; transcripts will be generated post-meeting" | ≤ 5 min |
| 6 | Engage axis-foundry-runtime if model-level issue | ≤ 10 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| GPU pool capacity insufficient for peak | concurrent caption sessions > forecast | review `capacity-model.md` Whisper sizing; expand pool |
| GPU node hardware failure | per-node CUDA errors | drain node; cloud-iac replaces |
| Whisper model load OOM | container OOM | review per-pod resource limits; downgrade model if needed |
| Audio quality degraded upstream | LiveKit audio bitrate dropped | inspect LiveKit; bandwidth-limit issue at meet-rest |
| Locale spike (new language unsupported) | unusual `language_code=xx` distribution | engage axis-foundry-runtime to extend coverage |
| faster-whisper / CTranslate2 CUDA mismatch | CUDA-version error logs | re-pin to known-good CUDA driver |

## Degradation Cascade

1. **Tier-1 healthy**: Whisper-large streaming on A10 GPU; p99 ≤ 500ms.
2. **Tier-2 degraded**: Whisper-medium streaming; p99 ≤ 700ms; -3 BLEU vs Whisper-large.
3. **Tier-3 emergency**: Whisper-tiny streaming or skip-streaming + batch-only at meeting-end.
4. **Tier-4 unavailable**: "Captions unavailable" banner; post-meeting transcript still available via batch.

Transitions: Tier-1→2 at GPU saturation; Tier-2→3 if Whisper-medium also saturated; Tier-3→4 if no GPU available.

## Post-Meeting Transcript Backup

Even if live captions fail entirely, batch transcription (Whisper-large, GPU-batch pool) still produces a post-meeting transcript within 60s of meeting-end. Surface this in the host banner.

## Recovery Verification

- `oya_meet_live_caption_lag_seconds` p99 back to ≤ 500ms sustained for ≥ 15 min.
- Whisper pool depth < 2 sustained.
- All affected meetings have post-meeting transcripts produced.

## Postmortem Triggers

- Sustained > 30 min: postmortem within 5 business days.
- pack-us-healthcare clinical-meeting impact: clinical-team notification; potential BAA review.
- Capacity gap surfaced: revisit `capacity-model.md` Whisper sizing.

## References

- ADR-MEET-0002 (transcription pipeline).
- ADR-MEET-0006 (EU AI Act risk class).
- OpenAI Whisper paper.
- faster-whisper / CTranslate2 docs.
- `microservices/meet/threat-model.md` T-D-03.
- `comms/meet/capabilities/T1-assist.yaml`.
