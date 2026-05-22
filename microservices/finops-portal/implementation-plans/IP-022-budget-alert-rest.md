---
ip_id: IP-022
microservice: finops-portal
bounded_context: budget-alerts
layer: rest
related_adrs: [ADR-0253, ADR-0258]
---

# IP-022 — budget-alert REST

## Goal

Tenant-admin CRUD on budgets + alert subscriptions.

## Routes

- `POST /v1/budgets`
- `GET /v1/budgets/{bid}`
- `PATCH /v1/budgets/{bid}/thresholds`
- `GET /v1/budgets/{bid}/firings`

## Acceptance

- HTTP/3 + ECH + PQC.
- Cedar gate enforced.
- Audit events sealed.
