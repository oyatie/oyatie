---
doc_class: Runbook
title: Channel ACL drift recovery
microservice: messenger
severity: "Sev-1 (security risk)"
status: Accepted
owner_team: ops-security + axis-messenger
date: 2026-05-17
related_artifacts:
  - microservices/messenger/failure-modes.md (FM-05)
  - microservices/messenger/threat-model.md (T-T-04)
  - comms/messenger/policy/tenant-scope.cedar
doc_status: published
---

# Runbook: Channel ACL drift (FM-05)

## Trigger

- `messenger_acl_drift_total` > 0 (periodic drift detector compares Postgres ACL vs audit-chain authoritative replay).
- Manual reconciliation reveals member-list mismatch.
- Replication conflict observed.

## Severity

Sev-1 (security / privacy risk). Engage ops-security immediately.

## Immediate Mitigation (≤ 30 min per channel)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-sec-<id>`; engage ops-security + council-privacy if exposure suspected | immediate |
| 2 | Quarantine affected channel: gateway tags channel as `quarantined`; reads/writes blocked except ops-security under JIT | ≤ 5 min |
| 3 | Snapshot current Postgres ACL state to immutable evidence store | ≤ 5 min |
| 4 | Run audit-chain replay: re-derive authoritative ACL from `ChannelMemberGrantedRevoked` events for the channel | ≤ 15 min |
| 5 | Diff current vs authoritative: identify drift rows | ≤ 5 min |
| 6 | If over-permitted: enumerate reads-during-drift-window via Postgres audit log; if any reads of members not in authoritative list → BREACH path | ≤ 15 min |
| 7 | Apply authoritative ACL to Postgres; verify | ≤ 5 min |
| 8 | Unquarantine channel | ≤ 2 min |

## Breach-Path Activation

If reads-during-drift exposed messages to non-authoritative members:

- Confirmed breach: PrivacyLead engages; GDPR Art. 33 / KR PIPA Art. 34 / HIPAA §164.412 clocks may start.
- Notify affected tenant + impacted channel members.
- Forensic preservation: snapshot all related Postgres rows + audit-chain seals.

## Root Cause Analysis

| Hypothesis | Investigation |
|---|---|
| Direct Postgres mutation (bypassing service) | Postgres audit log; correlate with OpenBao JIT elevations |
| Replication conflict | Logical-replication slot status; primary→replica drift |
| Backup-restore inconsistency | Recent restore events; verify restore points are post-event-stream |
| Bug in channel-store-usecase | Code review of recent merges touching channel-store-* |

## Prevention

- Replication slot monitoring (drift > 30s → page).
- Postgres audit log forwarded to audit-chain µservice; cross-correlated weekly.
- LEAN lane `oya-check-acl-drift-coverage` asserts every ACL write path emits the audit event.

## Recovery Verification

- `messenger_acl_drift_total` rate = 0 for ≥ 24h.
- Audit-chain replay matches Postgres ACL for affected channel.
- No active alerts on channel boundary integrity.

## Postmortem

- Sev-1 postmortem within 5 business days.
- council-privacy + ops-security sign-off.
- If pattern (≥ 2 in 90d): redesign ACL durability story.

## References

- `microservices/messenger/failure-modes.md` FM-05.
- `microservices/messenger/threat-model.md` T-T-04.
- `comms/messenger/policy/tenant-scope.cedar`.
- `microservices/messenger/incident-response.md` (breach-suspect path).
