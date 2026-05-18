---
adr_id: finops-portal-005
authored: 2026-05-18
status: accepted
authority_chain: ADR-0186 observability backplane
microservice: finops-portal
---

# ADR finops-portal-005 — Grafana iframe embed via signed URL

## Context

`finops-portal` needs to expose drill-down dashboards inside the
tenant-facing UI. Three options:

1. **Re-implement** the dashboards in our own UI.
2. **Render via API** — call Mimir directly + draw with a charting
   library.
3. **Embed** the existing Grafana via iframe with signed URL.

## Decision

Embed Grafana via iframe. Signed URLs are issued by the API tier
(IP-005) on demand; TTL 5 min; HMAC key rotated quarterly.

## Rationale

1. We adopt OpenCost + Mimir as canonical data plane; re-
   implementing dashboards is wasted effort.
2. Grafana's dashboard maturity (templating, alerting, annotations)
   would take years to replicate.
3. The iframe sandbox + Cedar gate at the issuance moment is a
   sufficient security boundary; the dashboard ID + tenant_id
   variable are locked server-side.
4. Hyperscaler-bar competitive products (CloudHealth, Apptio,
   Vantage) embed dashboards similarly.

## Consequences

- Tenants experience Grafana's UI style; we accept this trade-off
  in exchange for shipping fast.
- A Grafana outage affects drill-down (but not invoice render,
  which uses postgres rollup directly).
- Signed-URL TTL is short (5 min); refresh logic in the tenant UI
  is required.

## Alternatives considered

- **Re-implement**: rejected for opportunity cost.
- **Server-side render of Grafana to PNG**: rejected because the
  interactive drill-down is the whole point.

## References

- ADR-0186 observability backplane (Mimir + Grafana).
- IP-008 Grafana-embedded dashboards.
- `dashboards/*.grafana.json`.
