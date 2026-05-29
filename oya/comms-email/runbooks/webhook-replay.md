# Runbook — Webhook replay

> ADR anchor: ADR-0201, IP-008, IP-012, ADR-0145.
> Severity: SEV-2 when DLQ depth > 10k; SEV-3 otherwise.

## When to use

- DLQ depth > threshold.
- Audit-chain availability incident requires replay after
  restoration.
- Operator confirms a class of events did not reach the audit
  chain.

## Prereqs

- Audit chain healthy.
- Schema registry healthy.
- IP-008 pipeline healthy.

## Procedure

1. Confirm DLQ depth:
   `oya-cli comms-email dlq depth`.
2. (Optional) Pause new ingestion for clean ordering:
   `oya-cli comms-email webhook-ingest pause` —
   reduces risk of reordering during replay.
3. Replay batch:
   `oya-cli comms-email dlq replay --window 2026-05-18T00:00Z..2026-05-18T01:00Z --batch-size 500`.
4. Monitor metric `comms_email_webhook_audit_emitted_total`
   for sustained increase.
5. Verify DLQ depth decreases.
6. Audit-chain entry `replay.started` + `replay.completed`
   emitted automatically.

## Validation

- DLQ depth returns to 0 (or steady-state low).
- Spot-check 5-10 sample events landed in the audit chain on
  the correct schema id.
- No new entries to the DLQ at replay rate.

## Rollback

- Replay is idempotent. If a replay batch surfaces a downstream
  problem, pause replay and engage IP-008 owner.

## Anti-patterns

- Replaying outside the canonical pipeline (raw SQL inserts
  into the audit chain).
- Skipping the audit-chain entries for the replay batch —
  compliance posture requires the trail.

## References

- IP-008 webhook delivery pipeline.
- IP-012 audit-chain emission.
- ADR-0145 audit chain.
- `backfill-replay.md`.
