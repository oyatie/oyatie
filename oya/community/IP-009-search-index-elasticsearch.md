---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-009
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009 — search-index (Elasticsearch)

## Intent

Ship the search BC with per-tenant Elasticsearch index, cross-BC ranking, tag taxonomy, and reindex pipeline.

## Scope

- Types: `Document`, `Index`, `Tag`, `RankSignal`.
- Storage: Elasticsearch (per-tenant index `community-<tenant_id_short>-<bc>`).
- Operations: `search(q, scope)`, `reindex(scope)`, `index_document`, `delete_document`.
- Ranking signals: vote score + recency + accepted-answer + moderator-endorsed + tag-match.
- Reindex pipeline: worker drains `SearchReindexRequested` events; staggered per tenant.

## Deliverables

- Crate set: kernel + domain + usecase + api + adapter + adapter-search + worker + sdk.
- ES mapping template per BC.
- Reindex CLI.

## Acceptance

- Search p99 ≤ 500 ms.
- Reindex of 10⁷ docs ≤ 60 min.
- Per-tenant index isolation verified.
- Search relevance smoke tests pass.

## Owner

axis-community.

## Wave 15 substance conversion

### A. Problem this IP closes

Community search is the discovery layer for posts, replies, KB articles, moderation-visible reports, jobs, employer pages, and professional-profile content, but the old IP still named Elasticsearch while the current PRD and ADR-COMM-0004 prefer Meilisearch 0.10.0 LTS with Tantivy fallback.
This IP closes that drift and defines a search-index slice that can serve Reddit-style subreddit search, Teamblind workplace topics, Handshake jobs/community search, and GitHub Discussions-like developer forums.

### B. Approach

Implement search as an adapter boundary with Meilisearch primary and Tantivy embedded fallback, not hard-coded Elasticsearch.
Index documents are derived from community events and contain tenant, space, document kind, target ID, tags, moderation state, ranking signals, and redacted searchable text.
Search queries always include tenant and allowed-space filters before adapter execution.
Ranking uses vote score, recency, accepted answer, moderator endorsement, tag match, and employment-safe relevance where jobs/profile modes apply.

### C. Deliverables

- Rename or amend the IP scope away from Elasticsearch in implementation notes while preserving file name for traceability.
- Add crates `oya-community-search-index-kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-search`, `worker`, and `sdk`.
- Update catalog files under `microservices/community/catalog/oya-community-search-index-*.yaml`.
- Add event consumers for `PostCreated`, `PostEdited`, `PostDeleted`, `ReplyPosted`, `KBArticlePublished`, and `SearchReindexRequested`.
- Add index schema for posts, replies, KB articles, job/employer/profile subset documents, and moderator-only records.
- Bind SLO `microservices/community/slos/search-query-latency.openslo.yaml` and runbook `runbooks/search-rebuild.md`.

### D. Implementation steps

1. Read `decisions/ADR-COMM-0004-content-search-backend.md` and confirm Meilisearch/Tantivy backend ownership before coding.
2. Define `SearchDocumentKind` for post, reply, KB article, job posting, employer page, professional profile, and moderator record where the latter is not public.
3. Normalize document events from AsyncAPI into index commands.
4. Apply tenant and allowed-space filters before query text reaches the adapter.
5. Add redaction for anonymous workplace author identity and sensitive moderation notes.
6. Implement rank-signal weighting without engagement-feed or sponsored amplification.
7. Add full reindex worker for `community.search.reindex.requested` with per-tenant stagger and checkpoint.
8. Add contract tests for `GET /search` if present or document the API gap if no route exists.
9. Add smoke relevance tests for tag match, accepted answer, KB title, and employer/job search.
10. Add rebuild runbook evidence with maximum stale-search window.

### E. Acceptance

- IP explicitly rejects Elasticsearch as stale unless ADR-COMM-0004 is superseded.
- Search queries cannot run without tenant scope and allowed-space filters.
- Reindex worker can rebuild one tenant without touching another tenant's index.
- Search latency is checked against `search-query-latency.openslo.yaml`.
- Counterpart coverage includes Reddit, Teamblind, Handshake, GitHub Discussions, and Zendesk Help Center search.

### F. Evidence

- `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md`.
- `microservices/community/contracts/asyncapi/community-events.yaml` `SearchReindexRequested`.
- `microservices/community/slos/search-query-latency.openslo.yaml`.
- `microservices/community/runbooks/search-rebuild.md`.
- `microservices/community/feature-parity-matrix-2026-05-20.md` Discourse/Circle/Vanilla search findings.

### G. Counterpart closure

| Counterpart | Search expectation | This IP closure |
|---|---|---|
| Reddit | search by community, tag, post, comment | tenant/space/tag indexed documents |
| Teamblind | workplace topic search without identity leakage | redacted author identity and workplace-scoped filters |
| Handshake | job, employer, and candidate community search | document kinds for jobs/profile subset |
| Zendesk Help Center | public KB search | article title/body index with public-read filter |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-009-search-index-elasticsearch.md` matched `asyncapi`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-009-search-index-elasticsearch.md` matched `p99, SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
