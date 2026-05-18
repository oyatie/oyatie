---
dashboard: finops-cost-attribution
authored: 2026-05-18
canonical_authority: ADR-0174
related_specs:
  - /specs/finops-cost-attribution.json
related_adrs:
  - ADR-0174
status: dashboard-schema
owner_team: ops-finops
---

# Observability dashboard — finops-cost-attribution

## Purpose

Visualizes per-tenant + per-cost-center + per-cell + per-microservice
spend with anomaly thresholds per ADR-0174.

## Panels

### Panel 1: spend by cost-center (current quarter)

- Type: stacked bar chart.
- X axis: cost-center enum (`cc-foundry-runtime`, `cc-cloud-substrate`,
  ...).
- Y axis: $USD.
- Series: per-µservice within the cost-center.
- Source: `oya_cloud_spend_usd_total{cost_center, microservice}` counter
  aggregated to quarter.

### Panel 2: top 20 tenants by quarterly spend

- Type: table.
- Columns: tenant_id, tenant_tier, quarterly_spend_usd, headroom_vs_budget_pct,
  anomaly_class (none / spike / creep / headroom-low).
- Sorted by quarterly_spend_usd DESC.
- Source: aggregator query joining tenant table with
  `oya_cloud_spend_usd_total{tenant_id}`.

### Panel 3: anomaly heatmap (per-tenant, rolling 14 days)

- Type: heatmap.
- X axis: rolling 14-day window (per day).
- Y axis: top 50 tenants by recent spend.
- Cell color: `(daily_spend − 14-day-baseline) / MAD`.
- Threshold lines: 3·MAD (cost-spike SEV-2), 2·MAD (warn).

### Panel 4: per-cell sustainability class

- Type: stacked area chart.
- X axis: time (last 30 days).
- Y axis: $USD spend.
- Series: `sustainability_class` (`pue-gte-1-2`, `pue-1-2-to-1-1`,
  `pue-lt-1-1`).
- Source: `oya_cloud_spend_usd_total{sustainability_class}`.

### Panel 5: provider-cost-deviation alert

- Type: alert table.
- Columns: provider, capability, registered_per_invocation_cost,
  actual_per_invocation_cost, deviation_pct.
- Filter: `deviation_pct > 50%` (per ADR-0174 anomaly threshold).
- Source: cross-join foundry capability registry with foundry
  invocation cost counter.

### Panel 6: per-plane spend split

- Type: pie chart.
- Slices: `control`, `data`, `analytics` (per ADR-0004).
- Source: `oya_cloud_spend_usd_total{plane}` aggregated to current
  quarter.

### Panel 7: per-environment spend split

- Type: pie chart.
- Slices: `dev`, `staging`, `production`, `dr`.
- Source: `oya_cloud_spend_usd_total{environment}`.

## Alerts (PrometheusRule)

```yaml
groups:
  - name: finops-cost-attribution-anomaly
    rules:
      - alert: TenantCostSpike
        expr: >
          (sum by (tenant_id) (rate(oya_cloud_spend_usd_total[1h])))
          > 3 * (sum by (tenant_id) (rate(oya_cloud_spend_usd_total[14d])))
          AND sum by (tenant_id) (rate(oya_cloud_spend_usd_total[1h])) > 1000
        for: 10m
        labels:
          severity: SEV-2
        annotations:
          summary: "Tenant {{ $labels.tenant_id }} cost spike — 3*MAD over 14-day baseline AND > $1000/hr"
          runbook_url: "https://docs.oyatie.dev/runbooks/finops/cost-spike.md"

      - alert: TenantCostCreep
        expr: >
          (sum by (tenant_id) (rate(oya_cloud_spend_usd_total[24h])))
          > 1.25 * (sum by (tenant_id) (rate(oya_cloud_spend_usd_total[7d])))
        for: 24h
        labels:
          severity: SEV-3

      - alert: TenantBudgetHeadroomLow
        expr: oya_tenant_budget_remaining_pct < 0.10
        for: 5m
        labels:
          severity: SEV-3

      - alert: TenantBudgetExhausted
        expr: oya_tenant_budget_remaining_pct <= 0
        for: 1m
        labels:
          severity: SEV-2

      - alert: ProviderCostDeviation
        expr: |
          abs(oya_foundry_provider_actual_per_invocation_cost
              - oya_foundry_provider_registered_per_invocation_cost)
          / oya_foundry_provider_registered_per_invocation_cost > 0.5
        for: 30m
        labels:
          severity: SEV-2
```

## Owners

- Dashboard owner: ops-finops.
- Alert routing: ops-finops + axis-owner for the relevant cost-center.
- Quarterly review: ops-finops + council-architecture.
