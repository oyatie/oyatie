# IP-026 Incident Management pagerduty-event-orchestration-displacement

Service: incident-management
ChangeSet scope: microservices/incident-management/IP-026-pagerduty-event-orchestration-displacement.md
Benchmark displacement: PagerDuty event orchestration, services, escalation, runbook automation, status pages
Primary sources: ADR-0321 D-084; PRD-incident-management sections A-F; competitor-parity-matrix.md; ADR-0169
Related IPs: IP-001, IP-002, IP-005, IP-006, IP-009, IP-016, IP-021
Non-goals: no PagerDuty-named service boundary; no manifest change; no journey edit
Acceptance floor: >=200 lines; every numbered row cites at least one local authority

## Objective
- PD-OBJ-001: Displace PagerDuty trigger-incident by routing alert ingress into page-dispatch with tenant_id first [ADR-0321 D-084; PRD FR-001].
- PD-OBJ-002: Displace PagerDuty acknowledge by binding ack to principal_id, responder audience, and schedule version [ADR-0321 D-084; IP-001].
- PD-OBJ-003: Displace PagerDuty escalate by evaluating escalation_policy_version before any comms rail fanout [ADR-0321 D-084; PRD section F].
- PD-OBJ-004: Displace PagerDuty resolve by requiring incident_timeline_id, stakeholder status, and audit closeout [ADR-0321 D-084; PRD section E].
- PD-OBJ-005: Displace PagerDuty snooze by requiring duration, reason, tenant policy, and refusal evidence on overrun [ADR-0321 D-084; IP-002].
- PD-OBJ-006: Displace PagerDuty configure-escalation-policy by importing rules into tenant-owned policy graphs [ADR-0321 D-084; competitor-parity].
- PD-OBJ-007: Displace PagerDuty configure-on-call-schedule by preserving rotations, overrides, holidays, and gap checks [ADR-0321 D-084; PRD].
- PD-OBJ-008: Displace PagerDuty configure-service by projecting service ownership without vendor service ids as authority [ADR-0321 D-084; ARCHITECTURE.md].
- PD-OBJ-009: Displace PagerDuty run-runbook-automation by adding Cedar approval gates for destructive actions [ADR-0321 D-084; IP-002].
- PD-OBJ-010: Displace PagerDuty run-event-orchestration by making event_rule_id tenant-local and replay-safe [ADR-0321 D-084; ADR-0169].
- PD-OBJ-011: Displace PagerDuty trigger-via-event-rules by requiring dedup_key plus idempotency_key [ADR-0169; IP-001].
- PD-OBJ-012: Displace PagerDuty run-status-update by mapping public messages to ADR-0168-compatible component payloads [ADR-0168; ADR-0321 D-084].
- PD-OBJ-013: Displace PagerDuty run-postmortem by sealing action items through incident-management postmortem evidence [ADR-0321 D-084; PRD].
- PD-OBJ-014: Displace PagerDuty status pages by adopting Statuspage-compatible schema while keeping tenant authority local [ADR-0168; PRD].
- PD-OBJ-015: Displace PagerDuty AIOps handoff by emitting clean intelligence context without moving canonical incident ownership [ADR-0321 D-084; PRD].
- PD-OBJ-016: Displace PagerDuty automation by keeping workflow-engine as template runner and incident-management as state owner [ADR-0321 D-084; PRD section F].
- PD-OBJ-017: Displace PagerDuty subscription lock-in by requiring rollback bundle before vendor subscription sunset [ADR-0321 D-084; backfill-replay.md].
- PD-OBJ-018: Displace PagerDuty hidden state by making every import, transform, command, and rollback auditable [PRD section E; AUDIT-FINDINGS].

## Source export intake
- PD-INTAKE-001: Export services with source_service_id, integration keys, owning team, and dependency graph [ADR-0321 D-084; PRD].
- PD-INTAKE-002: Export escalation policies with levels, targets, delays, repeat rules, and cycle candidates [ADR-0321 D-084; IP-002].
- PD-INTAKE-003: Export on-call schedules with layers, rotations, overrides, holidays, and timezone data [ADR-0321 D-084; IP-001].
- PD-INTAKE-004: Export teams and users only as identity-mapping inputs, not authorization principals [ADR-0321 D-084; IP-002].
- PD-INTAKE-005: Export event rules and orchestration paths with source dedup semantics [ADR-0321 D-084; ADR-0169].
- PD-INTAKE-006: Export runbook automation scripts with risk class and external credential references [ADR-0321 D-084; IP-009].
- PD-INTAKE-007: Export status pages with component ids, subscribers, incident history, and visibility [ADR-0321 D-084; ADR-0168].
- PD-INTAKE-008: Export postmortem templates and historical postmortems as migration annotations [ADR-0321 D-084; PRD].
- PD-INTAKE-009: Export maintenance windows as incident-management suppression windows with explicit tenant permits [ADR-0321 D-084; IP-002].
- PD-INTAKE-010: Export service dependencies as ontology projections, not as vendor graph authority [ADR-0321 D-084; IP-003].
- PD-INTAKE-011: Export integration targets for Splunk, Datadog, Prometheus, and NewRelic as credential-sidecar candidates [ADR-0321 D-084; IP-009].
- PD-INTAKE-012: Export notification rules as comms rail preferences, not as incident policy [ADR-0321 D-084; PRD].
- PD-INTAKE-013: Export alert payload examples into replay fixtures for IP-016 backfill validation [ADR-0321 D-084; IP-016].
- PD-INTAKE-014: Export audit logs where available and mark gaps in dpia.md evidence [PRD section E; dpia.md].
- PD-INTAKE-015: Export webhook delivery settings and translate retry expectations through ADR-0169 [ADR-0169; ADR-0321 D-084].
- PD-INTAKE-016: Export source account plan metadata only for migration completeness, not pricing authority [ADR-0314; PRD section J].
- PD-INTAKE-017: Export source ids with checksums before transform so rollback can prove exact lineage [IP-016; AUDIT-FINDINGS].
- PD-INTAKE-018: Export tenant cutover intent as an approval artifact before any command promotion [ADR-0321 D-084; PRD].

## Transform and ontology
- PD-ONTO-001: Map PagerDuty incident to incident.incident with tenant_id and incident_timeline_id [ADR-0321 D-084; IP-003].
- PD-ONTO-002: Map PagerDuty service to incident.service plus service-catalog reference without vendor namespace [ADR-0321 D-084; IP-020].
- PD-ONTO-003: Map PagerDuty escalation policy to incident.escalation_policy with graph checksum [ADR-0321 D-084; IP-002].
- PD-ONTO-004: Map PagerDuty on-call schedule to incident.on_call_schedule with layer checksums [ADR-0321 D-084; IP-001].
- PD-ONTO-005: Map PagerDuty team to incident.team only after identity mapping approval [ADR-0321 D-084; IP-002].
- PD-ONTO-006: Map PagerDuty user to incident.user alias, never direct principal authority [ADR-0321 D-084; IP-001].
- PD-ONTO-007: Map PagerDuty event orchestration to incident.event_orchestration with replay safety [ADR-0321 D-084; ADR-0169].
- PD-ONTO-008: Map PagerDuty event rule to incident.event_rule with source payload filter hash [ADR-0321 D-084; IP-006].
- PD-ONTO-009: Map PagerDuty runbook automation to incident.runbook_automation with risk class [ADR-0321 D-084; IP-004].
- PD-ONTO-010: Map PagerDuty postmortem to incident.postmortem with sealed revision model [ADR-0321 D-084; PRD].
- PD-ONTO-011: Map PagerDuty maintenance window to incident.maintenance_window with suppression audit [ADR-0321 D-084; IP-011].
- PD-ONTO-012: Map PagerDuty status page to incident.status_page and ADR-0168 component shape [ADR-0321 D-084; ADR-0168].
- PD-ONTO-013: Map integration key to credential-sidecar secret reference, never cleartext config [ADR-0321 D-084; IP-009].
- PD-ONTO-014: Map dedup key to idempotency envelope for webhook retry safety [ADR-0169; IP-005].
- PD-ONTO-015: Map severity to tenant severity policy, not source vendor enum alone [PRD section C; IP-001].
- PD-ONTO-016: Map urgency to escalation evaluation input with tenant policy version [ADR-0321 D-084; IP-002].
- PD-ONTO-017: Map public incident impact to ADR-0168 status enum compatibility [ADR-0168; IP-030].
- PD-ONTO-018: Map migration provenance to audit-chain evidence bundle [PRD section E; IP-023].

## Command contracts
- PD-CMD-001: POST page-dispatch requires tenant_id, principal_id, service_id, dedup_key, and idempotency_key [IP-005; ADR-0169].
- PD-CMD-002: POST acknowledge requires incident_id, page_id, responder principal, and schedule_version [IP-005; IP-001].
- PD-CMD-003: POST escalate requires escalation_policy_id, policy_version, and reason [IP-005; ADR-0321 D-084].
- PD-CMD-004: POST resolve requires incident_timeline_id, resolution state, and stakeholder disclosure state [IP-005; PRD].
- PD-CMD-005: POST snooze requires duration, reason, and automatic expiry audit [IP-005; IP-011].
- PD-CMD-006: PUT escalation-policy requires graph checksum and cycle-check evidence [IP-005; IP-002].
- PD-CMD-007: PUT on-call-schedule requires rotation checksum and no-gap evidence [IP-005; IP-001].
- PD-CMD-008: PUT service requires tenant service ownership and catalog reference [IP-005; IP-020].
- PD-CMD-009: POST runbook-automation requires runbook id, risk class, approval id, and rollback command [IP-005; IP-004].
- PD-CMD-010: POST event-orchestration requires event_rule_id, dedup key, and credential-sidecar reference [IP-005; IP-009].
- PD-CMD-011: POST status-update requires ADR-0168 component, impact, summary, and disclosure scope [IP-005; ADR-0168].
- PD-CMD-012: POST postmortem-seal requires evidence bundle, action item owners, and retention policy [IP-005; PRD].
- PD-CMD-013: POST import-preview requires source checksum and transform version [IP-016; PRD].
- PD-CMD-014: POST import-approve requires tenant approver and rollback bundle id [IP-016; IP-001].
- PD-CMD-015: POST shadow-route requires cutover phase and dual-route window [ADR-0321 D-084; IP-016].
- PD-CMD-016: POST cutover requires passing SLO, audit, and rollback checks [IP-021; IP-025].
- PD-CMD-017: POST vendor-sunset requires subscription sunset evidence and no active dual-route dependency [ADR-0321 D-084; IP-016].
- PD-CMD-018: GET evidence-bundle returns tenant-readable provenance without vendor credentials [IP-023; IP-009].

## Async events
- PD-EVT-001: page.dispatched publishes tenant_id, page_id, incident_id, service_id, and escalation_policy_version [IP-006; IP-001].
- PD-EVT-002: page.acknowledged publishes responder principal, ack latency, and schedule_version [IP-006; local-page-to-acknowledge SLO].
- PD-EVT-003: escalation.evaluated publishes policy graph checksum and selected targets [IP-006; IP-002].
- PD-EVT-004: incident.resolved publishes resolution reason, timeline checksum, and postmortem requirement [IP-006; PRD].
- PD-EVT-005: schedule.imported publishes source checksum, rotation checksum, and gap result [IP-006; IP-016].
- PD-EVT-006: policy.imported publishes graph checksum, cycle result, and rollback bundle [IP-006; IP-016].
- PD-EVT-007: event.rule.matched publishes dedup key, event_rule_id, and replay flag [IP-006; ADR-0169].
- PD-EVT-008: runbook.automation.requested publishes risk class, approval id, and dry-run result [IP-006; IP-004].
- PD-EVT-009: statuspage.component.synced publishes ADR-0168 component id and freshness SLO dimensions [IP-006; ADR-0168].
- PD-EVT-010: stakeholder.update.published publishes disclosure audience and supersession id [IP-006; IP-030].
- PD-EVT-011: postmortem.sealed publishes evidence bundle checksum and revision id [IP-006; PRD].
- PD-EVT-012: import.previewed publishes source_vendor=PagerDuty and transform version [IP-016; ADR-0321 D-084].
- PD-EVT-013: import.approved publishes approver principal and rollback bundle id [IP-016; IP-001].
- PD-EVT-014: shadow.route.started publishes dual-route window and target services [ADR-0321 D-084; IP-016].
- PD-EVT-015: cutover.completed publishes command ids and source deactivation checklist [ADR-0321 D-084; IP-021].
- PD-EVT-016: vendor.sunset.recorded publishes subscription sunset evidence without billing mutation [ADR-0321 D-084; PRD section J].
- PD-EVT-017: permit.refused publishes policy id, reason, and trace id [IP-002; IP-011].
- PD-EVT-018: rollback.executed publishes compensating command and before/after checksum [IP-016; IP-025].

## Cedar and policy gates
- PD-CEDAR-001: trigger-incident permit requires tenant match and event_rule_id ownership [ADR-0321 D-084; IP-002].
- PD-CEDAR-002: acknowledge permit requires current responder or delegated backup [ADR-0321 D-084; IP-002].
- PD-CEDAR-003: escalate permit requires policy version freshness and non-cyclic graph [ADR-0321 D-084; IP-002].
- PD-CEDAR-004: resolve permit requires open timeline and required stakeholder update state [ADR-0321 D-084; IP-002].
- PD-CEDAR-005: snooze permit requires bounded duration and audit reason [ADR-0321 D-084; IP-002].
- PD-CEDAR-006: configure-escalation-policy permit requires admin audience and graph validation [ADR-0321 D-084; IP-002].
- PD-CEDAR-007: configure-on-call-schedule permit requires admin audience and no-gap preview [ADR-0321 D-084; IP-002].
- PD-CEDAR-008: configure-service permit requires service owner and catalog match [ADR-0321 D-084; IP-020].
- PD-CEDAR-009: run-runbook-automation permit requires risk-class approval [ADR-0321 D-084; IP-004].
- PD-CEDAR-010: run-event-orchestration permit requires credential-sidecar reference and replay safety [ADR-0321 D-084; IP-009].
- PD-CEDAR-011: trigger-via-event-rules permit requires dedup and idempotency proof [ADR-0169; IP-002].
- PD-CEDAR-012: run-status-update permit requires public component tenant ownership [ADR-0168; IP-002].
- PD-CEDAR-013: run-postmortem permit requires complete action item ownership [ADR-0321 D-084; IP-002].
- PD-CEDAR-014: import-preview permit requires source checksum and no live page side effect [IP-016; IP-002].
- PD-CEDAR-015: import-approve permit requires tenant approver and rollback bundle [IP-016; IP-002].
- PD-CEDAR-016: shadow-route permit requires dual-route window and SLO budget [ADR-0321 D-084; IP-021].
- PD-CEDAR-017: cutover permit requires promotion gate success [IP-021; IP-025].
- PD-CEDAR-018: vendor-sunset permit requires no active source dependency [ADR-0321 D-084; IP-025].

## Cutover and shadow run
- PD-CUT-001: Run import-preview for every service, policy, schedule, event rule, runbook, postmortem, and status page [ADR-0321 D-084; IP-016].
- PD-CUT-002: Run transform validation before any live command [IP-016; PRD section F].
- PD-CUT-003: Run identity mapping review for users and teams [ADR-0321 D-084; IP-002].
- PD-CUT-004: Run credential-sidecar binding for every integration key [ADR-0321 D-084; IP-009].
- PD-CUT-005: Run replay fixtures with live paging disabled [ADR-0169; IP-016].
- PD-CUT-006: Run shadow service for 14 days before cutover [ADR-0321 D-084; IP-021].
- PD-CUT-007: Run dual-route with source and Oyatie outcomes compared by dedup key [ADR-0321 D-084; IP-011].
- PD-CUT-008: Run responder acknowledgement drills against local page-to-acknowledge SLO [IP-021; slos/local-page-to-acknowledge.openslo.yaml].
- PD-CUT-009: Run incident-room creation drills against local war-room creation SLO [IP-021; slos/local-war-room-creation-latency.openslo.yaml].
- PD-CUT-010: Run statuspage sync drills against ADR-0168 component output [ADR-0168; IP-021].
- PD-CUT-011: Run stakeholder update disclosure tests before public subscriber fanout [IP-002; IP-030].
- PD-CUT-012: Run postmortem seal tests before subscription sunset [PRD; IP-025].
- PD-CUT-013: Run rollback simulation for schedule, policy, runbook, and statuspage objects [IP-016; IP-025].
- PD-CUT-014: Run audit completeness check across every migration command [IP-011; AUDIT-FINDINGS].
- PD-CUT-015: Run cost budget check for dual-route paging [IP-017; ADR-0321 D-084].
- PD-CUT-016: Run capacity admission check before high-volume imports [IP-018; IP-016].
- PD-CUT-017: Run promotion only after SLO gate and closeout evidence pass [IP-021; IP-025].
- PD-CUT-018: Run vendor-sunset only after rollback bundle remains valid [ADR-0321 D-084; IP-016].

## Observability and SLOs
- PD-OBS-001: Emit page dispatch metric with tenant, cell, source_vendor, and service dimensions [IP-011; PRD section E].
- PD-OBS-002: Emit ack latency metric linked to local-page-to-acknowledge SLO [IP-021; slos/local-page-to-acknowledge.openslo.yaml].
- PD-OBS-003: Emit war-room creation metric linked to local-war-room-creation-latency SLO [IP-021; slos/local-war-room-creation-latency.openslo.yaml].
- PD-OBS-004: Emit postmortem seal metric linked to local-postmortem-seal-completeness SLO [IP-021; slos/local-postmortem-seal-completeness.openslo.yaml].
- PD-OBS-005: Emit stakeholder update metric linked to local-stakeholder-update-latency SLO [IP-021; slos/local-stakeholder-update-latency.openslo.yaml].
- PD-OBS-006: Emit statuspage sync metric linked to local-statuspage-sync-freshness SLO [IP-021; slos/local-statuspage-sync-freshness.openslo.yaml].
- PD-OBS-007: Emit import throughput metric with source checksum counts [IP-016; IP-011].
- PD-OBS-008: Emit replay freshness metric for backfill safety [IP-016; slos/replay-freshness.openslo.yaml].
- PD-OBS-009: Emit permit decision metric with allow/refuse reason [IP-002; dashboards/local-policy-decisions.json].
- PD-OBS-010: Emit audit emission lag metric for closeout evidence [IP-011; slos/audit-emission-lag.openslo.yaml].
- PD-OBS-011: Emit paging storm metric for abuse defense and capacity admission [IP-012; IP-018].
- PD-OBS-012: Emit dual-route comparison metric during shadow run [ADR-0321 D-084; IP-021].
- PD-OBS-013: Emit source credential rotation metric through IP-009 [IP-009; IP-011].
- PD-OBS-014: Emit runbook risk-class metric before automation execution [IP-004; IP-011].
- PD-OBS-015: Emit status subscriber fanout metric for ADR-0168 compatibility [ADR-0168; IP-017].
- PD-OBS-016: Emit rollback readiness metric until vendor sunset [IP-016; IP-025].
- PD-OBS-017: Emit source-vendor label only for migration windows and remove it from steady-state dashboards after sunset [ADR-0321 D-084; IP-017].
- PD-OBS-018: Emit evidence packet completeness metric for auditor review [IP-023; PRD section E].

## Rollback
- PD-RB-001: Roll back imported service by restoring previous service ownership and integration routing [ADR-0321 D-084; IP-016].
- PD-RB-002: Roll back escalation policy by restoring prior graph checksum [IP-002; IP-016].
- PD-RB-003: Roll back on-call schedule by restoring rotation layer checksum [IP-001; IP-016].
- PD-RB-004: Roll back event rule by disabling local rule and preserving source rule metadata [ADR-0169; IP-016].
- PD-RB-005: Roll back runbook automation by revoking credential-sidecar grant [IP-009; IP-016].
- PD-RB-006: Roll back statuspage component by publishing supersession, not deleting public history [ADR-0168; IP-030].
- PD-RB-007: Roll back stakeholder update by publishing correction with tenant disclosure scope [IP-030; IP-011].
- PD-RB-008: Roll back postmortem seal by creating a new revision [PRD; IP-025].
- PD-RB-009: Roll back shadow route by stopping local fanout while preserving comparison evidence [ADR-0321 D-084; IP-021].
- PD-RB-010: Roll back cutover by re-enabling source route only with explicit permit [IP-002; IP-016].
- PD-RB-011: Roll back vendor sunset by refusing if source subscription already ended [ADR-0321 D-084; IP-025].
- PD-RB-012: Roll back credential migration by rotating local secret and quarantining source token [IP-009; IP-024].
- PD-RB-013: Roll back import batch by marking commands compensated in audit-chain [IP-011; IP-016].
- PD-RB-014: Roll back replay batch by deleting no history and appending compensation records [IP-016; PRD].
- PD-RB-015: Roll back public subscriber state by disabling fanout and retaining opt-in evidence [ADR-0168; IP-030].
- PD-RB-016: Roll back SLO promotion by returning to shadow-run phase [IP-021; IP-025].
- PD-RB-017: Roll back marketplace automation by preserving ADR-0314 DealSet settlement evidence [ADR-0314; IP-014].
- PD-RB-018: Roll back final claim by keeping this IP as implementation-plan scope, not service completion proof [PRD; AUDIT-FINDINGS].

## Acceptance evidence
- PD-ACCEPT-001: Line-count gate passes at >=200 lines for this IP [task constraint; wc evidence].
- PD-ACCEPT-002: Citation-density gate counts local authority references on numbered rows [task constraint; grep evidence].
- PD-ACCEPT-003: PagerDuty verbs from ADR-0321 D-084 appear in objectives and Cedar gates [ADR-0321 D-084; rg evidence].
- PD-ACCEPT-004: PagerDuty ontology objects from ADR-0321 D-084 appear in transform rows [ADR-0321 D-084; rg evidence].
- PD-ACCEPT-005: Migration path includes export, credential migration, shadow run, cutover, and sunset [ADR-0321 D-084; rg evidence].
- PD-ACCEPT-006: ADR-0169 retry/idempotency appears in event and command rows [ADR-0169; rg evidence].
- PD-ACCEPT-007: ADR-0168 statuspage compatibility appears in status rows [ADR-0168; rg evidence].
- PD-ACCEPT-008: PRD tenant-scope and audit evidence appear in every section [PRD; rg evidence].
- PD-ACCEPT-009: IP-001 tenant-scope dependency appears in command and policy rows [IP-001; rg evidence].
- PD-ACCEPT-010: IP-002 Cedar dependency appears in default-deny rows [IP-002; rg evidence].
- PD-ACCEPT-011: IP-009 credential-sidecar dependency appears in integration rows [IP-009; rg evidence].
- PD-ACCEPT-012: IP-016 backfill dependency appears in migration rows [IP-016; rg evidence].
- PD-ACCEPT-013: IP-021 SLO gate dependency appears in cutover rows [IP-021; rg evidence].
- PD-ACCEPT-014: IP-025 closeout dependency appears in rollback rows [IP-025; rg evidence].
- PD-ACCEPT-015: No manifests, journeys, ADR files, ERP, or other microservices are edited by this IP [task constraint; git diff evidence].
- PD-ACCEPT-016: No PagerDuty-named service boundary is introduced [ADR-0321 D-084; PRD section J].
- PD-ACCEPT-017: All vendor state becomes tenant-owned commands, events, policies, evidence, or rollback bundles [PRD; competitor-parity].
- PD-ACCEPT-018: Completion report must state exact changed path, line count, citation count, and any remaining thin IPs [task constraint; final evidence].
