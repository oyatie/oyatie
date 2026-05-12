# ADR-0030: Search axis — crawler / parser / index / ranker / SERP architecture with KR-first morphology and Data-Use-Boundary segregation

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-search`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0008, ADR-0011, ADR-0028, ADR-0031, ADR-0034, ADR-0046, ADR-0047, ADR-0048, ADR-0049

---

## Context

Axis 6 (Search) is the deepest software-only axis in the ecosystem. It is also the axis with the highest per-query economics sensitivity (sub-100ms P95 latency budget), the highest data-class blast radius (a single ranking signal leak can taint the entire SERP), and the strongest jurisdictional binding (KR youth-protection, RTBF, PIPA Art 27). Foundation ADRs named Search as an axis but did not pin its sub-architecture, its KR-specific morphology stack, the Data-Use-Boundary index segregation pattern, or the Search↔Foundry RAG endpoint that lets agents query the same indexes humans query.

KR launch is the binding constraint: the index must understand Korean morphology (조사/어미 separation, mecab-ko-class tokenization), must integrate with Naver/Kakao API surfaces for KR content that lives behind those walled gardens, and must respect KR-specific signals (e.g. blog spam patterns endemic to KR, Naver-style 검색광고 vs organic separation). The per-tier index segregation per Data Use Boundary (§2.2.9) is the cohesion-thesis projection of Search: an index built from public crawl data cannot be queried with private-tenant data conditioning unless that tenant explicitly grants the cross-tier flow.

---

## Decision

We adopt a **five-stage Search architecture** — Crawler → Parser → Indexer → Ranker → SERP — plus three cross-cutting subsystems (Query Understanding, Safety, Search↔Foundry/Ads bridges). Each stage is its own bounded context under `crates/oya-search-<stage>-*`. Per-tier index segregation is enforced at the Indexer layer; cross-tier query is forbidden by default and gated by the Data Use Boundary policy kernel (per ADR-0008).

### Crawler (`crates/oya-search-crawler-*`)

- **Politeness.** Per-host rate limiting; per-host robots.txt cache (per RFC 9309); per-host crawl-delay + per-tenant override.
- **Discovery.** Sitemaps (XML + sitemap-index); link discovery; seed lists per pack; KR-specific seeds (정부24, 공공데이터포털, K-startup).
- **Render farm.** Headless Chromium pool (per-cell isolated) for JS-rendered pages; per-page render budget; per-render trace emitted to audit chain.
- **Dedup.** Content-hash + simhash near-duplicate detection; per-tenant dedup namespace.
- **Spam.** Per-pack spam classifier (rule + ML); KR-specific spam patterns (blog cloaking, 어뷰징 link farms).
- **Tenant-private crawl.** Per-tenant crawler authenticates via tenant-supplied OAuth or service-account credentials; tenant-private content lands in tenant-private indexes only (never in public index, never blended into ranking signals).
- **KR-specific API integration.** Naver Search API + Kakao Search API + Naver/Kakao Cloud OBS + KT/LG U+ CDN; subscription-mode session-token rotation per ADR-0043.

### Parser (`crates/oya-search-parser-*`)

- **HTML.** WHATWG DOM-equivalent parser in Rust; semantic-tag extraction (article/main/header).
- **PDF.** pdfium FFI day-1; in-house Rust PDF parser long-horizon.
- **OCR.** Tesseract FFI + per-pack model registry; KR OCR uses a custom-trained model (Tesseract + per-pack KR fine-tune).
- **Audio/Video transcript.** Foundry capability `workflow.search.transcribe` (per ADR-0011); transcript indexed alongside source media.
- **Korean morphology (KR pack).** mecab-ko + khaiii via FFI day-1; in-house Rust port long-horizon (per ADR-0048); per-token POS tag retained as feature.
- **Multi-locale tokenization.** Per-pack tokenizer impl (JP MeCab-ja, ZH jieba, EN NLTK, IndicNLP, Stanza-Arabic) — see ADR-0048 for the trait surface.
- **NER + KG entity linking.** Per-pack NER model; entity ID resolves to a Knowledge Graph node (`crates/oya-search-kg-*`); ambiguous entities scored.
- **Semantic embeddings.** Per-pack embedding model (Korean: KoSimCSE-class; English: bge-large; multilingual: bge-m3); vector emitted to vector index.

### Indexer (`crates/oya-search-indexer-*`)

- **Inverted by term, sharded by region/locale.** Per-pack shard map; per-tenant private shards isolated.
- **Vector index.** HNSW + IVF (per ADR-0046); per-pack embedding dimension; per-tier segregated.
- **Knowledge Graph.** Property graph (entity / relation / attribute); per-pack ontology overlay.
- **Geo index.** S2 / H3 cells; per-pack cell resolution.
- **Image / video index.** Per-asset embedding + per-frame embedding for video; reverse-image-search via vector index.
- **Per-tenant private + cross-tenant per consent.** A tenant's private content goes into a tenant-private namespace; cross-tenant blending requires an explicit tenant-to-tenant consent record + per-query Cedar gate.
- **Per-tier segregation.** Public crawl tier / tenant-public tier / tenant-private tier / regulated tier — each in its own physical shard set; cross-tier query is a hard-fail without an explicit DUBO grant.

### Ranker (`crates/oya-search-ranker-*`)

- **First stage: BM25 (Tantivy-class).** Per-shard BM25 + per-pack stopword/stemming overlay.
- **Second stage: semantic rerank.** Per-pack rerank model; takes top-k candidates from BM25 + vector retrieval.
- **Freshness.** Per-pack decay function; per-vertical override (news = aggressive; reference = mild).
- **Authority.** Per-host trust signal (PageRank-class + per-pack curated list); KR-specific (정부 도메인 boost, 학술 도메인 boost).
- **Diversity.** Per-result-cluster diversity penalty; per-host diversity cap.
- **Korean signals.** KR-specific (Naver-style 어뷰징 penalty, Tistory/Brunch authority, 카페 vs 블로그 separation).

### Query Understanding (`crates/oya-search-query-*`)

- **Parser.** Query-language parser (operators: site:, filetype:, intitle:, inurl:, before:, after:); per-pack locale-aware.
- **Expansion.** Synonym + morphological expansion; KR-specific (한자 ↔ 한글 ↔ 영문 transliteration).
- **Spelling.** Per-pack spelling correction (Korean Hangul phonetic + 자모 confusion model).
- **Autocomplete.** Per-tenant + per-pack suggestion index; per-query log emitted to audit chain.
- **QA + RAG.** Per-pack QA reader; RAG endpoint calls Foundry capability `workflow.search.rag` (per ADR-0011).

### SERP + features (`crates/oya-search-serp-*`)

- **Standard SERP.** 10 organic results per page; pagination; per-pack snippet rendering.
- **Features.** Knowledge panel; featured snippet; image carousel; video carousel; news; map; people-also-ask.
- **KR features.** 지식백과 panel; 학술 panel; 쇼핑 panel (gated by tenant-ads consent per ADR-0031).
- **Per-tier mark.** Results from regulated tiers (e.g. clinical) carry a tier badge.

### Safety + RTBF/PIPA + youth protection

- **Per-pack safety classifier.** Adult / violence / self-harm / illegal-drug; per-pack threshold.
- **Youth protection (KR).** 청소년 유해정보 차단 per 「청소년보호법」 §16; per-tenant 청소년 모드 toggle; default-on for tenants flagged as K12 (per ADR-0034).
- **RTBF.** Per-jurisdiction takedown queue (KR PIPA Art 36 + 「언론중재법」; EU GDPR Art 17; CA CCPA); per-takedown audit-chained; cosign-signed proof-of-removal per ADR-0038.
- **PIPA Art 27 (location/health/etc).** Sensitive-data tier results require a per-pack opt-in.

### Search ↔ Foundry RAG endpoint

```rust
// crates/oya-search-foundry-bridge
pub trait RagEndpoint {
    fn retrieve(
        &self,
        query: SearchQuery,
        tenant_consent: ConsentReceipt, // from ADR-0008 DUBO
        autonomy_tier: PersonaTier,     // from ADR-0007
    ) -> Result<RetrievalSet>;
}
```

The capability `workflow.search.rag` is the only way Foundry agents query Search. The bridge enforces per-tier segregation and per-tenant consent.

### Search ↔ Ads sponsored slot policy

Per ADR-0031, sponsored slots on the SERP are sourced exclusively via the singleton tenant-ads-gate. Search itself does not run an auction; it reserves slot positions and queries the ads service for fillment. Sponsored slots are visibly marked per KR 「표시광고법」.

### KR-launch specifics

- **Day-1 indexes.** KR public web (≥ 1B URLs target by W12); KR public sector (정부24 + 공공데이터); KR academic (RISS + KCI).
- **Day-1 SLA.** P95 < 200ms cold; P95 < 80ms warm; P99 < 500ms.
- **Day-1 query languages.** Korean, English (other locales follow per pack).

### Anti-scope

Search does not own ad sourcing (ADR-0031), does not own the agent runtime (ADR-0007), does not own the autonomy ceiling (ADR-0007), does not own the audit chain (ADR-0003).

---

## Consequences

### Positive

- Per-tier segregation makes Data Use Boundary enforcement mechanical: a query at the public tier physically cannot return tenant-private results.
- KR-first morphology is the moat against incumbents who treat Korean as a second-class locale.
- Search↔Foundry RAG endpoint gives every Foundry agent a single, audited, consent-gated retrieval surface — no agent ships its own retriever.
- Search↔Ads boundary keeps ad sourcing out of Search ranking signals, which is the structural defense against the failure modes that broke incumbent ad-funded search.

### Negative

- Five-stage architecture has a lot of surface; per-stage SLO ownership must be sharp.
- KR morphology FFI dependencies (mecab-ko / khaiii) are LGPL/Apache-2; the legal-isolation analysis (per ADR-0048) is a real cost.
- The render farm at scale is expensive; per-cell capacity planning must front-load.
- KR Naver/Kakao API integration is subject to those vendors' rate limits and terms; we depend on them for some KR content.

### Operational

- Per-stage SLO dashboards; per-shard saturation alerts; per-pack index freshness SLO (KR public web = 24h; KR news = 1h).
- Crawler politeness audited weekly; per-host abuse complaints triaged within 24h.
- RTBF queue staffed during KR business hours; per-takedown audit-chained.
- Per-cell index rebuild drill quarterly.
- Foundry RAG endpoint audit reviewed weekly; per-tenant consent grants reviewed monthly.

---

## Alternatives considered

### Alternative A — White-label Elasticsearch / OpenSearch

- **Pros:** off-the-shelf; large community.
- **Cons:** Elasticsearch SSPL is forbidden per License Policy ADR; OpenSearch (Apache-2) is fine but does not give us per-tier segregation primitives or KR morphology integration. We would re-write the moat layer above it anyway.
- **Rejected because:** OpenSearch is acceptable as an adapter (per ADR-0047) but not as the architecture.

### Alternative B — Single-stage retrieval (no separate parser/indexer/ranker boundary)

- **Pros:** simpler pipeline.
- **Cons:** parser bugs poison the index; ranker changes require re-indexing; per-tier segregation cannot be enforced at one boundary.
- **Rejected because:** the five-stage boundary is what makes per-tier segregation enforceable.

### Alternative C — Skip KR-specific morphology, use multilingual model only

- **Pros:** simpler stack.
- **Cons:** Korean tokenization quality with multilingual models is materially worse; KR users notice immediately; the moat collapses.
- **Rejected because:** KR is the launch market.

### Alternative D — Ship Search↔Ads as a single service

- **Pros:** simpler ad slot fulfillment.
- **Cons:** the entire failure mode of ad-funded search comes from this coupling.
- **Rejected because:** the structural separation is the defense.

---

## Open questions

1. **Q1.** Does the day-1 vector index use 768-dim or 1024-dim embeddings? Default: 1024-dim (bge-m3 multilingual + KoSimCSE-large for KR). → ADR-0046.
2. **Q2.** Does the render farm at Phase 1 burst to GPU (for video frame embedding)? Default: NO at Phase 1; CPU-only; GPU at Phase 2 when own colo. → ADR-0028.
3. **Q3.** Crawler honors `noai`/`noimageai` per IETF ai.txt draft? Default: YES; per-pack opt-out enforced. → owner: `axis-search`.
4. **Q4.** Per-tier blending (e.g. tenant-private + public results in one SERP) UI pattern? Default: tabbed (private tab + public tab); blended only with explicit per-query toggle. → owner: `axis-search`.
5. **Q5.** Search shipping a per-tenant private SaaS-grade enterprise-search SKU at GA? Default: YES; same indexer, tenant-private namespace, no public exposure. → owner: `axis-search`.

---

## References

- `docs/PRD.md` §7 (search axis), §11 (data use boundary)
- `docs/DESIGN.md` §4 (search architecture), §11 (cross-axis contradictions)
- KR 「개인정보보호법」 Art 27 (sensitive data), Art 36 (correction/deletion); 「청소년보호법」 §16; 「언론중재법」 §17 (정정보도); 「표시광고법」 §3
- IETF ai.txt draft; RFC 9309 (robots.txt)
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0008 (data use boundary), ADR-0011 (capability registry), ADR-0028 (cloud), ADR-0031 (ads), ADR-0034 (per-vertical data class overrides), ADR-0046 (vector store), ADR-0047 (search backend), ADR-0048 (Korean morphology), ADR-0049 (residency)
