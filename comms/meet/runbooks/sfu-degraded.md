---
doc_class: Runbook
title: LiveKit SFU degraded — voice/video quality drop
microservice: meet
severity: "Sev-2 (degradation) / Sev-1 (sustained > 15 min)"
status: Accepted
owner_team: ops-sre-reliability + axis-meet
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/meet/failure-modes.md (FM-01)
  - comms/meet/dashboards/meeting-quality-mos.json
  - microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml
  - microservices/meet/slos/participant-join-latency.openslo.yaml
doc_status: published
---

# Runbook: LiveKit SFU degraded (meet)

## Trigger

Any of:
- `oya_meet_media_packet_loss_pct` p95 > 3 % for ≥ 5 min.
- `oya_meet_media_mos` mean < 3.5 for ≥ 5 min (ITU-T G.107 E-model).
- LiveKit SFU pod CPU > 90 % sustained.
- `oya_meet_participant_join_seconds` p95 > 2.0 s for ≥ 5 min.
- `oya_meet_media_glass_to_glass_seconds` p95 > 0.25 s for ≥ 5 min (intra-region) or > 0.4 s (inter-region).

## Severity

Sev-2 default; Sev-1 if sustained > 15 min OR if a pack-us-financial / pack-us-healthcare recording session is degrading (compliance-grade recording at risk).

## Immediate Mitigation (≤ 10 min)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect `dashboards/meeting-quality-mos.json`: pinpoint pack + tenant + pod + active room count | ≤ 2 min |
| 2 | If single pod hot: cordon + drain; LiveKit room-migration moves active rooms to other pods within 30s | ≤ 5 min |
| 3 | If region-wide: enable degraded-mode fallback — disable simulcast (force single-layer); downgrade default video resolution to 360p | ≤ 5 min |
| 4 | If TURN saturated: see `runbooks/coturn-key-rotation.md` for capacity expansion | ≤ 5 min |
| 5 | If upstream LiveKit CVE: pin to known-good version; emergency Helm rollback | ≤ 10 min |
| 6 | Active-meeting host notification banner: "Service degradation detected; please rejoin if you experience persistent issues" | ≤ 2 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Symmetric-NAT cascade | ICE relay-candidate selection > 30 % | check carrier outage news; coturn capacity |
| SFU pod resource starvation | LiveKit CPU > 90 %; mem at limit | HPA scale-up; verify resource limits |
| Codec mismatch | high VP8↔AV1↔H264 transcoding ratio | review tenant client mix; codec negotiation |
| Inter-region SFU mesh saturation (cross-pack attendance) | unusual inter-region traffic | verify inter-region link health |
| Bad LiveKit upstream release | latency regressed after deploy | helm rollback to prior pinned LTS |
| GPU node starvation cascading from transcription | Whisper pool depth > 5 | downgrade Whisper-large → Whisper-medium |

## Recovery Verification

- p95 packet-loss back to < 1 % for ≥ 15 min.
- Mean MOS ≥ 4.0 for ≥ 15 min.
- p95 participant-join ≤ 1.5 s sustained.
- p95 media glass-to-glass ≤ 150 ms intra-region sustained.
- No active Alertmanager alerts on meet media path.

## Postmortem Triggers

- Root-cause identified within 5 business days.
- If LiveKit upstream bug: file issue + pin avoided version.
- If capacity insufficient: revisit `capacity-model.md` sizing.
- If pack-us-healthcare or pack-us-financial recording lost: legal + DPO + supervisor notification per `incident-response.md`.

## References

- LiveKit ops docs `docs.livekit.io/realtime/server/`.
- coturn ops `github.com/coturn/coturn/wiki`.
- ITU-T G.107 (E-model for MOS computation).
- ITU-T Y.1541 (IPTV class).
- `comms/meet/dashboards/meeting-quality-mos.json`.
- `comms/messenger/runbooks/huddle-sfu-degraded.md` (sibling pattern).
