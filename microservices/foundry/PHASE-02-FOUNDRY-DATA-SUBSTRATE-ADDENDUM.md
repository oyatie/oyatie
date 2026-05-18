# PHASE-02 Foundry Data Substrate — Addendum

**Authored:** 2026-05-18
**Authority:** ADR-0192 (vector database canonical Milvus), ADR-0136 (foundry-as-single-microservice), ADR-0131 (per-microservice flat layout).
**Status:** Drafting — promotes alongside IP-091..IP-097 acceptance.

## Why an addendum

The Foundry µservice's existing `PHASE-01-FOUNDRY-FOUNDATION.md` predates the Milvus canonical decision (ADR-0192 promoted 2026-05-18). The 7 Milvus IPs (091..097) introduced 2026-05-18 by Fix-R require a new phase entry — **PHASE-02 Foundry Data Substrate** — to anchor them inside the M01 milestone sequencing.

This addendum is the temporary anchor. When the next PHASE consolidation lands, this file is folded into `PHASE-02-FOUNDRY-DATA-SUBSTRATE.md` (canonical) and retired.

## Phase scope

PHASE-02 stands up the Foundry-owned Milvus 2.6.x disaggregated cluster + its tenant lifecycle + ingest pipeline + adapter crate, and wires GPU acceleration, backup, and cross-region replication. The phase exit gate is the Milvus search-latency SLO unburned over 30 consecutive days with all 7 IPs Accepted.

## Phase work items (IP traceability)

### §"Data substrate bootstrap — first work item" (IP-091)

Cluster IaC — Helm chart, per-pack overlays, namespace bootstrap, smoke test. Loads ServiceMonitor + PrometheusRule. **All other work items in this phase depend on IP-091.**

### §"Tenant lifecycle reconciliation" (IP-092)

Per-tenant collection + partition + quota controller. Listens for `tenant.onboarded` / `tenant.offboarded` / `tenant.tier_changed` events from the tenancy µservice. Implements the DSR cascade (offboard drops all per-tenant collections + emits proof-of-erasure).

### §"Embedding-to-vector ingest path" (IP-093)

Pulsar consumer subscribing to `oya.foundry.embeddings.outbox`; per-record idempotency by `source_id`; per-tenant backpressure; dimension-mismatch quarantine. Per-pack model binding (KoSimCSE / bge-large / bge-m3 per ADR-0026).

### §"Adapter crate + recall benchmark" (IP-094)

`oya-shared-vector-store-milvus-adapter` — the canonical adapter implementing `VectorStore` against Milvus 2.6.x via tonic. Pins HNSW parameters per workload class. MS-MARCO recall ≥ 0.95; p99 latency ≤ 30ms.

### §"GPU acceleration overlay" (IP-095, opt-in)

Per-cell opt-in GPU pool hosting Milvus query + index nodes. GPU_CAGRA via NVIDIA RAFT. Gated on cost-budget approval (audit-chain `capacity.gpu.budget.approved`). Pool shared with robotics-vision-speech sub-substrate per ADR-0027.

### §"Backup + drill cadence" (IP-096)

Milvus backup tool emits per-collection backups to SeaweedFS S3-compat. Daily incremental + weekly full + quarterly drill. RPO ≤ 24h, RTO ≤ 30min.

### §"Residency-gated replication" (IP-097)

Cross-region replication governed by per-tenant `ResidencyClass` (StrictKr / StrictEu / KrWithUsFailover / Global). Cedar policy fragments per tenant; network-policy egress denial; quarterly residency drill.

## Phase exit gate

- All 7 IPs at `Accepted`.
- Milvus search-latency SLO (`microservices/foundry/slos/milvus-search-latency.openslo.yaml`) unburned over 30 consecutive days.
- Milvus ingest-lag SLO (`microservices/foundry/slos/milvus-ingest-lag.openslo.yaml`) unburned over 30 consecutive days.
- First quarterly backup drill complete with RTO + RPO targets met.
- First quarterly residency drill complete with no findings.
- Foundry-oncall has drilled all 4 Milvus runbooks (`milvus.md`, `milvus-restore.md`, `milvus-tenant-quota.md`, `milvus-residency-incident.md`).

## Dependency graph

```
IP-091 (cluster IaC)
  ├── IP-092 (tenant bootstrap)
  ├── IP-094 (adapter crate)
  │     └── IP-093 (ingest pipeline)
  │           depends on IP-091, IP-092, IP-094
  ├── IP-095 (GPU overlay, opt-in)
  │     depends on IP-091, IP-094
  ├── IP-096 (backup + drill)
  │     depends on IP-091
  └── IP-097 (cross-region replication)
        depends on IP-091, IP-096
```

## References

- ADR-0192 — vector database canonical Milvus.
- ADR-0131 — per-microservice flat layout.
- ADR-0136 — foundry-as-single-microservice.
- ADR-0184 — storage tier layering.
- ADR-0145 — inter-microservice communication.
- ADR-0152 — RPO/RTO targets.
- ADR-0241 — DR business-continuity portfolio.
- ADR-0049 — cross-region replication.
- ADR-0010 — regional packs.
- ADR-0026 — AI substrate.
- ADR-0027 — robotics-vision-speech shared GPU pool.
- ADR-0001 — cohesion authority (cost attribution).
