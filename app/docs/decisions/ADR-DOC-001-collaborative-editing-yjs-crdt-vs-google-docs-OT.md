---
id: ADR-DOC-001
title: Collaborative Editing Yjs CRDT versus Google Docs Operational Transform
status: Proposed
date: 2026-05-20
microservice: docs
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-docs
---

# ADR-DOC-001: Collaborative Editing Yjs CRDT versus Google Docs Operational Transform

## Context

- Docs owns document store, block types, collaborative editing, comments, suggestions, version history, sharing, import/export, embeds, and AI assist boundaries.
- Existing ADR-DOCS-0001 selected Loro for the accepted minimum docs CRDT library.
- This ADR records the broader product decision between Yjs-style CRDT collaboration and Google Docs-style operational transform.
- Named pressure DOC-P1: collaborative docs must support offline-first editing without silent loss.
- Named pressure DOC-P2: browser clients, mobile clients, and future desktop clients need compatible merge semantics.
- Named pressure DOC-P3: block trees, comments, suggestions, and embeds evolve faster than plain text operations.
- Named pressure DOC-P4: tenants need audit evidence for accepted operations, rejected operations, and conflict surfacing.
- Named pressure DOC-P5: users expect Google Docs-grade collaboration but Oyatie cannot depend on a central OT server for correctness.
- Named precedent: Google Docs popularized server-mediated operational transform for real-time collaborative documents.
- Named precedent: Yjs popularized production JavaScript CRDT collaboration with offline update exchange.
- Named precedent: Automerge and Loro show state-based and op-based CRDT approaches for local-first software.
- Constraint DOC-C1: tenant and document scope come from ADR-0244.
- Constraint DOC-C2: accepted ops, rejected ops, compaction, snapshot, and divergence events emit evidence per ADR-0263.
- Constraint DOC-C3: Cedar gates document read, edit, comment, suggest, share, export, and AI assist per ADR-0243.
- Constraint DOC-C4: collaboration wire contracts follow ADR-0258.
- Constraint DOC-C5: offline-first means a client can accept local edits while temporarily disconnected.
- Constraint DOC-C6: no server component may silently drop an accepted user edit.
- Constraint DOC-C7: conflicts must surface or merge deterministically.
- Constraint DOC-C8: CRDT library choice must be hidden behind service-local port traits.
- Constraint DOC-C9: existing Loro accepted ADR remains the current implementation authority until explicitly superseded.
- Constraint DOC-C10: if Yjs is adopted, migration must preserve document history and audit evidence.
- OT can provide excellent live collaboration when a central server sees every op in order.
- OT becomes expensive when operation types grow and offline branches widen.
- Yjs CRDT gives mature browser ecosystem and update exchange semantics.
- This ADR is Proposed because it would supersede or amend existing Loro implementation authority only after migration review.

## Decision

- Choose CRDT over Google Docs-style operational transform as the docs collaboration family.
- Prefer a Yjs-compatible CRDT protocol for the product-level collaboration contract.
- Keep the concrete library hidden behind `DocsCollabEngine` port traits.
- Treat existing Loro implementation as a compatible current adapter until a migration ADR supersedes it.
- Do not build a bespoke Google Docs-style OT transform matrix.
- Keep offline-first as a non-negotiable invariant.
- Accept local edits while offline and sync them as signed update envelopes later.
- Represent document collaboration updates as immutable `DocUpdateEnvelope` records.
- Bind every update to tenant id, document id, actor id, device id, base snapshot, and permission epoch.
- Use vector-clock or state-vector style sync metadata.
- Use awareness updates for cursors and presence with shorter retention than document updates.
- Store snapshots separately from update logs.
- Compact update logs only after deterministic projection passes.
- Keep comments and suggestions as structured side-channel objects linked to document ranges.
- Keep range anchors resilient to concurrent edits.
- Use Cedar at sync admission time and export time.
- Reject updates from revoked actors even if the CRDT merge would be mathematically valid.
- Keep unauthorized local updates client-local until user resolves permission state.
- Surface conflicts through explicit conflict UI when semantic merge cannot preserve intent.
- Maintain reference document corpus for projection stability.
- Support Yjs provider interoperability only through a controlled gateway, not arbitrary public provider connections.
- Keep server authoritative for persistence, policy, audit, and snapshot publication.
- Keep clients authoritative for offline local editing until sync.
- Keep document canonical projection as block-tree JSON.
- Keep CRDT internal state out of public REST contracts.
- Expose collaboration through WebSocket and event envelopes.
- Keep end-to-end encrypted document mode as a future compatibility requirement.
- Make no public promise that internal updates are raw Yjs binary format until security review accepts it.
- Name the contract `DocsOfflineCrdtCollab v1`.
- Name OT rejection reason `transform-matrix-explosion-and-offline-centrality`.

## Alternatives Considered

### Google Docs-Style Operational Transform

- Pros: proven by Google Docs at massive scale.
- Pros: central server can enforce ordering and transformation.
- Pros: efficient for plain-text collaborative editing.
- Cons: every new operation type needs transform logic against every other operation type.
- Cons: offline edits require complex rebasing when branches diverge.
- Cons: central server ordering becomes correctness-critical.
- Rejected because docs needs offline-first block-tree collaboration with evolving operation types.

### Yjs CRDT as Direct Implementation

- Pros: mature browser ecosystem.
- Pros: strong offline update exchange model.
- Pros: rich provider and editor integrations.
- Cons: Rust server-side integration is less native than JavaScript.
- Cons: binary update format needs careful audit and replay tooling.
- Cons: existing accepted implementation chose Loro.
- Accepted at product-contract level, pending separate migration and adapter proof.

### Loro CRDT as Current Implementation

- Pros: existing accepted service ADR selected it.
- Pros: Rust and WASM fit the repo implementation posture.
- Pros: cross-service alignment with workflow-studio exists.
- Cons: smaller ecosystem than Yjs.
- Cons: external editor integrations often expect Yjs.
- Cons: this ADR's topic specifically compares Yjs CRDT with OT.
- Retained as current adapter until a supersession ADR changes implementation.

### Automerge CRDT

- Pros: strong local-first research lineage.
- Pros: JSON-shaped document support.
- Pros: clear change history model.
- Cons: bundle and storage costs can be higher.
- Cons: fewer production editor integrations than Yjs.
- Cons: not the requested interoperability target.
- Rejected as default but useful as a reference in property testing.

### Server-Locked Single Writer

- Pros: simplest correctness model.
- Pros: no transform or CRDT merge complexity.
- Pros: easy audit ordering.
- Cons: not collaborative editing.
- Cons: offline editing becomes read-only.
- Cons: user experience is unacceptable.
- Rejected because it fails the product promise.

## Consequences

- Positive: offline edits can sync without central transform ordering.
- Positive: collaborative semantics can evolve with block operations.
- Positive: Yjs ecosystem gives editor integration leverage.
- Positive: CRDT updates can be exchanged across client platforms.
- Positive: server can remain policy and audit authority without being merge origin for every keystroke.
- Positive: no silent loss invariant is naturally testable through CRDT convergence properties.
- Positive: OT transform-matrix complexity is avoided.
- Positive: a gateway can interoperate with Yjs tooling where approved.
- Negative: CRDT update logs and compaction need disciplined storage management.
- Negative: semantic conflicts still need UX design.
- Negative: Yjs binary updates require security review before direct exposure.
- Negative: existing Loro implementation creates a migration question.
- Negative: CRDT convergence bugs can be subtle and require property testing.
- Neutral: this ADR is Proposed and does not supersede ADR-DOCS-0001 by itself.
- Neutral: public document API remains canonical block-tree JSON.
- Neutral: WebSocket remains the realtime transport.
- Neutral: comments and suggestions remain structured side channels.
- Neutral: Google Docs remains a precedent for UX quality, not implementation topology.

## Implementation Notes

- Data shape `DocUpdateEnvelope`: `{tenant_id, document_id, update_id, actor_id, device_id, engine_family, update_bytes_ref, state_vector_ref, permission_epoch, signed_at}`.
- Data shape `DocSnapshot`: `{tenant_id, document_id, snapshot_id, engine_family, canonical_block_tree_hash, update_high_watermark, created_at}`.
- Data shape `DocAwarenessState`: `{document_id, actor_id, device_id, cursor_anchor, selection_anchor, expires_at}`.
- Data shape `DocConflict`: `{document_id, conflict_id, range_anchor, update_ids, conflict_kind, surfaced_at}`.
- Data shape `DocsCollabEngine`: `{engine_family, engine_version, adapter_crate, projection_version, migration_state}`.
- Data shape `CollabAdmissionDecision`: `{update_id, permit_id, effect, policy_hash, permission_epoch, reason}`.
- Data shape `UpdateCompactionJob`: `{document_id, job_id, from_update_id, to_snapshot_id, state, divergence_count}`.
- Canonical document projection remains block-tree JSON.
- Internal CRDT update bytes are stored by reference, not in audit events.
- WebSocket endpoint `/v1/docs/collab/{document_id}` accepts signed update envelopes.
- REST endpoint `GET /v1/docs/{document_id}/snapshots/{snapshot_id}` returns canonical projection.
- REST endpoint `POST /v1/docs/{document_id}/collab/compact` starts compaction.
- REST endpoint `GET /v1/docs/{document_id}/collab/state-vector` returns sync metadata.
- REST endpoint `POST /v1/docs/{document_id}/collab/replay` recomputes projection for audit.
- REST endpoint `POST /v1/docs/{document_id}/collab/migrate-engine` is future migration-only.
- AsyncAPI channel `docs.collab.update.accepted.v1` publishes accepted updates.
- AsyncAPI channel `docs.collab.update.rejected.v1` publishes policy or parse rejection.
- AsyncAPI channel `docs.collab.snapshot.created.v1` publishes snapshot creation.
- AsyncAPI channel `docs.collab.divergence.detected.v1` publishes projection mismatch.
- AsyncAPI channel `docs.collab.conflict.surfaced.v1` publishes semantic conflicts.
- Cedar permit `docs::collab::sync_update` requires edit permission and current permission epoch.
- Cedar permit `docs::collab::read_snapshot` requires document read permission.
- Cedar permit `docs::collab::compact` requires service identity or maintainer role.
- Cedar forbid `docs::collab::sync_update` when actor has been removed from document ACL.
- Cedar forbid `docs::collab::export_update_bytes` unless security-reviewed gateway is enabled.
- Audit event `EVT-DOCS-COLLAB-UPDATE-ACCEPTED` includes update hash, actor, device, and permission epoch.
- Audit event `EVT-DOCS-COLLAB-UPDATE-REJECTED` includes reason and policy id.
- Audit event `EVT-DOCS-COLLAB-SNAPSHOT-CREATED` includes snapshot hash and high watermark.
- Audit event `EVT-DOCS-COLLAB-DIVERGENCE-DETECTED` includes replay verdict.
- Metric `docs_collab_update_accept_latency_ms` tracks sync admission.
- Metric `docs_collab_update_reject_total` tracks rejection reason.
- Metric `docs_collab_snapshot_compaction_seconds` tracks compaction.
- Metric `docs_collab_divergence_total` tracks projection failures.
- Metric `docs_collab_awareness_active_clients` tracks presence cardinality.
- Trace span `docs.collab.sync_update` records engine family and permission epoch.
- Trace span `docs.collab.project_snapshot` records update count and projection version.
- Trace span `docs.collab.compact` records input update count and snapshot hash.
- Log schema `DocsCollabDecisionLog` includes document hash, engine family, update hash, and verdict.
- SLO target: update admission p99 <= 100 ms.
- SLO target: awareness fanout p99 <= 150 ms.
- SLO target: no-silent-loss divergence count equals zero.
- SLO target: compaction completes within 5 minutes for documents with 100k updates.
- SLO target: offline sync of 10k updates completes p95 <= 30 seconds on broadband.
- Capacity math: 1 million active documents with average 500 retained updates each yields 500 million update records before compaction, so snapshot cadence is mandatory.
- Capacity math: if average update is 400 bytes and a document receives 100k updates, raw update log is about 40 MiB before compression.
- Capacity math: awareness states expire quickly and should stay in hot storage only.
- Rollback path: disable new engine migrations and keep current adapter.
- Rollback path: reject direct Yjs gateway export while preserving canonical snapshots.
- Rollback path: replay from last trusted snapshot and accepted update envelope log.
- Multi-region path: sync writes happen in document home cell; remote cells subscribe to snapshots.
- Sovereign-cell path: update bytes, snapshots, and awareness logs remain in approved cells.
- Versioning: `DocsOfflineCrdtCollab v1` is additive by envelope field.
- Deprecation: engine family migration requires a new ADR and 365-day read support.

## Verification

- Unit test `sync_update_requires_current_permission_epoch` checks Cedar gate.
- Unit test `revoked_actor_update_rejected_even_if_crdt_valid` checks policy over math.
- Unit test `awareness_state_expires_without_snapshot_mutation` checks presence separation.
- Unit test `ot_transform_matrix_not_present_in_engine_ports` checks design boundary.
- Unit test `engine_types_do_not_leak_to_public_rest_contract` checks abstraction.
- Property test `offline_updates_converge_without_silent_loss` checks CRDT invariant.
- Property test `projection_from_update_log_matches_snapshot` checks replay.
- Property test `compaction_preserves_canonical_block_tree_hash` checks snapshots.
- Fuzz test `collab_update_parser_rejects_malformed_binary` checks security.
- Integration test `two_clients_edit_offline_then_sync` checks offline-first UX.
- Integration test `removed_collaborator_local_update_rejected_on_sync` checks ACL.
- Integration test `comments_anchor_survives_concurrent_text_edit` checks semantic side channel.
- Integration test `gateway_interop_disabled_by_default` checks controlled Yjs path.
- Load test `ten_k_offline_updates_sync_under_budget` validates sync SLO.
- Load test `hundred_k_update_compaction_under_budget` validates compaction.
- Chaos test `snapshot_worker_crash_replays_update_log` checks recovery.
- Chaos test `policy_bundle_drift_rejects_stale_permission_epoch` checks safety.
- Metric check: dashboard `docs/collab-health` adds engine family and divergence panels.
- Metric check: dashboard `docs/editor-experience` adds update admission latency.
- Alert check: `docs_collab_divergence_total` above zero pages immediately.
- Audit check: every rejected update has `EVT-DOCS-COLLAB-UPDATE-REJECTED`.
- Static check: no public REST response returns raw update bytes by default.
- Contract check: AsyncAPI documents update accepted and rejected channels.
- Regression check: ADR-DOCS-0001 remains current implementation until superseded.

## References

- Yjs documentation.
- y-protocols awareness documentation.
- Google Wave operational transform papers.
- Google Docs collaboration architecture public talks and papers.
- Shapiro et al., Conflict-free Replicated Data Types.
- Kleppmann et al., local-first software references.
- Automerge documentation.
- Loro documentation.
- ADR-DOCS-0001 CRDT library selection.
- ADR-DOCS-0002 block type system.
- ADR-DOCS-0004 ACL granularity per block.
- ADR-0243 Cedar-as-universal-gate.
- ADR-0263 observability-emission-contract.
- microservices/docs/PRD.md.
- microservices/docs/dashboards/collab-health.json.
