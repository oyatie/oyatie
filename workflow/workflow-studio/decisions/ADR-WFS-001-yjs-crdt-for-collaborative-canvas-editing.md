---
id: ADR-WFS-001
title: Yjs CRDT for Collaborative Canvas Editing
status: Proposed
date: 2026-05-20
microservice: workflow-studio
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-workflow-studio
---

# ADR-WFS-001: Yjs CRDT for Collaborative Canvas Editing

## Context

- Workflow Studio is the visual editor and DSL authoring surface for Oyatie workflows.
- Workflow Studio must let multiple users edit the same workflow canvas concurrently.
- Workflow Studio must support offline-first editing when a browser loses connectivity.
- Workflow Studio must emit canonical `workflow_spec.v1.json` for workflow-engine execution.
- Workflow Studio must preserve byte-stable round trips between the visual canvas, DSL, and canonical spec.
- Workflow Studio must not make transient UI presence part of the runtime workflow source of truth.
- Workflow Studio must support node movement, edge editing, comments, selections, validation findings, and draft metadata.
- Workflow Studio must preserve every committed semantic edit even when collaborators reconnect out of order.
- Workflow Studio must detect semantic conflicts that cannot be merged automatically.
- Workflow Studio must expose conflicts explicitly instead of silently dropping edits.
- Existing service architecture includes collab CRDT crates and merge-latency SLOs.
- Existing service architecture includes `collab-crdt-no-silent-loss` as a correctness target.
- ADR-0145 requires collaboration sync to use explicit service contracts rather than hidden browser-to-engine coupling.
- ADR-0211 discourages surrendering a core editing substrate to an opaque external control plane.
- ADR-0243 requires Cedar gates for joining sessions, reading drafts, syncing updates, snapshotting, and emitting specs.
- ADR-0244 requires tenant identity to scope every collaboration session and persisted update.
- ADR-0245 places Workflow Studio in the product/tooling layer, while workflow-engine remains the substrate runtime.
- ADR-0257 requires ontology object type changes to have versioning and deprecation handshakes.
- ADR-0263 requires collaboration, projection, and validation operations to emit trace and metric data.
- The primary technical choice is CRDT-based collaboration versus operational transformation.
- Yjs provides a mature CRDT implementation for shared documents and offline update merging.
- Operational transformation can power collaborative text editors, but it usually depends on a central transform authority.
- Canvas editing differs from plain text because nodes, edges, handles, comments, and layout metadata are semantically typed.
- Canvas editing also has mixed conflict classes: geometry changes often merge, while edge target changes may require explicit user choice.
- Browser tabs may edit for hours while disconnected from the collaboration service.
- Mobile or low-connectivity users may reconnect after other users have changed the same canvas.
- Studio must support local persistence in IndexedDB so page reloads do not lose drafts.
- Studio must support websocket sync when online.
- Studio must support server-side snapshots to bound replay time.
- Studio must support append-only binary update storage for audit and recovery.
- Studio must not store awareness heartbeats as durable semantic edits.
- Studio must not feed partially merged canvas state directly into workflow-engine.
- Studio must compile CRDT state into `workflow_spec.v1.json` through a deterministic projection step.
- Studio must validate that projected specs satisfy workflow-engine schema and ontology constraints.
- Studio must attach projection diagnostics to the collaboration session.
- Studio must let Cedar policy deny spec emission even if local CRDT sync succeeds.
- Studio must let users continue local drafts while disconnected from policy services.
- Studio must re-check policy before accepting remote sync and before publishing a spec.
- The collaboration substrate must expose enough metadata for audit-chain without logging sensitive draft payloads indiscriminately.
- The collaboration substrate must handle tenant-scoped encryption and key rotation.
- The collaboration substrate must support export for incident review.
- The collaboration substrate must avoid write locks for normal editing.
- The collaboration substrate must avoid last-writer-wins semantics for semantic canvas edits.
- The collaboration substrate must provide deterministic convergence for supported edit types.
- The collaboration substrate must separate semantic document state from ephemeral presence state.
- The collaboration substrate must provide compatibility boundaries for future editor clients.
- The decision is about active collaborative editing state, not about the workflow-engine runtime format.

## Decision

- Adopt Yjs as the CRDT substrate for Workflow Studio collaborative canvas editing.
- Use one Y.Doc per collaboration session.
- Store active canvas semantic state in Yjs shared maps and arrays.
- Store binary Yjs updates as append-only records on the server.
- Store periodic compacted Yjs snapshots to bound recovery and reconnect cost.
- Use websocket sync for online collaboration.
- Use IndexedDB persistence for browser-local offline drafts.
- Use Yjs awareness for cursor, selection, viewport, and presence state.
- Do not persist awareness state as audit-chain semantic evidence.
- Treat Yjs state as the collaborative draft source during an active session.
- Treat `workflow_spec.v1.json` as the canonical execution artifact after deterministic projection.
- Treat workflow-engine as the only durable execution runtime.
- Do not let workflow-engine consume raw Yjs documents.
- Compile Yjs canvas state into canonical spec through a `SpecProjection` pipeline.
- Require projection output to be byte-stable for unchanged semantic state.
- Require projection output to include ontology type versions.
- Require projection output to include workflow definition version intent.
- Require projection output to include validation findings when publication is blocked.
- Reject projection when edge endpoints, node handlers, ontology types, or policy tags are unresolved.
- Represent canvas nodes as typed CRDT entries keyed by stable node id.
- Represent edges as typed CRDT entries keyed by stable edge id.
- Represent comments as typed CRDT entries keyed by stable comment id.
- Represent layout as a CRDT map that can merge independent node movements.
- Represent text fields as Y.Text only when collaborative text editing is needed.
- Represent DSL text views as projections, not as separate authoritative documents.
- Maintain a deterministic ordering field for nodes and edges where order affects generated spec bytes.
- Use explicit conflict markers for semantic conflicts that converge structurally but cannot be published.
- Use conflict marker type `semantic_conflict.v1`.
- Examples of semantic conflicts include two users assigning different activity handlers to the same node.
- Examples of semantic conflicts include an edge moved to a deleted target node.
- Examples of semantic conflicts include ontology type downgrade after a node already uses a removed field.
- Examples of mergeable edits include independent node moves, independent comment edits, and adding unrelated nodes.
- Snapshot every 500 server-accepted updates or every 60 seconds, whichever comes first.
- Keep offline local update queues for 30 days or 100 MiB per browser profile, whichever comes first.
- Require users to explicitly export or discard drafts that exceed offline limits.
- Target merge latency p99 under 100 ms for updates under 64 KiB.
- Target sync acknowledgement p99 under 250 ms inside a healthy region.
- Target spec projection p99 under 1 second for workflows with up to 500 nodes.
- Target `collab-crdt-no-silent-loss` at 1.0 for accepted semantic updates.
- Target websocket session availability at 99.9 percent monthly per certified cell.
- Use server-side admission to reject updates that exceed document, tenant, or policy limits.
- Use document size warning at 20 MiB compressed snapshot size.
- Use hard document size stop at 50 MiB compressed snapshot size until split support exists.
- Use awareness heartbeat every 15 seconds.
- Expire awareness peers after 45 seconds without heartbeat.
- Encrypt persisted updates and snapshots with tenant-scoped keys.
- Store update hashes so audit-chain can verify sequence integrity without storing every payload inline.
- The existing `ADR-WS-*` series remains valid local history; this ADR records the Batch B per-microservice collaboration posture.

## Alternatives Considered

### Alternative 1: Operational transformation with central transform server

- Operational transformation has a long history in collaborative editors.
- Operational transformation can provide intuitive text editing semantics.
- Operational transformation can support a central ordered operation log.
- Operational transformation is easier to reason about when all clients remain online.
- Operational transformation requires transform functions for every operation pair.
- Operational transformation becomes difficult for typed graph operations with offline branches.
- Operational transformation usually relies on a central authority to order and transform operations.
- Operational transformation makes multi-day disconnected editing harder to support.
- Operational transformation increases risk that a rare operation pair silently corrupts canvas semantics.
- Operational transformation would require bespoke transforms for node move, edge retarget, handler change, ontology upgrade, comment edit, and deletion.
- Operational transformation was rejected because offline-first graph editing is a better fit for CRDT convergence.

### Alternative 2: Server locks with pessimistic checkout

- Server locks are simple to implement for single-user editing.
- Server locks can prevent conflicting concurrent writes.
- Server locks can make compliance review easier for strict change windows.
- Server locks can be useful for final publication approval.
- Server locks block natural pair editing and review workflows.
- Server locks fail poorly when a browser disconnects while holding a lock.
- Server locks create manual unlock operations and stale ownership problems.
- Server locks do not satisfy offline-first collaboration.
- Server locks reduce the editor to sequential editing rather than concurrent drafting.
- Server locks were rejected for normal canvas editing.
- Server locks may still be used for publication freeze or certified release approval.

### Alternative 3: Last-writer-wins document replacement

- Last-writer-wins replacement is operationally simple.
- Last-writer-wins replacement is easy to store in a versioned document table.
- Last-writer-wins replacement can work for single-user drafts.
- Last-writer-wins replacement discards concurrent semantic edits.
- Last-writer-wins replacement makes no-silent-loss impossible for collaborative editing.
- Last-writer-wins replacement creates confusing behavior when offline users reconnect.
- Last-writer-wins replacement hides conflicts until users notice missing work.
- Last-writer-wins replacement conflicts with the `collab-crdt-no-silent-loss` target.
- Last-writer-wins replacement was rejected for collaboration.
- It remains acceptable for non-collaborative preference fields that do not affect workflow semantics.

### Alternative 4: Custom CRDT implementation

- A custom CRDT could be shaped exactly around Workflow Studio graph semantics.
- A custom CRDT could avoid unused Yjs features.
- A custom CRDT could provide Rust-native storage and validation paths from day one.
- A custom CRDT would require substantial algorithm design and testing.
- A custom CRDT would delay editor delivery.
- A custom CRDT would increase correctness risk in a critical authoring surface.
- A custom CRDT would need independent interoperability, persistence, and awareness protocols.
- A custom CRDT would duplicate mature behavior already available in Yjs.
- A custom CRDT was rejected for the initial collaboration substrate.
- Domain-specific semantic validation will still be built above Yjs.

### Alternative 5: Git-like branch and merge only

- Git-like branch and merge is familiar for engineers.
- Git-like branch and merge gives explicit review points.
- Git-like branch and merge can support durable offline work.
- Git-like branch and merge is awkward for real-time cursor and canvas collaboration.
- Git-like branch and merge makes non-technical workflow authors resolve diffs.
- Git-like branch and merge would require visual merge tooling for every graph operation.
- Git-like branch and merge can still be useful for published workflow version history.
- Git-like branch and merge was rejected as the active collaboration model.
- Version branches remain useful after spec projection and publication.

## Consequences

- Positive: concurrent canvas edits converge without central transform ordering.
- Positive: disconnected users can keep editing local drafts.
- Positive: reconnect can merge binary CRDT updates instead of replacing whole documents.
- Positive: local IndexedDB persistence reduces accidental browser data loss.
- Positive: Yjs awareness cleanly separates presence from semantic document state.
- Positive: Workflow Studio can provide real-time collaboration without making workflow-engine aware of editor internals.
- Positive: deterministic spec projection preserves the runtime boundary.
- Positive: update hashes and snapshots provide audit and recovery anchors.
- Positive: CRDT state supports future editor surfaces beyond the current Leptos app.
- Positive: merge behavior can be tested independently of workflow-engine execution.
- Negative: CRDT documents are harder to inspect manually than plain JSON.
- Negative: Yjs binary updates require careful versioning and backup tooling.
- Negative: semantic conflicts still require domain-specific detection above CRDT convergence.
- Negative: debugging offline merge cases requires specialized fixtures.
- Negative: CRDT snapshots can grow large if canvas structure is not compacted.
- Negative: projection bugs can produce byte instability even when CRDT convergence is correct.
- Neutral: OT remains a valid pattern for text-only editors but is not selected here.
- Neutral: server locks may still apply to final publication approval.
- Neutral: Git-like version branches remain useful after projected spec publication.
- Neutral: this decision does not decide workflow-engine execution semantics.
- Follow-up: define `CanvasYDoc` schema version `canvas_doc.v1`.
- Follow-up: define `workflow_spec.v1.json` projection contract shared with workflow-engine.
- Follow-up: define semantic conflict marker rendering in the Studio UI.
- Follow-up: define IndexedDB eviction and export UX for offline queue limits.
- Follow-up: define Yjs snapshot compaction and update garbage-collection policy.
- Follow-up: define tenant-key rotation handling for persisted updates and snapshots.
- Follow-up: define projection byte-stability tests for every node and edge type.
- Follow-up: define import migration from existing `ADR-WS-*` collaboration decisions if names diverge.
- Follow-up: define incident runbook for corrupt update sequence or failed snapshot restore.
- Follow-up: define cross-tab coordination so multiple tabs from one user do not duplicate offline queues.

## Implementation Notes

- Use data shape `CanvasYDoc` for the active collaboration document.
- `CanvasYDoc` contains `meta`, `nodes`, `edges`, `comments`, `layout`, `validation`, and `conflicts`.
- `CanvasYDoc.meta` is a Y.Map keyed by `doc_id`, `tenant_id`, `cell_id`, `schema_version`, `created_at`, `updated_at`, and `base_spec_hash`.
- `CanvasYDoc.nodes` is a Y.Map from `node_id` to `CanvasNode`.
- `CanvasYDoc.edges` is a Y.Map from `edge_id` to `CanvasEdge`.
- `CanvasYDoc.comments` is a Y.Map from `comment_id` to `CanvasComment`.
- `CanvasYDoc.layout` is a Y.Map from `node_id` to `CanvasLayout`.
- `CanvasYDoc.validation` is a Y.Map from `finding_id` to `ValidationFinding`.
- `CanvasYDoc.conflicts` is a Y.Map from `conflict_id` to `SemanticConflict`.
- `CanvasNode` contains `node_id`, `node_type`, `handler_ref`, `ontology_type_ref`, `label`, `input_ports`, `output_ports`, `config`, `policy_tags`, and `order_key`.
- `CanvasEdge` contains `edge_id`, `source_node_id`, `source_port`, `target_node_id`, `target_port`, `condition`, `order_key`, and `policy_tags`.
- `CanvasComment` contains `comment_id`, `anchor_ref`, `body`, `author_principal`, `created_at`, and `resolved_at`.
- `CanvasLayout` contains `x`, `y`, `width`, `height`, `lane_id`, and `collapsed`.
- `ValidationFinding` contains `finding_id`, `severity`, `code`, `message`, `anchor_ref`, and `blocking`.
- `SemanticConflict` contains `conflict_id`, `conflict_type`, `anchors`, `detected_at`, `choices`, and `resolution_status`.
- `YjsUpdateEnvelope` contains `update_id`, `session_id`, `tenant_id`, `cell_id`, `client_id`, `clock`, `update_bytes`, `update_hash`, `base_snapshot_id`, `received_at`, and `policy_decision_id`.
- `YjsSnapshot` contains `snapshot_id`, `session_id`, `tenant_id`, `cell_id`, `schema_version`, `snapshot_bytes`, `snapshot_hash`, `last_update_id`, `created_at`, and `document_size_bytes`.
- `AwarenessState` contains `client_id`, `principal_id`, `cursor`, `selection`, `viewport`, `active_node_id`, `last_seen_at`, and `color_token`.
- `SpecProjection` contains `projection_id`, `session_id`, `source_snapshot_id`, `spec_hash`, `spec_bytes`, `diagnostics`, `created_by`, and `created_at`.
- Persist updates in append-only table `workflow_studio_yjs_update`.
- Persist snapshots in table `workflow_studio_yjs_snapshot`.
- Persist active sessions in table `workflow_studio_collab_session`.
- Persist projection records in table `workflow_studio_spec_projection`.
- Use server-side sequence numbers only for storage order, not for CRDT convergence semantics.
- Use update hashes to anchor audit-chain integrity evidence.
- Use snapshot hashes to prove restore identity.
- Use content-addressed object storage for large snapshots above inline storage thresholds.
- Use `POST /v1/workflow-studio/collab/sessions` to create a collaboration session.
- Use `GET /v1/workflow-studio/collab/sessions/{session_id}` to read session metadata.
- Use websocket `/v1/workflow-studio/collab/sessions/{session_id}/sync` for Yjs update sync.
- Use `POST /v1/workflow-studio/collab/sessions/{session_id}/updates` for batch update upload after offline reconnect.
- Use `GET /v1/workflow-studio/collab/sessions/{session_id}/snapshot` to fetch the latest authorized snapshot.
- Use `POST /v1/workflow-studio/collab/sessions/{session_id}/snapshot` for server snapshot creation.
- Use `POST /v1/workflow-studio/collab/sessions/{session_id}/presence` for awareness fallback when websocket reconnects.
- Use `POST /v1/workflow-studio/collab/sessions/{session_id}/spec-projections` to emit a deterministic spec projection.
- Use `GET /v1/workflow-studio/collab/sessions/{session_id}/spec-projections/{projection_id}` to fetch projection output and diagnostics.
- Use `POST /v1/workflow-studio/collab/sessions/{session_id}/conflicts/{conflict_id}/resolve` to record explicit conflict resolution.
- Use `POST /v1/workflow-studio/collab/sessions/{session_id}/publish` to request publication to workflow-engine definitions.
- Cedar action `workflow_studio::collab::create_session` requires tenant membership and workflow authoring permission.
- Cedar action `workflow_studio::collab::join` requires tenant match, session access, and draft visibility.
- Cedar action `workflow_studio::collab::sync_update` requires client registration and session write permission.
- Cedar action `workflow_studio::collab::read_snapshot` requires session read permission and data classification grant.
- Cedar action `workflow_studio::collab::write_snapshot` requires trusted service principal or session owner automation.
- Cedar action `workflow_studio::collab::presence_read` requires session participant status.
- Cedar action `workflow_studio::collab::presence_write` requires active session participant status.
- Cedar action `workflow_studio::collab::resolve_conflict` requires edit permission and conflict anchor visibility.
- Cedar action `workflow_studio::spec::project` requires draft read permission and ontology validation grant.
- Cedar action `workflow_studio::spec::publish` requires workflow definition approval permission and clean blocking diagnostics.
- Cedar action `workflow_studio::audit::export_updates` requires auditor scope and tenant-cell match.
- Deny sync updates when the session is frozen for publication review.
- Deny publication when unresolved blocking conflicts remain.
- Deny publication when ontology references target deprecated types without ADR-0257 handshake evidence.
- Deny publication when projected spec bytes do not match the projection hash.
- SLO `collab-crdt-merge-latency` target is p99 under 100 ms for updates below 64 KiB.
- SLO `collab-sync-ack-latency` target is p99 under 250 ms inside a healthy region.
- SLO `collab-crdt-no-silent-loss` target is 1.0 for accepted semantic updates.
- SLO `collab-session-availability` target is 99.9 percent monthly per certified cell.
- SLO `spec-projection-latency` target is p99 under 1 second for documents up to 500 nodes.
- SLO `spec-round-trip-byte-stability` target is 100 percent for unchanged fixtures.
- SLO `awareness-freshness` target is p95 under 20 seconds for online users.
- Emit trace span `workflow_studio.collab.session.create`.
- Emit trace span `workflow_studio.collab.update.accept`.
- Emit trace span `workflow_studio.collab.update.merge`.
- Emit trace span `workflow_studio.collab.snapshot.write`.
- Emit trace span `workflow_studio.collab.snapshot.restore`.
- Emit trace span `workflow_studio.spec.project`.
- Emit trace span `workflow_studio.conflict.detect`.
- Emit metric `workflow_studio_yjs_updates_total` tagged by tenant, cell, session, and update source.
- Emit metric `workflow_studio_yjs_update_bytes_total` tagged by tenant and cell.
- Emit metric `workflow_studio_yjs_snapshot_bytes` tagged by tenant, cell, and schema version.
- Emit metric `workflow_studio_collab_merge_seconds` as a histogram.
- Emit metric `workflow_studio_collab_silent_loss_violations_total` with expected value zero.
- Emit metric `workflow_studio_spec_projection_seconds` as a histogram.
- Emit metric `workflow_studio_semantic_conflicts_total` tagged by conflict type.
- Emit metric `workflow_studio_awareness_peers` tagged by session and cell.
- Dashboard `workflow-studio-collab-health` shows active sessions, websocket reconnects, merge latency, and update bytes.
- Dashboard `workflow-studio-no-silent-loss` shows accepted update counts and loss violation count.
- Dashboard `workflow-studio-spec-projection` shows projection latency, blocking diagnostics, and byte-stability failures.
- Dashboard `workflow-studio-conflict-resolution` shows open conflicts, time-to-resolution, and conflict types.
- Dashboard `workflow-studio-offline-reconnect` shows offline queue uploads, failures, and discarded drafts.

## Verification

- Test `yjs_updates_converge_for_independent_node_moves` applies updates in different orders and expects identical state.
- Test `yjs_updates_converge_for_independent_edge_additions` verifies graph merge convergence.
- Test `offline_queue_reconnect_merges_without_loss` simulates 30 days of local updates within limit.
- Test `offline_queue_limit_blocks_additional_updates` enforces 100 MiB local queue limit.
- Test `snapshot_every_500_updates` verifies snapshot trigger by accepted update count.
- Test `snapshot_every_60_seconds` verifies snapshot trigger by age.
- Test `awareness_state_not_persisted_as_semantic_update` verifies presence is excluded from snapshots.
- Test `awareness_peer_expires_after_45_seconds` verifies stale presence cleanup.
- Test `semantic_conflict_detects_handler_divergence` creates competing handler changes for one node.
- Test `semantic_conflict_detects_edge_to_deleted_node` creates an edge target conflict.
- Test `semantic_conflict_blocks_publication` verifies unresolved blocking conflict denial.
- Test `projection_is_byte_stable_for_unchanged_canvas` projects the same Yjs state twice.
- Test `projection_rejects_unresolved_ontology_type` verifies ADR-0257 validation.
- Test `projection_rejects_missing_policy_tags` verifies publication gating.
- Test `projected_spec_matches_hash` verifies hash integrity.
- Test `workflow_engine_never_receives_raw_yjs_doc` verifies publish API sends canonical spec only.
- Test `sync_update_requires_cedar_grant` denies unauthorized websocket update.
- Test `join_requires_tenant_match` denies cross-tenant session join.
- Test `read_snapshot_redacts_when_classification_denied` verifies data classification control.
- Test `publish_denied_when_session_frozen_for_other_reviewer` verifies publication freeze.
- Test `yjs_update_hash_chain_detects_gap` verifies missing update detection.
- Test `snapshot_restore_replays_tail_updates` verifies snapshot plus later updates restores final state.
- Test `indexeddb_reload_restores_unflushed_updates` verifies browser reload safety.
- Test `cross_tab_coordination_prevents_duplicate_upload` verifies one user with two tabs.
- Test `large_snapshot_moves_to_object_storage` verifies content-addressed storage behavior.
- Metric check `histogram_quantile(0.99, workflow_studio_collab_merge_seconds) < 0.1`.
- Metric check `histogram_quantile(0.99, workflow_studio_spec_projection_seconds) < 1.0`.
- Metric check `workflow_studio_collab_silent_loss_violations_total == 0`.
- Metric check `workflow_studio_semantic_conflicts_total` increments for expected conflict fixtures.
- Metric check `workflow_studio_yjs_snapshot_bytes` stays below warning threshold for normal fixtures.
- Metric check `workflow_studio_awareness_peers` drops stale peers after expiry.
- Dashboard check `workflow-studio-collab-health` shows reconnect and merge latency panels.
- Dashboard check `workflow-studio-no-silent-loss` shows zero violation target and accepted update volume.
- Dashboard check `workflow-studio-spec-projection` shows byte-stability failures by schema version.
- Dashboard check `workflow-studio-conflict-resolution` shows conflict age and resolver principal.
- Dashboard check `workflow-studio-offline-reconnect` shows queued update count and upload failures.
- Browser test edits the same workflow from two sessions, disconnects one session, reconnects, and verifies both edits remain.
- Browser test moves a node while another user edits its handler, then verifies explicit conflict behavior.
- Browser test reloads the page while offline and verifies local draft recovery.
- Browser test publishes a projected spec and verifies workflow-engine receives `workflow_spec.v1.json`.
- Security test attempts websocket sync with a different tenant principal and expects Cedar denial.
- Security test attempts snapshot export without auditor scope and expects Cedar denial.
- Recovery test deletes latest snapshot and verifies restore from previous snapshot plus append-only updates.
- Recovery test corrupts one update and verifies hash-chain alerting.
- Load test runs 100 active sessions with 10 collaborators each and keeps merge p99 under 100 ms.
- Compatibility test opens an older `canvas_doc.v1` fixture after schema migration and verifies projection.

## References

- ADR-0145, Inter Microservice Communication Reform, `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- ADR-0211, In House Tech Stack Policy, `docs/decisions/ADR-0709-general-live-apex.md`.
- ADR-0243, Cedar as Universal Gate, `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0244, Tenant as Universal Scoping Primitive, `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- ADR-0245, Substrate vs Product Layering, `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- ADR-0257, Ontology Object Type Versioning Deprecation Handshake, `docs/decisions/ADR-0709-general-live-apex.md`.
- ADR-0263, Observability Emission Contract, `docs/decisions/ADR-0706-observability-live-apex.md`.
- Yjs documentation, https://docs.yjs.dev/.
- Yjs API documentation, https://docs.yjs.dev/api/shared-types/y.doc.
- y-websocket documentation, https://docs.yjs.dev/ecosystem/connection-provider/y-websocket.
- y-indexeddb documentation, https://docs.yjs.dev/ecosystem/database-provider/y-indexeddb.
- Marc Shapiro, Nuno Preguica, Carlos Baquero, and Marek Zawirski, Conflict-free Replicated Data Types, 2011.
- Martin Kleppmann et al., Local-first software: You own your data, in spite of the cloud, 2019.
- Clarence Ellis and Simon Gibbs, Concurrency Control in Groupware Systems, ACM SIGMOD 1989.
- RFC 6455, The WebSocket Protocol, https://www.rfc-editor.org/rfc/rfc6455.
- RFC 6902, JavaScript Object Notation Patch, https://www.rfc-editor.org/rfc/rfc6902.
