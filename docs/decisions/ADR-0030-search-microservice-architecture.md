---
id: ADR-0030
status: Accepted
doc_status: published
---

# ADR-0030: Search microservice — crawler / parser / index / ranker / SERP architecture with KR-first morphology and Data-Use-Boundary segregation

> **Status:** Accepted
> **Owner:** `oya-search`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — Search is a flat µservice, not an "axis")
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0008, ADR-0011, ADR-0028, ADR-0031, ADR-0046, ADR-0047, ADR-0048, ADR-0049, ADR-0058

---

## Context

Search is a microservice in the flat catalog. Like every other microservice, it is independent, modular, and integrates with other microservices via Workflow (ADR-0035) and Ontology (ADR-0006/ADR-0055) — never via direct cross-service imports. Search is the deepest software-only microservice: sub-100ms P95 latency budget, highest data-class blast radius, strongest jurisdictional binding (KR youth-protection, RTBF, PIPA Art 27).

KR launch is the binding constraint: the index must understand Korean morphology, must integrate with Naver/Kakao API surfaces, and must respect KR-specific signals.

---

## Decision

We adopt a **five-stage Search architecture** — Crawler → Parser → Indexer → Ranker → SERP — plus three cross-cutting subsystems (Query Understanding, Safety, Search↔Foundry/Ads bridges). Each stage is its own bounded context under `oya-search-<stage>-*`. Per-tier index segregation is enforced at the Indexer layer; cross-tier query is forbidden by default and gated by the Data Use Boundary policy kernel (ADR-0008).

**Naming justification (BNF v4.1, ADR-0056):**
- `oya-search-crawler-kernel`: slot2 = `search` (registered µservice); slot3 = `crawler` (BC); slot4 = `kernel`
- `oya-search-foundry-bridge-adapter`: slot2 = `search`; slot3 = `foundry-bridge` (multi-token BC); slot4 = `adapter`

### Crawler (`oya-search-crawler-*`)

- **Politeness.** Per-host rate limiting; robots.txt cache (RFC 9309); per-host crawl-delay.
- **Discovery.** Sitemaps; link discovery; KR-specific seeds (정부24, 공공데이터포털, K-startup).
- **Render farm.** Headless Chromium pool (per-cell isolated) for JS-rendered pages.
- **Dedup.** Content-hash + simhash near-duplicate detection.
- **KR-specific API integration.** Naver Search API + Kakao Search API.
- **Tenant-private crawl.** Per-tenant crawler authenticates via tenant-supplied credentials; tenant-private content lands in tenant-private indexes only.

### Parser (`oya-search-parser-*`)

- **HTML.** WHATWG DOM-equivalent parser in Rust.
- **Korean morphology (KR pack).** mecab-ko + khaiii via FFI day-1; in-house Rust port long-horizon (per ADR-0048).
- **Multi-locale tokenization.** Per-pack tokenizer trait surface (ADR-0048).
- **NER + KG entity linking.** Per-pack NER model; entity ID resolves to a Knowledge Graph node.
- **Semantic embeddings.** Per-pack embedding model; vector emitted to vector index.

### Indexer (`oya-search-indexer-*`)

- **Inverted by term, sharded by region/locale.** Per-microservice shard map; per-tenant private shards isolated.
- **HTAP boundary.** Search may project operational freshness counters into an HTAP store only when transactional writes and analytical scans remain separately governed by the Data Use Boundary.
- **Vector index.** HNSW + IVF (ADR-0046); per-tenant private + public tiers segregated.
- **Per-tier segregation.** Public crawl / tenant-public / tenant-private / regulated — each its own physical shard set; cross-tier query is a hard-fail without an explicit DUBO grant.

### Ranker (`oya-search-ranker-*`)

- **First stage: BM25 (Tantivy-class), with TF-IDF fallback for sparse or low-resource corpora.**
- **Second stage: semantic rerank.** Per-pack rerank model on top-k candidates.
- **Korean signals.** KR-specific (어뷰징 penalty, Tistory/Brunch authority, 카페 vs 블로그 separation).

### Query Understanding (`oya-search-query-*`)

- **Parser.** Query-language operators: site:, filetype:, intitle:, inurl:, before:, after:.
- **QA + RAG.** Per-pack QA reader; RAG endpoint calls Foundry capability `workflow.search.rag`.

### SERP (`oya-search-serp-*`)

- Standard SERP + KR features (지식백과 panel, 학술 panel).
- Per-tier mark: results from regulated tiers carry a tier badge.

### Safety + RTBF/PIPA

- Per-pack safety classifier (adult / violence / self-harm / illegal-drug).
- Youth protection (KR): 청소년 유해정보 차단 per 「청소년보호법」 §16.
- RTBF: per-jurisdiction takedown queue (KR PIPA Art 36; EU GDPR Art 17).

### Search ↔ Foundry RAG endpoint

```rust
// oya-search-foundry-bridge-adapter
pub trait RagEndpoint {
    fn retrieve(
        &self,
        query: SearchQuery,
        tenant_consent: ConsentReceipt,
        autonomy_tier: PersonaTier,
    ) -> Result<RetrievalSet>;
}
```

The capability `workflow.search.rag` is the only way Foundry agents query Search. The bridge enforces per-tier segregation and per-tenant consent. This is a Workflow-mediated integration — Search does not call Foundry directly.

### Search ↔ Ads sponsored slot policy

Per ADR-0031, sponsored slots on the SERP are sourced exclusively via the singleton tenant-ads-gate. Search reserves slot positions and queries the ads service for fillment. Search ranking signals never see ad bidding signals.

---

## Consequences

### Concrete crate layout (BNF v4.1)

```
oya-search-crawler-kernel
oya-search-crawler-worker        — crawl scheduling + politeness
oya-search-parser-kernel         — tokenizer trait + NER trait
oya-search-parser-adapter        — mecab-ko FFI, khaiii FFI, embedding models
oya-search-indexer-kernel        — shard-map types + segregation policy
oya-search-indexer-adapter       — Tantivy + pgvector + HNSW impls
oya-search-indexer-worker        — indexing pipeline worker
oya-search-ranker-kernel         — ranking signal trait surface
oya-search-ranker-domain         — BM25 + rerank + freshness + authority logic
oya-search-query-kernel          — query parse types
oya-search-query-domain          — expansion + spelling + autocomplete
oya-search-serp-kernel           — SERP result types + feature types
oya-search-serp-rest             — SERP HTTP API
oya-search-safety-kernel         — safety classifier trait
oya-search-safety-adapter        — per-pack safety model impls
oya-search-foundry-bridge-adapter — RAG endpoint (Workflow-mediated)
oya-search-kg-kernel             — Knowledge Graph node types
oya-search-kg-adapter            — property graph impl
oya-search-rest                  — Search API surface
oya-search-grpc                  — Search gRPC surface
oya-search-app                   — composition-root binary
```

All crates registered under `search` in `[workspace.metadata.oya.microservices]`.

### Positive

- Per-tier segregation makes Data Use Boundary enforcement mechanical.
- KR-first morphology is the moat against incumbents who treat Korean as a second-class locale.
- Search↔Foundry RAG endpoint gives every Foundry agent a single, audited, consent-gated retrieval surface.
- Search↔Ads boundary keeps ad sourcing out of Search ranking signals.

### Negative

- Five-stage architecture has large surface; per-stage SLO ownership must be sharp.
- KR morphology FFI dependencies (mecab-ko / khaiii) are LGPL/Apache-2; legal-isolation analysis per ADR-0048 is a real cost.
- Render farm at scale is expensive.

---

## Related

- ADR-0001 (cohesion — Search is a µservice in the flat catalog)
- ADR-0008 (data use boundary — per-tier segregation)
- ADR-0031 (ads — sponsored slot sourcing via ads-gate)
- ADR-0046 (vector store)
- ADR-0047 (search backend strategy)
- ADR-0048 (Korean morphology)
- ADR-0058 (Flat microservice catalog)
- `[[feedback-flat-product-catalog]]` — Search is a shared µservice, not an axis
