---
doc_class: Runbook
title: Transcode pipeline failure (ffmpeg / gVisor sandbox)
microservice: recordings
severity: "Sev-2 (encode failure rate > 5 %)"
status: Accepted
owner_team: axis-recordings + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/recordings/decisions/ADR-RECORDINGS-0004-playback-and-cdn-strategy.md
  - microservices/recordings/capacity-model.md
doc_status: published
---

# Runbook: Transcode pipeline failure

## Purpose

Recover from ffmpeg 7.x transcode failures inside the gVisor sandbox — most
likely cause is malformed source media (CVE bait) or transient gVisor pool
exhaustion.

## Symptoms

- `recordings.transcode.failure-rate` > 5 % over 5 min.
- HLS multi-bitrate ladder incomplete for recent ingests (playback works
  for the source bitrate but not for adaptive rungs).
- Thumbnail-pack generation lagging.

## Diagnosis

1. Identify failing recordings + their source format (codec / container).
2. Check gVisor sandbox pool health.
3. Check ffmpeg-log for repeated codec / demuxer / muxer errors.
4. Cross-reference with upstream ffmpeg CVE list (quarterly review).

## Procedure

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page axis-recordings + ops-sre | on-call | immediate |
| 2 | Quarantine the failing source recordings; mark with `transcode_quarantine=true` | server | automatic |
| 3 | If a single codec is causing failures: disable that codec's input demuxer in the sandbox config + alert ops-security | ops-sre | ≤ 15 min |
| 4 | Scale ffmpeg pool ×2 if pool exhaustion | ops-sre | ≤ 5 min |
| 5 | If a gVisor CVE is suspected: pin to last-known-safe gVisor; verify no sandbox escape | ops-security | ≤ 1h |
| 6 | For quarantined recordings: source bitrate still plays directly from S3-hot via degraded playback path (no adaptive ladder); thumbnail-pack generation scheduled-for-distinct-tracked-work | axis-recordings | automatic |
| 7 | Once root-cause identified: re-encode the quarantined recordings with the corrected sandbox + emit `TranscodeRestored` | server | scheduled |

## Verification

- `recordings.transcode.failure-rate` < 0.5 % sustained.
- HLS ladder rungs reach 100 % completion for new ingests within ingest-SLO
  budget.

## Postmortem Triggers

- Any ffmpeg CVE engagement.
- Any gVisor sandbox escape (Sev-1).
- Any tenant impacted > 4h.

## Preventive Controls

- ffmpeg 7.x pinned per ADR-RECORDINGS-0004; quarterly CVE-freshness CI
  lane.
- gVisor sandbox restart cadence (4h).
- ffmpeg demuxer/muxer allowlist per codec; refuse exotic formats at ingest.

## References

- ADR-RECORDINGS-0004.
- `runbooks/playback-cdn-cache-cascade.md`.
- ffmpeg 7.x upstream changelog.
- gVisor security advisories.
