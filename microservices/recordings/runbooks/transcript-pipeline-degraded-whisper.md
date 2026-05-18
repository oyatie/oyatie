---
doc_class: Runbook
title: Transcript pipeline degraded (Whisper-large queue overflow)
microservice: recordings
severity: "Sev-2 (queue > 60 min) escalating to Sev-1 if queue > 4h"
status: Accepted
owner_team: axis-recordings + axis-foundry-runtime + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/recordings/decisions/ADR-RECORDINGS-0001-transcription-and-diarization-pipeline.md
  - microservices/recordings/capacity-model.md
doc_status: published
---

# Runbook: Transcript pipeline degraded (Whisper-large queue overflow)

## Purpose

Restore on-time transcription when the foundry-runtime Whisper-large queue
backs up beyond SLO targets. Two fallback levers: Whisper-medium fallback +
priority-lane shedding.

## Symptoms

- `recordings.transcript-pipeline.queue-depth` > 60 min (Sev-2 page).
- `recordings.transcript-pipeline.queue-depth` > 4h (Sev-1 page).
- Per-tenant transcription latency SLO breach.

## Diagnosis

1. Check Whisper-large GPU pool utilisation: should be < 80 %.
2. Check pyannote pool: independent; rule out diarization-side issue.
3. Inspect queue distribution by tenant: identify whether one tenant is
   monopolising.
4. Check foundry-runtime gVisor pool health.

## Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page axis-recordings + axis-foundry-runtime | on-call | immediate |
| 2 | Scale Whisper-large GPU pool ×2 (HPA-bound) | ops-sre | ≤ 5 min |
| 3 | If still degraded after 10 min: activate Whisper-medium fallback for backlog ≥ 30 min old (per ADR-RECORDINGS-0001) | axis-recordings | ≤ 5 min |
| 4 | Emit `TranscriptModelDowngraded` Workflow event with reason + scope | server | automatic |
| 5 | If a tenant is monopolising: invoke per-tenant rate-limit cap | axis-recordings | ≤ 5 min |
| 6 | If still degraded after 1h: shed lowest-priority tenant tier (starter / dev cells) | ops-sre | ≤ 5 min |
| 7 | When queue back below 30 min: ramp Whisper-large back; emit `TranscriptModelRestored` | server | automatic |
| 8 | Backfill any Whisper-medium-fallback transcripts with Whisper-large (per `backfill-replay.md`) if tenant policy requires Whisper-large | axis-recordings | scheduled |

## Whisper-medium vs Whisper-large tradeoffs (per ADR-RECORDINGS-0001)

| Metric | Whisper-large | Whisper-medium |
|---|---|---|
| WER (English avg) | ~5.5 % | ~7.0 % |
| WER (KR / JP / ES / FR / DE) | ~7-9 % | ~10-13 % |
| GPU memory | 10 GB | 5 GB |
| Cost / hr audio | $0.30 | $0.18 |
| Throughput (A10) | 6 min wall-time per 60 min audio | 3 min wall-time per 60 min audio |

## Verification

- Queue depth drops below SLO target within RTO.
- Per-tenant transcript latency SLO recovers within 30 min.
- `TranscriptModelDowngraded` event audit-chained.

## Postmortem Triggers

- Any Sev-1 escalation (queue > 4h).
- Any tenant impacted > 2h on Whisper-medium.

## References

- ADR-RECORDINGS-0001.
- `slos/transcript-search-latency.openslo.yaml`.
- foundry-runtime capacity model.
