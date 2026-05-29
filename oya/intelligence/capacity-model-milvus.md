# Foundry Milvus — Capacity Model

**Authority:** ADR-0192, hyperscaler-architecture-invariants.json (4-INV overlay)
**Last reviewed:** 2026-05-18
**Numbers:** Concrete. No aspirational targets.

## Production-validation evidence (Milvus at billion scale)

Public references documenting Milvus at the 1B+ vector scale (validation for ADR-0192's Phase-2 trigger condition):

| Operator | Scale | Public reference |
|---|---|---|
| Zilliz Cloud (Milvus commercial parent) | Multi-customer deployments at 10B+ vectors | Zilliz public case studies |
| NVIDIA RAFT reference customer | Billion-scale GPU-CAGRA | NVIDIA RAPIDS RAFT public docs |
| Salesforce Einstein vector store | Customer-scale embedding retrieval | Milvus public case studies |
| eBay | Item-similarity index at hundred-million-to-billion scale | Milvus public case studies |
| IKEA Marketplace | Product-similarity recommendation | Milvus public case studies |

The disaggregated four-plane architecture (per ADR-0192) is what enables billion-scale: coord planes scale independently from worker planes; storage plane (object-store + log broker) decouples from compute. This is the same architectural shape NVIDIA / Cloudflare / Uber use for their internal vector substrates.

## Per-cluster (per-cell) concrete ceilings

| Dimension | Steady-state target | Hard ceiling | Trigger above hard ceiling |
|---|---|---|---|
| Vectors per cluster | 100 M | 1 B | **Promotes ADR-0192 Phase-2 trigger** — move to in-house oya-vector-store |
| Collections per cluster | 5,000 | 20,000 | Shard tenants across multiple Milvus clusters per cell |
| Tenants per cluster | 5,000 | 10,000 | Shard cell-locally |
| QPS per cluster | 10 K | 50 K | Add query nodes; consider GPU CAGRA |
| Search p99 latency (HNSW hot) | < 30 ms | 50 ms | Tune ef_search; review M parameter |
| Search p99 latency (DiskANN cold) | < 200 ms | 500 ms | Move hotter partition to HNSW |
| Ingest rate (vectors/sec/cluster) | 10 K | 50 K | GPU acceleration via IP-095 |
| Index build queue (sealed segments) | < 100 | 5 K | Add index nodes; consider GPU |
| Coordinator plane (per-coord pair) | 1 active + 1 passive | always | Pre-provisioned; not a scaling lever |
| Worker plane: query nodes | 6 | 24 | Linear scale-out |
| Worker plane: data nodes | 4 | 16 | Linear scale-out |
| Worker plane: index nodes | 2 | 16 (GPU optional) | GPU scaling per IP-095 |
| etcd quorum nodes | 3 | always 3 or 5 | Static; not a scaling lever |
| Pulsar broker count | 3 | 9 | Add brokers as ingest grows |

## HNSW parameter tradeoffs (concrete)

Per ADR-0192 §"Index types — pinned per workload class":

| Parameter | Value | Cost | Benefit |
|---|---|---|---|
| M (graph degree) | 16 | ~50 GB index per 100M vectors at 1024 dim | Recall ≥ 0.95 |
| ef_construction | 200 | Build time ~1× per increment | Recall +2-3% per +100 |
| ef_search | 64 | Query latency ~linear in ef_search | Recall +1-2% per +32 |

The pinned values balance recall vs cost; per-collection override is permitted at collection-creation time only (no live tuning to avoid query-plan flapping).

## DiskANN cold-tier trigger

When a collection's vector count exceeds 100 M and the recall-vs-latency profile shifts (older vectors queried < 10% of the time), the collection migrates from HNSW to DiskANN. Per ADR-0192 §"Index types":

- DiskANN p99 latency ≤ 200 ms (vs HNSW ≤ 30 ms) — accept for cold-tier workload.
- DiskANN memory footprint ~1/10th of HNSW for same recall — enables billion-scale.

## GPU acceleration (IP-095) — concrete trigger

GPU CAGRA index build engages when:

- Ingest rate sustained > 30 K vectors/sec for 7 consecutive days, OR
- Index build queue exceeds 1 K sealed segments for 24 hours, OR
- Cell's CPU index-build capacity at >80% utilization for 7 days.

Each trigger is value-anchored, not date-anchored.

## Concrete Phase-2 in-house trigger (per ADR-0192 §"In-house roadmap")

Any one of the following PROMOTES the `oya-vector-store-server` in-house lane to active development:

1. ≥ 1 × 10⁹ vectors per cluster sustained for ≥ 30 days. **(VALUE-ANCHORED, not date-anchored.)**
2. RAG retrieval p99 latency budget breached (> 30 ms hot path) for 7 consecutive days despite tuning.
3. Milvus license posture changes (relicense from Apache-2.0 / CNCF-graduated to a more restrictive license — analogous to the 2024 Valkey relicense event).
4. Cross-cell residency requirements exceed Milvus's multi-cell capability.

Date "Q3 2027" in ADR-0192 §"In-house roadmap" is a planning anchor; the actual gate is value-anchored above.

## Per-tenant resource quotas (concrete, project from ADR-0155 + B2bTenantTier)

| Tier | QPS/collection | upsertRate (rows/sec) | replica.num | Max collections per tenant |
|---|---|---|---|---|
| Trial | 50 | 100 | 1 | 5 |
| Starter | 500 | 1,000 | 2 | 50 |
| Growth | 5,000 | 10,000 | 2 | 500 |
| Enterprise | 50,000 | 100,000 | 3 | unlimited |

## 4-INV overlay

| Invariant | Status for Milvus extension | Evidence |
|---|---|---|
| INV-1: Idempotent writes | YES — Milvus 2.6 native upsert by `source_id` | IP-093 |
| INV-2: OTel trace propagation | YES — adapter emits spans tagged with `tenant_id, collection, latency_ms` | IP-094 + ADR-0186 |
| INV-3: Ontology projection | YES — per-tenant embeddings flow through canonical outbox CDC | IP-093 + ADR-0145 |
| INV-4: Per-tenant resource quotas | YES — Milvus QUOTA per ADR-0155 + per-tier matrix above | IP-092 |

## References

- ADR-0192, ADR-0155, ADR-0026 AI substrate, ADR-0145 inter-microservice comm.
- specs/hyperscaler-architecture-invariants.json
- Milvus 2.6 release blog: "Affordable Vector Search at Billion Scale" — Zilliz, 2026-03.
- NVIDIA RAPIDS RAFT integration docs — billion-scale GPU CAGRA validation.
