---
doc_class: Benchmark
microservice: workflow-engine
benchmark_date: 2026-05-20
related_adrs: [ADR-0263, ADR-0329, ADR-0330, ADR-0331, ADR-0145]
doc_status: published
---

# Benchmarks — oyatie workflow-engine vs Temporal Cloud vs Camunda 8 vs Apache Airflow vs AWS Step Functions

Workloads measured: (a) workflow-start throughput, (b) step-execution latency, (c) signal delivery latency, (d) compensation wall-clock, (e) cross-AZ failover RTO, (f) annual TCO at 100 M workflow executions/year.

Hardware (oyatie paid tenant-class multi-AZ on-prem profile): 16× engine-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe) spread across 3 AZs, PostgreSQL 16.6 primary + 4 replicas (32 vCPU, 128 GiB, 3.84 TiB NVMe), Valkey 7.4 6-node, 25 GbE leaf-spine.

Comparators: Temporal Cloud tested via the EU region (latency-adjacent to our cell). Camunda 8 SaaS tested EU region. Airflow 2.10 self-hosted on equivalent hardware. AWS Step Functions tested in us-west-2.

## Workload (a) — workflow-start throughput (workflows/sec sustained)

| Platform | Sustained (workflows/sec) | Burst (workflows/sec, ≤ 60 s) | Notes |
|---|---:|---:|---|
| oyatie workflow-engine (paid tenant-class single-cell profile) | 100 | 500 | Per-tenant worker pool dispatch |
| oyatie workflow-engine (paid tenant-class multi-AZ profile) | 5 000 | 20 000 | Sharded worker pool across 3 AZs |
| oyatie workflow-engine (paid tenant-class sovereign-pack profile) | 10 000 | 50 000 | Pack-bound; sized for sovereign-pack throughput |
| Temporal Cloud (EU, "Production" tier) | 4 000 | 12 000 | Per Temporal Cloud SLA |
| Camunda 8 SaaS ("Enterprise" tier) | 1 200 | 4 800 | Per Camunda SLA |
| Apache Airflow 2.10 (Celery executor, 32 workers) | 800 | 2 400 | Self-hosted; DAG-shape only |
| AWS Step Functions Standard | 2 000 | (no published burst) | Per AWS quota |
| AWS Step Functions Express | 100 000 | 1 000 000 | Express is short-flow only; ≤ 5 min duration |

Reading: oyatie paid tenant-class multi-AZ profile matches Temporal Cloud at sustained throughput while providing tighter sovereignty + audit integration. AWS Step Functions Express is the highest throughput but caps workflow duration; not comparable for long-running sagas.

## Workload (b) — step-execution latency (engine overhead only, excluding user code)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie workflow-engine (paid tenant-class multi-AZ profile) | 38 | 92 |
| Temporal Cloud (EU) | 64 | 184 |
| Camunda 8 SaaS | 142 | 420 |
| Apache Airflow 2.10 | 1 800 | 6 400 |
| AWS Step Functions Standard | 28 | 88 |
| AWS Step Functions Express | 16 | 48 |

Reading: oyatie is competitive with Step Functions Standard. Airflow is orders of magnitude slower because DAGs require scheduler-tick to advance (1 s tick default; longer if loaded). Step Functions Express is the fastest but lacks durability for long-running flows.

## Workload (c) — signal delivery latency (external signal sent → workflow resumes)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie workflow-engine (paid tenant-class multi-AZ profile) | 84 | 220 |
| Temporal Cloud (EU) | 124 | 380 |
| Camunda 8 SaaS | 210 | 620 |
| Apache Airflow | N/A (no native signal model) | N/A |
| AWS Step Functions (callback tokens) | 1 200 | 4 800 |

Reading: oyatie leads on signal delivery. Step Functions callback tokens go through API Gateway + Lambda + Step Functions state machine; the multi-hop adds latency. Airflow doesn't natively support external signaling.

## Workload (d) — compensation wall-clock (4-step saga, all forward succeed except final, compensation runs in reverse)

| Platform | Forward path (s) | Compensation path (s) | Total (s) |
|---|---:|---:|---:|
| oyatie workflow-engine (paid tenant-class multi-AZ profile) | 12.4 | 4.8 | 17.2 |
| Temporal Cloud (EU) | 14.2 | 6.1 | 20.3 |
| Camunda 8 SaaS | 28.4 | 14.8 | 43.2 |
| AWS Step Functions Standard (Catch + Retry) | 18.6 | 8.2 | 26.8 |

Reading: oyatie's compensation runs in parallel where dependencies permit (reverse-order with parallel-where-safe); Temporal + Step Functions are strictly sequential.

## Workload (e) — cross-AZ failover RTO (AZ-down event → workflows resume in surviving AZs)

| Platform | RTO (s) | RPO (s) | In-flight workflow loss |
|---|---:|---:|---:|
| oyatie workflow-engine (paid tenant-class multi-AZ active-active profile) | 28 | 0 | 0 % |
| Temporal Cloud (EU, multi-region) | 60-180 | 0-60 | 0 % (Temporal Cloud High Availability) |
| Camunda 8 SaaS (multi-region) | 60-300 | 0-60 | 0 % |
| Apache Airflow (self-host, no multi-AZ) | (self-build; typically 5-30 min) | (varies) | (variable) |
| AWS Step Functions (in-region only) | (within-region only; failure of region = total loss until region recovers) | N/A | 0 % within-region; total cross-region |

Reading: oyatie's 28 s RTO is best-in-class. Temporal Cloud's High Availability tier is within an order of magnitude. Step Functions is within-region only — region-down is total loss until AWS recovers.

## Workload (f) — annual TCO for 100M workflow executions/year + 1k concurrent + 50% with saga compensation

Assumptions: 100M executions/year = ~ 3 200 workflows/sec average × 8 events/workflow = 25 600 events/sec average. 1 000 concurrent in-flight. 50 % use saga compensation. Average workflow lifetime ~ 60 s.

| Platform | Hardware/Compute (USD) | Per-execution (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|---:|
| oyatie workflow-engine (paid tenant-class multi-AZ self-hosted profile) | 1 200 000 (16 nodes × 3 AZs + PG + Valkey) | 0 | 0 | 372 000 (3 SRE × 0.4 FTE) | 1 572 000 |
| oyatie workflow-engine (paid tenant-class single-cell profile) | 480 000 | 0 | 0 | 248 000 (2 SRE × 0.4 FTE) | 728 000 |
| Temporal Cloud (Production EU, 100M action/year) | 0 | ~ 1.85M (per Temporal Cloud per-action pricing) | 0 | 124 000 (1 SRE) | 1 974 000 |
| Camunda 8 SaaS (Enterprise tier) | 0 | (managed) | 1 600 000 (per-process-instance pricing) | 124 000 | 1 724 000 |
| Apache Airflow (self-host, 32 workers) | 240 000 | 0 | 0 | 620 000 (5 SRE × 0.4 FTE; Airflow ops complexity) | 860 000 |
| AWS Step Functions Standard (100M state-transitions) | 0 | ~ 2 500 000 (per-state-transition pricing) | 0 | 124 000 | 2 624 000 |
| AWS Step Functions Express | 0 | ~ 250 000 (much cheaper; limited features) | 0 | 124 000 | 374 000 |

Reading: oyatie paid tenant-class single-cell profile beats Temporal Cloud + Camunda + Step Functions Standard on TCO at this scale + provides sovereign-pack + audit-chain integration. Step Functions Express is cheapest for short workflows but lacks durability. Airflow self-host is cost-competitive but has higher ops surface + lacks durable-function semantics.

Caveats:

- Temporal Cloud per-action pricing depends on contract; enterprise-level deals get significant discounts. The listed price is 2026-Q2 list pricing.
- Camunda 8 per-process-instance pricing is similar; enterprise discounts apply.
- The 5-SRE × 0.4 FTE for Airflow reflects the operational complexity (Celery worker management, DAG version control, scheduler tuning).

## Reproducibility

The benchmark harness lives at `benchmarks/workflowbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks workflow-engine \
    --workload 100m-executions-per-year \
    --tenant-class paid \
    --comparators temporal-cloud,step-functions,airflow \
    --output ./benchmark-results.json
```

Cloud comparators require valid Temporal Cloud + AWS + Camunda credentials. Results live at `benchmarks/results/workflow-engine/<date>.csv` and are re-run quarterly.
