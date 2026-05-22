# IP-030 Incident Management incident-io-statuspage-stakeholder-displacement

Service: incident-management
ChangeSet scope: microservices/incident-management/IP-030-incident-io-statuspage-stakeholder-displacement.md
Benchmark displacement: incident.io incident command, stakeholder updates, status pages, workflows, retrospectives; Statuspage-compatible subscriber surfaces
Primary sources: ADR-0168; ADR-0321 D-084; PRD-incident-management; competitor-parity-matrix.md; ADR-0003
Related IPs: IP-001, IP-002, IP-005, IP-006, IP-011, IP-016, IP-021
Non-goals: no community/statuspage service edit; no manifest edit; no journey edit; no incident.io boundary
Acceptance floor: >=200 lines; numbered rows cite local authority

## Objective
- IO-OBJ-001: Displace incident.io incident creation through tenant-scoped incident-room-open [ADR-0321 D-084; PRD].
- IO-OBJ-002: Displace incident.io roles with identity-mapped commander, liaison, and responder principals [IP-002; PRD].
- IO-OBJ-003: Displace incident.io workflows with approved workflow templates [IP-004; PRD].
- IO-OBJ-004: Displace incident.io custom fields with ontology-projected metadata [IP-003; PRD].
- IO-OBJ-005: Displace incident.io announcements with disclosure-scoped stakeholder updates [PRD; IP-002].
- IO-OBJ-006: Displace incident.io status page publishing with ADR-0168-compatible components [ADR-0168; PRD].
- IO-OBJ-007: Displace Atlassian Statuspage subscriber APIs with local webhook, email, and RSS surfaces [ADR-0168; PRD].
- IO-OBJ-008: Displace Statuspage summary JSON with local summary endpoint shape [ADR-0168; PRD].
- IO-OBJ-009: Displace Statuspage component JSON with tenant component namespace [ADR-0168; IP-001].
- IO-OBJ-010: Displace Statuspage incident history with audit-chain backed public history [ADR-0168; ADR-0003].
- IO-OBJ-011: Displace Statuspage RSS with local per-product RSS feeds [ADR-0168; PRD].
- IO-OBJ-012: Displace Statuspage webhook subscribers with tenant-owned signed delivery [ADR-0168; ADR-0169].
- IO-OBJ-013: Displace incident.io retrospectives with postmortem-seal [PRD; IP-025].
- IO-OBJ-014: Displace incident.io follow-ups with action item owner gates [PRD; IP-002].
- IO-OBJ-015: Displace incident.io integrations with credential-sidecar references [IP-009; PRD].
- IO-OBJ-016: Displace public/private confusion with explicit disclosure policy [IP-002; PRD].
- IO-OBJ-017: Displace vendor subscriber lock-in with exportable tenant subscriber lists [ADR-0168; IP-016].
- IO-OBJ-018: Displace source sunset uncertainty with rollback and supersession evidence [IP-016; IP-025].

## Source export intake
- IO-INTAKE-001: Export incidents with source id, severity, status, roles, and timestamps [ADR-0321 D-084; IP-016].
- IO-INTAKE-002: Export workflows with triggers, steps, approvals, and integration targets [IP-004; IP-016].
- IO-INTAKE-003: Export custom fields as metadata candidates [IP-003; PRD].
- IO-INTAKE-004: Export roles as identity mapping candidates [IP-002; PRD].
- IO-INTAKE-005: Export announcements as stakeholder update history [PRD; IP-030].
- IO-INTAKE-006: Export status pages with components, subscribers, incidents, and metrics [ADR-0168; IP-016].
- IO-INTAKE-007: Export Statuspage summary shape for compatibility validation [ADR-0168; IP-021].
- IO-INTAKE-008: Export Statuspage components shape for compatibility validation [ADR-0168; IP-021].
- IO-INTAKE-009: Export Statuspage incident history for public history migration [ADR-0168; ADR-0003].
- IO-INTAKE-010: Export RSS subscribers as local subscriber candidates [ADR-0168; IP-016].
- IO-INTAKE-011: Export webhook subscribers as signed delivery candidates [ADR-0168; ADR-0169].
- IO-INTAKE-012: Export email subscribers as tenant-owned subscription records [ADR-0168; IP-016].
- IO-INTAKE-013: Export retrospective documents as postmortem drafts [PRD; IP-025].
- IO-INTAKE-014: Export follow-up actions as postmortem action items [PRD; IP-002].
- IO-INTAKE-015: Export integrations as credential-sidecar candidates [IP-009; PRD].
- IO-INTAKE-016: Export audit logs as public/private disclosure provenance [ADR-0003; IP-011].
- IO-INTAKE-017: Export source checksums for rollback bundles [IP-016; IP-025].
- IO-INTAKE-018: Export source account status for vendor sunset evidence [IP-025; PRD].

## Transform and ontology
- IO-ONTO-001: Map incident.io incident to incident.incident [ADR-0321 D-084; IP-003].
- IO-ONTO-002: Map incident.io role to tenant principal alias [IP-002; PRD].
- IO-ONTO-003: Map incident.io workflow to workflow template id [IP-004; PRD].
- IO-ONTO-004: Map custom field to tenant metadata with reserved-key filter [IP-003; IP-002].
- IO-ONTO-005: Map announcement to stakeholder update [PRD; IP-005].
- IO-ONTO-006: Map status page to incident.status_page [ADR-0168; ADR-0321 D-084].
- IO-ONTO-007: Map Statuspage component to tenant component namespace [ADR-0168; IP-001].
- IO-ONTO-008: Map Statuspage summary to local summary JSON [ADR-0168; PRD].
- IO-ONTO-009: Map Statuspage incident history to public history records [ADR-0168; ADR-0003].
- IO-ONTO-010: Map RSS feed to tenant subscriber feed [ADR-0168; PRD].
- IO-ONTO-011: Map webhook subscription to signed delivery endpoint [ADR-0168; ADR-0169].
- IO-ONTO-012: Map email subscription to tenant subscriber record [ADR-0168; PRD].
- IO-ONTO-013: Map retrospective to postmortem draft [PRD; IP-025].
- IO-ONTO-014: Map follow-up to action item with owner and due date [PRD; IP-002].
- IO-ONTO-015: Map integration token to credential-sidecar reference [IP-009; IP-024].
- IO-ONTO-016: Map status impact to ADR-0168 status enum [ADR-0168; PRD].
- IO-ONTO-017: Map public correction to supersession record [ADR-0168; ADR-0003].
- IO-ONTO-018: Map migration provenance to rollback bundle [IP-016; IP-025].

## Command contracts
- IO-CMD-001: incident.import-preview validates source checksum [IP-016; ADR-0003].
- IO-CMD-002: incident.import-approve opens local incident room [PRD; IP-005].
- IO-CMD-003: workflow.import-preview validates workflow template [IP-004; IP-016].
- IO-CMD-004: workflow.execute runs approved workflow [IP-004; IP-002].
- IO-CMD-005: custom-field.import-preview validates ontology metadata [IP-003; IP-016].
- IO-CMD-006: role.map approves commander, liaison, and responder aliases [IP-002; PRD].
- IO-CMD-007: stakeholder.update publishes scoped message [PRD; IP-002].
- IO-CMD-008: status.summary.publish emits ADR-0168 summary JSON [ADR-0168; IP-005].
- IO-CMD-009: status.component.publish emits ADR-0168 component JSON [ADR-0168; IP-005].
- IO-CMD-010: status.incident-history.publish emits public history [ADR-0168; ADR-0003].
- IO-CMD-011: status.rss.publish emits all-product and per-product RSS [ADR-0168; PRD].
- IO-CMD-012: subscriber.webhook.import imports signed webhook subscriber [ADR-0168; ADR-0169].
- IO-CMD-013: subscriber.email.import imports email subscriber [ADR-0168; PRD].
- IO-CMD-014: postmortem.draft imports retrospective [PRD; IP-025].
- IO-CMD-015: action-item.add imports follow-up with owner and due date [PRD; IP-002].
- IO-CMD-016: postmortem.seal creates immutable revision [PRD; IP-025].
- IO-CMD-017: cutover.complete records SLO and audit success [IP-021; IP-025].
- IO-CMD-018: rollback.execute appends supersession or compensation record [IP-016; IP-025].

## Async events
- IO-EVT-001: incident.previewed emits checksum [IP-016; IP-006].
- IO-EVT-002: incident.room.opened emits timeline id [PRD; IP-006].
- IO-EVT-003: workflow.previewed emits template validation [IP-004; IP-006].
- IO-EVT-004: workflow.executed emits step result [IP-004; IP-006].
- IO-EVT-005: custom_field.imported emits metadata key [IP-003; IP-006].
- IO-EVT-006: role.mapped emits identity result [IP-002; IP-006].
- IO-EVT-007: stakeholder.update.published emits audience scope [PRD; IP-006].
- IO-EVT-008: status.summary.published emits ADR-0168 summary version [ADR-0168; IP-006].
- IO-EVT-009: status.component.published emits component id [ADR-0168; IP-006].
- IO-EVT-010: status.incident_history.published emits public history id [ADR-0168; ADR-0003].
- IO-EVT-011: status.rss.published emits feed path [ADR-0168; IP-006].
- IO-EVT-012: subscriber.webhook.imported emits signed delivery status [ADR-0168; ADR-0169].
- IO-EVT-013: subscriber.email.imported emits subscription id [ADR-0168; PRD].
- IO-EVT-014: postmortem.drafted emits draft id [PRD; IP-006].
- IO-EVT-015: action_item.added emits owner and due date [PRD; IP-006].
- IO-EVT-016: postmortem.sealed emits revision checksum [PRD; IP-025].
- IO-EVT-017: cutover.completed emits gate result [IP-021; IP-025].
- IO-EVT-018: rollback.executed emits compensation checksum [IP-016; IP-025].

## Cedar gates
- IO-CEDAR-001: incident import denies on tenant mismatch [IP-002; IP-001].
- IO-CEDAR-002: workflow import denies missing risk class [IP-002; IP-004].
- IO-CEDAR-003: workflow execute denies missing approval [IP-002; IP-004].
- IO-CEDAR-004: custom field import denies reserved key collision [IP-002; IP-003].
- IO-CEDAR-005: role map denies missing identity proof [IP-002; PRD].
- IO-CEDAR-006: stakeholder update denies private content in public audience [IP-002; PRD].
- IO-CEDAR-007: status summary publish denies missing component namespace [IP-002; ADR-0168].
- IO-CEDAR-008: status component publish denies cross-tenant component id [IP-002; ADR-0168].
- IO-CEDAR-009: status history publish denies deletion of public history [IP-002; ADR-0003].
- IO-CEDAR-010: RSS publish denies missing public disclosure scope [IP-002; ADR-0168].
- IO-CEDAR-011: webhook subscriber import denies unsigned delivery config [IP-002; ADR-0169].
- IO-CEDAR-012: email subscriber import denies missing tenant opt-in evidence [IP-002; ADR-0168].
- IO-CEDAR-013: postmortem draft denies missing timeline reference [IP-002; PRD].
- IO-CEDAR-014: action item add denies missing owner or due date [IP-002; PRD].
- IO-CEDAR-015: postmortem seal denies incomplete action items [IP-002; IP-025].
- IO-CEDAR-016: cutover denies missing status sync freshness SLO [IP-002; IP-021].
- IO-CEDAR-017: rollback denies destructive public history mutation [IP-002; ADR-0168].
- IO-CEDAR-018: source vendor admin denies as direct authority [IP-002; PRD].

## Migration and cutover
- IO-CUT-001: Stage incident exports before room creation [IP-016; PRD].
- IO-CUT-002: Stage workflow exports before template execution [IP-004; IP-016].
- IO-CUT-003: Stage role mapping before responder or commander assignment [IP-002; PRD].
- IO-CUT-004: Stage custom fields before timeline import [IP-003; IP-016].
- IO-CUT-005: Stage stakeholder update disclosure policy before public replay [IP-002; PRD].
- IO-CUT-006: Stage Statuspage components before summary JSON publication [ADR-0168; IP-030].
- IO-CUT-007: Stage incident history before RSS generation [ADR-0168; ADR-0003].
- IO-CUT-008: Stage webhook subscribers before signed delivery test [ADR-0168; ADR-0169].
- IO-CUT-009: Stage email subscribers before opt-in proof test [ADR-0168; PRD].
- IO-CUT-010: Stage retrospective docs before postmortem draft import [PRD; IP-025].
- IO-CUT-011: Stage follow-up owner mapping before action item import [PRD; IP-002].
- IO-CUT-012: Run status summary compatibility check [ADR-0168; IP-021].
- IO-CUT-013: Run status component compatibility check [ADR-0168; IP-021].
- IO-CUT-014: Run subscriber webhook retry check [ADR-0169; IP-021].
- IO-CUT-015: Run stakeholder update latency SLO [IP-021; slos/local-stakeholder-update-latency.openslo.yaml].
- IO-CUT-016: Run statuspage sync freshness SLO [IP-021; slos/local-statuspage-sync-freshness.openslo.yaml].
- IO-CUT-017: Run public correction supersession drill [ADR-0168; IP-025].
- IO-CUT-018: Run vendor sunset after rollback bundle is valid [IP-025; IP-016].

## Observability
- IO-OBS-001: Count incident imports by checksum [IP-011; IP-016].
- IO-OBS-002: Count workflow execution latency by template [IP-004; IP-011].
- IO-OBS-003: Count custom field reserved-key refusals [IP-002; IP-011].
- IO-OBS-004: Count role mapping refusals [IP-002; IP-011].
- IO-OBS-005: Count stakeholder update latency [IP-021; IP-011].
- IO-OBS-006: Count public/private disclosure refusals [IP-002; IP-011].
- IO-OBS-007: Count status summary freshness [ADR-0168; IP-021].
- IO-OBS-008: Count status component freshness [ADR-0168; IP-021].
- IO-OBS-009: Count status incident history supersessions [ADR-0168; ADR-0003].
- IO-OBS-010: Count RSS publication lag [ADR-0168; IP-011].
- IO-OBS-011: Count webhook subscriber retries [ADR-0169; IP-011].
- IO-OBS-012: Count email subscriber opt-in gaps [ADR-0168; IP-011].
- IO-OBS-013: Count postmortem draft completeness [PRD; IP-011].
- IO-OBS-014: Count action item orphan refusals [IP-002; IP-011].
- IO-OBS-015: Count postmortem seal completeness [IP-021; IP-025].
- IO-OBS-016: Count cutover blockers by status surface [IP-021; IP-025].
- IO-OBS-017: Count rollback readiness by public/private surface [IP-016; IP-025].
- IO-OBS-018: Count audit emission lag for public history [ADR-0003; slos/audit-emission-lag.openslo.yaml].

## Rollback
- IO-RB-001: Roll back incident import with compensation record [IP-016; PRD].
- IO-RB-002: Roll back workflow import by disabling template [IP-004; IP-016].
- IO-RB-003: Roll back workflow execution by compensating command [IP-016; IP-004].
- IO-RB-004: Roll back custom field by inactive annotation [IP-003; PRD].
- IO-RB-005: Roll back role map by quarantining alias [IP-002; IP-016].
- IO-RB-006: Roll back stakeholder update by supersession [PRD; IP-025].
- IO-RB-007: Roll back status summary by publishing correction [ADR-0168; IP-025].
- IO-RB-008: Roll back status component by publishing supersession [ADR-0168; IP-025].
- IO-RB-009: Roll back status history by appending correction [ADR-0168; ADR-0003].
- IO-RB-010: Roll back RSS feed by publishing corrected entry [ADR-0168; IP-025].
- IO-RB-011: Roll back webhook subscriber import by disabling local fanout [ADR-0168; ADR-0169].
- IO-RB-012: Roll back email subscriber import by disabling local subscription [ADR-0168; PRD].
- IO-RB-013: Roll back postmortem draft by new revision [PRD; IP-025].
- IO-RB-014: Roll back action item by closure reason [PRD; IP-025].
- IO-RB-015: Roll back postmortem seal by new revision [PRD; IP-025].
- IO-RB-016: Roll back cutover by returning to shadow phase [IP-021; IP-016].
- IO-RB-017: Roll back vendor sunset only before source account termination [IP-025; PRD].
- IO-RB-018: Roll back final claim by reporting this as IP substance only [task constraint; final evidence].

## Acceptance evidence
- IO-ACCEPT-001: File line count is >=200 [task constraint; wc].
- IO-ACCEPT-002: Citation density counts local authority refs [task constraint; grep].
- IO-ACCEPT-003: incident.io displacement remains inside incident-management IP scope [PRD; competitor-parity].
- IO-ACCEPT-004: Statuspage-compatible summary, component, incident, RSS, webhook, and email surfaces cite ADR-0168 [ADR-0168; rg].
- IO-ACCEPT-005: Webhook retry rows cite ADR-0169 [ADR-0169; rg].
- IO-ACCEPT-006: Public history rows cite ADR-0003 audit-chain [ADR-0003; rg].
- IO-ACCEPT-007: Workflow rows cite IP-004 [IP-004; rg].
- IO-ACCEPT-008: Cedar rows cite IP-002 [IP-002; rg].
- IO-ACCEPT-009: Backfill rows cite IP-016 [IP-016; rg].
- IO-ACCEPT-010: SLO rows cite IP-021 [IP-021; rg].
- IO-ACCEPT-011: No community, manifest, journey, ADR, or neighboring service file is edited [task constraint; git diff].
- IO-ACCEPT-012: No incident.io or Statuspage service boundary is introduced [PRD; ADR-0168].
- IO-ACCEPT-013: Public updates use supersession instead of deletion [ADR-0168; ADR-0003].
- IO-ACCEPT-014: Subscriber imports keep opt-in evidence [ADR-0168; PRD].
- IO-ACCEPT-015: Source vendor metadata remains migration evidence only [IP-002; PRD].
- IO-ACCEPT-016: Cutover waits for status sync and stakeholder update SLO gates [IP-021; IP-025].
- IO-ACCEPT-017: Rollback preserves public history [ADR-0168; ADR-0003].
- IO-ACCEPT-018: Final report includes path, line count, citation count, and blockers [task constraint; final evidence].

## Wave 15 counterpart anchor
- Counterpart baseline: PagerDuty, OpsGenie, xMatters, FireHydrant, ServiceNow, and Slack define the incident-management parity envelope; this displacement IP must close its slice with tenant-scoped policy, audit, and rollback evidence.
