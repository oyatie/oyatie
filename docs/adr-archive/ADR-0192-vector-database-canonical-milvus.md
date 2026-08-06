---
id: ADR-0192
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-foundry, axis-ontology
date: 2026-05-18
owner: council-architecture
supersedes: [ADR-0046]
superseded_by: [ADR-709]
related: [ADR-0026, ADR-0030, ADR-0038, ADR-0043-secrets-management-openbao-and-hsm-per-cell, ADR-0049, ADR-0131-per-microservice-flat-layout, ADR-0145, ADR-0153, ADR-0155, ADR-0184, ADR-0186, ADR-0193, ADR-0194, ADR-0195]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0192 — Vector database canonical: Milvus disaggregated cluster; pgvector adapter only for ≤10M-vector tenants

## Status

Accepted (2026-05-18). Mandates **Milvus 2.6.x** as the canonical vector-database substrate fleet-wide for any workload at hyperscaler-bar scale (>10M vectors per tenant or per index). Supersedes ADR-0046 (which scoped pgvector as the day-1 canonical and an in-house Rust HNSW/IVF as the billion-scale long-horizon target).

User-directive anchor (2026-05-18): Milvus chosen over Qdrant, Weaviate, Pinecone, and in-house engines as the canonical vector store for semantic retrieval, agent memory, similarity matching, and embedding pipelines across all 32 µservices.

## Context

Per ADR-0145 (Invariant 3 — every canonical entity projects into Ontology) and ADR-0026 (in-house AI substrate roadmap), oyatie's AI workloads — Foundry agent retrieval, Search RAG, per-µservice semantic matching, Workflow Studio node recommendation, agent cross-session memory — all rely on dense-vector ANN at fleet scale. The pgvector path scoped in ADR-0046 holds up to ~10–100M vectors per index but breaks down on:

1. **Disaggregated scale.** pgvector lives in OLTP Postgres; once vector count + dimension * fp32 exceeds OLTP cache, ANN search competes with transactional read load and ruins p99 latency for both workloads.
2. **GPU acceleration.** pgvector has no path to CUDA/GPU index build or search; the AI substrate (ADR-0026) requires GPU-accelerated index build at ingest peak.
3. **Disk-tiered cold storage.** pgvector indices live in Postgres heap; cold-tier ANN (DiskANN) is not a first-class index type.
4. **Coordinator-free multi-tenancy at hyperscaler scale.** Per-tenant index isolation in pgvector requires partial indices or schema-per-tenant; both schemes thrash Postgres catalog at >5K tenants.

Hyperscaler practice for vector retrieval at oyatie's target scale:

- **NVIDIA** — Milvus is the named vector substrate in the NVIDIA AI Enterprise reference stack for RAG and embedding retrieval; Milvus integrates first-class with NVIDIA RAPIDS RAFT (GPU index build + GPU ANN search).
- **Shopify** — moved off pgvector to a purpose-built vector substrate once their RAG workload crossed the ~100M-vector ceiling per cell.
- **Uber** — Michelangelo's feature/embedding retrieval is a purpose-built vector engine that mirrors Milvus's disaggregated shape (compute/storage/coord separation).
- **Cloudflare** — Vectorize (their managed vector DB) implements the same coordinator-free per-tenant collection model that Milvus exposes natively.

Anti-patterns this ADR forecloses:

1. Per-µservice vector engine choice (one µservice on Qdrant, another on pgvector, another on Pinecone) — license sprawl, per-engine DSR cascade integration, no per-tenant residency uniformity.
2. pgvector as primary at multi-hundred-M scale — OLTP collision; no GPU index; no cold tier.
3. Pinecone or any closed-source managed service — sovereignty failure (data leaves cells), license posture impossible to audit per ADR-0014.

## Decision

Oyatie adopts **Milvus 2.6.x** (latest stable: 2.6.15 as of 2026-05-18; Apache-2.0; CNCF Graduated) as the canonical vector-database substrate fleet-wide. Milvus runs as a disaggregated cluster owned by the `foundry` µservice (since embedding retrieval is a Foundry AI-workload primitive) and is consumed by all µservices through the `oya-shared-vector-store-kernel` port.

### Cluster shape — disaggregated, four-plane

Milvus 2.6.x ships a four-plane disaggregated architecture:

| Plane | Components | Responsibility | Scale-out unit |
|---|---|---|---|
| Access | Proxy nodes | gRPC/REST ingress, request routing, auth, rate-limit | per-tenant QPS |
| Coordinator | Root coord, query coord, data coord, index coord | Cluster metadata, query planning, data shard placement, index lifecycle | active/passive pair |
| Worker | Query nodes, data nodes, index nodes | Search execution, segment write/flush, index build | independent per workload |
| Storage | MetaStore (etcd or Postgres-compat), Log broker (Pulsar 4.2 or Kafka), Object store (SeaweedFS S3-compat per Fix-S) | Cluster state, WAL, vector segments | persistent |

- **MetaStore.** etcd 3.5.x for cluster metadata. (Milvus 2.6 also supports Postgres-compat MetaStore via TiKV; etcd is the canonical default for oyatie's deployment.)
- **Log broker.** Apache Pulsar 4.2.x (canonical; matches the existing log-broker substrate). Kafka is supported but not canonical.
- **Object store.** SeaweedFS (S3-compat) per the storage Fix-S; per-cell scoped.

### Index types — pinned per workload class

| Workload | Index | Recall target | Latency p99 |
|---|---|---|---|
| Hot RAG retrieval (top-K, K≤50) | **HNSW** M=16 ef_construction=200 ef_search=64 | ≥0.95 | <30ms |
| Bulk similarity match (top-K, K≤500) | **IVF_FLAT** nlist=4096 nprobe=32 | ≥0.90 | <100ms |
| Memory-constrained (per-cell ≤ 16GiB RAM) | **IVF_SQ8** scalar-quantized | ≥0.88 | <80ms |
| Cold-tier billion-scale | **DiskANN** | ≥0.95 | <200ms (disk-served) |
| GPU-accelerated index build | **GPU_CAGRA** (NVIDIA RAFT) | ≥0.95 | build 10× CPU equivalent |

Per-tenant per-µservice index type is selected at collection creation via manifest field `data.vector_store.collections[].index_type`.

### Multi-tenancy isolation

Milvus 2.6 supports three nested isolation primitives:

1. **Database.** Top-level namespace — one Milvus `database` per µservice.
2. **Collection.** Per-tenant-per-domain — naming pattern `tenant_{tenant_id}__{domain}` (e.g., `tenant_ten_acme__rag_corpus`, `tenant_ten_acme__agent_memory`).
3. **Partition.** Per-tenant per data-class slice — e.g., `partition_public`, `partition_tenant_private`, `partition_regulated`.

Per-tenant resource quotas (ADR-0155) project into Milvus via per-database / per-collection limits: `db.replica.num`, `db.resource_groups`, `quota.dml.upsertRate`, `quota.dql.searchRate`. Cross-tenant query is forbidden at the kernel layer (`MilvusVectorStore::search` requires a `TenantId` and emits the per-tenant collection name; no fall-through code path can query a different tenant's collection).

### Embedding pipeline integration

Foundry-emitted vectors flow into Milvus via the canonical pipeline:

1. **Embedding source.** Foundry's embedding-model adapter (ADR-0026) emits dense vectors with `(tenant_id, source_id, embedding, data_class, dimension, model_id, inserted_at)`.
2. **Outbox.** Vectors land in the µservice's transactional outbox table (ADR-0153 outbox pattern); the canonical CDC pipeline projects them into Milvus.
3. **Idempotency.** Per-`source_id` upsert; Milvus 2.6 native upsert is used (no read-then-write race).
4. **DSR cascade.** Per-row delete by `source_id`; per-tenant tombstone on tenant offboard; proof-of-erasure per ADR-0038.

### pgvector — degraded fallback only for ≤10M-vector tenants

pgvector (PostgreSQL `vector` extension, PostgreSQL License — clean) remains permitted as the **embedded-tier** vector store for µservices whose per-tenant vector count is ≤10M AND whose ANN latency budget tolerates Postgres OLTP-side execution. Above 10M vectors per tenant per index, the kernel's `pgvector` adapter MUST delegate to Milvus. The `oya-check-vector-store-discipline` lane (advisory, ADR-0135 aspirational at slice gate; BLOCKER post-PR-#145 wave) enforces this ceiling.

### Sizing model (target capacity)

Per-cell sizing target for the largest tenant class (10–100M vector range):

- 1B vectors per cell across all tenants.
- HNSW M=16 ef_construction=200 → ~50GB index per 100M vectors (1024 dim, fp32).
- Worker plane: 6 query nodes (3 search + 3 index), 4 data nodes, 2 index nodes per cell.
- Coordinator plane: 1 active + 1 passive per coord type (4 active coords; 4 passive).
- Log broker: Pulsar with 3-broker minimum; per-collection topic.
- Object store: SeaweedFS volume per data-class with cross-pack residency overlay per ADR-0049.

### GPU acceleration (optional per cell)

- Milvus 2.6 supports `GPU_CAGRA` and `GPU_IVF_FLAT` via NVIDIA RAFT (CUDA 12.x).
- Per-cell GPU node pool optional; gated by `cell-meta` capacity-planning model.
- CPU-only deployment is the canonical default; GPU is an opt-in performance lane for cells whose embedding ingest peak exceeds CPU index-build capacity.

### Backup and disaster recovery

- **Milvus backup tool** (Apache-2.0; ships with Milvus) emits per-collection backup to S3-compat (SeaweedFS).
- Backup cadence: daily incremental + weekly full per ADR-0152 RPO/RTO canonical (RPO ≤ 24h for vector retrieval; vectors are re-derivable from canonical entities via re-embedding if backup loss exceeds RPO).
- Cross-cell DR per ADR-0241-dr-business-continuity-portfolio-policy.

### Secrets — OpenBao SecretReference

- Milvus root credentials live in OpenBao at `secret/foundry/milvus/root-password` per ADR-0043.
- Per-µservice service-account credentials at `secret/<ms>/milvus/api-token`; rotated on the cell's 90-day schedule.
- The `MilvusVectorStore` adapter reads via the canonical OpenBao SecretReference type; no plaintext credentials in Helm values.

### Helm chart — `microservices/foundry/iac/helm/milvus/`

Canonical Helm chart at `microservices/foundry/iac/helm/milvus/`:

- `Chart.yaml` — `appVersion: "2.6.15"`, dependencies on Pulsar, etcd, SeaweedFS.
- `values.yaml` — disaggregated cluster shape with replica counts.
- `templates/` — Milvus operator CRs (or upstream Bitnami/Zilliz chart consumption).
- Per-pack overlays at `microservices/foundry/iac/kustomize/overlays/pack-kr/milvus/` and `pack-eu/milvus/`.

### Kustomize per-tenant collection bootstrap

Per-tenant collection bootstrap lives at `microservices/foundry/iac/kustomize/components/milvus-collections/`. Per-tenant Kustomize patches emit Milvus CRs declaring:

- `MilvusDatabase` per µservice.
- `MilvusCollection` per tenant per data-domain.
- `MilvusPartition` per data-class.

## Alternatives considered

### (a) **CHOSEN: Milvus 2.6.x disaggregated cluster, foundry-owned**

- **Pros:** Apache-2.0; CNCF Graduated (mature governance); disaggregated architecture matches NVIDIA / Cloudflare / Shopify reference stacks; native multi-tenancy via database/collection/partition; GPU support via RAFT; first-class DiskANN cold tier; backup tool ships with the product; large open-source community; pure Rust SDK exists (`milvus-sdk-rust`) with gRPC fallback.
- **Cons:** four-plane operational surface (coord + access + worker + storage); etcd + Pulsar + SeaweedFS dependency stack; CPU-only deployment still requires non-trivial cell capacity.
- **Accepted.**

### (b) Qdrant (Rust-native, single-binary) — REJECTED at hyperscaler scale

- **Pros:** Rust-native (lower operator-skill surface for oyatie's Rust-first fleet); single-binary deployment; Apache-2.0; simpler ops; built-in filtered ANN; first-class HNSW.
- **Cons:** scale ceiling materially lower than Milvus at the 100M+ vector tier per cell — Qdrant's coordinator-free design is a virtue at the 1–10M scale and a constraint at the 100M+ scale (cluster mode exists but is younger than Milvus's multi-coord model). Smaller community for the disaggregated-plane operational shape oyatie needs. No first-class DiskANN equivalent for the cold tier.
- **Rejected** at the hyperscaler scale ceiling. Qdrant remains permitted as an opt-in adapter under the `oya-shared-vector-store-kernel` port for µservices whose vector count stays under 10M and whose operational profile favors a single-binary substrate; the adapter is not the canonical primary.

### (c) pgvector as primary at >10M vectors — REJECTED

- **Pros:** lives in OLTP Postgres (already deployed); PostgreSQL License (clean); per-tenant via partial index; DSR cascade via standard SQL delete.
- **Cons:** practical ceiling around 10M vectors per index per cell with acceptable p99; OLTP collision at index build / index rebuild; no GPU index; no cold tier (no DiskANN equivalent); per-tenant partial index thrashes the Postgres catalog at >5K tenants per cell.
- **Rejected** at the >10M scale. pgvector remains permitted for ≤10M per tenant via the embedded-tier path; the kernel enforces the ceiling.

### (d) Weaviate (Go) — REJECTED

- **Pros:** mature; built-in module pipeline; BSD 3-Clause.
- **Cons:** Go runtime adds operator-skill surface separate from oyatie's Rust/JVM-light fleet; modules ecosystem encourages tight coupling between embedding-model and vector engine (anti-pattern per ADR-0145 — embedding source lives in Foundry, vector store is a substrate); smaller community than Milvus at the disaggregated scale tier; per-tenant isolation requires class-per-tenant which doesn't scale to >10K tenants.
- **Rejected** on operator-skill alignment + per-tenant scaling profile.

### (e) Pinecone (managed SaaS) — REJECTED

- **Pros:** zero ops; first-class managed scaling.
- **Cons:** closed-source; vendor lock-in; data leaves cells (sovereignty failure per ADR-0049); per-pack residency (KR / EU strict residency) impossible; license posture impossible to audit per ADR-0014.
- **Rejected** on sovereignty + lock-in.

### (f) In-house Rust HNSW/IVF (long-horizon path from ADR-0046) — DEFERRED

- **Pros:** zero external dependency; in-house license; tuneable to oyatie's exact workload shape.
- **Cons:** very large engineering investment; Milvus already shipped at the scale oyatie needs; building an in-house substrate when an Apache-2.0 CNCF-graduated substrate exists is gold-plating per the build-vs-buy policy (ADR-0014).
- **Deferred** indefinitely. Milvus 2.6.x is canonical; the in-house option from ADR-0046 is closed by this ADR's supersession; re-opening requires a future ADR with concrete workload data demonstrating Milvus inadequacy.

### (g) FAISS / Annoy / NMSLIB as primary — REJECTED

- These are libraries, not databases. They lack multi-tenancy, persistence, replication, and a server protocol. They remain permitted as in-process adapters behind the kernel port for research/eval workloads (per ADR-0046's adapter posture).

## Consequences

### Positive

1. **One vector-database substrate fleet-wide.** Every µservice retrieves through the `MilvusVectorStore` adapter; no per-µservice engine drift.
2. **Hyperscaler-bar scale.** 1B vectors per cell sizing target; disaggregated planes scale independently.
3. **GPU-accelerated index build.** RAFT integration handles ingest peak when CPU index build is insufficient.
4. **Apache-2.0 + CNCF Graduated.** Permissive license; CNCF governance; broad operator-skill availability.
5. **Per-tenant residency uniform across µservices.** Per-cell Milvus + per-pack Kustomize overlay deliver KR / EU residency without per-µservice ad-hoc work.
6. **Per-row DSR cascade native.** Milvus 2.6 upsert + delete primitives map cleanly to ADR-0038 proof-of-erasure.

### Negative

1. **Four-plane operational surface.** Mitigation: canonical Helm chart at `microservices/foundry/iac/helm/milvus/`; runbook at `microservices/foundry/runbooks/milvus.md`; ops-sre-reliability operates Milvus as a substrate service.
2. **Pulsar + etcd + SeaweedFS dependency stack.** Mitigation: each is already deployed for adjacent substrate purposes (Pulsar = event backbone per ADR-0005; etcd = Kubernetes substrate; SeaweedFS = object storage per Fix-S); Milvus reuses, not introduces.
3. **`milvus-sdk-rust` (v0.1.0) is young.** Mitigation: the `MilvusVectorStore` adapter is gRPC-first (Milvus's tonic-generated gRPC client is the canonical wire protocol); the high-level SDK is an optional layer over gRPC. The kernel binds to the gRPC protobuf surface, not the SDK convenience layer, so adapter stability does not depend on SDK maturity.

### Operational

1. Per-µservice manifest declares `data.vector_store.enabled: true` when the µservice retrieves vectors; `data.vector_store.collections[]` declares per-collection schema (dim, index type, partitions).
2. The `oya-check-vector-store-discipline` lane is advisory at PR-#145 ship date; flips to BLOCKER post-wave when all µservices have migrated.
3. SLO: vector-retrieval p99 ≤ 30ms for hot RAG path (HNSW), ≤ 100ms for bulk similarity (IVF), ≤ 200ms for cold-tier (DiskANN). Authored at `microservices/foundry/slos/milvus.openslo.yaml`.
4. Capacity: per-cell sizing tracked via `microservices/foundry/capacity-model.md` Milvus section.
5. Per-tenant offboard: tenant deletion cascades to Milvus collection drop + partition tombstone + proof-of-erasure emission per ADR-0038.

## In-house roadmap

Per the in-house tech stack policy (user directive 2026-05-18) — "wherever possible, support in-house tech stack like AWS / Google / Microsoft / Oracle do" — Milvus is **vendor-replaceable** in oyatie's substrate plan:

### Phase 0 — Milvus-via-adapter (current, this ADR)

- Milvus 2.6.x deployed per the decision above.
- All consumer µservices retrieve through the `oya-shared-vector-store-kernel` port; the `MilvusVectorStore` adapter is the only Milvus-binding code.
- Kernel trait surface is engine-agnostic (CRUD + ANN search + filtering + per-tenant collection management).

### Phase 1 — Operational maturity (~Q4 2026 → Q3 2027)

- Cell-meta capacity model includes per-tenant vector count + per-collection index size + GPU-vs-CPU index-build mix.
- DiskANN cold tier proven at ≥100M vectors per cell.
- RAFT GPU index build proven on the canonical NVIDIA cell SKU.
- Backup + DR drills per ADR-0152 across all KR / EU / US cells.

### Phase 2 — In-house replacement: `oya-vector-store-server` (~Q3 2027)

A Rust-native vector database, shipped as `oya-vector-store-server` under `crates/oya-vector-store-*` and `microservices/foundry/iac/helm/oya-vector-store/`:

- **Architecture.** Disaggregated planes mirroring Milvus's coord/access/worker/storage split, but native to oyatie's primitives (etcd → oyatie internal coord; Pulsar reused as log broker; SeaweedFS reused as object store).
- **Index types.** HNSW (canonical hot path) + IVF (bulk similarity) + DiskANN (cold tier). GPU acceleration via CUDA bindings (optional; CPU-only is the default).
- **Multi-tenancy.** Per-tenant collection isolation native at the engine layer (no row-level-policy retrofit).
- **Wire protocol.** gRPC + protobuf compatible with the Milvus client surface where practical, so consumer µservices migrate by repointing the kernel adapter — no consumer-side code change required.

**Trigger conditions** (value-anchored, NOT date-anchored — any one promotes the in-house lane to active development; the date below is a planning anchor only):

1. ≥1×10⁹ vectors per cluster sustained for ≥30 days. **(value-anchored)**
2. Foundry's RAG retrieval p99 latency budget breached (>30ms hot path) for 7 consecutive days despite tuning. **(value-anchored)**
3. Milvus license posture changes (e.g., a relicense event from Zilliz comparable to the 2024 Redis event). **(event-anchored)**
4. Cross-cell residency requirements exceed Milvus's multi-cell capability. **(capability-anchored)**

**Production-validation evidence** for Milvus at billion scale (basis for trigger 1's value): Zilliz Cloud customers at 10B+ vectors (case studies); NVIDIA RAPIDS RAFT reference customer (CUDA-VS); Salesforce Einstein vector store; eBay item-similarity index; IKEA Marketplace recommendation. Documented in `microservices/foundry/capacity-model-milvus.md` §"Production-validation evidence".

**Industry parallels** — AWS Bedrock Knowledge Bases (managed RAG retrieval; in-house at AWS), Google Vertex AI Vector Search (in-house at Google), Microsoft Azure AI Search vector capability (in-house at Microsoft), Oracle Database 23ai vector store (in-house at Oracle). Each hyperscaler runs their own vector substrate in production for their own AI workloads; oyatie's Phase 2 plan follows the same trajectory once Milvus's substrate is exercised long enough to validate the requirement set.

### Phase 3 — Migration (post Phase-2 GA)

- Per-µservice repointing of the kernel adapter: `vector_store_backend: "in_house"` in manifest.
- Per-collection migration via dual-write window + cutover after parity validation.
- Milvus retired per cell after all consumer µservices migrate.

The in-house roadmap is a commitment of trajectory, not a near-term deliverable. Phase 0 ships now; Phase 2 is a real engineering investment behind concrete trigger conditions.

## Rollback

- **Cluster rollback:** `helm rollback foundry-milvus` reverts to the prior Helm release; vector data persists in object store; etcd metadata may need restore from backup.
- **Index rollback:** drop the collection's index; rebuild from segments; downtime measured in minutes per-collection segment count.
- **Wholesale rollback to pgvector:** not supported at hyperscaler scale (the ≥10M ceiling forecloses it). Per-tenant rollback for ≤10M tenants is possible by re-embedding into pgvector and pointing the kernel adapter at the embedded-tier path.

## References

- Milvus — https://milvus.io/ ; Apache 2.0; CNCF Graduated.
- Milvus 2.6 release announcement — https://milvus.io/blog/introduce-milvus-2-6-built-for-scale-designed-to-reduce-costs.md
- Milvus 2.6.15 release notes — https://github.com/milvus-io/milvus/releases/tag/v2.6.15
- `milvus-sdk-rust` — https://crates.io/crates/milvus-sdk-rust
- NVIDIA RAPIDS RAFT integration — https://github.com/rapidsai/raft
- Apache Pulsar 4.2 — https://pulsar.apache.org/release-notes/versioned/pulsar-4.2.0/
- etcd 3.5 — https://etcd.io/
- ADR-0014 — build-vs-buy policy.
- ADR-0026 — in-house AI substrate roadmap.
- ADR-0030 — search µservice architecture.
- ADR-0038 — trust framework + DSR cascade + proof of erasure.
- ADR-0043-secrets-management-openbao-and-hsm-per-cell — OpenBao SecretReference.
- ADR-0046 — superseded by this ADR.
- ADR-0049 — cross-region replication and residency.
- ADR-0131-per-microservice-flat-layout — flat layout at `microservices/foundry/iac/helm/milvus/`.
- ADR-0145 — inter-microservice communication reform (Invariant 3 ontology projections).
- ADR-0152 — RPO / RTO canonical.
- ADR-0153 — outbox pattern.
- ADR-0155 — per-tenant resource quotas.
- ADR-0241-dr-business-continuity-portfolio-policy — cross-cell DR.
- ADR-0184 — storage tier layering (Milvus is the vector-database addition to the four-tier storage model; conceptually Tier-5 specialty store).
- ADR-0186 — observability backplane layering (Milvus metrics/logs/traces flow through OpenTelemetry Collector).
- ADR-0193 — OLAP analytics warehouse (ClickHouse).
- ADR-0194 — time-series for tenant-facing (TimescaleDB extension).
- ADR-0195 — stream processing tier (ClickHouse MV default; Flink escalation).
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098.

## Historical residual from ADR-46 (E3 fold 2026-08-06)

**Title:** ADR-0046-vector-store-strategy

**Preserved decision gist:** We adopt **pgvector** as the day-1 canonical vector store; an **in-house Rust HNSW/IVF implementation** as the billion-scale long-horizon target; **FAISS (MIT)** only as an adapter behind a port; **Milvus / Qdrant / Pinecone** only as adapters with explicit ADR review (license check + per-vendor integration risk). ### pgvector at day-1 ```sql -- per-tenant per-cell schema CREATE EXTENSION IF NOT EXISTS vector; CREATE TABLE search_embeddings ( id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, source_id BIGINT NOT NULL, embedding VECTOR(1024) NOT NULL, data_class VARCHAR(64) NOT NULL, inserted_

_Source file archived after fold; full body in git history / docs/adr-archive/._
