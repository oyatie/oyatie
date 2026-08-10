---
id: ADR-IM-001
title: escalation-routing-and-incident-command-state-machine
status: Proposed
date: 2026-05-20
microservice: incident-management
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
decision_owner: incident-management-platform-architecture
---

# ADR-IM-001: Escalation Routing And Incident Command State Machine

## Context

- Architectural pressure: Time-Critical Incident Command Pressure.
- Incident Management owns paging, escalation, incident command, stakeholder communication, status updates, and postmortem evidence.
- The service must convert alerts, user reports, deployment failures, and synthetic checks into the right response workflow without becoming a generic ticketing system.
- Paging is high-risk because missed or duplicate pages directly affect mean time to acknowledge and mean time to restore.
- Incident command is high-risk because ambiguous ownership causes parallel, conflicting mitigation work.
- Stakeholder communication is high-risk because stale or inconsistent updates erode trust during active outages.
- Postmortem evidence is high-risk because investigations need exact timeline, role, decision, and remediation commitments.
- Named constraint: Page-To-Ack Constraint.
- The Page-To-Ack Constraint requires acknowledged ownership within 5 minutes for sev1 and sev2 incidents unless an escalation policy has an explicit lower threshold.
- Named constraint: Single Incident Commander Constraint.
- The Single Incident Commander Constraint requires exactly one active incident commander for sev1 and sev2 incidents.
- Named constraint: No Silent Suppression Constraint.
- The No Silent Suppression Constraint requires every suppression, dedupe, and correlation decision to preserve an evidence record.
- Named constraint: Escalation Policy Determinism Constraint.
- The Escalation Policy Determinism Constraint requires the same alert fingerprint, service, tenant, and severity to route to the same on-call layer within one policy version.
- Named constraint: Stakeholder Freshness Constraint.
- The Stakeholder Freshness Constraint requires sev1 public or customer-visible incidents to publish an update at least every 15 minutes until resolved.
- Named constraint: Postmortem Seal Constraint.
- The Postmortem Seal Constraint requires a postmortem to bind incident timeline, contributing factors, actions, owners, and review state before closure.
- Named constraint: Cross-Service Event Constraint.
- The Cross-Service Event Constraint requires deployment, observability, support, and status-page integrations to publish events instead of writing incident internals.
- Named constraint: Audit Chain Constraint.
- The Audit Chain Constraint requires route, page, acknowledge, command transfer, status update, mitigation, and postmortem actions to emit audit evidence.
- Current service docs name on-call-schedule, escalation-policy, incident-room, status-update, and postmortem as bounded contexts.
- The benchmark set includes PagerDuty, OpsGenie, xMatters, FireHydrant, and mature SRE incident processes.
- Oyatie needs equivalent operational rigor while preserving internal authorization, tenant isolation, and flat product catalog semantics.
- The service must support manual declaration of incidents and automatic declaration from alert correlation.
- The service must route pages by service ownership, skill matrix, geography, time window, severity, and tenant support entitlement.
- The service must avoid alert storms overwhelming responders during correlated outages.
- The service must keep an append-only timeline for incident review and legal or customer communication.
- The service must be usable during degraded platform conditions, so core state transitions need simple deterministic storage and low dependency count.
- The service has to distinguish alert ingestion, page dispatch, incident command, stakeholder communication, and postmortem closure.
- This decision defines the routing and command core that other incident features must compose through.

## Decision

- We will implement the Escalation Routing Engine with Incident Command State Machine.
- The named pattern is deterministic routing plus skill-matrix selection plus command-state ledger plus transactional outbox.
- The named technology choice is service-local Postgres for incident state, Valkey lease cache only for short-lived dispatch locks, Cedar for action authorization, and AsyncAPI CloudEvents for integration events.
- AlertFingerprint will canonicalize source, tenant, affected_service, severity, region, symptom, deployment_ref, and dedupe_window.
- EscalationPolicy will define layers, schedules, responder groups, skill requirements, delay intervals, and repeat limits.
- OnCallSchedule will resolve primary, secondary, backup, and manager responders for a given policy_version and time window.
- PageDispatch will track pending, delivered, acknowledged, escalated, failed, suppressed, and expired states.
- IncidentRoom will track detected, triaged, declared, mitigated, resolved, closed, cancelled, and reopened states.
- IncidentRoleAssignment will enforce exactly one active incident commander for sev1 and sev2 incidents.
- StakeholderUpdate will track draft, approved, published, corrected, and retracted states.
- PostmortemSeal will track draft, review_requested, sealed, reopened, and accepted states.
- A sev1 incident must dispatch its first page within p95 30 seconds and p99 90 seconds after declaration.
- A sev2 incident must dispatch its first page within p95 60 seconds and p99 120 seconds after declaration.
- Page acknowledgement must be recorded within 5 minutes for sev1 and sev2 or the next escalation layer is paged automatically.
- War-room creation must complete within p95 60 seconds and p99 180 seconds for sev1 incidents.
- Stakeholder update publication must complete within p95 2 minutes after approval.
- Customer-visible sev1 incidents must have a published update at least every 15 minutes.
- Alert dedupe windows default to 10 minutes for identical fingerprints and 3 minutes for correlated deployment fingerprints.
- Suppression is allowed only when an active incident already owns the fingerprint or a maintenance window policy explicitly matches.
- Suppression must emit an IncidentTimelineEvent with suppression_reason and owning_incident_id.
- Incident commander transfer requires current commander, target commander, reason, and audit event.
- Closing a sev1 or sev2 incident requires a sealed or explicitly waived postmortem.
- Postmortem waiver requires service owner and reliability lead approval.
- Postmortem action items must have owner_principal, due_at, severity, and linked remediation reference.
- The service will publish incident_management.alert_correlated.v1, incident_management.page_dispatched.v1, incident_management.incident_state_changed.v1, incident_management.status_update_published.v1, and incident_management.postmortem_sealed.v1 events.
- Every event must include incident_id when known, traceparent, tenant_id, affected_service, severity, policy_version, and evidence_hash.
- Consumers must not infer incident state from chat rooms or status pages; the IncidentRoom projection is authoritative.

## Alternatives Considered

- Alternative 1: Chat-First Incident Command.
- Pros: Responders already coordinate in chat and can move quickly.
- Pros: Chat channels are flexible and familiar.
- Cons: Chat does not enforce exactly one incident commander.
- Cons: Paging, acknowledgement, and escalation policy evidence becomes fragmented.
- Cons: Postmortem timelines become dependent on channel exports and manual cleanup.
- Rejection reason: Chat is an adapter for collaboration, not the authority for incident state.

- Alternative 2: Generic Ticket Queue For Incidents.
- Pros: Existing ticket concepts are easy for support teams to understand.
- Pros: Assignment and comments are familiar primitives.
- Cons: Tickets do not provide deterministic page routing or escalation timers.
- Cons: Sev1 command roles and stakeholder cadence need stronger invariants.
- Cons: Ticket closure can hide unresolved postmortem obligations.
- Rejection reason: Incident command requires operational state machines beyond ticket lifecycle.

- Alternative 3: External Pager Delegation Only.
- Pros: PagerDuty and OpsGenie provide mature paging and schedules.
- Pros: External routing can reduce implementation load.
- Cons: Oyatie would lose policy evidence, tenant entitlement controls, and command-state ownership.
- Cons: External systems cannot reliably enforce postmortem seal and stakeholder cadence.
- Cons: Customer-specific incident workflows would be split across tools.
- Rejection reason: External pagers can be connectors but not the source of Oyatie incident truth.

- Alternative 4: Alert-Only Correlation Without Incident Declaration.
- Pros: Automated correlation can reduce manual steps.
- Pros: Repeated alerts can be grouped by fingerprint.
- Cons: Responders still need explicit command roles, severity, room, updates, and closure criteria.
- Cons: Stakeholder communication cannot be safely inferred from alert status.
- Cons: Manual customer reports may not map to existing observability alerts.
- Rejection reason: Alerts are inputs; incident declaration is a domain decision.

- Alternative 5: Fully Configurable Workflow Engine.
- Pros: Teams could customize escalation and closure processes extensively.
- Pros: Low-code changes could support varied organizations.
- Cons: Safety invariants like one commander and page-to-ack timers become configurable footguns.
- Cons: Workflow history would fragment incident timeline evidence.
- Cons: Incident command needs predictable operations during outages.
- Rejection reason: Domain-specific state machines should own the safety-critical core.

## Consequences

- Positive consequence: Paging, escalation, incident roles, stakeholder updates, and postmortem closure share one timeline.
- Positive consequence: Sev1 and sev2 command ownership becomes explicit and enforceable.
- Positive consequence: Alert suppression becomes auditable instead of invisible.
- Positive consequence: Status-page and chat integrations can be replaced without losing incident truth.
- Positive consequence: Postmortem closure is connected to incident lifecycle instead of a separate reminder process.
- Positive consequence: Metrics can directly track page dispatch latency, acknowledgement latency, update cadence, and postmortem seal time.
- Positive consequence: Tenant-specific support entitlements can influence routing through policy rather than custom code.
- Negative consequence: Incident Management becomes a dependency during platform incidents and must be hardened accordingly.
- Negative consequence: Routing correctness depends on schedule freshness and skill matrix quality.
- Negative consequence: False correlations can suppress alerts into the wrong active incident if fingerprints are too broad.
- Negative consequence: Strict postmortem sealing can delay formal closure when teams are overloaded.
- Negative consequence: External pager connectors need careful idempotency and callback handling.
- Neutral consequence: Chat, status page, and external pager systems remain adapters attached to service-owned state.
- Neutral consequence: Valkey-backed locks are an optimization for dispatch concurrency, not the source of truth.
- Neutral consequence: Incident timelines will contain both machine and human events.
- Follow-up work: IM-FW-001 will define the alert fingerprint registry and correlation test fixtures.
- Follow-up work: IM-FW-002 will add external pager connector conformance tests.
- Follow-up work: IM-FW-003 will build the incident commander transfer UI contract.
- Follow-up work: IM-FW-004 will add stakeholder update templates by severity and visibility.
- Follow-up work: IM-FW-005 will add postmortem action item sync to planning and work tracking services.

## Implementation Notes

- Data shape: AlertFingerprint.
- AlertFingerprint fields: fingerprint_id, source_system, tenant_id, affected_service, severity, region, symptom_code, deployment_ref.
- AlertFingerprint fields: dedupe_window_seconds, correlation_key, policy_version, first_seen_at, last_seen_at.
- Data shape: EscalationPolicy.
- EscalationPolicy fields: policy_id, tenant_id, affected_service, severity, policy_version, layers, repeat_limit, maintenance_windows.
- EscalationPolicy fields: skill_requirements, entitlement_requirement, default_dedupe_window_seconds, created_at, updated_at.
- Data shape: OnCallSchedule.
- OnCallSchedule fields: schedule_id, team_id, timezone, rotation_version, primary_principal, secondary_principal, manager_principal.
- OnCallSchedule fields: effective_from, effective_to, override_reason, override_principal.
- Data shape: PageDispatch.
- PageDispatch fields: page_id, incident_id, fingerprint_id, policy_id, layer_index, target_principal, channel.
- PageDispatch fields: dispatch_state, dispatched_at, delivered_at, acknowledged_at, escalated_at, failed_reason.
- Data shape: IncidentRoom.
- IncidentRoom fields: incident_id, tenant_id, affected_service, severity, customer_visible, incident_state, commander_principal.
- IncidentRoom fields: room_ref, status_page_ref, declared_at, mitigated_at, resolved_at, closed_at, reopened_at.
- Data shape: IncidentTimelineEvent.
- IncidentTimelineEvent fields: timeline_event_id, incident_id, event_kind, actor_principal, event_payload_hash, occurred_at, traceparent.
- Data shape: StakeholderUpdate.
- StakeholderUpdate fields: update_id, incident_id, audience, update_state, summary, impact, next_update_due_at, published_at.
- Data shape: PostmortemSeal.
- PostmortemSeal fields: postmortem_id, incident_id, seal_state, contributing_factors_hash, action_items_hash, reviewer_principal, sealed_at.
- API endpoint: POST /v1/incident-management/events ingests alerts, deployment failures, synthetic checks, and manual reports.
- API endpoint: POST /v1/incident-management/incidents declares or updates an incident room.
- API endpoint: POST /v1/incident-management/pages dispatches a page through resolved escalation policy.
- API endpoint: POST /v1/incident-management/pages/{page_id}/ack records acknowledgement by the target or delegate.
- API endpoint: POST /v1/incident-management/incidents/{incident_id}/commander transfers command.
- API endpoint: POST /v1/incident-management/incidents/{incident_id}/mitigate records mitigation.
- API endpoint: POST /v1/incident-management/incidents/{incident_id}/resolve records resolution.
- API endpoint: POST /v1/incident-management/incidents/{incident_id}/status-updates creates stakeholder updates.
- API endpoint: POST /v1/incident-management/status-updates/{update_id}/publish publishes approved updates.
- API endpoint: POST /v1/incident-management/postmortems/{postmortem_id}/seal seals postmortem evidence.
- Event: incident_management.alert_correlated.v1 records new, deduped, suppressed, and attached alert decisions.
- Event: incident_management.page_dispatched.v1 records page target, channel, layer, and policy version.
- Event: incident_management.incident_state_changed.v1 records declared, mitigated, resolved, closed, reopened, and cancelled states.
- Event: incident_management.status_update_published.v1 records published, corrected, and retracted stakeholder updates.
- Event: incident_management.postmortem_sealed.v1 records seal, reopen, waiver, and acceptance states.
- Cedar policy: sre-incident-command-authorization.cedar permits declaration and commander transfer for service owner, SRE lead, and delegated incident commander roles.
- Cedar policy: local-page-dispatch-guard.cedar permits automated dispatch when escalation policy version is active and maintenance window does not suppress.
- Cedar policy: local-page-acknowledge-scope.cedar permits acknowledgement by target_principal, active delegate, or escalation manager.
- Cedar policy: local-war-room-open-approval.cedar permits war-room creation for sev1, sev2, and customer-visible incidents.
- Cedar policy: local-escalation-policy-control.cedar restricts policy edits to reliability administrators and service owners.
- Cedar policy: local-stakeholder-update-egress.cedar requires approval before public or customer-visible updates.
- Cedar policy: local-postmortem-seal-required.cedar denies closure for sev1 and sev2 without sealed or waived postmortem.
- Cedar policy: auditor-scope.cedar permits read-only timeline and policy evidence for audit principals.
- SLO target: alert ingestion availability is 99.95 percent monthly.
- SLO target: sev1 first page dispatch latency is p95 30 seconds and p99 90 seconds.
- SLO target: sev2 first page dispatch latency is p95 60 seconds and p99 120 seconds.
- SLO target: page acknowledgement timer escalation fires within 5 minutes plus 15 seconds clock skew.
- SLO target: sev1 war-room creation latency is p95 60 seconds and p99 180 seconds.
- SLO target: approved stakeholder update publication latency is p95 2 minutes.
- SLO target: incident state event publication lag is p99 under 3 seconds.
- SLO target: postmortem seal completion for sev1 is p90 within 5 business days.
- Dashboard: incident-management-overview shows incidents by severity, state, service, customer visibility, and commander assignment.
- Dashboard: escalation-routing-health shows page dispatch latency, ack latency, escalation layer depth, and failed channels.
- Dashboard: stakeholder-update-freshness shows next update due, overdue updates, and publication latency.
- Dashboard: postmortem-evidence shows seal state, action item ownership, waiver count, and review age.
- Dashboard: alert-correlation-quality shows dedupe rate, suppression reason, false-correlation reopen rate, and alert storm volume.
- Runbook: missed-page-escalation describes channel fallback, manager escalation, and schedule override.
- Runbook: incident-commander-transfer describes transfer authorization and timeline entry expectations.
- Runbook: stale-stakeholder-update describes update owner paging and correction publication.
- Runbook: postmortem-seal-overdue describes waiver, escalation, and action item export.

## Verification

- Test: sev1_first_page_dispatched_under_threshold declares sev1 and asserts dispatch event scheduled before 30 seconds in test clock.
- Test: page_ack_timeout_escalates_next_layer simulates no ack for 5 minutes and asserts layer two dispatch.
- Test: duplicate_alert_suppressed_with_timeline sends identical fingerprint and asserts suppression evidence.
- Test: deployment_correlated_alert_attaches_to_active_incident validates deployment_ref correlation window.
- Test: one_commander_enforced_for_sev1 attempts second active commander and asserts rejection.
- Test: commander_transfer_requires_authorized_actor asserts Cedar denies unauthorized transfer.
- Test: stakeholder_update_overdue_for_customer_visible_sev1 advances clock and asserts overdue state.
- Test: postmortem_required_before_sev1_close asserts closure denied without sealed or waived postmortem.
- Test: postmortem_waiver_requires_two_roles asserts service owner plus reliability lead approvals.
- Test: external_pager_callback_idempotent submits duplicate callback and asserts one ack transition.
- Test: maintenance_window_suppression_requires_policy_match asserts no silent suppression.
- Test: incident_state_event_contains_policy_version validates AsyncAPI schema for state change.
- Test: audit_event_written_before_page_dispatch validates transaction ordering.
- Test: route_determinism_same_policy_version resolves same target for same fingerprint and time window.
- Metric: incident_alert_ingest_total by source_system, affected_service, severity, and outcome.
- Metric: incident_page_dispatch_latency_seconds by severity, policy_id, channel, and layer_index.
- Metric: incident_page_ack_latency_seconds by severity, target_team, and channel.
- Metric: incident_escalation_depth_total by affected_service and severity.
- Metric: incident_commander_transfer_total by severity and reason_code.
- Metric: incident_stakeholder_update_overdue_total by audience and severity.
- Metric: incident_status_update_publication_seconds by audience and approval_role.
- Metric: incident_postmortem_seal_age_seconds by severity and service_owner.
- Metric: incident_alert_suppression_total by suppression_reason and owning_incident_state.
- Dashboard: Incident Command Safety shows active sev1 commander gaps, ack breaches, and stale status updates.
- Dashboard: Escalation Policy Drift shows schedule overrides, failed channels, and fallback pages.
- Dashboard: Postmortem Closure shows overdue seals, waived postmortems, and action item age.
- Dashboard: Correlation Quality shows suppressed alerts, reopened incidents, and false correlation reviews.
- Alert: IncidentPageDispatchLatencyHigh fires when sev1 p95 dispatch exceeds 30 seconds for 5 minutes.
- Alert: IncidentAckBreach fires when a sev1 or sev2 page exceeds 5 minutes without ack.
- Alert: IncidentCommanderMissing fires when sev1 or sev2 incident lacks active commander for 60 seconds.
- Alert: IncidentStakeholderUpdateOverdue fires when customer-visible sev1 update is overdue.
- Alert: IncidentPostmortemSealOverdue fires when sev1 postmortem is unsealed after 5 business days.
- Promotion gate: run escalation policy determinism tests across schedule time zones and overrides.
- Promotion gate: run external pager connector contract tests with duplicate, delayed, and failed callbacks.
- Promotion gate: run Cedar tests for command, page ack, war-room open, stakeholder egress, and postmortem closure.
- Promotion gate: run load test with 1000 alert events per minute and 100 concurrent incidents.

## References

- Google Site Reliability Engineering, Managing Incidents.
- Google Site Reliability Workbook, Incident Response.
- PagerDuty Event Orchestration and Incident API documentation.
- Atlassian OpsGenie API documentation.
- FireHydrant API and incident lifecycle documentation.
- AWS Health API documentation.
- OpenTelemetry Specification.
- CloudEvents Specification 1.0.2.
- OpenAPI Specification 3.1.0.
- AsyncAPI Specification 3.0.0.
- RFC 9110, HTTP Semantics.
- W3C Trace Context Recommendation.
- ITIL 4, incident management practice guidance.
- NIST SP 800-61, Computer Security Incident Handling Guide.
