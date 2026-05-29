# SVC-ADR-002 — Suppression list policy

- Status: Accepted
- Date: 2026-05-18
- Scope: `comms-email` µservice only
- ADR anchors: ADR-0201, IP-010

## Context

Removing addresses from the suppression list undoes a
deliverability-protection decision. Allowing it casually
re-creates the conditions that put the address there.

## Decision

- **Removal is always operator-initiated** — never automatic.
- **GDPR-erasure / Regulatory-opt-out entries** can only be
  removed by substrate-authority (enforced in
  `policy/comms-email-suppression-list.cedar`).
- **HardBounce / Complained entries** can be removed by
  tenant-admin or substrate-authority — but only after an
  explicit "operator confirms recipient is valid" step in the
  CLI.
- **All removals emit audit-chain entries** with operator
  identity + reason.

## Alternatives considered

- Auto-expire after N days: rejected — re-creates the problem.
- Tenant-self-service removal without confirmation: rejected
  — too easy to mistakenly resume sending to a complaining
  recipient.

## Consequences

- Some legitimate addresses stay suppressed longer than the
  tenant might like; the trade-off favors deliverability
  reputation.
- The audit-chain has full removal provenance.

## Open

- A future addendum may add a tenant-admin "request removal"
  workflow that an operator approves; not in Phase 1.
