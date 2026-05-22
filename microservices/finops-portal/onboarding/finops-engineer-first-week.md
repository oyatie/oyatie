---
doc_class: Onboarding
microservice: finops-portal
persona: finops-engineer + tenant-cfo-engineer
related_adrs: [ADR-0263, ADR-0131, ADR-0316]
date: 2026-05-20
doc_status: published
---

# FinOps Engineer onboarding — first 5 working days on `finops-portal`

Audience: a new FinOps engineer or tenant-CFO-side platform engineer joining the `finops-portal` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, ingested cost data from `cloud-billing`, authored a per-tenant chargeback dashboard, exercised the forecast pipeline, run a cost-anomaly drill, and walked the budget-threshold-crossed runbook.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 30 min) + `ARCHITECTURE.md` § ingest-pipeline + § FOCUS-mapping (∼ 45 min).
2. Read the FOCUS v1.0 spec (`docs/external/focus-v1.0.md`; ∼ 60 min). Note the columns: `BilledCost`, `EffectiveCost`, `ListCost`, `ContractedCost`, `ResourceId`, `ResourceName`, `ServiceName`, `Region`, `ChargeCategory`, `ChargeClass`, etc.
3. Open the Grafana folder `finops-portal`. core boards: `finops-cost-attribution`, `finops-budget-burn`, `finops-anomaly-rate`, `finops-forecast-accuracy`, `finops-dashboard-render-latency`.
4. Walk `runbooks/README.md`. The on-call runbooks: `cost-ingest-stalled.md`, `budget-threshold-spurious.md`, `forecast-mape-degraded.md`, `anomaly-storm.md`, `dashboard-render-slow.md`, `fx-rate-stale.md`, `cloud-billing-reconciliation-drift.md`, `chargeback-allocation-orphan.md`.
5. Sit in on the Wednesday FinOps handoff. Watch the outgoing rotation review the past-week budget-burn ledger + anomaly-rate distribution.

Acceptance: you can sketch the read path: tenant API → Cedar gate → ClickHouse cost warehouse query → tag-allocation evaluation → currency conversion → JSON response. And the write path: `cloud-billing` accrual → Pulsar → ClickHouse cost warehouse → MV refresh → Grafana panel.

## Day 2 — demo_trial finops-portal cell bootstrap

```sh
cargo run -p oya-dev-cli -- finops-portal bootstrap \
    --profile demo_trial \
    --cell drill-syd-1 \
    --analytics-cluster drill-clickhouse-syd-1 \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/finops_portal \
    --pulsar-endpoint pulsar://drill-pulsar-syd-1:6650 \
    --cloud-billing-endpoint http://drill-cloud-billing-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 8 min. Verify after bootstrap:

```sh
oya finops-portal health --cell drill-syd-1
# Expected:
#   ingest.cloud-billing: connected (last_snapshot=2026-05-19T00:00:00Z)
#   warehouse.clickhouse: up (cost_event table exists, 0 rows)
#   forecast.engine: not-configured (demo_trial: no forecasting)
#   currency.fx-source: payments-µservice (fx_age=8h)
```

Acceptance: cell is live; you can describe why demo_trial uses the analytics ClickHouse cluster (cost-saving — no separate warehouse).

## Day 3 — Cost ingest + per-tenant dashboard authoring

Trigger a manual cost-ingest snapshot:

```sh
oya finops-portal ingest manual-snapshot \
    --cell drill-syd-1 \
    --tenant drill-acme \
    --since 2026-05-13T00:00:00Z \
    --until 2026-05-19T23:59:59Z
```

This pulls 7 days of cost data from `cloud-billing` into the ClickHouse cost warehouse. Verify ingest:

```sh
clickhouse-client --host drill-ch-1 --query "
    SELECT
        toYYYYMMDD(charge_period_start) AS day,
        service_name,
        sum(effective_cost) AS daily_cost_usd
    FROM tenant_drill_acme.cost_event
    GROUP BY day, service_name
    ORDER BY day, service_name
    LIMIT 50"
```

Expected: 7 days × 8-12 services × 1-3 cost rows per day = ~ 150-250 rows. Costs in USD with `Decimal128(2)` precision.

Now author a tenant dashboard via the API:

```sh
oya finops-portal dashboard create \
    --tenant drill-acme \
    --dashboard-id daily-cost-by-service \
    --title "Daily cost by service (last 7 days)" \
    --query-file ./dashboards/daily-cost-by-service.yaml \
    --refresh-interval 1h \
    --visibility tenant-admin,tenant-cfo
```

The dashboard YAML (`./dashboards/daily-cost-by-service.yaml`):

```yaml
panels:
  - type: stacked-bar
    x_axis: day
    y_axis: daily_cost_usd
    series: service_name
    title: "Daily cost by service"
    query: |
      SELECT
          toYYYYMMDD(charge_period_start) AS day,
          service_name,
          sum(effective_cost) AS daily_cost_usd
      FROM {{tenant_db}}.cost_event
      WHERE charge_period_start >= now() - INTERVAL 7 DAY
      GROUP BY day, service_name
      ORDER BY day, service_name
```

Acceptance: dashboard renders correctly in the portal UI; you can explain why we use ClickHouse for cost-data queries (low latency for OLAP-shaped queries; same warehouse as `analytics` µservice; tenants get the same query model).

## Day 4 — Forecast (paid tenant_class feature backed by billing_components; preview at demo_trial)

Even though demo_trial doesn't run the full forecast pipeline, you can preview the model in shadow mode:

```sh
oya finops-portal forecast preview \
    --tenant drill-acme \
    --horizon-days 30 \
    --model prophet \
    --output ./forecast-preview.json
```

The preview pulls trailing 90 d (synthetic data we ingested in Step 2 will have 7 d only; the preview backfills synthetic from 90 d for the rehearsal). The output:

```json
{
  "tenant_id": "drill-acme",
  "model": "prophet",
  "horizon_days": 30,
  "forecast": [
    {"date": "2026-05-21", "p50_cost_usd": 412.50, "p10_cost_usd": 380.15, "p90_cost_usd": 448.20},
    ...
  ],
  "training_data_points": 90,
  "mape_on_holdout": 6.4
}
```

A MAPE of 6.4 % on 30-d holdout is within paid tenant_class SLO backed by per_usage billing_component (< 8 %). Note that demo_trial doesn't run the forecast continuously — paid tenant_class does.

Authoring a budget rule:

```sh
oya finops-portal budget create \
    --tenant drill-acme \
    --budget-id monthly-compute-budget \
    --scope-filter service_name=k8s-compute \
    --period monthly \
    --threshold-usd 5000 \
    --alert-thresholds 50,75,90,100
```

Walk the threshold-burn alert path: open Grafana `finops-budget-burn` panel for `drill-acme`; simulate threshold cross via:

```sh
oya finops-portal budget simulate-threshold-cross \
    --tenant drill-acme \
    --budget monthly-compute-budget \
    --simulated-fraction 0.92
```

This emits `finops_portal.budget_threshold_crossed` to audit-chain + alerts via the `oya-on-call` notification path. Validate the event lands:

```sh
oya audit query --tenant drill-acme --event-class finops_portal.budget_threshold_crossed --since 5m
```

Acceptance: budget rule authored, threshold-cross simulated, alert path verified.

## Day 5 — Anomaly drill + chargeback walk

Read `runbooks/anomaly-storm.md` end-to-end.

Run the anomaly-detection drill:

```sh
oya finops-portal drill cost-anomaly-injection \
    --cell drill-syd-1 \
    --tenant drill-acme \
    --anomaly-class "k8s-compute 3x-spike sustained 2h" \
    --duration 5m
```

The drill injects synthetic cost data simulating a 3× compute-cost spike over 2 hours. The STL+3σ detector should flag within ~ 90 s. Verify:

```sh
oya finops-portal anomaly list --tenant drill-acme --since 10m
# Expected: 1 anomaly with class=cost_spike_3x, severity=high, attributed_resource=k8s-compute
```

Now walk a tenant chargeback model. The tenant `drill-acme` has 3 cost centers: `engineering`, `sales`, `marketing`. Per their tag-allocation policy:

- `engineering`: 60 % of compute, 100 % of storage, 80 % of network.
- `sales`: 25 % of compute, 0 % of storage, 15 % of network.
- `marketing`: 15 % of compute, 0 % of storage, 5 % of network.

```sh
oya finops-portal chargeback configure \
    --tenant drill-acme \
    --policy-file ./chargeback-policy.yaml
```

The policy:

```yaml
cost_centers:
  engineering:
    compute_share: 0.60
    storage_share: 1.00
    network_share: 0.80
  sales:
    compute_share: 0.25
    storage_share: 0.00
    network_share: 0.15
  marketing:
    compute_share: 0.15
    storage_share: 0.00
    network_share: 0.05
```

Run the chargeback computation for the past week:

```sh
oya finops-portal chargeback compute \
    --tenant drill-acme \
    --since 2026-05-13T00:00:00Z \
    --until 2026-05-19T23:59:59Z \
    --output ./chargeback-week.csv
```

Each row: `cost_center, service, daily_cost_usd, allocated_cost_usd`.

Acceptance: chargeback computed; you can explain the tag-vs-allocation trade-off (tag-allocation = explicit % per resource; alternative is direct-tag where the tenant tags every resource with the consuming cost center — direct-tag is more accurate but requires every resource to be tagged, which is rare at the start).

## What you've learned

- demo_trial bootstrap + cost ingest from `cloud-billing`.
- Per-tenant dashboard authoring + budget rules + threshold-cross alerting.
- Anomaly-detection drill (the most-likely page on paid with per_seat billing_component).
- Chargeback model authoring + tag-allocation policy.
- The FOCUS spec mapping from `cloud-billing` events into the portal warehouse.

Next week: paid with per_seat billing_component promotion (hourly refresh + per-resource attribution + multi-currency rendering), paid with per_usage billing_component tour (ML forecast pipeline + chargeback ledger), paid with compliance_pack gating tour (sovereign-pack cost residency), and your first production shadow.
