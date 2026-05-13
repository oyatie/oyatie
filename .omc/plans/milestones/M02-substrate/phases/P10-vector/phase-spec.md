---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P10-vector
status: Proposed
acceptance_lanes: []
entry_gate: |
  M01-P05 complete; P02-ontology merged; Postgres 16 with pgvector extension
  available (pgvector ≥0.7.0, supporting HNSW index); cargo check exits 0.
exit_gate: |
  All vector crates compile; pgvector HNSW index created on
  vector.embeddings.vector column; per-tenant per-object-type embedding tables
  verified (no cross-tenant query possible); cosine similarity search returns
  top-k results with correct RLS enforcement; embedding upsert round-trip green;
  k6 vector search p99≤200ms at 1k RPS; grit done; ICM row emitted.
depends_on:
  - milestone: M01
    phase: P05-scaffold-locks
    reason: "workspace scaffold prerequisite"
  - milestone: M02
    phase: P02-ontology
    reason: "Ontology objects are the primary vector corpus; embeddings reference object_id"
owner_team: council-architecture
---

# P10-vector: Vector substrate — pgvector DAY-1, per-tenant per-object-type embedding tables, in-house HNSW/IVF long-horizon

## Purpose

This phase delivers the complete Vector substrate. Day-1 implementation uses pgvector (Postgres extension, HNSW index with `lists=100, ef_construction=128, m=16` defaults) for ANN (Approximate Nearest Neighbor) similarity search with RLS-enforced per-tenant isolation. The in-house HNSW/IVF long-horizon path (planned for M04+) is designed for now: the `VectorStorePort` trait abstracts the backend so migrating from pgvector to an in-house engine requires only a new adapter crate with no kernel or domain changes. Per-tenant per-object-type embedding tables (one `vector.embeddings_{object_type}` table per object type per tenant) ensure Citus sharding alignment and prevent cross-object-type distance comparison errors. This phase enables semantic search, RAG (Retrieval-Augmented Generation) pipelines, and recommendation surfaces across all products.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `vector` | `embeddings`, `similarity` | `crates/oya-vector-{embeddings,similarity}-{kernel,domain,application,adapter}/`, `crates/oya-vector-worker/`, `crates/oya-vector-rest/`, `crates/oya-vector-app/` | 2×4 + 1 worker + 1 rest + 1 app = 12 crates |

Naming justification:

```
NAME: oya-vector-embeddings-kernel
JUSTIFICATION:
- microservice = vector: the vector/embedding substrate; pgvector day-1
- bc-tokens = embeddings: the embedding storage BC (upsert/delete embeddings
  by Ontology object_id); distinct from similarity (ANN query execution)
- layer = kernel: EmbeddingStorePort + Embedding, ObjectEmbeddingId types; zero I/O
- exemptions claimed: none

NAME: oya-vector-similarity-kernel
JUSTIFICATION:
- microservice = vector: same µservice
- bc-tokens = similarity: the ANN/similarity-search query BC; cosine/dot-product/L2
  distance metric selection; top-k retrieval
- layer = kernel: SimilaritySearchPort + SimilarityQuery, SearchHit types
- exemptions claimed: none
```

### Out-of-scope

- Embedding model hosting / inference — products supply pre-computed vectors via the `EmbeddingStorePort`; this substrate only stores and queries.
- In-house HNSW/IVF engine implementation — long-horizon target; M04+ scope; port trait is designed for it.
- Cross-tenant vector federation — forbidden; per-tenant isolation is invariant.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full DDL + EmbeddingStorePort + SimilaritySearchPort + pgvector adapter + HNSW index + RLS + load test | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P10-vector
oya gate validate lean-a2 --phase P10-vector
oya gate validate lean-a3 --phase P10-vector
oya gate validate lean-a4 --phase P10-vector
```

### Vector correctness gates

```bash
# pgvector HNSW cosine similarity top-k
cargo nextest run -p oya-vector-similarity-adapter --test pgvector_cosine_topk  # exit 0
# Per-tenant RLS enforcement
cargo nextest run -p oya-vector-embeddings-application --test tenant_embedding_isolation  # exit 0
# Upsert idempotency (same object_id → overwrite embedding)
cargo nextest run -p oya-vector-embeddings-application --test upsert_idempotent  # exit 0
# Distance metric selection (cosine / dot-product / L2)
cargo nextest run -p oya-vector-similarity-domain --test distance_metric_selection  # exit 0
```

### Load test gate

```bash
k6 run tests/load/smoke-vector-search.js --env BASE_URL=http://localhost:8088
# Pass: p99 ≤200ms on top-10 cosine similarity search (1536-dim vectors, 1M rows/tenant)
vegeta attack -rate=1000/s -duration=60s -targets=tests/load/vector-targets.txt | vegeta report
# Pass: p99 ≤200ms; 0 errors at 1k RPS
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-vector-embeddings-kernel` | `kernel` | Yes — `EmbeddingStorePort` | N/A | No |
| `oya-vector-similarity-kernel` | `kernel` | Yes — `SimilaritySearchPort` | N/A | No |
| `oya-vector-embeddings-domain` | `domain` | N/A — Embedding value object, dim validation | N/A | No |
| `oya-vector-embeddings-adapter` | `adapter` | N/A | Yes — `PgVectorAdapter` | No |
| `oya-vector-similarity-adapter` | `adapter` | N/A | Yes — `PgVectorHnswAdapter` | No |
| `oya-vector-worker` | `worker` | N/A | No direct adapter | No |
| `oya-vector-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-vector-embeddings-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait EmbeddingStorePort: Send + Sync + sealed::Sealed {
    /// Upsert: if object_id already has an embedding for this model_id, overwrite.
    async fn upsert(&self, tenant_id: TenantId, embedding: Embedding)
        -> Result<EmbeddingId, VectorError>;
    /// Delete embedding for object_id + model_id combination.
    async fn delete(&self, tenant_id: TenantId, object_id: ObjectId, model_id: ModelId)
        -> Result<(), VectorError>;
    /// Batch upsert for bulk ingestion (e.g., after Ontology migration).
    async fn batch_upsert(&self, tenant_id: TenantId, embeddings: Vec<Embedding>)
        -> Result<BatchUpsertResult, VectorError>;
}

// oya-vector-similarity-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait SimilaritySearchPort: Send + Sync + sealed::Sealed {
    /// ANN top-k search. Metric: cosine | dot_product | l2. RLS-enforced per tenant.
    async fn search(&self, tenant_id: TenantId, query: SimilarityQuery)
        -> Result<Vec<SearchHit>, VectorError>;
    /// Filtered ANN: restrict to object_type + optional metadata filter.
    async fn filtered_search(&self, tenant_id: TenantId, query: FilteredSimilarityQuery)
        -> Result<Vec<SearchHit>, VectorError>;
}

#[derive(Debug, Clone)]
pub struct SimilarityQuery {
    pub vector: Vec<f32>,            // query embedding; dims must match stored
    pub model_id: ModelId,           // ensures same embedding space
    pub object_type: Option<String>, // restrict to object type
    pub metric: DistanceMetric,      // Cosine | DotProduct | L2
    pub top_k: u32,                  // max 1000
    pub ef_search: Option<u32>,      // HNSW ef_search override; default 64
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P10-vector` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P10-vector` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P10-vector` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `embeddings` | `vector` | pending |
| `similarity` | `vector` | pending |

---

## Grit Claim Symbols

```
crates/oya-vector-embeddings-kernel/src/ports.rs::EmbeddingStorePort
crates/oya-vector-similarity-kernel/src/ports.rs::SimilaritySearchPort
crates/oya-vector-embeddings-adapter/src/pgvector.rs::PgVectorAdapter
crates/oya-vector-similarity-adapter/src/pgvector_hnsw.rs::PgVectorHnswAdapter
migrations/vector/V001__vector_init.sql::vector_schema
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P10-vector started; scope: 12 crates (embeddings/similarity BCs); pgvector HNSW day-1; in-house HNSW/IVF port abstracted; per-tenant per-object-type tables; RAG-ready" \
  -i high \
  -k "M02,P10,phase-start,vector"

icm store \
  -t context-oyatie \
  -c "Phase P10-vector complete; pgvector HNSW cosine search green; RLS isolation verified; upsert idempotent; p99≤200ms at 1k RPS; next: P11-finance-library" \
  -i high \
  -k "M02,P10,phase-complete,vector"
```

---

## References

- Bominal ADRs inherited: ADR-0108 (vector property type), ADR-0107 (Ontology agent gateway RAG path)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05, M02-P02-ontology
- unblocks: LLM RAG pipelines in Wave-B (agent-gateway function invocations with semantic retrieval), medical/connect recommendation surfaces
