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
