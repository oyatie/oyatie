---
doc_class: Runbook
title: WebSocket gateway storm
microservice: messenger
severity: "Sev-2 (degradation) / Sev-1 (sustained > 30 min)"
status: Accepted
owner_team: ops-sre-reliability + axis-messenger
date: 2026-05-17
related_artifacts:
  - microservices/messenger/failure-modes.md (FM-01)
  - microservices/messenger/multi-region.md
  - microservices/messenger/capacity-model.md
doc_status: published
---

# Runbook: WebSocket gateway storm (FM-01)

## Trigger

`messenger_gateway_connection_attempts_per_sec` > 10× baseline for ≥ 1 min OR gateway CPU sustained > 90 %.

## Severity

Sev-2 default; escalate to Sev-1 if sustained > 30 min or if cascades to message-send failure.

## Immediate Mitigation (≤ 10 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify HPA scaling: `kubectl -n messenger get hpa websocket-gateway` (replicas climbing) | ≤ 2 min |
| 2 | Inspect connection-attempts breakdown: `messenger_gateway_connection_attempts_per_sec` by `client_version`, `client_geo` | ≤ 2 min |
| 3 | If single bad client version: enable per-version rate cap in gateway runtime config | ≤ 5 min |
| 4 | Enable per-tenant connection rate limit (tighter): tenant-scoped limit from 1k/min → 500/min globally | ≤ 5 min |
| 5 | Toggle jittered-backoff client-SDK kill-switch via tenancy config flag (signals SDK to lengthen reconnect backoff) | ≤ 5 min |
| 6 | If CPU still > 90 %: pre-warm 50% additional gateway replicas; ConfigMap-driven scale-out | ≤ 10 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Mobile-network handoff cascade | `client_geo` clustering | check carrier outage news |
| Client bug (reconnect loop) | `client_version` clustering | SDK telemetry; reproduce with synthetic client |
| Targeted attack | `source_ip` clustering + non-OIDC | WAF inspection; engage ops-security |
| Cluster maintenance trigger | preceding deployment | ArgoCD audit log |

## Recovery Verification

- Gateway CPU back to ≤ 50 % for ≥ 15 min.
- `messenger_gateway_active_connections_total` stable for ≥ 15 min.
- `messenger_message_send_p99_seconds` ≤ 0.1 sustained.
- No active Alertmanager alerts on gateway path.

## Postmortem Triggers

- Root-cause identified within 5 business days.
- If targeted attack: ops-security report + WAF rule update.
- If client SDK bug: hotfix release; communicate to tenant SDK consumers.
- If capacity insufficient: revisit `capacity-model.md` gateway sizing.

## References

- `microservices/messenger/failure-modes.md` FM-01.
- `microservices/messenger/capacity-model.md` §"WebSocket Gateway Sizing".
- `microservices/messenger/multi-region.md`.
- Envoy ratelimit docs.
