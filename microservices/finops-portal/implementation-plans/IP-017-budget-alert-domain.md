---
ip_id: IP-017
microservice: finops-portal
bounded_context: budget-alerts
layer: domain
related_adrs: [ADR-0199, ADR-0263]
---

# IP-017 — budget-alert domain

## Goal

Compose budget-alert kernel with notifications fan-out to comms-email +
notifications-engine.

## Crate

`oya-finops-portal-budget-alert-domain`.

## Acceptance

- Audit events: `BudgetAlertFired`, `BudgetAlertResolved`.
- Cross-region propagation via TrueTime for cross-cell tenants.
- Test coverage on alert dedup (no flap).
