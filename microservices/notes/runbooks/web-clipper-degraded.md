---
doc_class: Runbook
title: Web-clipper degraded
microservice: notes
severity: "Sev-2 (token leak) / Sev-3 (latency)"
status: Accepted
owner_team: axis-notes + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/notes/threat-model.md (A-08; T-S-08, T-I-08, T-E-08)
  - microservices/notes/PRD.md (FR-07; AC-04)
doc_status: published
---

# Runbook: Web-clipper degraded

## When

Triggers:

1. `oya_notes_web_clipper_invalid_token_total > 50/min` per tenant (Sev-2 — possible token leak / replay).
2. `oya_notes_web_clipper_capture_duration_seconds_p95 > 0.5` over 10min (Sev-3 — latency regression).
3. Extension store reports surge in user-reported errors (Sev-3).
4. Chrome / Firefox / Safari / Edge upstream API change reported.

## Sev-2 — Token Leak / Replay Spike

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Page ops-security oncall | observability | t+0 |
| 2 | Identify affected installation IDs from invalid-token logs | oncall | t+10m |
| 3 | If concentrated on one install: force-rotate tokens for that user; notify user via in-product banner | oncall | t+15m |
| 4 | If diffuse across many users: force-rotate all tokens for affected pack; notify all users via in-product banner | oncall | t+30m |
| 5 | Investigate extension version + browser version distribution | oncall + axis-notes | t+60m |
| 6 | If specific extension version compromised: pull from Chrome Web Store / AMO / Apple / Edge; publish corrected version | axis-notes | within 24h |
| 7 | Audit-chain query for any successful clip in affected window | ops-security | within 24h |
| 8 | Post-mortem within 5 business days | ops-security + axis-notes | |

## Sev-3 — Capture Latency Regression

| Step | Action | Owner | Time |
|---|---|---|---|
| 1 | Acknowledge alert; check dashboard for affected pack / tenant | axis-notes oncall | t+15m |
| 2 | Inspect: is it network-side (slow upstream) or server-side (REST p99 spike)? | oncall | t+30m |
| 3 | If REST p99 spike: scale `web-clipper-bridge` REST replicas via HPA (manual nudge) | oncall | t+45m |
| 4 | If network-side: confirm CDN health; check user-side connection quality (telemetry) | oncall | t+60m |
| 5 | If sustained: consider degraded-mode banner: "clip-capture may be slow; trimmed mode active" | axis-notes | t+2h |
| 6 | Roll-forward fix or roll-back recent deploy | axis-notes | t+4h |

## Sev-3 — Upstream API Change

| Step | Action | Owner |
|---|---|---|
| 1 | Identify which browser; capture upstream change-log | axis-notes |
| 2 | Update extension manifest + republish to affected stores | axis-notes |
| 3 | Communicate via in-product banner: "please update your clipper extension" | axis-notes |
| 4 | Backwards-compat seam: old extension version still works for 30 days post-update | axis-notes |

## Degraded Modes

| Mode | Trigger | Behaviour |
|---|---|---|
| `trimmed-capture` | latency > 1s p95 for 10min | clipper sends URL + title only; full-HTML scheduled-for-distinct-tracked-work to backend fetch |
| `metadata-only` | server-side overload | URL + title + selected-text-snippet; no full HTML |
| `local-queue` | server unreachable | extension queues clip in `chrome.storage.local` (max 50 clips); replays when online |

## Failure Modes

| Failure | Recovery |
|---|---|
| Extension fails to write to `chrome.storage.local` (quota full) | extension warns user; refuses new clips until cleared |
| Extension manifest revoked by store | publish updated manifest; users must reinstall (banner via Workflow Studio shell) |
| User installs forged "oyatie" extension | report to store; banner warning users to verify publisher signature |

## Metrics

- `oya_notes_web_clipper_capture_total{result}` — captures.
- `oya_notes_web_clipper_capture_duration_seconds` — latency histogram.
- `oya_notes_web_clipper_invalid_token_total` — replay / leak proxy.
- `oya_notes_web_clipper_degraded_mode_active` — gauge (0/1).
- `oya_notes_web_clipper_install_token_rotation_total` — rotation count.

## Extension Security Posture

Per `threat-model.md` A-08:

- MV3 minimum-permission manifest (no broad `host_permissions`; `activeTab` only).
- Isolated world execution; never expose installation token via DOM.
- Per-installation token rotation 90d.
- HSM-signed extension artifact at release.
- Verified publisher on every store.

## Pack Overlays

| Pack | Notes |
|---|---|
| pack-eu | ePrivacy Art. 5(3) — installation manifest discloses what is captured at install |
| pack-kr | 정보통신망법 Art. 50-7 — disclosure pop-up at install |

## References

- `microservices/notes/threat-model.md` A-08.
- `microservices/notes/PRD.md` FR-07.
- Chrome Web Store policies.
- Mozilla AMO review policy.
- Apple Safari Web Extensions review policy.
- Microsoft Edge Add-ons policy.
