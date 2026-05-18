---
dashboard_id: finops-portal/tenant-cost-drilldown
authored: 2026-05-18
status: seed
adr_authority: ADR-0199
---

# Dashboard — Tenant cost drill-down (seed spec)

## Purpose

Tenant admins + ops-finops view per-tenant cost decomposed by
`cost-center`, `workload-class`, time. The dashboard is Grafana-embedded
and feeds from Mimir per ADR-0186.

## Panels (specified; Grafana JSON exported in follow-up IP)

1. **Headline panel** — current-month total + trend vs last month.
2. **Cost by `cost-center`** — stacked bar, daily, 30-day window.
3. **Cost by `workload-class`** — pie, current month.
4. **Cost by cell** — table; columns: cell, region, current-month
   cost, headroom %.
5. **Capability invocation cost** — per-capability cost over time.
6. **Storage cost** — object store + backup retention.
7. **Anomaly markers** — `TenantCostAnomalySpike` fires overlay.

## Filters

- `tenant_id` (required; default to current tenant via auth).
- Time window (default 30 days).
- `regulatory-pack`.

## Auth + isolation

- Tenant admins see only their own `tenant_id` (Cedar policy enforced
  upstream of Grafana).
- ops-finops sees all tenants.

## SLI ties

- The drill-down latency SLI lives in
  `slos/tenant-invoice-render-latency.openslo.yaml` (this seed).

## Follow-up

Full Grafana dashboard JSON authored as part of the
`finops-portal-ip-fanout` slice IP-008.
