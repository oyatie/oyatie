---
doc_class: Runbook
title: InMail-bridge to messenger degraded
microservice: network
severity: "Sev-2 (multi-tenant degradation)"
status: Accepted
owner_team: axis-network + axis-messenger
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/network/failure-modes.md (FM-22)
  - microservices/network/decisions/ADR-NET-0003-inmail-bridge-to-messenger.md
  - microservices/network/multi-region.md
doc_status: published
---

# Runbook: InMail-bridge to messenger degraded (FM-22)

## Trigger

- `network_inmail_bridge_queue_depth` > 100k OR
- `network_inmail_send_failure_rate` > 5 % sustained ≥ 2 min OR
- messenger µservice health-check failing for ≥ 1 min.

## Severity

Sev-2 default. Escalate to Sev-1 if messenger µservice is itself in Sev-1 outage (joint incident).

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify messenger µservice health: `kubectl -n messenger get pods`; check messenger µservice OpenSLO burn-rate | ≤ 2 min |
| 2 | Engage axis-messenger on-call if messenger is the root cause | ≤ 5 min |
| 3 | Hold InMail sends in Valkey Streams (Redis wire-compat) `network:inmail:bridge:queue:<tenant_id>` (worker continues batching, but does not drop messages) | ≤ 5 min |
| 4 | Surface backlog UI to senders: "InMail delivery pending — typically resolves within minutes" | ≤ 5 min |
| 5 | Apply per-tenant rate limit on new InMail sends (drop to 50% of normal) to slow queue growth | ≤ 5 min |
| 6 | Verify spam-classifier verdicts are still being emitted; do not bypass spam-check during backlog | ≤ 5 min |
| 7 | Once messenger restored: drain queue at controlled rate (1k InMails/min per tenant) to avoid thundering herd | up to 30 min queue drain |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| messenger µservice outage | messenger OpenSLO burn red; messenger gateway health-check fails | engage axis-messenger; await recovery; queue continues to buffer |
| messenger rate-limit refusing InMail-bridge | `network_inmail_send_failure_rate` includes 429 status; messenger logs show rate-limit-hit | coordinate with axis-messenger on per-µservice rate-budget; raise budget if appropriate |
| Spam-classifier mass-rejecting (FM-17 cascade) | `network_inmail_spam_classifier_reject_rate` spike | engage `runbooks/recruiter-classifier-rollback.md` §"ranker fallback" pattern; rollback spam classifier if drift detected |
| Per-tenant InMail rate-budget exhausted (legitimate burst) | one tenant dominates queue | coordinate with gtm-customer-success on temporary budget raise |
| Network partition between network and messenger clusters | `network_inmail_bridge_grpc_dial_error_rate` spike; pod-level connectivity test fails | engage cloud-k8s for NetworkPolicy / mesh debug |

## Recovery Verification

- `network_inmail_bridge_queue_depth` returns to < 10k sustained ≥ 30 min.
- `network_inmail_send_failure_rate` < 0.5 % for ≥ 30 min.
- Per-tenant InMail send latency p95 ≤ 100 ms.
- Spam-classifier verdict rate at baseline (no drift).
- No active alerts on InMail-bridge.

## Cross-Context Safety

Per ADR-NET-0003: the InMail-bridge **NEVER** sends to messenger's Personal-tier DM surface. If during recovery the messenger µservice returns a `context_kind: Personal` confirmation, the bridge worker MUST:

1. Reject the response at runtime guard (Sev-1 alarm).
2. Audit-chain seal the violation.
3. Quarantine the affected InMail; do NOT mark as delivered.
4. Engage council-privacy + ops-security per `runbooks/connection-graph-corruption.md` §FM-10.

This invariant is also enforced compile-time via `policy/professional-context-isolation.md` PCI-09.

## Postmortem Triggers

- Recurring queue backlog: review InMail-bridge worker sizing in `capacity-model.md`.
- messenger-side root cause: cross-µservice retrospective with axis-messenger.
- Spam-classifier drift: engage `runbooks/recruiter-classifier-rollback.md` postmortem pattern.

## References

- `microservices/network/failure-modes.md` FM-22.
- `microservices/network/decisions/ADR-NET-0003-inmail-bridge-to-messenger.md`.
- `microservices/network/multi-region.md` §"Cross-µservice Bridge Failure Modes".
- `microservices/network/policy/professional-context-isolation.md` Invariant PCI-09.
- `microservices/messenger/runbooks/` (sibling reference if available).
