---
id: ADR-ITSM-001
title: cmdb-reconciliation-graph-and-change-risk-gate
status: Proposed
date: 2026-05-20
microservice: itsm
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
decision_owner: itsm-platform-architecture
---

# ADR-ITSM-001: CMDB Reconciliation Graph And Change Risk Gate

## Context

- Architectural pressure: Service Management Control Plane Pressure.
- ITSM owns incident tickets, problems, changes, service catalog, configuration items, CMDB-style relationships, SLA clocks, and knowledge publication controls.
- The service must preserve ITIL-compatible workflows without becoming a generic workflow or support-ticket clone.
- Incident, problem, and change records need shared service context because change risk, impact analysis, and root cause depend on configuration relationships.
- A CMDB that accepts arbitrary writes becomes inaccurate quickly, while a CMDB that requires perfect discovery blocks operational progress.
- Change approval must account for service criticality, blackout windows, affected configuration items, related incidents, active problems, and deployment blast radius.
- SLA timers must reflect ticket state transitions and entitlement rules without being recalculated by each consumer.
- Knowledge articles and service catalog items need approval flows because they affect customer and operator behavior.
- Named constraint: CMDB Evidence Constraint.
- The CMDB Evidence Constraint requires every configuration item and relationship to retain source_system, discovery_run_id, confidence, effective_at, and reconciliation_state.
- Named constraint: Change Risk Gate Constraint.
- The Change Risk Gate Constraint requires standard, normal, emergency, and retrospective changes to compute risk before approval.
- Named constraint: SLA Clock Authority Constraint.
- The SLA Clock Authority Constraint requires ITSM to own pause, resume, breach, and remediation timer transitions.
- Named constraint: Incident-Problem Link Constraint.
- The Incident-Problem Link Constraint requires problem records to link affected incident tickets and known-error evidence without closing incidents implicitly.
- Named constraint: Catalog Entitlement Constraint.
- The Catalog Entitlement Constraint requires service request eligibility to evaluate tenant, persona, product license, and service catalog policy.
- Named constraint: Change Freeze Constraint.
- The Change Freeze Constraint requires blackout windows to deny normal change approval unless emergency override evidence is attached.
- Named constraint: Audit Chain Constraint.
- The Audit Chain Constraint requires ticket triage, CMDB relation writes, change approval, SLA recompute, problem link, and knowledge publication to emit audit events.
- Existing service docs name incident-ticket, problem, change, service-request, and configuration-item as bounded contexts.
- Current IP slices include ITIL process normalizer, CMDB reconciliation graph, service catalog entitlement, change freeze risk calculator, and SLA breach remediation loop.
- Benchmark systems include ServiceNow ITSM, Jira Service Management, BMC Remedy, Zendesk Support, and Freshdesk.
- Oyatie needs equivalent service-management depth while keeping policy, evidence, and tenant-region controls in-house.
- The service must integrate with incident-management without duplicating incident command or paging semantics.
- The service must integrate with deployment and asset sources without letting them mutate service-management decisions directly.
- The service must support local and global operating contracts already represented by local-openapi, local-asyncapi, and local operations surfaces.
- The decision must produce concrete data shapes, APIs, Cedar policies, dashboards, and tests.
- The design has to keep CMDB graph relations useful for impact analysis while acknowledging discovery uncertainty.
- The design has to make change failure rate measurable and connected to actual affected services.

## Decision

- We will implement the CMDB Reconciliation Graph with Change Risk Gate.
- The named pattern is typed configuration graph plus reconciliation ledger plus ITIL state machines plus policy-backed risk scoring gate.
- The named technology choice is service-local Postgres for ITIL records, adjacency-list graph tables for CMDB relations, deterministic risk scoring in application code, Cedar for approval and relation-write authorization, and AsyncAPI CloudEvents for state publication.
- ConfigurationItem will represent service, application, host, database, queue, endpoint, runbook, synthetic check, and external dependency records.
- CmdbRelation will represent depends_on, hosted_on, owns, monitors, routes_to, stores_data_in, calls, protected_by, and backs_service relationships.
- ReconciliationRun will import discovery evidence and propose relation changes without immediately overwriting trusted manual relations.
- Relation confidence >= 0.95 from trusted discovery can auto-activate when no conflicting active relation exists.
- Relation confidence >= 0.80 and < 0.95 moves to pending_review.
- Relation confidence < 0.80 is retained as rejected_candidate unless a CMDB steward promotes it.
- IncidentTicket will own new, triaged, assigned, in_progress, pending_customer, pending_vendor, resolved, closed, and reopened states.
- ProblemRecord will own suspected, investigating, known_error, workaround_available, fixed, closed, and reopened states.
- ChangeRequest will own draft, risk_scored, approval_pending, approved, scheduled, implementing, verifying, completed, failed, cancelled, and retrospectively_approved states.
- ChangeRiskScore will compute risk from affected CI criticality, active incidents, related problems, deployment window, recent change failure history, data class, and region pack.
- Standard changes require risk_score <= 30 and matching pre-approved template.
- Normal changes require approval when risk_score > 30 and always require approval when affecting tier_0 or tier_1 services.
- Emergency changes may bypass blackout windows only with emergency_reason, incident_ref, approver_principal, and retrospective_review_due_at.
- Change freeze windows deny normal changes when scheduled_start_at overlaps a freeze period by more than 5 minutes.
- Service request entitlement will evaluate catalog item, tenant license, persona, region, data class, and delegated admin scope.
- SLA clocks will recompute from ticket transitions, priority, entitlement, pause reason, and business calendar.
- SLA breach detection must run within 1 minute p95 and 5 minutes p99 after a relevant ticket transition.
- Ticket triage projection must complete within p95 300 ms for single updates and p99 2 seconds under 100 updates per second.
- CMDB relation freshness must be p95 under 15 minutes from discovery run completion to active or pending_review projection.
- Change risk scoring must complete within p95 500 ms and p99 2 seconds for changes affecting up to 500 configuration items.
- The service will publish itsm.incident_ticket_state_changed.v1, itsm.problem_state_changed.v1, itsm.change_state_changed.v1, itsm.cmdb_relation_changed.v1, itsm.sla_breach_detected.v1, and itsm.catalog_request_state_changed.v1 events.
- Consumers must use ITSM APIs or events rather than reading CMDB graph tables directly.
- Every state transition and relation activation must write ITSMAuditEvent before outbox publication.

## Alternatives Considered

- Alternative 1: Ticket-Centric ITSM Without CMDB Graph.
- Pros: Faster initial incident, problem, and change workflow implementation.
- Pros: Aligns with simple support-ticket mental models.
- Cons: Change risk cannot reliably assess blast radius.
- Cons: Problem analysis lacks affected service and dependency context.
- Cons: SLA and service catalog decisions lose service criticality evidence.
- Rejection reason: Service management needs configuration context to be operationally useful.

- Alternative 2: External CMDB As Source Of Truth.
- Pros: ServiceNow or existing asset systems may already contain configuration data.
- Pros: Discovery tools can keep asset inventory current.
- Cons: External relation semantics vary and may not match Oyatie service ownership.
- Cons: Cedar policy, tenant isolation, and regional evidence would be split.
- Cons: Change risk would depend on external availability and opaque confidence.
- Rejection reason: External systems can feed reconciliation, but Oyatie needs local graph authority.

- Alternative 3: Property Graph Database.
- Pros: Graph traversal and impact queries can be expressive.
- Pros: Existing graph query languages support relationship-heavy data.
- Cons: Adds an operational dependency for the first ITSM graph slice.
- Cons: Transactional coupling to ITIL records, audit events, and outbox becomes harder.
- Cons: Current relation shapes can be served by typed adjacency tables and indexed traversals.
- Rejection reason: Postgres graph tables preserve simplicity while meeting current traversal needs.

- Alternative 4: Fully Manual Change Advisory Board.
- Pros: Human review can catch context missed by automation.
- Pros: Easy to map to legacy ITIL change processes.
- Cons: Low-risk standard changes would be slowed unnecessarily.
- Cons: Emergency changes would still need structured retrospective evidence.
- Cons: Risk scoring metrics and change failure rate analysis would remain subjective.
- Rejection reason: Humans approve policy exceptions; deterministic risk scoring should prepare the gate.

- Alternative 5: Generic Workflow Builder For ITIL Processes.
- Pros: Many organizations customize ITSM lifecycle states.
- Pros: Admins could configure processes without code.
- Cons: Core invariants like SLA authority, change freeze, and problem links would become fragile.
- Cons: Audit and metrics would vary across tenant-specific workflows.
- Cons: The initial product needs clear contracts before customization.
- Rejection reason: ITIL state machines need stable product semantics first.

## Consequences

- Positive consequence: Change approvals can reference affected configuration items and dependency blast radius.
- Positive consequence: CMDB relation uncertainty is explicit through confidence and reconciliation_state.
- Positive consequence: SLA breach detection becomes a service-owned metric instead of a report query.
- Positive consequence: Problem records can link incident patterns and known errors without changing incident lifecycle.
- Positive consequence: Service catalog requests can enforce entitlement and delegated admin boundaries.
- Positive consequence: Change failure rate can be computed from completed and failed changes tied to affected services.
- Positive consequence: External discovery systems become evidence providers rather than unreviewed authorities.
- Negative consequence: CMDB stewards need review workflows for mid-confidence relation candidates.
- Negative consequence: Risk scoring thresholds will require calibration across tenants and service criticality levels.
- Negative consequence: Postgres adjacency traversal may need optimization as graph depth and CI count grow.
- Negative consequence: Strict change freeze policy may slow operational work without good emergency override UX.
- Negative consequence: SLA recomputation errors can affect customer commitments and support operations.
- Neutral consequence: The service adopts ITIL vocabulary while preserving Oyatie's policy and evidence architecture.
- Neutral consequence: External ITSM integrations can be built later as adapters.
- Neutral consequence: Incident Management remains the incident command owner; ITSM owns ticket and problem process.
- Follow-up work: ITSM-FW-001 will define CMDB relation type registry and traversal limits.
- Follow-up work: ITSM-FW-002 will calibrate change risk weights against historical deployment outcomes.
- Follow-up work: ITSM-FW-003 will add service catalog entitlement tests by persona and license tier.
- Follow-up work: ITSM-FW-004 will build SLA recompute backfill and breach repair tools.
- Follow-up work: ITSM-FW-005 will define knowledge publication approval workflows and rollback.

## Implementation Notes

- Data shape: ConfigurationItem.
- ConfigurationItem fields: ci_id, tenant_id, ci_type, name, service_ref, owner_principal, criticality_tier, data_class.
- ConfigurationItem fields: region_pack, lifecycle_state, source_system, source_record_id, confidence, created_at, updated_at.
- Data shape: CmdbRelation.
- CmdbRelation fields: relation_id, tenant_id, source_ci_id, relation_type, target_ci_id, confidence, reconciliation_state.
- CmdbRelation fields: source_system, discovery_run_id, effective_at, expires_at, activated_by, reviewed_at.
- Data shape: ReconciliationRun.
- ReconciliationRun fields: reconciliation_run_id, tenant_id, source_system, started_at, completed_at, discovered_count, activated_count.
- ReconciliationRun fields: pending_review_count, rejected_count, error_count, evidence_hash.
- Data shape: IncidentTicket.
- IncidentTicket fields: ticket_id, tenant_id, requester_principal, affected_service, priority, ticket_state, assignment_group.
- IncidentTicket fields: sla_policy_id, sla_clock_id, related_incident_id, related_problem_id, opened_at, resolved_at, closed_at.
- Data shape: ProblemRecord.
- ProblemRecord fields: problem_id, tenant_id, affected_service, problem_state, known_error_ref, workaround_ref, linked_ticket_count.
- ProblemRecord fields: root_cause_summary_hash, owner_principal, opened_at, fixed_at, closed_at.
- Data shape: ChangeRequest.
- ChangeRequest fields: change_id, tenant_id, change_type, affected_ci_ids, requested_by, implementation_owner, change_state.
- ChangeRequest fields: scheduled_start_at, scheduled_end_at, risk_score, risk_band, freeze_overlap_seconds, rollback_plan_hash.
- Data shape: ChangeRiskScore.
- ChangeRiskScore fields: score_id, change_id, score_value, risk_band, ci_criticality_points, active_incident_points.
- ChangeRiskScore fields: problem_points, freeze_points, data_class_points, recent_failure_points, computed_at.
- Data shape: SlaClock.
- SlaClock fields: sla_clock_id, ticket_id, policy_id, started_at, paused_at, resumed_at, due_at, breached_at, pause_reason.
- API endpoint: POST /v1/itsm/incidents creates or updates incident tickets.
- API endpoint: POST /v1/itsm/problems creates or links problem records.
- API endpoint: POST /v1/itsm/changes creates a change request and computes initial risk.
- API endpoint: POST /v1/itsm/changes/{change_id}/risk recomputes risk after affected CI or schedule changes.
- API endpoint: POST /v1/itsm/changes/{change_id}/approve records approval or denial.
- API endpoint: POST /v1/itsm/changes/{change_id}/implement moves approved change into implementation.
- API endpoint: POST /v1/itsm/cmdb/items creates or updates configuration items through authorized writers.
- API endpoint: POST /v1/itsm/cmdb/relations proposes relation writes or review candidates.
- API endpoint: POST /v1/itsm/cmdb/reconciliation-runs imports discovery evidence.
- API endpoint: POST /v1/itsm/sla/recompute recomputes SLA clocks for bounded ticket sets.
- API endpoint: POST /v1/itsm/service-requests creates catalog-backed requests with entitlement checks.
- Event: itsm.incident_ticket_state_changed.v1 records ticket lifecycle transitions.
- Event: itsm.problem_state_changed.v1 records problem lifecycle and known-error transitions.
- Event: itsm.change_state_changed.v1 records change state, risk band, approval, implementation, and failure.
- Event: itsm.cmdb_relation_changed.v1 records relation activation, review, rejection, expiry, and conflict.
- Event: itsm.sla_breach_detected.v1 records breach clock and remediation loop triggers.
- Event: itsm.catalog_request_state_changed.v1 records service request entitlement, approval, fulfillment, and denial.
- Cedar policy: service-management-authorization.cedar permits ticket, problem, change, and catalog actions by persona and tenant scope.
- Cedar policy: local-cmdb-relation-write.cedar permits trusted discovery adapters to propose and CMDB stewards to activate relations.
- Cedar policy: local-change-approval-window.cedar denies normal change approval during freeze overlap greater than 5 minutes.
- Cedar policy: local-sla-recompute-guard.cedar permits recompute only to service-management automation and support operations admins.
- Cedar policy: local-problem-link-control.cedar restricts problem linking to problem managers and affected service owners.
- Cedar policy: local-knowledge-publish-approval.cedar requires owner and reviewer approval before publishing knowledge articles.
- Cedar policy: local-incident-ticket-scope.cedar restricts ticket reads by requester, support assignment, service owner, and auditor scope.
- Cedar policy: auditor-scope.cedar permits read-only access to state transitions, risk scores, and relation evidence.
- SLO target: incident ticket write availability is 99.95 percent monthly.
- SLO target: ticket triage projection latency is p95 300 ms and p99 2 seconds.
- SLO target: CMDB relation freshness is p95 under 15 minutes from discovery completion.
- SLO target: change risk scoring latency is p95 500 ms and p99 2 seconds for up to 500 CIs.
- SLO target: SLA breach detection runs p95 within 1 minute and p99 within 5 minutes after transition.
- SLO target: change failure rate alerting evaluates every 15 minutes.
- SLO target: audit event completeness is 100 percent for state transitions and relation activation.
- Dashboard: operating-bar-overview shows ticket volume, change risk, SLA health, and CMDB drift.
- Dashboard: local-domain-throughput shows ticket, problem, change, catalog, and CMDB write throughput.
- Dashboard: local-slo-burn shows SLA breach detection, ticket triage latency, and read/write latency.
- Dashboard: local-cmdb-relation-freshness shows discovery-to-projection lag and pending review counts.
- Dashboard: local-change-failure-rate shows failed changes by service, risk band, and freeze override.
- Dashboard: local-problem-link-correctness shows problem links, known errors, and reopened incidents.
- Runbook: local-cmdb-relation-drift describes reviewing relation conflicts and stale discovery sources.
- Runbook: local-change-freeze-override describes emergency override evidence and retrospective review.
- Runbook: local-sla-breach-recompute-stall describes recompute worker recovery and breach replay.
- Runbook: local-problem-link-loop describes loop detection and problem relationship correction.

## Verification

- Test: cmdb_high_confidence_relation_auto_activates creates confidence 0.96 relation and asserts active state.
- Test: cmdb_mid_confidence_relation_pending_review creates confidence 0.85 relation and asserts pending_review.
- Test: cmdb_low_confidence_relation_retained_as_candidate creates confidence 0.60 relation and asserts rejected_candidate.
- Test: relation_conflict_requires_steward_review submits conflicting active relation and asserts no overwrite.
- Test: standard_change_low_risk_auto_approvable creates score 25 with pre-approved template and asserts approval path.
- Test: normal_change_tier1_requires_approval creates tier_1 affected CI and asserts approval_pending.
- Test: freeze_overlap_denies_normal_change schedules within freeze by 10 minutes and asserts Cedar deny.
- Test: emergency_change_requires_incident_ref asserts emergency override denied without incident_ref and approver.
- Test: sla_clock_pauses_and_resumes_from_ticket_state transitions pending_customer and asserts due_at adjustment.
- Test: sla_breach_detected_within_threshold advances test clock and asserts breach event.
- Test: problem_link_does_not_close_incident links problem and asserts incident ticket remains unchanged.
- Test: service_catalog_entitlement_denies_unlicensed_persona asserts Cedar denies catalog request.
- Test: audit_event_before_relation_activation validates transaction ordering.
- Test: change_failure_updates_failure_rate_metric completes failed change and asserts metric label by service.
- Metric: itsm_ticket_triage_latency_seconds by priority, assignment_group, and tenant_id.
- Metric: itsm_cmdb_relation_freshness_seconds by source_system, relation_type, and reconciliation_state.
- Metric: itsm_cmdb_relation_pending_review_total by tenant_id, source_system, and ci_type.
- Metric: itsm_change_risk_score_value by risk_band, change_type, and affected_service.
- Metric: itsm_change_failure_total by risk_band, affected_service, change_type, and freeze_override.
- Metric: itsm_sla_breach_detection_seconds by priority, entitlement, and assignment_group.
- Metric: itsm_sla_breach_total by priority, tenant_id, and breach_reason.
- Metric: itsm_problem_link_total by affected_service and problem_state.
- Metric: itsm_catalog_entitlement_denial_total by catalog_item, persona, and license_tier.
- Metric: itsm_audit_event_missing_total by transition_kind and writer_principal.
- Dashboard: ITSM Service Control shows open tickets, breached SLA, active changes, and CMDB pending review.
- Dashboard: CMDB Reconciliation Quality shows relation freshness, conflict rate, and source health.
- Dashboard: Change Risk Gate shows risk bands, approvals, freeze overrides, and failures.
- Dashboard: SLA Remediation Loop shows breach detection lag, recompute queue, and remediation outcomes.
- Dashboard: Problem Management Quality shows known errors, linked incidents, and reopen rates.
- Alert: ITSMSlaBreachDetectionLagHigh fires when p99 detection exceeds 5 minutes for 10 minutes.
- Alert: ITSMCmdbFreshnessHigh fires when p95 relation freshness exceeds 15 minutes for 15 minutes.
- Alert: ITSMChangeFailureRateHigh fires when failed completed changes exceed 15 percent in a 7-day window.
- Alert: ITSMRelationConflictSpike fires when relation conflicts exceed 5 percent for a source system.
- Alert: ITSMAuditCompletenessBroken fires on any state transition without audit event.
- Promotion gate: run CMDB reconciliation tests with conflicting, stale, and multi-source discovery evidence.
- Promotion gate: run change risk tests across standard, normal, emergency, retrospective, freeze, and tier_0 service cases.
- Promotion gate: run SLA clock tests with pause, resume, breach, reopen, and backfill scenarios.
- Promotion gate: run Cedar tests for ticket scope, CMDB relation write, change approval, SLA recompute, problem link, and knowledge publish.
- Promotion gate: run load test with 100 ticket updates per second, 20 change risk computations per second, and 50 relation updates per second.

## References

- ITIL Foundation, ITIL 4 Edition, incident, problem, change, and service configuration management practices.
- ServiceNow, CMDB and Common Service Data Model documentation.
- Atlassian Jira Service Management REST API documentation.
- BMC Helix ITSM and Remedy documentation.
- Zendesk Support API documentation.
- Freshservice ITSM documentation.
- AWS Systems Manager OpsCenter documentation.
- OpenTelemetry Specification.
- CloudEvents Specification 1.0.2.
- OpenAPI Specification 3.1.0.
- AsyncAPI Specification 3.0.0.
- RFC 9110, HTTP Semantics.
- W3C Trace Context Recommendation.
- NIST SP 800-128, Guide for Security-Focused Configuration Management.
