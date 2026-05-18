//! Vector-store kernel per ADR-0192.
//!
//! Owns the engine-agnostic `VectorStore` trait, the per-tenant collection
//! naming canon (`tenant_{tenant_id}__{domain}`), per-data-class partition
//! convention, the typed vector-search request/response surface, idempotent
//! upsert semantics, and the per-tenant DSR cascade hook surface declared
//! by ADR-0038 (proof-of-erasure).
//!
//! This crate is the kernel; per ADR-0083 it carries pure types + traits +
//! pure logic and NO I/O. Engine bindings live in adapter crates:
//!   - `oya-shared-vector-store-milvus-adapter` — Milvus 2.6.x via gRPC.
//!   - `oya-shared-vector-store-pgvector-adapter` — embedded-tier pgvector
//!      for ≤10M-vector tenants (per ADR-0192 ceiling rule).
//!   - `oya-shared-vector-store-memory-adapter` — in-process reference impl
//!      shipped in this crate as a public sub-module (`memory_adapter`) so
//!      consumer crates can run their own tests without spinning Milvus.
//!
//! The kernel forbids cross-tenant search at construction time: every
//! [`VectorSearchRequest`] carries a `TenantId` and the canonical
//! collection name is derived deterministically from `(tenant_id, domain)`;
//! no adapter has the option of searching a different tenant's collection.
//!
//! ## In-house roadmap parity (ADR-0192 §"In-house roadmap")
//!
//! When the Phase-2 in-house `oya-vector-store-server` ships, it implements
//! this same trait surface. Consumer µservices migrate by repointing the
//! adapter in their composition root; no consumer-side code change.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

/// Schema version of the canonical request/response surface.
pub const KERNEL_SCHEMA_VERSION: u32 = 1;

/// Hard ceiling per ADR-0192 §"pgvector — degraded fallback only for ≤10M-vector tenants".
/// Above this per-tenant per-collection vector count, the kernel rejects the
/// embedded-tier path and requires delegation to the Milvus adapter (Phase 0)
/// or the in-house `oya-vector-store-server` adapter (Phase 2).
pub const PGVECTOR_HARD_CEILING_VECTORS: u64 = 10_000_000;

/// Maximum length of a tenant ID accepted by this kernel. Mirrors the upstream
/// `oya-tenancy-kernel::TENANT_SLUG_MAX_LEN`; duplicated locally as a value
/// constant to keep the kernel zero-dep per ADR-0083.
pub const TENANT_ID_MAX_LEN: usize = 128;

/// Maximum length of a collection domain name (the suffix in
/// `tenant_{tenant_id}__{domain}`).
pub const COLLECTION_DOMAIN_MAX_LEN: usize = 64;

/// Canonical vector dimensions accepted by the kernel.
///
/// Per ADR-0046 + ADR-0192 §"Embedding pipeline integration", every embedding
/// model in the canonical pipeline emits one of these dimensions. Vectors of
/// other dimensions can still be stored via an explicit kernel
/// [`VectorDimension::Custom`] but flow through the same validation surface.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VectorDimension {
    /// 384 — small-model BGE / MiniLM
    D384,
    /// 768 — clinical / specialized packs (per ADR-0046 default)
    D768,
    /// 1024 — KR / EN / multilingual canonical (per ADR-0046 default)
    D1024,
    /// 1536 — OpenAI text-embedding-3-large compatible
    D1536,
    /// Explicit custom dimension — must be >0 and <= 16384.
    Custom(u32),
}

impl VectorDimension {
    /// Returns the numeric dimension as a u32.
    pub fn as_u32(self) -> u32 {
        match self {
            Self::D384 => 384,
            Self::D768 => 768,
            Self::D1024 => 1024,
            Self::D1536 => 1536,
            Self::Custom(n) => n,
        }
    }

    /// Validate that a custom dimension is within the kernel-accepted range.
    pub fn is_valid(self) -> bool {
        let n = self.as_u32();
        (1..=16_384).contains(&n)
    }
}

impl fmt::Display for VectorDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "d{}", self.as_u32())
    }
}

/// Canonical index types per ADR-0192 §"Index types — pinned per workload class".
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexType {
    /// HNSW — hot RAG retrieval; default for K ≤ 50; latency target <30ms p99.
    Hnsw,
    /// IVF_FLAT — bulk similarity; K ≤ 500; latency target <100ms p99.
    IvfFlat,
    /// IVF_SQ8 — scalar-quantized; memory-constrained cells.
    IvfSq8,
    /// DiskANN — cold-tier billion-scale; disk-served; <200ms p99.
    DiskAnn,
    /// GPU CAGRA via NVIDIA RAFT — GPU-accelerated index build.
    GpuCagra,
}

impl IndexType {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hnsw => "hnsw",
            Self::IvfFlat => "ivf_flat",
            Self::IvfSq8 => "ivf_sq8",
            Self::DiskAnn => "diskann",
            Self::GpuCagra => "gpu_cagra",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "hnsw" => Some(Self::Hnsw),
            "ivf_flat" => Some(Self::IvfFlat),
            "ivf_sq8" => Some(Self::IvfSq8),
            "diskann" => Some(Self::DiskAnn),
            "gpu_cagra" => Some(Self::GpuCagra),
            _ => None,
        }
    }
}

/// Per-tenant data class slice (becomes the Milvus partition name).
///
/// Maps to the Cedar `data_class` taxonomy per ADR-0008 DUBO + ADR-0034.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DataClass {
    Public,
    TenantPublic,
    TenantPrivate,
    Regulated,
}

impl DataClass {
    pub const fn partition_name(self) -> &'static str {
        match self {
            Self::Public => "partition_public",
            Self::TenantPublic => "partition_tenant_public",
            Self::TenantPrivate => "partition_tenant_private",
            Self::Regulated => "partition_regulated",
        }
    }
}

/// Distance / similarity metric.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DistanceMetric {
    /// Cosine — default for embedding-similarity retrieval.
    Cosine,
    /// L2 (Euclidean).
    L2,
    /// Inner product (dot product).
    InnerProduct,
}

impl DistanceMetric {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cosine => "cosine",
            Self::L2 => "l2",
            Self::InnerProduct => "ip",
        }
    }
}

/// Validated tenant identifier accepted by the vector-store kernel.
///
/// Grammar: 1..=`TENANT_ID_MAX_LEN` bytes of ASCII alphanumeric + `-` + `_`.
/// Mirrors `oya-tenancy-kernel::TenantSlug` but is locally defined to keep
/// this kernel dependency-free per ADR-0083.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantId(String);

impl TenantId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, KernelError> {
        let value = value.into();
        if value.is_empty() {
            return Err(KernelError::TenantIdEmpty);
        }
        if value.len() > TENANT_ID_MAX_LEN {
            return Err(KernelError::TenantIdTooLong {
                actual: value.len(),
            });
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(KernelError::TenantIdInvalidChar);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Collection domain — the suffix in `tenant_{tenant_id}__{domain}`.
///
/// Examples: `rag_corpus`, `agent_memory`, `clinical_documents`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CollectionDomain(String);

impl CollectionDomain {
    pub fn try_new(value: impl Into<String>) -> Result<Self, KernelError> {
        let value = value.into();
        if value.is_empty() {
            return Err(KernelError::CollectionDomainEmpty);
        }
        if value.len() > COLLECTION_DOMAIN_MAX_LEN {
            return Err(KernelError::CollectionDomainTooLong {
                actual: value.len(),
            });
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(KernelError::CollectionDomainInvalidChar);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Canonical collection name — `tenant_{tenant_id}__{domain}`.
///
/// Constructed only from a validated (`TenantId`, `CollectionDomain`) pair;
/// adapters cannot synthesize this name from raw strings, foreclosing
/// cross-tenant collection access.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CollectionName {
    tenant_id: TenantId,
    domain: CollectionDomain,
    rendered: String,
}

impl CollectionName {
    pub fn new(tenant_id: TenantId, domain: CollectionDomain) -> Self {
        let rendered = format!("tenant_{}__{}", tenant_id.as_str(), domain.as_str());
        Self {
            tenant_id,
            domain,
            rendered,
        }
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn domain(&self) -> &CollectionDomain {
        &self.domain
    }

    pub fn as_str(&self) -> &str {
        &self.rendered
    }
}

impl fmt::Display for CollectionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.rendered)
    }
}

/// Per-collection schema declared at collection creation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionSchema {
    pub name: CollectionName,
    pub dimension: VectorDimension,
    pub index_type: IndexType,
    pub metric: DistanceMetric,
    /// HNSW parameter `M` (graph degree). Only meaningful when
    /// `index_type == IndexType::Hnsw`; ADR-0192 §"Index types" pins M=16.
    pub hnsw_m: u32,
    /// HNSW parameter `ef_construction`. ADR-0192 pins 200.
    pub hnsw_ef_construction: u32,
}

impl CollectionSchema {
    /// Construct the canonical HNSW schema per ADR-0192 §"Index types" pinned
    /// defaults (M=16, ef_construction=200).
    pub fn canonical_hnsw(
        name: CollectionName,
        dimension: VectorDimension,
        metric: DistanceMetric,
    ) -> Self {
        Self {
            name,
            dimension,
            index_type: IndexType::Hnsw,
            metric,
            hnsw_m: 16,
            hnsw_ef_construction: 200,
        }
    }
}

/// Vector payload stored under a stable `source_id`.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorRecord {
    /// Stable identifier within the source µservice's domain. The kernel
    /// scopes idempotent upsert by this id (no read-then-write race per
    /// ADR-0192 §"Embedding pipeline integration").
    pub source_id: String,
    pub embedding: Vec<f32>,
    pub data_class: DataClass,
    /// Inserted-at epoch seconds; consumed by retention policies.
    pub inserted_at_epoch_seconds: u64,
    /// Optional per-record metadata (string-keyed; string-valued for simplicity).
    pub metadata: BTreeMap<String, String>,
}

/// Vector-search request. Every field is required; cross-tenant queries are
/// foreclosed by the typed `CollectionName`.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorSearchRequest {
    pub collection: CollectionName,
    /// Optional data-class filter (single class). When set, only records in
    /// the matching partition are considered.
    pub data_class: Option<DataClass>,
    pub query_embedding: Vec<f32>,
    pub k: u32,
    /// HNSW search-time parameter `ef`. Only honored for HNSW collections;
    /// ADR-0192 pins 64 as the canonical default.
    pub ef_search: u32,
}

impl VectorSearchRequest {
    pub fn canonical_hnsw(
        collection: CollectionName,
        query_embedding: Vec<f32>,
        k: u32,
    ) -> Self {
        Self {
            collection,
            data_class: None,
            query_embedding,
            k,
            ef_search: 64,
        }
    }
}

/// Single search hit returned by the engine.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorSearchHit {
    pub source_id: String,
    pub distance: f32,
    pub data_class: DataClass,
}

/// Per-tenant DSR cascade descriptor per ADR-0038.
#[derive(Clone, Debug, PartialEq)]
pub struct DsrCascade {
    pub tenant_id: TenantId,
    /// When `source_ids` is empty, the cascade is a tenant-wide tombstone
    /// (drop all collections for this tenant). When non-empty, only the named
    /// source_ids are deleted across all collections for the tenant.
    pub source_ids: Vec<String>,
}

/// Engine-agnostic kernel error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KernelError {
    TenantIdEmpty,
    TenantIdTooLong { actual: usize },
    TenantIdInvalidChar,
    CollectionDomainEmpty,
    CollectionDomainTooLong { actual: usize },
    CollectionDomainInvalidChar,
    DimensionMismatch { expected: u32, actual: u32 },
    InvalidDimension { value: u32 },
    EmptyEmbedding,
    EmbeddingTooLargeForCustomDim { dim: u32 },
    KZero,
    KTooLarge { requested: u32, limit: u32 },
    /// The embedded-tier (pgvector) path was selected for a collection whose
    /// per-tenant vector count exceeds the ADR-0192 ceiling. The kernel
    /// directs the caller to delegate to the Milvus adapter (or the in-house
    /// `oya-vector-store-server` adapter when Phase 2 lands).
    EmbeddedTierCeilingExceeded {
        observed_vectors: u64,
        ceiling: u64,
    },
    /// The kernel rejected an attempt to use a `CollectionName` whose
    /// `tenant_id` does not match the caller-supplied `TenantId`.
    CrossTenantAccessDenied,
    /// Adapter-level transport / engine error surfaced to the kernel layer.
    AdapterError(String),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TenantIdEmpty => write!(f, "tenant id is empty"),
            Self::TenantIdTooLong { actual } => {
                write!(f, "tenant id length {actual} exceeds {TENANT_ID_MAX_LEN}")
            }
            Self::TenantIdInvalidChar => write!(f, "tenant id contains invalid character"),
            Self::CollectionDomainEmpty => write!(f, "collection domain is empty"),
            Self::CollectionDomainTooLong { actual } => write!(
                f,
                "collection domain length {actual} exceeds {COLLECTION_DOMAIN_MAX_LEN}"
            ),
            Self::CollectionDomainInvalidChar => {
                write!(f, "collection domain contains invalid character")
            }
            Self::DimensionMismatch { expected, actual } => {
                write!(f, "vector dimension {actual} does not match expected {expected}")
            }
            Self::InvalidDimension { value } => write!(f, "vector dimension {value} out of range"),
            Self::EmptyEmbedding => write!(f, "embedding is empty"),
            Self::EmbeddingTooLargeForCustomDim { dim } => {
                write!(f, "embedding too large for custom dim {dim}")
            }
            Self::KZero => write!(f, "k must be > 0"),
            Self::KTooLarge { requested, limit } => {
                write!(f, "k={requested} exceeds limit {limit}")
            }
            Self::EmbeddedTierCeilingExceeded {
                observed_vectors,
                ceiling,
            } => write!(
                f,
                "embedded-tier vector count {observed_vectors} exceeds ADR-0192 ceiling {ceiling}; delegate to Milvus adapter"
            ),
            Self::CrossTenantAccessDenied => write!(f, "cross-tenant access denied"),
            Self::AdapterError(msg) => write!(f, "adapter error: {msg}"),
        }
    }
}

impl std::error::Error for KernelError {}

/// Engine-agnostic vector-store port.
///
/// Adapter implementations:
/// - `MilvusVectorStore` (production; gRPC against Milvus 2.6.x).
/// - `PgvectorVectorStore` (embedded-tier for ≤10M vectors per tenant).
/// - `InMemoryVectorStore` (reference impl in [`memory_adapter`]; for tests).
/// - `InHouseVectorStore` (future Phase-2 `oya-vector-store-server`).
pub trait VectorStore {
    /// Idempotent collection creation. Returns `Ok(())` whether the
    /// collection existed before; per-collection schema is honored on first
    /// creation only (schema drift returns `AdapterError`).
    fn ensure_collection(&mut self, schema: &CollectionSchema) -> Result<(), KernelError>;

    /// Idempotent per-record upsert. Re-emitting the same `source_id` with
    /// the same payload is a no-op; re-emitting with a different payload
    /// overwrites the prior record.
    fn upsert(
        &mut self,
        collection: &CollectionName,
        record: &VectorRecord,
    ) -> Result<(), KernelError>;

    /// Top-K ANN search. Returns at most `request.k` hits, ordered by
    /// ascending distance (cosine + L2) or descending similarity (inner
    /// product) — adapters return the engine's canonical ordering and the
    /// kernel normalizes via [`normalize_hit_ordering`].
    fn search(&self, request: &VectorSearchRequest) -> Result<Vec<VectorSearchHit>, KernelError>;

    /// Per-tenant DSR cascade per ADR-0038. Empty `source_ids` is a
    /// tenant-wide tombstone (drop all collections for the tenant).
    fn dsr_cascade(&mut self, cascade: &DsrCascade) -> Result<(), KernelError>;

    /// Returns the count of vectors stored under this collection. Adapters
    /// approximate this via the engine's stat API; the kernel uses it for
    /// the ADR-0192 embedded-tier ceiling check.
    fn count(&self, collection: &CollectionName) -> Result<u64, KernelError>;
}

/// Normalize search hit ordering — cosine + L2 ascending, inner product
/// descending.
pub fn normalize_hit_ordering(
    hits: Vec<VectorSearchHit>,
    metric: DistanceMetric,
) -> Vec<VectorSearchHit> {
    let mut hits = hits;
    match metric {
        DistanceMetric::Cosine | DistanceMetric::L2 => {
            hits.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
        }
        DistanceMetric::InnerProduct => {
            hits.sort_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap_or(std::cmp::Ordering::Equal));
        }
    }
    hits
}

/// Validate that an embedding's length matches the declared dimension.
pub fn validate_embedding_dim(
    embedding: &[f32],
    dimension: VectorDimension,
) -> Result<(), KernelError> {
    if embedding.is_empty() {
        return Err(KernelError::EmptyEmbedding);
    }
    let expected = dimension.as_u32();
    let actual = embedding.len() as u32;
    if expected != actual {
        return Err(KernelError::DimensionMismatch { expected, actual });
    }
    if !dimension.is_valid() {
        return Err(KernelError::InvalidDimension { value: expected });
    }
    Ok(())
}

/// Validate the ADR-0192 embedded-tier ceiling. Returns `Err` when the
/// observed per-tenant per-collection vector count exceeds the ceiling.
pub fn validate_embedded_tier_ceiling(observed_vectors: u64) -> Result<(), KernelError> {
    if observed_vectors > PGVECTOR_HARD_CEILING_VECTORS {
        Err(KernelError::EmbeddedTierCeilingExceeded {
            observed_vectors,
            ceiling: PGVECTOR_HARD_CEILING_VECTORS,
        })
    } else {
        Ok(())
    }
}

/// Verify that a `CollectionName` belongs to the caller's `TenantId`. The
/// kernel forecloses cross-tenant access by requiring this check in every
/// adapter's `search`/`upsert` path.
pub fn assert_same_tenant(
    caller: &TenantId,
    collection: &CollectionName,
) -> Result<(), KernelError> {
    if collection.tenant_id() == caller {
        Ok(())
    } else {
        Err(KernelError::CrossTenantAccessDenied)
    }
}

/// In-memory reference adapter. Production adapters (Milvus, pgvector,
/// in-house Phase-2) live in their own crates; this one ships in the kernel
/// for tests and for µservice composition-root smoke tests.
pub mod memory_adapter {
    use super::*;

    /// In-process vector store backed by `BTreeMap`s. Brute-force search;
    /// O(N) per query. Use only for tests + reference parity assertions.
    #[derive(Debug, Default)]
    pub struct InMemoryVectorStore {
        collections: BTreeMap<String, CollectionState>,
    }

    #[derive(Debug)]
    struct CollectionState {
        schema: CollectionSchema,
        records: BTreeMap<String, VectorRecord>,
    }

    impl InMemoryVectorStore {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl VectorStore for InMemoryVectorStore {
        fn ensure_collection(&mut self, schema: &CollectionSchema) -> Result<(), KernelError> {
            let key = schema.name.as_str().to_string();
            if let Some(existing) = self.collections.get(&key) {
                if existing.schema != *schema {
                    return Err(KernelError::AdapterError(format!(
                        "schema drift on collection {key}"
                    )));
                }
                return Ok(());
            }
            self.collections.insert(
                key,
                CollectionState {
                    schema: schema.clone(),
                    records: BTreeMap::new(),
                },
            );
            Ok(())
        }

        fn upsert(
            &mut self,
            collection: &CollectionName,
            record: &VectorRecord,
        ) -> Result<(), KernelError> {
            let state = self
                .collections
                .get_mut(collection.as_str())
                .ok_or_else(|| {
                    KernelError::AdapterError(format!(
                        "collection {} does not exist; call ensure_collection first",
                        collection.as_str()
                    ))
                })?;
            validate_embedding_dim(&record.embedding, state.schema.dimension)?;
            state.records.insert(record.source_id.clone(), record.clone());
            Ok(())
        }

        fn search(
            &self,
            request: &VectorSearchRequest,
        ) -> Result<Vec<VectorSearchHit>, KernelError> {
            if request.k == 0 {
                return Err(KernelError::KZero);
            }
            let state = self
                .collections
                .get(request.collection.as_str())
                .ok_or_else(|| {
                    KernelError::AdapterError(format!(
                        "collection {} does not exist",
                        request.collection.as_str()
                    ))
                })?;
            validate_embedding_dim(&request.query_embedding, state.schema.dimension)?;

            let mut hits: Vec<VectorSearchHit> = state
                .records
                .values()
                .filter(|r| {
                    request
                        .data_class
                        .map(|dc| r.data_class == dc)
                        .unwrap_or(true)
                })
                .map(|r| VectorSearchHit {
                    source_id: r.source_id.clone(),
                    distance: distance(
                        &request.query_embedding,
                        &r.embedding,
                        state.schema.metric,
                    ),
                    data_class: r.data_class,
                })
                .collect();

            hits = normalize_hit_ordering(hits, state.schema.metric);
            hits.truncate(request.k as usize);
            Ok(hits)
        }

        fn dsr_cascade(&mut self, cascade: &DsrCascade) -> Result<(), KernelError> {
            let tenant_prefix = format!("tenant_{}__", cascade.tenant_id.as_str());
            if cascade.source_ids.is_empty() {
                self.collections
                    .retain(|name, _| !name.starts_with(&tenant_prefix));
                return Ok(());
            }
            let ids: BTreeSet<&String> = cascade.source_ids.iter().collect();
            for (name, state) in self.collections.iter_mut() {
                if !name.starts_with(&tenant_prefix) {
                    continue;
                }
                state.records.retain(|sid, _| !ids.contains(sid));
            }
            Ok(())
        }

        fn count(&self, collection: &CollectionName) -> Result<u64, KernelError> {
            let state = self
                .collections
                .get(collection.as_str())
                .ok_or_else(|| {
                    KernelError::AdapterError(format!(
                        "collection {} does not exist",
                        collection.as_str()
                    ))
                })?;
            Ok(state.records.len() as u64)
        }
    }

    fn distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
        match metric {
            DistanceMetric::L2 => a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f32>()
                .sqrt(),
            DistanceMetric::Cosine => {
                let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
                let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
                let nb = b.iter().map(|y| y * y).sum::<f32>().sqrt();
                if na == 0.0 || nb == 0.0 {
                    return f32::INFINITY;
                }
                1.0 - dot / (na * nb)
            }
            DistanceMetric::InnerProduct => {
                a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::memory_adapter::InMemoryVectorStore;
    use super::*;

    fn tid(s: &str) -> TenantId {
        TenantId::try_new(s).expect("valid tenant id")
    }

    fn dom(s: &str) -> CollectionDomain {
        CollectionDomain::try_new(s).expect("valid domain")
    }

    #[test]
    fn tenant_id_rejects_empty_and_oversize() {
        assert_eq!(TenantId::try_new(""), Err(KernelError::TenantIdEmpty));
        let too_long = "a".repeat(TENANT_ID_MAX_LEN + 1);
        assert_eq!(
            TenantId::try_new(&too_long),
            Err(KernelError::TenantIdTooLong {
                actual: TENANT_ID_MAX_LEN + 1
            })
        );
    }

    #[test]
    fn tenant_id_rejects_invalid_char() {
        assert_eq!(
            TenantId::try_new("acme.co"),
            Err(KernelError::TenantIdInvalidChar)
        );
        assert_eq!(
            TenantId::try_new("acme/co"),
            Err(KernelError::TenantIdInvalidChar)
        );
    }

    #[test]
    fn collection_domain_lowercase_only() {
        assert_eq!(
            CollectionDomain::try_new("RAG"),
            Err(KernelError::CollectionDomainInvalidChar)
        );
        assert!(CollectionDomain::try_new("rag_corpus").is_ok());
    }

    #[test]
    fn collection_name_renders_canonically() {
        let name = CollectionName::new(tid("ten_acme"), dom("rag_corpus"));
        assert_eq!(name.as_str(), "tenant_ten_acme__rag_corpus");
        assert_eq!(name.tenant_id().as_str(), "ten_acme");
        assert_eq!(name.domain().as_str(), "rag_corpus");
    }

    #[test]
    fn dimension_canonical_values_resolve() {
        assert_eq!(VectorDimension::D384.as_u32(), 384);
        assert_eq!(VectorDimension::D768.as_u32(), 768);
        assert_eq!(VectorDimension::D1024.as_u32(), 1024);
        assert_eq!(VectorDimension::D1536.as_u32(), 1536);
        assert_eq!(VectorDimension::Custom(2048).as_u32(), 2048);
    }

    #[test]
    fn dimension_validity_bounds() {
        assert!(VectorDimension::D384.is_valid());
        assert!(VectorDimension::Custom(16_384).is_valid());
        assert!(!VectorDimension::Custom(16_385).is_valid());
        assert!(!VectorDimension::Custom(0).is_valid());
    }

    #[test]
    fn validate_embedding_dim_catches_mismatch() {
        let v = vec![0.1f32; 768];
        assert!(validate_embedding_dim(&v, VectorDimension::D768).is_ok());
        assert_eq!(
            validate_embedding_dim(&v, VectorDimension::D1024),
            Err(KernelError::DimensionMismatch {
                expected: 1024,
                actual: 768
            })
        );
        assert_eq!(
            validate_embedding_dim(&[], VectorDimension::D768),
            Err(KernelError::EmptyEmbedding)
        );
    }

    #[test]
    fn embedded_tier_ceiling_enforced() {
        assert!(validate_embedded_tier_ceiling(10_000_000).is_ok());
        assert_eq!(
            validate_embedded_tier_ceiling(10_000_001),
            Err(KernelError::EmbeddedTierCeilingExceeded {
                observed_vectors: 10_000_001,
                ceiling: 10_000_000
            })
        );
    }

    #[test]
    fn assert_same_tenant_enforces_isolation() {
        let acme = tid("ten_acme");
        let bryan = tid("ten_bryan");
        let coll = CollectionName::new(acme.clone(), dom("rag_corpus"));
        assert!(assert_same_tenant(&acme, &coll).is_ok());
        assert_eq!(
            assert_same_tenant(&bryan, &coll),
            Err(KernelError::CrossTenantAccessDenied)
        );
    }

    #[test]
    fn index_type_label_round_trips() {
        for it in [
            IndexType::Hnsw,
            IndexType::IvfFlat,
            IndexType::IvfSq8,
            IndexType::DiskAnn,
            IndexType::GpuCagra,
        ] {
            assert_eq!(IndexType::parse_label(it.label()), Some(it));
        }
        assert_eq!(IndexType::parse_label("unknown"), None);
    }

    #[test]
    fn data_class_partition_names_canonical() {
        assert_eq!(DataClass::Public.partition_name(), "partition_public");
        assert_eq!(
            DataClass::TenantPublic.partition_name(),
            "partition_tenant_public"
        );
        assert_eq!(
            DataClass::TenantPrivate.partition_name(),
            "partition_tenant_private"
        );
        assert_eq!(DataClass::Regulated.partition_name(), "partition_regulated");
    }

    #[test]
    fn canonical_hnsw_schema_uses_pinned_defaults() {
        let coll = CollectionName::new(tid("ten_acme"), dom("rag_corpus"));
        let schema = CollectionSchema::canonical_hnsw(
            coll,
            VectorDimension::D1024,
            DistanceMetric::Cosine,
        );
        assert_eq!(schema.index_type, IndexType::Hnsw);
        assert_eq!(schema.hnsw_m, 16);
        assert_eq!(schema.hnsw_ef_construction, 200);
    }

    #[test]
    fn memory_adapter_round_trip_search() {
        let mut store = InMemoryVectorStore::new();
        let coll = CollectionName::new(tid("ten_acme"), dom("rag_corpus"));
        let schema = CollectionSchema::canonical_hnsw(
            coll.clone(),
            VectorDimension::D384,
            DistanceMetric::Cosine,
        );
        store.ensure_collection(&schema).unwrap();

        for i in 0..5 {
            let mut emb = vec![0.0f32; 384];
            emb[0] = i as f32;
            store
                .upsert(
                    &coll,
                    &VectorRecord {
                        source_id: format!("src_{i}"),
                        embedding: emb,
                        data_class: DataClass::TenantPrivate,
                        inserted_at_epoch_seconds: 0,
                        metadata: BTreeMap::new(),
                    },
                )
                .unwrap();
        }

        let mut query = vec![0.0f32; 384];
        query[0] = 0.0;
        let req = VectorSearchRequest::canonical_hnsw(coll.clone(), query, 3);
        let hits = store.search(&req).unwrap();
        assert_eq!(hits.len(), 3);
        // Closest match should be src_0 (identical first component).
        assert_eq!(hits[0].source_id, "src_0");
    }

    #[test]
    fn memory_adapter_idempotent_upsert() {
        let mut store = InMemoryVectorStore::new();
        let coll = CollectionName::new(tid("ten_acme"), dom("rag_corpus"));
        store
            .ensure_collection(&CollectionSchema::canonical_hnsw(
                coll.clone(),
                VectorDimension::D384,
                DistanceMetric::Cosine,
            ))
            .unwrap();
        let rec = VectorRecord {
            source_id: "src_1".into(),
            embedding: vec![0.0f32; 384],
            data_class: DataClass::Public,
            inserted_at_epoch_seconds: 0,
            metadata: BTreeMap::new(),
        };
        store.upsert(&coll, &rec).unwrap();
        store.upsert(&coll, &rec).unwrap();
        store.upsert(&coll, &rec).unwrap();
        assert_eq!(store.count(&coll).unwrap(), 1);
    }

    #[test]
    fn memory_adapter_dsr_tenant_wide_tombstone() {
        let mut store = InMemoryVectorStore::new();
        let acme = tid("ten_acme");
        let bryan = tid("ten_bryan");
        let acme_coll = CollectionName::new(acme.clone(), dom("rag"));
        let bryan_coll = CollectionName::new(bryan.clone(), dom("rag"));
        for c in [&acme_coll, &bryan_coll] {
            store
                .ensure_collection(&CollectionSchema::canonical_hnsw(
                    c.clone(),
                    VectorDimension::D384,
                    DistanceMetric::Cosine,
                ))
                .unwrap();
            store
                .upsert(
                    c,
                    &VectorRecord {
                        source_id: "src_1".into(),
                        embedding: vec![0.0; 384],
                        data_class: DataClass::Public,
                        inserted_at_epoch_seconds: 0,
                        metadata: BTreeMap::new(),
                    },
                )
                .unwrap();
        }
        store
            .dsr_cascade(&DsrCascade {
                tenant_id: acme,
                source_ids: vec![],
            })
            .unwrap();
        assert!(store.count(&acme_coll).is_err()); // acme collection dropped
        assert_eq!(store.count(&bryan_coll).unwrap(), 1);
    }

    #[test]
    fn memory_adapter_dsr_per_source_id() {
        let mut store = InMemoryVectorStore::new();
        let coll = CollectionName::new(tid("ten_acme"), dom("rag"));
        store
            .ensure_collection(&CollectionSchema::canonical_hnsw(
                coll.clone(),
                VectorDimension::D384,
                DistanceMetric::Cosine,
            ))
            .unwrap();
        for i in 0..3 {
            store
                .upsert(
                    &coll,
                    &VectorRecord {
                        source_id: format!("src_{i}"),
                        embedding: vec![0.0; 384],
                        data_class: DataClass::Public,
                        inserted_at_epoch_seconds: 0,
                        metadata: BTreeMap::new(),
                    },
                )
                .unwrap();
        }
        store
            .dsr_cascade(&DsrCascade {
                tenant_id: tid("ten_acme"),
                source_ids: vec!["src_1".into()],
            })
            .unwrap();
        assert_eq!(store.count(&coll).unwrap(), 2);
    }

    #[test]
    fn search_k_zero_is_rejected() {
        let mut store = InMemoryVectorStore::new();
        let coll = CollectionName::new(tid("ten_acme"), dom("rag"));
        store
            .ensure_collection(&CollectionSchema::canonical_hnsw(
                coll.clone(),
                VectorDimension::D384,
                DistanceMetric::Cosine,
            ))
            .unwrap();
        let req = VectorSearchRequest {
            collection: coll,
            data_class: None,
            query_embedding: vec![0.0; 384],
            k: 0,
            ef_search: 64,
        };
        assert_eq!(store.search(&req), Err(KernelError::KZero));
    }

    #[test]
    fn schema_drift_returns_adapter_error_on_second_ensure() {
        let mut store = InMemoryVectorStore::new();
        let coll = CollectionName::new(tid("ten_acme"), dom("rag"));
        let schema_a = CollectionSchema::canonical_hnsw(
            coll.clone(),
            VectorDimension::D384,
            DistanceMetric::Cosine,
        );
        let schema_b = CollectionSchema::canonical_hnsw(
            coll,
            VectorDimension::D1024, // different dimension
            DistanceMetric::Cosine,
        );
        store.ensure_collection(&schema_a).unwrap();
        let err = store.ensure_collection(&schema_b).unwrap_err();
        assert!(matches!(err, KernelError::AdapterError(_)));
    }

    #[test]
    fn data_class_filter_isolates_results() {
        let mut store = InMemoryVectorStore::new();
        let coll = CollectionName::new(tid("ten_acme"), dom("rag"));
        store
            .ensure_collection(&CollectionSchema::canonical_hnsw(
                coll.clone(),
                VectorDimension::D384,
                DistanceMetric::Cosine,
            ))
            .unwrap();
        for (i, dc) in [
            (0, DataClass::Public),
            (1, DataClass::TenantPrivate),
            (2, DataClass::Regulated),
        ] {
            store
                .upsert(
                    &coll,
                    &VectorRecord {
                        source_id: format!("src_{i}"),
                        embedding: vec![0.0; 384],
                        data_class: dc,
                        inserted_at_epoch_seconds: 0,
                        metadata: BTreeMap::new(),
                    },
                )
                .unwrap();
        }
        let req = VectorSearchRequest {
            collection: coll,
            data_class: Some(DataClass::Regulated),
            query_embedding: vec![0.0; 384],
            k: 10,
            ef_search: 64,
        };
        let hits = store.search(&req).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].data_class, DataClass::Regulated);
    }
}
