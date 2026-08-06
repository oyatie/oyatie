---
id: ADR-NTS-001
title: Block-Based Data Model with Bidirectional Links
status: Proposed
date: 2026-05-20
microservice: notes
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-notes
---

# ADR-NTS-001: Block-Based Data Model with Bidirectional Links

## Context

- Notes owns personal notes, professional notes, notebooks, backlink graph, search, encrypted personal-tier storage, import/export, and optional collaboration.
- Existing ADR-NOTES-0002 chose note-level bidirectional links for the minimum-shippable tier.
- This ADR proposes the product-grade successor model for block-level notes while preserving personal-tier privacy.
- Named pressure NTS-P1: Notion-style users expect paragraphs, headings, tasks, embeds, callouts, and databases to move independently.
- Named pressure NTS-P2: Roam-style users expect stable links to blocks, not only pages.
- Named pressure NTS-P3: enterprise knowledge bases need comment, mention, and task extraction at sub-note granularity.
- Named pressure NTS-P4: personal-tier encryption prevents server-side parsing, so link derivation must be tier-shaped.
- Named pressure NTS-P5: flat documents with string references make partial sync and conflict isolation too coarse.
- Named precedent: Notion stores documents as typed blocks with parent-child relationships.
- Named precedent: Roam Research popularized block references and backlinks as the primary knowledge primitive.
- Named precedent: Obsidian and Logseq treat links as first-class graph edges but differ on block granularity.
- Constraint NTS-C1: tenant and user scope must come from ADR-0244.
- Constraint NTS-C2: every block mutation and link-resolution event must emit evidence per ADR-0263.
- Constraint NTS-C3: Cedar must authorize block read, block write, backlink resolve, and graph export per ADR-0243.
- Constraint NTS-C4: public block APIs must follow ADR-0258 additive versioning.
- Constraint NTS-C5: personal-tier plaintext block contents remain client-side unless the user installs a tenant-controlled sync enclave.
- Constraint NTS-C6: block ids must be stable across export, import, offline edit, and merge.
- Constraint NTS-C7: block tree cycles are impossible by construction.
- Constraint NTS-C8: dangling links are valid product state and must not block save.
- Constraint NTS-C9: server-side graph views for professional tier must stay under the 5k note graph p95 budget.
- Constraint NTS-C10: export must not trap user content in a proprietary-only representation.
- The existing note-level adjacency table remains a compatibility projection.
- The block model must be implementable without changing notes into the docs microservice.
- The block model must support task extraction without making tasks the persistence owner of notes content.
- The block model must support offline-first mobile editing.
- The block model must keep backlinks separate from search indexing because encryption and retention differ.
- This ADR is Proposed because it widens the notes model beyond the minimum accepted note-level ADR.

## Decision

- Adopt a block-based note data model for the next notes service model.
- Represent every note as a rooted ordered tree of blocks.
- Use a stable ULID `block_id` for every block.
- Use `note_id` as the root aggregate id and `root_block_id` as the tree root.
- Store block hierarchy as parent id plus fractional order key.
- Store block content as typed JSON with a canonical schema per block type.
- Define core block types `paragraph`, `heading`, `todo`, `quote`, `code`, `callout`, `embed`, `table`, and `attachment_ref`.
- Store inline references as structured spans, not raw markdown-only strings.
- Represent `[[Page]]`, `((block_id))`, tags, mentions, and attachments as `LinkSpan` values.
- Maintain a materialized `note_link_edge` projection for professional-tier server graph views.
- Maintain a client-local link projection for personal-tier encrypted vaults.
- Preserve the old note-level backlink projection as a derived view.
- Resolve page links by explicit `target_note_id` when present.
- Resolve unresolved page links with deterministic title search and user-visible disambiguation.
- Resolve block links only by stable `block_id`.
- Keep dangling links as rows with `target_state = "dangling"`.
- Keep backlinks bidirectional by projection, not by mutating the target note body.
- Use optimistic concurrency with block-level version stamps.
- Merge non-overlapping block edits without rewriting the full note.
- Detect parent-child cycle attempts before write.
- Store deleted blocks as tombstones until retention and sync windows expire.
- Emit one audit event per accepted block mutation.
- Emit one audit event per backlink projection rebuild.
- Enforce Cedar at note, block, and graph export levels.
- Keep export formats Markdown, JSON, and HTML available.
- Encode block references in Markdown export using stable HTML anchors.
- Allow import to preserve external block ids only under a namespaced import map.
- Keep graph rendering client-side with server JSON snapshots.
- Keep block search index optional and pack-gated for personal-tier data.
- Expose block APIs under `/v1/notes/blocks`.
- Keep note-level APIs working by projecting root block children to legacy note body views.

## Alternatives Considered

### Flat Document with Inline References

- Pros: simplest storage shape.
- Pros: compatible with markdown export.
- Pros: easier full-text indexing.
- Cons: partial sync touches the whole note.
- Cons: block-level comments, tasks, and embeds are awkward.
- Cons: backlinks cannot target stable sub-document units.
- Rejected because the product direction needs stable sub-note objects.

### Note-Level Graph Only

- Pros: already accepted for the minimum-shippable tier.
- Pros: graph cardinality is lower and easier to cache.
- Pros: personal-tier client derivation is simple.
- Cons: cannot support Roam-style block references.
- Cons: task extraction loses source location.
- Cons: note-level conflicts are too coarse for collaboration.
- Rejected as the long-term model, retained as a compatibility projection.

### Full Notion-Style Database Engine Inside Notes

- Pros: most powerful workspace primitive.
- Pros: supports tables, views, and structured records.
- Pros: competitive with Notion databases.
- Cons: turns notes into a second sheets/tasks/database product.
- Cons: expands schema, query, and permission surface dramatically.
- Cons: conflicts with flat microservice boundaries.
- Rejected because notes should embed and link to other products, not own their engines.

### Graph Database as Primary Store

- Pros: native traversal for backlinks.
- Pros: flexible graph queries.
- Pros: block and link are equally first-class.
- Cons: adds a new substrate service.
- Cons: tenant isolation and backup semantics are harder than Postgres rows.
- Cons: ordered tree editing remains awkward in a graph database.
- Rejected because Postgres hierarchy plus projection is enough.

### Block Tree with Materialized Link Projection

- Pros: supports stable block references and Notion-style editing.
- Pros: keeps primary writes in Postgres with clear tenancy.
- Pros: lets personal-tier derive graph client-side.
- Cons: more schema and merge logic than flat documents.
- Cons: import/export must preserve block ids carefully.
- Cons: projection lag can make backlinks briefly stale.
- Accepted as the proposed long-term model.

## Consequences

- Positive: note edits can sync and merge at block granularity.
- Positive: backlinks can target paragraphs, tasks, and embedded objects.
- Positive: task extraction can preserve source block lineage.
- Positive: personal-tier privacy remains intact through client-local projection.
- Positive: professional-tier graph views keep server-side performance.
- Positive: flat markdown export remains possible as a projection.
- Positive: comments and mentions can attach to stable blocks.
- Positive: future mobile offline editing has smaller conflict windows.
- Negative: schema complexity increases relative to the existing note body.
- Negative: block id preservation becomes a portability requirement.
- Negative: link projection lag must be visible and observable.
- Negative: UI must explain dangling blocks and dangling note links.
- Negative: server cannot build personal-tier backlinks without client help.
- Neutral: old note-level ADR remains valid for minimum tier.
- Neutral: note-level APIs stay as compatibility views.
- Neutral: block-level references can be gated behind capability tiers.
- Neutral: docs microservice remains the richer collaborative document editor.
- Neutral: graph database adoption remains a future option if projection queries dominate.

## Implementation Notes

- Data shape `NoteRoot`: `{tenant_id, user_id, note_id, root_block_id, title, vault_id, encryption_tier, created_at, updated_at}`.
- Data shape `NoteBlock`: `{tenant_id, note_id, block_id, parent_block_id, order_key, block_type, content_json, block_version, tombstoned_at}`.
- Data shape `BlockTextSpan`: `{span_id, kind, start_utf16, end_utf16, attrs, target_ref}`.
- Data shape `LinkSpan`: `{source_block_id, span_id, link_kind, target_note_id, target_block_id, unresolved_label, target_state}`.
- Data shape `NoteLinkEdge`: `{tenant_id, source_note_id, source_block_id, target_note_id, target_block_id, kind, resolved_at}`.
- Data shape `BlockMutation`: `{mutation_id, tenant_id, note_id, actor_id, base_block_version, op, payload, causal_event_id}`.
- Data shape `BlockExportMap`: `{export_id, note_id, block_id, external_anchor, import_namespace}`.
- Postgres table `note_block` is partitioned by tenant hash.
- Postgres index `note_block_parent_idx` covers `(tenant_id, note_id, parent_block_id, order_key)`.
- Postgres index `note_link_target_idx` covers `(tenant_id, target_note_id, target_block_id)`.
- Personal-tier block content is ciphertext; server stores block envelope and metadata only.
- Professional-tier block content can be parsed by backlink worker if policy allows.
- Fractional order keys use lexicographic base-62 strings with rebalance job after high density.
- Block tombstones remain for 90 days by default or longer under legal hold.
- REST endpoint `POST /v1/notes/{note_id}/blocks` creates a child block.
- REST endpoint `PATCH /v1/notes/{note_id}/blocks/{block_id}` applies one block mutation.
- REST endpoint `POST /v1/notes/{note_id}/blocks/{block_id}/move` changes parent and order.
- REST endpoint `GET /v1/notes/{note_id}/blocks/tree` returns an ordered block tree.
- REST endpoint `GET /v1/notes/{note_id}/backlinks` returns block-aware backlinks.
- REST endpoint `POST /v1/notes/graph/snapshots` builds a professional-tier graph snapshot.
- REST endpoint `POST /v1/notes/import/block-map` imports external block anchors.
- AsyncAPI channel `notes.block.created.v1` publishes block creation.
- AsyncAPI channel `notes.block.mutated.v1` publishes accepted mutation.
- AsyncAPI channel `notes.link.resolved.v1` publishes link projection result.
- AsyncAPI channel `notes.graph.snapshot.ready.v1` publishes graph availability.
- Cedar permit `notes::block::read` requires note read permission and block not tombstoned.
- Cedar permit `notes::block::write` requires note edit permission and actor scope.
- Cedar permit `notes::graph::export` requires tenant graph export entitlement.
- Cedar forbid `notes::block::server_parse` when `resource.encryption_tier == "personal_e2ee"`.
- Cedar forbid `notes::block::move` when move would create parent-child cycle.
- Audit event `EVT-NOTES-BLOCK-CREATED` includes note id, block id, parent id, and block type.
- Audit event `EVT-NOTES-BLOCK-MUTATED` includes base version, new version, and mutation hash.
- Audit event `EVT-NOTES-LINK-RESOLVED` includes source block and target state.
- Audit event `EVT-NOTES-GRAPH-REBUILT` includes node count, edge count, and projection lag.
- Metric `notes_block_mutation_latency_ms` tracks accepted write latency.
- Metric `notes_link_projection_lag_seconds` tracks worker lag.
- Metric `notes_block_tree_depth` tracks pathological tree depth.
- Metric `notes_dangling_link_total` tracks unresolved link count by vault.
- Metric `notes_personal_client_projection_age_seconds` tracks client-reported projection freshness.
- Trace span `notes.block.mutate` links REST write to projection events.
- Trace span `notes.link.resolve` records parser version and projection mode.
- Log schema `NotesBlockMutationLog` includes tenant hash, note hash, block type, op, and result.
- SLO target: block mutation p99 <= 100 ms for professional-tier hot notes.
- SLO target: backlink projection p99 <= 5 seconds after accepted mutation.
- SLO target: graph snapshot p95 <= 1 second for 5k note vaults.
- SLO target: client-local personal projection freshness p95 <= 30 seconds when online.
- Capacity math: 10 million notes with 20 blocks each yields 200 million block rows; hash partition by tenant and monthly tombstone pruning keeps active partitions bounded.
- Capacity math: average 4 link spans per note and 20 blocks per note gives 40 million link rows for 10 million notes, below backlink index budget with tenant partitioning.
- Capacity math: a 5k note vault with 20 blocks per note has 100k blocks; graph snapshot exports note and hot block references, not every text span.
- Rollback path: expose legacy note body projection while disabling new block creation.
- Rollback path: rebuild `note_link_edge` from block content or client reports.
- Rollback path: restore previous root tree from block mutation event log.
- Multi-region path: write block mutations in home cell and replicate read-only graph snapshots.
- Sovereign-cell path: personal and regulated professional notes remain in approved cells only.
- Versioning: block schema v1 is additive by block type and content field.
- Deprecation: block types require 365-day read support after write deprecation.

## Verification

- Unit test `block_move_rejects_cycle` prevents parent-child cycles.
- Unit test `personal_tier_server_parse_forbidden` proves privacy boundary.
- Unit test `dangling_link_persists_without_save_failure` proves dangling links are valid.
- Unit test `block_reference_resolves_by_block_id` avoids title ambiguity.
- Unit test `legacy_note_body_projection_matches_root_children` preserves compatibility.
- Property test `fractional_order_round_trips_after_random_moves` checks ordering.
- Property test `block_mutations_merge_when_disjoint` checks offline sync.
- Property test `link_projection_matches_parser_output` checks professional-tier projection.
- Fuzz test `block_content_schema_rejects_malformed_spans` protects parser.
- Integration test `professional_backlink_visible_after_edit` checks projection lag.
- Integration test `personal_backlink_client_report_never_sends_plaintext` checks E2E mode.
- Integration test `task_extraction_preserves_source_block_id` checks tasks integration.
- Integration test `markdown_export_preserves_block_anchors` checks portability.
- Load test `five_k_note_graph_snapshot_under_one_second` verifies graph budget.
- Load test `hundred_k_block_mutation_stream_rebuild` verifies recovery.
- Chaos test `projection_worker_crash_replays_from_block_events` checks idempotency.
- Chaos test `postgres_partition_unavailable_serves_cached_read_only_tree` checks degradation.
- Metric check: dashboard `notes/graph-health` adds projection lag and dangling-link panels.
- Metric check: dashboard `notes/editor-experience` adds block mutation latency.
- Audit check: every accepted block mutation emits `EVT-NOTES-BLOCK-MUTATED`.
- Static check: server parser code cannot import personal-tier decryptor.
- Contract check: OpenAPI marks `/v1/notes/blocks` as additive v1.
- Regression check: existing note-level ADR-NOTES-0002 remains linked as predecessor.

## References

- Notion public API block object documentation.
- Roam Research block reference documentation.
- Obsidian backlinks and graph view documentation.
- Logseq block and page reference documentation.
- PostgreSQL ltree and adjacency-list design notes.
- CRDT literature for offline-first tree editing.
- Cedar policy language documentation.
- ADR-NOTES-0001 e2e encryption default personal tier.
- ADR-NOTES-0002 bidirectional link and graph storage.
- ADR-NOTES-0003 CRDT library for optional collaboration.
- ADR-0244 tenant-as-universal-scoping-primitive.
- ADR-0263 observability-emission-contract.
- microservices/notes/PRD.md.
- microservices/notes/capacity-model.md.
- microservices/notes/runbooks/crdt-divergence-recovery.md.
