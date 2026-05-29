# SVC-ADR-003 — Webhook retry policy

- Status: Accepted
- Date: 2026-05-18
- Scope: `comms-email` µservice only
- ADR anchors: ADR-0201, IP-008

## Context

Webhooks may fail to land in the audit chain due to chain
unavailability, schema-registry rejection, credential drift,
or transient network failure. A retry policy must balance
not-losing-events vs not-overloading-recovery.

## Decision

- **Exponential back-off**: 1s → 2s → 4s → 8s → 16s → 32s →
  64s → 128s → 256s.
- **Max 8 retries** before DLQ.
- **After DLQ**: events wait for operator-initiated replay per
  `runbooks/webhook-replay.md`.
- **DLQ depth alert** fires at > 100 entries in 5 min and at
  > 10k absolute.
- **Idempotency-fingerprint dedup** at every retry attempt.

## Alternatives considered

- Infinite retry: rejected — never converges; masks systemic
  faults.
- Single retry: rejected — too fragile against transient
  faults.
- Linear back-off: rejected — exponential is the industry
  standard for this class of work.

## Consequences

- Worst-case time-to-DLQ ≈ 8.5 min from first attempt.
- DLQ replay is the operator's tool for the long tail.

## Open

- Per-tenant configurable retry budget — deferred.
