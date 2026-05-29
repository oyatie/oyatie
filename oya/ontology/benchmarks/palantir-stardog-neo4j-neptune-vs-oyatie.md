---
doc_class: Benchmark
microservice: ontology
benchmark_date: 2026-05-20
related_adrs: [ADR-0263, ADR-0329, ADR-0330, ADR-0331, ADR-0145]
doc_status: published
---

# Benchmarks — oyatie ontology vs Palantir Foundry Ontology vs Stardog vs Neo4j AuraDB vs Amazon Neptune

Workloads measured: (a) single-entity read latency, (b) 1-hop traversal, (c) 4-hop graph-pattern, (d) schema migration wall-clock, (e) GraphQL endpoint render, (f) annual TCO for 100M entities + 500M relationships.

Hardware (oyatie paid tenant_class on-prem): 6× ontology-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe), PostgreSQL 16.6 with Citus 13.0 + Apache AGE 1.5, 3 shards × 2 replicas (16 vCPU, 64 GiB, 3.84 TiB NVMe each).

Comparators: Palantir Foundry Cloud (eu-west region). Stardog 11.0 self-hosted on equivalent hardware. Neo4j AuraDB Enterprise (eu-west). Amazon Neptune (us-west-2, r6g.4xlarge × 3-replica).

## Workload (a) — single-entity read latency (primary key lookup, 100M entity table)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie ontology (paid tenant_class on-prem) | 8 | 22 |
| oyatie ontology (paid tenant_class multi-AZ) | 6 | 18 |
| Palantir Foundry Ontology | 28 | 84 |
| Stardog 11.0 | 14 | 42 |
| Neo4j AuraDB Enterprise | 12 | 38 |
| Amazon Neptune | 22 | 68 |

Reading: oyatie leads thanks to PostgreSQL + indexed primary key + Citus shard pushdown. Palantir's REST API has higher overhead (Foundry's auth + middleware stack adds ~ 20 ms).

## Workload (b) — 1-hop traversal (Customer → Order edges, 10k results)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie ontology (paid tenant_class on-prem) | 38 | 92 |
| oyatie ontology (paid tenant_class multi-AZ) | 28 | 72 |
| Palantir Foundry | 124 | 380 |
| Stardog | 68 | 184 |
| Neo4j AuraDB | 42 | 118 |
| Amazon Neptune | 84 | 240 |

Reading: Neo4j is competitive on 1-hop; oyatie + Neo4j both lead. Foundry's optimization is for analytical workloads not OLTP-style traversals.

## Workload (c) — 4-hop graph-pattern (Customer → Order → Shipment → Carrier → ServiceRegion)

| Platform | p50 (ms) | p99 (ms) | Notes |
|---|---:|---:|---|
| oyatie ontology (paid tenant_class on-prem) | 184 | 480 | Citus shard pushdown |
| oyatie ontology (paid tenant_class multi-AZ) | 124 | 320 | Multi-AZ parallel traversal |
| Palantir Foundry | 480 | 1 200 | Foundry's Compute Module |
| Stardog | 412 | 980 | RDF SPARQL query optimizer |
| Neo4j AuraDB | 184 | 460 | Native graph traversal |
| Amazon Neptune | 320 | 820 | OpenCypher backend |

Reading: oyatie + Neo4j are competitive at 4-hop. Above 6-7 hops Neo4j pulls ahead (native graph storage); we cap depth at 10 with cost-based optimizer.

## Workload (d) — schema migration wall-clock (add property to 1M-entity table)

| Platform | Wall-clock | Online (zero-downtime) |
|---|---:|---|
| oyatie ontology (paid tenant_class on-prem) | 14 s | Yes (PostgreSQL ALTER TABLE with default) |
| Palantir Foundry | 4-12 min | Yes (Foundry-managed) |
| Stardog | 28 s | Yes (RDF triple insert) |
| Neo4j AuraDB | 2-4 min | Yes (online property addition) |
| Amazon Neptune | 3-8 min | Yes (managed; depends on cluster size) |

Reading: oyatie leads on schema migration speed thanks to PostgreSQL's efficient ALTER TABLE with default (the new column is metadata-only until rewrites; reads return the default).

## Workload (e) — GraphQL endpoint render (3-level nested query, ~ 200 entities)

| Platform | p50 (ms) | p99 (ms) | Auto-generated? |
|---|---:|---:|---|
| oyatie ontology (paid tenant_class multi-AZ) | 84 | 220 | Yes (from schema) |
| Palantir Foundry GraphQL | 184 | 520 | Yes |
| Stardog GraphQL | 240 | 680 | Yes (via Stardog GraphQL) |
| Neo4j AuraDB + Neo4j GraphQL Library | 124 | 320 | Yes (after configuration) |
| Amazon Neptune (no native GraphQL) | N/A | N/A | No (must self-build) |

Reading: oyatie's auto-generated GraphQL is fastest because the resolvers use the Citus shard-aware planner directly. Neo4j with the GraphQL Library is competitive once configured.

## Workload (f) — annual TCO for 100M entities + 500M relationships + 50k queries/sec

| Platform | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie ontology (paid tenant_class self-hosted) | 480 000 | 0 | 248 000 (2 SRE × 0.4 FTE) | 728 000 |
| oyatie ontology (paid tenant_class multi-AZ) | 1 200 000 | 0 | 372 000 | 1 572 000 |
| Palantir Foundry (Enterprise contract) | 0 (managed) | 4 800 000 - 12 000 000 | 248 000 | 5 048 000 - 12 248 000 |
| Stardog Enterprise (self-host) | 480 000 | 360 000 (per-database licence) | 372 000 | 1 212 000 |
| Neo4j AuraDB Enterprise (managed) | 0 | 1 800 000 (Enterprise per-GB pricing) | 124 000 | 1 924 000 |
| Amazon Neptune (r6g.4xlarge × 3) | 384 000 | 0 | 248 000 | 632 000 |

Reading: oyatie paid tenant_class self-hosted matches Neptune on cost while providing far richer features (Cedar, audit-chain, multi-tenant native). Palantir is 7-15× more expensive but ships an end-to-end product surface beyond the ontology substrate. Stardog Enterprise is competitive on cost.

Caveats:

- Palantir's pricing is contract-dependent; the range reflects mid-market to enterprise contracts. The value proposition includes the full Foundry product (Pipeline Builder, Workshop, etc.) not just the ontology.
- Neo4j AuraDB Enterprise per-GB pricing scales with data volume; large datasets get more expensive.
- Neptune cost is amortised over 4-year r6g reserved instances.

## Reproducibility

The benchmark harness is at `benchmarks/ontologybench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks ontology \
    --workload 100m-entities-500m-relationships \
    --tenant-class paid \
    --comparators palantir,stardog,neo4j,neptune \
    --output ./benchmark-results.json
```

Cloud-comparator runs require valid Palantir / Neo4j AuraDB / Neptune credentials. Stardog comparator requires a self-host instance. Results live at `benchmarks/results/ontology/<date>.csv` and are re-run quarterly.
