# Runbook — Evidence collector tier degraded (Sev-2)

## Trigger

HPA at max OR queue depth > 8000 (circuit-break activated) OR p99 emit lag > 5 minutes.

## Immediate actions

1. Ack page.
2. Check downstream SeaweedFS health.
3. Check per-µservice fan-in event rate; identify hot µservice.
4. Apply per-µservice rate-limit override if one µservice is dominating.

## Cross-references

- IP-011 — cross-µservice fan-in.
- capacity-model.md.
