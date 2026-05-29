---
microservice: connector
doc_class: BackfillReplay
date: 2026-05-20
owner_team: axis-integration
status: Accepted
related_adrs: [ADR-0263, ADR-0276]
doc_status: published
---

# Backfill + Replay — connector

## Audit-chain replay

Per ADR-0263, every event in `manifest.json:audit_chain.seal_events` is Merkle-sealed. Replay:
1. Auditor or tenant admin requests evidence pack via ops-dashboard-control-center.
2. Evidence collector queries audit chain for `(tenant_id, time-window, event_class)` tuple.
3. Merkle proofs verified; tampering detection.
4. Pack exported as signed bundle (cosign keyless OIDC).

## DLQ replay

Per `runbooks/dlq-overflow.md`. TIE can replay any DLQ entry:
- Idempotency-key preserved; vendor sees same key → idempotent (no double-charge / double-message).
- Replay attempts logged as `DLQEntryReplayed` audit event.
- Mass replay (>100 entries) requires step-up auth.

## Schema-drift backfill

When a vendor schema changes:
1. Schema-drift worker detects diff during hourly poll.
2. Affected wirings flagged.
3. TIE prompted to remap (or auto-map if non-breaking).
4. Optional backfill: replay all messages from last N hours through new mapping (idempotency preserved).

## GDPR Art. 20 portability (ADR-0276)

Per-tenant backup export:
- Format: JSONL bundle of audit chain + active OAuth grant metadata (NOT raw tokens) + wiring configurations + DLQ entries.
- Encryption: cosign-signed bundle; encryption with tenant-provided public key.
- Cadence: on-demand via tenant-admin surface.
- Retention: 30d delivery window; expires per tenant policy.

## Cross-region failover replay

If a region fails and DR takes over:
- Pending webhook receives in old region are not lost (Valkey cross-region replication for idempotency-keys; PG cross-region for grants).
- In-flight connector actions: subject to circuit-breaker; replay via DLQ once region recovers.
- Audit chain: replicated to DR pair synchronously (per ADR-0263 §F).

## References

- ADR-0263 audit-event emission contract
- ADR-0276 backup portability GDPR Art. 20
