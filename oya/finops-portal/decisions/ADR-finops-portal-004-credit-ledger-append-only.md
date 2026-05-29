---
adr_id: finops-portal-004
authored: 2026-05-18
status: accepted
authority_chain: ADR-0162 audit-log integrity
microservice: finops-portal
---

# ADR finops-portal-004 — Credit ledger is append-only

## Context

The credit ledger records customer-success-applied credits,
committed-use amortization, and SLA refunds. Each entry has a
direct financial impact + an audit-chain seal. If entries are
mutable, the audit-chain seal envelope hash diverges retroactively
which violates ADR-0162 integrity.

Two design options:

1. **Mutable** — edit-in-place; simpler queries; faster.
2. **Append-only** — every change is a new entry; never mutate.

## Decision

Credit ledger is **append-only**. Reversals are by new entries
with `reverses_id = Some(prior_id)`, not by mutation. The kernel
(IP-013) enforces this at the type level.

## Rationale

1. Audit-chain seal envelope hashes are stable forever.
2. Regulator-evidence emit (IP-015) produces a verifiable record
   that doesn't change retroactively.
3. Disputes produce a clear forensic trail.
4. The integer-cents storage (i64) + ULID-based ids ensure
   determinism + uniqueness.

## Consequences

- Queries are slightly more complex (must filter out
  `reverses_id`-paired entries).
- A `view_for_period` function (IP-013) collapses entries to a net
  position, so callers don't deal with raw entries directly.
- Storage grows linearly with credit activity (acceptable).

## Alternatives considered

- **Mutable**: rejected because of audit-chain integrity violation.
- **Soft-delete (flag-only)**: rejected because re-querying a
  soft-deleted entry still produces a stale envelope hash.

## References

- ADR-0162 per-tenant audit-log slicing + integrity.
- IP-013 credit-ledger kernel.
- `runbooks/credit-application-reconciliation.md`.
