---
doc_class: Benchmark
microservice: finops-portal
benchmark_date: 2026-05-20
related_adrs: [ADR-0263, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie finops-portal vs AWS Cost Explorer vs GCP Billing vs Apptio Cloudability vs CloudHealth

Workloads measured: (a) dashboard render latency, (b) cost data freshness, (c) forecast accuracy at 30/90-d horizon, (d) anomaly detection precision/recall, (e) FOCUS export wall-clock, (f) annual TCO for a 5 000-tenant fleet at $10M/year aggregate cost.

Hardware (oyatie paid with per_seat billing_component on-prem): 4× portal-api nodes (8 vCPU EPYC 9354P, 32 GiB RAM, 500 GiB NVMe), shared ClickHouse cost warehouse (with `analytics`), PostgreSQL 16.6 + 1 replica. Network: 25 GbE leaf-spine.

## Workload (a) — dashboard render latency (5-panel dashboard, 30-day cost window, 8 cost-centers, multi-currency)

| Platform | p50 (ms) | p99 (ms) | Render cadence supported |
|---|---:|---:|---|
| oyatie finops-portal (paid with per_seat billing_component) | 380 | 740 | Up to hourly |
| oyatie finops-portal (paid with per_usage billing_component) | 240 | 480 | Up to 15-min |
| AWS Cost Explorer | 1 200 | 4 800 | Up to daily |
| GCP Billing Console | 1 800 | 6 500 | Up to daily |
| Azure Cost Management | 1 100 | 4 200 | Up to daily |
| Apptio Cloudability | 800 | 2 800 | Up to daily |
| CloudHealth | 950 | 3 100 | Up to 8-hour |

Reading: oyatie's edge is the ClickHouse OLAP backend — the same warehouse that powers `analytics`. Cloud-native cost consoles are bottlenecked by their managed-warehouse query latency. Apptio + CloudHealth use Snowflake-like backends and are competitive but trail oyatie at p99.

## Workload (b) — cost data freshness (charge incurred → visible in dashboard)

| Platform | Best-case latency | Typical | Refresh model |
|---|---:|---:|---|
| oyatie finops-portal (demo_trial) | 24 h | 24 h | Daily snapshot from `cloud-billing` |
| oyatie finops-portal (paid with per_seat billing_component) | 60 min | 90 min | Hourly accrual snapshot |
| oyatie finops-portal (paid with per_usage billing_component) | 15 min | 30 min | Streaming ingest from `cloud-billing` event stream |
| AWS Cost Explorer | 24 h | 24-48 h | CUR daily |
| AWS Cost Explorer (hourly) | 8 h | 8-24 h | Beta hourly granularity |
| GCP Billing (standard) | 24 h | 24-48 h | Detailed Usage Cost daily |
| Azure Cost Management | 24-72 h | 24-72 h | Daily |
| Apptio Cloudability | 24 h | 24-48 h | Daily ETL from CSP APIs |
| CloudHealth | 8 h | 8-24 h | 3× daily ETL |

Reading: oyatie paid with per_usage billing_component's 15-min refresh is best-in-class. The cloud-native consoles (AWS / GCP / Azure) are bottlenecked by their own billing-export cadence; even Apptio + CloudHealth can't beat what the CSP exports.

## Workload (c) — forecast accuracy (MAPE on 30-day and 90-day horizon, 5-tenant fleet sample)

| Platform | 30-d MAPE | 90-d MAPE | Model |
|---|---:|---:|---|
| oyatie finops-portal (paid with per_usage billing_component; Prophet+ARIMA ensemble) | 6.2 % | 12.4 % | Per-tenant auto-select |
| AWS Cost Explorer Forecasting | 11.8 % | 22.4 % | AWS's published model (gradient boosting) |
| GCP Billing Forecast (beta) | 9.4 % | 18.6 % | GCP's published model |
| Apptio AI Cost Forecast | 7.8 % | 15.2 % | Apptio's published model |
| CloudHealth Forecast | 8.6 % | 16.8 % | CloudHealth's published model |

Reading: oyatie leads at both horizons. The ensemble approach + per-tenant auto-select captures patterns that single-model approaches miss. The cloud-native forecasts are at the bottom because they're trained on aggregate workloads, not per-tenant histories.

Caveat: this is a 5-tenant sample over Q1-2026. Tenants with highly variable workloads (HPC, ML training bursts) will see worse MAPE everywhere.

## Workload (d) — anomaly detection precision/recall (per 1 000 cost-events, F1 score)

Test set: 1 000 cost-events with 30 known-anomalies injected (3× spike, 0.5× drop, sustained 2-h elevation, sudden 24-h plateau).

| Platform | True positives | False positives | False negatives | Precision | Recall | F1 |
|---|---:|---:|---:|---:|---:|---:|
| oyatie finops-portal (paid with per_usage billing_component; STL + Holt-Winters + 3σ) | 27 | 4 | 3 | 0.871 | 0.900 | 0.885 |
| AWS Cost Anomaly Detection | 21 | 7 | 9 | 0.750 | 0.700 | 0.724 |
| GCP Billing Anomalies | 18 | 8 | 12 | 0.692 | 0.600 | 0.643 |
| Apptio AI Anomaly | 24 | 6 | 6 | 0.800 | 0.800 | 0.800 |
| CloudHealth Anomaly | 22 | 5 | 8 | 0.815 | 0.733 | 0.772 |

Reading: oyatie's F1 of 0.885 leads the field. STL + Holt-Winters captures seasonal patterns that pure-statistical approaches (3σ alone) miss; 3σ residual catches the spikes that pure-seasonal approaches miss.

## Workload (e) — FOCUS export wall-clock (30 d × 5 000 tenants × 10 services)

| Platform | Wall-clock | Output format | FOCUS-compliant |
|---|---:|---|---|
| oyatie finops-portal (paid with per_seat billing_component, parallel) | 8 min | Parquet + CSV | Yes (v1.0) |
| oyatie finops-portal (paid with per_usage billing_component, parallel) | 4 min | Parquet + CSV + Iceberg | Yes (v1.0) |
| AWS CUR + FOCUS adapter | 30-60 min (daily ETL window) | Parquet | Beta (CUR2.0) |
| GCP Detailed Usage Cost + FOCUS adapter | 20-45 min | Parquet | Beta |
| Azure Cost Export + FOCUS adapter | 25-50 min | Parquet | Beta |
| Apptio Cloudability FOCUS export | 12-25 min | Parquet | Yes (v1.0) |

Reading: oyatie's edge is the streaming-pipeline architecture; the export is a query against an OLAP warehouse, not a daily ETL job. At 5 000 tenants, 30 days the data set is ~ 40-80 GiB Parquet.

## Workload (f) — annual TCO for a 5 000-tenant FinOps platform

Assumptions: 5 000 tenants total, $10 M/year aggregate cost being tracked, multi-cloud (AWS + GCP + Azure + on-prem).

| Platform | Licence (USD) | Hardware/compute (USD) | Ops (USD) | Total (USD) |
|---|---:|---:|---:|---:|
| oyatie finops-portal (paid with per_seat billing_component, self-hosted) | 0 | 192 000 (4 nodes + shared CH) | 248 000 (2 SRE × 0.4 FTE) | 440 000 |
| oyatie finops-portal (paid with per_usage billing_component, self-hosted) | 0 | 540 000 (8 nodes multi-AZ + GPU forecast pipeline) | 372 000 (3 SRE × 0.4 FTE) | 912 000 |
| AWS Cost Explorer (free; limited features) | 0 | 0 (managed) | 248 000 | 248 000 |
| AWS Cost Explorer + Cost & Usage Reports + custom BI | 0 | 84 000 (Athena + S3) | 372 000 (3 SRE) | 456 000 |
| Apptio Cloudability | 480 000 (~ $96 per tenant per year list) | 0 (managed SaaS) | 124 000 | 604 000 |
| CloudHealth (VMware) | 420 000 (~ $84 per tenant per year list) | 0 (managed SaaS) | 124 000 | 544 000 |
| Vantage | 360 000 (~ $72 per tenant per year list) | 0 (managed SaaS) | 124 000 | 484 000 |

Reading: oyatie paid with per_seat billing_component beats Apptio + CloudHealth + Vantage on TCO AND ships richer features (audit-chain integration, sovereign-pack support, native multi-cloud) that SaaS competitors charge extra for or don't offer.

Caveats:

- "Tenants" here means cost-attribution scopes (sub-accounts, projects, business-units). Most SaaS competitors price per-scope-per-month; we price per-cell.
- The ops cost includes the FinOps platform engineer + 0.2 SRE for the forecast pipeline. Lower-touch tenants (those who only use the basic dashboard, no custom forecast) need ~ 0.4 FTE per 5 000 tenants.
- AWS Cost Explorer "free" assumes the tenant is comfortable with daily-granularity + AWS-only. Multi-cloud tenants need a third-party tool anyway.

## Reproducibility

The benchmark harness lives at `benchmarks/finopsbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks finops-portal \
    --workload 5000-tenants-30d-cost \
    --tenant-class paid \
    --output ./benchmark-results.json
```

Cloud-comparator runs require valid CSP credentials + a sample tenant fleet that the CSP can attribute. Apptio + CloudHealth + Vantage runs require the respective trial/sandbox accounts. Results live at `benchmarks/results/finops-portal/<date>.csv` and are re-run quarterly to detect drift.
