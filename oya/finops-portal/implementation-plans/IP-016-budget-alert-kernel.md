---
ip_id: IP-016
microservice: finops-portal
bounded_context: budget-alerts
layer: kernel
related_adrs: [ADR-0199, ADR-0263]
---

# IP-016 — budget-alert kernel

## Goal

Stateless kernel for budget-alert evaluation. Pure-function `evaluate(budget, actual,
forecast) → alert_state`. Hyperscaler precedent: AWS Budgets + GCP Budget Alerts + Azure Cost
Management Budgets.

## Crate

`oya-finops-portal-budget-alert-kernel`.

## Acceptance

- Threshold types: absolute / percentage / forecasted.
- Property tests over edge cases (negative spend = credits applied).
- Closed enum for alert_state (NORMAL / SOFT_BREACH / HARD_BREACH / FORECAST_BREACH).
