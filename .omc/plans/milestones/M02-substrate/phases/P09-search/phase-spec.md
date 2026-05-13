---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P09-search
status: Proposed
acceptance_lanes: []
entry_gate: |
  M01-P05 complete; P02-ontology merged (Ontology objects are the primary
  searchable corpus); Postgres 16 with pgroonga extension available;
  mecab-ko / khaiii FFI available in build environment; cargo check exits 0.
exit_gate: |
  All search crates compile; pgroonga full-text index created on
  ontology.objects.payload; Tantivy index directory initialized per tenant;
  mecab-ko tokenization returns correct Korean morphemes for test strings;
  khaiii FFI binding compiles; per-tenant index isolation verified (tenant A
  cannot query tenant B index); k6 search p99≤50ms; grit done; ICM row emitted.
depends_on:
  - milestone: M01
    phase: P05-scaffold-locks
    reason: "workspace scaffold prerequisite"
  - milestone: M02
    phase: P02-ontology
    reason: "Ontology objects are the primary searchable corpus"
owner_team: council-search
---

# P09-search: Search substrate — pgroonga + Tantivy + Korean morphology (mecab-ko/khaiii FFI), per-tenant index isolation

## Purpose

This phase delivers the complete Search substrate. Two complementary engines are deployed: (1) pgroonga (Postgres extension, PGroonga 3.x) for SQL-integrated full-text search with native Korean support via the built-in groonga tokenizer; (2) Tantivy (Rust-native) for inverted-index search with sub-10ms query latency at scale, plugged with Korean morphology via mecab-ko (Dictionary-based) and khaiii (Kakao Hangul Analyzer III — deep-learning morpheme segmentation, FFI binding). Per-tenant index isolation means each tenant's search index is physically separate — no cross-tenant result leakage. Ontology objects are the primary corpus; product phases register additional searchable types. This phase targets Algolia / OpenSearch parity for the oyatie platform search surface.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `search` | `index`, `query`, `morphology` | `crates/oya-search-{index,query,morphology}-{kernel,domain,application,adapter}/`, `crates/oya-search-worker/`, `crates/oya-search-rest/`, `crates/oya-search-app/` | 3×4 + 1 worker + 1 rest + 1 app = 16 crates |

Naming justification:

```
NAME: oya-search-index-kernel
JUSTIFICATION:
- microservice = search: the full-text search substrate; pgroonga + Tantivy + KR morphology
- bc-tokens = index: the index management BC (create/rebuild/delete per-tenant indexes)
- layer = kernel: IndexPort + IndexedDocument types; zero I/O
- exemptions claimed: none

NAME: oya-search-morphology-kernel
JUSTIFICATION:
- microservice = search: same µservice
- bc-tokens = morphology: Korean morpheme analysis BC; mecab-ko + khaiii FFI;
  distinct from generic indexing and query execution
- layer = kernel: MorphologyAnalyzerPort trait; language-agnostic interface
  over both mecab-ko and khaiii backends
- exemptions claimed: none
```

### Out-of-scope

- Vector/semantic search — owned by P10-vector.
- Cross-language search (Japanese/Chinese) — deferred to M04+.
- Elasticsearch/OpenSearch compatibility layer — not in scope; native pgroonga+Tantivy only.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full DDL + IndexPort + MorphologyAnalyzerPort + pgroonga adapter + Tantivy adapter + mecab-ko/khaiii FFI + load test | pending | `council-search` |

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
oya gate validate lean-a1 --phase P09-search
oya gate validate lean-a2 --phase P09-search
oya gate validate lean-a3 --phase P09-search
oya gate validate lean-a4 --phase P09-search
```

### Search correctness gates

```bash
# pgroonga: Korean full-text search returns expected Ontology objects
cargo nextest run -p oya-search-query-adapter --test pgroonga_korean_search  # exit 0
# Tantivy: same query via Tantivy returns matching documents
cargo nextest run -p oya-search-query-adapter --test tantivy_korean_search   # exit 0
# mecab-ko morpheme tokenization
cargo nextest run -p oya-search-morphology-adapter --test mecab_ko_tokenize  # exit 0
# khaiii FFI: Korean morpheme analysis produces correct POS tags
cargo nextest run -p oya-search-morphology-adapter --test khaiii_pos_tags    # exit 0
# Per-tenant isolation: search on tenant B index returns 0 results for tenant A query
cargo nextest run -p oya-search-index-application --test tenant_index_isolation  # exit 0
```

### Load test gate

```bash
k6 run tests/load/smoke-search-query.js --env BASE_URL=http://localhost:8087
# Pass: p99 ≤50ms on Ontology Function search (per quality bar read-only target)
vegeta attack -rate=1000/s -duration=60s -targets=tests/load/search-targets.txt | vegeta report
# Pass: p99 ≤50ms; p999 ≤200ms; 0 errors at 1k RPS
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-search-index-kernel` | `kernel` | Yes — `IndexPort` | N/A | No |
| `oya-search-query-kernel` | `kernel` | Yes — `SearchQueryPort` | N/A | No |
| `oya-search-morphology-kernel` | `kernel` | Yes — `MorphologyAnalyzerPort` | N/A | No |
| `oya-search-index-adapter` | `adapter` | N/A | Yes — `PgroongaAdapter`, `TantivyAdapter` | No |
| `oya-search-morphology-adapter` | `adapter` | N/A | Yes — `MecabKoAdapter`, `KhaiiiAdapter` (FFI) | No |
| `oya-search-worker` | `worker` | N/A | No direct adapter | No |
| `oya-search-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-search-index-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait IndexPort: Send + Sync + sealed::Sealed {
    /// Index a document into the tenant's search index. Upsert semantics.
    async fn index_document(&self, tenant_id: TenantId, doc: IndexedDocument)
        -> Result<(), SearchError>;
    /// Delete a document by ID from the tenant index.
    async fn delete_document(&self, tenant_id: TenantId, doc_id: &DocumentId)
        -> Result<(), SearchError>;
    /// Rebuild the full tenant index from Ontology (used after schema migration).
    async fn rebuild_index(&self, tenant_id: TenantId) -> Result<IndexStats, SearchError>;
}

// oya-search-query-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait SearchQueryPort: Send + Sync + sealed::Sealed {
    /// Full-text search with optional object_type filter.
    /// Returns ranked hits with snippet highlight.
    async fn search(&self, tenant_id: TenantId, query: SearchQuery)
        -> Result<SearchResults, SearchError>;
    /// Typeahead / prefix autocomplete.
    async fn autocomplete(&self, tenant_id: TenantId, prefix: &str, limit: u32)
        -> Result<Vec<AutocompleteHit>, SearchError>;
}

// oya-search-morphology-kernel/src/ports.rs
pub trait MorphologyAnalyzerPort: Send + Sync + sealed::Sealed {
    /// Tokenize text into morphemes with POS tags. Language-agnostic interface.
    fn analyze(&self, text: &str, lang: Language) -> Result<Vec<Morpheme>, SearchError>;
    /// Returns stemmed / normalized tokens for index writing.
    fn normalize(&self, morphemes: &[Morpheme]) -> Vec<String>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P09-search` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P09-search` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P09-search` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P09-search` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `index` | `search` | pending |
| `query` | `search` | pending |
| `morphology` | `search` | pending |

---

## Grit Claim Symbols

```
crates/oya-search-index-kernel/src/ports.rs::IndexPort
crates/oya-search-query-kernel/src/ports.rs::SearchQueryPort
crates/oya-search-morphology-kernel/src/ports.rs::MorphologyAnalyzerPort
crates/oya-search-morphology-adapter/src/khaiii.rs::KhaiiiAdapter
crates/oya-search-index-adapter/src/tantivy.rs::TantivyAdapter
migrations/search/V001__search_init.sql::search_schema
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P09-search started; scope: 16 crates (index/query/morphology BCs); pgroonga+Tantivy dual engine; mecab-ko+khaiii Korean morphology FFI; per-tenant isolation" \
  -i high \
  -k "M02,P09,phase-start,search"

icm store \
  -t context-oyatie \
  -c "Phase P09-search complete; pgroonga Korean search green; Tantivy adapter green; khaiii FFI compiles; per-tenant isolation tested; p99≤50ms; next: P10-vector" \
  -i high \
  -k "M02,P09,phase-complete,search"
```

---

## References

- Bominal ADRs inherited: search quality bar from ADR-0107 (Ontology Functions p99≤50ms)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05, M02-P02-ontology
- unblocks: Wave-B product search surfaces (medical record search, HR employee search, connect people search)
