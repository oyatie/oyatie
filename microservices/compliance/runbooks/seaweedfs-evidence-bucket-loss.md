# Runbook — SeaweedFS evidence bucket loss (Sev-2 → Sev-1 if cold tier also lost)

## Trigger

SeaweedFS hot bucket unreachable OR data-loss event reported.

## Immediate actions

1. Ack page.
2. **Stop new emission** (collector → in-memory buffer; 24-hour persistence).
3. Verify cold tier integrity (3-way replication; off-site backup).
4. If cold tier intact: run IP-012 replay.
5. If cold tier also lost: Sev-1 escalation; full DR.

## Cross-references

- IP-006 — SeaweedFS storage.
- IP-012 — evidence replay.
- ADR-0180 — DR portfolio.
