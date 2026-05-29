---
doc_class: Benchmark
microservice: detection
benchmark_date: 2026-05-20
related_adrs: [ADR-0307, ADR-0308, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie detection vs Stripe Radar / AWS GuardDuty / Google Chronicle / Adyen RevenueProtect

Workloads measured: (a) streaming detection latency at 5 k events/sec, (b) feature-store lookup latency, (c) graph-traversal community detection on a 1 B-edge corpus, (d) fairness-audit pipeline throughput, (e) annual TCO at 20 k events/sec sustained.

Hardware (oyatie paid on-prem-connected cell_topology): 24× Flink TaskManagers + 16× Spark executors + JanusGraph 1.1 + ScyllaDB 6.2 across 3 AZs per cell.

Comparators measured against published latency figures (where available) + our independent test rig against their tenant-facing API endpoints (where benchmarking-with-tenant-consent applies).

## Workload (a) — streaming detection latency at 5 k events/sec, payment-fraud family

| Platform | p50 (ms) | p99 (ms) | Notes |
|---|---:|---:|---|
| oyatie detection paid dedicated-cloud (Flink + ONNX inline) | 28 | 142 | Rule + model serving on TaskManager; T4 GPU |
| oyatie detection paid on-prem-connected (multi-AZ Flink + L4 GPU + graph feature inline) | 32 | 168 | +20 ms from graph-feature inline lookup |
| Stripe Radar | ~ 80 (published) | ~ 200 (published) | Stripe's published "real-time risk evaluation" envelope |
| Adyen RevenueProtect | ~ 100 (published) | ~ 260 (published) | Adyen's published "real-time" envelope |
| AWS GuardDuty (not payment-fraud per se; threat detection) | ~ 5 min (analysis latency) | n/a | GuardDuty is forensic; not streaming-decision |
| Google Chronicle (SecOps) | ~ 30 s (rule-evaluation latency) | n/a | Chronicle is forensic; not streaming-decision |
| Sift Science | ~ 110 (published) | ~ 280 (published) | Sift's published "real-time" envelope |
| Featurespace ARIC | ~ 60 (published) | ~ 180 (published) | Featurespace ARIC (acquired by Visa 2024) |

Reading: oyatie's streaming decision latency is competitive with the leaders (Featurespace, Stripe Radar). GuardDuty and Chronicle play a different game (forensic + threat-hunting) and aren't directly comparable.

## Workload (b) — feature-store lookup latency (30-day-window cardholder features, 50 GiB state)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie detection (Flink RocksDB state) | 0.4 | 1.8 |
| oyatie detection (Pulsar log-compacted lookup) | 6 | 18 |
| oyatie detection (ClickHouse aggregation MV) | 12 | 38 |
| Feast (open-source feature store) | 8 | 32 |
| Tecton (commercial feature store) | 4 | 14 |
| AWS SageMaker Feature Store | 12 | 42 |
| Vertex AI Feature Store | 14 | 46 |

Reading: in-process RocksDB lookup (Flink's primary state path) is sub-millisecond. The cross-process Pulsar / ClickHouse paths are used for warm-cache / cross-job feature serving + are competitive with the commercial feature stores.

## Workload (c) — graph-traversal community detection on 1 B-edge corpus (Louvain modularity)

| Platform | Wall-clock (min) | Memory peak (GiB) | Notes |
|---|---:|---:|---|
| oyatie detection paid on-prem-connected (Spark GraphFrames + JanusGraph subgraph extract) | 38 | 480 | Per ADR-DET-001-streaming-vs-batch §"Spark for batch" |
| Neo4j Bloom + GDS Louvain | 52 | 620 | Single-server Neo4j; commercial |
| TigerGraph community detection | 32 | 560 | TigerGraph proprietary; commercial |
| AWS Neptune Streams + Gremlin | 84 | 720 | Slower; Gremlin's traversal not optimized for community detection |
| ArangoDB SmartGraph Louvain | 46 | 540 | Sharded community detection |
| Apache GraphX (Spark) | 62 | 800 | Open-source baseline; we use Spark GraphFrames over GraphX |

Reading: oyatie is competitive with TigerGraph (the commercial leader) and beats Neo4j + Neptune + ArangoDB on this workload. The advantage comes from Spark + GraphFrames over our JanusGraph extract — the extract is the bottleneck-eliminator (JanusGraph's BLP traversal is slow per-edge; we batch-extract into Spark, then Louvain there).

## Workload (d) — fairness-audit pipeline throughput (12 478 decisions, EEOC + EU AI Act)

| Platform | Wall-clock (min) | Notes |
|---|---:|---|
| oyatie detection fairness-audit (per IP-019) | 8 | Bootstrap CI 10 000 samples; protected-class stratification; 4/5ths rule |
| FairLearn (Microsoft open-source) | 22 | Single-machine; not pipeline-aware |
| AIF360 (IBM open-source) | 28 | Single-machine; not pipeline-aware |
| Holistic AI (commercial fairness platform) | 14 | Pipeline-aware; competitive |
| Saidot AI Compliance (EU AI Act-focused) | 18 | EU-AI-Act-shaped; not US-EEOC-focused |

Reading: oyatie's pipeline-aware fairness audit (the audit reads from the same feature store as production scoring, no manual extract step) is fastest at our envelope. Holistic AI is competitive; FairLearn and AIF360 are single-machine libraries that don't scale to our decision-count.

## Workload (e) — annual TCO at 20 k events/sec sustained, 8-family detection

| Platform | Compute (USD) | Storage (USD) | Licence (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|---:|
| oyatie detection paid on-prem-connected (on-prem 24 Flink + 16 Spark + JanusGraph fleet) | 624 000 | 192 000 | 0 (OSS substrate) | 372 000 (3 SRE × 0.4 FTE) | 1 188 000 |
| Stripe Radar (per-transaction priced at $0.05 per Radar-screened transaction) | 0 | 0 | 31 536 000 (20k events/sec × $0.05 × 31.5M sec/year) | 124 000 | 31 660 000 |
| Adyen RevenueProtect (per-transaction priced; ~ $0.03 typical) | 0 | 0 | 18 921 600 | 124 000 | 19 045 600 |
| Sift Science (commercial; ~ $0.04/event typical) | 0 | 0 | 25 228 800 | 124 000 | 25 352 800 |
| Featurespace ARIC (commercial; per-server license) | 0 | 0 | 4 800 000 (per their enterprise list) | 248 000 | 5 048 000 |
| AWS GuardDuty + Lambda + DynamoDB rule engine (DIY) | 1 240 000 | 480 000 | 0 | 496 000 (4 SRE × 0.4 FTE; DIY ops) | 2 216 000 |
| Google Chronicle SecOps (commercial threat-detection only) | n/a (Chronicle does not do payment-fraud / aml) | n/a | n/a | n/a | n/a (different domain) |

oyatie's edge vs Stripe Radar / Adyen / Sift is the absence of per-event SaaS pricing. The edge vs DIY-on-AWS is the integrated fairness audit + investigation case-management + sandbox replay — building these on AWS-primitives costs 4× the SRE FTE we estimate.

Caveats:

- These numbers assume 24×7 20 k events/sec utilisation. Bursty workloads (e.g., flash-sales) tilt managed services favourably (auto-scale-to-zero); on-prem doesn't shrink.
- Featurespace ARIC's $4.8M license is the per-server enterprise list price (8 servers; actual quoted prices vary).
- The DIY-on-AWS estimate assumes you build only the substrate, not the model registry + fairness audit + investigation case-management. Including those adds ~ $800k.

## Reproducibility

Benchmark harness at `benchmarks/detectionbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks detection \
    --workload streaming-payment-fraud-5krps \
    --tier oyatie-paid-onprem-connected \
    --output ./benchmark-results.json
```

External comparators require valid `--external-credentials`. Results at `benchmarks/results/detection/<date>.csv`; re-run weekly in CI.
