# Oyatie — Product PRD: Search Engine (Google/Naver-class)

> **Status:** draft → preview *(industry-standard labels per [GLOSSARY.md §11](../../GLOSSARY.md))*
> **Owning team:** [`teams/axis-search/CHARTER.md`](../../teams/axis-search/CHARTER.md)
> **Owning axis:** search
> **Catalog reference:** `registry/catalog/oya-search-*.yaml`
> **Last updated:** 2026-05-09 by Architecture Council

---

## 1. North star (required)

The Search axis is **the universal findability layer for every object Oyatie touches**: tenant-private SaaS objects, public Connect content, Marketplace listings, regulatory corpora, and (post-W-Search-Stable) the open web. It is also the **canonical retrieval surface for Foundry agents** — every agent capability that needs grounded retrieval (RAG, citation, source-of-record lookup, document-question-answering) calls into one search axis instead of re-implementing retrieval per axis. Sponsored slots in the public SERP land in the ads axis, sharing the same ranking signal. Without this axis, every Foundry agent re-invents retrieval, every SaaS tenant ships its own (stale, half-complete) per-vertical search, and Oyatie's "find anything Oyatie touches" promise is a marketing claim without substrate.

A standalone "Oyatie Search" public web search exists at W-Search-Stable as a real commercial product (KR-first SERP, then JP and beyond). The primary architectural job, however, is **non-leakage with SaaS, Vertical, Foundry, and Ads**: one Data Use Boundary (per [PRIVACY-PROGRAM.md §2](../../PRIVACY-PROGRAM.md)), one consent ladder, one audit-chain emission, one ranking signal shared with sponsored slots. **No search work begins before the Data Use Boundary ADR is Accepted** (per PRD §6).

## 2. Target users (required)

| Persona | What they get | What they pay for |
|---|---|---|
| **Tenant operator / end-user** (in-tenant search) | Per-tenant private full-text + vector search across Object Graph, workflow runs, plugin outputs, Connect threads (within consent boundary) | (Bundled with SaaS subscription; metered above tier) |
| **Foundry agent** | RAG endpoint (`search.query`, `search.retrieve_passages`, `search.cite_sources` capabilities per `product-control/capabilities/`); per-capability authorization (namespace × consent-tier × max-k); shared embedding cache | (Internal — agent-run cost metered to tenant) |
| **Public web searcher** (post-W-Search-Stable) | KR-first SERP (브라우저, mobile-web), then JP / EN; vertical search tabs (Shopping, News, Maps, Images, Videos); KR Naver-class UX expectations | (Free for end-user; ads pay) |
| **Tenant builder / IT** | Per-tenant index lifecycle controls; private corpus configuration; per-document indexability flags via Object Graph property tier | (Bundled with builder seats) |
| **Marketplace ISV** | Listing indexed in marketplace search; sponsored-listing eligibility (via ads axis at W-Ads-Preview) | Marketplace listing fees |
| **Sponsored advertiser** (cross-link to ads axis) | SERP-slot bidding through `oya-ads-auction-kernel`; quality signal derives from organic ranker | Ad spend (cross-axis to ads PRD) |
| **Regulator / privacy auditor** | Per-corpus class-allowed audit; index lifecycle audit-chain export; DSR cascade evidence | (Compliance — bundled) |
| **Vertical apps** (healthcare, fintech, industrial) | Per-vertical legal corpus search (ADR-0033, ADR-0033), per-vertical clinical/manufacturing/legal index | (Bundled with vertical subscription) |

## 3. In-scope / out-of-scope (required)

### 3.1 In-scope at each wave (preview / stable / GA)

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| **W-Foundation** | `Document`, `Index`, `Query`, `Result`, `Ranker` kernels (`oya-search-*-kernel`); per-tenant private inverted index (PG + pgroonga ADR-0047); pgvector vector index (ADR-0047); KR morphology (mecab-ko / khaiii); Data Use Boundary ADR (P0 prereq, blocks everything below) | None public — kernels and DUB ADR |
| **W-Substrate** | Foundry RAG endpoint (`search.query` capability per `product-control/capabilities/search.query.yaml`); per-capability authorization (I02 in greenfield); shared embedding cache; Foundry binding | Internal `search.query` and `search.retrieve_passages` capabilities surfaced to Foundry |
| **W-Search-Preview** | Tenant-private indexes for full-text + vector + faceted; Object Graph property → search-mapping cascade; per-class consent enforcement at index ingest; tenant-private query API; Foundry RAG fully wired; per-tenant index lifecycle (refresh / re-embed / DSR purge); KR-tokenizer regional pack; index sharding strategy | Tenant `Search API` (per-tenant private), Foundry `search.*` capability surface, internal RAG endpoint, tenant `Index Lifecycle` console |
| **W-Search-Stable** | Public web search: crawler (politeness, robots.txt, host-quota, sitemap), parser + enrichment (boilerplate strip, KR mecab, JP MeCab, language detection, per-vertical extractors), KG (knowledge graph) preview, freshness signal, ranker (lexical + vector + KG + click-through learn), SERP web frontend (KR-first per ADR-0037, ADR-0033), SERP API (internal+external), per-tab surfaces (Web/News/Images/Videos/Shopping/Maps); sponsored-result-slot infrastructure ready (ads serving still off — wired at W-Ads-Preview) | `Public SERP` (oya.com / 오야티.com), `Search API v1`, `News API`, `Image / Video / Map / Shopping API`, `Sponsored slot integration point` (no ad serving yet), KR ranking quality bar |
| **W-Public-GA** | SLA 99.99% on tenant-private query; SLA 99.95% on public SERP; full sponsored-result serving (with ads axis at W-Ads-Preview/Stable); freshness SLO; index coverage SLO per consent tier; KG with vertical extensions; trending-queries surface (safety-gated by design — per greenfield E11) | All surfaces SLA-backed; sponsored serving live |
| **W-Region-Fan-Out** | Per-pack tokenizer (KR / JP / EN / DE / FR / ES / PT / HI / AR / TH / VI), per-pack KG vertical extensions (KR-EDI 보건의료 / US-LOINC / JP-ReceiptCode), per-pack ranker tuning, per-pack UI typography & input (IME-aware) | Regional SERP surfaces (KR / JP / US / EU / IN / BR / KSA / UAE / ANZ / SG) |

### 3.2 Out-of-scope (anti-scope)

- Indexing tenant data without an explicit per-tenant per-class consent record (PHI / PII / PCI / KR-신용정보 / KR-PIPA-Art-23 → **HARD DENY** for cross-tenant indexing per PRIVACY-PROGRAM §2.2.1).
- Cross-tenant search index containing tenant-private documents without explicit `CROSS_TENANT_AGGREGATE` consent + k-anonymity ≥ 10.
- Consumer social-network search (per PRD §1.3 non-goal). Public search serves business intent; re-evaluate at W6+ if data shows organic consumer pull.
- Ranking driven by ad bid alone. Sponsored slots use ad-bid + organic-quality signal jointly; pure-bid ranking is forbidden.
- Real-time trending-queries surface in the Naver "실시간 급상승" style without safety design from day 1 (per greenfield E11; KR Naver discontinued for safety reasons).
- Ranker tuning by Foundry agents at autonomy ≥ T3 without human approval. Ranker changes are reversible-only via Argo Rollouts (ADR-0050) with metric-gated rollback.
- Forking the Object Graph property model. Search consumes `oya-platform-object-graph-kernel` and respects the `indexable` flag + tier annotation.
- Forking the canonical eventing backbone. Search uses Outbox + Kafka per ADR-0046.

## 4. Architecture overview (required) — *the slice-level architecture*

### 4.1 Bounded context

The Search axis owns the **`search` bounded context** per [DESIGN.md §1](../../DESIGN.md). Crate prefix:

- `crates/oya-search-{crawler,parser,index,rank,query,serp,kg,tokenizer}-*`

Per ADR-0015 §1: `oya-<context>-<role>[-<capability>]`.

### 4.2 Layered structure (clean architecture inside the bounded context)

```
kernel    — entities, invariants, no I/O
domain    — use cases, sealed-port traits
app       — orchestration, sagas, commands
adapter   — pgroonga, pgvector, Milvus (gated), Vespa (gated end-state per ADR-0047)
api       — inbound HTTP/gRPC servers (search query, RAG endpoint, SERP)
worker    — inbound queue/Kafka consumers (crawler, indexer, re-embedder, DSR purge)
runtime   — composition root
```

| Crate | Role | One-line role |
|---|---|---|
| `oya-search-document-kernel` | kernel | Document, Passage, Citation primitives |
| `oya-search-index-kernel` | kernel | Index, Mapping, Refresh policy, Shard |
| `oya-search-index-domain` | domain | Index lifecycle (create / refresh / re-embed / purge) |
| `oya-search-index-adapter-pgroonga` | adapter | Postgres + pgroonga full-text adapter (ADR-0047) |
| `oya-search-index-adapter-pgvector` | adapter | Postgres + pgvector vector adapter (ADR-0047) |
| `oya-search-index-adapter-milvus` | adapter | Milvus billion-scale vector (gated, ADR-0047) |
| `oya-search-index-adapter-opensearch` | adapter | OpenSearch (gated end-state, ADR-0047) |
| `oya-search-index-adapter-vespa` | adapter | Vespa hybrid+ML ranking (gated end-state, ADR-0047) |
| `oya-search-tokenizer-kernel` | kernel | Tokenizer trait + per-language family contracts |
| `oya-search-tokenizer-adapter-mecab-ko` | adapter | KR morphology (mecab-ko) |
| `oya-search-tokenizer-adapter-khaiii` | adapter | KR alternative morphology (khaiii) |
| `oya-search-tokenizer-adapter-mecab-jp` | adapter | JP MeCab + Sudachi |
| `oya-search-tokenizer-adapter-icu` | adapter | ICU + UAX-29 default for EN/EU/etc. |
| `oya-search-crawler-kernel` | kernel | CrawlPlan, FetchTask, Politeness, RobotsRules |
| `oya-search-crawler-domain` | domain | Scheduler, host-quota enforcement, retry/backoff curves |
| `oya-search-crawler-adapter` | adapter | HTTP/HTTPS fetcher; sitemap + robots.txt parser |
| `oya-search-crawler-worker` | worker | Crawl-task consumer |
| `oya-search-parser-kernel` | kernel | RawDocument → ParsedDocument trait + extracted fields |
| `oya-search-parser-domain` | domain | Boilerplate strip, charset detect, language detect, per-vertical extractor |
| `oya-search-parser-adapter` | adapter | Per-format parsers (HTML, PDF, EPUB, Office, plain) |
| `oya-search-rank-kernel` | kernel | Ranker trait, signal vector, SERP-slot config |
| `oya-search-rank-domain` | domain | Lexical + vector + KG + click-through fusion |
| `oya-search-rank-adapter` | adapter | Per-tenant tunable rank weights + RL-tuned weights (Foundry-driven) |
| `oya-search-query-kernel` | kernel | Query, FilterTree, Facet, Pagination, RankingHint |
| `oya-search-query-domain` | domain | Query understanding (intent, entity recognition, spell-correct, query rewrite) |
| `oya-search-query-api` | api | Tenant-private Search API + public SERP API + Foundry RAG endpoint |
| `oya-search-kg-kernel` | kernel | Entity, Relation, Triple, KGNode |
| `oya-search-kg-domain` | domain | KG construction (entity linking, relation extraction, vertical overlay) |
| `oya-search-kg-adapter` | adapter | Postgres + graph backend (Apache AGE day-1; Neo4j gated) |
| `oya-search-serp-kernel` | kernel | SerpComposition, FeaturedSnippet, Tab, SponsoredSlot |
| `oya-search-serp-app` | app | SERP composition saga |
| `oya-search-serp-frontend` | api | SERP web frontend (Leptos per ADR-0033) |
| `oya-search-rag-app` | app | RAG saga (retrieve → rerank → cite → emit audit) |
| `oya-search-cap-kernel` | kernel | Per-capability authorization (namespace × consent-tier × max-k) — greenfield I02 |
| `oya-search-cap-app` | app | Capability authorization enforcement |
| `oya-search-runtime` | runtime | Composition root |

### 4.3 External-facing surfaces

| Surface | Contract location | Plane (control / data / analytics) | SLO target |
|---|---|---|---|
| `Tenant Search API` (per-tenant private) | `contracts/search-tenant-api.openapi.yaml` | data | p99 ≤ 200 ms; 99.95% (preview) → 99.99% (GA) |
| `Public SERP Web Frontend` | `apps/oyatie-serp-fe/` (Leptos, ADR-0033) | data | p95 ≤ 500 ms; 99.95% (GA) |
| `Public SERP API v1` | `contracts/search-serp-api.openapi.yaml` | data | p99 ≤ 300 ms; 99.95% |
| `Foundry RAG Endpoint` (gRPC) | `contracts/search-rag.proto` | data + audit | p99 ≤ 250 ms; every call audit-emits |
| `Search Capability Surface` (`search.query`, `search.retrieve_passages`, `search.cite_sources`) | `product-control/capabilities/search.*.yaml` | data + audit | per-capability SLO; every call audit-emits |
| `Index Lifecycle Console` (tenant-side) | `apps/oyatie-search-console/` | control | p95 ≤ 1 000 ms; 99.9% |
| `Crawler Submit API` (sitemap submit, indexnow-equivalent) | `contracts/search-crawler-submit.openapi.yaml` | control | 99.9% |
| `KG Read API` | `contracts/search-kg-read.openapi.yaml` | data | p99 ≤ 100 ms |
| `Sponsored Slot Integration Point` | `contracts/search-sponsored-slot.openapi.yaml` | data + control | per-slot SLO; consumed by `oya-ads-*` |

### 4.4 Internal seams (depended on by other products)

| Seam | Trait / interface name | Consumer products |
|---|---|---|
| Search index lifecycle | `Index`, `Mapping`, `IndexRepo` in `oya-search-index-kernel` | Foundry (RAG ground), SaaS (tenant search), Ads (sponsored slot eligibility) |
| Document ingest | `Document`, `Passage` in `oya-search-document-kernel` | SaaS (Object Graph cascade), Vertical (clinical/manufacturing/legal corpora), Marketplace |
| Ranker signal | `RankSignal`, `Ranker` in `oya-search-rank-kernel` | Ads (quality score derives from organic-rank) |
| Query understanding | `Query`, `FilterTree`, `RankingHint` in `oya-search-query-kernel` | Foundry (capability authoring), SaaS (Workflow Studio query authoring) |
| KG read | `Entity`, `Relation`, `Triple` in `oya-search-kg-kernel` | Foundry (grounded reasoning), Vertical (vertical KG overlay), Ads (entity-targeted ad targeting) |
| Tokenizer | `Tokenizer` in `oya-search-tokenizer-kernel` | Vertical (per-vertical text), Connect (message indexing per ADR-0008) |
| Capability authorization | `SearchCap` in `oya-search-cap-kernel` | Foundry (every `search.*` capability call) |

### 4.5 Dependencies on other axes (cross-axis contracts)

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| Tenant kernel | SaaS | `oya-platform-tenant-kernel` | Cross-axis (mandatory all-axis) |
| Object Graph property tier | SaaS | `oya-platform-object-graph-kernel` (ADR-0006..0112) | Object-graph + Data Use Boundary check |
| Identity / Cedar policy | SaaS | `oya-platform-identity-kernel` | Two-ADR lockstep |
| Capability invocation | Foundry | `contracts/foundry-capability.openapi.yaml` | Cross-axis (foundry + search) |
| Autonomy ceiling | Foundry | `oya-foundry-policy-kernel` | Governance + security |
| Audit-chain event | SaaS / Audit | `oya-platform-audit-chain-kernel` | Audit + downstream-consumer review |
| Eventing backbone | SaaS | `oya-platform-eventing-kernel` | Cross-axis on topic shape |
| DSR cascade | SaaS | `oya-platform-dsr-kernel` | All data-touching axes mandatory ack |
| Ad slot inventory (SERP) | Ads | `oya-ads-slot-kernel` | Ads + SERP-owner |
| Cloud Region / Cell / Bucket | Cloud | `oya-cloud-region-kernel`, `oya-cloud-storage-kernel` | Multi-axis (residency-impact) |
| Regulatory pack (per-vertical legal corpus, ADR-0033, ADR-0033) | SaaS / Vertical | `oya-platform-regulatory-kernel` | Vertical + regulatory review |

(Mirror in [DESIGN.md §10](../../DESIGN.md).)

## 5. Data structures (required) — *the slice-level domain model*

### 5.1 Kernel entities (in `crates/oya-search-*-kernel`)

```rust
// oya-search-document-kernel
pub struct Document {
    pub id: DocumentId,                                // ulid
    pub tenant_id: TenantId,                            // every record carries tenant
    pub namespace: NamespaceId,                         // index namespace (per-tenant or public)
    pub source_kind: SourceKind,                        // og_entity | crawled_url | plugin_output | message
    pub source_ref: SourceRef,                          // FK to Object Graph entity, URL, etc.
    pub region: RegionCode,                             // for cell-routing
    pub data_class: DataClass,                          // per [PRIVACY-PROGRAM §2.2.1]; refusal at ingest if disallowed
    pub language: LanguageTag,                          // ISO 639-1 / 639-3
    pub content_hash: ContentHash,                      // for dedupe
    pub title: Option<String>,                          // data_class inherited
    pub body: DocumentBody,                             // tokenized + chunked at ingest
    pub passages: Vec<Passage>,                         // for retrieve_passages
    pub embedding: Option<EmbeddingVec>,                // dim per ADR-0006; cached embedding
    pub embedding_model: Option<EmbeddingModelRef>,
    pub kg_links: Vec<KgEntityRef>,                     // entity-linked to KG
    pub freshness_score: f32,                           // per E03/E04 freshness signal
    pub indexed_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: data
// data_class: declared per document; CI fitness checks against tenant consent

pub struct Passage {
    pub id: PassageId,
    pub document_id: DocumentId,
    pub seq: u32,                                       // chunk ordinal
    pub text: String,                                   // 100..512 token chunk
    pub embedding: EmbeddingVec,                        // for vector search
    pub data_class: DataClass,                          // inherited from document
    pub citation_anchor: Option<CitationAnchor>,        // for cite_sources
    pub schema_version: u32,
}
// plane: data
```

```rust
// oya-search-index-kernel
pub struct Index {
    pub id: IndexId,
    pub tenant_id: Option<TenantId>,                    // None = public corpus
    pub namespace: NamespaceId,
    pub kind: IndexKind,                                // FullText | Vector | Hybrid | Faceted | Geo | KG
    pub mapping: IndexMapping,                          // field → analyzer + type
    pub allowed_data_classes: BTreeSet<DataClass>,      // ingestion gate
    pub refresh_policy: RefreshPolicy,                  // realtime | batch_15m | hybrid | on_demand
    pub region: RegionCode,                             // residency
    pub shard_count: u16,                               // per-namespace shard count
    pub replica_count: u8,                              // per-shard replica
    pub backend: SearchBackend,                         // pgroonga | pgvector | milvus | opensearch | vespa
    pub state: IndexState,                              // building | live | rebuilding | retiring
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control
// data_class: PUBLIC (metadata)

pub struct IndexMapping {
    pub field_defs: Vec<FieldDef>,                      // (name, type, analyzer, tier_class)
    pub vector_dim: Option<u16>,                        // for vector indexes
    pub embedding_model_ref: Option<EmbeddingModelRef>,
    pub tokenizer_chain: Vec<TokenizerRef>,             // per-language morph
    pub kg_extraction_profile: Option<KgExtractionProfile>,
}
```

```rust
// oya-search-query-kernel
pub struct Query {
    pub id: QueryId,                                    // request-scoped ulid
    pub tenant_id: Option<TenantId>,                    // None = anonymous public
    pub principal: Option<PrincipalId>,                 // None = anonymous
    pub namespace: NamespaceId,                         // tenant-private | tenant-public | global-public
    pub raw: String,                                    // user-typed query
    pub parsed: ParsedQuery,                            // intent, entities, filters, hints
    pub locale: LocaleTag,                              // for tokenizer + UX
    pub region: RegionCode,                             // for cell-routing
    pub max_k: u16,                                     // ≤ capability cap
    pub consent_tier_at_call: ConsentTier,              // copied at evaluation time
    pub data_class: DataClass,                          // PUBLIC (the query metadata)
    pub data_classes_eligible: BTreeSet<DataClass>,     // computed from consent
    pub issued_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: data + audit (every query audit-emits)

pub struct Result {
    pub query_id: QueryId,
    pub hits: Vec<Hit>,
    pub facets: BTreeMap<FacetKey, Vec<FacetBucket>>,
    pub kg_panel: Option<KgPanel>,
    pub featured_snippet: Option<FeaturedSnippet>,
    pub sponsored_slots: Vec<SponsoredSlotRef>,         // ads axis fills these
    pub ranker_signals: RankerSignals,
    pub data_class: DataClass,                          // hits are filtered to data_classes_eligible
    pub schema_version: u32,
}
// plane: data

pub struct Hit {
    pub document_id: DocumentId,
    pub passage_ids: Vec<PassageId>,                    // when retrieve_passages
    pub score: RankScore,
    pub citation_anchors: Vec<CitationAnchor>,
    pub data_class: DataClass,
}
```

```rust
// oya-search-crawler-kernel
pub struct CrawlPlan {
    pub id: CrawlPlanId,
    pub seed_urls: Vec<Url>,
    pub host_quota: HostQuota,                          // requests/sec/host
    pub politeness_window_ms: u32,
    pub robots_compliance: RobotsCompliance,
    pub language_preference: Vec<LanguageTag>,
    pub region: RegionCode,
    pub data_class: DataClass,                          // PUBLIC (crawled web)
    pub freshness_target: Duration,                     // re-crawl cadence
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: control

pub struct FetchTask {
    pub id: FetchTaskId,
    pub crawl_plan_id: CrawlPlanId,
    pub url: Url,
    pub priority: FetchPriority,
    pub attempt: u8,
    pub backoff_until: Option<DateTime<Utc>>,
    pub state: FetchState,                              // pending | fetching | succeeded | failed | refused
    pub last_status: Option<HttpStatus>,
    pub data_class: DataClass,                          // PUBLIC for open web; vertical-pack overrides for legal corpora (ADR-0033)
    pub schema_version: u32,
}
// plane: data
```

```rust
// oya-search-rank-kernel
pub struct Ranker {
    pub id: RankerId,
    pub namespace: NamespaceId,
    pub kind: RankerKind,                               // BM25 | DenseVector | Hybrid | LearnedToRank
    pub signal_weights: SignalWeights,                  // {bm25, dense, freshness, kg, click}
    pub model_ref: Option<ModelRef>,                    // for L2R
    pub trained_at: Option<DateTime<Utc>>,
    pub eval_metric: Option<EvalMetric>,                // nDCG@10, MRR, ...
    pub state: RankerState,                             // shadow | canary | live | retired
    pub data_class: DataClass,                          // PUBLIC (model metadata; training data class declared separately)
    pub schema_version: u32,
}
// plane: control

pub struct SponsoredSlot {
    pub id: SponsoredSlotRef,
    pub query_id: QueryId,
    pub position: SlotPosition,                         // top1 | top2 | sidebar | bottom
    pub ad_id: Option<AdId>,                            // filled by ads axis
    pub ad_quality_score: Option<f32>,                  // derived from organic ranker; cross-axis seam
    pub bid_value: Option<Money>,                       // filled by ads axis
    pub data_class: DataClass,                          // PUBLIC (slot metadata)
    pub schema_version: u32,
}
// plane: data + control (cross-axis)
```

```rust
// oya-search-kg-kernel
pub struct KgEntity {
    pub id: KgEntityId,
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub entity_type: EntityType,                        // person | org | place | concept | product | medical_concept
    pub vertical_overlay: Option<VerticalKind>,         // medical (LOINC, ICD-10, KR-EDI), legal (ADR-0033), finance
    pub source_refs: Vec<SourceRef>,                    // documents linking here
    pub embedding: Option<EmbeddingVec>,
    pub data_class: DataClass,                          // PUBLIC for canonical entities; PII_QUASI for person entities (k-anonymity required)
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}
// plane: data

pub struct KgRelation {
    pub id: KgRelationId,
    pub head: KgEntityId,
    pub relation_type: RelationType,
    pub tail: KgEntityId,
    pub confidence: f32,
    pub source_refs: Vec<SourceRef>,
    pub data_class: DataClass,
    pub schema_version: u32,
}
```

### 5.2 Aggregate boundaries

- **Document aggregate**: `Document` + its `Passage[]` cluster as one unit (consistent ingest); embeddings update separately on re-embed.
- **Index aggregate**: `Index` + `IndexMapping` change as one unit (mapping change = re-build).
- **Ranker aggregate**: `Ranker` + `SignalWeights` change as one unit (canary→live promotion via Argo Rollouts ADR-0050).
- **CrawlPlan aggregate**: `CrawlPlan` + active `FetchTask[]` cluster (host-quota constraint).
- **KG aggregate**: `KgEntity` is the consistency boundary; `KgRelation[]` cluster on the source entity.
- **Query / Result**: stateless; per-request transient.

### 5.3 Persistence layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Document (tenant-private) | Postgres + pgroonga + pgvector (per-region) | `(tenant_id, namespace)` | per-tenant per-namespace | 3-AZ; cross-region per residency | DSR-purge cascade; per-class retention |
| Document (public crawled) | Postgres + pgroonga + pgvector → Vespa (gated end-state per ADR-0047) | `url_hash` | per-shard hash-partition | 3-AZ + cross-region read | TTL per freshness target |
| Passage | Same store as Document | inherited | inherited | inherited | inherited |
| Index metadata | Postgres | `(tenant_id, namespace)` | per-tenant | 3-AZ | indefinite |
| Ranker | Postgres + Object store for model artifacts | `namespace` | per-namespace | 3-AZ + Object store cross-region | versioned indefinitely |
| CrawlPlan / FetchTask | Postgres (per-region) + Redis (active fetch queue) | `host` | per-host bucket | 3-AZ | rolling 30 d |
| KG Entity / Relation | Postgres + Apache AGE (graph) day-1; Neo4j gated | `entity_type` | per-type partition | 3-AZ | indefinite (cascade on DSR for PII-class) |
| Query log | ClickHouse (ADR-0045) | `tenant_id` + time | per-tenant per-day | 3-AZ + cold to Iceberg per ADR-0045 | 90 d (private query); 7y (anonymized aggregate) |
| Click-through stream | ClickHouse | `tenant_id` + time | per-tenant per-day | 3-AZ + cold | 90 d full; 1y aggregate |
| Embedding cache | Redis + per-region object store | `model_ref + content_hash` | content-hash-sharded | 3-replica + object store | per-model TTL |
| Audit-chain block (search-emitted) | Postgres + S3-class anchor | tenant + time | per-tenant per-day | 3-AZ + cross-region | indefinite |

### 5.4 Event schemas (events emitted)

All events go through the canonical eventing backbone per ADR-0050/0174 + outbox pattern.

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `search.document_indexed.v1` | `oya.search.document` | `contracts/events/search.document_indexed.v1.avsc` | Foundry (RAG cache invalidate), Audit, FinOps (per-tenant ingest cost) | 30 d | `(tenant_id, document_id, version)` |
| `search.document_purged.v1` | `oya.search.document` | `contracts/events/search.document_purged.v1.avsc` | Audit (DSR proof-of-erasure), Foundry (RAG cache invalidate), Tenant trust portal | indefinite | `(tenant_id, document_id, dsr_request_id)` |
| `search.index_built.v1` | `oya.search.index` | `contracts/events/search.index_built.v1.avsc` | Cloud (capacity), FinOps, Tenant Index Lifecycle Console | 90 d | `(tenant_id, index_id, build_seq)` |
| `search.query_issued.v1` | `oya.search.query` | `contracts/events/search.query_issued.v1.avsc` | Audit (per-query record), Analytics, Foundry (capability echo), Ads (sponsored-slot signal) | 90 d (private); 1y aggregate | `query_id` |
| `search.click_recorded.v1` | `oya.search.query` | `contracts/events/search.click_recorded.v1.avsc` | Ranker (training), Ads (cross-link to attribution), Analytics | 90 d full; 1y aggregate | `(query_id, hit_seq)` |
| `search.ranker_promoted.v1` | `oya.search.rank` | `contracts/events/search.ranker_promoted.v1.avsc` | Audit, FinOps, Ads (quality-score recompute) | indefinite | `(namespace, ranker_id, version)` |
| `search.crawl_completed.v1` | `oya.search.crawl` | `contracts/events/search.crawl_completed.v1.avsc` | Index (ingest), Audit (per-fetch host record), FinOps | 30 d | `fetch_task_id` |
| `search.kg_entity_changed.v1` | `oya.search.kg` | `contracts/events/search.kg_entity_changed.v1.avsc` | Foundry (grounded-reasoning cache invalidate), Vertical (KG overlay reconcile), Ads (entity-targeting recompute) | 30 d | `(kg_entity_id, version)` |
| `search.consent_class_changed.v1` | `oya.search.governance` | `contracts/events/search.consent_class_changed.v1.avsc` | Document re-evaluate, Index re-evaluate, Audit | 90 d | `(tenant_id, change_seq)` |
| `search.embedding_model_promoted.v1` | `oya.search.embed` | `contracts/events/search.embedding_model_promoted.v1.avsc` | Re-embed worker, Foundry (RAG embedding cache), FinOps | indefinite | `embedding_model_ref` |

### 5.5 Index / search-index touchpoints

| Entity field | Index | Class allowed (per consent tier) | Cascade-on-DSR? |
|---|---|---|---|
| Object Graph `Entity.properties[k]` (when `indexable=true`) | `oya-search-tenant-private` (per-tenant) | per-property tier (ADR-0008); typically `BEHAVIORAL_TENANT_PRODUCT` (7), `DECLARED_PREFERENCE` (9), `PUBLIC` (1) | Yes |
| Object Graph vector property | `oya-search-vector-tenant-private` | as above | Yes |
| Connect message body (per ADR-0008) | `oya-search-connect-private` | tenant-class-aware; healthcare/fintech blocklist applies | Yes |
| Marketplace listing | `oya-search-marketplace-public` | `PUBLIC` | Yes |
| Crawled web URL + body | `oya-search-public-web` | `PUBLIC` | n/a (public web) |
| Vertical legal corpus (ADR-0033, ADR-0033) | `oya-search-vertical-legal-{kr,jp,us,eu,...}` | `PUBLIC` (legal text); per-pack regulator binding | n/a |
| Cross-tenant aggregate (k-anonymity ≥ 10) | `oya-search-cross-tenant-aggregate` | `CROSS_TENANT_AGGREGATE` consent tier only | Yes |

### 5.6 Audit-chain emission contract

Per [DESIGN.md §7](../../DESIGN.md) + ADR-0003, every regulated capability must emit.

| Operation | Emits topic | Required fields |
|---|---|---|
| Document indexed | `oya.audit.search_doc_index` | `tenant_id`, `document_id`, `namespace`, `data_class`, `consent_receipt_ref`, `actor`, `timestamp`, `prev_hash` |
| Document purged (DSR) | `oya.audit.search_doc_purge` | `tenant_id`, `document_id`, `dsr_request_id`, `proof_of_erasure_root`, `timestamp`, `prev_hash` |
| Query issued (Foundry capability) | `oya.audit.search_query` | `tenant_id`, `query_id`, `principal`, `namespace`, `data_classes_eligible`, `consent_tier_at_call`, `capability_id`, `timestamp`, `prev_hash` |
| Ranker promoted | `oya.audit.search_ranker_promote` | `namespace`, `ranker_id`, `before_hash`, `after_hash`, `eval_metric`, `actor`, `timestamp`, `prev_hash` |
| Index lifecycle (build / rebuild / retire) | `oya.audit.search_index_lifecycle` | `tenant_id`, `index_id`, `op`, `actor`, `timestamp`, `prev_hash` |
| Cross-axis flow (Search → Ads quality signal) | `oya.audit.search_to_ads_signal` | `tenant_id`, `query_id`, `signal_kind`, `data_classes_used`, `consent_receipt_ref`, `timestamp`, `prev_hash` |
| Crawl-task fetch | `oya.audit.search_crawl_fetch` | `host`, `url_hash`, `respect_robots`, `data_class_observed`, `timestamp`, `prev_hash` |
| KG entity ingest | `oya.audit.search_kg_ingest` | `entity_id`, `entity_type`, `data_class`, `vertical_overlay`, `source_refs`, `timestamp`, `prev_hash` |
| Embedding model promotion | `oya.audit.search_embed_promote` | `embedding_model_ref`, `before_hash`, `after_hash`, `eval_metric`, `actor`, `timestamp`, `prev_hash` |

### 5.7 Schema migration policy

- **Versioning**: `schema_version: u32` per kernel entity; index `Mapping` versions are monotonic — mapping change forces re-build.
- **Reversibility**: index re-build ships dual-index (old serves while new builds); cutover by atomic alias swap; per-region rollout via Argo Rollouts (ADR-0050).
- **Dry-run gate**: Foundry fitness function `oya-foundry-fitness-search-mapping` runs every mapping change against a synthetic 1M-document corpus before merge.
- **Embedding model migration**: re-embed cascade is async; old vectors served until new model passes eval-metric gate.

## 6. Optimization practices (required) — *slice-level*

| Practice | Implementation choice |
|---|---|
| Cell routing | `Tenant.region` → tenant-private index runs in tenant cell; public corpus in per-region public cell; query router reads `x-oya-tenant` header |
| Sharding strategy | Per-term hash-shard for inverted index (greenfield C-section); per-tenant shard for tenant-private indexes; per-shape vector shard (pgvector → Milvus when shard exceeds 100M); per-host shard for crawler |
| Caching tier | In-memory (moka) for hot ranker + tokenizer state; Redis for embedding cache (per `model_ref + content_hash`); CDN for SERP static assets and featured-snippet HTML; per-query result-cache TTL 5 min for public SERP |
| Bulk endpoint contract | `BatchIndexDocuments` (per-tenant, max 10 000 docs/batch), `BulkPurgeDocuments` (DSR cascade), `BatchEmbed` (model-side rate-limited) |
| Pagination | Cursor-based on `(score, doc_id)` opaque token; default page 10 (SERP), 100 (API); max page 1 000 |
| Idempotency | `Idempotency-Key` on every ingest mutation; outbox dedupes 24 h; per-document `(tenant_id, source_ref, content_hash)` natural key prevents duplicate ingest |
| Batch dispatch | Indexer batches every 1 s or 256 docs; re-embed worker batches every 5 s or 64 docs (model-call efficiency); crawler dispatches per-host with politeness window |
| Backpressure | Indexer reads from Kafka with consumer-group rebalance; embeds dropped to dead-letter at 95% lag; query API returns `429`+`Retry-After` on tenant-rate-limit; per-host crawl shed on host-quota-exceeded |
| Hot-path benchmarks | RAG retrieve (`p99 ≤ 250 ms`), tenant-private full-text query (`p99 ≤ 200 ms`), public SERP (`p99 ≤ 300 ms`), KG entity lookup (`p99 ≤ 100 ms`) — wired to `oya-foundry-fitness-bench` per ADR-0044 |
| Agent-driven optimization loops | Foundry capability `search.ranker.tune` (autonomy ≤ T2): proposes signal-weight adjustments from click-through deltas; `search.crawler.schedule-tune` (≤ T1): adjusts per-host quota based on freshness target; `search.kg.entity-link-improve` (≤ T2): proposes new entity link from query log; human approves at T2 |
| FinOps unit-economics | Per-tenant cost = (index ingest events × per-doc-rate) + (query events × per-query-rate) + (embedding compute × per-token-rate); per-call cost in metering kernel; target gross margin ≥ 40% at GA |
| Build-cache and CI affected-graph | `oya-search-*` is a single-axis subgraph; per-namespace mapping changes are isolated; ranker model-artifact builds cached in Object store keyed by `(model_ref, eval_metric)` |

## 7. Regional pack interactions (required) — *which seams this product plugs into*

Per [DESIGN.md §12](../../DESIGN.md):

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Tokenizer | `Tokenizer` in `oya-search-tokenizer-kernel` | yes | KR (mecab-ko, khaiii), JP (MeCab + Sudachi), EN (ICU), DE/FR/ES/PT (ICU + per-pack stemming), HI (ICU + Hindi-specific), AR (ICU + RTL + Arabic stemming), TH (ICU + Thai segmentation), VI (ICU + Vietnamese tone-mark) |
| KG vertical overlay | `KgExtractionProfile` in `oya-search-kg-kernel` | yes | KR-EDI 보건의료, US-LOINC + ICD-10 + SNOMED, JP-ReceiptCode + JP-LabCode, EU-IDMP, IN-Ayushman, BR-CID-10, KSA-MOH coding |
| Per-pack crawler politeness | `CrawlerPolicyOverlay` in `oya-search-crawler-domain` | yes | KR (`robots.txt` + 한국어 sitemap conventions), JP (METI sitemap), US (IndexNow), EU (sovereignty cookies), CN (mainland blocked by policy) |
| SERP UX (typography, IME, RTL) | `LocaleBundle` + `SerpTheme` in `oya-search-serp-frontend` | yes | KR-first per ADR-0037 / ADR-0033 (Pretendard / KR Hangul vertical alignment), JP (BIZ UDPGothic, vertical-text option), EN (Inter), AR (RTL + Noto Naskh), HI (Devanagari) |
| Featured-snippet legal disclaimer | `FeaturedSnippetDisclaimer` per pack | yes | KR (의료법 / 변호사법 disclaimers for medical / legal queries), JP (薬機法), US (FTC), EU (GDPR cookie), KSA (PDPL) |
| Per-pack ranker tuning | `RankerOverlay` per pack | yes | KR (Naver-class freshness expectations, Korean stop-word list), JP (Yahoo!JP-class), US (Google-class), EU (privacy-preferred ranker), IN (Hindi+English code-switch), BR (Portuguese morphology) |
| Per-vertical legal corpus | `LegalCorpus` per pack per vertical (ADR-0033, ADR-0033) | yes | per pack × per vertical (KR healthcare, KR fintech, US healthcare, EU GDPR, etc.) |
| Per-pack regulator portal (search-axis surface for trust portal) | `RegulatorPortal` | yes | KR, JP, US, EU, IN, BR, KSA, UAE, ANZ, SG |

## 8. In-house vs external dependency posture (required)

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `axum` / `tokio` / `serde` / `tonic` / `rustls` | kernel-grade | MIT/Apache-2 | no | adopt |
| `pgroonga` (Postgres extension) | secondary | LGPL-2.1 *(extension; not linked)* | own KR full-text — rejected | **adopt as Postgres extension only** (boundary respected; ADR-0047) |
| `pgvector` (Postgres extension) | secondary | PostgreSQL License | own vector — rejected | adopt (ADR-0047) |
| `mecab-ko` | secondary | BSD | own KR morph — rejected | adopt (regional-pack-kr tokenizer) |
| `khaiii` (Kakao) | secondary | Apache-2 | own KR morph — rejected | adopt (alternative KR tokenizer) |
| `MeCab` (JP) | secondary | BSD/LGPL/GPL triple-licensed | own JP morph — rejected | adopt under BSD election (boundary respected) |
| `ICU` | kernel-grade | ICU License (BSD-style) | no | adopt |
| `Apache AGE` (Postgres graph extension) | secondary | Apache-2 | own KG store — rejected | adopt for KG backend day-1 |
| `Neo4j` (gated end-state for KG) | secondary | GPL-3 *(community)* / commercial *(enterprise)* | Apache AGE primary | **GATED**; if adopted, only via commercial license to avoid GPL link |
| `Milvus` (gated, ADR-0047) | secondary | Apache-2 | pgvector primary | adopt gated when shard exceeds 100M vectors |
| `OpenSearch` (gated end-state, ADR-0047) | secondary | Apache-2 | pgroonga primary | adopt gated when corpus exceeds 1B docs |
| `Vespa` (gated end-state, ADR-0047) | secondary | Apache-2 | OpenSearch alternative | adopt gated for hybrid+ML ranking at billion-scale |
| `Wasmtime` (per-vertical extractor sandbox) | secondary | Apache-2 | reuse from SaaS axis (ADR-0023) | adopt |
| `OpenTelemetry` | kernel-grade | Apache-2 | no | adopt |
| `Apache Kafka` | secondary | Apache-2 | own event bus — rejected; outbox is day-1 | adopt gated (ADR-0046) |
| `Tokenizers` (HuggingFace, for embedding model tokenization) | secondary | Apache-2 | own tokenizer — rejected for ML models | adopt for embedding-model side |
| `Cosign` / `Trivy` / `Rekor` | secondary | Apache-2 | own — rejected | adopt (ADR-0039) |
| `OpenBao` (secrets) | secondary | MPL-2 | reuse from cloud axis | adopt (ADR-0043) |

License gate: Apache-2 / MIT / BSD / MPL-2 — allowed; AGPL / GPL — forbidden in product code; SSPL / BUSL — ADR review. LGPL/GPL extensions (pgroonga, MeCab default, Neo4j community) are allowed only at process / extension boundary, never linked into Rust crate code; the boundary is enforced by `oya-foundry-fitness-license` (ADR-0039).

## 9. Success metrics (required)

| Metric | W-Search-Preview target | W-Search-Stable target | W-Public-GA target | W-Region-Fan-Out target |
|---|---|---|---|---|
| Tenant-private search query p99 | ≤ 300 ms | ≤ 250 ms | ≤ 200 ms | per-region |
| Public SERP query p95 | n/a | ≤ 600 ms | ≤ 500 ms | per-region |
| Foundry RAG p99 | ≤ 300 ms | ≤ 250 ms | ≤ 250 ms | per-region |
| Index ingest throughput | ≥ 1 000 docs/s/tenant | ≥ 10 000 | ≥ 100 000 | per-region |
| Public web index size | n/a | ≥ 100M docs (KR-pack first) | ≥ 1B docs | per-pack |
| Index coverage of tenant content (per consent) | ≥ 95% on consenting tier | ≥ 99% | 100% | 100% |
| Audit-chain emission completeness | ≥ 99% | 100% | 100% | 100% |
| DSR cascade ack ≤ 30 days | ≥ 95% | 100% | 100% | 100% |
| Embedding cache hit rate | ≥ 70% | ≥ 85% | ≥ 90% | per-region |
| Ranker promotion via Argo Rollouts | manual | semi-auto | auto with metric gate (no quality-regression) | per-pack |
| KR ranking quality (vs Naver baseline on KR queries) | n/a (private only) | within 10% nDCG | within 5% nDCG | KR-pack baseline beat |
| Cross-axis contract violations on `main` | 0 | 0 | 0 | 0 |
| Tenant-private search adoption | ≥ 25 internal pilots | ≥ 250 paying tenants | ≥ 2 500 paying tenants | per-region |

## 10. Risks + mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Data Use Boundary ADR not landed before search work begins | Catastrophic | **HARD GATE**: no `oya-search-*` crate may merge before DUB ADR is Accepted; CI fitness `oya-foundry-fitness-search-dub` checks status | Search + Privacy + Architecture |
| Tenant data leak into public/cross-tenant index via PHI/PII | Catastrophic | Per-class allowed-data-classes on Index; ingest gate refuses on class violation; runtime guard at query-eval; audit-chain emission per ingest; Tenant-class overrides (healthcare/fintech blocklist) per [PRIVACY-PROGRAM §2.2.3](../../PRIVACY-PROGRAM.md) | Search + Privacy |
| Foundry RAG capability over-shares (capability scope leaks namespaces) | Catastrophic | Per-capability `(namespace × consent-tier × max-k)` enforcement (greenfield I02); audit-chain on every call; capability schema at `product-control/capabilities/search.*.yaml` | Search + Foundry + Governance |
| Crawler abuse / robots.txt non-compliance | High | Strict `RobotsCompliance::Strict` default; per-host quota; politeness window enforced at scheduler; abuse-complaint feedback loop reduces quota | Search-crawler team |
| Ranker drift causes quality regression | High | Argo Rollouts canary + automated metric-gated rollback (ADR-0050); shadow-mode evaluation before promotion; Foundry tuning only ≤ T2 with human approval | Search + SRE |
| KG ingest pollutes with low-confidence entity links | High | Per-link confidence threshold; vertical-overlay-aware extractors; Foundry `search.kg.entity-link-improve` capability ≤ T2 with human approval | Search-KG team |
| Per-region tokenizer license drift (e.g. MeCab GPL election) | Medium | License-policy gate (`oya-foundry-fitness-license`); MeCab adopted under BSD election only; CI verifies | Search + Foundry |
| Vector store scale beyond pgvector | Medium | Hexagonal port keeps Milvus (ADR-0047) as drop-in; per-namespace cutover when shard exceeds 100M | Search + Cloud |
| Public SERP latency under load | Medium | Per-region edge cache (CDN); query result-cache TTL 5 min; per-shard replica scale-out; degraded mode (lexical-only fallback) under p99 violation | Search + SRE |
| Sponsored-slot integration leaks ranker signal to advertisers | High | Cross-axis contract `search → ads quality signal` is one-way; advertisers never receive raw query log; signal is post-aggregated | Search + Ads + Privacy |
| Trending-queries surface (greenfield E11) introduces safety incident | High | Safety-first design from day 1: rate-limit + anomaly detection + manual-curation gate; KR Naver "실시간 급상승" cautionary precedent | Search + Trust & Safety |

## 11. Open questions

1. **Search axis monetization at W-Search-Stable**: purely sponsored-result auction (ads axis), or also subscription tiers for ad-free SERP? Default proposed: ad-supported public + ad-free for paying SaaS tenants on tenant-private search.
2. **Cross-tenant aggregate search**: per-record opt-in by tenant, or per-collection (per [PRIVACY-PROGRAM §2.5 Q5](../../PRIVACY-PROGRAM.md))? Default proposed: per-record (more work, cleaner) — confirm.
3. **End-state ranking backend**: Vespa or OpenSearch hybrid+ML — when to commit (after which scale milestone)? Decision deferred to W-Search-Stable + 6 months of public corpus operation.
4. **KG end-state**: Apache AGE (Postgres) vs Neo4j commercial — when to switch and at what scale? Default proposed: AGE day-1, Neo4j commercial gated at 100M+ entities.
5. **Trending-queries surface inclusion**: ship with safety design at W-Search-Stable, or defer to W-Public-GA with more time to harden? Council decision pending.

## 12. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-05-09 | Data Use Boundary ADR is hard P0 gate | PRD §6 + DESIGN §11 + greenfield H01 |
| 2026-05-09 | pgroonga + pgvector day-1; OpenSearch / Vespa / Milvus gated | ADR-0047, ADR-0047; cost + license + scale |
| 2026-05-09 | Per-capability authorization on every `search.*` Foundry call | Greenfield I02 |
| 2026-05-09 | Apache AGE day-1 for KG; Neo4j gated commercial-only | License posture (Neo4j community is GPL-3) |
| 2026-05-09 | Sponsored-slot integration is one-way (signal flows search→ads; raw query log never crosses) | Privacy-program-stricter-than-Google |

## 13. Sources scanned

- [`docs/PRD.md`](../../PRD.md)
- [`docs/DESIGN.md`](../../DESIGN.md) §1, §3, §4, §10, §12
- [`docs/PRIVACY-PROGRAM.md`](../../PRIVACY-PROGRAM.md) §2.2.1, §2.2.2, §2.2.3, §2.5, §3
- [`docs/GLOSSARY.md`](../../GLOSSARY.md) §5 (data + search + ML)
- `/Users/jasonlee/oyatie/docs/raw/greenfield-search.md` (295 leaves: A Crawling, B Parsing+Enrichment, C Indexing, D Ranking, E Query Understanding, F SERP+Features, G Infrastructure, H Safety+Compliance, I Search↔AI-Agent-Runtime, J Search↔Ads, K Clean-arch, L KR-launch, M Counts, N Highest-regret deferrals, O Integration contracts)
- ADR-0006..0112 (Object Graph property tiers), ADR-0021 (OG Agent Gateway), ADR-0006 (Vector property — pgvector), ADR-0050 (Vector store tiering), ADR-0047 (Vector store at billion-scale — Milvus gated), ADR-0047 (Search backend — OpenSearch gated, pgroonga day-1, Vespa end-state), ADR-0013 (Envoy gateway), ADR-0045 (ClickHouse OLAP), ADR-0046 (Kafka eventing), ADR-0045 (Iceberg cold tier gated), ADR-0050 (Argo Rollouts), ADR-0039 (Supply chain), ADR-0033 (Platform legal corpus), ADR-0037 (Mobile parity), ADR-0033 (Leptos client), ADR-0008 (Connect retention), ADR-0033 (Regulated-vertical legal corpus), ADR-0015 (Flat crates), ADR-0044 (Deploy platform consolidation), ADR-0003 (Trust framework), ADR-0050 (Data + AI governance), ADR-0017 (Roadmap wave integration)

---

## Doc-catalog row (paste into `DOC-CATALOG.md §2.5`)

```
| `search` | `axis-search` | scope, contract, capability | monthly | PRD.md, DESIGN.md, PRIVACY-PROGRAM.md, GLOSSARY.md |
```

## Catalog mirror (machine-readable)

When this PRD is created or updated, also update:
- `machine-readable/products.json` — add `search` row
- `machine-readable/catalog.json` — pointer at this PRD path
- `machine-readable/contracts.json` — every cross-axis contract row in §4.5
- `machine-readable/risks.json` — risks from §10
- `machine-readable/glossary.json` — Document, Index, Query, Ranker, KgEntity canonical terms

## Validation checks

`oya-foundry-fitness-product-prd` runs:
- All required sections present
- Every flat-crates target referenced exists in `Cargo.toml` or planned roadmap
- Every entity field has a `data_class` annotation
- Every external dep has a license-tier row
- Every cross-axis contract is in DESIGN §10
- **Search-specific**: `oya-foundry-fitness-search-dub` blocks merge if Data Use Boundary ADR not Accepted
