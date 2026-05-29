# IP-010 — Suppression list

> ADR anchor: ADR-0201, ADR-0145, ADR-0184.
> Owner: `oya-substrate-comms`.
> Estimate: 3 days.

## Goal

Maintain a canonical suppression list across all adapters. An
address suppressed for a tenant is rejected at preflight on every
adapter — SES, Postal, Mailgun, SMTP. No silent re-send.

## Why this IP

The suppression list is what makes ADR-0201's
"`RecipientSuppressed` at preflight" work in practice. Without a
canonical, adapter-agnostic list, each provider would maintain
its own and rotations between providers would re-deliver to
suppressed addresses.

## Pre-conditions

- ADR-0184 storage tier policy.
- IP-009 bounce / complaint handler.

## Tasks

### 1. Storage

- Postgres table `comms_email_suppressions`:
  - `tenant_id text not null`
  - `recipient text not null`
  - `reason enum (HardBounce, Complained, OperatorManual, RegulatoryOptOut, GdprErasure)`
  - `inserted_at timestamptz not null`
  - `removed_at timestamptz null`
  - `provider text` (for traceability)
  - Primary key `(tenant_id, recipient)`.

### 2. Preflight check

- Every send queries the suppression table for
  `(tenant_id, recipient)`. Hit + `removed_at IS NULL`
  triggers `EmailCommsError::RecipientSuppressed`.

### 3. Insertion

- Sources: IP-009 (bounce / complaint), operator manual
  insertion, regulatory opt-out (CAN-SPAM unsubscribe link),
  GDPR right-to-erasure request.

### 4. Removal

- Removal sets `removed_at = now()` and emits an audit chain
  entry. Removal is **always operator-initiated** — never
  automatic. Reason for removal is captured.

### 5. Performance

- Hot-path lookup p99 ≤ 5 ms. Achieved by:
  - Partial index `where removed_at is null`.
  - Process-local 5-minute cache of negative-result lookups
    (most addresses are NOT suppressed; caching the
    "not-suppressed" answer dominates).

### 6. GDPR erasure

- Right-to-erasure: when a recipient invokes Art. 17, the
  address lands in the suppression list with
  `reason = GdprErasure` and additionally the audit chain
  emits an erasure-completed event referencing the source
  request id.

### 7. Tests

- Unit tests for insertion + lookup + removal.
- Performance test asserting p99 lookup ≤ 5 ms over a 10M-row
  table.
- GDPR erasure flow test.

## Failure modes

- Postgres unavailable: comms-email µservice degrades to
  read-only (existing cache served, no new sends accepted) per
  ADR-0184 storage tier failure mode. Runbook
  `blacklist-recovery.md` covers the failover.
- Cache corruption (false-negative): hot path checks
  authoritative table on every send; cache only short-circuits
  positive non-suppressed reads.

## Acceptance criteria

- p99 preflight suppression check ≤ 5 ms over 10M-row table.
- 100% of provider hard bounces land in suppression within 5 s.
- GDPR erasure end-to-end ≤ 30 days per Art. 17 deadline.

## Rollback

The suppression list is non-negotiable — there is no rollback
path that lets the substrate continue to send to suppressed
addresses. If the table is corrupted, the µservice degrades
to read-only mode until restored.

## References

- ADR-0201.
- ADR-0184 storage tier policy.
- IP-009 bounce / complaint handler.
- GDPR Art. 17 (right to erasure).
