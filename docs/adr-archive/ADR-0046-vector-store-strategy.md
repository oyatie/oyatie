---
id: ADR-0046
status: Superseded
doc_status: published
superseded_by: [ADR-0192]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0046: Vector store strategy — pgvector day-1, in-house Rust HNSW/IVF at billion-scale long-horizon, FAISS only as adapter

> **Status:** Superseded by [ADR-0192](ADR-0192-vector-database-canonical-milvus.md)
> **Supersedes:** -
> **Superseded-by:** [ADR-0192](ADR-0192-vector-database-canonical-milvus.md)
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0026, ADR-0030, ADR-0033, ADR-0045, ADR-0047

---

## Context

Vector search underlies semantic retrieval (Search RAG per ADR-0030), Foundry agent memory (per ADR-0007 cross-session memory equivalent), per-vertical entity matching (e.g. clinical document similarity), and Workspace Drive content search. Without a pinned vector-store strategy, every axis would adopt its own vector engine; license posture would fragment; per-tenant residency would be a per-engine retrofit.

The pack-of-19 foundation ADRs named vector search as a need but did not pin the engine, the license posture, or the scale trajectory. This ADR pins all three: pgvector at day-1 (lives in our OLTP tier per ADR-0045; license-clean PostgreSQL Lic + extension), in-house Rust HNSW/IVF at billion-scale long-horizon (under `crates/oya-vector-*`), and FAISS / Milvus / Qdrant only as adapters behind a port — never as primary stores.

---

## Decision

We adopt **pgvector** as the day-1 canonical vector store; an **in-house Rust HNSW/IVF implementation** as the billion-scale long-horizon target; **FAISS (MIT)** only as an adapter behind a port; **Milvus / Qdrant / Pinecone** only as adapters with explicit ADR review (license check + per-vendor integration risk).

### pgvector at day-1

```sql
-- per-tenant per-cell schema
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE search_embeddings (
    id BIGSERIAL PRIMARY KEY,
    tenant_id UUID NOT NULL,
    source_id BIGINT NOT NULL,
    embedding VECTOR(1024) NOT NULL,
    data_class VARCHAR(64) NOT NULL,
    inserted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX search_embeddings_hnsw
  ON search_embeddings
  USING hnsw (embedding vector_cosine_ops)
  WITH (m = 16, ef_construction = 64)
  WHERE tenant_id = current_setting('app.tenant_id')::UUID;
```

- **Living in OLTP.** pgvector is a PostgreSQL extension (per ADR-0045 OLTP tier); per-tenant per-cell sharding inherits.
- **License.** PostgreSQL Lic (clean).
- **Index types.** HNSW (default for high recall) + IVFFlat (default for high throughput).
- **Per-tenant scoping.** Embeddings labeled with `tenant_id`; per-tenant index partial via `WHERE tenant_id = ...`.
- **Per-data-class scoping.** Embeddings labeled with `data_class` per ADR-0008 DUBO.
- **Per-DSR delete.** Per-row delete via DSR cascade (ADR-0038).

### In-house Rust HNSW/IVF at billion-scale long-horizon

When per-tenant embedding count exceeds the practical pgvector ceiling (~100M per index per cell), we transition to an in-house Rust implementation under `crates/oya-vector-*`:

```rust
// crates/oya-vector-hnsw-kernel
pub struct HnswIndex<const DIM: usize> {
    pub graph: HnswGraph,
    pub vectors: VectorStore,     // disk-backed, mmap-friendly
    pub config: HnswConfig,       // m / ef_construction / ef_search
    pub tenant_id: TenantId,
    pub data_class: DataClass,
}

pub struct IvfIndex<const DIM: usize> {
    pub centroids: Vec<Vector<DIM>>,
    pub posting_lists: Vec<PostingList>,
    pub vectors: VectorStore,
    pub tenant_id: TenantId,
    pub data_class: DataClass,
}
```

- **Per-tenant per-cell index.** Same isolation pattern as pgvector, but at object-store scale.
- **Disk + memory tiering.** Hot vectors mmap; cold vectors object storage.
- **License.** In-house = our license (Apache-2 outbound where applicable).
- **Long-horizon target.** GA at W+24+, conditional on usage telemetry signaling pgvector ceiling reached.

### FAISS (MIT) only as adapter

```rust
// crates/oya-vector-faiss-adapter
pub struct FaissAdapter {
    pub index: faiss::Index,    // FFI to libfaiss
}

impl VectorStorePort for FaissAdapter { /* trait from kernel */ }
```

- FAISS (Meta; MIT) is permitted **only** as an adapter behind a port.
- Use cases: research / eval workloads where FAISS-specific algorithms (e.g. PQ + IVFADC) are required.
- Not used as primary tenant-facing store (no per-tenant ops, no DSR cascade hook out of the box).

### Milvus / Qdrant / Pinecone — adapter only with ADR review

- **Milvus.** Apache-2 (verified at adoption); usable as adapter; per-adapter ADR.
- **Qdrant.** Apache-2 (verified); usable as adapter.
- **Pinecone.** Commercial SaaS; data leaves our cells; adapter only for tenants explicitly opting in with cross-region transfer consent (per ADR-0049).

Each adapter requires its own ADR before adoption.

### Per-pack embedding model + dimension

Per ADR-0030 search architecture:

| Pack | Embedding model | Dimension |
|---|---|---|
| KR | KoSimCSE-large | 1024 |
| EN | bge-large-v1.5 | 1024 |
| Multilingual | bge-m3 | 1024 |
| Specialized (clinical) | per-pack fine-tuned | 768 (default) or 1024 |

Per-tenant model selection from per-pack default; per-tenant override allowed for plus-tier subscriptions.

### Per-tenant + per-tier index segregation

Per ADR-0030 + ADR-0034:

- Public-tier embeddings in shared index.
- Tenant-public embeddings in per-tenant namespace.
- Tenant-private embeddings in per-tenant + per-data-class index.
- Regulated-tier embeddings (PHI/PCI/Sensitive) in dedicated cell + dedicated index.

Cross-tier query is forbidden by default; opt-in via DUBO grant.

### Per-tenant DSR cascade

Per ADR-0038:

- Per-row delete in pgvector via standard PostgreSQL `DELETE`.
- Per-row delete in in-house store via per-shard tombstone + scheduled compaction.
- Per-row delete in adapter stores via adapter-specific delete API.
- Proof-of-erasure per ADR-0038 emitted per affected store.

### Anti-scope

This ADR does not own the search backend (per ADR-0047, but vector index informs Search). Does not own the embedding-model serving (per ADR-0026 in-house AI substrate). Does not own per-microservice embedding usage policy (per-microservice ADR governs).

---

## Consequences

### Positive

- pgvector at day-1 means we don't ship a separate vector engine — operational surface stays small.
- In-house long-horizon eliminates dep risk at billion-scale.
- Adapter-only posture for FAISS / Milvus / Qdrant / Pinecone means we never become dependent on an external vector engine.
- Per-tenant per-tier index segregation maps cleanly to DUBO + DSR cascade.

### Negative

- pgvector has practical ceiling around 100M vectors per index per cell; very large tenants will need in-house earlier than W+24.
- In-house HNSW/IVF is real engineering investment.
- Adapter API has to support all credible engines, which constrains the kernel trait surface.

### Operational

- Per-cell pgvector index health monitored (per-index size, per-index recall via per-week sample query).
- Per-tenant embedding count tracked; tenants approaching ceiling alerted to capacity team.
- Per-quarter recall benchmark per pack (KR / EN / multilingual).
- Per-DSR vector deletion latency tracked.

---

## Alternatives considered

### Alternative A — Milvus or Qdrant as primary

- **Pros:** purpose-built; mature.
- **Cons:** separate operational surface; per-tenant residency requires retrofit; DSR cascade requires per-engine integration.
- **Rejected because:** pgvector is good enough at day-1 scale and lives in our OLTP tier already.

### Alternative B — Pinecone as primary

- **Pros:** managed.
- **Cons:** commercial SaaS; data leaves our cells; KR sovereignty concerns.
- **Rejected because:** sovereignty.

### Alternative C — Skip pgvector; build in-house from day 1

- **Pros:** no transition.
- **Cons:** delays day-1 vector capability; we ship pgvector working then transition when scale demands.
- **Rejected because:** day-1 pgvector is a 0-cost-marginal capability via the OLTP tier.

### Alternative D — One engine per axis

- **Pros:** axis flexibility.
- **Cons:** N engines; per-engine drift; cohesion violated.
- **Rejected because:** vector store is a substrate concern.

---

## Open questions

1. **Q1.** pgvector → in-house transition trigger — vector count or query latency? Default: query latency P95 > 100ms per cell triggers transition. → owner: `foundry`.
2. **Q2.** In-house disk format — Parquet-derived or Arrow-IPC? Default: Arrow-IPC (consistent with DataFusion per ADR-0045). → owner: `foundry`.
3. **Q3.** Per-tenant BYO embedding model — at GA or W+24? Default: GA opt-in for plus-tier tenants. → ADR-0026.
4. **Q4.** Per-pack embedding model versioning — per-tenant pinning at GA? Default: yes; per-tenant migration is opt-in. → ADR-0026.
5. **Q5.** Quantization (PQ / SQ / binary) — at GA or W+12? Default: SQ (scalar quantization) at GA; PQ at W+12 if cost reductions warrant. → owner: `foundry`.

---

## References

- `docs/PRD.md` §10 (data plane)
- `docs/DESIGN.md` §11 (vector store), §10 (cross-microservice contracts)
- pgvector docs (PostgreSQL Lic); FAISS docs (MIT); Milvus + Qdrant docs (Apache-2 verified)
- HNSW (Malkov & Yashunin 2018); IVF / IVFADC (Jégou et al.)
- ADR-0001 (cohesion), ADR-0026 (AI substrate), ADR-0030 (search), ADR-0033 (vertical pack), ADR-0045 (database tier), ADR-0047 (search backend)
