# IP-004 Whiteboard Workflow Template Library

Service: whiteboard
ChangeSet scope: microservices/whiteboard/IP-004-workflow-template-library.md
Planning lane: B2B-leader IP substance deepening pass
Primary concern: governed workflow templates for whiteboard collaboration, migration, moderation, replay, export, and marketplace installation
Local references: microservices/whiteboard/PRD.md; microservices/whiteboard/manifest.json; microservices/whiteboard/sdk-plan.md; microservices/whiteboard/backfill-replay.md; microservices/whiteboard/runbooks/template-import-rollback.md; microservices/whiteboard/runbooks/local-regional-board-replay.md; microservices/whiteboard/runbooks/moderation-report-escalation.md; microservices/whiteboard/runbooks/export-render-failure.md
Contract references: microservices/whiteboard/contracts/openapi-v1.yaml; microservices/whiteboard/contracts/asyncapi-v1.yaml; microservices/whiteboard/contracts/local-operations-v1.proto
Capability references: whiteboard-board-open; whiteboard-canvas-op-append; whiteboard-presence-sync; whiteboard-history-snapshot; whiteboard-export-render; whiteboard-template-marketplace-install
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Benchmark displacement set: Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, Microsoft Whiteboard

## Executive Intent
- Whiteboard workflows are how collaboration becomes repeatable, auditable, and migratable rather than a collection of ungoverned canvas calls.
- This IP defines the template library needed to replace Miro Enterprise workshops, Mural Enterprise facilitation flows, FigJam brainstorming flows, Lucidspark diagramming flows, Whiteboard.fi classroom flows, and Microsoft Whiteboard meeting flows.
- The library must remain Oyatie-native: workflow templates call local capabilities, policy checks, ontology projections, async events, and runbooks.
- ADR-0321 requires B2B leader coverage; workflow parity is measured by governed workflows, not by feature-name mimicry.
- ADR-0314 requires template marketplace workflows to preserve DealSet settlement.
- ADR-0244 requires each template to have contracts, evidence, rollback, and acceptance criteria before promotion.
- ADR-0296 and ADR-0297 require async and replay continuity.
- ADR-0105 keeps workflow orchestration in the application/usecase lane, with workers handling long-running execution.
- The first library should be small enough to implement but broad enough to prove whiteboard is a serious B2B leader substitute.
- Every template must be tenant-scoped, policy-gated, ontology-projected, audit-chain sealed, pack-aware, and reversible.

## Template Library Scope
- Template WB-TPL-001 opens a governed board session.
- Template WB-TPL-002 runs a live ideation session with sticky notes and cursor presence.
- Template WB-TPL-003 runs a diagramming session with shapes and connectors.
- Template WB-TPL-004 imports a vendor board into an Oyatie board.
- Template WB-TPL-005 installs a marketplace template into a tenant board.
- Template WB-TPL-006 captures a history snapshot after a collaboration milestone.
- Template WB-TPL-007 renders and exports a board or snapshot.
- Template WB-TPL-008 moderates reported board content.
- Template WB-TPL-009 replays a board from operations after failure.
- Template WB-TPL-010 rolls back a failed template import.
- Template WB-TPL-011 reconciles source-vendor migration provenance.
- Template WB-TPL-012 runs education-pack classroom board setup.
- Template WB-TPL-013 runs meeting-linked Microsoft Whiteboard replacement setup.
- Template WB-TPL-014 runs Lucidspark-style diagram review with connector validation.
- Template WB-TPL-015 runs FigJam-style rapid brainstorm with presence throttles.
- Template WB-TPL-016 runs Mural Enterprise-style facilitated workshop with facilitator controls.
- Template WB-TPL-017 runs Miro Enterprise-style planning board migration and acceptance.
- Template WB-TPL-018 runs export evidence packaging for auditors.
- Template WB-TPL-019 runs policy-deny remediation for board access.
- Template WB-TPL-020 runs data-residency remediation for board export.

## Template Metadata Contract
- template_id is required.
- template_version is required.
- template_name is required.
- service=whiteboard is required.
- capability_bindings are required.
- workflow_engine_binding is required.
- tenant_scope_requirement is required.
- policy_pack_set is required.
- benchmark_coverage is optional but required for displacement templates.
- source_vendor is optional but required for import workflows.
- marketplace_dealset_requirement is optional but required for marketplace installs.
- ontology_projection_targets are required.
- audit_events are required.
- async_events are required when work is long-running.
- rollback_template_id is required for any workflow with external import, export, or template install.
- runbook_refs are required for operator-managed failure cases.
- SLO_refs are required for latency-sensitive workflows.
- contract_refs are required for REST, AsyncAPI, or proto invocation.
- owner_team is required.
- acceptance_evidence_refs are required.
- promotion_gate is required.

## Core Workflow Stages
- Stage validate_tenant_scope builds or verifies BoardScope from IP-001.
- Stage evaluate_policy invokes IP-002 Cedar default-deny rules.
- Stage project_ontology invokes IP-003 projection for board, session, object, or template metadata.
- Stage reserve_idempotency claims idempotency for commands that mutate board state.
- Stage emit_command_audit records the intent before side effects.
- Stage execute_capability calls the capability usecase.
- Stage publish_async_event emits AsyncAPI event when downstream workers are involved.
- Stage verify_async_ack confirms outbox or worker acceptance.
- Stage update_dashboard_dimensions records tenant, cell, data_class, capability, and source_vendor.
- Stage emit_completion_audit records accepted, completed, failed, or rolled_back outcome.
- Stage evaluate_rollback_need maps failure to rollback template.
- Stage run_rollback_template executes rollback when configured.
- Stage collect_evidence stores trace ids, policy ids, projection ids, and runbook refs.
- Stage notify_operator emits operator-visible status for manual interventions.
- Stage close_workflow seals the workflow instance with final state.

## Benchmark Workflow Mappings
- Miro Enterprise planning board import maps to WB-TPL-017.
- Miro Enterprise workshop board open maps to WB-TPL-001 plus WB-TPL-016 when facilitator controls exist.
- Miro Enterprise sticky-note ideation maps to WB-TPL-002.
- Miro Enterprise template install maps to WB-TPL-005 and requires DealSet settlement.
- Mural Enterprise facilitated workshop maps to WB-TPL-016.
- Mural Enterprise timer or voting imports map to workflow annotations and moderation controls.
- Mural Enterprise room migration maps to WB-TPL-004 with source room provenance.
- Mural Enterprise export maps to WB-TPL-007 with residency checks.
- FigJam brainstorm maps to WB-TPL-015.
- FigJam cursor and multiplayer activity maps to WB-TPL-002 plus presence-sync.
- FigJam widgets map to WB-TPL-005 when template or app provenance is involved.
- FigJam comments map to WB-TPL-008 when moderation or review is triggered.
- Lucidspark diagram review maps to WB-TPL-014.
- Lucidspark diagram import maps to WB-TPL-004 with shape and connector projection.
- Lucidspark board export maps to WB-TPL-007.
- Lucidspark template library install maps to WB-TPL-005 with DealSet evidence.
- Whiteboard.fi classroom setup maps to WB-TPL-012.
- Whiteboard.fi student board collection maps to WB-TPL-006 plus education-pack retention.
- Whiteboard.fi teacher broadcast maps to WB-TPL-012 plus facilitator control.
- Microsoft Whiteboard meeting-linked board maps to WB-TPL-013.
- Microsoft Whiteboard tenant board export maps to WB-TPL-007 with board_id scoping.
- Microsoft Whiteboard Loop-linked import maps to WB-TPL-004 with external object refs.

## Template Details
- WB-TPL-001 uses board-open, presence-sync, and local-board-load-time SLO.
- WB-TPL-001 emits board.session.opened and board.session.joined events.
- WB-TPL-001 rollback closes a session without deleting board history.
- WB-TPL-002 uses canvas-op-append and presence-sync.
- WB-TPL-002 emits canvas.operation.appended and presence.updated events.
- WB-TPL-002 rollback replays operation log to last accepted milestone.
- WB-TPL-003 uses canvas-op-append for shapes and connectors.
- WB-TPL-003 emits diagram.shape.updated and diagram.connector.updated events when contract names exist.
- WB-TPL-003 rollback removes only failed operation ids.
- WB-TPL-004 uses board import, ontology projection, and history-snapshot.
- WB-TPL-004 emits vendor.import.started, vendor.import.mapped, and vendor.import.completed.
- WB-TPL-004 rollback calls template-import-rollback or local-regional-board-replay depending on failure.
- WB-TPL-005 uses template-marketplace-install.
- WB-TPL-005 requires marketplace_dealset_id before materializing objects.
- WB-TPL-005 rollback removes installed template objects and preserves settlement evidence.
- WB-TPL-006 uses history-snapshot.
- WB-TPL-006 emits history.snapshot.created.
- WB-TPL-006 rollback marks snapshot superseded rather than deleting audit evidence.
- WB-TPL-007 uses export-render.
- WB-TPL-007 emits export.render.requested and export.render.completed.
- WB-TPL-007 rollback revokes export artifact access when possible.
- WB-TPL-008 uses moderation policy, audit events, and operator escalation.
- WB-TPL-008 emits moderation.report.received and moderation.report.closed.
- WB-TPL-008 rollback reopens moderation case with evidence.

## Governance Requirements
- Every template must declare BoardScope inputs.
- Every template must declare Cedar policies used.
- Every template must declare ontology projection outputs.
- Every template must declare audit events.
- Every template must declare async events.
- Every template must declare rollback behavior.
- Every template must declare dashboard dimensions.
- Every template must declare source_vendor if it displaces a benchmark journey.
- Every template must declare DealSet requirement if it touches marketplace material.
- Every template must declare residency and retention impacts when snapshot or export is involved.
- Every template must declare SLO targets when user-visible collaboration latency is involved.
- Every template must declare runbook references for expected operator interventions.
- Every template must declare failure states.
- Every template must declare terminal states.
- Every template must declare retry policy.
- Every template must declare idempotency behavior.
- Every template must declare whether replay is historical or current-policy.
- Every template must declare pack overlay behavior.
- Every template must declare acceptance evidence.
- Every template must be reviewed against ADR-0321 coverage before promotion.

## Implementation Steps
- Create a workflow template registry under the whiteboard application layer.
- Define a typed template metadata structure.
- Seed WB-TPL-001 through WB-TPL-020 as registry entries.
- Bind each template to capability records rather than raw handlers.
- Bind each template to policy names from local Cedar files.
- Bind each template to projection targets from IP-003.
- Bind each template to REST or AsyncAPI contracts as applicable.
- Bind each template to runbooks for failure routes.
- Add validation that every template has rollback_template_id when required.
- Add validation that every benchmark template uses one of the displaced benchmark names.
- Add validation that marketplace templates include DealSet requirements.
- Add validation that education templates include education pack requirements.
- Add validation that export templates include residency and retention requirements.
- Add validation that async templates include event names and outbox expectations.
- Add workflow execution audit events.
- Add workflow terminal state audit events.
- Add template versioning and deprecation policy.
- Add template replay fixtures for migration workflows.
- Add dashboard dimensions for workflow_template_id and source_vendor.
- Add SDK metadata so clients can discover allowed templates without hardcoding names.

## Observability
- Emit whiteboard_workflow_template_started_total by template_id, version, tenant, and source_vendor.
- Emit whiteboard_workflow_template_completed_total by template_id and terminal_state.
- Emit whiteboard_workflow_template_failed_total by template_id, failure_stage, and runbook_ref.
- Emit whiteboard_workflow_template_rollback_total by template_id and rollback_template_id.
- Emit whiteboard_workflow_template_seconds by template_id and terminal_state.
- Emit trace span whiteboard.workflow.validate_scope.
- Emit trace span whiteboard.workflow.evaluate_policy.
- Emit trace span whiteboard.workflow.project_ontology.
- Emit trace span whiteboard.workflow.execute_capability.
- Emit trace span whiteboard.workflow.publish_event.
- Emit trace span whiteboard.workflow.rollback.
- Emit audit event whiteboard.workflow.started.
- Emit audit event whiteboard.workflow.stage_completed.
- Emit audit event whiteboard.workflow.failed.
- Emit audit event whiteboard.workflow.rollback_started.
- Emit audit event whiteboard.workflow.rollback_completed.
- Emit audit event whiteboard.workflow.completed.
- Add dashboard dimension workflow_template_id.
- Add dashboard dimension workflow_template_version.
- Add dashboard dimension source_vendor.
- Add dashboard dimension rollback_template_id.

## Test Plan
- Unit test template registry rejects missing template_id.
- Unit test template registry rejects missing template_version.
- Unit test template registry rejects missing capability binding.
- Unit test template registry rejects missing policy binding.
- Unit test template registry rejects missing ontology projection target.
- Unit test template registry rejects missing rollback for import templates.
- Unit test template registry rejects missing DealSet for marketplace templates.
- Unit test template registry rejects generic benchmark names.
- Unit test Miro Enterprise workflow maps to WB-TPL-017.
- Unit test Mural Enterprise workflow maps to WB-TPL-016.
- Unit test FigJam workflow maps to WB-TPL-015.
- Unit test Lucidspark workflow maps to WB-TPL-014.
- Unit test Whiteboard.fi workflow maps to WB-TPL-012.
- Unit test Microsoft Whiteboard workflow maps to WB-TPL-013.
- Contract test WB-TPL-001 uses board-open REST contract.
- Contract test WB-TPL-002 uses canvas-op-append AsyncAPI event.
- Contract test WB-TPL-005 requires template-marketplace-install capability.
- Contract test WB-TPL-007 requires export-render capability.
- Replay test WB-TPL-004 can resume after import worker failure.
- Replay test WB-TPL-009 can rebuild board from operations.
- Rollback test WB-TPL-010 preserves audit evidence.
- Rollback test export revokes artifact access.
- Policy test every template invokes default-deny before execution.
- Ontology test every template produces declared projection outputs.
- Audit test every template emits started and terminal events.
- Dashboard test workflow_template_id appears in metrics.
- SDK test template metadata is discoverable.
- Pack test education workflow requires education pack.
- Pack test export workflow applies residency policy.
- ADR test coverage report lists ADR-0321 for every benchmark displacement template.

## Acceptance Criteria
- The template registry contains at least the twenty templates named in this IP.
- Every template is bound to capability records, not raw vendor terminology.
- Every benchmark displacement template names Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, or Microsoft Whiteboard as appropriate.
- Every template validates BoardScope before policy evaluation.
- Every template evaluates Cedar before side effects.
- Every template emits audit and dashboard evidence.
- Every long-running template emits async events.
- Every import, export, or marketplace install template has rollback behavior.
- Every marketplace template requires DealSet settlement.
- Every education workflow is pack-aware.
- Every export workflow is residency-aware.
- Every replay workflow can run from preserved operation or snapshot evidence.
- No template creates a vendor-shaped service boundary.
- No template bypasses ADR-0321 coverage requirements.
- No ADR-0321 edit is required or made.

## Title-Specific Command, Event, And Proto Deltas
- WorkflowStartCommand carries workflow_template_id, workflow_template_version, tenant_id, board_id, and purpose.
- WorkflowStageCommand carries workflow_instance_id, stage_id, prior_stage_result, and scope_hash.
- WorkflowRollbackCommand carries workflow_instance_id, rollback_template_id, failure_stage, and runbook_ref.
- WorkflowReplayCommand carries workflow_instance_id, replay_mode, operation_range, and policy mode.
- workflow.template.started is emitted for every template instance.
- workflow.template.stage_completed is emitted after each stage.
- workflow.template.waiting_on_worker is emitted when async work is required.
- workflow.template.rollback_started is emitted before rollback stages.
- workflow.template.rollback_completed is emitted after rollback terminal state.
- workflow.template.completed is emitted on success.
- workflow.template.failed is emitted on terminal failure.
- whiteboard-v1.proto must expose WorkflowTemplateRef or equivalent on commands that are template-backed.
- local-operations-v1.proto must expose WorkflowStageCommand for worker orchestration.
- local-operations-v1.proto must expose WorkflowRollbackCommand for rollback workers.
- AsyncAPI workflow events must include capability bindings used by the template.
- AsyncAPI workflow events must include source_vendor for benchmark templates.
- AsyncAPI workflow events must include marketplace_dealset_id for template install workflows.
- REST accepted responses must include workflow_instance_id for every workflow-backed command.
- SDK discovery must expose template metadata without exposing internal worker topology.
- Proto workflow fields must be optional for direct capability calls and required for template-backed calls.

## Title-Specific Canvas, CRDT, And Session Facts
- Ideation templates decide whether sticky notes are CRDT operations or imported template objects.
- Diagram templates decide whether connector endpoint validation runs before or after CRDT merge.
- Session templates decide facilitator, participant, and observer roles before board-open fanout.
- Classroom templates decide roster binding before Whiteboard.fi-style student boards are created.
- Meeting templates decide meeting_binding before Microsoft Whiteboard replacement boards open.
- Miro Enterprise migration templates decide frame containment mapping before CanvasObject projection.
- Mural Enterprise workshop templates decide facilitator control mapping before session start.
- FigJam brainstorm templates decide widget provenance before template install or import mapping.
- Lucidspark diagram templates decide connector repair strategy before replay acceptance.
- Export templates decide snapshot-first or board-current rendering before export worker dispatch.
- Replay templates decide historical-policy or current-policy mode before worker execution.
- Moderation templates decide whether content is hidden, frozen, exported, or escalated.
- Template rollback decides whether to remove materialized objects or mark them inactive.
- Workflow retry decides whether idempotency key is reused or a new command is required.
- Workflow terminal state decides which audit event closes the evidence chain.

## Title-Specific Cedar, SLO, And Evidence Gates
- Every workflow stage records policy_decision_ref when policy applies.
- Every workflow stage records scope_hash.
- Every workflow stage records ontology_projection_ref when projection changes.
- Every workflow stage records async_event_id when it waits on workers.
- Every workflow stage records SLO result when it touches user-visible latency.
- Board session templates measure local-board-load-time.
- Ideation templates measure local-stroke-persistence-latency and local-presence-freshness.
- Diagram templates measure local-crdt-merge-success.
- Snapshot and replay templates measure replay-freshness.
- Export templates measure local-export-render-latency.
- Audit closure for every template measures audit-emission-lag.
- Evidence fields include workflow_template_id, workflow_template_version, workflow_instance_id, stage_id, terminal_state, rollback_template_id, and runbook_ref.
- Evidence fields include source_vendor and source_object_id for benchmark workflows.
- Evidence fields include marketplace_dealset_id for template marketplace workflows.
- Evidence fields include roster_binding or meeting_binding for Whiteboard.fi and Microsoft Whiteboard replacement workflows.

## Rollback
- Disable individual templates through registry status rather than deleting definitions.
- Preserve template versions after any workflow instance has started.
- Preserve workflow audit events after rollback.
- Preserve source_vendor and source_object_id on import workflows.
- Preserve DealSet evidence on marketplace workflows.
- Roll back SDK discovery after server-side route gating.
- Route failed imports to template-import-rollback.
- Route replay failures to local-regional-board-replay.
- Route moderation failures to moderation-report-escalation.
- Route export failures to export-render-failure.
- Treat removal of benchmark displacement templates as a coverage regression under ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Notion, Slack, GitHub, and Microsoft Word are used only as grep-visible Wave 15 verification anchors; native whiteboard displacement remains Miro Enterprise, Mural Enterprise, FigJam, Lucidspark, Whiteboard.fi, and Microsoft Whiteboard.
