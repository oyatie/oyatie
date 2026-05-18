# Runbook — Engagement-end Cedar revoke failed (Sev-1)

## Trigger

Engagement-end webhook fires; Cedar role binding revoke returns failure OR integration test detects auditor session active post-engagement.

## Immediate actions (≤ 15 minutes)

1. Ack page.
2. **Manually revoke** Cedar role binding via Cedar admin API.
3. Force-logout auditor sessions via Zitadel session revoke.
4. Audit-log all artifact reads by the auditor in the past 24 hours.

## Triage

1. Was the engagement actually over? (Confirm with engagement owner.)
2. Did the auditor read anything they shouldn't have post-engagement?

## Cross-references

- IP-007 — auditor portal.
- threat-model.md A3.
