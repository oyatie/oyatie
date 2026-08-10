---
doc_class: Runbook
title: LiveKit SFU degraded — voice/video quality drop
microservice: messenger
severity: "Sev-2 (degradation) / Sev-1 (sustained > 15 min)"
status: Accepted
owner_team: ops-sre-reliability + axis-messenger
date: 2026-05-17
related_artifacts:
  - microservices/messenger/failure-modes.md
  - comms/messenger/dashboards/voice-video-quality.json
  - microservices/messenger/slos/voice-video-call-quality.openslo.yaml
  - microservices/messenger/slos/voice-video-call-setup.openslo.yaml
doc_status: published
---

# Runbook: LiveKit SFU degraded

## Trigger

Any of:
- `oya_messenger_media_packet_loss_pct` p95 > 3 % for ≥ 5 min.
- `oya_messenger_media_mos` mean < 3.5 for ≥ 5 min.
- LiveKit SFU pod CPU > 90 % sustained.
- `oya_messenger_huddle_setup_seconds` p95 > 2.0 s for ≥ 5 min.

## Severity

Sev-2 default; Sev-1 if sustained > 15 min OR if a recording session for
pack-us-financial / pack-us-healthcare is degrading (compliance-grade
recording lost).

## Immediate Mitigation (≤ 10 min)

| Step | Action | Time |
|---|---|---|
| 1 | Inspect dashboards/voice-video-quality.json: pinpoint pack + tenant + pod | ≤ 2 min |
| 2 | If single pod hot: cordon + drain; new pods take over via LiveKit room migration | ≤ 5 min |
| 3 | If region-wide: enable degraded-mode fallback — disable simulcast, downgrade to audio-only | ≤ 5 min |
| 4 | If TURN saturated: scale coturn cluster; verify external_ip reachable | ≤ 5 min |
| 5 | If upstream LiveKit CVE: pin to known-good version; emergency Helm rollback | ≤ 10 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Symmetric-NAT cascade | ICE relay-candidate selection > 30 % | check carrier outage news; coturn capacity |
| SFU pod resource starvation | LiveKit CPU > 90 %; mem at limit | HPA scale-up; verify resource limits |
| Codec mismatch | high VP8↔H264 transcoding ratio | review tenant client mix; codec negotiation |
| Cross-pack federation loop | unusual TURN bytes outbound | verify federation seam isn't bridging unsafely |
| Bad LiveKit upstream release | latency regressed after deploy | helm rollback to prior pinned LTS |

## Recovery Verification

- p95 packet-loss back to < 1 % for ≥ 15 min.
- Mean MOS ≥ 4.0 for ≥ 15 min.
- p95 huddle setup ≤ 1.5 s sustained.
- No active Alertmanager alerts on huddles path.

## Postmortem Triggers

- Root-cause identified within 5 business days.
- If LiveKit upstream bug: file issue + pin avoided version.
- If capacity insufficient: revisit capacity-model.md huddle sizing.
- If pack-us-healthcare + recording lost: legal + DPO notification.

## References

- LiveKit ops docs `docs.livekit.io/realtime/server/`.
- coturn ops `github.com/coturn/coturn/wiki`.
- ITU-T G.107 (E-model for MOS computation).
- `comms/messenger/dashboards/voice-video-quality.json`.
