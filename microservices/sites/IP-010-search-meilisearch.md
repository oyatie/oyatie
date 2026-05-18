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

## References

- Meilisearch documentation — `meilisearch.com/docs`.
- ADR-SITES-0005.
