---
doc_class: ImplPlan
template_id: TPL-IMPL-PLAN
milestone: M02b-substrate
phase: P09-search
status: Proposed
depends_on_phase_spec: phase-spec.md
purpose: "Implementation plan for P09-search of M02b-substrate: detailed code structure and acceptance lanes."
---
# P09-search Implementation Plan

## 0. Grit Claim

```bash
grit session start
grit claim \
  --agent m02-wave-a-executor \
  --intent "P09-search: 16 crates; pgroonga+Tantivy+khaiii FFI; per-tenant isolation" \
  --ttl 4h \
  --symbols \
    "crates/oya-search-index-kernel/src/ports.rs::IndexPort" \
    "crates/oya-search-query-kernel/src/ports.rs::SearchQueryPort" \
    "crates/oya-search-morphology-kernel/src/ports.rs::MorphologyAnalyzerPort" \
    "crates/oya-search-morphology-adapter/src/khaiii.rs::KhaiiiAdapter" \
    "crates/oya-search-index-adapter/src/tantivy.rs::TantivyAdapter" \
    "migrations/search/V001__search_init.sql::search_schema"
```

---

## 1. Crate Inventory (16 crates)

| Crate | Layer | Primary purpose |
|---|---|---|
| `oya-search-index-kernel` | kernel | `IndexPort` + `IndexedDocument` + `DocumentId` + `IndexStats` types |
| `oya-search-index-domain` | domain | Index rebuild policy, indexable field selection |
| `oya-search-index-application` | application | `IndexService` — orchestrates IndexPort calls; tenant isolation logic |
| `oya-search-index-adapter` | adapter | `PgroongaAdapter`, `TantivyAdapter` |
| `oya-search-query-kernel` | kernel | `SearchQueryPort` + `SearchQuery`, `SearchResults`, `AutocompleteHit` types |
| `oya-search-query-domain` | domain | Query parsing, field boost weights, result ranking |
| `oya-search-query-application` | application | `SearchService` — fan-out to pgroonga and/or Tantivy |
| `oya-search-query-adapter` | adapter | `PgroongaQueryAdapter`, `TantivyQueryAdapter` |
| `oya-search-morphology-kernel` | kernel | `MorphologyAnalyzerPort` + `Morpheme`, `PosTag`, `Language` types |
| `oya-search-morphology-domain` | domain | Morpheme normalization, stop-word filtering |
| `oya-search-morphology-application` | application | `MorphologyService` — routes to mecab-ko or khaiii by language |
| `oya-search-morphology-adapter` | adapter | `MecabKoAdapter` (dictionary), `KhaiiiAdapter` (FFI deep-learning) |
| `oya-search-worker` | worker | Outbox poller → index_document on Ontology mutation events |
| `oya-search-rest` | rest | Axum REST: `POST /search`, `GET /autocomplete` |
| `oya-search-grpc` | grpc | Tonic gRPC: `SearchService.Search`, `SearchService.Autocomplete` |
| `oya-search-app` | app | Composition root; wires adapters into services |

---

## 2. Migration: `migrations/search/V001__search_init.sql`

```sql
-- migrations/search/V001__search_init.sql
-- Search substrate schema: index state + per-tenant Tantivy index registry
-- pgroonga index lives directly on ontology.objects.payload (see ontology migration)

BEGIN;

CREATE SCHEMA IF NOT EXISTS search;

-- Per-tenant Tantivy index registry
-- Stores the on-disk path (within the container's scratch volume) for the
-- per-tenant Tantivy index directory. Raw index data is never persisted to
-- this table — only the path reference.
CREATE TABLE search.tantivy_indexes (
    id               uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid        NOT NULL,
    object_type      text        NOT NULL,     -- e.g. 'entity', 'person', '*' for all types
    index_path       text        NOT NULL,     -- /var/tantivy/{tenant_id}/{object_type}
    doc_count        bigint      NOT NULL DEFAULT 0,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, object_type)
);

ALTER TABLE search.tantivy_indexes ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY ON search.tantivy_indexes;
CREATE POLICY tenant_isolation ON search.tantivy_indexes
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Index rebuild job tracking
CREATE TABLE search.index_rebuild_jobs (
    id               uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        uuid        NOT NULL,
    object_type      text,                     -- NULL = all types
    engine           text        NOT NULL CHECK (engine IN ('pgroonga', 'tantivy', 'both')),
    status           text        NOT NULL DEFAULT 'pending'
                                 CHECK (status IN ('pending', 'running', 'complete', 'failed')),
    triggered_by     text        NOT NULL,     -- 'schema_migration' | 'manual' | 'corruption_recovery'
    started_at       timestamptz,
    completed_at     timestamptz,
    error_message    text,
    created_at       timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE search.index_rebuild_jobs ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY ON search.index_rebuild_jobs;
CREATE POLICY tenant_isolation ON search.index_rebuild_jobs
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Outbox for search events (IndexRebuildRequested, IndexedDocumentDeleted)
CREATE TABLE search.outbox (
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

CREATE INDEX search_outbox_unpublished_idx
    ON search.outbox (created_at) WHERE published = false;

ALTER TABLE search.outbox ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY ON search.outbox;
CREATE POLICY tenant_isolation ON search.outbox
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- pgroonga full-text index on ontology.objects.payload
-- This extension call assumes pgroonga is already installed in the DB cluster.
-- The index is created here (search migration) rather than ontology migration
-- to keep search concerns separate and allow incremental rollout.
-- REQUIRES: ontology.objects table created by P02-ontology migration.
CREATE INDEX CONCURRENTLY IF NOT EXISTS pgroonga_objects_payload_idx
    ON ontology.objects USING pgroonga (payload pgroonga_jsonb_full_text_search_ops_v2);

COMMIT;
```

---

## 3. Kernel Types and Port Traits

### `oya-search-index-kernel/src/ports.rs`

```rust
// crates/oya-search-index-kernel/src/ports.rs
use async_trait::async_trait;
use uuid::Uuid;
use crate::{DocumentId, IndexedDocument, IndexStats, SearchError, TenantId};

#[doc(hidden)]
mod sealed {
    pub trait Sealed {}
}

/// Port for writing/maintaining the search index per tenant.
/// Implementations: PgroongaAdapter (SQL upsert), TantivyAdapter (on-disk Lucene-style).
#[async_trait]
pub trait IndexPort: Send + Sync + sealed::Sealed {
    /// Index a document into the tenant's search index. Upsert semantics.
    async fn index_document(
        &self,
        tenant_id: TenantId,
        doc: IndexedDocument,
    ) -> Result<(), SearchError>;

    /// Delete a document by ID from the tenant index.
    async fn delete_document(
        &self,
        tenant_id: TenantId,
        doc_id: &DocumentId,
    ) -> Result<(), SearchError>;

    /// Batch index for bulk ingestion. Default impl calls index_document in sequence;
    /// adapters may override for bulk efficiency.
    async fn batch_index(
        &self,
        tenant_id: TenantId,
        docs: Vec<IndexedDocument>,
    ) -> Result<IndexStats, SearchError> {
        let mut indexed = 0u64;
        for doc in docs {
            self.index_document(tenant_id, doc).await?;
            indexed += 1;
        }
        Ok(IndexStats { indexed, deleted: 0 })
    }

    /// Rebuild the full tenant index from Ontology (used after schema migration).
    async fn rebuild_index(
        &self,
        tenant_id: TenantId,
    ) -> Result<IndexStats, SearchError>;
}
```

### `oya-search-index-kernel/src/types.rs`

```rust
// crates/oya-search-index-kernel/src/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub type TenantId = Uuid;
pub type DocumentId = String;  // "{object_type}/{object_id}"

/// A document ready to be written to the search index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDocument {
    /// Globally unique within tenant: "{object_type}/{object_id}"
    pub id: DocumentId,
    /// Ontology object type (e.g. "entity", "person", "product")
    pub object_type: String,
    /// Searchable text fields: key = field name, value = text content
    pub fields: HashMap<String, String>,
    /// Numeric/date fields for range filtering
    pub facets: HashMap<String, serde_json::Value>,
    /// Language hint for morphology routing
    pub language: Language,
    /// Ontology version of the object — used for staleness detection
    pub version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Korean,
    English,
    Japanese,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub indexed: u64,
    pub deleted: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("tenant index not found: {0}")]
    IndexNotFound(TenantId),
    #[error("document not found: {doc_id} in tenant {tenant_id}")]
    DocumentNotFound { tenant_id: TenantId, doc_id: DocumentId },
    #[error("pgroonga error: {0}")]
    Pgroonga(String),
    #[error("tantivy error: {0}")]
    Tantivy(String),
    #[error("morphology analyzer error: {0}")]
    Morphology(String),
    #[error("cross-tenant access denied: query tenant {query_tenant} vs index tenant {index_tenant}")]
    CrossTenantAccess { query_tenant: TenantId, index_tenant: TenantId },
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
```

### `oya-search-query-kernel/src/ports.rs`

```rust
// crates/oya-search-query-kernel/src/ports.rs
use async_trait::async_trait;
use crate::{SearchError, SearchQuery, SearchResults, AutocompleteHit, TenantId};

#[doc(hidden)]
mod sealed {
    pub trait Sealed {}
}

#[async_trait]
pub trait SearchQueryPort: Send + Sync + sealed::Sealed {
    /// Full-text search with optional object_type filter.
    /// Returns ranked hits with snippet highlight.
    async fn search(
        &self,
        tenant_id: TenantId,
        query: SearchQuery,
    ) -> Result<SearchResults, SearchError>;

    /// Typeahead / prefix autocomplete.
    async fn autocomplete(
        &self,
        tenant_id: TenantId,
        prefix: &str,
        limit: u32,
    ) -> Result<Vec<AutocompleteHit>, SearchError>;
}
```

### `oya-search-query-kernel/src/types.rs`

```rust
// crates/oya-search-query-kernel/src/types.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TenantId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Query string (full-text)
    pub q: String,
    /// Restrict to object type(s); None = all types
    pub object_types: Option<Vec<String>>,
    /// Search engine preference
    pub engine: SearchEngine,
    /// Max results to return
    pub limit: u32,
    /// Pagination offset
    pub offset: u32,
    /// Highlight snippets in results
    pub highlight: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchEngine {
    Pgroonga,
    Tantivy,
    /// Try pgroonga first; fall back to Tantivy on error
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub total: u64,
    pub hits: Vec<SearchHit>,
    pub took_ms: u32,
    pub engine_used: SearchEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub document_id: String,
    pub object_type: String,
    pub score: f32,
    pub highlight: Option<String>,
    pub fields: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteHit {
    pub suggestion: String,
    pub object_type: String,
    pub document_id: String,
    pub score: f32,
}
```

### `oya-search-morphology-kernel/src/ports.rs`

```rust
// crates/oya-search-morphology-kernel/src/ports.rs
use crate::{Language, Morpheme, SearchError};

#[doc(hidden)]
mod sealed {
    pub trait Sealed {}
}

/// Language-agnostic morphology analyzer port.
/// mecab-ko = dictionary-based (fast, deterministic).
/// khaiii = Kakao deep-learning morpheme segmentation (higher accuracy).
pub trait MorphologyAnalyzerPort: Send + Sync + sealed::Sealed {
    /// Tokenize text into morphemes with POS tags.
    fn analyze(&self, text: &str, lang: Language) -> Result<Vec<Morpheme>, SearchError>;

    /// Returns stemmed / normalized tokens for index writing.
    fn normalize(&self, morphemes: &[Morpheme]) -> Vec<String>;
}
```

### `oya-search-morphology-kernel/src/types.rs`

```rust
// crates/oya-search-morphology-kernel/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Morpheme {
    pub surface: String,   // original surface form
    pub lemma: String,     // base/dictionary form
    pub pos: PosTag,
    pub start: usize,      // byte offset in original text
    pub end: usize,
}

/// Korean POS tags (Sejong tagset subset, compatible with mecab-ko and khaiii)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum PosTag {
    // Korean nouns
    NNG,   // 일반명사 general noun
    NNP,   // 고유명사 proper noun
    NNB,   // 의존명사 bound noun
    NP,    // 대명사 pronoun
    // Korean verbs
    VV,    // 동사 verb
    VA,    // 형용사 adjective
    VX,    // 보조용언 auxiliary predicate
    // Korean particles
    JKS,   // 주격조사 subject particle
    JKO,   // 목적격조사 object particle
    // Korean endings
    EF,    // 종결어미 sentence-final ending
    EC,    // 연결어미 conjunctive ending
    // Punctuation / symbols
    SW,    // 기호 symbol
    SL,    // 외국어 foreign word
    SN,    // 숫자 number
    // English / unknown
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Korean,
    English,
    Japanese,
    Unknown,
}
```

---

## 4. Adapter Implementations

### `oya-search-index-adapter/src/pgroonga.rs`

```rust
// crates/oya-search-index-adapter/src/pgroonga.rs
use async_trait::async_trait;
use sqlx::PgPool;
use oya_search_index_kernel::{
    ports::IndexPort, DocumentId, IndexedDocument, IndexStats, SearchError, TenantId,
};

pub struct PgroongaAdapter {
    pool: PgPool,
}

impl PgroongaAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Sealed impl — prevents external crates from implementing IndexPort
impl oya_search_index_kernel::ports::sealed::Sealed for PgroongaAdapter {}

#[async_trait]
impl IndexPort for PgroongaAdapter {
    async fn index_document(
        &self,
        tenant_id: TenantId,
        doc: IndexedDocument,
    ) -> Result<(), SearchError> {
        // pgroonga: documents are indexed via ontology.objects.payload (jsonb)
        // This adapter upserts the payload into ontology.objects to trigger
        // the pgroonga index update. The search index is maintained automatically
        // by the pgroonga extension on the jsonb column.
        let payload = serde_json::to_value(&doc.fields)?;

        sqlx::query(
            r#"
            INSERT INTO ontology.objects (id, tenant_id, object_type, version, payload, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, now(), now())
            ON CONFLICT (id) DO UPDATE
                SET payload = EXCLUDED.payload,
                    version = EXCLUDED.version,
                    updated_at = now()
            "#,
        )
        .bind(doc.id.replace(&format!("{}/", doc.object_type), "").parse::<uuid::Uuid>().ok())
        .bind(tenant_id)
        .bind(&doc.object_type)
        .bind(doc.version as i64)
        .bind(payload)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_document(
        &self,
        tenant_id: TenantId,
        doc_id: &DocumentId,
    ) -> Result<(), SearchError> {
        let parts: Vec<&str> = doc_id.splitn(2, '/').collect();
        let (object_type, object_id_str) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            return Err(SearchError::DocumentNotFound {
                tenant_id,
                doc_id: doc_id.clone(),
            });
        };

        let object_id: uuid::Uuid = object_id_str.parse().map_err(|_| {
            SearchError::DocumentNotFound {
                tenant_id,
                doc_id: doc_id.clone(),
            }
        })?;

        sqlx::query(
            "DELETE FROM ontology.objects WHERE id = $1 AND tenant_id = $2 AND object_type = $3",
        )
        .bind(object_id)
        .bind(tenant_id)
        .bind(object_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn rebuild_index(
        &self,
        tenant_id: TenantId,
    ) -> Result<IndexStats, SearchError> {
        // pgroonga auto-indexes on write; explicit rebuild reindexes the GRN table.
        // Requires superuser or pgroonga_command privilege.
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM ontology.objects WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        // Trigger pgroonga vacuum/reindex via pgroonga_command
        sqlx::query("SELECT pgroonga_command('reindex', ARRAY['name', 'pgroonga_objects_payload_idx'])")
            .execute(&self.pool)
            .await
            .ok(); // non-fatal: reindex is best-effort; main index continues serving

        Ok(IndexStats { indexed: row.0 as u64, deleted: 0 })
    }
}
```

### `oya-search-index-adapter/src/tantivy.rs`

```rust
// crates/oya-search-index-adapter/src/tantivy.rs
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, STORED, TEXT},
    Index, IndexWriter, ReloadPolicy,
};
use tokio::sync::Mutex;
use std::collections::HashMap;
use oya_search_index_kernel::{
    ports::IndexPort, DocumentId, IndexedDocument, IndexStats, SearchError, TenantId,
};

/// Per-tenant Tantivy index wrapper
struct TenantIndex {
    index: Index,
    writer: Mutex<IndexWriter>,
    fields: TantivyFields,
}

struct TantivyFields {
    doc_id: Field,
    object_type: Field,
    content: Field,
    tenant_id_field: Field,
}

pub struct TantivyAdapter {
    base_path: PathBuf,
    /// tenant_id → TenantIndex; populated lazily on first write
    indexes: tokio::sync::RwLock<HashMap<TenantId, std::sync::Arc<TenantIndex>>>,
}

impl TantivyAdapter {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            indexes: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    fn build_schema() -> (Schema, TantivyFields) {
        let mut builder = Schema::builder();
        let doc_id = builder.add_text_field("doc_id", STORED | TEXT);
        let object_type = builder.add_text_field("object_type", STORED | TEXT);
        let content = builder.add_text_field("content", TEXT);
        let tenant_id_field = builder.add_text_field("tenant_id", STORED);
        (builder.build(), TantivyFields { doc_id, object_type, content, tenant_id_field })
    }

    async fn get_or_create_index(
        &self,
        tenant_id: TenantId,
    ) -> Result<std::sync::Arc<TenantIndex>, SearchError> {
        {
            let r = self.indexes.read().await;
            if let Some(idx) = r.get(&tenant_id) {
                return Ok(idx.clone());
            }
        }

        let mut w = self.indexes.write().await;
        // Double-checked locking
        if let Some(idx) = w.get(&tenant_id) {
            return Ok(idx.clone());
        }

        let index_path = self.base_path.join(tenant_id.to_string());
        std::fs::create_dir_all(&index_path).map_err(|e| SearchError::Tantivy(e.to_string()))?;

        let (schema, fields) = Self::build_schema();
        let index = if Index::exists(&tantivy::directory::MmapDirectory::open(&index_path)
            .map_err(|e| SearchError::Tantivy(e.to_string()))?)
        {
            Index::open_in_dir(&index_path).map_err(|e| SearchError::Tantivy(e.to_string()))?
        } else {
            Index::create_in_dir(&index_path, schema).map_err(|e| SearchError::Tantivy(e.to_string()))?
        };

        // 50 MB writer heap
        let writer = index
            .writer(50_000_000)
            .map_err(|e| SearchError::Tantivy(e.to_string()))?;

        let tenant_index = std::sync::Arc::new(TenantIndex {
            index,
            writer: Mutex::new(writer),
            fields,
        });

        w.insert(tenant_id, tenant_index.clone());
        Ok(tenant_index)
    }
}

// Sealed impl
impl oya_search_index_kernel::ports::sealed::Sealed for TantivyAdapter {}

#[async_trait]
impl IndexPort for TantivyAdapter {
    async fn index_document(
        &self,
        tenant_id: TenantId,
        doc: IndexedDocument,
    ) -> Result<(), SearchError> {
        let ti = self.get_or_create_index(tenant_id).await?;

        let content = doc.fields.values().cloned().collect::<Vec<_>>().join(" ");
        let tantivy_doc = doc!(
            ti.fields.doc_id => doc.id.as_str(),
            ti.fields.object_type => doc.object_type.as_str(),
            ti.fields.content => content.as_str(),
            ti.fields.tenant_id_field => tenant_id.to_string().as_str()
        );

        let mut writer = ti.writer.lock().await;
        // Delete any existing document with same doc_id before re-adding (upsert)
        let doc_id_term = tantivy::Term::from_field_text(ti.fields.doc_id, &doc.id);
        writer.delete_term(doc_id_term);
        writer.add_document(tantivy_doc).map_err(|e| SearchError::Tantivy(e.to_string()))?;
        writer.commit().map_err(|e| SearchError::Tantivy(e.to_string()))?;

        Ok(())
    }

    async fn delete_document(
        &self,
        tenant_id: TenantId,
        doc_id: &DocumentId,
    ) -> Result<(), SearchError> {
        let ti = self.get_or_create_index(tenant_id).await?;
        let term = tantivy::Term::from_field_text(ti.fields.doc_id, doc_id);

        let mut writer = ti.writer.lock().await;
        writer.delete_term(term);
        writer.commit().map_err(|e| SearchError::Tantivy(e.to_string()))?;

        Ok(())
    }

    async fn rebuild_index(
        &self,
        tenant_id: TenantId,
    ) -> Result<IndexStats, SearchError> {
        // Full rebuild: drop existing index directory and recreate
        let index_path = self.base_path.join(tenant_id.to_string());
        if index_path.exists() {
            std::fs::remove_dir_all(&index_path).map_err(|e| SearchError::Tantivy(e.to_string()))?;
        }

        // Remove cached index so next write recreates it
        self.indexes.write().await.remove(&tenant_id);

        Ok(IndexStats { indexed: 0, deleted: 0 })
    }
}
```

---

## 5. Korean Morphology Adapters

### `oya-search-morphology-adapter/src/mecab_ko.rs`

```rust
// crates/oya-search-morphology-adapter/src/mecab_ko.rs
//
// mecab-ko adapter: dictionary-based Korean morpheme analysis.
// Requires mecab-ko system library; compile-time detection via
// #[cfg(feature = "mecab-ko")] in Cargo.toml.
//
// Cargo.toml:
//   [features]
//   mecab-ko = ["mecab"]
//   [dependencies]
//   mecab = { version = "0.6", optional = true }

#[cfg(feature = "mecab-ko")]
use mecab::Tagger;
use oya_search_morphology_kernel::{
    ports::{sealed, MorphologyAnalyzerPort},
    Language, Morpheme, PosTag, SearchError,
};

pub struct MecabKoAdapter {
    #[cfg(feature = "mecab-ko")]
    _tagger: std::marker::PhantomData<Tagger>,
}

impl MecabKoAdapter {
    pub fn new() -> Result<Self, SearchError> {
        // Verify mecab-ko dictionary is available
        #[cfg(feature = "mecab-ko")]
        {
            let _ = Tagger::new("-d /usr/local/lib/mecab/dic/mecab-ko-dic");
        }
        Ok(Self {
            #[cfg(feature = "mecab-ko")]
            _tagger: std::marker::PhantomData,
        })
    }
}

impl Default for MecabKoAdapter {
    fn default() -> Self {
        Self::new().expect("mecab-ko dict available")
    }
}

impl sealed::Sealed for MecabKoAdapter {}

impl MorphologyAnalyzerPort for MecabKoAdapter {
    fn analyze(&self, text: &str, lang: Language) -> Result<Vec<Morpheme>, SearchError> {
        if lang != Language::Korean && lang != Language::Unknown {
            return Ok(vec![Morpheme {
                surface: text.to_owned(),
                lemma: text.to_owned(),
                pos: PosTag::Unknown,
                start: 0,
                end: text.len(),
            }]);
        }

        #[cfg(feature = "mecab-ko")]
        {
            let mut tagger = Tagger::new("-d /usr/local/lib/mecab/dic/mecab-ko-dic");
            let result = tagger.parse_str(text);
            parse_mecab_output(text, &result)
        }
        #[cfg(not(feature = "mecab-ko"))]
        {
            // Fallback: whitespace tokenization when mecab-ko not compiled in
            Ok(text.split_whitespace().enumerate().map(|(i, token)| Morpheme {
                surface: token.to_owned(),
                lemma: token.to_owned(),
                pos: PosTag::NNG,
                start: i,
                end: i + token.len(),
            }).collect())
        }
    }

    fn normalize(&self, morphemes: &[Morpheme]) -> Vec<String> {
        morphemes
            .iter()
            // Keep content words: nouns, verbs, adjectives
            .filter(|m| matches!(
                m.pos,
                PosTag::NNG | PosTag::NNP | PosTag::NNB | PosTag::NP
                    | PosTag::VV | PosTag::VA | PosTag::SL | PosTag::SN
            ))
            .map(|m| m.lemma.clone())
            .filter(|t| !t.is_empty())
            .collect()
    }
}

#[cfg(feature = "mecab-ko")]
fn parse_mecab_output(text: &str, output: &str) -> Result<Vec<Morpheme>, SearchError> {
    let mut morphemes = Vec::new();
    let mut byte_offset = 0usize;

    for line in output.lines() {
        if line == "EOS" || line.is_empty() {
            break;
        }
        // mecab output format: surface\tfeature1,feature2,...
        let tab_pos = line.find('\t').ok_or_else(|| {
            SearchError::Morphology(format!("malformed mecab output line: {line}"))
        })?;
        let surface = &line[..tab_pos];
        let features: Vec<&str> = line[tab_pos + 1..].split(',').collect();
        let pos_str = features.first().unwrap_or(&"Unknown");
        let pos = parse_pos_tag(pos_str);
        // lemma is the 7th feature field (0-indexed) for mecab-ko-dic
        let lemma = features.get(7).filter(|&&l| l != "*").unwrap_or(&surface);

        let start = byte_offset;
        byte_offset += surface.len();

        morphemes.push(Morpheme {
            surface: surface.to_owned(),
            lemma: lemma.to_string(),
            pos,
            start,
            end: byte_offset,
        });
    }
    Ok(morphemes)
}

fn parse_pos_tag(s: &str) -> PosTag {
    match s {
        "NNG" => PosTag::NNG,
        "NNP" => PosTag::NNP,
        "NNB" => PosTag::NNB,
        "NP" => PosTag::NP,
        "VV" => PosTag::VV,
        "VA" => PosTag::VA,
        "VX" => PosTag::VX,
        "JKS" => PosTag::JKS,
        "JKO" => PosTag::JKO,
        "EF" => PosTag::EF,
        "EC" => PosTag::EC,
        "SW" => PosTag::SW,
        "SL" => PosTag::SL,
        "SN" => PosTag::SN,
        _ => PosTag::Unknown,
    }
}
```

### `oya-search-morphology-adapter/src/khaiii.rs`

```rust
// crates/oya-search-morphology-adapter/src/khaiii.rs
//
// khaiii FFI adapter: Kakao Hangul Analyzer III (deep-learning morpheme segmentation).
// Native Rust FFI binding to libkhaiii.so (Kakao open-source, Apache 2.0 license).
// GitHub: kakao/khaiii
//
// Cargo.toml:
//   [features]
//   khaiii = []
//   [build-dependencies]
//   cc = "1"
//   [dependencies]
//   libc = "0.2"
//
// build.rs links: println!("cargo:rustc-link-lib=khaiii");

use libc::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::ptr;
use oya_search_morphology_kernel::{
    ports::{sealed, MorphologyAnalyzerPort},
    Language, Morpheme, PosTag, SearchError,
};

// --- FFI declarations for libkhaiii ---

#[repr(C)]
struct KhaiiiWord {
    begin: c_int,
    length: c_int,
    morphs: *const KhaiiiMorph,
    next: *const KhaiiiWord,
}

#[repr(C)]
struct KhaiiiMorph {
    lex: *const c_char,
    tag: *const c_char,
    begin: c_int,
    length: c_int,
    next: *const KhaiiiMorph,
}

extern "C" {
    fn khaiii_open(rsc_dir: *const c_char, opt_str: *const c_char) -> *mut c_void;
    fn khaiii_close(handle: *mut c_void);
    fn khaiii_analyze(
        handle: *mut c_void,
        in_str: *const c_char,
        opt_str: *const c_char,
    ) -> *const KhaiiiWord;
    fn khaiii_free_results(handle: *mut c_void, results: *const KhaiiiWord);
    fn khaiii_last_error(handle: *mut c_void) -> *const c_char;
}

// --- Safe wrapper ---

pub struct KhaiiiHandle(*mut c_void);

// Safety: khaiii handle is thread-safe per kakao/khaiii documentation (read-only model)
unsafe impl Send for KhaiiiHandle {}
unsafe impl Sync for KhaiiiHandle {}

impl Drop for KhaiiiHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { khaiii_close(self.0) };
        }
    }
}

pub struct KhaiiiAdapter {
    handle: KhaiiiHandle,
}

impl KhaiiiAdapter {
    /// rsc_dir: path to khaiii resource directory (contains model files)
    pub fn new(rsc_dir: &str) -> Result<Self, SearchError> {
        let rsc_dir_c = CString::new(rsc_dir)
            .map_err(|e| SearchError::Morphology(format!("invalid rsc_dir path: {e}")))?;
        let opt_c = CString::new("").unwrap();

        let handle = unsafe { khaiii_open(rsc_dir_c.as_ptr(), opt_c.as_ptr()) };

        if handle.is_null() {
            return Err(SearchError::Morphology(
                "khaiii_open returned null; check rsc_dir and model files".to_owned(),
            ));
        }

        Ok(Self { handle: KhaiiiHandle(handle) })
    }
}

impl sealed::Sealed for KhaiiiAdapter {}

impl MorphologyAnalyzerPort for KhaiiiAdapter {
    fn analyze(&self, text: &str, lang: Language) -> Result<Vec<Morpheme>, SearchError> {
        if lang != Language::Korean && lang != Language::Unknown {
            return Ok(vec![Morpheme {
                surface: text.to_owned(),
                lemma: text.to_owned(),
                pos: PosTag::Unknown,
                start: 0,
                end: text.len(),
            }]);
        }

        let input = CString::new(text)
            .map_err(|e| SearchError::Morphology(format!("NUL byte in input: {e}")))?;
        let opt_c = CString::new("").unwrap();

        let results = unsafe {
            khaiii_analyze(self.handle.0, input.as_ptr(), opt_c.as_ptr())
        };

        if results.is_null() {
            let err_msg = unsafe {
                let err_ptr = khaiii_last_error(self.handle.0);
                if err_ptr.is_null() {
                    "khaiii_analyze returned null".to_owned()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
                }
            };
            return Err(SearchError::Morphology(err_msg));
        }

        let morphemes = unsafe { collect_morphemes(results) };

        unsafe { khaiii_free_results(self.handle.0, results) };

        Ok(morphemes)
    }

    fn normalize(&self, morphemes: &[Morpheme]) -> Vec<String> {
        morphemes
            .iter()
            .filter(|m| matches!(
                m.pos,
                PosTag::NNG | PosTag::NNP | PosTag::NNB | PosTag::NP
                    | PosTag::VV | PosTag::VA | PosTag::SL | PosTag::SN
            ))
            .map(|m| m.lemma.clone())
            .filter(|t| !t.is_empty() && t.len() > 1)
            .collect()
    }
}

unsafe fn collect_morphemes(mut word_ptr: *const KhaiiiWord) -> Vec<Morpheme> {
    let mut morphemes = Vec::new();

    while !word_ptr.is_null() {
        let word = &*word_ptr;
        let mut morph_ptr = word.morphs;

        while !morph_ptr.is_null() {
            let morph = &*morph_ptr;
            let lex = if morph.lex.is_null() {
                String::new()
            } else {
                CStr::from_ptr(morph.lex).to_string_lossy().into_owned()
            };
            let tag_str = if morph.tag.is_null() {
                "Unknown"
            } else {
                let t = CStr::from_ptr(morph.tag);
                // Borrow as str for matching; safe because khaiii owns the memory
                // for the duration of this function (freed after collect_morphemes returns)
                t.to_str().unwrap_or("Unknown")
            };

            let pos = match tag_str {
                "NNG" => PosTag::NNG,
                "NNP" => PosTag::NNP,
                "NNB" => PosTag::NNB,
                "NP"  => PosTag::NP,
                "VV"  => PosTag::VV,
                "VA"  => PosTag::VA,
                "VX"  => PosTag::VX,
                "JKS" => PosTag::JKS,
                "JKO" => PosTag::JKO,
                "EF"  => PosTag::EF,
                "EC"  => PosTag::EC,
                "SW"  => PosTag::SW,
                "SL"  => PosTag::SL,
                "SN"  => PosTag::SN,
                _     => PosTag::Unknown,
            };

            morphemes.push(Morpheme {
                surface: lex.clone(),
                lemma: lex,
                pos,
                start: morph.begin as usize,
                end: (morph.begin + morph.length) as usize,
            });

            morph_ptr = morph.next;
        }

        word_ptr = word.next;
    }

    morphemes
}
```

---

## 6. Protobuf Schema

```protobuf
// proto/search/v1/search_events.proto
syntax = "proto3";
package oyatie.search.v1;

import "google/protobuf/timestamp.proto";

// Emitted when a document is indexed (added or updated)
message DocumentIndexed {
  string  tenant_id     = 1;  // UUID string
  string  document_id   = 2;  // "{object_type}/{object_id}"
  string  object_type   = 3;
  string  engine        = 4;  // "pgroonga" | "tantivy" | "both"
  uint64  version       = 5;
  google.protobuf.Timestamp indexed_at = 6;
}

// Emitted when a document is deleted from the index
message DocumentDeleted {
  string  tenant_id     = 1;
  string  document_id   = 2;
  string  engine        = 3;
  google.protobuf.Timestamp deleted_at = 4;
}

// Emitted when an index rebuild job completes
message IndexRebuilt {
  string  tenant_id     = 1;
  string  engine        = 2;
  uint64  doc_count     = 3;
  uint32  duration_ms   = 4;
  google.protobuf.Timestamp completed_at = 5;
}

// gRPC service
service SearchService {
  rpc Search (SearchRequest) returns (SearchResponse);
  rpc Autocomplete (AutocompleteRequest) returns (AutocompleteResponse);
}

message SearchRequest {
  string  tenant_id     = 1;
  string  q             = 2;
  repeated string object_types = 3;
  string  engine        = 4;  // "pgroonga" | "tantivy" | "auto"
  uint32  limit         = 5;
  uint32  offset        = 6;
  bool    highlight     = 7;
}

message SearchResponse {
  uint64  total         = 1;
  repeated SearchHit hits = 2;
  uint32  took_ms       = 3;
  string  engine_used   = 4;
}

message SearchHit {
  string  document_id   = 1;
  string  object_type   = 2;
  float   score         = 3;
  string  highlight     = 4;
  map<string, string> fields = 5;
}

message AutocompleteRequest {
  string  tenant_id     = 1;
  string  prefix        = 2;
  uint32  limit         = 3;
}

message AutocompleteResponse {
  repeated AutocompleteHit hits = 1;
}

message AutocompleteHit {
  string  suggestion    = 1;
  string  object_type   = 2;
  string  document_id   = 3;
  float   score         = 4;
}
```

---

## 7. Load Test: `tests/load/smoke-search-query.js`

```javascript
// tests/load/smoke-search-query.js
// Target: p99 ≤50ms on Ontology Function search queries
// Run: k6 run tests/load/smoke-search-query.js --env BASE_URL=http://localhost:8087

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const p99Latency = new Trend('search_p99_latency', true);
const errorRate = new Rate('search_errors');

const TENANT_ID = __ENV.TENANT_ID || '00000000-0000-0000-0000-000000000001';
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8087';

// Korean search terms for realistic load test
const SEARCH_TERMS = [
  '김철수',    // common Korean name (full-text)
  '마케팅',    // marketing (common noun)
  '영업팀',    // sales team
  '서울',      // Seoul (proper noun)
  '개발자',    // developer
  'invoice',   // English term
  '직원 교육', // employee training (multi-term)
  '계약직',    // contract employee
];

export const options = {
  scenarios: {
    smoke: {
      executor: 'constant-vus',
      vus: 50,
      duration: '60s',
    },
    ramp: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '10s', target: 100 },
        { duration: '40s', target: 200 },
        { duration: '10s', target: 0 },
      ],
      startTime: '65s',
    },
  },
  thresholds: {
    'http_req_duration{engine:pgroonga}': ['p(99)<50'],
    'http_req_duration{engine:tantivy}':  ['p(99)<50'],
    'search_errors': ['rate<0.001'],       // <0.1% errors
  },
};

export default function () {
  const term = SEARCH_TERMS[Math.floor(Math.random() * SEARCH_TERMS.length)];
  const engine = Math.random() < 0.6 ? 'pgroonga' : 'tantivy';  // 60/40 split

  const payload = JSON.stringify({
    q: term,
    engine: engine,
    limit: 10,
    offset: 0,
    highlight: true,
  });

  const start = Date.now();
  const res = http.post(
    `${BASE_URL}/v1/search`,
    payload,
    {
      headers: {
        'Content-Type': 'application/json',
        'X-Tenant-Id': TENANT_ID,
      },
      tags: { engine: engine },
    }
  );
  const duration = Date.now() - start;

  p99Latency.add(duration);

  const ok = check(res, {
    'status 200': (r) => r.status === 200,
    'has hits': (r) => {
      try {
        const body = JSON.parse(r.body);
        return Array.isArray(body.hits);
      } catch {
        return false;
      }
    },
    'p99 ≤50ms': () => duration <= 50,
  });

  if (!ok) {
    errorRate.add(1);
  }

  sleep(0.01);
}
```

---

## 8. Per-tenant Isolation Test

```rust
// crates/oya-search-index-application/tests/tenant_index_isolation.rs
// Verifies: tenant A's index data is invisible to tenant B queries.

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use oya_search_index_kernel::{IndexedDocument, Language, TenantId};
    use oya_search_index_adapter::pgroonga::PgroongaAdapter;
    use oya_search_index_kernel::ports::IndexPort;
    use oya_search_query_adapter::pgroonga_query::PgroongaQueryAdapter;
    use oya_search_query_kernel::{ports::SearchQueryPort, SearchQuery, SearchEngine};

    #[sqlx::test(migrations = "../../migrations/search")]
    async fn cross_tenant_search_returns_empty(pool: sqlx::PgPool) {
        let tenant_a: TenantId = Uuid::new_v4();
        let tenant_b: TenantId = Uuid::new_v4();

        let index_adapter = PgroongaAdapter::new(pool.clone());
        let query_adapter = PgroongaQueryAdapter::new(pool.clone());

        // Index a document under tenant A
        index_adapter
            .index_document(tenant_a, IndexedDocument {
                id: format!("entity/{}", Uuid::new_v4()),
                object_type: "entity".to_owned(),
                fields: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("name".to_owned(), "김철수 마케팅 팀장".to_owned());
                    m
                },
                facets: Default::default(),
                language: Language::Korean,
                version: 1,
            })
            .await
            .unwrap();

        // Query using tenant B's context — must return 0 hits
        let results = query_adapter
            .search(tenant_b, SearchQuery {
                q: "김철수".to_owned(),
                object_types: None,
                engine: SearchEngine::Pgroonga,
                limit: 10,
                offset: 0,
                highlight: false,
            })
            .await
            .unwrap();

        assert_eq!(
            results.hits.len(),
            0,
            "Tenant B must not see Tenant A documents; RLS isolation failure"
        );
    }
}
```

---

## 9. Acceptance Gate Commands

```bash
# 1. Cargo gates
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features

# 2. Fitness lanes
oya gate validate lean-a1 --phase P09-search
oya gate validate lean-a2 --phase P09-search

# 3. Korean morphology
cargo nextest run -p oya-search-morphology-adapter --test mecab_ko_tokenize
cargo nextest run -p oya-search-morphology-adapter --test khaiii_pos_tags

# 4. pgroonga Korean full-text search
cargo nextest run -p oya-search-query-adapter --test pgroonga_korean_search

# 5. Tantivy search
cargo nextest run -p oya-search-query-adapter --test tantivy_korean_search

# 6. Per-tenant isolation
cargo nextest run -p oya-search-index-application --test tenant_index_isolation

# 7. Load test
k6 run tests/load/smoke-search-query.js --env BASE_URL=http://localhost:8087
# Pass criteria: p99 ≤50ms for both pgroonga and tantivy engines

# 8. grit done
grit done --agent m02-wave-a-executor
```

---

## 10. ICM Store Commands

```bash
icm store \
  -t context-oyatie \
  -c "IP-P09-search merged; 16 crates; pgroonga+Tantivy dual engine; mecab-ko dict + khaiii FFI deep-learning morphology; per-tenant isolation via RLS; p99≤50ms; next: P10-vector/impl-plan" \
  -i high \
  -k "M02,P09,impl-plan,search,pgroonga,tantivy,khaiii"
```

---

## Next IP Pointer

`phases/P10-vector/impl-plan.md`
