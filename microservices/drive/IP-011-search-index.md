---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-011-search-index
status: pending
execution_unit: ChangeSet
owner: axis-drive + foundry-runtime
acceptance_lanes: [cargo-build, cargo-nextest, oya-check-search-tenant-scoped]
---

# IP-011: search-index BC — Meilisearch + Apache Tika full-text + OCR handoff

## Intent

Stand up `oya-drive-search-index-*` BC. Per-tenant Meilisearch index + Apache Tika full-text extract pipeline + foundry-runtime OCR handoff per T1-drive-ocr capability. Per-context query separation; cross-tenant query refused.

## Crates

`oya-drive-search-index-{kernel,domain,usecase,api,adapter,adapter-meilisearch,adapter-tika,rest,worker,app}` (10 crates).

## Acceptance Gates

```bash
cargo nextest run -p oya-drive-search-index-domain -- per_tenant_index
cargo nextest run -p oya-drive-search-index-domain -- cross_tenant_refused
cargo nextest run -p oya-drive-search-index-adapter-meilisearch -- query_1m
cargo run -p oya-dev-cli -- gate validate search-tenant-scoped --microservice drive
```

## References

- PRD-drive §FR-08; AC-07.
- Meilisearch docs; Apache Tika docs.
