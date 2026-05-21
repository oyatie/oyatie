---
doc_class: FAQ
microservice: finops-portal
persona: finops-engineer + tenant-cfo + finops-platform-engineer
date: 2026-05-20
doc_status: published
---

# FinOps Engineer FAQ — finops-portal

## Why FOCUS spec v1.0 and not the CSP-specific cost formats (AWS CUR, GCP Billing Export, Azure Cost Export)?

Per ADR-XXX-finops-focus-canonical. The FOCUS spec (FinOps Foundation Open Cost & Usage Specification) is the cross-cloud canonical format published by the FinOps Foundation in 2024-Q4 v1.0. It normalizes columns across AWS, GCP, Azure, OCI, IBM Cloud + on-prem so a tenant gets ONE schema regardless of their cloud mix.

Without FOCUS, tenants with multi-cloud get N-times-the-engineering for cost dashboards. AWS CUR has 100+ columns with AWS-specific semantics (`lineItem/UsageType` is AWS-specific). GCP BigQuery billing export has different semantics (`sku.id` ≠ AWS UsageType). FOCUS gives us `ServiceName, SkuPriceId, BilledCost, EffectiveCost, ListCost` — same column names across clouds.

AWS, GCP, Azure all ship FOCUS-format exports natively as of 2025 (AWS via CUR2.0, GCP via Detailed Usage Cost, Azure via Cost Management exports). We ingest these directly without re-normalization.

## Why hourly refresh at paid with per_seat billing_component, not 5-minute like analytics?

Per ADR-XXX-finops-refresh-cadence. Cost data has lower temporal resolution than analytics events:

- Cloud providers emit cost meters at hour-granularity (AWS CUR), 30-min (GCP), or hour (Azure). The freshest possible refresh is ~ 1 h.
- Tenant decisions on cost (scale-up, scale-down, budget alert) have hour+ latency tolerance; sub-hour refresh doesn't change outcomes.
- Hourly refresh keeps the ingest pipeline simple; per-event streaming (15-min at paid with per_usage billing_component) adds complexity that's only justified for high-spend tenants.

Tenants who need real-time cost (rare; mostly HPC tenants with bursty workloads) use the `cloud-billing` µservice's per-event API directly + write their own dashboard.

## What's the difference between BilledCost, EffectiveCost, ListCost, ContractedCost?

Per FOCUS v1.0 § "Cost columns":

- **BilledCost**: the invoice line-item amount actually billed to the CSP. May reflect tax, regional pricing, currency.
- **EffectiveCost**: the cost after applying all commitments / negotiated discounts (RIs, savings plans, EDPs, etc.). This is the "real" cost to the tenant.
- **ListCost**: the public price-list cost — what the resource would cost without any discounts.
- **ContractedCost**: the cost under the active negotiated agreement (might differ from EffectiveCost if commitments fail to apply).

For tenants planning budgets: use `EffectiveCost` (it's the actual outflow). For tenants negotiating with CSPs: use `ListCost - EffectiveCost` to compute commitment savings.

## Why Prophet + ARIMA + ensemble at paid with per_usage billing_component and not just one model?

Per ADR-XXX-finops-forecast-model-mix. Three model classes capture different cost patterns:

1. **Prophet** (Meta open-source): captures yearly + weekly + daily seasonality, holiday effects. Best for tenants with predictable usage rhythms (e.g., B2B SaaS with weekday peaks).
2. **ARIMA**: best for non-seasonal trending workloads (e.g., growing SaaS startup with linear cost growth).
3. **Ensemble** (weighted average of Prophet + ARIMA + linear): best when neither pure model dominates; we weight by per-tenant historical performance.

Per-tenant we auto-select the best model based on holdout MAPE from the last 30 d. Cold-start tenants default to ensemble until 30 d of data exists.

The MAPE SLO is < 8 % at 30 d and < 15 % at 90 d. Tenants whose forecast MAPE degrades above these thresholds get re-trained nightly + alerted (`finops_portal.forecast_mape_degraded`).

## How is "chargeback" different from "showback"?

Per FinOps Foundation Framework v2.1:

- **Showback**: the portal displays cost attribution to internal cost centers without actually invoicing the cost centers. Awareness, not financial. Used by most enterprises at the start of FinOps adoption.
- **Chargeback**: cost is actually invoiced to internal cost centers; cost centers have their own budgets and pay for consumption. Used by mature FinOps organizations.

oyatie's `finops-portal` supports BOTH at paid with per_usage billing_component. The configuration knob is `chargeback_mode: showback | chargeback`. Showback is read-only attribution. Chargeback emits ledger postings into the tenant's internal accounting system via the `payments` µservice (for tenants where cost centers are themselves billable entities).

## Why STL + Holt-Winters + 3-sigma for anomaly detection? Why not Isolation Forest or LSTM?

Per ADR-XXX-finops-anomaly-algorithm. STL (Seasonal-Trend-Loess decomposition) + Holt-Winters seasonal smoothing + 3-sigma residual analysis is:

- Interpretable: when an anomaly fires, the runbook can explain WHY ("expected ~ $X based on trailing 30-d weekly pattern; observed $Y; residual exceeds 3σ threshold").
- Cheap: O(n log n) per series; we can run it per-tenant per-resource at 15-min cadence.
- Per-tenant per-resource: no shared model, no cross-tenant signal leakage.

Isolation Forest is competitive but less interpretable + costlier to run per-tenant-per-resource. LSTM/transformer-based anomaly detection (e.g., Donut, Anomaly-Transformer) achieves 5-15 % better F1 in published benchmarks but at 50-100× compute cost — not justified at our envelope. We may revisit at paid with compliance_pack gating if a sovereign-pack tenant requires deep-learning anomaly attribution for regulatory reasons.

## What's the FX rate source for multi-currency display?

Per ADR-XXX-finops-fx-source. `finops-portal` does NOT subscribe to FX data directly. Instead, it reads the `payments` µservice's daily-snapshot rate (the same rate used for cross-currency settlement). This guarantees: the cost number a tenant sees in EUR matches the cost the tenant would PAY in EUR if they settled today.

The snapshot is taken at 00:00 UTC daily; rates valid through next snapshot. Tenants needing real-time rate (rare) can opt into paid with per_usage billing_component's hourly snapshot.

## How do tenants tag resources for cost allocation?

Per ADR-XXX-finops-tag-allocation. Tenants can tag at three layers:

1. **Resource-direct**: every Kubernetes pod, every PostgreSQL instance, every SeaweedFS bucket carries `cost_center`, `project`, `environment`, `business_unit` labels. The portal projects cost by tag.
2. **Project-rollup**: tenants define logical projects in the portal; resources are mapped to projects via label-selector. Cost rolls up to project.
3. **Allocation-policy**: untagged or partially-tagged resources fall through to a tenant-authored allocation policy (e.g., "all untagged compute = 60% engineering, 25% sales, 15% marketing"). This is the chargeback policy.

Most mature tenants use a mix: tag what's tagged, allocate what's not.

The audit chain emits `finops_portal.allocation.applied` for every cost-event that hits the allocation policy (vs being direct-tagged). Tenants can audit their allocation drift over time.

## What's the difference between this µservice and `cloud-billing`?

- `cloud-billing`: emits cost events for every billable consumption (oyatie BILLING the tenant). The accrual ledger. The source of truth for invoicing.
- `finops-portal`: VISUALIZES the cost data for the tenant. Per-tenant dashboards, budgets, forecasts, anomalies, chargebacks.

A tenant sees the invoice in `cloud-billing`; the tenant sees the cost dashboard in `finops-portal`. The two are physically separate µservices with separate Cedar principals (cloud-billing is a write-path with stricter permissions; finops-portal is a read-path with self-service permissions).

`finops-portal` ingests from `cloud-billing` via Pulsar. The relationship is unidirectional: cloud-billing is upstream, finops-portal is downstream.

## How do I respond when a tenant says "Apptio shows different numbers than oyatie"?

This usually means:

1. Apptio's cost data includes commitments (RIs, savings plans) applied at the account level. oyatie's FOCUS-spec EffectiveCost also includes commitments but may apply them differently (FOCUS spec defines this; Apptio is pre-FOCUS so they have their own algorithm).
2. Apptio's tag-allocation rules differ from oyatie's. Walk the tenant through their oyatie allocation policy + the corresponding Apptio rules.
3. Apptio's snapshot cadence may be daily while oyatie's is hourly — recent activity differs.
4. Tax + cross-border surcharges may be classified differently. FOCUS spec puts these in `ChargeCategory='Tax'` or `ChargeCategory='Adjustment'`; Apptio may roll them into the line-item.

The reconciliation runbook is `runbooks/apptio-cloudability-reconciliation.md`.

## What's the per-tenant query budget?

Per ADR-XXX-finops-query-budget. Each tenant has a daily query budget (default 100k ClickHouse rows scanned for dashboard queries). Tenants exceeding budget get rate-limited; tenants on paid with per_usage billing_component can extend budget via per-tenant quota policy. The budget is NOT a per-query limit; it's a daily roll-up to prevent runaway dashboard authors.

The ClickHouse query log (`system.query_log`) tracks per-tenant `read_rows` + `read_bytes` for billing-quota purposes. Cedar enforces.
