# Runbook — PHI access anomaly (Sev-2)

## Trigger

Per IP-013 anomaly detector: > 100 PHI accesses to one subject in 1 hour by a single accessor.

## Immediate actions (≤ 1 hour)

1. Ack page.
2. Suspend accessor's PHI-read Cedar capability (temporary).
3. Notify accessor's manager via Slack DM.
4. Open investigation ticket.

## Triage

1. Review accessor's recent activity: legitimate workflow (e.g., chart review) OR exfiltration pattern?
2. Check if subject(s) recently changed (legitimate change-of-care triggers high access).
3. Check Cedar policy decisions — were any DENIED but accessor retried?

## Possible outcomes

- **Legitimate**: re-instate capability + adjust threshold for this accessor's baseline.
- **Compromised account**: Sev-1 escalation; full incident response.
- **Insider threat**: HR + legal engagement.

## Cross-references

- IP-004 — HIPAA min-necessary log.
- IP-013 — audit anomaly detection.
