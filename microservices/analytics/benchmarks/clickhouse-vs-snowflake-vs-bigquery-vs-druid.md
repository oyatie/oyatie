---
doc_class: Benchmark
microservice: analytics
benchmark_date: 2026-05-20
related_adrs: [ADR-0193, ADR-0184, ADR-0329, ADR-0330, ADR-0331]
doc_status: published
---

# Benchmarks — oyatie analytics (ClickHouse) vs Snowflake vs BigQuery vs Druid vs Pinot

Workloads measured: (a) point lookup, (b) standard dashboard aggregate (1 day window, GROUP BY 3 columns, 7-day range), (c) cardinality estimation over 100 M rows, (d) funnel query (5 stages, 24 h window, 10 M users), (e) annual TCO at 50 TiB hot tier + 1 PiB cold tier + 30 k qps fleet-wide.

Hardware (for on-prem ClickHouse): 6× bare-metal nodes, each 32 vCPU AMD EPYC 7543P, 256 GiB DDR4, 4× 1.92 TiB NVMe (RAID-10), dual 25 GbE NIC. Network: 25 GbE leaf-spine, MTU 9000.

Cloud comparators measured on equivalent compute / managed service. Snowflake: M-size warehouse (32 vCPU equivalent). BigQuery: on-demand pricing, slot-allocation flat-rate equivalent. Druid: AWS m6i.8xlarge × 6 (broker + historical + middle-manager). Pinot: AWS m6i.8xlarge × 6 (broker + server + controller).

## Workload (a) — point lookup (single row by primary key, table at 10 B rows)

| Engine | p50 (ms) | p99 (ms) | Notes |
|---|---:|---:|---|
| oyatie ClickHouse 26.3 LTS | 4 | 18 | ORDER BY tuple is the primary index; sparse-index sweep is fast |
| Snowflake (M-warehouse) | 850 | 1 800 | Snowflake doesn't have point-lookup indexes; full micro-partition scan |
| BigQuery (on-demand) | 740 | 1 600 | Same; BigQuery has clustering + partitioning but no PK index |
| Druid 30.0 | 12 | 32 | Druid has segment-level indexes; competitive |
| Pinot 1.2 | 8 | 22 | Pinot has inverted indexes; competitive |

Reading: ClickHouse's primary-key ORDER BY tuple gives us near-OLTP point-lookup latency that warehouses (Snowflake, BigQuery) cannot match. Druid and Pinot are competitive but trail by 2-4×.

## Workload (b) — standard dashboard aggregate (1 d window, GROUP BY 3 cols, 7 d range, 5 M rows scanned)

| Engine | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie ClickHouse 26.3 LTS | 38 | 92 |
| Snowflake (M-warehouse) | 220 | 480 |
| BigQuery (on-demand) | 280 | 520 |
| Druid 30.0 | 65 | 145 |
| Pinot 1.2 | 42 | 110 |

Reading: ClickHouse leads at this shape; Pinot is within margin (Pinot's columnar + bitmap indexes are highly optimised for this exact shape). Snowflake / BigQuery have higher fixed overhead (~ 100-200 ms warehouse warm + planning) that dominates at the dashboard scale.

## Workload (c) — cardinality estimation over 100 M rows (`uniqExact` / `COUNT DISTINCT`)

| Engine | p50 (ms) | p99 (ms) | Memory peak |
|---|---:|---:|---:|
| oyatie ClickHouse 26.3 LTS (`uniqHLL12`) | 220 | 480 | 1.8 GiB |
| oyatie ClickHouse 26.3 LTS (`uniqExact`) | 1 100 | 2 400 | 8.2 GiB |
| Snowflake `APPROX_COUNT_DISTINCT` | 1 800 | 3 200 | (managed) |
| Snowflake `COUNT(DISTINCT)` | 11 000 | 24 000 | (managed) |
| BigQuery `APPROX_COUNT_DISTINCT` | 2 400 | 4 800 | (managed) |
| Druid 30.0 (HyperLogLog) | 480 | 1 100 | 2.4 GiB |
| Pinot 1.2 (HyperLogLog) | 380 | 920 | 2.1 GiB |

Reading: HyperLogLog-based approximations are ~ 5-50× faster than exact counts. Tenants who want exact counts at this scale should run the MV pattern; ad-hoc exact counts above 100 M rows are budget-bounded by the per-tenant quota.

## Workload (d) — funnel query (5 stages, 24 h window, 10 M users)

| Engine | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie ClickHouse 26.3 LTS (`windowFunnel`) | 280 | 620 |
| Snowflake (custom UDF + sort) | 4 200 | 8 800 |
| BigQuery (custom SQL with arrays + sort) | 3 800 | 7 600 |
| Druid 30.0 (no native funnel; sequence scan) | 1 200 | 2 800 |
| Pinot 1.2 (no native funnel; sequence scan) | 1 400 | 3 200 |
| Amplitude (managed funnel product) | (managed; ~ 800 ms p99 published) | n/a |
| Mixpanel (managed funnel product) | (managed; ~ 1 200 ms p99 published) | n/a |

Reading: ClickHouse's native `windowFunnel` aggregate puts our funnel latency in the same league as Amplitude / Mixpanel managed funnel products — but the data stays in our tenant's database. Snowflake / BigQuery funnel emulation is multi-second; not competitive.

## Workload (e) — annual TCO at 50 TiB hot + 1 PiB cold + 30 k qps fleet-wide

| Platform | Hardware / compute (USD) | Cold storage (USD) | Licence (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|---:|
| oyatie ClickHouse (deployment_context=on-prem, tenant_class=paid, 9-node) | 384 000 | 92 000 (SeaweedFS @ 1 PiB) | 0 | 248 000 (2 SRE × 0.4 FTE) | 724 000 |
| Snowflake M-warehouse + cold storage | 1 800 000 (auto-suspend; 30 k qps avg) | 240 000 | 0 | 124 000 (1 SRE × 0.2 FTE) | 2 164 000 |
| BigQuery on-demand | 1 400 000 (per-query) | 240 000 | 0 | 124 000 | 1 764 000 |
| BigQuery flat-rate 2 000 slots | 1 152 000 | 240 000 | 0 | 124 000 | 1 516 000 |
| Druid (AWS m6i.8xlarge × 12) | 642 000 | 240 000 | 0 | 372 000 (3 SRE × 0.4 FTE; Druid ops complexity) | 1 254 000 |
| Pinot (AWS m6i.8xlarge × 12) | 642 000 | 240 000 | 0 | 372 000 | 1 254 000 |
| Mixpanel (managed; 100 M events/mo) | (managed) | (managed) | 1 800 000 | 0 | 1 800 000 |
| Amplitude (managed; 100 M events/mo) | (managed) | (managed) | 1 560 000 | 0 | 1 560 000 |

oyatie's edge vs Snowflake / BigQuery is the absence of per-query / per-slot warehouse fee. The edge vs Druid / Pinot is the ops complexity (a 9-node ClickHouse cell with `clickhouse-backup` is fewer moving parts than a Druid broker + historical + middle-manager + overlord topology). The edge vs Mixpanel / Amplitude is the data residency (tenant data stays in our cell; no third-party processor; no GDPR Art. 28 sub-processor list extension).

Caveats:

- These numbers assume 24×7 30 k qps utilisation. Bursty workloads tilt Snowflake's auto-suspend favourably; if the tenant is < 10 % utilised, Snowflake auto-suspend wins on cost (but loses on cold-start latency).
- The ops-time number is from our 2026-Q1 internal ops survey. Other organisations should re-baseline.
- The Mixpanel / Amplitude pricing is the published list price for 100 M-events-per-month tier; volume discounts apply.

## Reproducibility

The benchmark harness is at `benchmarks/analyticsbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks analytics \
    --workload funnel-10m-users \
    --engine oyatie-clickhouse-paid-onprem \
    --output ./benchmark-results.json
```

Cloud comparators require valid `--cloud-credentials`. The results live at `benchmarks/results/analytics/<date>.csv` and are re-run weekly in CI to detect drift.
