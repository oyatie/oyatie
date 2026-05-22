# IP-006 Whiteboard Async Event Surface

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-006-async-event-surface.md
Planning lane: B2B-leader IP substance deepening pass
Primary concern: AsyncAPI, outbox, replay, and worker event surface for whiteboard collaboration and migration
Contract references: microservices/whiteboard/contracts/asyncapi-v1.yaml; microservices/whiteboard/contracts/local-asyncapi-v1.yaml; microservices/whiteboard/contracts/whiteboard-v1.proto; microservices/whiteboard/contracts/local-operations-v1.proto
Runbook references: microservices/whiteboard/runbooks/local-stroke-persistence-lag.md; microservices/whiteboard/runbooks/local-crdt-merge-conflict.md; microservices/whiteboard/runbooks/local-presence-stale.md; microservices/whiteboard/runbooks/local-regional-board-replay.md; microservices/whiteboard/runbooks/export-render-failure.md; microservices/whiteboard/runbooks/template-import-rollback.md; microservices/whiteboard/runbooks/board-history-corruption.md
SLO references: microservices/whiteboard/slos/local-stroke-persistence-latency.openslo.yaml; microservices/whiteboard/slos/local-crdt-merge-success.openslo.yaml; microservices/whiteboard/slos/local-presence-freshness.openslo.yaml; microservices/whiteboard/slos/replay-freshness.openslo.yaml; microservices/whiteboard/slos/audit-emission-lag.openslo.yaml
Capability references: whiteboard-board-open; whiteboard-canvas-op-append; whiteboard-presence-sync; whiteboard-history-snapshot; whiteboard-export-render; whiteboard-template-marketplace-install
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Benchmark displacement set: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard

## Executive Intent
- The async event surface is the durable collaboration spine for whiteboard because canvas operations, presence updates, snapshots, exports, imports, and replay repairs cannot all be synchronous REST calls.
- This IP displaces Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard by making their real-time and background behavior observable in Oyatie-owned events.
- The event surface must preserve BoardScope from IP-001 and Cedar decisions from IP-002.
- The event surface must preserve ontology projection ids from IP-003.
- The event surface must preserve workflow template ids from IP-004.
- The event surface must remain reachable from REST command responses from IP-005.
- ADR-0321 requires event parity for B2B leader collaboration, not only static API parity.
- ADR-0296 requires worker evidence to survive async boundaries.
- ADR-0297 requires replay-safe event material.
- ADR-0314 requires marketplace template events to include DealSet evidence.
- ADR-0105 requires event contracts and workers to stay in their own layers.
- ADR-0253-amendment means event publication must retain trace and identity context across transport upgrades.

## Event Families
- board.session.requested is emitted after REST accepts a session start or join command.
- board.session.opened is emitted when the session becomes active.
- board.session.closed is emitted when the session terminal state is reached.
- canvas.operation.accepted is emitted after policy allow and idempotency reservation.
- canvas.operation.persisted is emitted after the operation log write succeeds.
- canvas.operation.rejected is emitted when scope, policy, idempotency, or payload validation fails.
- canvas.crdt.merge.requested is emitted when a materialized board view needs merge.
- canvas.crdt.merge.completed is emitted when CRDT merge succeeds.
- canvas.crdt.merge.failed is emitted when conflict or corruption requires operator route.
- presence.sync.requested is emitted for cursor or presence updates.
- presence.sync.broadcasted is emitted after authorized fanout.
- presence.sync.throttled is emitted when rate controls apply.
- history.snapshot.requested is emitted when snapshot workflow starts.
- history.snapshot.materialized is emitted when snapshot content is complete.
- export.render.requested is emitted after export policy allow.
- export.render.completed is emitted when artifact metadata is ready.
- export.render.failed is emitted when rendering or residency fails.
- template.install.requested is emitted after template install acceptance.
- template.install.completed is emitted after objects materialize.
- template.install.failed is emitted when DealSet, projection, or import fails.
- vendor.import.requested is emitted for benchmark migration.
- vendor.import.mapped is emitted after source object mapping.
- vendor.import.completed is emitted after import terminal success.
- board.replay.requested is emitted for repair or historical rebuild.
- board.replay.completed is emitted after replay success.
- board.replay.failed is emitted when scope, hash, version, or operation history blocks replay.
- audit.evidence.emitted is emitted when audit chain writes complete.

## Canvas And CRDT Domain Model
- CanvasOperationEvent is append-only.
- CanvasOperationEvent includes operation_id.
- CanvasOperationEvent includes operation_sequence.
- CanvasOperationEvent includes board_id.
- CanvasOperationEvent includes session_id when live.
- CanvasOperationEvent includes tenant_id.
- CanvasOperationEvent includes principal_id.
- CanvasOperationEvent includes data_class=canvas_operation.
- CanvasOperationEvent includes operation_kind.
- CanvasOperationEvent includes object_id.
- CanvasOperationEvent includes parent_object_id when frame or group membership changes.
- CanvasOperationEvent includes connector_endpoint_ids when connector topology changes.
- CanvasOperationEvent includes idempotency_key.
- CanvasOperationEvent includes scope_hash.
- CanvasOperationEvent includes policy_decision_ref.
- CanvasOperationEvent includes ontology_projection_ref when materialized.
- CanvasOperationEvent includes crdt_clock.
- CanvasOperationEvent includes previous_operation_id when causality is known.
- CanvasOperationEvent includes payload_digest.
- CanvasOperationEvent includes payload_size_bytes.
- CanvasOperationEvent excludes raw payload in audit-safe topics.
- CRDTMergeEvent includes merge_id.
- CRDTMergeEvent includes board_id.
- CRDTMergeEvent includes operation_range_start.
- CRDTMergeEvent includes operation_range_end.
- CRDTMergeEvent includes merge_strategy.
- CRDTMergeEvent includes conflict_count.
- CRDTMergeEvent includes conflict_resolution_policy.
- CRDTMergeEvent includes materialized_view_version.
- CRDTMergeEvent includes replay_cursor.
- CRDTMergeEvent includes local-crdt-merge-success SLO result.

## Session And Presence Domain Model
- BoardSessionEvent includes session_id.
- BoardSessionEvent includes board_id.
- BoardSessionEvent includes tenant_id.
- BoardSessionEvent includes tenant_home_cell.
- BoardSessionEvent includes request_cell.
- BoardSessionEvent includes facilitator_principal_ids.
- BoardSessionEvent includes participant_count.
- BoardSessionEvent includes audience_type.
- BoardSessionEvent includes workflow_template_id when session was template-backed.
- BoardSessionEvent includes meeting_binding for Microsoft Whiteboard replacement flows.
- BoardSessionEvent includes roster_binding for Whiteboard.fi replacement flows.
- BoardSessionEvent includes source_vendor when imported session semantics are present.
- BoardSessionEvent includes policy_pack_set.
- BoardSessionEvent includes session_state.
- BoardSessionEvent includes opened_at.
- BoardSessionEvent includes closed_at when terminal.
- PresenceEvent includes presence_id.
- PresenceEvent includes session_id.
- PresenceEvent includes board_id.
- PresenceEvent includes principal_id.
- PresenceEvent includes data_class=presence_cursor.
- PresenceEvent includes cursor_region, not raw pixel trail, when audit-safe topic is used.
- PresenceEvent includes rate_limit_bucket.
- PresenceEvent includes freshness_deadline.
- PresenceEvent includes fanout_count.
- PresenceEvent includes local-presence-freshness SLO result.
- PresenceEvent includes throttled_reason when denied or delayed.

## Command And Event Delta
- REST POST /boards maps to board.session.requested only when the request creates or opens a session.
- REST POST /boards/{board_id}/sessions maps to board.session.requested.
- REST POST /boards/{board_id}/operations maps to canvas.operation.accepted or canvas.operation.rejected.
- REST POST /boards/{board_id}/snapshots maps to history.snapshot.requested.
- REST POST /boards/{board_id}/exports maps to export.render.requested.
- REST POST /boards/{board_id}/templates maps to template.install.requested.
- REST POST /imports maps to vendor.import.requested.
- REST POST /replays maps to board.replay.requested.
- Async canvas.operation.persisted may trigger canvas.crdt.merge.requested.
- Async canvas.crdt.merge.completed may trigger ontology projection updates.
- Async history.snapshot.materialized may trigger export.render.requested when a workflow chains snapshot to export.
- Async template.install.completed may trigger canvas.crdt.merge.requested for materialized objects.
- Async vendor.import.mapped may trigger template.install.requested when source material is template-like.
- Async board.replay.completed may trigger history.snapshot.materialized when repair requests proof snapshot.
- Async audit.evidence.emitted closes evidence gaps for dashboards.
- Every accepted command event includes command_id.
- Every worker-produced event includes worker_id.
- Every replay-produced event includes replay_id.
- Every terminal event includes terminal_state.
- Every failure event includes runbook_ref.
- Every event includes schema_version.

## Proto Delta
- whiteboard-v1.proto must define BoardSessionEvent message or equivalent event envelope.
- whiteboard-v1.proto must define CanvasOperationEvent message or equivalent event envelope.
- whiteboard-v1.proto must define PresenceEvent message or equivalent event envelope.
- whiteboard-v1.proto must define HistorySnapshotEvent message or equivalent event envelope.
- whiteboard-v1.proto must define ExportRenderEvent message or equivalent event envelope.
- whiteboard-v1.proto must define TemplateInstallEvent message or equivalent event envelope.
- whiteboard-v1.proto must define VendorImportEvent message or equivalent event envelope.
- whiteboard-v1.proto must define BoardReplayEvent message or equivalent event envelope.
- local-operations-v1.proto must define worker command envelopes for merge, snapshot, export, template install, import, and replay.
- Proto envelopes must include tenant_id.
- Proto envelopes must include board_id when board scoped.
- Proto envelopes must include session_id when session scoped.
- Proto envelopes must include scope_hash.
- Proto envelopes must include scope_version.
- Proto envelopes must include policy_decision_ref.
- Proto envelopes must include audit_chain_target.
- Proto envelopes must include workflow_instance_id when workflow-backed.
- Proto envelopes must include source_vendor when import or displacement context exists.
- Proto envelopes must include source_object_id when import context exists.
- Proto envelopes must include marketplace_dealset_id for template material.
- Proto envelopes must not include raw vendor tokens.

## Cedar Facts In Events
- Events must carry policy_decision_ref, not raw Cedar internals.
- Events must carry policy_version.
- Events must carry evaluation_trace_id.
- Events must carry result allow, deny, bypass-approved, or not-applicable.
- Events must carry capability name.
- Events must carry action name.
- Events must carry data_class.
- Events must carry principal_id or worker principal.
- Events must carry tenant_id.
- Events must carry board_id when policy was board-scoped.
- Events must carry session_id when policy was session-scoped.
- Events must carry source_vendor for benchmark migrations.
- Events must carry marketplace_dealset_id for marketplace template operations.
- Events must carry residency_zone for export and snapshot operations.
- Events must carry retention_class for snapshot and export operations.
- Events must carry runbook_ref when policy denial leads to operator work.
- Events must never recalculate policy in a worker without recording current policy_version.
- Events must distinguish historical replay decision reuse from current-policy re-evaluation.
- Events must preserve precheck deny reasons from IP-002.
- Events must be queryable by dashboards without reading raw payloads.
- Events must be suitable for audit-chain emission within audit-emission-lag SLO.

## Workflow Decisions
- Workflow templates from IP-004 own event choreography.
- The outbox owns exactly-once publication attempts, not exactly-once external side effects.
- Idempotency owns duplicate command responses.
- Operation sequence owns canvas replay order.
- CRDT clock owns merge conflict detection.
- Scope hash owns tenant and policy input continuity.
- Policy decision refs own authorization continuity.
- Ontology projection refs own graph continuity.
- Audit event refs own compliance continuity.
- Workflow instance id owns business-process continuity.
- Worker id owns operational accountability.
- Replay id owns repair accountability.
- Runbook ref owns operator routing.
- Source vendor owns benchmark displacement provenance.
- DealSet id owns marketplace settlement provenance.
- Residency zone owns export and snapshot routing constraints.
- Retention class owns snapshot and artifact lifecycle constraints.
- Event schema version owns contract evolution.
- AsyncAPI channel version owns client subscription compatibility.
- Proto message version owns internal worker compatibility.

## Failure And Replay Cases
- If outbox insert fails, command must fail before client acceptance.
- If outbox publish fails after acceptance, retry through outbox worker.
- If canvas operation persist fails, emit canvas.operation.rejected or canvas.operation.failed according to acceptance stage.
- If CRDT merge conflicts, emit canvas.crdt.merge.failed and route to local-crdt-merge-conflict.
- If presence fanout lags, emit presence.sync.throttled or presence.sync.failed and route to local-presence-stale.
- If stroke persistence lags, emit operation lag metrics and route to local-stroke-persistence-lag.
- If board replay sees scope_hash mismatch, emit board.replay.failed and route to local-regional-board-replay.
- If replay sees unsupported scope_version, emit board.replay.failed with scope_version_unsupported.
- If replay sees missing operation range, emit board.replay.failed and route to board-history-corruption.
- If snapshot materialization fails, emit history.snapshot.failed and preserve requested range.
- If export render fails, emit export.render.failed and route to export-render-failure.
- If export egress policy changes during render, emit export.render.failed with policy_version_changed.
- If template import fails, emit template.install.failed and route to template-import-rollback.
- If DealSet settlement is revoked during install, emit template.install.failed with dealset_revoked.
- If vendor import source mapping fails, emit vendor.import.failed with source_mapping_failed.
- If Miro Enterprise import drops frame hierarchy, emit vendor.import.failed with frame_hierarchy_loss.
- If Mural Enterprise import drops facilitator controls, emit vendor.import.failed with facilitator_loss.
- If FigJam import drops widget provenance, emit vendor.import.failed with widget_provenance_loss.
- If Lucidspark import drops connector endpoints, emit vendor.import.failed with connector_endpoint_loss.
- If Whiteboard.fi import drops roster binding, emit vendor.import.failed with roster_binding_loss.
- If Microsoft Whiteboard import drops meeting binding, emit vendor.import.failed with meeting_binding_loss.
- Replay must be able to rebuild materialized board state from canvas.operation.persisted.
- Replay must be able to rebuild ontology projection refs from projection events.
- Replay must be able to rebuild audit completeness from audit.evidence.emitted.
- Replay must not rebuild denied or rejected operations as accepted.

## Evidence Fields
- event_id is required.
- event_type is required.
- schema_version is required.
- occurred_at is required.
- tenant_id is required.
- tenant_home_cell is required.
- request_cell is required when applicable.
- board_id is required for board events.
- session_id is required for session and presence events.
- operation_id is required for canvas operation events.
- workflow_instance_id is required for workflow-backed events.
- workflow_template_id is required for template-backed events.
- command_id is required for command-derived events.
- worker_id is required for worker-derived events.
- replay_id is required for replay-derived events.
- scope_hash is required.
- scope_version is required.
- policy_decision_ref is required when policy applies.
- audit_event_ref is required for accepted and terminal events.
- ontology_projection_ref is required when projection is emitted.
- source_vendor is required for benchmark migration events.
- source_object_id is required for benchmark migration events.
- marketplace_dealset_id is required for template marketplace events.
- residency_zone is required for snapshot and export events.
- retention_class is required for snapshot and export events.
- runbook_ref is required for failure events.
- slo_result is required for latency-sensitive terminal events.
- payload_digest is required for payload-bearing events.
- payload_size_bytes is required for payload-bearing events.

## SLO Mapping
- board.session.opened contributes to local-board-load-time.
- canvas.operation.persisted contributes to local-stroke-persistence-latency.
- canvas.crdt.merge.completed contributes to local-crdt-merge-success.
- presence.sync.broadcasted contributes to local-presence-freshness.
- history.snapshot.materialized contributes to replay-freshness when replay-triggered.
- export.render.completed contributes to local-export-render-latency.
- audit.evidence.emitted contributes to audit-emission-lag.
- vendor.import.completed contributes to backfill and replay freshness dashboards.
- template.install.completed contributes to workflow template completion SLO.
- board.replay.completed contributes to replay-freshness.
- Failure events must include whether the SLO was burned.
- Retry events must include next_attempt_at.
- Dead-letter events must include dead_letter_reason.
- Throttle events must include throttled_duration_ms.
- Merge events must include conflict_count.
- Presence events must include freshness_deadline.
- Export events must include render_duration_ms.
- Snapshot events must include operation_count.
- Import events must include source_object_count when known.
- Audit events must include emission_lag_ms.

## Benchmark Displacement Events
- Miro Enterprise migration emits vendor.import.requested with source_vendor=Miro Enterprise.
- Miro Enterprise frame mapping emits vendor.import.mapped with mapped_object_kind=Frame.
- Miro Enterprise sticky mapping emits vendor.import.mapped with mapped_object_kind=StickyNote.
- Mural Enterprise migration emits vendor.import.requested with source_vendor=Mural Enterprise.
- Mural Enterprise facilitator mapping emits board.session.opened with facilitator evidence.
- Mural Enterprise room mapping emits vendor.import.mapped with room_provenance.
- FigJam migration emits vendor.import.requested with source_vendor=FigJam.
- FigJam widget mapping emits template.install.requested or vendor.import.mapped with widget provenance.
- FigJam cursor behavior emits presence.sync.broadcasted with freshness evidence.
- Lucidspark migration emits vendor.import.requested with source_vendor=Lucidspark.
- Lucidspark connector mapping emits canvas.crdt.merge.completed with connector endpoint counts.
- Lucidspark export emits export.render.completed with diagram export evidence.
- Whiteboard.fi migration emits vendor.import.requested with source_vendor=Whiteboard.fi.
- Whiteboard.fi classroom setup emits board.session.opened with roster_binding evidence.
- Whiteboard.fi student board export emits export.render.completed with education retention evidence.
- Microsoft Whiteboard migration emits vendor.import.requested with source_vendor=Microsoft Whiteboard.
- Microsoft Whiteboard meeting board emits board.session.opened with meeting_binding evidence.
- Microsoft Whiteboard tenant export emits export.render.requested with explicit board_id list.
- All benchmark events preserve source_object_id.
- All benchmark events avoid vendor workspace, room, file, class, or meeting as tenant authority.

## Implementation Steps
- Update asyncapi-v1.yaml with event families from this IP.
- Update local-asyncapi-v1.yaml with worker-only and replay-only channels.
- Update whiteboard-v1.proto with event envelopes.
- Update local-operations-v1.proto with worker command envelopes.
- Add outbox table or repository binding for accepted command events.
- Add publisher worker for outbox events.
- Add canvas operation persistence worker.
- Add CRDT merge worker.
- Add presence fanout worker.
- Add snapshot worker.
- Add export render worker.
- Add template install worker.
- Add vendor import worker.
- Add replay worker.
- Add audit evidence worker or binding.
- Add dead-letter handling with runbook_ref.
- Add event schema version constants.
- Add benchmark source_vendor enum using displaced names.
- Add payload digest calculation.
- Add scope_hash verification in every worker.
- Add policy decision ref verification in every worker.
- Add idempotency verification for command-derived events.
- Add replay mode handling for historical and current-policy replays.

## Test Plan
- AsyncAPI contract test validates all event names.
- AsyncAPI contract test validates required common envelope fields.
- AsyncAPI contract test validates benchmark source_vendor enum.
- Proto contract test validates BoardSessionEvent fields.
- Proto contract test validates CanvasOperationEvent fields.
- Proto contract test validates PresenceEvent fields.
- Proto contract test validates ExportRenderEvent fields.
- Proto contract test validates VendorImportEvent fields.
- Worker unit test rejects missing scope_hash.
- Worker unit test rejects unsupported scope_version.
- Worker unit test rejects missing policy_decision_ref when policy applies.
- Worker unit test rejects missing source_object_id on benchmark import.
- Worker unit test rejects missing DealSet id on marketplace template event.
- Replay test rebuilds board from canvas.operation.persisted.
- Replay test skips canvas.operation.rejected.
- Replay test fails on scope_hash mismatch.
- Replay test preserves Miro Enterprise source provenance.
- Replay test preserves Mural Enterprise facilitator provenance.
- Replay test preserves FigJam widget provenance.
- Replay test preserves Lucidspark connector endpoints.
- Replay test preserves Whiteboard.fi roster binding.
- Replay test preserves Microsoft Whiteboard meeting binding.
- CRDT test merge conflict emits canvas.crdt.merge.failed.
- Presence test stale cursor emits presence.sync.throttled.
- Export test residency conflict emits export.render.failed.
- Template test DealSet revocation emits template.install.failed.
- Audit test accepted events produce audit.evidence.emitted.
- SLO test operation persistence emits local-stroke-persistence result.
- SLO test presence broadcast emits local-presence-freshness result.
- Dead-letter test failure events include runbook_ref.
- Dashboard test metrics include source_vendor and event_type.

## Acceptance Criteria
- AsyncAPI defines every event family named in this IP.
- Local AsyncAPI defines worker and replay channels.
- Proto contracts carry event envelopes with scope, policy, audit, and source provenance fields.
- Every worker verifies scope_hash before side effects.
- Every worker preserves policy_decision_ref.
- Every event includes schema_version.
- Every failure event includes runbook_ref.
- Every benchmark event uses Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, or Microsoft Whiteboard as applicable.
- Every marketplace template event includes DealSet evidence.
- Every export and snapshot event includes residency and retention evidence.
- Every canvas operation event includes CRDT and idempotency fields.
- Every session and presence event includes session domain fields.
- Every replay path can distinguish accepted, rejected, failed, and terminal events.
- Every SLO-sensitive event emits SLO result fields.
- No event treats vendor workspace, room, file, class, or meeting as tenant authority.
- No event surface requires editing ADR-0321.

## Rollback
- Roll back consumers before producers when changing event schemas.
- Keep schema_version compatibility for already published events.
- Keep dead-letter topics available during rollback.
- Keep source_vendor and source_object_id in benchmark events once published.
- Keep scope_hash and policy_decision_ref in all event schemas.
- Keep DealSet fields in marketplace template events.
- Keep residency and retention fields in export and snapshot events.
- Disable benchmark import channels per source_vendor rather than deleting shared event types.
- Disable worker execution through workflow gating rather than dropping accepted outbox rows.
- Route stuck outbox rows to local-stroke-persistence-lag or export-render-failure based on event type.
- Route replay rollback issues to local-regional-board-replay.
- Route template rollback issues to template-import-rollback.
- Treat incompatible event schema removal as a contract-breaking change requiring ADR-level approval.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
