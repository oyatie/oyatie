---
doc_class: Tutorial
microservice: finops-portal
persona: finops-engineer + tenant-cfo-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a per-cost-center chargeback dashboard for a tenant

You will: configure a tenant's tag-allocation policy, ingest 30 days of synthetic cost data, author a per-cost-center chargeback view, set budget alerts, exercise the forecast pipeline, and emit a FOCUS-compliant cost export. Total time ≤ 60 minutes.

## Pre-requisites

- A tenant cell on paid with per_seat billing_component tenant_class (`ADR-0330 and ADR-0331 tenant_class model`).
- `oya-dev-cli` ≥ 1.42.0.
- A tenant principal in the `finops_admin` Cedar role.
- A FOCUS export consumer ready (e.g., the tenant's Apptio/Cloudability instance, or a downstream BI tool).

## Step 1 — Configure the tag-allocation policy (≤ 10 min)

The tenant `acme-corp` has 3 cost centers. Two-thirds of their resources carry direct tags; one-third are untagged and need an allocation policy.

```sh
oya finops-portal chargeback configure \
    --tenant acme-corp \
    --policy-file ./chargeback-policy.yaml
```

The policy:

```yaml
version: 1
tenant_id: acme-corp
mode: chargeback  # vs "showback"

cost_centers:
  - id: engineering
    display_name: "Engineering R&D"
    parent: null
  - id: sales
    display_name: "Sales & GTM"
    parent: null
  - id: marketing
    display_name: "Marketing"
    parent: null
  - id: engineering-platform
    display_name: "Platform Engineering"
    parent: engineering
  - id: engineering-product
    display_name: "Product Engineering"
    parent: engineering

# How to allocate cost
direct_tag_attribution:
  enabled: true
  tag_key: cost_center  # resource tag whose value = cost_center.id
  fallback_to_allocation_policy: true

allocation_policy:
  # Applied to resources where direct_tag is missing or invalid
  compute:
    engineering-platform: 0.40
    engineering-product:  0.20
    sales:                0.25
    marketing:            0.15
  storage:
    engineering-platform: 0.60
    engineering-product:  0.30
    sales:                0.08
    marketing:            0.02
  network:
    engineering-platform: 0.50
    engineering-product:  0.25
    sales:                0.15
    marketing:            0.10
  default:  # fallback if service-category doesn't match
    engineering-platform: 0.45
    engineering-product:  0.25
    sales:                0.20
    marketing:            0.10

# Budget per cost center (monthly)
budgets:
  - cost_center_id: engineering-platform
    monthly_usd: 28000
  - cost_center_id: engineering-product
    monthly_usd: 18000
  - cost_center_id: sales
    monthly_usd: 6000
  - cost_center_id: marketing
    monthly_usd: 4000
```

Verify the policy applies:

```sh
oya finops-portal chargeback show-policy --tenant acme-corp
```

Expected: full policy printed with effective-from timestamp.

## Step 2 — Ingest 30 days of synthetic cost data (≤ 10 min)

```sh
oya synthetic finops-portal emit-cost-data \
    --tenant acme-corp \
    --since 2026-04-21 \
    --until 2026-05-20 \
    --shape multi-cost-center \
    --total-monthly-usd 56000 \
    --tagged-fraction 0.67
```

This emits ~ 30 days × ~ 8 services × ~ 5 resources × ~ 24 hours = ~ 28 800 synthetic cost events. The `--shape multi-cost-center` arg creates realistic distributions: ~67% of resources directly tagged with cost_center, ~33% untagged (will hit the allocation policy).

Verify ingest:

```sh
clickhouse-client --host prod-ch-1 --query "
    SELECT
        count() AS events,
        sum(effective_cost) AS total_cost_usd,
        sum(if(tag_cost_center != '', 1, 0)) AS tagged_count
    FROM tenant_acme_corp.cost_event
    WHERE charge_period_start >= '2026-04-21'"
```

Expected: ~ 28 800 events, total ~ $56 000, ~ 19 300 tagged (~ 67%).

## Step 3 — Author the chargeback dashboard (≤ 20 min)

```sh
oya finops-portal dashboard create \
    --tenant acme-corp \
    --dashboard-id chargeback-by-cost-center \
    --title "Monthly chargeback by cost center" \
    --visibility tenant-cfo,tenant-admin,tenant-cost-center-leads \
    --refresh-interval 15m \
    --panels-file ./chargeback-panels.yaml
```

The panels (`./chargeback-panels.yaml`):

```yaml
panels:
  - title: "Total monthly cost (last 30 d)"
    type: scalar
    query: |
      SELECT round(sum(effective_cost), 2) AS total_cost_usd
      FROM {{tenant_db}}.cost_event_allocated
      WHERE charge_period_start >= today() - INTERVAL 30 DAY

  - title: "Cost by cost center (current month)"
    type: stacked-bar
    x_axis: day
    y_axis: allocated_cost_usd
    series: cost_center_id
    query: |
      SELECT
          toYYYYMMDD(charge_period_start) AS day,
          cost_center_id,
          round(sum(allocated_cost_usd), 2) AS allocated_cost_usd
      FROM {{tenant_db}}.cost_event_allocated
      WHERE charge_period_start >= toStartOfMonth(now())
      GROUP BY day, cost_center_id
      ORDER BY day, cost_center_id

  - title: "Budget burn (current month)"
    type: gauge
    series: cost_center_id
    query: |
      WITH burned AS (
          SELECT
              cost_center_id,
              sum(allocated_cost_usd) AS burned_usd
          FROM {{tenant_db}}.cost_event_allocated
          WHERE charge_period_start >= toStartOfMonth(now())
          GROUP BY cost_center_id
      )
      SELECT
          b.cost_center_id,
          b.burned_usd,
          bm.monthly_usd AS budget_usd,
          round(b.burned_usd * 100.0 / bm.monthly_usd, 1) AS burn_pct
      FROM burned b
      JOIN {{tenant_db}}.budget_monthly bm USING (cost_center_id)

  - title: "30-day forecast (per cost center)"
    type: line
    x_axis: forecast_date
    y_axis: p50_cost_usd
    series: cost_center_id
    confidence_bands: [p10_cost_usd, p90_cost_usd]
    query: |
      SELECT
          cost_center_id,
          forecast_date,
          p50_cost_usd,
          p10_cost_usd,
          p90_cost_usd
      FROM {{tenant_db}}.forecast
      WHERE forecast_horizon_days <= 30
        AND forecast_made_at = (SELECT max(forecast_made_at) FROM {{tenant_db}}.forecast)
      ORDER BY cost_center_id, forecast_date

  - title: "Cost anomalies (last 30 d)"
    type: table
    columns: [detected_at, cost_center_id, service_name, severity, anomaly_class, residual_usd]
    query: |
      SELECT
          detected_at,
          cost_center_id,
          service_name,
          severity,
          anomaly_class,
          round(residual_usd, 2) AS residual_usd
      FROM {{tenant_db}}.cost_anomaly
      WHERE detected_at >= now() - INTERVAL 30 DAY
      ORDER BY detected_at DESC
      LIMIT 50
```

Open the dashboard URL printed in the response. Expected: 5 panels rendered within ≤ 2 s.

## Step 4 — Configure budget threshold alerts (≤ 5 min)

```sh
oya finops-portal budget alert-config \
    --tenant acme-corp \
    --notification-channels email:cfo@acme-corp.example,slack:#finops-alerts \
    --thresholds 50,75,90,100
```

This configures alerts at 50%, 75%, 90%, 100% of monthly budget per cost center.

Test a threshold cross:

```sh
oya finops-portal budget simulate-threshold-cross \
    --tenant acme-corp \
    --cost-center engineering-platform \
    --simulated-fraction 0.91
```

Expected: alert fires within 30 s; email + Slack notification sent; `finops_portal.budget_threshold_crossed` event emitted to audit-chain.

Verify:

```sh
oya audit query --tenant acme-corp --event-class finops_portal.budget_threshold_crossed --since 5m
```

## Step 5 — Trigger a forecast refresh (≤ 5 min)

The forecast runs nightly at paid with per_usage billing_component, but on-demand is supported:

```sh
oya finops-portal forecast refresh \
    --tenant acme-corp \
    --horizon-days 30,60,90 \
    --model auto-select
```

Expected runtime: ~ 4 min for 30 days of trailing data. Output:

```json
{
  "tenant_id": "acme-corp",
  "models_evaluated": [
    {"name": "prophet", "mape_30d_holdout": 6.2, "mape_90d_holdout": 12.4},
    {"name": "arima", "mape_30d_holdout": 7.8, "mape_90d_holdout": 15.1},
    {"name": "ensemble", "mape_30d_holdout": 5.9, "mape_90d_holdout": 11.8}
  ],
  "selected_model": "ensemble",
  "forecast_horizon_days_supported": [30, 60, 90],
  "trained_at": "2026-05-20T14:32:00Z"
}
```

The dashboard's forecast panel now displays the new forecast.

## Step 6 — Export FOCUS-compliant data to Apptio (≤ 10 min)

```sh
oya finops-portal export focus \
    --tenant acme-corp \
    --since 2026-04-21 \
    --until 2026-05-20 \
    --format parquet \
    --output ./acme-corp-focus-2026-04-21.parquet
```

This emits a Parquet file with the FOCUS v1.0 column set:

| Column | Description |
|---|---|
| ServiceName | "k8s-compute", "postgres", "seaweedfs-s3", etc. |
| ServiceCategory | "Compute", "Storage", "Network", etc. |
| BilledCost | Decimal128(2) USD |
| EffectiveCost | Decimal128(2) USD (after commitments) |
| ListCost | Decimal128(2) USD (pre-discount) |
| ContractedCost | Decimal128(2) USD |
| ResourceId | Per-resource unique ID |
| ResourceName | Human-readable resource name |
| Region | "us-west-2", "syd-1", etc. |
| ChargeCategory | "Usage", "Tax", "Adjustment", "Purchase", "Credit" |
| ChargePeriodStart | DateTime64(3) UTC |
| ChargePeriodEnd | DateTime64(3) UTC |
| BillingCurrency | "USD", "EUR", etc. |
| TenantId | "acme-corp" |
| CostCenterId | "engineering-platform", "sales", etc. (oyatie extension) |

The Parquet file is consumable by Apptio Cloudability, CloudHealth, Anodot, Vantage, or any FOCUS-spec-compliant consumer.

Import to Apptio:

```sh
apptio-cli import-focus \
    --source ./acme-corp-focus-2026-04-21.parquet \
    --apptio-instance https://apptio.acme-corp.example
```

## What you've learned

- Tag-allocation policy authoring for chargeback.
- Per-cost-center dashboard with forecast + anomaly + budget-burn panels.
- Budget threshold alerts + Cedar-enforced notification routing.
- FOCUS-spec export consumable by downstream FinOps tools.

Next tutorial: `tutorials/cost-anomaly-investigation.md` — investigate a cost spike attributed to a specific cost center using the STL + Holt-Winters residual breakdown.
