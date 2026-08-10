---
doc_class: Runbook
title: Follow-graph corruption recovery
microservice: social
severity: "Sev-1 (security + privacy risk)"
status: Accepted
owner_team: ops-security + axis-social
date: 2026-05-17
related_artifacts:
  - microservices/social/failure-modes.md (FM-05)
  - microservices/social/threat-model.md (T-T-03)
  - microservices/social/policy/tenant-scope.cedar
doc_status: published
---

# Runbook: Follow-graph corruption (FM-05)

## Trigger

- `social_follow_graph_drift_total` > 0 (periodic drift detector compares Postgres adjacency-list vs audit-chain authoritative replay).
- Manual reconciliation reveals follower-count mismatch.
- Replication conflict observed in graph partition.
- Mass-follow / unfollow attack detected.

## Severity

Sev-1 (relationship-data integrity risk; possible privacy implication if mass-follow attack leaks who-follows-whom). Engage ops-security immediately.

## Immediate Mitigation (≤ 30 min per affected scope)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-sec-<id>`; engage ops-security + council-privacy if exposure suspected | immediate |
| 2 | Quarantine affected scope: gateway tags accounts as `quarantined`; follow / unfollow blocked except ops-security under JIT | ≤ 5 min |
| 3 | Snapshot current Postgres follow-graph state to immutable evidence store | ≤ 5 min |
| 4 | Run audit-chain replay: re-derive authoritative follow-graph from `FollowEdgeAdded` / `FollowEdgeRemoved` events for the scope | ≤ 15 min |
| 5 | Diff current vs authoritative: identify drift edges | ≤ 5 min |
| 6 | If mass-follow attack suspected: bot-detection scan via foundry-guardrails; identify offending principals | ≤ 15 min |
| 7 | Apply authoritative graph to Postgres; verify | ≤ 5 min |
| 8 | Unquarantine scope | ≤ 2 min |

## Mass-Follow / Mass-Unfollow Attack Path

If audit shows mass-follow attack pattern:

- Confirmed attack: bot accounts identified; per-account suspension; tenant-admin notified.
- Forensic preservation: snapshot related Postgres rows + audit-chain seals.
- If pattern suggests organised coordinated attack: ops-security takes over; coordinate with foundry-guardrails sybil detector.

## Privacy-Breach Path Activation

If unauthorised reads-during-drift exposed who-follows-whom relationships:

- Confirmed breach: PrivacyLead engages; GDPR Art. 33 / KR PIPA Art. 34 clocks may start.
- Notify affected accounts.

## Root Cause Analysis

| Hypothesis | Investigation |
|---|---|
| Direct Postgres mutation (bypassing service) | Postgres audit log; correlate with OpenBao JIT elevations |
| Replication conflict in graph partition | Logical-replication slot status; primary→replica drift |
| Backup-restore inconsistency | Recent restore events; verify restore points are post-event-stream |
| Bug in follow-graph-usecase | Code review of recent merges touching follow-graph-* |
| Mass-follow attack (sybil) | foundry-guardrails sybil detector signal; per-IP rate-limit logs |

## Prevention

- Replication slot monitoring (drift > 30s → page).
- Postgres audit log forwarded to audit-chain µservice; cross-correlated weekly.
- LEAN lane `oya-check-follow-graph-audit-coverage` asserts every follow / unfollow write path emits the audit event.
- Per-user follow rate limit (default 100/hr); tenant-configurable.
- foundry-guardrails sybil detector wired to follow-edge stream.

## Recovery Verification

- `social_follow_graph_drift_total` rate = 0 for ≥ 24h.
- Audit-chain replay matches Postgres follow-graph for affected scope.
- No active alerts on graph integrity.

## Postmortem

- Sev-1 postmortem within 5 business days.
- council-privacy + ops-security sign-off.
- If pattern (≥ 2 in 90d): redesign graph durability story.

## References

- `microservices/social/failure-modes.md` FM-05.
- `microservices/social/threat-model.md` T-T-03.
- `microservices/social/policy/tenant-scope.cedar`.
- `microservices/social/incident-response.md` (breach-suspect path).
- ADR-SOC-0002 (follow-graph storage rationale).
