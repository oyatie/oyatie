# IP-030 ITSM sla-breach-remediation-loop

Service: itsm
ChangeSet scope: microservices/itsm/IP-030-sla-breach-remediation-loop.md
Benchmarks displaced: ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, Freshservice
Binding ADRs: ADR-0105, ADR-0244, ADR-0246, ADR-0258, ADR-0263, ADR-0316, ADR-0321

## Objective
- Objective 001: Build the ITSM SLA breach remediation loop for incident, service request, catalog fulfillment, and change verification commitments.
- Objective 002: Displace ServiceNow SLA engine, Jira Service Management SLA rules, BMC service targets, Ivanti service level workflows, and Freshservice SLA policies.
- Objective 003: Preserve SLA clock state, pause reasons, breach evidence, recompute evidence, remediation owner, and sealed audit history.
- Objective 004: Prevent vendor SLA recalculation from rewriting already sealed breach evidence.
- Objective 005: Make breach remediation buildable from this IP under docs/standards/documentation-rigor.md section 1.1.
- Objective 006: Keep this IP scoped to ITSM and avoid manifests, journeys, ERP, ADR-0321 edits, and other B2B leader batches.

## SLA objects
- Object 001: `ItsmSlaPolicy` defines target, clock rules, pause rules, breach threshold, pack overlay, and data class.
- Object 002: `ItsmSlaClock` records not_started, running, paused, breached, met, recomputed, sealed, and rolled_back states.
- Object 003: `ItsmSlaTarget` records response, resolution, fulfillment, approval, verification, or custom target kind.
- Object 004: `ItsmSlaBreach` records breach time, target, reason, affected object, owner, and evidence refs.
- Object 005: `ItsmSlaPause` records pause reason, actor, policy, start, end, and approval evidence.
- Object 006: `ItsmSlaRecompute` records old state, new state, recompute reason, reviewer, and immutable prior evidence.
- Object 007: `ItsmSlaRemediation` records owner, action plan, due date, workflow run, escalation target, and completion evidence.
- Object 008: `ItsmSlaAppeal` records contested breach, requester, reviewer, decision, and audit refs.
- Object 009: `ItsmSlaExport` records tenant-scoped export of SLA evidence for audit and customer reporting.
- Object 010: `ItsmSlaRollback` records compensation for erroneous mutable effects without deleting breach evidence.

## Clock rules
- Clock rule 001: Clock start requires accepted incident, request, change, or catalog fulfillment object.
- Clock rule 002: Clock start requires tenant context and data_class.
- Clock rule 003: Clock start requires SLA policy version.
- Clock rule 004: Clock pause requires permitted pause reason and actor authority.
- Clock rule 005: Clock pause does not erase elapsed time before pause.
- Clock rule 006: Clock resume records pause interval and audit event.
- Clock rule 007: Clock breach records sealed breach evidence and remediation trigger.
- Clock rule 008: Clock met records target completion and completion evidence.
- Clock rule 009: Clock recompute creates recompute evidence and never overwrites sealed prior breach.
- Clock rule 010: Clock rollback compensates downstream notifications or escalations but preserves audit events.
- Clock rule 011: Pack overlays can require additional targets or shorter thresholds.
- Clock rule 012: Pack overlays cannot silently lengthen already active targets without audit evidence.

## Vendor displacement behavior
- Vendor behavior 001: ServiceNow SLA definitions map to `ItsmSlaPolicy` with source policy id as provenance.
- Vendor behavior 002: ServiceNow task SLA records map to `ItsmSlaClock` and breach evidence.
- Vendor behavior 003: ServiceNow retroactive pause or recalculation maps to recompute evidence and cannot rewrite sealed breach.
- Vendor behavior 004: Jira Service Management SLA goals map to response or resolution targets.
- Vendor behavior 005: Jira calendar and pause conditions map to clock rules and pack overlays.
- Vendor behavior 006: Jira SLA breach notifications map to remediation actions and not direct escalation authority.
- Vendor behavior 007: BMC service targets map to `ItsmSlaPolicy` with target kind and source ref.
- Vendor behavior 008: BMC measurement records map to clocks and breach evidence.
- Vendor behavior 009: Ivanti service level workflows map to remediation workflows and cannot bypass approval.
- Vendor behavior 010: Ivanti automated reassignment maps to remediation suggestion, not ownership authority.
- Vendor behavior 011: Freshservice SLA policies map to targets and escalation policies with tenant-scoped evidence.
- Vendor behavior 012: Freshservice business hours map to calendar refs and cannot silently change historical clock behavior.

## Remediation loop
- Loop 001: Detect breach from clock tick, event update, replay, or recompute.
- Loop 002: Validate tenant context and affected object ref.
- Loop 003: Evaluate Cedar policy for remediation owner assignment.
- Loop 004: Seal breach evidence with target, elapsed time, pause intervals, policy version, and affected object.
- Loop 005: Select remediation workflow template based on object type, data class, target kind, severity, and pack overlays.
- Loop 006: Notify owner, backup owner, tenant admin, or incident bridge according to policy.
- Loop 007: Create remediation task with due date and escalation rules.
- Loop 008: Track remediation progress through workflow run.
- Loop 009: Verify remediation completion against original target and current object state.
- Loop 010: Seal remediation completion evidence.
- Loop 011: Export audit report when customer or regulator policy requires it.
- Loop 012: Escalate unresolved breach after configured interval.
- Loop 013: Allow appeal only through explicit appeal record and reviewer decision.
- Loop 014: Recompute only through recompute object that preserves old breach state.
- Loop 015: Roll back erroneous notifications or tasks without deleting breach evidence.

## Data model
- Data model 001: `sla_policy_id` is tenant-scoped and versioned.
- Data model 002: `sla_clock_id` is deterministic from tenant_id, target object ref, target kind, and policy version.
- Data model 003: `breach_id` is deterministic from clock id, breach timestamp, target kind, and version.
- Data model 004: `pause_interval_id` records actor, reason, start, end, and policy decision id.
- Data model 005: `recompute_id` records old digest, new digest, reviewer, reason, and evidence refs.
- Data model 006: `remediation_id` records breach id, owner, workflow run, due date, and completion state.
- Data model 007: `appeal_id` records contested breach, reviewer, decision, and outcome.
- Data model 008: `export_id` records tenant-scoped report, recipient, retention, and pack overlay.
- Data model 009: `audit_event_id` records every clock state transition and remediation step.
- Data model 010: `policy_decision_id` records every pause, recompute, owner assignment, export, and appeal decision.

## Implementation sequence
- Implementation 001: Add SLA policy, target, clock, breach, pause, recompute, remediation, appeal, export, and rollback structs.
- Implementation 002: Add clock state transition validator.
- Implementation 003: Add target evaluator for response, resolution, fulfillment, approval, and verification targets.
- Implementation 004: Add business-calendar resolver with tenant and pack overlays.
- Implementation 005: Add pause evaluator with policy and reason checks.
- Implementation 006: Add breach detector with deterministic breach id generation.
- Implementation 007: Add remediation owner selector with Cedar default-deny evidence.
- Implementation 008: Add remediation workflow selector.
- Implementation 009: Add recompute service that preserves prior evidence.
- Implementation 010: Add appeal service with reviewer decision evidence.
- Implementation 011: Add export service with tenant-scoped report generation.
- Implementation 012: Add replay service that recalculates clocks and detects drift.
- Implementation 013: Add rollback service for erroneous mutable downstream effects.
- Implementation 014: Add REST examples for SLA policy, clock, breach, remediation, recompute, appeal, and export.
- Implementation 015: Add AsyncAPI events for clock_started, clock_paused, clock_resumed, breach_detected, remediation_completed, recompute_completed, appeal_decided, and export_completed.

## Test matrix
- Test 001: Unit test starts clock only for accepted target object.
- Test 002: Unit test rejects missing tenant context.
- Test 003: Unit test rejects pause without allowed reason.
- Test 004: Unit test records pause interval without erasing elapsed time.
- Test 005: Unit test detects breach at configured threshold.
- Test 006: Unit test preserves sealed breach evidence after recompute.
- Test 007: Unit test selects remediation owner through Cedar evidence.
- Test 008: Unit test selects remediation workflow by object type and data class.
- Test 009: Unit test exports SLA report only with export authority.
- Test 010: Unit test appeal decision preserves original breach state.
- Test 011: ServiceNow fixture maps task SLA to canonical clock.
- Test 012: ServiceNow fixture denies retroactive rewrite of sealed breach.
- Test 013: Jira fixture maps SLA goal to target and calendar.
- Test 014: BMC fixture maps service target to policy and clock.
- Test 015: Ivanti fixture maps automated reassignment to remediation suggestion only.
- Test 016: Freshservice fixture maps business hours to calendar ref.
- Test 017: Property test proves recompute never deletes breach evidence.
- Test 018: Replay test proves identical event sequence produces identical clock digest.
- Test 019: Rollback test compensates notification task while preserving breach event.
- Test 020: Contract test verifies breach detection response includes audit_event_id and remediation_id.

## Failure handling
- Failure 001: Missing SLA policy blocks clock start.
- Failure 002: Missing target object blocks clock start.
- Failure 003: Calendar resolver failure blocks breach calculation and emits remediation hint.
- Failure 004: Pause policy denial emits denial event and keeps clock running.
- Failure 005: Breach detector storage failure emits failed evidence.
- Failure 006: Remediation owner selection failure escalates to tenant admin policy path.
- Failure 007: Remediation workflow dispatch failure records pending-remediation state.
- Failure 008: Recompute drift creates recompute review task rather than rewriting clock.
- Failure 009: Export authorization denial returns policy_decision_id and audit event.
- Failure 010: Rollback failure preserves original breach and remediation evidence.

## Acceptance criteria
- Acceptance 001: An intern can implement SLA policy, target, clock, breach, pause, recompute, remediation, appeal, export, and rollback types.
- Acceptance 002: An intern can implement clock state validation and breach detection.
- Acceptance 003: An intern can explain how ServiceNow, Jira, BMC, Ivanti, and Freshservice SLA features are displaced.
- Acceptance 004: An intern can implement recompute without deleting sealed breach evidence.
- Acceptance 005: An intern can implement remediation workflow selection, owner selection, escalation, appeal, and export.
- Acceptance 006: An intern can implement benchmark fixtures and property tests.
- Acceptance 007: An intern can implement REST, AsyncAPI, SDK, metrics, dashboard, runbook, replay, and rollback evidence.
- Acceptance 008: An intern can prove pack overlays can narrow or add targets but cannot silently lengthen active targets.
- Acceptance 009: An intern can avoid touching manifests, journeys, ERP, ADR-0321, and other B2B leader services.
- Acceptance 010: An intern can produce a PR that keeps SLA evidence tenant-scoped, reversible, and audit-sealed.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 defines the intern-buildability bar.
- Citation 002: microservices/itsm/manifest.json defines benchmark roster, audience type, compliance packs, and layer conformance.
- Citation 003: microservices/itsm/PRD.md defines incident, change, service request, and service catalog contexts affected by SLA behavior.
- Citation 004: microservices/itsm/runbooks/local-sla-breach-recompute-stall.md anchors recompute remediation.
- Citation 005: microservices/itsm/runbooks/sla-breach-recompute.md anchors breach recompute operations.
- Citation 006: microservices/itsm/slos/local-sla-breach-detection.openslo.yaml anchors breach detection SLOs.
- Citation 007: ADR-0105 defines layer boundaries for SLA domain, usecase, worker, REST, adapter, and governance code.
- Citation 008: ADR-0244 defines default-deny policy expectations for pause, recompute, export, and appeal.
- Citation 009: ADR-0246 defines reusable library expectations for SLA clock and remediation logic.
- Citation 010: ADR-0258 defines contract versioning for SLA APIs and events.
- Citation 011: ADR-0263 defines audit-chain event discipline for clock, breach, recompute, export, and rollback.
- Citation 012: ADR-0316 prevents vendor SLA product labels from becoming service boundaries.
- Citation 013: ADR-0321 defines B2B leader parity expectations for ITSM SLA depth.

## Detailed build checklist
- Build checklist 001: Add fixture `servicenow_task_sla_breach_valid.json`.
- Build checklist 002: Add fixture `servicenow_task_sla_retroactive_rewrite_denied.json`.
- Build checklist 003: Add fixture `jira_sla_goal_calendar_valid.json`.
- Build checklist 004: Add fixture `jira_sla_pause_missing_reason_denied.json`.
- Build checklist 005: Add fixture `bmc_service_target_valid.json`.
- Build checklist 006: Add fixture `bmc_measurement_recompute_preserves_breach.json`.
- Build checklist 007: Add fixture `ivanti_service_level_reassignment_suggestion.json`.
- Build checklist 008: Add fixture `ivanti_automation_without_policy_denied.json`.
- Build checklist 009: Add fixture `freshservice_business_hours_valid.json`.
- Build checklist 010: Add fixture `freshservice_policy_lengthen_active_target_denied.json`.
- Build checklist 011: Add canonicalen clock digest for response target met.
- Build checklist 012: Add canonicalen clock digest for resolution target breached.
- Build checklist 013: Add canonicalen clock digest for pause and resume interval.
- Build checklist 014: Add canonicalen clock digest for recompute preserving prior breach.
- Build checklist 015: Add canonicalen remediation record for owner-selected workflow.
- Build checklist 016: Add canonicalen export record for tenant-scoped SLA evidence.
- Build checklist 017: Add OpenAPI example for SLA policy read.
- Build checklist 018: Add OpenAPI example for SLA breach query.
- Build checklist 019: Add OpenAPI example for remediation task status.
- Build checklist 020: Add OpenAPI example for recompute request.
- Build checklist 021: Add OpenAPI example for appeal decision.
- Build checklist 022: Add AsyncAPI fixture for `itsm.sla.clock_started.v1`.
- Build checklist 023: Add AsyncAPI fixture for `itsm.sla.clock_paused.v1`.
- Build checklist 024: Add AsyncAPI fixture for `itsm.sla.clock_resumed.v1`.
- Build checklist 025: Add AsyncAPI fixture for `itsm.sla.breach_detected.v1`.
- Build checklist 026: Add AsyncAPI fixture for `itsm.sla.remediation_completed.v1`.
- Build checklist 027: Add AsyncAPI fixture for `itsm.sla.recompute_completed.v1`.
- Build checklist 028: Add AsyncAPI fixture for `itsm.sla.appeal_decided.v1`.
- Build checklist 029: Add AsyncAPI fixture for `itsm.sla.export_completed.v1`.
- Build checklist 030: Add Grafana panel query for breach detection lag.
- Build checklist 031: Add Grafana panel query for remediation completion latency.
- Build checklist 032: Add Grafana panel query for recompute queue depth.
- Build checklist 033: Add Grafana panel query for pause reason distribution.
- Build checklist 034: Add runbook pointer for local SLA breach recompute stall.
- Build checklist 035: Add runbook pointer for SLA breach recompute.
- Build checklist 036: Add replay test for imported ServiceNow and Jira SLA timelines.
- Build checklist 037: Add rollback test for erroneous notification compensation.
- Build checklist 038: Add final verification command for line count and citation density.
- Build checklist 039: Add PR summary line naming this as net-new IP-030.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-030-sla-breach-remediation-loop.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/itsm/IP-030-sla-breach-remediation-loop.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
