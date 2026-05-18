# SVC-ADR-004 — Tenant domain onboard flow

- Status: Accepted
- Date: 2026-05-18
- Scope: `comms-email` µservice only
- ADR anchors: ADR-0201, IP-011

## Context

Per-tenant from-domain onboarding has multiple async stages
(DNS, provider identity, warm-up). The flow must be explicit,
auditable, and operator-observable.

## Decision

- **State machine** with nine states (see IP-011 §5).
- **Stage timeouts** with explicit alerts at each stage.
- **Auto-progression** between stages when conditions are met;
  no manual approval required for the happy path.
- **Manual intervention** required only on stuck states (see
  `runbooks/per-tenant-from-domain-onboard.md`).

## Alternatives considered

- Single-stage atomic onboard: impossible — DNS + provider +
  warm-up are inherently async.
- Manual approval at every stage: rejected — too slow.

## Consequences

- New tenants reach `active` in ≤ 24h p95 (SLO
  `from-domain-onboarding-time`).
- Audit-chain captures full state machine.

## Open

- Tenant-self-service stage status dashboard — Phase 2.
