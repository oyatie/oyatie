---
doc_class: ImplementationPlan
impl_plan_id: IP-010-search-and-graph-view
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location, meilisearch-version-pin]
---

# IP-010: search-index + graph-view-data

## Intent

Land `oya-notes-search-index-*` (Meilisearch 0.10.0 LTS Professional-tier per ADR-NOTES-0004) + `oya-notes-graph-view-data-*` (server-side graph JSON snapshot assembly).

## Search Index

- Per-tenant Meilisearch namespace `tenant_<id>`.
- Cedar-scoped server-side filter.
- Faceted by tag + notebook + created_at.
- Personal-tier MUST NOT enter this index (compile-time refusal).

## Graph View Data

- Server emits `{nodes, edges, stats}` JSON snapshot.
- Cap 50k nodes; beyond cap, paginate by tag-cluster.
- Per ADR-NOTES-0002 client renders force-directed via WebGL.

## Acceptance Gates

```bash
cargo check -p oya-notes-search-index-kernel
cargo check -p oya-notes-search-index-adapter-meilisearch
cargo check -p oya-notes-graph-view-data-kernel
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Next IP

[`IP-011-collab-edit-loro.md`](IP-011-collab-edit-loro.md)
