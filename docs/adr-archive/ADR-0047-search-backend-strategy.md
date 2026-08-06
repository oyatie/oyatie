---
id: ADR-0047
status: Superseded
superseded_by: [ADR-700]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Search backend strategy

# ADR-0047: Search backend strategy — pgroonga day-1 (LGPL legal isolation), Tantivy in-Rust at scale, OpenSearch as Apache-2 adapter, in-house long-horizon

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-search`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0030, ADR-0045, ADR-0046, ADR-0048

---

## Context

Search backend is the indexing + retrieval engine that sits behind the search microservice (per ADR-0030). The pack-of-19 foundation ADRs decided that search is critical but did not pin the engine. The decision is constrained on three axes: (a) **license**: Elasticsearch is SSPL (forbidden in product surface); OpenSearch is Apache-2 (clean); Tantivy is MIT (clean); pgroonga is LGPL (requires legal isolation per License Policy); (b) **scale**: pgroonga is appropriate for tens of millions of docs; Tantivy / OpenSearch handles billions; (c) **KR-specific**: pgroonga's mecab-ko integration ships day-1 KR morphology with minimal additional work.

This ADR pins a four-stage trajectory: pgroonga day-1 (KR launch with legal-isolation analysis), Tantivy in-Rust at scale (transition when we cross 100M docs per cell), OpenSearch as Apache-2-only adapter for tenants that need per-tenant private enterprise search at large scale before in-house Tantivy is GA-ready, and in-house long-horizon (KR morphology + Tantivy + custom ranker).

---

## Decision

We adopt **pgroonga** day-1 with **legal isolation** per License Policy ADR + replacement plan; **Tantivy** (MIT) in-Rust at scale; **OpenSearch** (Apache-2) only as an adapter behind a port; **Elasticsearch SSPL forbidden** in product surface; **in-house long-horizon** (KR morphology + Tantivy + custom ranker) under `crates/oya-search-backend-*`.

### pgroonga day-1 (KR launch)

```sql
-- per-tenant per-cell schema (lives in OLTP per ADR-0045)
CREATE EXTENSION IF NOT EXISTS pgroonga;

CREATE TABLE search_documents (
    id BIGSERIAL PRIMARY KEY,
    tenant_id UUID NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    locale VARCHAR(8) NOT NULL,
    data_class VARCHAR(64) NOT NULL,
    inserted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX search_documents_pgroonga
  ON search_documents
  USING pgroonga (title, body)
  WITH (
    tokenizer = 'TokenMecab',     -- KR morphology via mecab-ko (per ADR-0048)
    normalizer = 'NormalizerNFKC100'
  );
```

- **License posture.** pgroonga is LGPL-2.1+. Per License Policy ADR, LGPL libraries require **legal isolation analysis**: pgroonga runs as a PostgreSQL extension in the database process; it is not statically linked to our product binaries; per FSF guidance + per License Policy this configuration is permissible with documented isolation evidence. Legal-isolation analysis ships as `docs/legal/pgroonga-legal-isolation.md`.
- **Replacement plan.** When Tantivy in-Rust reaches GA (target W+18), pgroonga is replaced for new tenant onboarding; existing tenants migrate per per-tenant migration tooling; pgroonga removed from the codebase by W+30.
- **Living in OLTP.** pgroonga is a PostgreSQL extension (per ADR-0045 OLTP tier); per-tenant per-cell sharding inherits.

### Tantivy in-Rust at scale (long-horizon transition target)

```rust
// crates/oya-search-backend-tantivy
pub struct TantivyIndex {
    pub schema: tantivy::schema::Schema,
    pub index: tantivy::Index,
    pub tenant_id: TenantId,
    pub locale: LocaleId,
    pub data_class: DataClass,
}
```

- **License.** MIT (clean).
- **Per-tenant per-cell index.** Same isolation pattern as pgvector / pgroonga.
- **KR morphology.** mecab-ko / khaiii via FFI day-1; in-house Rust port long-horizon (per ADR-0048).
- **Multi-locale tokenizer dispatch.** Per-locale tokenizer (per ADR-0048).
- **Distributed.** Per-cell shard with cross-shard merge at query time.

### OpenSearch as Apache-2-only adapter

```rust
// crates/oya-search-backend-opensearch-adapter
pub struct OpenSearchAdapter {
    pub client: opensearch::OpenSearch,
}

impl SearchBackendPort for OpenSearchAdapter { /* trait from kernel */ }
```

- **License.** Apache-2 (clean; OpenSearch is the AWS fork of pre-SSPL Elasticsearch).
- **Use case.** Per-tenant private enterprise-search SKU at scale before in-house Tantivy is ready, where pgroonga's per-cell ceiling is exceeded.
- **Adapter only.** Not the primary backend; per-tenant adapter selection.

### Elasticsearch SSPL forbidden

Per License Policy ADR (and per OSI's classification of SSPL as non-open-source):

- Elasticsearch SSPL forbidden in product surface.
- Including Elasticsearch is forbidden via dep-policy lane (per ADR-0039 supply-chain scan).
- Elastic-licensed Elasticsearch (commercial) requires per-engagement commercial licensing per License Policy.

### In-house long-horizon

`crates/oya-search-backend-*` is the long-horizon home:

- KR morphology in-Rust (per ADR-0048 in-house port).
- Tantivy as the substrate for inverted index.
- Custom ranker per ADR-0030 (BM25 + semantic rerank + freshness + authority + diversity + KR signals).
- Per-tier index segregation per ADR-0030 + ADR-0034.

Long-horizon target: GA at W+18 (replace pgroonga for new tenants).

### Per-engine matrix

| Engine | License | Day-1 use | Long-horizon use |
|---|---|---|---|
| **pgroonga** | LGPL (legal isolation) | KR launch, ≤100M docs/cell | (replaced by Tantivy) |
| **Tantivy in-house** | MIT (in-house Apache-2 wrapper) | (post-transition) | primary |
| **OpenSearch adapter** | Apache-2 | scale-burst before Tantivy ready | adapter only |
| **Elasticsearch** | SSPL | _forbidden_ | _forbidden_ |
| **Vespa** | Apache-2 | _adapter only with ADR review_ | _adapter only with ADR review_ |
| **MeiliSearch** | MIT | _adapter only with ADR review_ | _adapter only with ADR review_ |
| **Typesense** | GPL-3 (legal isolation required) | _adapter only with ADR review_ | _adapter only with ADR review_ |

### Per-cell index topology

Per ADR-0030 + ADR-0028:

- Per-cell shard map by region/locale.
- Per-tenant private namespace.
- Per-data-class segregation.
- Cross-tenant per consent only.

### Per-tenant DSR cascade

Per ADR-0038:

- pgroonga: per-row `DELETE` + index update.
- Tantivy: per-doc `delete_term` + scheduled merge.
- OpenSearch adapter: per-doc delete via adapter API.
- Per-store proof-of-erasure emitted per ADR-0038.

### Anti-scope

This ADR does not own the search architecture (per ADR-0030, but supplies the engine). Does not own the vector store (per ADR-0046, but co-located in OLTP). Does not own KR morphology FFI (per ADR-0048).

---

## Consequences

### Positive

- pgroonga day-1 ships KR morphology + KR launch capability without separate engine deployment.
- Legal-isolation analysis path lets us use a high-quality LGPL component without product-surface contamination.
- Tantivy long-horizon eliminates LGPL dependency.
- OpenSearch adapter handles the scale-burst window cleanly without requiring SSPL.
- Per-tenant per-cell index pattern is uniform across all engines.

### Negative

- LGPL legal-isolation analysis is a recurring legal cost.
- Engine transition from pgroonga to Tantivy is a per-tenant migration cost.
- Per-engine adapter parity (kernel trait surface) is a real engineering investment.
- KR morphology in-house port is a multi-year project (per ADR-0048).

### Operational

- Per-cell pgroonga / Tantivy index health monitored.
- Per-tenant index size approaching cell ceiling alerted.
- Per-quarter relevance benchmark per pack.
- Per-quarter LGPL legal-isolation evidence review.
- Per-PR forbidden-license scan (per ADR-0039 supply-chain) blocks Elasticsearch.

### Addendum 2026-06-25 -- office search rebuild planning

`oya/office/oya-office-search-kernel/src/lib.rs` now carries the provider-neutral
`SearchIndexRebuildPlan` contract for Drive search index rebuilds. The contract keeps rebuild
planning in the kernel: tenant/cell scope, active-vs-rebuild index names, source cursor, and batch
size are validated before any pgroonga, Tantivy, or OpenSearch adapter performs lifecycle work. This
preserves ADR-0047's adapter-only backend strategy while adding a rollback-safe lifecycle seam for
future managed-search parity.

The review evidence for this addendum is
`evidence/multispectrum/wavea-office-search-rebuild-20260625-1782426039.json` and
`evidence/multispectrum/wavea-office-cell-scoped-intents-20260625-1782429856.json`.

---

## Alternatives considered

### Alternative A — Elasticsearch from day 1 (SSPL)

- **Pros:** mature; large community.
- **Cons:** SSPL forbidden.
- **Rejected because:** license posture incompatible.

### Alternative B — Tantivy from day 1 (no pgroonga)

- **Pros:** no LGPL dep at all.
- **Cons:** KR morphology integration requires in-house port from day 1, which delays KR launch by 6-12 months.
- **Rejected because:** KR launch timing is critical.

### Alternative C — OpenSearch as primary

- **Pros:** Apache-2 mature engine.
- **Cons:** separate operational surface; less Rust-stack-native than Tantivy in-house long-horizon; per-tenant residency requires retrofit.
- **Rejected because:** in-house long-horizon is more aligned with the cohesion + sovereignty stance.

### Alternative D — pgroonga forever

- **Pros:** simplest.
- **Cons:** LGPL exposure perpetual; pgroonga ceiling at scale (~100M docs/cell) blocks growth.
- **Rejected because:** scale ceiling.

---

## Open questions

1. **Q1.** Tantivy in-house GA target — W+18 or W+24? Default: W+18 stretch; W+24 conservative. → ADR-0048.
2. **Q2.** OpenSearch adapter in production — only as opt-in for specific tenants, or default-on at certain scale tier? Default: opt-in; never default. → owner: `axis-search`.
3. **Q3.** pgroonga removal target — W+30 or earlier? Default: W+30, contingent on tenant migration completion. → owner: `axis-search`.
4. **Q4.** Per-tenant migration tool from pgroonga to Tantivy — at GA Tantivy or just-in-time? Default: at GA Tantivy. → owner: `axis-search`.
5. **Q5.** Vespa adapter consideration — at GA or never? Default: never at GA; revisit if specific tenant requires. → owner: `axis-search`.

---

## References

- `docs/PRD.md` §10 (search backend)
- `docs/DESIGN.md` §11 (search engine), §10 (cross-microservice contracts)
- pgroonga docs (LGPL); Tantivy docs (MIT); OpenSearch docs (Apache-2)
- OSI position on SSPL; FSF guidance on LGPL linking
- ADR-0001 (cohesion), ADR-0030 (search), ADR-0045 (database tier), ADR-0046 (vector store), ADR-0048 (Korean morphology)
