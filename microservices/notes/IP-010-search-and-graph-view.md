---
doc_class: ImplementationPlan
impl_plan_id: IP-010-search-and-graph-view
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location, meilisearch-version-pin]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## ChangeSet metadata

```yaml
changeset_id: CS-NOTES-IP-010-search-and-graph-view
depends_on_changesets: [CS-NOTES-IP-003-note-store-kernel-domain, CS-NOTES-IP-005-tag-graph-and-backlink]
parallel_safe_with_changesets: [CS-NOTES-IP-008-share-link-and-embed, CS-NOTES-IP-009-checklist-and-version-history]
enables: [CS-NOTES-IP-011-collab-edit-loro]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Personal-tier notes refused at compile time from entering search index | `cargo nextest run -p oya-notes-search-index-domain -- personal_tier_refused_at_type` |
| AC-02 | Per-tenant Meilisearch namespace `tenant_<id>` enforced | `cargo nextest run -p oya-notes-search-index-adapter-meilisearch -- per_tenant_namespace` |
| AC-03 | Full-text search p95 ≤ 200ms on Professional corpus (PRD perf table) | `cargo nextest run -p oya-notes-search-index-adapter-meilisearch -- fts_p95` |
| AC-04 | Graph snapshot emits `{nodes, edges, stats}` JSON ≤ 50k nodes | `cargo nextest run -p oya-notes-graph-view-data-domain -- node_cap_50k` |
| AC-05 | 5k-note vault graph emits in p95 ≤ 1s server-side | `cargo nextest run -p oya-notes-graph-view-data-domain -- graph_5k_p95` |

## Build Sequence

1. Kernel: `SearchIndex`, `GraphSnapshotter`, `TenantScope` ports.
2. Domain: `IndexableNote`, `SearchQuery`, `GraphNode`, `GraphEdge`.
3. Usecase: `IndexNote`, `Query`, `EmitGraphSnapshot`.
4. Adapter: `-adapter-meilisearch` pinned to 0.10.0 LTS (ADR-NOTES-0004).
5. `cargo nextest run -p oya-notes-search-index-* -p oya-notes-graph-view-data-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-notes FR | FR-13 (search), FR-14 (graph view) |
| PRD-notes NFR | NFR perf — full-text p95 ≤ 200ms; graph 5k p95 ≤ 1s |
| ADR | ADR-NOTES-0002, ADR-NOTES-0004 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Personal-tier note accidentally indexed | Compile-time type refusal + adapter cross-check |
| 50k+ node vault renders unusable graph | Tag-cluster pagination beyond 50k cap |
| Meilisearch version drift breaks query semantics | Version pinned 0.10.0 LTS; `version-pinning-conformance` gate |

## References

- Meilisearch documentation (`meilisearch.com/docs`).
- Obsidian graph view design notes (Obsidian Help — "Graph view").
- Force-Atlas 2 algorithm — Jacomy et al. (PLOS ONE 2014).
- ADR-NOTES-0002 (graph view), ADR-NOTES-0004 (Meilisearch pinning).

## Next IP

[`IP-011-collab-edit-loro.md`](IP-011-collab-edit-loro.md)
