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
buck2 build //:quality-lane-registry-authority-check # lane=version-pinning-conformance
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


## A. Problem
`IP-010: search-index + graph-view-data` is not a generic implementation packet; it closes the `010 search and graph view` gap for `notes` using the service artifacts that exist in this checkout. The gap is that the current service contract names the capability, but reviewers need a concrete boundary tying the plan to real contracts, policies, SLOs, and catalog records instead of a line-count shell. Domain vocabulary for this IP: Note, PersonalNoteRef, ProfessionalNoteRef, tag-graph, backlink graph, Loro CRDT, MLS key package, share-link, E2E refusal.

## B. Approach
Knowledge retrieval combines Markdown/frontmatter note records, tag adjacency, wikilink backlink materialization, search-index facets, and graph snapshots without indexing Personal E2E plaintext. The implementation must keep the µservice boundary intact: contracts remain under `microservices/notes/contracts/openapi/notes.yaml` / `microservices/notes/contracts/proto/notes.proto`, policy decisions remain in `microservices/notes/policy/tenant-scope.cedar`, operational proof remains in `microservices/notes/slos/note-open-latency.openslo.yaml`, and the parity claim is checked against `microservices/notes/competitor-parity-matrix.md`.

## C. Deliverables
- `microservices/notes/PRD.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/ARCHITECTURE.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/openapi/notes.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/proto/notes.proto` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/contracts/asyncapi/notes-events.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/policy/tenant-scope.cedar` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/slos/note-open-latency.openslo.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/runbooks/sync-conflict-resolution.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/competitor-parity-matrix.md` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-tag-graph-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- `microservices/notes/catalog/oya-notes-backlink-graph-kernel.yaml` — verify/update as the authoritative artifact for this IP.
- Named code targets declared by this IP and `manifest.json` must be created only when the implementation PR actually adds the crates/types; this scrub does not pretend source files exist.

## D. Implementation Steps
1. Read `microservices/notes/PRD.md` and `microservices/notes/ARCHITECTURE.md` to confirm the bounded context, tenant class, and first-ship milestone for `notes`.
2. Diff the declared contract in `microservices/notes/contracts/openapi/notes.yaml` and `microservices/notes/contracts/proto/notes.proto` against the IP title so every endpoint/message has a matching domain type or explicit backlog gap.
3. Check `microservices/notes/policy/tenant-scope.cedar` plus adjacent Cedar/policy files before adding any mutation, share, webhook, agent, AI, or cross-tenant path.
4. Wire observability to `microservices/notes/slos/note-open-latency.openslo.yaml` and the relevant dashboard/runbook; no acceptance claim counts without a metric or sealed evidence path.
5. Update the catalog/capability record such as `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` so the service registry can discover the new boundary.
6. Run the IP-specific test/gate commands listed above; if a source crate is absent, record the absent crate as implementation debt rather than faking a green result.

## E. Acceptance
- Local artifact links resolve for `microservices/notes/PRD.md`, `microservices/notes/ARCHITECTURE.md`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/policy/tenant-scope.cedar`, `microservices/notes/slos/note-open-latency.openslo.yaml`, and `microservices/notes/competitor-parity-matrix.md`.
- The implementation exposes no cross-tenant, cross-pack, credential, E2E, or vendor-call path without the policy file cited in this IP.
- At least one targeted unit/contract/gate command verifies the named behavior, and any skipped command is documented with the missing artifact.
- The final PR includes evidence that counterpart parity is improved or explicitly marks the remaining gap.

## F. Evidence
- `microservices/notes/PRD.md`
- `microservices/notes/ARCHITECTURE.md`
- `microservices/notes/contracts/openapi/notes.yaml`
- `microservices/notes/contracts/proto/notes.proto`
- `microservices/notes/contracts/asyncapi/notes-events.yaml`
- `microservices/notes/policy/tenant-scope.cedar`
- `microservices/notes/slos/note-open-latency.openslo.yaml`
- `microservices/notes/runbooks/sync-conflict-resolution.md`
- `microservices/notes/catalog/oya-notes-note-store-kernel.yaml`
- `microservices/notes/competitor-parity-matrix.md`
- `microservices/notes/competitor-parity-matrix.md` — counterpart gap table used for the comparison below.

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Apple Notes, Google Keep, OneNote, Notion, Bear, Obsidian, Standard Notes, Evernote, Roam, Logseq, Joplin, Reflect, Tana, Mem, and Heptabase | Notion and OneNote define workspace/collab parity; Obsidian/Roam/Logseq define backlink and graph parity; Standard Notes and Apple Notes define privacy pressure; Evernote/Bear/Google Keep define capture/import expectations. This IP closes the relevant gap by binding `010 search and graph view` to concrete `notes` contracts, policy, SLO, catalog, and runbook evidence rather than a reusable scaffold. |
