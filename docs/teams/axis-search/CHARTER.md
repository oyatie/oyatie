---
doc_status: published
---

# Team: Axis — Search Engine

## Mission
This team owns Oyatie's search engine axis: crawler, parser, inverted index, vector index, ranking, query understanding, SERP, and the RAG endpoint that Foundry consumes. It exists because tenant content must be findable — per consent tier — without rebuilding retrieval for every vertical, and because the same search infrastructure powers both organic results and sponsored slots. It does **not** begin index ingestion until the Data Use Boundary ADR is Accepted, and it does **not** ingest tenant PHI/PII/PCI into any shared index regardless of consent.

## Owned axes / surfaces / contracts
- **Axis(es):** Search engine (Axis 6)
- **Surfaces:**
  - `search-document-kernel` — `Document`, `DocumentClass`, `IndexEntry`, `TenantPrivateIndex`
  - `search-index-kernel` — `IndexLifecycle`, `ShardSpec`, `SnapshotRef`
  - `search-crawler-*` — web + tenant-content crawler (politeness, robots, dedup, spam)
  - `search-parser-*` — HTML/PDF/OCR, Korean morphology (mecab-ko/khaiii), NER, embeddings
  - `search-index-*` — inverted index (pgroonga day-1), vector index (pgvector), knowledge graph
  - `search-rank-*` — BM25 + semantic reranker, freshness, authority, diversity, KR signals
  - `search-query-*` — query understanding, expansion, spelling, autocomplete, QA, RAG retrieval
  - `search-serp-*` — SERP assembly, sponsored slot stitching (slot inventory from `axis-ads-analytics`)
  - Trust portal evidence surface (read side hosted on search axis infrastructure)
  - Products owned: `products/search/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Search index lifecycle` (owner) — Foundry (RAG), SaaS (tenant search), Ads (sponsored slot)
  - `Ad slot inventory` (consumer from `axis-ads-analytics`) — sponsored slots stitched into SERP
  - `RAG endpoint` (co-owner with `axis-foundry`) — search provides the retrieval; Foundry exposes the endpoint
- **Catalog records:** `crates/search-*`
- **Runbooks:** `runbooks/search-index-dsr-cascade.md`, `runbooks/crawler-politeness-incident.md`, `runbooks/serp-sponsored-slot-failure.md`
- **ADRs:** ADR-0047 (search architecture), KR morphology tokenizer seam

## In-scope work
- Crawler: politeness (robots.txt, crawl-delay), KR-specific crawl compliance (저작권), tenant-private crawl, dedup, spam detection, render farm
- Parser: HTML/PDF/OCR/transcript, Korean morphology (mecab-ko, khaiii), NER, embedding generation
- Index: pgroonga (day-1 inverted), pgvector (vector), per-tenant private index, cross-tenant shared index (consent-gated), knowledge graph, geo index
- Ranking: BM25 + semantic reranker, freshness, authority signals, diversity, Korean-specific ranking signals, click-stream feedback (privacy-gated via Data Use Boundary)
- Query understanding: parser, query expansion, spelling correction, autocomplete, QA, RAG retrieval path
- SERP: organic result assembly, sponsored slot stitching, result diversity, RTBF/PIPA/GDPR safety filter
- RAG endpoint: expose indexed content to Foundry via `intelligence-rag`; consent gate on every retrieval
- DSR cascade: delete index entries on DSR trigger; emit proof-of-erasure to `platform-audit-evidence`
- Trust portal evidence read surface (the search infra hosts the read side; content comes from audit chain)
- Per-tenant index isolation: tenant-private indexes never surface in cross-tenant results
- Data Use Boundary enforcement: index ingestion reads `DataUseConsent.search_indexable_classes`

## Out-of-scope (anti-scope)
- Ad auction logic (→ `axis-ads-analytics` — search provides the slot; ads fills it)
- Agent runtime (→ `axis-foundry` — RAG endpoint is Foundry's surface; search provides the backing index)
- SaaS workflow engine (→ `axis-saas`)
- Ingesting PHI/PII/PCI into any search index (always blocked regardless of consent)
- Does NOT begin substantive index ingestion before Data Use Boundary ADR is Accepted

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-privacy-dub` | Data Use Boundary ADR Accepted (gate); `DataUseConsent.search_indexable_classes` | ADR gate + per ingestion |
| `platform-tenancy-identity` | `TenantId` for per-tenant index isolation | Per-release |
| `platform-audit-evidence` | DSR cascade proof-of-erasure chain record; audit emission for index lifecycle | Per DSR + per lifecycle |
| `axis-foundry` | RAG endpoint contract co-ownership; Foundry queries the endpoint | Wave gate |
| `axis-ads-analytics` | Ad slot inventory for SERP stitching | Wave gate |
| `axis-cloud` | Compute cells for index shards, storage for snapshots | Wave gate |
| `platform-eventing-og` | OG property-tier schema for search-indexable property classification | Monthly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `axis-foundry` | RAG retrieval (Foundry agents query via RAG endpoint) | Every RAG retrieval |
| `axis-ads-analytics` | Organic search signals for ad quality score; impression/click stream (privacy-gated) | Auction loop |
| `axis-saas` | Tenant-private search for in-app content discovery | Monthly |
| All vertical teams | Vertical-content search via tenant-private index | Per vertical onboard |
| `gtm-customer-success` | Search quality metrics for design-partner health dashboards | Monthly |

## Success metrics
- **Search index coverage of tenant content per consent tier:** consent-target hit rate (PRD §4.2)
- **DSR cascade index deletion completion:** 100% within 24 h
- **SERP organic result relevance (KR benchmark):** quality bar set at W-Search-Stable gate (PRD §3.1)
- **RAG endpoint p99 latency:** < 200 ms
- **Tenant-private index isolation violations:** 0
- **PHI/PII/PCI detected in cross-tenant shared index:** 0 (hard zero)
- **Index lifecycle audit records:** 100% (fitness gate)

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) — index lifecycle contract disputes
- Privacy: privacy council — consent-gate disputes, DSR cascade disputes
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 45-min sync — crawler health, index lag, DSR queue, ranking quality
- Cross-team review: monthly cross-axis contract audit for search index lifecycle row

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; consent-gate PRs require privacy-reviewer
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Tenant PHI ingested into shared search index | Catastrophic | Data Use Boundary gate; `governance-data-use-boundary` CI; PHI class forced `internal_only` |
| Cross-tenant search result leak | Catastrophic | Per-tenant index isolation; CI test set for result isolation |
| DSR cascade partial completion leaves residual data in index | High | Cascade ack protocol; proof-of-erasure chain record; automated monitor |
| Crawler violates robots.txt / KR copyright law | Medium | Politeness enforcer with hard-fail; KR 저작권 compliance review quarterly |

## Sources scanned
PRD.md §3.1 (W-Search-Preview, W-Search-Stable), §4.2 (search index coverage metric), DESIGN.md §1 (Axis 6), §10 (search index lifecycle, ad slot inventory, RAG endpoint rows), products/search/PRD.md (draft).
