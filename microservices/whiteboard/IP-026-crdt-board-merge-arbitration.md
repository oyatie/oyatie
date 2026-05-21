# IP-026 Whiteboard CRDT board merge arbitration

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-026-crdt-board-merge-arbitration.md
Capability focus: canvas-op-append, presence-sync, history-snapshot, board-open
Benchmarks: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard
Binding ADRs: ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253-amendment, ADR-0257, ADR-0263, ADR-0297, ADR-0314, ADR-0316, ADR-0321
Repo-local references: microservices/whiteboard/PRD.md; microservices/whiteboard/ARCHITECTURE.md; microservices/whiteboard/capabilities/canvas-op-append.yaml; microservices/whiteboard/capabilities/presence-sync.yaml; microservices/whiteboard/capabilities/history-snapshot.yaml; microservices/whiteboard/runbooks/local-crdt-merge-conflict.md; microservices/whiteboard/slos/local-crdt-merge-success.openslo.yaml; microservices/whiteboard/policies/local-crdt-merge-control.cedar

## Objective
- Define the deterministic merge-arbitration plan for concurrent canvas operations.
- Keep board merge semantics inside the whiteboard microservice boundary.
- Preserve ADR-0316 capability-tier discipline while satisfying ADR-0321 leader coverage.
- Replace hidden vendor merge rules with inspectable tenant-scoped evidence.
- Make CRDT convergence reviewable by SRE, auditor, and tenant admin personas.
- Treat Miro Enterprise multiplayer canvas as the benchmark for high-volume object mutation.
- Treat Mural Enterprise facilitation boards as the benchmark for workshop burst edits.
- Treat FigJam as the benchmark for cursor-rich sticky-note collaboration.
- Treat Lucidspark as the benchmark for structured diagram and sticky hybrid merges.
- Treat Whiteboard.fi as the benchmark for classroom fan-out with teacher-visible state.
- Treat Microsoft Whiteboard as the benchmark for Microsoft 365-style identity and export handoff.

## Current repo anchors
- anchor 001: PRD-whiteboard states that low-latency multi-user canvas operations are not document-file semantics.
- anchor 002: ARCHITECTURE.md declares canvas, board-session, sticky-note, template, and export contexts.
- anchor 003: canvas-op-append capability binds tenant_id, principal_id, audience_type, purpose, and data_class.
- anchor 004: presence-sync capability gives cursor and participant state a first-class record.
- anchor 005: history-snapshot capability gives arbitration a durable replay target.
- anchor 006: local-crdt-merge-control.cedar is the first policy hook for risky merge acceptance.
- anchor 007: local-crdt-merge-conflict.md is the operator path for unresolved or divergent merges.
- anchor 008: local-crdt-merge-success.openslo.yaml is the SLO evidence target for convergence.
- anchor 009: ADR-0321 authorizes whiteboard as a B2B SaaS leader anchor.
- anchor 010: ADR-0105 keeps merge logic separated across API, usecase, domain, kernel, adapter, and worker layers.

## Domain vocabulary
- vocabulary 001: `board_id` identifies the tenant-scoped canvas aggregate.
- vocabulary 002: `object_id` identifies a shape, sticky, connector, frame, timer marker, vote marker, or export marker.
- vocabulary 003: `operation_id` is the idempotent append key for a single canvas mutation.
- vocabulary 004: `client_epoch` is the local editor epoch advertised by the participant.
- vocabulary 005: `server_epoch` is the monotonic admission epoch assigned by whiteboard.
- vocabulary 006: `causal_frontier` is the vector summary attached to each mutation batch.
- vocabulary 007: `merge_lane` groups operations that may commute without semantic arbitration.
- vocabulary 008: `arbitration_reason` names why an operation needed deterministic policy.
- vocabulary 009: `semantic_conflict` means convergence is possible but user intent could be lost.
- vocabulary 010: `hard_conflict` means an operation violates tenant, policy, object, or pack invariants.
- vocabulary 011: `visible_repair` is a user-facing conflict marker placed on the board.
- vocabulary 012: `silent_merge` is allowed only when both mathematical convergence and semantic preservation pass.

## Arbitration model
- model 001: Append-only operation logs are the source of truth.
- model 002: Snapshot materialization is a cache and never the legal board record.
- model 003: CRDT convergence occurs in the kernel layer before adapter persistence fan-out.
- model 004: Cedar checks run before admission and again before risky repair promotion.
- model 005: Tenant scope is evaluated before causal frontier validation.
- model 006: Region and residency labels are evaluated before cross-cell replay.
- model 007: Operation ordering uses server_epoch first and operation_id as a deterministic tie-breaker.
- model 008: Object creation wins over property mutation only when the object tombstone is absent.
- model 009: Tombstone state wins over late property mutation unless recovery policy permits visible repair.
- model 010: Frame membership changes commute only when the target frame is still live.
- model 011: Connector endpoint changes require both endpoints to remain visible in the resolved snapshot.
- model 012: Sticky-note text merges use field-level intent preservation, not last-writer-wins.
- model 013: Freehand stroke chunks merge by stroke_id and chunk ordinal.
- model 014: Vote marker edits require governance policy when the session is sealed.
- model 015: Timer marker edits require facilitator authority when countdown state is active.
- model 016: Education-room student strokes never override teacher lock state.
- model 017: Template-instantiated objects carry immutable template_origin until export.
- model 018: Marketplace template license state is checked before merge replay exposes paid content.
- model 019: Audit-chain events seal admitted operations and rejected operations.
- model 020: Merge decisions emit refusal evidence for denied operations rather than dropping them.

## Required command flow
- flow 001: API receives `canvas.op.append` with tenant_id, principal_id, board_id, operation_id, and causal_frontier.
- flow 002: REST validates OpenAPI shape from microservices/whiteboard/contracts/openapi-v1.yaml.
- flow 003: Application resolves board session, pack overlay, home cell, and data class.
- flow 004: Usecase requests Cedar permit through the caller-side policy library.
- flow 005: Domain validates aggregate invariants before kernel merge.
- flow 006: Kernel computes deterministic CRDT merge candidate.
- flow 007: Kernel classifies silent_merge, visible_repair, or hard_conflict.
- flow 008: Usecase records server_epoch and arbitration_reason.
- flow 009: Adapter appends operation and decision evidence atomically.
- flow 010: Worker materializes updated read model and presence fan-out.
- flow 011: AsyncAPI emits canvas-operation-admitted or canvas-operation-rejected.
- flow 012: History snapshot worker records replay checkpoint on configured cadence.
- flow 013: Dashboard receives convergence metric without raw tenant_id cardinality.
- flow 014: Audit-chain receives tenant id in signed evidence.
- flow 015: Runbook trigger fires when hard_conflict rate breaches SLO.

## Conflict taxonomy
- taxonomy 001: `missing_parent_frame` covers object move into absent frame.
- taxonomy 002: `stale_tombstone_mutation` covers edit after deletion.
- taxonomy 003: `double_connector_endpoint` covers contradictory connector endpoint edits.
- taxonomy 004: `sticky_text_divergence` covers concurrent note text edits with overlapping ranges.
- taxonomy 005: `locked_region_write` covers student or participant writes into a locked board zone.
- taxonomy 006: `sealed_vote_mutation` covers vote changes after governance closure.
- taxonomy 007: `timer_authority_mismatch` covers countdown control by a non-facilitator.
- taxonomy 008: `template_license_unresolved` covers marketplace-origin object replay without entitlement.
- taxonomy 009: `pack_residency_mismatch` covers cross-region replay forbidden by pack overlay.
- taxonomy 010: `object_schema_revision_gap` covers operation generated against unsupported object schema.
- taxonomy 011: `presence_epoch_rewind` covers cursor state older than accepted session frontier.
- taxonomy 012: `export_snapshot_race` covers export render reading a pre-arbitration materialization.

## Policy hooks
- policy 001: `local-crdt-merge-control.cedar` denies mutation when tenant scope is absent.
- policy 002: The policy hook denies merge repair when principal lacks board edit authority.
- policy 003: The policy hook denies teacher lock override unless principal has education facilitator role.
- policy 004: The policy hook denies sealed-vote repair unless governance reopening is approved.
- policy 005: The policy hook denies marketplace-origin replay when DealSet settlement is unresolved.
- policy 006: The policy hook denies export snapshot inclusion for unlicensed template fragments.
- policy 007: The policy hook requires audit evidence id before admitting hard-conflict repair.
- policy 008: The policy hook requires purpose binding for imported Miro Enterprise board operations.
- policy 009: The policy hook requires purpose binding for imported Mural Enterprise workshop boards.
- policy 010: The policy hook requires purpose binding for imported FigJam sticky clusters.
- policy 011: The policy hook requires purpose binding for imported Lucidspark diagram objects.
- policy 012: The policy hook requires purpose binding for imported Whiteboard.fi classroom sessions.
- policy 013: The policy hook requires purpose binding for imported Microsoft Whiteboard boards.

## Data model additions
- data 001: Add `merge_decision_id` to the operation decision record.
- data 002: Add `causal_frontier_hash` to avoid unbounded vector-cardinality indexing.
- data 003: Add `arbitration_reason` enum with the conflict taxonomy above.
- data 004: Add `repair_visibility` enum: silent, visible_marker, rejected.
- data 005: Add `source_vendor_hint` for migration provenance only, never for authorization.
- data 006: Add `benchmark_profile` for evidence grouping: enterprise, workshop, education, microsoft365.
- data 007: Add `board_region_lock` to prevent residency-breaking replay.
- data 008: Add `object_schema_revision` to every operation payload.
- data 009: Add `template_origin_ref` to marketplace-instantiated objects.
- data 010: Add `export_snapshot_epoch` to render jobs that depend on arbitration.
- data 011: Add `teacher_lock_epoch` to education-room lock state.
- data 012: Add `facilitation_epoch` to timer and voting governance overlays.

## SLO and telemetry
- telemetry 001: Measure merge success as accepted operations divided by admissible operations.
- telemetry 002: Exclude policy-denied attacks from merge success numerator and denominator.
- telemetry 003: Count visible repairs separately from silent merges.
- telemetry 004: Emit p95 and p99 merge latency by operation class.
- telemetry 005: Emit causal frontier width distribution by board size bucket.
- telemetry 006: Emit hard_conflict rate by benchmark_profile.
- telemetry 007: Emit export snapshot race count.
- telemetry 008: Emit education lock override denial count.
- telemetry 009: Emit sealed vote mutation denial count.
- telemetry 010: Emit marketplace entitlement replay denial count.
- telemetry 011: Link traces to operation_id, board_id hash, and audit event id.
- telemetry 012: Never put raw tenant_id into high-cardinality metric labels.

## Replay and import requirements
- replay 001: Miro Enterprise import must preserve frame hierarchy and connector endpoints.
- replay 002: Mural Enterprise import must preserve facilitation areas and voting artifacts.
- replay 003: FigJam import must preserve sticky clusters, stamps, reactions, and cursor replay where exported.
- replay 004: Lucidspark import must preserve diagram object identity and line routing metadata.
- replay 005: Whiteboard.fi import must preserve teacher-board and student-board relationships.
- replay 006: Microsoft Whiteboard import must preserve Microsoft identity provenance separately from Oyatie principal ids.
- replay 007: Imports run dry-run arbitration before state mutation.
- replay 008: Dry-run output names each rejected operation and repair candidate.
- replay 009: Replay is idempotent across retry, worker restart, and cell failover.
- replay 010: Replay never rewrites original source operation ids.
- replay 011: Replay snapshot hashes are recorded in audit evidence.
- replay 012: Replay rejects unsupported object revisions with a migration plan, not partial silent loss.

## Acceptance criteria
- acceptance 001: Every admitted canvas operation has server_epoch, operation_id, and merge_decision_id.
- acceptance 002: Every rejected operation has denial evidence and arbitration_reason.
- acceptance 003: Silent_merge decisions are deterministic across two fresh replay runs.
- acceptance 004: Visible_repair decisions create board-visible conflict markers.
- acceptance 005: Hard_conflict decisions do not mutate the board aggregate.
- acceptance 006: Policy denial occurs before adapter persistence.
- acceptance 007: History snapshot references an export_snapshot_epoch after merge completion.
- acceptance 008: SLO evidence covers local-crdt-merge-success.
- acceptance 009: Runbook evidence covers local-crdt-merge-conflict.
- acceptance 010: Benchmark evidence names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
- acceptance 011: ADR-0321 is cited in the final evidence packet.
- acceptance 012: ADR-0316 capability-tier constraints remain intact.

## Test plan
- test 001: Unit-test operation ordering with equal client epochs and different server epochs.
- test 002: Property-test convergence for shuffled operation batches.
- test 003: Property-test idempotent replay under duplicate operation submission.
- test 004: Contract-test OpenAPI request fields for canvas-op-append.
- test 005: AsyncAPI-test admitted and rejected event shapes.
- test 006: Cedar-fixture-test tenant absence denial.
- test 007: Cedar-fixture-test teacher lock override denial.
- test 008: Cedar-fixture-test facilitator timer authority denial.
- test 009: Migration-fixture-test Miro Enterprise frame import.
- test 010: Migration-fixture-test Mural Enterprise voting artifact import.
- test 011: Migration-fixture-test FigJam sticky text overlap.
- test 012: Migration-fixture-test Lucidspark connector endpoint conflict.
- test 013: Migration-fixture-test Whiteboard.fi student board lock.
- test 014: Migration-fixture-test Microsoft Whiteboard identity provenance.
- test 015: Replay-test export snapshot race prevention.
- test 016: SLO-test merge success metric emission.

## Rollback and recovery
- rollback 001: Disable risky visible repair promotion through policy flag.
- rollback 002: Continue accepting safe silent merges when policy permits.
- rollback 003: Quarantine hard-conflict operations in append-only evidence storage.
- rollback 004: Rebuild materialized board state from last clean history snapshot.
- rollback 005: Re-run operation replay with old arbitration profile when the new profile fails.
- rollback 006: Prevent export-render from using quarantined epochs.
- rollback 007: Notify workflow-engine for tenant admin review when repair visibility changes.
- rollback 008: Attach runbook local-crdt-merge-conflict to incident-response evidence.
- rollback 009: Preserve all source vendor ids for migration rollback.
- rollback 010: Never perform destructive correction on the operation log.

## Command and proto deltas
- proto 001: Add `AppendCanvasOperationRequest.board_id` as a required tenant-scoped board identifier.
- proto 002: Add `AppendCanvasOperationRequest.operation_id` as the idempotency key visible to REST and gRPC.
- proto 003: Add `AppendCanvasOperationRequest.causal_frontier_hash` to avoid shipping unbounded vector state through every internal hop.
- proto 004: Add `AppendCanvasOperationRequest.object_schema_revision` so kernel merge can reject stale clients before storage.
- proto 005: Add `MergeDecision.merge_decision_id` as the durable join key between operation, audit, and snapshot records.
- proto 006: Add `MergeDecision.arbitration_reason` with values for tombstone, frame, connector, lock, vote, timer, license, and residency conflicts.
- proto 007: Add `MergeDecision.repair_visibility` so clients know whether to render a visible conflict marker.
- proto 008: Add `MergeDecision.source_vendor_hint` only for migration provenance, not for permit evaluation.
- proto 009: Add `BoardMergeReplayRequest.snapshot_epoch` to replay a specific board state deterministically.
- proto 010: Add `BoardMergeReplayResponse.materialized_board_hash` to compare replay outputs across cells.
- proto 011: Add `CanvasOperationRejected.reason_detail` with safe user text and signed internal audit detail separated.
- proto 012: Add `ExportSnapshotFence.merge_decision_watermark` so export-render cannot race ahead of arbitration.

## Cedar facts
- cedar-fact 001: `principal_tenant_id` must equal `board_tenant_id` before merge admission.
- cedar-fact 002: `board_data_class` must be compatible with `operation_data_class`.
- cedar-fact 003: `object_lock_state` blocks writes unless `principal_role` includes facilitator, teacher, or board owner authority for that lock.
- cedar-fact 004: `vote_sealed=true` blocks vote marker mutation unless `workflow_reopen_approved=true`.
- cedar-fact 005: `timer_active=true` blocks countdown mutation unless `principal_is_active_facilitator=true`.
- cedar-fact 006: `template_dealset_state` must be settled before replay exposes marketplace-origin objects.
- cedar-fact 007: `education_peer_hidden=true` blocks student-origin writes into peer board shards.
- cedar-fact 008: `residency_boundary_crossed=true` blocks replay even when CRDT convergence would be mathematically safe.
- cedar-fact 009: `source_vendor_hint` is not an authorization input and is logged only as provenance.
- cedar-fact 010: `audit_event_id` is required before visible repair promotion.

## Workflow decisions
- workflow 001: Merge admission is synchronous for single-object property edits under the latency budget.
- workflow 002: Merge admission is asynchronous for import batches, region failover replay, and snapshot rebuild.
- workflow 003: Visible repair opens a workflow-engine review task only when user intent cannot be preserved.
- workflow 004: Hard conflict never blocks unrelated merge lanes on the same board.
- workflow 005: Export-render waits for a merge watermark when the requested snapshot overlaps active arbitration.
- workflow 006: Presence-sync can publish optimistic cursor state but not unadmitted object state.
- workflow 007: History-snapshot records both accepted operations and rejected-operation evidence.
- workflow 008: Tenant admin remediation can approve a repair profile but cannot rewrite source operations.

## Failure and replay cases
- failure 001: Replayed Miro Enterprise frame moves must converge even when child object operations arrive first.
- failure 002: Replayed Mural Enterprise voting artifacts must reject late vote edits after sealed state.
- failure 003: Replayed FigJam sticky edits must preserve overlapping text intent or show visible repair.
- failure 004: Replayed Lucidspark connector edits must not leave dangling endpoints.
- failure 005: Replayed Whiteboard.fi student boards must keep teacher locks authoritative.
- failure 006: Replayed Microsoft Whiteboard imports must not treat Microsoft account ids as Oyatie principals.
- failure 007: Regional failover replay must produce the same materialized_board_hash for the same snapshot_epoch.
- failure 008: Worker retry must not duplicate repair markers.
- failure 009: Snapshot rebuild must preserve rejected-operation evidence.
- failure 010: Export snapshot fencing must prevent partially merged board artifacts.

## Evidence fields
- evidence 001: `merge_decision_id` links API response, event, audit-chain record, and history snapshot.
- evidence 002: `operation_id` links idempotency storage to replay output.
- evidence 003: `causal_frontier_hash` proves which client frontier was admitted.
- evidence 004: `server_epoch` proves admission order.
- evidence 005: `arbitration_reason` proves why repair or denial occurred.
- evidence 006: `repair_visibility` proves whether the user saw a conflict marker.
- evidence 007: `policy_decision_id` proves Cedar evaluated the merge.
- evidence 008: `source_vendor_hint` proves migration source without controlling authorization.
- evidence 009: `materialized_board_hash` proves deterministic replay.
- evidence 010: `export_snapshot_epoch` proves export used settled merge state.

## Done definition
- done 001: IP references PRD, architecture, capability, policy, SLO, and runbook anchors.
- done 002: IP names the displaced benchmark set in enterprise-specific language.
- done 003: IP defines domain vocabulary and conflict taxonomy.
- done 004: IP defines policy, telemetry, replay, tests, and rollback.
- done 005: IP stays inside microservices/whiteboard and does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
