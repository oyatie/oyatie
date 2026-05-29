# SVC-ADR-001 — DKIM rotation cadence

- Status: Accepted
- Date: 2026-05-18
- Scope: `comms-email` µservice only
- ADR anchors: ADR-0201, IP-005

## Context

ADR-0201 mandates DKIM signing on every send + key rotation
"annual or on-revocation". This service-scoped ADR pins the
cadence.

## Decision

- **Annual scheduled rotation** every 12 months from initial
  onboarding, on the first business day of the anniversary
  month. Selector format `oya{YYYYMM}`.
- **14-day overlap window** between old + new selector.
- **Emergency revocation** completes within ≤ 5 minutes from
  declaration.
- **No partial overlap shortcuts** — even when operationally
  inconvenient, the 14-day window stands.

## Alternatives considered

- Quarterly rotation: rejected — increases DNS churn without
  proportional security benefit.
- 30-day rotation: rejected — same reason.
- No rotation: rejected — ADR-0201 prohibits.

## Consequences

- IP-005 implementation honors the 12-month cadence.
- Audit-chain entries provide proof of rotation cadence to
  auditors.

## Open

- Future: rotate on a tenant-configured cadence if a
  compliance regime requires it.
