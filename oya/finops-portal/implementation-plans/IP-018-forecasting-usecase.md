---
ip_id: IP-018
microservice: finops-portal
bounded_context: forecasting
layer: usecase
related_adrs: [ADR-0199, ADR-0255]
---

# IP-018 — forecasting usecase

## Goal

Per-tenant + per-resource-group spend forecasting. Hyperscaler precedent: AWS Cost Forecasting
+ GCP Forecasting + Vantage Forecasts.

## Crate

`oya-finops-portal-forecasting-usecase`.

## Acceptance

- 7d / 30d / 90d / 365d windows.
- P50 + P95 bands.
- Backtest accuracy ≥85% within band.
- Calls Intelligence library-first per ADR-0255.
- Audit event `ForecastEmitted`.
