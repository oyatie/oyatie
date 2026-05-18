---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-010-search-meilisearch
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-search-tenant-scope]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: search BC + Meilisearch adapter

## Intent

Author the `search` BC per ADR-SITES-0005. Per-tenant Meilisearch 0.10.0 index. Reindex worker triggered on `PagePublished` + `CmsCollectionUpdated` events. Search scope (public_only / intranet / all) bound by Cedar policy.

## ChangeSet boundary

7 crates: `oya-sites-search-{kernel,domain,usecase,api,adapter,adapter-meilisearch,rest,worker,app}`. Cross-tenant index isolation invariant covered.

## Acceptance Gates

```bash
cargo nextest run -p oya-sites-search-adapter-meilisearch -- per_tenant_index_isolation
cargo nextest run -p oya-sites-search-adapter-meilisearch -- query_p95_lt_300ms
cargo nextest run -p oya-sites-search-worker -- reindex_on_publish
cargo run -p oya-dev-cli -- gate validate search-tenant-scope --microservice sites
```

## Test Plan

- Unit: per-tenant index naming + cross-tenant query refusal.
- Integration: reindex on `PagePublished`, `CmsCollectionUpdated`.
- Integration: 5000-page tenant query p95 ≤ 300ms.
- Pen-test: cross-tenant result leak attempt.

## ChangeSet metadata

```yaml
changeset_id: CS-SITES-IP-010-search-meilisearch
depends_on_changesets: [CS-SITES-IP-003-site-and-page-bcs, CS-SITES-IP-008-cms-collection]
parallel_safe_with_changesets: [CS-SITES-IP-006-url-routing, CS-SITES-IP-009-seo]
enables: [CS-SITES-IP-011-cdn-delivery]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Per-tenant Meilisearch index isolation: cross-tenant query refused | `cargo nextest run -p oya-sites-search-adapter-meilisearch -- per_tenant_index_isolation` |
| AC-02 | 5000-page tenant query p95 ≤ 300ms | `cargo nextest run -p oya-sites-search-adapter-meilisearch -- query_p95_lt_300ms` |
| AC-03 | Reindex triggered on `PagePublished` + `CmsCollectionUpdated` events | `cargo nextest run -p oya-sites-search-worker -- reindex_on_publish` |
| AC-04 | Cedar-scoped server-side filter excludes drafts from anonymous queries | `cargo nextest run -p oya-sites-search-domain -- cedar_scope_filter` |
| AC-05 | `oya gate validate search-tenant-scope --microservice sites` exits 0 | governance lane |

## Build Sequence

1. Kernel: `SearchIndex`, `Reindexer`, `QueryScope` ports.
2. Domain: `IndexableDocument`, `Query`, `Hit`.
3. Usecase: `IndexPage`, `IndexCmsEntry`, `Query`.
4. Adapter: `-adapter-meilisearch` pinned to 0.10.0 LTS.
5. Worker drains reindex queue.
6. `cargo nextest run -p oya-sites-search-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-sites FR | FR-12 (site search) |
| PRD-sites NFR | NFR perf — site-search p95 ≤ 300ms |
| PRD-sites AC | AC-07 (site search) |
| ADR | ADR-SITES-0005 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Cross-tenant result leak | Per-tenant Meilisearch master key + namespace; cross-tenant test |
| Reindex storm on bulk publish | Coalesce events; batch size 500 |
| Stale index after CMS schema change | Schema-version stamped on index; rebuild on bump |

## References

- Meilisearch documentation (`meilisearch.com/docs`).
- Algolia multi-tenancy guide (Algolia Docs — "Multi-tenant indexing").
- Elasticsearch multi-tenancy reference (Elastic Docs).
- ADR-SITES-0005.
