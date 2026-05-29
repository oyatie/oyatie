---
ip_id: IP-025
microservice: finops-portal
bounded_context: rightsizing-recommendations
layer: rest
related_adrs: [ADR-0253, ADR-0258]
---

# IP-025 — rightsizing REST + dashboard

## Goal

Tenant-admin reads + acts on rightsizing recommendations.

## Routes

- `GET /v1/tenants/{tid}/rightsizing/recommendations`
- `POST /v1/tenants/{tid}/rightsizing/recommendations/{rid}/apply`
- `POST /v1/tenants/{tid}/rightsizing/recommendations/{rid}/dismiss`

## Acceptance

- HTTP/3 + ECH + PQC.
- Dashboard `dashboards/rightsizing-recommendations.json` linked.
