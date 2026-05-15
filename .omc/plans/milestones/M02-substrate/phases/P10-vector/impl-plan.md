---
doc_class: ImplPlan
template_id: TPL-IMPL-PLAN
milestone: M02-substrate
phase: P10-vector
status: Proposed
depends_on_phase_spec: phase-spec.md
purpose: "Implementation plan for P10-vector of M02-substrate: detailed code structure and acceptance lanes."
---
# P10-vector Implementation Plan

## 0. Grit Claim

```bash
grit session start
grit claim \
  --agent m02-wave-a-executor \
  --intent "P10-vector: 12 crates; pgvector HNSW day-1; per-tenant per-object-type tables; RLS" \
  --ttl 3h \
  --symbols \
    "crates/oya-vector-embeddings-kernel/src/ports.rs::EmbeddingStorePort" \
    "crates/oya-vector-similarity-kernel/src/ports.rs::SimilaritySearchPort" \
    "crates/oya-vector-embeddings-adapter/src/pgvector.rs::PgVectorAdapter" \
    "crates/oya-vector-similarity-adapter/src/pgvector_hnsw.rs::PgVectorHnswAdapter" \
    "migrations/vector/V001__vector_init.sql::vector_schema"
```

---

## 1. Crate Inventory (12 crates)

| Crate | Layer | Primary purpose |
|---|---|---|
| `oya-vector-embeddings-kernel` | kernel | `EmbeddingStorePort` + `Embedding`, `EmbeddingId`, `ObjectEmbeddingId`, `ModelId` types |
| `oya-vector-embeddings-domain` | domain | `Embedding` dimension validation, `DimMismatch` guard |
| `oya-vector-embeddings-application` | application | `EmbeddingService` — upsert/delete, table routing by object_type |
| `oya-vector-embeddings-adapter` | adapter | `PgVectorAdapter` — Postgres + pgvector extension |
| `oya-vector-similarity-kernel` | kernel | `SimilaritySearchPort` + `SimilarityQuery`, `FilteredSimilarityQuery`, `SearchHit`, `DistanceMetric` types |
| `oya-vector-similarity-domain` | domain | Distance metric validation, top-k bounds check |
| `oya-vector-similarity-application` | application | `SimilarityService` — routes to adapter; RLS enforcement check |
| `oya-vector-similarity-adapter` | adapter | `PgVectorHnswAdapter` — ANN search with HNSW index |
| `oya-vector-worker` | worker | Outbox poller → upsert/delete embeddings on object mutation |
| `oya-vector-rest` | rest | Axum REST: `POST /embeddings`, `POST /similarity/search` |
| `oya-vector-grpc` | grpc | Tonic gRPC: `VectorService.UpsertEmbedding`, `VectorService.Search` |
| `oya-vector-app` | app | Composition root; wires adapters |

---

## 2. Migration: `migrations/vector/V001__vector_init.sql`

```sql
-- migrations/vector/V001__vector_init.sql
-- Vector substrate: per-tenant per-object-type embedding tables + HNSW index
-- Requires: pgvector extension ≥0.7.0 (HNSW support)

BEGIN;

CREATE SCHEMA IF NOT EXISTS vector;

-- Ensure pgvector extension is loaded
CREATE EXTENSION IF NOT EXISTS vector;

-- Global embedding model registry (no tenant scoping — models are platform-wide)
CREATE TABLE vector.embedding_models (
    id               text        PRIMARY KEY,  -- model identifier, e.g. 'text-embedding-3-large'
    dimensions       int         NOT NULL CHECK (dimensions > 0 AND dimensions <= 4096),
    distance_metric  text        NOT NULL DEFAULT 'cosine'
                                 CHECK (distance_metric IN ('cosine', 'dot_product', 'l2')),
    description      text,
    deprecated_at    timestamptz,
    created_at       timestamptz NOT NULL DEFAULT now()
);

-- Insert well-known models as platform defaults
INSERT INTO vector.embedding_models (id, dimensions, distance_metric, description) VALUES
    ('text-embedding-3-large', 3072, 'cosine',      'OpenAI text-embedding-3-large'),
    ('text-embedding-3-small', 1536, 'cosine',      'OpenAI text-embedding-3-small'),
    ('text-embedding-ada-002',  1536, 'cosine',     'OpenAI text-embedding-ada-002 (legacy)'),
    ('bge-m3',                  1024, 'cosine',     'BAAI/bge-m3 multilingual, Korean-strong'),
    ('gte-qwen2-1.5b-instruct', 1536, 'cosine',     'Qwen2 1.5B GTE multilingual');

-- Per-tenant per-object-type embeddings table
-- DESIGN DECISION: One physical table for all embeddings, with (tenant_id, object_type)
-- as partition discriminator, rather than one table per (tenant_id, object_type) pair.
-- Rationale:
--   - Dynamic DDL per-tenant/per-object-type creates operational risk (thousands of tables).
--   - Citus sharding on tenant_id works correctly with a single table + RLS.
--   - HNSW index filters via WHERE tenant_id = ? + object_type = ? before ANN scan
--     (pgvector 0.7+ supports partial indexes for per-class ANN).
--   - Per-class partial HNSW indexes (see below) restore query performance.
CREATE TABLE vector.embeddings (
    id               uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid        NOT NULL,
    object_id        uuid        NOT NULL,    -- references ontology.objects.id (not FK; cross-schema)
    object_type      text        NOT NULL,    -- 'entity' | 'person' | 'product' | ...
    model_id         text        NOT NULL REFERENCES vector.embedding_models(id),
    embedding        vector(4096),            -- max dims; NULL if dims < 4096 for this model
    --  For models with dim < 4096, actual vector stored using partial column hack:
    --  see note below about model-specific sub-tables.
    dims             int         NOT NULL,    -- actual dimension count for this embedding
    metadata         jsonb       NOT NULL DEFAULT '{}'::jsonb,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, object_id, model_id)  -- one embedding per (object, model) pair
);

-- NOTE on variable dimension storage:
-- pgvector requires a fixed vector() dimension column per table.
-- We use vector(4096) as the max dimension column and store actual dims in the `dims` column.
-- For production, the recommendation is to create per-model-dimension sub-tables
-- (e.g., vector.embeddings_1536 for 1536-dim models) to enable efficient HNSW indexing
-- without wasting storage. This migration creates the base table; product migrations
-- add model-specific tables as needed.
-- The PgVectorAdapter routes to the correct table based on model_id dimensions.

-- RLS: strict per-tenant isolation
ALTER TABLE vector.embeddings ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY ON vector.embeddings;
CREATE POLICY tenant_isolation ON vector.embeddings
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- HNSW index (pgvector 0.7+): per-object-type partial indexes for ANN search
-- These are created for the most common object types; additional types added via product migrations.
-- HNSW parameters: m=16 (max connections per layer), ef_construction=128 (index-time beam search),
-- equivalent to lists=100 in IVFFlat (but HNSW has better recall/latency trade-off)
CREATE INDEX CONCURRENTLY IF NOT EXISTS
    embeddings_hnsw_entity_cosine_idx
    ON vector.embeddings USING hnsw ((embedding::vector(1536)) vector_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE object_type = 'entity' AND model_id = 'text-embedding-3-small';

CREATE INDEX CONCURRENTLY IF NOT EXISTS
    embeddings_hnsw_person_cosine_idx
    ON vector.embeddings USING hnsw ((embedding::vector(1536)) vector_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE object_type = 'person' AND model_id = 'text-embedding-3-small';

-- B-tree index for point-lookup: given (tenant, object, model) → embedding
CREATE INDEX embeddings_lookup_idx
    ON vector.embeddings (tenant_id, object_id, model_id);

-- Outbox for vector events
CREATE TABLE vector.outbox (
    id               bigserial   PRIMARY KEY,
    tenant_id        uuid        NOT NULL,
    aggregate_type   text        NOT NULL,
    aggregate_id     uuid        NOT NULL,
    event_type       text        NOT NULL,
    payload          jsonb       NOT NULL,
    topic            text        NOT NULL,
    published        boolean     NOT NULL DEFAULT false,
    created_at       timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX vector_outbox_unpublished_idx
    ON vector.outbox (created_at) WHERE published = false;

ALTER TABLE vector.outbox ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY ON vector.outbox;
CREATE POLICY tenant_isolation ON vector.outbox
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

COMMIT;
```

---

## 3. Kernel Types and Port Traits

### `oya-vector-embeddings-kernel/src/ports.rs`

```rust
// crates/oya-vector-embeddings-kernel/src/ports.rs
use async_trait::async_trait;
use uuid::Uuid;
use crate::{BatchUpsertResult, Embedding, EmbeddingId, ModelId, ObjectId, TenantId, VectorError};

#[doc(hidden)]
pub mod sealed {
    pub trait Sealed {}
}

/// Port for writing/maintaining embedding vectors per tenant.
/// Day-1 implementation: PgVectorAdapter (Postgres + pgvector extension).
/// Long-horizon: in-house HNSW/IVF engine with same port contract.
#[async_trait]
pub trait EmbeddingStorePort: Send + Sync + sealed::Sealed {
    /// Upsert: if (tenant_id, object_id, model_id) already exists, overwrite.
    async fn upsert(
        &self,
        tenant_id: TenantId,
        embedding: Embedding,
    ) -> Result<EmbeddingId, VectorError>;

    /// Delete embedding for (object_id, model_id) combination.
    async fn delete(
        &self,
        tenant_id: TenantId,
        object_id: ObjectId,
        model_id: ModelId,
    ) -> Result<(), VectorError>;

    /// Batch upsert for bulk ingestion (e.g., after Ontology migration or model re-embed).
    async fn batch_upsert(
        &self,
        tenant_id: TenantId,
        embeddings: Vec<Embedding>,
    ) -> Result<BatchUpsertResult, VectorError>;
}
```

### `oya-vector-embeddings-kernel/src/types.rs`

```rust
// crates/oya-vector-embeddings-kernel/src/types.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TenantId = Uuid;
pub type EmbeddingId = Uuid;
pub type ObjectId = Uuid;
pub type ModelId = String;

/// An embedding vector to be stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub object_id: ObjectId,
    pub object_type: String,
    pub model_id: ModelId,
    /// The dense vector. Dimensions must match model_id's registered dimension count.
    pub vector: Vec<f32>,
    /// Optional metadata (e.g., embedding model version, input hash for dedup)
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpsertResult {
    pub inserted: u64,
    pub updated: u64,
    pub failed: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("dimension mismatch: expected {expected}, got {actual} for model {model_id}")]
    DimMismatch { expected: usize, actual: usize, model_id: ModelId },
    #[error("unknown model: {0}")]
    UnknownModel(ModelId),
    #[error("cross-tenant access denied")]
    CrossTenantAccess,
    #[error("embedding not found: object {object_id} model {model_id}")]
    NotFound { object_id: ObjectId, model_id: ModelId },
    #[error("pgvector error: {0}")]
    PgVector(String),
    #[error("top_k exceeds maximum: {requested} > {max}")]
    TopKTooLarge { requested: u32, max: u32 },
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
```

### `oya-vector-similarity-kernel/src/ports.rs`

```rust
// crates/oya-vector-similarity-kernel/src/ports.rs
use async_trait::async_trait;
use crate::{FilteredSimilarityQuery, SearchHit, SimilarityQuery, TenantId, VectorError};

#[doc(hidden)]
pub mod sealed {
    pub trait Sealed {}
}

/// ANN similarity search port.
/// RLS is enforced at the Postgres layer (SET LOCAL oyatie.tenant_id before query).
/// This port must NEVER return results from a different tenant.
#[async_trait]
pub trait SimilaritySearchPort: Send + Sync + sealed::Sealed {
    /// ANN top-k search.
    async fn search(
        &self,
        tenant_id: TenantId,
        query: SimilarityQuery,
    ) -> Result<Vec<SearchHit>, VectorError>;

    /// Filtered ANN: restrict to object_type + optional metadata filter.
    async fn filtered_search(
        &self,
        tenant_id: TenantId,
        query: FilteredSimilarityQuery,
    ) -> Result<Vec<SearchHit>, VectorError>;
}
```

### `oya-vector-similarity-kernel/src/types.rs`

```rust
// crates/oya-vector-similarity-kernel/src/types.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TenantId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityQuery {
    /// Query embedding vector. Dims must match model_id registered dims.
    pub vector: Vec<f32>,
    /// Model that produced the query vector — restricts search to same model embeddings.
    pub model_id: String,
    /// Restrict to object type; None = all types (cross-type ANN — use with caution).
    pub object_type: Option<String>,
    /// Distance metric selection.
    pub metric: DistanceMetric,
    /// Max results to return. Hard cap: 1000.
    pub top_k: u32,
    /// HNSW ef_search override (higher = better recall, slower). Default: 64.
    pub ef_search: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredSimilarityQuery {
    pub base: SimilarityQuery,
    /// jsonb metadata filter applied to embeddings.metadata column.
    /// Example: {"source": "invoice"} matches embeddings with that metadata.
    pub metadata_filter: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceMetric {
    Cosine,
    DotProduct,
    L2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub embedding_id: Uuid,
    pub object_id: Uuid,
    pub object_type: String,
    pub model_id: String,
    /// Distance score (lower is closer for L2/cosine; higher for dot-product).
    pub distance: f32,
    pub metadata: serde_json::Value,
}
```

---

## 4. Adapter Implementations

### `oya-vector-embeddings-adapter/src/pgvector.rs`

```rust
// crates/oya-vector-embeddings-adapter/src/pgvector.rs
use async_trait::async_trait;
use pgvector::Vector;
use sqlx::PgPool;
use oya_vector_embeddings_kernel::{
    ports::{sealed, EmbeddingStorePort},
    BatchUpsertResult, Embedding, EmbeddingId, ModelId, ObjectId, TenantId, VectorError,
};

pub struct PgVectorAdapter {
    pool: PgPool,
}

impl PgVectorAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Set tenant_id in Postgres session for RLS enforcement.
    async fn set_tenant(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: TenantId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT set_config('oyatie.tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(tx.as_mut())
            .await?;
        Ok(())
    }
}

impl sealed::Sealed for PgVectorAdapter {}

#[async_trait]
impl EmbeddingStorePort for PgVectorAdapter {
    async fn upsert(
        &self,
        tenant_id: TenantId,
        embedding: Embedding,
    ) -> Result<EmbeddingId, VectorError> {
        // Validate dimensions against model registry
        let expected_dims: Option<i32> = sqlx::query_scalar(
            "SELECT dimensions FROM vector.embedding_models WHERE id = $1",
        )
        .bind(&embedding.model_id)
        .fetch_optional(&self.pool)
        .await?;

        let expected_dims = expected_dims.ok_or_else(|| {
            VectorError::UnknownModel(embedding.model_id.clone())
        })? as usize;

        if embedding.vector.len() != expected_dims {
            return Err(VectorError::DimMismatch {
                expected: expected_dims,
                actual: embedding.vector.len(),
                model_id: embedding.model_id.clone(),
            });
        }

        let vec = Vector::from(embedding.vector.clone());

        let mut tx = self.pool.begin().await?;
        self.set_tenant(&mut tx, tenant_id).await?;

        let id: EmbeddingId = sqlx::query_scalar(
            r#"
            INSERT INTO vector.embeddings
                (tenant_id, object_id, object_type, model_id, embedding, dims, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (tenant_id, object_id, model_id) DO UPDATE
                SET embedding = EXCLUDED.embedding,
                    dims = EXCLUDED.dims,
                    metadata = EXCLUDED.metadata,
                    updated_at = now()
            RETURNING id
            "#,
        )
        .bind(tenant_id)
        .bind(embedding.object_id)
        .bind(&embedding.object_type)
        .bind(&embedding.model_id)
        .bind(vec)
        .bind(embedding.vector.len() as i32)
        .bind(embedding.metadata)
        .fetch_one(tx.as_mut())
        .await?;

        tx.commit().await?;
        Ok(id)
    }

    async fn delete(
        &self,
        tenant_id: TenantId,
        object_id: ObjectId,
        model_id: ModelId,
    ) -> Result<(), VectorError> {
        let mut tx = self.pool.begin().await?;
        self.set_tenant(&mut tx, tenant_id).await?;

        let affected = sqlx::query(
            "DELETE FROM vector.embeddings WHERE tenant_id = $1 AND object_id = $2 AND model_id = $3",
        )
        .bind(tenant_id)
        .bind(object_id)
        .bind(&model_id)
        .execute(tx.as_mut())
        .await?
        .rows_affected();

        tx.commit().await?;

        if affected == 0 {
            return Err(VectorError::NotFound { object_id, model_id });
        }
        Ok(())
    }

    async fn batch_upsert(
        &self,
        tenant_id: TenantId,
        embeddings: Vec<Embedding>,
    ) -> Result<BatchUpsertResult, VectorError> {
        let mut inserted = 0u64;
        let mut updated = 0u64;
        let mut failed = 0u64;

        // Validate all dims before any writes
        for e in &embeddings {
            let expected: Option<i32> = sqlx::query_scalar(
                "SELECT dimensions FROM vector.embedding_models WHERE id = $1",
            )
            .bind(&e.model_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(exp) = expected {
                if e.vector.len() != exp as usize {
                    failed += 1;
                    continue;
                }
            }
        }

        let mut tx = self.pool.begin().await?;
        self.set_tenant(&mut tx, tenant_id).await?;

        for e in embeddings {
            let vec = Vector::from(e.vector.clone());
            let result = sqlx::query(
                r#"
                INSERT INTO vector.embeddings
                    (tenant_id, object_id, object_type, model_id, embedding, dims, metadata)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (tenant_id, object_id, model_id) DO UPDATE
                    SET embedding = EXCLUDED.embedding,
                        dims = EXCLUDED.dims,
                        metadata = EXCLUDED.metadata,
                        updated_at = now()
                "#,
            )
            .bind(tenant_id)
            .bind(e.object_id)
            .bind(&e.object_type)
            .bind(&e.model_id)
            .bind(vec)
            .bind(e.vector.len() as i32)
            .bind(e.metadata)
            .execute(tx.as_mut())
            .await;

            match result {
                Ok(r) if r.rows_affected() == 1 => inserted += 1,
                Ok(_) => updated += 1,
                Err(_) => failed += 1,
            }
        }

        tx.commit().await?;
        Ok(BatchUpsertResult { inserted, updated, failed })
    }
}
```

### `oya-vector-similarity-adapter/src/pgvector_hnsw.rs`

```rust
// crates/oya-vector-similarity-adapter/src/pgvector_hnsw.rs
use async_trait::async_trait;
use pgvector::Vector;
use sqlx::PgPool;
use oya_vector_similarity_kernel::{
    ports::{sealed, SimilaritySearchPort},
    DistanceMetric, FilteredSimilarityQuery, SearchHit, SimilarityQuery, TenantId, VectorError,
};

/// Top-k hard limit: protects against expensive full-scan at high k values
const MAX_TOP_K: u32 = 1000;
/// Default HNSW ef_search (beam width at query time; higher = better recall, slower)
const DEFAULT_EF_SEARCH: u32 = 64;

pub struct PgVectorHnswAdapter {
    pool: PgPool,
}

impl PgVectorHnswAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn set_tenant_and_ef(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        tenant_id: TenantId,
        ef_search: u32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT set_config('oyatie.tenant_id', $1, true)")
            .bind(tenant_id.to_string())
            .execute(tx.as_mut())
            .await?;
        // Set HNSW ef_search for this transaction
        sqlx::query("SET LOCAL hnsw.ef_search = $1")
            .bind(ef_search as i32)
            .execute(tx.as_mut())
            .await?;
        Ok(())
    }
}

impl sealed::Sealed for PgVectorHnswAdapter {}

#[async_trait]
impl SimilaritySearchPort for PgVectorHnswAdapter {
    async fn search(
        &self,
        tenant_id: TenantId,
        query: SimilarityQuery,
    ) -> Result<Vec<SearchHit>, VectorError> {
        if query.top_k > MAX_TOP_K {
            return Err(VectorError::TopKTooLarge { requested: query.top_k, max: MAX_TOP_K });
        }

        let ef_search = query.ef_search.unwrap_or(DEFAULT_EF_SEARCH);
        let vec = Vector::from(query.vector.clone());

        // Select distance operator based on metric
        let distance_expr = match query.metric {
            DistanceMetric::Cosine     => "embedding <=> $1",
            DistanceMetric::DotProduct => "-(embedding <#> $1)",  // negate: higher dot = closer
            DistanceMetric::L2         => "embedding <-> $1",
        };

        let sql = if let Some(ref ot) = query.object_type {
            format!(
                r#"
                SELECT id, object_id, object_type, model_id,
                       {distance_expr} AS distance,
                       metadata
                FROM vector.embeddings
                WHERE tenant_id = $2
                  AND model_id = $3
                  AND object_type = $4
                ORDER BY {distance_expr}
                LIMIT $5
                "#,
                distance_expr = distance_expr
            )
        } else {
            format!(
                r#"
                SELECT id, object_id, object_type, model_id,
                       {distance_expr} AS distance,
                       metadata
                FROM vector.embeddings
                WHERE tenant_id = $2
                  AND model_id = $3
                ORDER BY {distance_expr}
                LIMIT $4
                "#,
                distance_expr = distance_expr
            )
        };

        let mut tx = self.pool.begin().await?;
        self.set_tenant_and_ef(&mut tx, tenant_id, ef_search).await?;

        let rows: Vec<(uuid::Uuid, uuid::Uuid, String, String, f32, serde_json::Value)> =
            if let Some(ref ot) = query.object_type {
                sqlx::query_as(&sql)
                    .bind(&vec)
                    .bind(tenant_id)
                    .bind(&query.model_id)
                    .bind(ot)
                    .bind(query.top_k as i32)
                    .fetch_all(tx.as_mut())
                    .await?
            } else {
                sqlx::query_as(&sql)
                    .bind(&vec)
                    .bind(tenant_id)
                    .bind(&query.model_id)
                    .bind(query.top_k as i32)
                    .fetch_all(tx.as_mut())
                    .await?
            };

        tx.commit().await?;

        Ok(rows
            .into_iter()
            .map(|(id, object_id, object_type, model_id, distance, metadata)| SearchHit {
                embedding_id: id,
                object_id,
                object_type,
                model_id,
                distance,
                metadata,
            })
            .collect())
    }

    async fn filtered_search(
        &self,
        tenant_id: TenantId,
        query: FilteredSimilarityQuery,
    ) -> Result<Vec<SearchHit>, VectorError> {
        if query.base.top_k > MAX_TOP_K {
            return Err(VectorError::TopKTooLarge {
                requested: query.base.top_k,
                max: MAX_TOP_K,
            });
        }

        let ef_search = query.base.ef_search.unwrap_or(DEFAULT_EF_SEARCH);
        let vec = Vector::from(query.base.vector.clone());

        let distance_expr = match query.base.metric {
            DistanceMetric::Cosine     => "embedding <=> $1",
            DistanceMetric::DotProduct => "-(embedding <#> $1)",
            DistanceMetric::L2         => "embedding <-> $1",
        };

        // Build WHERE clause for optional metadata filter
        // Use @> for jsonb containment (index-compatible)
        let metadata_clause = if query.metadata_filter.is_some() {
            " AND metadata @> $5"
        } else {
            ""
        };

        let sql = format!(
            r#"
            SELECT id, object_id, object_type, model_id,
                   {distance_expr} AS distance,
                   metadata
            FROM vector.embeddings
            WHERE tenant_id = $2
              AND model_id = $3
              AND ($4::text IS NULL OR object_type = $4)
              {metadata_clause}
            ORDER BY {distance_expr}
            LIMIT $6
            "#,
            distance_expr = distance_expr,
            metadata_clause = metadata_clause
        );

        let mut tx = self.pool.begin().await?;
        self.set_tenant_and_ef(&mut tx, tenant_id, ef_search).await?;

        let rows: Vec<(uuid::Uuid, uuid::Uuid, String, String, f32, serde_json::Value)> = {
            let mut q = sqlx::query_as(&sql)
                .bind(&vec)
                .bind(tenant_id)
                .bind(&query.base.model_id)
                .bind(query.base.object_type.as_deref());

            if let Some(ref filter) = query.metadata_filter {
                q = q.bind(filter);
            } else {
                q = q.bind(serde_json::Value::Null);
            }

            q.bind(query.base.top_k as i32)
                .fetch_all(tx.as_mut())
                .await?
        };

        tx.commit().await?;

        Ok(rows
            .into_iter()
            .map(|(id, object_id, object_type, model_id, distance, metadata)| SearchHit {
                embedding_id: id,
                object_id,
                object_type,
                model_id,
                distance,
                metadata,
            })
            .collect())
    }
}
```

---

## 5. Protobuf Schema

```protobuf
// proto/vector/v1/vector_events.proto
syntax = "proto3";
package oyatie.vector.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

// Emitted when an embedding is upserted
message EmbeddingUpserted {
  string  tenant_id     = 1;  // UUID string
  string  embedding_id  = 2;
  string  object_id     = 3;
  string  object_type   = 4;
  string  model_id      = 5;
  int32   dims          = 6;
  bool    was_update    = 7;  // true if overwriting existing embedding
  google.protobuf.Timestamp upserted_at = 8;
}

// Emitted when an embedding is deleted
message EmbeddingDeleted {
  string  tenant_id     = 1;
  string  object_id     = 2;
  string  model_id      = 3;
  google.protobuf.Timestamp deleted_at = 4;
}

// Emitted when a batch upsert completes
message BatchUpsertCompleted {
  string  tenant_id     = 1;
  string  model_id      = 2;
  uint64  inserted      = 3;
  uint64  updated       = 4;
  uint64  failed        = 5;
  google.protobuf.Timestamp completed_at = 6;
}

// gRPC service
service VectorService {
  rpc UpsertEmbedding   (UpsertEmbeddingRequest)  returns (UpsertEmbeddingResponse);
  rpc DeleteEmbedding   (DeleteEmbeddingRequest)  returns (DeleteEmbeddingResponse);
  rpc BatchUpsert       (BatchUpsertRequest)       returns (BatchUpsertResponse);
  rpc Search            (SearchRequest)            returns (SearchResponse);
  rpc FilteredSearch    (FilteredSearchRequest)    returns (SearchResponse);
}

message UpsertEmbeddingRequest {
  string  tenant_id    = 1;
  string  object_id    = 2;
  string  object_type  = 3;
  string  model_id     = 4;
  repeated float vector = 5 [packed = true];
  google.protobuf.Struct metadata = 6;
}

message UpsertEmbeddingResponse {
  string  embedding_id = 1;
  bool    was_update   = 2;
}

message DeleteEmbeddingRequest {
  string  tenant_id    = 1;
  string  object_id    = 2;
  string  model_id     = 3;
}

message DeleteEmbeddingResponse {}

message BatchUpsertRequest {
  string  tenant_id    = 1;
  repeated UpsertEmbeddingRequest embeddings = 2;
}

message BatchUpsertResponse {
  uint64  inserted     = 1;
  uint64  updated      = 2;
  uint64  failed       = 3;
}

message SearchRequest {
  string  tenant_id    = 1;
  string  model_id     = 2;
  repeated float vector = 3 [packed = true];
  string  object_type  = 4;
  string  metric       = 5;  // "cosine" | "dot_product" | "l2"
  uint32  top_k        = 6;
  uint32  ef_search    = 7;
}

message FilteredSearchRequest {
  SearchRequest base    = 1;
  google.protobuf.Struct metadata_filter = 2;
}

message SearchResponse {
  repeated SearchHit hits = 1;
}

message SearchHit {
  string  embedding_id = 1;
  string  object_id    = 2;
  string  object_type  = 3;
  string  model_id     = 4;
  float   distance     = 5;
  google.protobuf.Struct metadata = 6;
}
```

---

## 6. Load Test: `tests/load/smoke-vector-search.js`

```javascript
// tests/load/smoke-vector-search.js
// Target: p99 ≤200ms on top-10 cosine similarity search (1536-dim, 1M rows/tenant)
// Run: k6 run tests/load/smoke-vector-search.js --env BASE_URL=http://localhost:8088

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const p99Latency = new Trend('vector_search_p99_latency', true);
const errorRate = new Rate('vector_search_errors');

const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';
const BASE_URL  = __ENV.BASE_URL  || 'http://localhost:8088';
const MODEL_ID  = __ENV.MODEL_ID  || 'text-embedding-3-small';
const DIMS      = 1536;

// Pre-generate random query vectors (avoid per-request generation overhead)
const QUERY_VECTORS = Array.from({ length: 20 }, () =>
  Array.from({ length: DIMS }, () => (Math.random() * 2) - 1)
);

export const options = {
  scenarios: {
    smoke: {
      executor: 'constant-vus',
      vus: 20,
      duration: '60s',
    },
    sustained: {
      executor: 'constant-arrival-rate',
      rate: 1000,            // 1000 RPS
      timeUnit: '1s',
      duration: '60s',
      preAllocatedVUs: 50,
      maxVUs: 200,
      startTime: '65s',
    },
  },
  thresholds: {
    'http_req_duration{scenario:smoke}':     ['p(99)<200'],
    'http_req_duration{scenario:sustained}': ['p(99)<200'],
    'vector_search_errors':                  ['rate<0.001'],
  },
};

export default function () {
  const qv = QUERY_VECTORS[Math.floor(Math.random() * QUERY_VECTORS.length)];

  const payload = JSON.stringify({
    model_id: MODEL_ID,
    vector: qv,
    object_type: 'entity',
    metric: 'cosine',
    top_k: 10,
    ef_search: 64,
  });

  const start = Date.now();
  const res = http.post(
    `${BASE_URL}/v1/vector/search`,
    payload,
    {
      headers: {
        'Content-Type': 'application/json',
        'X-Tenant-Id': TENANT_ID,
      },
    }
  );
  const duration = Date.now() - start;

  p99Latency.add(duration);

  const ok = check(res, {
    'status 200': (r) => r.status === 200,
    'has hits array': (r) => {
      try {
        const body = JSON.parse(r.body);
        return Array.isArray(body.hits);
      } catch {
        return false;
      }
    },
    'p99 ≤200ms': () => duration <= 200,
  });

  if (!ok) errorRate.add(1);
  sleep(0.001);
}
```

---

## 7. Per-tenant Isolation Test

```rust
// crates/oya-vector-embeddings-application/tests/tenant_embedding_isolation.rs
#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use oya_vector_embeddings_kernel::{Embedding, TenantId, ports::EmbeddingStorePort};
    use oya_vector_embeddings_adapter::pgvector::PgVectorAdapter;
    use oya_vector_similarity_kernel::{
        DistanceMetric, SimilarityQuery, ports::SimilaritySearchPort,
    };
    use oya_vector_similarity_adapter::pgvector_hnsw::PgVectorHnswAdapter;

    #[sqlx::test(migrations = "../../migrations/vector")]
    async fn cross_tenant_search_returns_empty(pool: sqlx::PgPool) {
        let tenant_a: TenantId = Uuid::new_v4();
        let tenant_b: TenantId = Uuid::new_v4();

        let store = PgVectorAdapter::new(pool.clone());
        let searcher = PgVectorHnswAdapter::new(pool.clone());

        // Insert a distinctive embedding under tenant A
        let query_vector = vec![1.0f32; 1536];
        store.upsert(tenant_a, Embedding {
            object_id: Uuid::new_v4(),
            object_type: "entity".to_owned(),
            model_id: "text-embedding-3-small".to_owned(),
            vector: query_vector.clone(),
            metadata: serde_json::json!({"source": "isolation_test"}),
        }).await.unwrap();

        // Search using tenant B — must return 0 hits (RLS enforces isolation)
        let hits = searcher.search(tenant_b, SimilarityQuery {
            vector: query_vector,
            model_id: "text-embedding-3-small".to_owned(),
            object_type: Some("entity".to_owned()),
            metric: DistanceMetric::Cosine,
            top_k: 10,
            ef_search: None,
        }).await.unwrap();

        assert!(
            hits.is_empty(),
            "Tenant B must not retrieve Tenant A embeddings; RLS cross-tenant leak detected"
        );
    }

    #[sqlx::test(migrations = "../../migrations/vector")]
    async fn upsert_overwrites_same_object_model(pool: sqlx::PgPool) {
        let tenant: TenantId = Uuid::new_v4();
        let object_id = Uuid::new_v4();
        let store = PgVectorAdapter::new(pool.clone());

        let v1 = vec![1.0f32; 1536];
        let v2 = vec![0.5f32; 1536];

        let id1 = store.upsert(tenant, Embedding {
            object_id,
            object_type: "entity".to_owned(),
            model_id: "text-embedding-3-small".to_owned(),
            vector: v1,
            metadata: serde_json::json!({"version": 1}),
        }).await.unwrap();

        let id2 = store.upsert(tenant, Embedding {
            object_id,
            object_type: "entity".to_owned(),
            model_id: "text-embedding-3-small".to_owned(),
            vector: v2,
            metadata: serde_json::json!({"version": 2}),
        }).await.unwrap();

        // Upsert returns same row id (overwrite, not insert)
        assert_eq!(id1, id2, "Upsert must return the same EmbeddingId on overwrite");
    }
}
```

---

## 8. Acceptance Gate Commands

```bash
# 1. Cargo gates
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features

# 2. Fitness lanes
oya gate validate lean-a1 --phase P10-vector
oya gate validate lean-a2 --phase P10-vector

# 3. pgvector HNSW cosine search
cargo nextest run -p oya-vector-similarity-adapter --test pgvector_cosine_topk

# 4. Per-tenant isolation
cargo nextest run -p oya-vector-embeddings-application --test tenant_embedding_isolation

# 5. Upsert idempotency
cargo nextest run -p oya-vector-embeddings-application --test upsert_idempotent

# 6. Distance metric selection
cargo nextest run -p oya-vector-similarity-domain --test distance_metric_selection

# 7. Load test
k6 run tests/load/smoke-vector-search.js --env BASE_URL=http://localhost:8088
# Pass criteria: p99 ≤200ms at 1k RPS, 1536-dim cosine, top-10

# 8. grit done
grit done --agent m02-wave-a-executor
```

---

## 9. ICM Store Commands

```bash
icm store \
  -t context-oyatie \
  -c "IP-P10-vector merged; 12 crates; pgvector HNSW day-1 (m=16 ef_construction=128); per-tenant RLS isolation; EmbeddingStorePort/SimilaritySearchPort; cosine+dot+L2 metric selection; p99≤200ms at 1k RPS; next: P11-finance-library/impl-plan" \
  -i high \
  -k "M02,P10,impl-plan,vector,pgvector,HNSW,RAG"
```

---

## Next IP Pointer

`phases/P11-finance-library/impl-plan.md`
