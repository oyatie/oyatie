---
ip_id: IP-023
microservice: finops-portal
bounded_context: forecasting
layer: rest
related_adrs: [ADR-0253, ADR-0263]
---

# IP-023 — forecasting REST + cache

## Goal

Surface forecasts + serve cached predictions (forecasts are expensive).

## Routes

- `GET /v1/tenants/{tid}/forecast?window=30d`
- `GET /v1/tenants/{tid}/forecast/series?from=&to=`

## Acceptance

- Cache TTL 1h on forecast results.
- Cedar gate enforced.
- HTTP/3 + ECH + PQC.
