# IP-094 — HNSW Tuning + Milvus Adapter Crate

**Phase:** PHASE-02-FOUNDRY-DATA-SUBSTRATE
**Owner:** backend (axis-foundry)
**Authority ADRs:** ADR-0192 §"Index types — pinned per workload class", ADR-0083 layer enum, ADR-0145 inter-microservice communication, ADR-0192 §"Naming + isolation primitives"
**Depends on:** IP-091
**Status:** Planned
**Phase trace:** PHASE-02 §"Adapter crate + recall benchmark" (addendum lines 44-50).

## Scope

Author the `oya-shared-vector-store-milvus-adapter` crate — the canonical adapter implementing the kernel's `VectorStore` trait against Milvus 2.6.x via gRPC (tonic + protobuf). This crate is the **only** code path that talks to Milvus; ingest (IP-093), retrieval (downstream IPs), tenant bootstrap (IP-092) all consume it.

The crate pins HNSW parameters per ADR-0192 §"Index types — pinned per workload class". Tuning knobs (`M`, `ef_construction`, `ef_search`) are exposed only at collection-creation time; runtime queries do not allow ef_search override per request (prevents tenant abuse of latency budget).

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `crates/oya-shared-vector-store-milvus-adapter/Cargo.toml` | create | 1-50 | tonic, prost, tokio, kernel + audit emitter deps |
| `crates/oya-shared-vector-store-milvus-adapter/build.rs` | create | 1-60 | invokes `tonic-build` against vendored Milvus .proto |
| `crates/oya-shared-vector-store-milvus-adapter/proto/milvus.proto` | vendor | n/a (~2KB) | upstream copy at vendored version |
| `crates/oya-shared-vector-store-milvus-adapter/proto/schema.proto` | vendor | n/a | upstream copy |
| `crates/oya-shared-vector-store-milvus-adapter/src/lib.rs` | create | 1-90 | re-exports + module roots |
| `crates/oya-shared-vector-store-milvus-adapter/src/adapter.rs` | create | 1-260 | `MilvusVectorStore` impl of kernel trait |
| `crates/oya-shared-vector-store-milvus-adapter/src/connection.rs` | create | 1-160 | pooled gRPC client; SPIFFE mTLS |
| `crates/oya-shared-vector-store-milvus-adapter/src/collection_ops.rs` | create | 1-220 | create / drop / describe / load / release |
| `crates/oya-shared-vector-store-milvus-adapter/src/partition_ops.rs` | create | 1-140 | per-data-class partition CRUD |
| `crates/oya-shared-vector-store-milvus-adapter/src/upsert.rs` | create | 1-180 | batch upsert with retry |
| `crates/oya-shared-vector-store-milvus-adapter/src/search.rs` | create | 1-200 | top-K search with ef_search per workload-class table |
| `crates/oya-shared-vector-store-milvus-adapter/src/index.rs` | create | 1-160 | index creation (HNSW / DiskANN / IVF_FLAT / GPU_CAGRA) |
| `crates/oya-shared-vector-store-milvus-adapter/src/error_mapping.rs` | create | 1-120 | Milvus error code → kernel error |
| `crates/oya-shared-vector-store-milvus-adapter/src/telemetry.rs` | create | 1-100 | OTel spans + metrics |
| `crates/oya-shared-vector-store-milvus-adapter/tests/integration/roundtrip.rs` | create | 1-200 | full lifecycle |
| `crates/oya-shared-vector-store-milvus-adapter/tests/integration/recall_msmarco.rs` | create | 1-180 | MS-MARCO ≥0.95 recall test |
| `crates/oya-shared-vector-store-milvus-adapter/tests/integration/p99_latency.rs` | create | 1-160 | ≤30ms p99 K=10 on 100K vectors |
| `crates/oya-shared-vector-store-milvus-adapter/tests/integration/index_types_matrix.rs` | create | 1-220 | HNSW / DiskANN / IVF_FLAT / GPU_CAGRA selection |
| `crates/oya-shared-vector-store-milvus-adapter/tests/integration/connection_pool_failover.rs` | create | 1-120 | proxy-node failover |

## Pinned parameters per workload class (per ADR-0192)

| Workload class | Index type | M | ef_construction | ef_search | Recall target | Latency target |
|---|---|---|---|---|---|---|
| Hot RAG retrieval (K ≤ 50, on-line) | **HNSW** | 16 | 200 | 64 | ≥ 0.95 | p99 ≤ 30ms |
| Conversational memory (K ≤ 20, low-cardinality) | **HNSW** | 16 | 200 | 32 | ≥ 0.92 | p99 ≤ 15ms |
| Semantic search corpus (K ≤ 100, mid-cardinality) | **HNSW** | 32 | 256 | 96 | ≥ 0.97 | p99 ≤ 80ms |
| Cold-tier corpus (rare reads, > 100M vectors) | **DiskANN** | n/a | n/a | n/a | ≥ 0.93 | p99 ≤ 200ms |
| Eval baseline-set (offline) | **IVF_FLAT** | n/a (nlist=4096) | n/a | n/a | 1.0 | best-effort |
| GPU-accelerated build (per IP-095) | **GPU_CAGRA** | n/a (graph_degree=32) | n/a | n/a | ≥ 0.95 | build 10x CPU |

## Adapter trait surface (consumed via `oya-shared-vector-store-kernel`)

```rust
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn create_collection(&self, spec: &CollectionSpec) -> Result<(), KernelError>;
    async fn drop_collection(&self, qname: &CollectionQName) -> Result<(), KernelError>;
    async fn create_partition(&self, qname: &CollectionQName, partition: &DataClass) -> Result<(), KernelError>;
    async fn upsert_batch(&self, qname: &CollectionQName, partition: &DataClass, vectors: &[Vector]) -> Result<UpsertReceipt, KernelError>;
    async fn search(&self, qname: &CollectionQName, partition: &DataClass, query: &SearchQuery) -> Result<SearchResults, KernelError>;
    async fn describe_collection(&self, qname: &CollectionQName) -> Result<CollectionState, KernelError>;
    async fn load_collection(&self, qname: &CollectionQName) -> Result<(), KernelError>;
    async fn release_collection(&self, qname: &CollectionQName) -> Result<(), KernelError>;
}
```

## Acceptance criteria

- **Round-trip test** — create collection + insert 100K vectors + search top-10 + delete tenant — succeeds against ephemeral Milvus (testcontainers).
- **HNSW recall ≥ 0.95** on the MS-MARCO eval set (`tests/integration/recall_msmarco.rs`).
- **p99 search latency ≤ 30ms** for K=10 on 100K-vector collection (`tests/integration/p99_latency.rs`).
- **Index types matrix** — all 6 workload classes pass per-class recall/latency targets.
- **Error mapping** — every Milvus error code is mapped to a kernel error variant; unknown codes panic in debug, map to `KernelError::Unexpected` in release (caught by clippy lint).
- **Connection pool** — proxy-node failover recovers within 5s without losing in-flight requests.
- **Compile fails** if an adapter consumer attempts to override `ef_search` per request (compile-fails-test).
- **SPIFFE mTLS** to Milvus proxy verified by `tests/integration/spiffe_mtls.rs`.

## Test plan

| Test | Verifies |
|---|---|
| `test_create_drop_collection` | basic lifecycle |
| `test_upsert_then_search_topk` | functional correctness |
| `test_partition_isolation` | search on `partition_pii` cannot see `partition_phi` rows |
| `test_msmarco_recall_at_10` | HNSW ≥ 0.95 |
| `test_p99_latency_k10_100k` | ≤ 30ms |
| `test_diskann_cold_tier_recall` | ≥ 0.93 on > 100M-class synthetic set |
| `test_ivf_flat_eval_exact_recall` | recall = 1.0 (brute) |
| `test_gpu_cagra_build_speedup` | requires GPU cell — gated test |
| `test_error_mapping_quota_exceeded` | Milvus 65535 → `KernelError::QuotaExceeded` |
| `test_connection_pool_proxy_failover` | proxy restart mid-flight; client recovers |
| `test_spiffe_mtls_required` | non-SPIFFE caller denied at proxy |
| `test_per_request_ef_search_override_compile_fails` | compile-fails-test |
| `test_telemetry_span_attributes` | span carries (collection, partition, k, latency, recall_est) |

## Evidence emission

- **Audit chain (ADR-0145):** adapter does NOT emit audit events directly — that responsibility lives in the calling app (ingest / tenant-bootstrap / retrieval). Adapter emits structured tracing only.
- **Metrics:** `vector_store_milvus_op_duration_seconds{op}` (histogram), `vector_store_milvus_op_errors_total{op,error_class}` (counter), `vector_store_milvus_recall_estimated` (gauge, populated by background eval-replay).
- **Tracing:** every adapter call is a root or child span depending on caller context.
- **Bench artifact:** `evidence/bench/milvus-adapter-msmarco-<commit>.json` on each release tag.

## Rollback procedure

1. The adapter is a library crate; rollback is a semver bump in callers (`oya-foundry-milvus-ingest-app`, `oya-foundry-milvus-tenant-bootstrap-app`, retrieval apps).
2. Per ADR-0145, the adapter follows the no-silent-regression policy — any breaking change is a major version bump + ADR.
3. If a recall or latency regression is detected post-merge (perf-budget lane catches it), revert PR via `git revert` + ship a patch.

## Blocking deps

- IP-091 (Milvus cluster available).
- ADR-0192 + ADR-0083 promoted to Accepted.
- `oya-shared-vector-store-kernel` published.
- Milvus .proto files vendored at the supported version (Milvus 2.6.x).

## Exit criteria

All test rows green; recall + p99 latency tests stable over 7 consecutive CI runs; MS-MARCO benchmark JSON in `evidence/bench/`; adapter consumed by IP-092 and IP-093 in dev cell; no clippy warnings; `cargo audit` clean.

## Layer placement (per ADR-0083)

This crate sits in the **adapter layer** of the 13-layer enum:

- **Kernel:** `oya-shared-vector-store-kernel` (port).
- **Adapter:** `oya-shared-vector-store-milvus-adapter` (this crate) — Milvus-specific implementation.
- **Composition root:** `oya-foundry-milvus-ingest-app`, `oya-foundry-milvus-tenant-bootstrap-app`, retrieval apps.

The adapter has zero compile-time dependencies on kernel callers (inward-only flow); kernel has zero compile-time dependencies on Milvus.

## Connection pooling

| Property | Value | Rationale |
|---|---|---|
| Pool size per app | 8 | Sustains 5K qps per app without head-of-line blocking |
| Max idle | 120s | Recycle stale TCP sockets |
| Connection retry | 3 with exp backoff (100ms / 500ms / 2s) | Bounded recovery time |
| Health probe | every 30s on idle conns | Detect dead proxies |
| Pool failover | on > 3 consecutive errors per conn | Recreate connection |

## Security posture

- `#![deny(unsafe_code)]` on the crate root.
- mTLS to the Milvus proxy via SPIFFE workload identity; `tonic`'s `ClientTlsConfig` carries the SPIFFE SVID.
- No secret material crosses the FFI boundary (Milvus user credentials are loaded at startup from ExternalSecret and held in an `Arc<SecretString>` with `zeroize` on drop).
- Compile-fails-test asserts the trait does not expose runtime `ef_search` override.

## References

- ADR-0192 §"Index types — pinned per workload class".
- ADR-0083 — layer enum.
- ADR-0145 — communication reform.
- Kernel crate: `oya-shared-vector-store-kernel`.
- OpenSLO: `microservices/intelligence/slos/milvus-search-latency.openslo.yaml`.
- Upstream Milvus protobuf: vendored at `crates/oya-shared-vector-store-milvus-adapter/proto/`.

## Wave 15 counterpart anchor

- Counterparts: Snowflake Cortex Search, Databricks Vector Search, OpenAI vector stores, and Palantir AIP ontology retrieval.
- Gap closure: this IP closes Foundry retrieval/vector substrate for tenant-isolated agent grounding and eval replay.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
