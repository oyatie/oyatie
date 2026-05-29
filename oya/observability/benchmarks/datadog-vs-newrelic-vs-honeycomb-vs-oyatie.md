---
doc_class: Benchmark
microservice: observability
benchmark_date: 2026-05-20
related_adrs: [ADR-0130, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
doc_status: published
---

# Benchmarks — oyatie observability vs Datadog APM vs New Relic vs Honeycomb vs Splunk Observability

Workloads measured: (a) sustained span ingest, (b) trace-id lookup p99, (c) PromQL range query p99, (d) cross-signal correlation query, (e) annual TCO at 1 M spans/sec sustained + 100 M active metric series + 2 TiB logs/day + 7-y promotion-gate evidence retention.

Hardware (oyatie on-prem paid tenant_class high-scale envelope, per ADR-0329 + ADR-0330 + ADR-0331): 9× ClickHouse data nodes (16 vCPU AMD EPYC 9354P, 64 GiB DDR5, 7.68 TiB NVMe RAID-10 each); Tempo 9-ingester / 9-querier fleet; Mimir 9-ingester with 3-replica TSDB; Loki 5-ingester with 3-replica chunk store; OTel collector fleet 12 nodes (8 vCPU, 32 GiB RAM each); Valkey Cluster 5 nodes for Mimir query-frontend cache. Network: 25 GbE leaf-spine, MTU 9000, ≤ 200 µs intra-AZ ping.

Cloud comparators measured on equivalent service tier per published 2026-Q2 list price.

## Workload (a) — sustained span ingest (spans/sec)

| Engine | Sustained (spans/sec) | Burst (spans/sec, ≤ 5 min) | Notes |
|---|---:|---:|---|
| oyatie observability (paid baseline) | 200 000 | 1 000 000 | Tempo 5-ingester + ClickHouse 5-node |
| oyatie observability (paid high-scale) | 1 000 000 | 5 000 000 | Tempo 9-ingester + ClickHouse 9-node, 3 AZs |
| Datadog APM (Enterprise) | 800 000 | 2 000 000 | Per Datadog 2026 SLA; tail-sampling at edge |
| New Relic (Data Plus tier) | 500 000 | 1 500 000 | Per NR 2026 SLA |
| Honeycomb (Pro tier) | 600 000 | 2 000 000 | Honeycomb Refinery sampling at edge |
| Splunk Observability (Cloud Enterprise) | 700 000 | 2 500 000 | Per Splunk 2026 SLA |

Reading: oyatie paid high-scale is competitive with the cloud SaaS leaders at sustained 1 M spans/sec. The hardware envelope (9 nodes × 16 vCPU + Tempo fleet) is the real cost; Datadog hides this cost in their per-host pricing.

## Workload (b) — trace-id lookup p99 latency (ms)

| Engine | 1 trace (ms) | 100 traces (ms) | 1 K traces (ms) |
|---|---:|---:|---:|
| oyatie observability (paid baseline) | 184 | 320 | 720 |
| oyatie observability (paid high-scale) | 148 | 240 | 480 |
| Datadog APM | 220 | 480 | 1 200 |
| New Relic distributed tracing | 280 | 620 | 1 800 |
| Honeycomb Retriever | 95 | 180 | 380 |
| Splunk Observability | 320 | 720 | 1 900 |

Reading: Honeycomb's Retriever (proprietary columnar trace store) wins on this workload by ~ 1.5× over oyatie paid high-scale. We accept this trade — Honeycomb optimised heavily for single-trace lookup. Our paid high-scale envelope is competitive with Datadog and beats New Relic / Splunk.

## Workload (c) — PromQL range query p99 latency (ms; 5 m range, 100 series)

| Engine | Simple metric (ms) | Aggregated (sum by tenant_id) (ms) | Histogram quantile (ms) |
|---|---:|---:|---:|
| oyatie observability (paid baseline) | 84 | 240 | 480 |
| oyatie observability (paid high-scale) | 42 | 120 | 240 |
| Datadog (DD metrics) | 180 | 420 | 720 |
| New Relic (NRQL metrics) | 220 | 480 | 920 |
| Honeycomb metrics | 280 | 580 | 1 100 |
| Splunk Observability | 120 | 280 | 540 |

Reading: oyatie paid high-scale beats every cloud SaaS comparator on PromQL range queries because Mimir's chunk-store is purpose-built for Prometheus-shape queries and our Valkey-backed query-frontend cache absorbs repeat queries. Datadog's metrics-Q-L is more flexible but slower; we trade flexibility for speed because the substrate doesn't need DD's surface area.

## Workload (d) — cross-signal correlation query (trace + metric + log over same request_id)

| Engine | Wall-clock (s) | Approach |
|---|---:|---|
| oyatie observability (paid baseline) | 1.8 | ClickHouse rollup join across traces.spans + metrics.samples + logs.entries on request_id |
| oyatie observability (paid high-scale) | 0.9 | Same with 3-AZ parallel exec |
| Datadog | 2.4 | Datadog APM's "trace context" auto-join (proprietary) |
| New Relic | 3.1 | New Relic One unified query (NRQL) |
| Honeycomb | (not natively supported) | Manual via columnar trace + separate metric/log queries |
| Splunk Observability | 4.2 | Splunk's SignalFlow cross-correlation |

Reading: oyatie's ClickHouse-rollup approach gives us a categorical edge — the rollup pipeline pre-joins the signals by request_id, so the query is just a `WHERE request_id = X` lookup. Datadog's auto-join is similar in spirit but slower at our cardinality.

## Workload (e) — annual TCO at 1 M spans/sec + 100 M active series + 2 TiB logs/day + 7-y retention

| Platform | Hardware / compute (USD) | Cold storage (USD) | Licence (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|---:|
| oyatie observability (paid high-scale on-prem, 9-node ClickHouse + Tempo/Mimir/Loki fleet) | 980 000 | 320 000 (SeaweedFS-S3 @ ~ 2 PiB total telemetry over 7 y) | 0 | 372 000 (3 SRE × 0.4 FTE) | 1 672 000 |
| Datadog APM + Metrics + Logs + APM Trace Search | 0 | 0 (bundled) | 4 200 000 (Enterprise tier, 1 M spans/sec custom contract) | 124 000 (1 SRE × 0.2 FTE) | 4 324 000 |
| New Relic Data Plus | 0 | 0 (bundled) | 3 400 000 (Data Plus, 100 GiB/host/mo @ 200 hosts + custom span tier) | 124 000 | 3 524 000 |
| Honeycomb Pro | 0 | 0 (bundled) | 2 600 000 (Pro tier 1 M spans/sec) | 124 000 | 2 724 000 |
| Splunk Observability Cloud Enterprise | 0 | 0 (bundled) | 3 800 000 (Cloud Enterprise span + metric tier) | 124 000 | 3 924 000 |

Reading: oyatie paid high-scale is roughly 40 % the cost of Honeycomb (cheapest cloud comparator) and 38 % the cost of Datadog (most expensive). The paid high-scale premium over paid baseline (510 k → 1 672 k) buys multi-AZ + cross-region + paid high-scale tail-sampling fidelity (25 %) + 100 M-series cardinality envelope.

Caveats:

- Hardware amortised over 5 years; refresh cycle assumed at year-5.
- Cloud comparator licence assumes no negotiation; enterprise contracts commonly receive 30-50 % discount. At 50 % discount, Datadog drops to ~ 2.2 M USD/yr, still 30 % more than oyatie paid high-scale.
- Ops cost includes ClickHouse + Tempo + Mimir + Loki lifecycle (3 SRE × 0.4 FTE = 1.2 FTE total) — this is the real hidden cost of self-hosted observability.

## Workload (f) — promotion-gate evidence query (specialised, oyatie-only)

This is a workload no cloud SaaS supports natively. Query: "give me every SLO breach for service X in environment Y over the last 30 d with multi-window burn rates and the runbook URLs that fired."

| Engine | Wall-clock (ms) | Result completeness |
|---|---:|---|
| oyatie observability (paid baseline) | 240 | Complete (every breach with timestamps, durations, burn rates, runbook URLs, ack status) |
| oyatie observability (paid high-scale) | 120 | Complete |
| Datadog SLOs API | 1 800 + manual stitching | Partial (Datadog SLO breach events lack the runbook URL + ack chain we keep in ClickHouse) |
| New Relic | (not natively supported) | Requires custom alerting workflow + manual collation |
| Honeycomb | (not natively supported) | Custom triggers + manual collation |
| Splunk | (not natively supported) | Custom dashboards + manual collation |

This is why ADR-0130 mandates the substrate. Promotion-gate evidence is a first-class query in oyatie and a third-class workaround everywhere else.

## Reproducibility

The benchmark harness is at `benchmarks/observability/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks observability \
    --workload sustained-ingest-1m-spans-sec \
    --tenant-class paid \
    --output ./benchmark-results.json
```

Cloud comparators (Datadog, New Relic, Honeycomb, Splunk) require valid `--cloud-credentials` for the relevant SaaS. Results live at `benchmarks/results/observability/<date>.csv` and are re-run weekly in CI to detect drift.
