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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
buck2 build //:quality-lane-registry-authority-check # lane=search-tenant-scoped --microservice drive
```

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-011-search-index
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain, CS-DRIVE-IP-010-permissions]
parallel_safe_with_changesets: [CS-DRIVE-IP-009-share-link, CS-DRIVE-IP-012-preview]
enables: []
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Per-tenant Meilisearch index namespace; cross-tenant query refused at compile + runtime | `cargo nextest run -p oya-drive-search-index-domain -- per_tenant_index` |
| AC-02 | Cross-tenant query returns 403 + audit-chain seal | `cargo nextest run -p oya-drive-search-index-domain -- cross_tenant_refused` |
| AC-03 | Search across 1M-file tenant corpus p95 ≤ 400ms | `cargo nextest run -p oya-drive-search-index-adapter-meilisearch -- query_1m` |
| AC-04 | `oya gate validate search-tenant-scoped --microservice drive` exits 0 | governance lane |

## Build Sequence

1. Kernel: `SearchIndex`, `TextExtractor`, `OcrHandoff` ports.
2. Domain: `IndexableDocument`, `Query`, `Hit`, `Facet`.
3. Usecase: `IndexFile`, `QueryFiles`, `ReindexAfterPermissionChange`.
4. Adapters: `-adapter-meilisearch` + `-adapter-tika`.
5. Worker drains index queue + OCR handoff via foundry-runtime event.
6. `cargo nextest run -p oya-drive-search-index-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-08 (search by filename + full-text) |
| PRD-drive NFR | NFR perf — search 1M corpus p99 ≤ 1s |
| PRD-drive AC | AC-07 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Tika sandbox escape via crafted document | Tika runs in gVisor sandbox; output rasterised to bytes |
| Cross-tenant index leak via Meilisearch global keys | Per-tenant Meilisearch master key; never shared |
| Reindex storm on bulk permission change | Coalesce reindex events; max-batch 500 docs |

## References

- PRD-drive §FR-08; AC-07.
- Meilisearch documentation (`meilisearch.com/docs`).
- Apache Tika 2.x documentation (`tika.apache.org/2.9.0/index.html`).
- Elasticsearch tenancy model reference (Elastic docs — "Multi-tenancy").
- Google Drive search reference (Workspace Help — "Search and find files").
