# IP-029 Incident Management firehydrant-rootly-incident-command-postmortem-displacement

Service: incident-management
ChangeSet scope: microservices/incident-management/IP-029-firehydrant-rootly-incident-command-postmortem-displacement.md
Benchmark displacement: FireHydrant and Rootly incident command, service catalog context, retrospectives, tasks, runbooks, and stakeholder updates
Primary sources: ADR-0321 D-084; PRD-incident-management; competitor-parity-matrix.md; ADR-0003
Related IPs: IP-001, IP-002, IP-003, IP-004, IP-011, IP-016, IP-025
Non-goals: no service-catalog edit; no task/journey edit; no FireHydrant or Rootly boundary
Acceptance floor: >=200 lines; numbered rows cite local authority

## Objective
- FR-OBJ-001: Displace FireHydrant incident declaration with incident-room-open [ADR-0321 D-084; PRD].
- FR-OBJ-002: Displace Rootly incident command with tenant-local workflow templates [PRD section F; IP-004].
- FR-OBJ-003: Displace FireHydrant service context with ontology projection references [IP-003; ADR-0321 D-084].
- FR-OBJ-004: Displace Rootly service ownership context without changing service-catalog files [IP-020 reference; PRD].
- FR-OBJ-005: Displace FireHydrant runbooks with workflow template library rows [IP-004; PRD].
- FR-OBJ-006: Displace Rootly playbooks with Cedar-approved workflow runs [IP-002; IP-004].
- FR-OBJ-007: Displace FireHydrant retrospectives with postmortem-seal [ADR-0321 D-084; PRD].
- FR-OBJ-008: Displace Rootly post-incident review with sealed revisions [PRD; IP-025].
- FR-OBJ-009: Displace FireHydrant tasks with postmortem action items as incident evidence [PRD; IP-025].
- FR-OBJ-010: Displace Rootly follow-ups with tenant-local action item owners [PRD; IP-002].
- FR-OBJ-011: Displace FireHydrant stakeholder updates with disclosure-scoped updates [PRD; IP-030].
- FR-OBJ-012: Displace Rootly status updates with ADR-0168-compatible public payloads when public [ADR-0168; IP-030].
- FR-OBJ-013: Displace FireHydrant timeline with audit-chain backed incident timeline [ADR-0003; IP-011].
- FR-OBJ-014: Displace Rootly timeline with immutable evidence bundle rows [ADR-0003; IP-023].
- FR-OBJ-015: Displace FireHydrant task sync with source annotations, not task service writes [PRD; task constraint].
- FR-OBJ-016: Displace Rootly integrations with credential-sidecar references [IP-009; PRD].
- FR-OBJ-017: Displace vendor retrospective templates with tenant-owned templates [IP-004; PRD].
- FR-OBJ-018: Displace vendor lock-in with rollback evidence before sunset [IP-016; IP-025].

## Source export intake
- FR-INTAKE-001: Export incidents with severity, commander, services, roles, and timestamps [ADR-0321 D-084; IP-016].
- FR-INTAKE-002: Export milestones and timeline events with source checksum [ADR-0003; IP-011].
- FR-INTAKE-003: Export roles as identity-mapping candidates [IP-002; PRD].
- FR-INTAKE-004: Export service references as ontology annotations [IP-003; PRD].
- FR-INTAKE-005: Export runbooks as workflow template candidates [IP-004; PRD].
- FR-INTAKE-006: Export playbooks as workflow template candidates [IP-004; PRD].
- FR-INTAKE-007: Export retrospectives as postmortem source documents [PRD; IP-025].
- FR-INTAKE-008: Export action items as postmortem action item candidates [PRD; IP-025].
- FR-INTAKE-009: Export stakeholder messages as disclosure-scoped source history [PRD; IP-030].
- FR-INTAKE-010: Export status updates as public/private split candidates [ADR-0168; IP-030].
- FR-INTAKE-011: Export integrations as credential-sidecar candidates [IP-009; IP-016].
- FR-INTAKE-012: Export incident types as tenant severity policy candidates [PRD; IP-001].
- FR-INTAKE-013: Export teams as incident team aliases [ADR-0321 D-084; IP-002].
- FR-INTAKE-014: Export user profiles as principal aliases [IP-002; IP-001].
- FR-INTAKE-015: Export attachments as evidence bundle source references [IP-023; ADR-0003].
- FR-INTAKE-016: Export custom fields as structured timeline metadata [IP-003; PRD].
- FR-INTAKE-017: Export audit rows as migration provenance [ADR-0003; IP-011].
- FR-INTAKE-018: Export source account status as sunset evidence input [IP-025; PRD].

## Transform and ontology
- FR-ONTO-001: Map incident to incident.incident with timeline id [ADR-0321 D-084; IP-003].
- FR-ONTO-002: Map service reference to incident.service annotation [IP-003; PRD].
- FR-ONTO-003: Map commander role to incident commander principal alias [IP-002; PRD].
- FR-ONTO-004: Map liaison role to stakeholder update audience [PRD; IP-030].
- FR-ONTO-005: Map communications lead to disclosure approver candidate [IP-002; IP-030].
- FR-ONTO-006: Map timeline event to audit-chain timeline row [ADR-0003; IP-011].
- FR-ONTO-007: Map milestone to workflow step completion [IP-004; PRD].
- FR-ONTO-008: Map runbook to workflow template reference [IP-004; PRD].
- FR-ONTO-009: Map playbook to workflow template reference [IP-004; PRD].
- FR-ONTO-010: Map retrospective to postmortem draft [PRD; IP-025].
- FR-ONTO-011: Map action item to postmortem action item [PRD; IP-025].
- FR-ONTO-012: Map attachment to evidence bundle artifact [IP-023; ADR-0003].
- FR-ONTO-013: Map stakeholder message to disclosure-scoped update [PRD; IP-030].
- FR-ONTO-014: Map status update to ADR-0168 component update when public [ADR-0168; IP-030].
- FR-ONTO-015: Map custom field to tenant metadata with reserved-key filtering [IP-003; IP-002].
- FR-ONTO-016: Map integration token to credential-sidecar reference [IP-009; IP-024].
- FR-ONTO-017: Map severity to tenant severity policy [PRD; IP-001].
- FR-ONTO-018: Map source provenance to rollback bundle [IP-016; IP-025].

## Command contracts
- FR-CMD-001: incident.import-preview validates source checksum [IP-016; ADR-0003].
- FR-CMD-002: incident.import-approve creates incident-room-open command [PRD; IP-005].
- FR-CMD-003: role.map approves incident commander and liaison aliases [IP-002; PRD].
- FR-CMD-004: service-context.attach records ontology annotation [IP-003; PRD].
- FR-CMD-005: runbook.import-preview validates workflow template [IP-004; IP-016].
- FR-CMD-006: playbook.import-preview validates workflow template [IP-004; IP-016].
- FR-CMD-007: workflow.execute runs approved incident template [IP-004; IP-002].
- FR-CMD-008: timeline.append records audit-chain event [ADR-0003; IP-011].
- FR-CMD-009: stakeholder.update publishes scoped message [PRD; IP-030].
- FR-CMD-010: status.update publishes ADR-0168-compatible message when public [ADR-0168; IP-030].
- FR-CMD-011: postmortem.draft creates local retrospective draft [PRD; IP-025].
- FR-CMD-012: postmortem.action-item.add requires owner and due date [PRD; IP-002].
- FR-CMD-013: postmortem.seal creates immutable revision [PRD; IP-025].
- FR-CMD-014: evidence.attach binds source artifact to evidence bundle [IP-023; ADR-0003].
- FR-CMD-015: shadow-run.start compares vendor and local workflow outcomes [IP-021; IP-016].
- FR-CMD-016: cutover.complete records SLO and audit success [IP-021; IP-025].
- FR-CMD-017: vendor-sunset.record records source retirement [IP-025; PRD].
- FR-CMD-018: rollback.execute appends compensation record [IP-016; IP-025].

## Async events
- FR-EVT-001: incident.previewed emits source checksum [IP-016; ADR-0003].
- FR-EVT-002: incident.room.opened emits timeline id [PRD; IP-006].
- FR-EVT-003: role.mapped emits identity decision [IP-002; IP-006].
- FR-EVT-004: service.context.attached emits ontology annotation [IP-003; IP-006].
- FR-EVT-005: runbook.previewed emits template validation [IP-004; IP-006].
- FR-EVT-006: playbook.previewed emits template validation [IP-004; IP-006].
- FR-EVT-007: workflow.executed emits step result [IP-004; IP-006].
- FR-EVT-008: timeline.appended emits audit-chain checksum [ADR-0003; IP-006].
- FR-EVT-009: stakeholder.update.published emits disclosure audience [PRD; IP-006].
- FR-EVT-010: status.update.published emits ADR-0168 component [ADR-0168; IP-006].
- FR-EVT-011: postmortem.drafted emits draft id [PRD; IP-006].
- FR-EVT-012: action_item.added emits owner and due date [PRD; IP-006].
- FR-EVT-013: postmortem.sealed emits revision checksum [IP-025; IP-006].
- FR-EVT-014: evidence.attached emits bundle artifact id [IP-023; IP-006].
- FR-EVT-015: shadow_run.started emits comparison window [IP-021; IP-006].
- FR-EVT-016: cutover.completed emits gate result [IP-021; IP-025].
- FR-EVT-017: vendor_sunset.recorded emits exit evidence [IP-025; PRD].
- FR-EVT-018: rollback.executed emits compensation checksum [IP-016; IP-025].

## Cedar gates
- FR-CEDAR-001: incident import denies on tenant mismatch [IP-002; IP-001].
- FR-CEDAR-002: room open denies without commander principal [IP-002; PRD].
- FR-CEDAR-003: role map denies without identity proof [IP-002; IP-001].
- FR-CEDAR-004: service context attach denies cross-tenant service [IP-002; IP-003].
- FR-CEDAR-005: runbook import denies missing risk class [IP-002; IP-004].
- FR-CEDAR-006: workflow execute denies missing approval [IP-002; IP-004].
- FR-CEDAR-007: timeline append denies missing audit target [IP-002; ADR-0003].
- FR-CEDAR-008: stakeholder update denies private content in public audience [IP-002; IP-030].
- FR-CEDAR-009: status update denies missing ADR-0168 component [IP-002; ADR-0168].
- FR-CEDAR-010: postmortem draft denies missing timeline reference [IP-002; PRD].
- FR-CEDAR-011: action item add denies missing owner [IP-002; PRD].
- FR-CEDAR-012: postmortem seal denies orphan action items [IP-002; IP-025].
- FR-CEDAR-013: evidence attach denies raw secret artifacts [IP-002; IP-009].
- FR-CEDAR-014: shadow run denies live side effect during replay [IP-002; IP-016].
- FR-CEDAR-015: cutover denies missing rollback bundle [IP-002; IP-016].
- FR-CEDAR-016: vendor sunset denies active source dependency [IP-002; IP-025].
- FR-CEDAR-017: rollback denies destructive history deletion [IP-002; ADR-0003].
- FR-CEDAR-018: source-vendor admin denies as direct authority [IP-002; PRD].

## Migration and cutover
- FR-CUT-001: Stage source incident exports before room creation [IP-016; PRD].
- FR-CUT-002: Stage role mappings before commander assignment [IP-002; PRD].
- FR-CUT-003: Stage service context before workflow template run [IP-003; IP-004].
- FR-CUT-004: Stage runbooks and playbooks before live execution [IP-004; IP-016].
- FR-CUT-005: Stage stakeholder disclosure policy before update migration [IP-030; IP-002].
- FR-CUT-006: Stage postmortem templates before seal testing [PRD; IP-025].
- FR-CUT-007: Stage action item owner mapping before action item import [IP-002; PRD].
- FR-CUT-008: Stage evidence attachments before final postmortem seal [IP-023; ADR-0003].
- FR-CUT-009: Run timeline replay without public subscriber fanout [IP-016; IP-030].
- FR-CUT-010: Run room creation SLO drill [IP-021; slos/local-war-room-creation-latency.openslo.yaml].
- FR-CUT-011: Run stakeholder update SLO drill [IP-021; slos/local-stakeholder-update-latency.openslo.yaml].
- FR-CUT-012: Run postmortem seal completeness drill [IP-021; slos/local-postmortem-seal-completeness.openslo.yaml].
- FR-CUT-013: Run workflow template dry-run for destructive steps [IP-004; IP-002].
- FR-CUT-014: Run evidence packet completeness check [IP-023; PRD].
- FR-CUT-015: Run rollback simulation for room, workflow, update, and postmortem [IP-016; IP-025].
- FR-CUT-016: Run SLO promotion before source cutover [IP-021; IP-025].
- FR-CUT-017: Run vendor sunset only after all action items and evidence bundles are local [IP-025; PRD].
- FR-CUT-018: Run tenant audit export after source retirement [IP-023; ADR-0003].

## Observability
- FR-OBS-001: Count incident imports by source checksum [IP-011; IP-016].
- FR-OBS-002: Count room creation latency by tenant and severity [IP-021; IP-011].
- FR-OBS-003: Count role mapping refusals by source role [IP-002; IP-011].
- FR-OBS-004: Count service context attach failures [IP-003; IP-011].
- FR-OBS-005: Count workflow dry-run failures [IP-004; IP-011].
- FR-OBS-006: Count timeline append audit lag [ADR-0003; IP-011].
- FR-OBS-007: Count stakeholder update latency [IP-021; IP-011].
- FR-OBS-008: Count status update component failures [ADR-0168; IP-030].
- FR-OBS-009: Count postmortem draft completeness [PRD; IP-011].
- FR-OBS-010: Count orphan action item refusals [IP-002; IP-011].
- FR-OBS-011: Count postmortem seal completeness [IP-021; IP-025].
- FR-OBS-012: Count evidence bundle completeness [IP-023; IP-011].
- FR-OBS-013: Count shadow-run differences by workflow template [IP-021; IP-004].
- FR-OBS-014: Count cutover blockers by incident phase [IP-025; PRD].
- FR-OBS-015: Count rollback readiness by object type [IP-016; IP-025].
- FR-OBS-016: Count public/private disclosure refusals [IP-002; IP-030].
- FR-OBS-017: Count source-vendor sunset readiness [IP-025; PRD].
- FR-OBS-018: Count audit export completeness [ADR-0003; IP-023].

## Rollback
- FR-RB-001: Roll back incident import by appending compensation [IP-016; PRD].
- FR-RB-002: Roll back room creation by archiving bridge credentials [IP-009; PRD].
- FR-RB-003: Roll back role mapping by quarantining alias [IP-002; IP-016].
- FR-RB-004: Roll back service context by marking annotation inactive [IP-003; PRD].
- FR-RB-005: Roll back runbook import by disabling workflow template [IP-004; IP-016].
- FR-RB-006: Roll back workflow execution with compensating command [IP-016; IP-004].
- FR-RB-007: Roll back stakeholder update by supersession [IP-030; PRD].
- FR-RB-008: Roll back status update by ADR-0168 correction [ADR-0168; IP-030].
- FR-RB-009: Roll back postmortem draft by new revision marker [PRD; IP-025].
- FR-RB-010: Roll back action item by closure reason, not deletion [PRD; IP-025].
- FR-RB-011: Roll back postmortem seal by new revision [IP-025; PRD].
- FR-RB-012: Roll back evidence attach by artifact supersession [IP-023; ADR-0003].
- FR-RB-013: Roll back shadow run by disabling local fanout [IP-021; IP-016].
- FR-RB-014: Roll back cutover by returning to shadow phase [IP-021; IP-025].
- FR-RB-015: Roll back vendor sunset only before source account termination [IP-025; PRD].
- FR-RB-016: Roll back audit packet by issuing corrected packet [IP-023; ADR-0003].
- FR-RB-017: Roll back policy bundle by preserving refusal evidence [IP-002; IP-011].
- FR-RB-018: Roll back final claim by reporting this as IP substance only [task constraint; final evidence].

## Acceptance evidence
- FR-ACCEPT-001: File line count is >=200 [task constraint; wc].
- FR-ACCEPT-002: Citation density counts local authority refs [task constraint; grep].
- FR-ACCEPT-003: FireHydrant and Rootly displacement remains inside incident-management IP scope [PRD; competitor-parity].
- FR-ACCEPT-004: Service context rows cite ontology rather than editing service-catalog [IP-003; task constraint].
- FR-ACCEPT-005: Runbook and playbook rows cite workflow template IP-004 [IP-004; rg].
- FR-ACCEPT-006: Postmortem rows cite PRD and IP-025 [PRD; IP-025].
- FR-ACCEPT-007: Stakeholder update rows cite IP-030 and disclosure policy [IP-030; IP-002].
- FR-ACCEPT-008: Timeline rows cite ADR-0003 and IP-011 [ADR-0003; IP-011].
- FR-ACCEPT-009: Backfill rows cite IP-016 [IP-016; rg].
- FR-ACCEPT-010: SLO rows cite IP-021 and local SLOs [IP-021; rg].
- FR-ACCEPT-011: No task, journey, manifest, ADR, or service-catalog file is edited [task constraint; git diff].
- FR-ACCEPT-012: No FireHydrant or Rootly service boundary is introduced [PRD; competitor-parity].
- FR-ACCEPT-013: Public updates use supersession instead of deletion [ADR-0168; IP-030].
- FR-ACCEPT-014: Action items require tenant-local owners [PRD; IP-002].
- FR-ACCEPT-015: Evidence bundle rows preserve source attachments without raw secrets [IP-023; IP-009].
- FR-ACCEPT-016: Cutover waits for room, stakeholder, and postmortem SLO gates [IP-021; IP-025].
- FR-ACCEPT-017: Rollback preserves audit-chain history [ADR-0003; IP-016].
- FR-ACCEPT-018: Final report includes path, line count, citation count, and blockers [task constraint; final evidence].
