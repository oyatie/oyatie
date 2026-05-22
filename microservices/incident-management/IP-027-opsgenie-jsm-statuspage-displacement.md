# IP-027 Incident Management opsgenie-jsm-statuspage-displacement

Service: incident-management
ChangeSet scope: microservices/incident-management/IP-027-opsgenie-jsm-statuspage-displacement.md
Benchmark displacement: Atlassian OpsGenie alerting, on-call, escalation, Jira bridge, Confluence postmortem, Statuspage component update, OEC
Primary sources: ADR-0321 D-085; ADR-0321 D-084; ADR-0168; PRD-incident-management; competitor-parity-matrix.md
Related IPs: IP-001, IP-002, IP-005, IP-006, IP-009, IP-016, IP-030
Non-goals: no Atlassian suite boundary; no Jira, Confluence, Statuspage, or community service edit
Acceptance floor: >=200 lines; numbered rows cite local authority

## Objective
- OG-OBJ-001: Displace OpsGenie alert create by projecting alert source into page-dispatch [ADR-0321 D-085; PRD].
- OG-OBJ-002: Displace OpsGenie alert deduplication with tenant-owned dedup keys [ADR-0321 D-085; ADR-0169].
- OG-OBJ-003: Displace OpsGenie on-call schedules with incident.on_call_schedule [ADR-0321 D-085; IP-001].
- OG-OBJ-004: Displace OpsGenie escalation policies with Cedar-gated policy graphs [ADR-0321 D-085; IP-002].
- OG-OBJ-005: Displace OpsGenie war-room integration through incident-room-open [ADR-0321 D-085; PRD].
- OG-OBJ-006: Displace OpsGenie Jira link by treating Jira issue id as external annotation [ADR-0321 D-085; PRD].
- OG-OBJ-007: Displace OpsGenie Confluence postmortem by sealing postmortem locally [ADR-0321 D-085; PRD].
- OG-OBJ-008: Displace OpsGenie Statuspage update through ADR-0168-compatible components [ADR-0321 D-085; ADR-0168].
- OG-OBJ-009: Displace OpsGenie Edge Connector by credential-sidecar controlled ingress [ADR-0321 D-085; IP-009].
- OG-OBJ-010: Displace Atlassian user roles by identity-mapped principals [ADR-0321 D-085; IP-002].
- OG-OBJ-011: Displace Atlassian team roles by tenant incident teams [ADR-0321 D-085; IP-001].
- OG-OBJ-012: Displace Atlassian product coupling by keeping workflow-engine templates separate [ADR-0321 D-039; PRD].
- OG-OBJ-013: Displace OpsGenie mobile ack with responder-audience Cedar permits [ADR-0321 D-085; IP-002].
- OG-OBJ-014: Displace OpsGenie notification routing with comms rail contracts [ADR-0321 D-085; PRD].
- OG-OBJ-015: Displace OpsGenie alert notes with tenant timeline entries [ADR-0321 D-085; PRD].
- OG-OBJ-016: Displace OpsGenie alert close with incident resolve command [ADR-0321 D-084; IP-005].
- OG-OBJ-017: Displace OpsGenie historical exports with replay-safe import batches [ADR-0321 D-085; IP-016].
- OG-OBJ-018: Displace OpsGenie subscription dependency with explicit sunset evidence [ADR-0321 D-085; IP-025].

## Source export intake
- OG-INTAKE-001: Export alerts with alias, message, priority, responders, tags, details, and source checksum [ADR-0321 D-085; IP-016].
- OG-INTAKE-002: Export teams with membership only as identity mapping input [ADR-0321 D-085; IP-002].
- OG-INTAKE-003: Export escalation policies with rules, recipients, repeats, and delays [ADR-0321 D-085; IP-002].
- OG-INTAKE-004: Export schedules with rotations, overrides, timezone, and holiday rules [ADR-0321 D-085; IP-001].
- OG-INTAKE-005: Export integrations with OEC marker and source routing metadata [ADR-0321 D-085; IP-009].
- OG-INTAKE-006: Export Jira links as external issue annotations [ADR-0321 D-085; PRD].
- OG-INTAKE-007: Export Confluence postmortem links as source document references [ADR-0321 D-085; PRD].
- OG-INTAKE-008: Export Statuspage component links as ADR-0168 component candidates [ADR-0321 D-085; ADR-0168].
- OG-INTAKE-009: Export alert timelines as incident_timeline migration rows [ADR-0321 D-085; PRD].
- OG-INTAKE-010: Export alert actions as replay fixtures, not live side effects [IP-016; ADR-0169].
- OG-INTAKE-011: Export notification policies as comms preference candidates [ADR-0321 D-085; PRD].
- OG-INTAKE-012: Export maintenance windows as suppression windows with tenant permits [ADR-0321 D-085; IP-002].
- OG-INTAKE-013: Export heartbeat monitors as event-rule input examples [ADR-0321 D-085; IP-006].
- OG-INTAKE-014: Export source API tokens only into credential-sidecar migration flow [ADR-0321 D-085; IP-009].
- OG-INTAKE-015: Export audit logs and mark missing rows in evidence packet [PRD section E; IP-023].
- OG-INTAKE-016: Export Atlassian organization and site ids only as migration metadata [ADR-0321 D-039; PRD].
- OG-INTAKE-017: Export alert dedup aliases for idempotency comparison [ADR-0169; IP-016].
- OG-INTAKE-018: Export source account status for final subscription sunset evidence [ADR-0321 D-085; IP-025].

## Transform and ontology
- OG-ONTO-001: Map OpsGenie alert to incident.incident plus page_event [ADR-0321 D-085; IP-003].
- OG-ONTO-002: Map OpsGenie team to incident.team after identity approval [ADR-0321 D-085; IP-002].
- OG-ONTO-003: Map OpsGenie policy_rotation to incident.on_call_schedule [ADR-0321 D-085; IP-001].
- OG-ONTO-004: Map OpsGenie escalation to incident.escalation_policy [ADR-0321 D-085; IP-002].
- OG-ONTO-005: Map OpsGenie OEC integration to incident.event_rule and credential binding [ADR-0321 D-085; IP-009].
- OG-ONTO-006: Map Jira issue link to incident.external_reference [ADR-0321 D-085; PRD].
- OG-ONTO-007: Map Confluence postmortem link to postmortem source annotation [ADR-0321 D-085; PRD].
- OG-ONTO-008: Map Statuspage component to incident.status_page component [ADR-0321 D-085; ADR-0168].
- OG-ONTO-009: Map priority P1-P5 to tenant severity policy [ADR-0321 D-085; PRD].
- OG-ONTO-010: Map responders to principal aliases before authorization [ADR-0321 D-085; IP-002].
- OG-ONTO-011: Map tags to tenant labels with reserved prefix filtering [ADR-0321 D-085; IP-003].
- OG-ONTO-012: Map details fields to structured timeline metadata [ADR-0321 D-085; PRD].
- OG-ONTO-013: Map alias to dedup_key and idempotency_key [ADR-0169; IP-006].
- OG-ONTO-014: Map actions to workflow template invocations [ADR-0321 D-085; IP-004].
- OG-ONTO-015: Map notes to incident timeline entries [ADR-0321 D-085; PRD].
- OG-ONTO-016: Map alert close to resolve command [ADR-0321 D-084; IP-005].
- OG-ONTO-017: Map alert acknowledgement to page acknowledgement command [ADR-0321 D-084; IP-005].
- OG-ONTO-018: Map source provenance to audit-chain bundle [PRD section E; IP-011].

## Command contracts
- OG-CMD-001: alert.import-preview accepts source checksum and no live notification permission [IP-016; ADR-0321 D-085].
- OG-CMD-002: alert.import-approve creates local incident and page_event with rollback bundle [IP-016; IP-001].
- OG-CMD-003: alert.dedup-preview compares source alias and local dedup key [ADR-0169; IP-006].
- OG-CMD-004: alert.acknowledge invokes page acknowledgement with responder principal [IP-005; IP-002].
- OG-CMD-005: alert.escalate invokes escalation-evaluate with policy version [IP-005; IP-002].
- OG-CMD-006: alert.close invokes incident resolve with timeline checksum [IP-005; PRD].
- OG-CMD-007: team.import-preview resolves identity aliases before promotion [ADR-0321 D-085; IP-002].
- OG-CMD-008: schedule.import-preview validates rotations and no-gap evidence [IP-001; IP-016].
- OG-CMD-009: policy.import-preview validates cycles and repeat edges [IP-002; IP-016].
- OG-CMD-010: oec.bind creates credential-sidecar reference and event rule [ADR-0321 D-085; IP-009].
- OG-CMD-011: jira.link records external issue reference without granting Jira authority [ADR-0321 D-085; PRD].
- OG-CMD-012: confluence.link records source postmortem reference without remote write [ADR-0321 D-085; PRD].
- OG-CMD-013: statuspage.component.bind maps component to ADR-0168 schema [ADR-0168; IP-030].
- OG-CMD-014: statuspage.update publishes tenant-scoped public update [ADR-0168; IP-030].
- OG-CMD-015: shadow-route.start begins Atlassian dual-route comparison [ADR-0321 D-085; IP-021].
- OG-CMD-016: cutover.complete records Atlassian link parity and local SLO pass [ADR-0321 D-085; IP-025].
- OG-CMD-017: vendor-sunset.record preserves subscription exit evidence [ADR-0321 D-085; IP-025].
- OG-CMD-018: rollback.execute compensates local commands without deleting public history [IP-016; IP-030].

## Async events
- OG-EVT-001: opsgenie.alert.previewed emits source checksum and transform version [IP-016; PRD].
- OG-EVT-002: opsgenie.alert.approved emits local incident id and page id [IP-006; IP-001].
- OG-EVT-003: opsgenie.alert.deduped emits alias and dedup key comparison [ADR-0169; IP-006].
- OG-EVT-004: opsgenie.alert.acknowledged emits responder and ack latency [IP-006; IP-021].
- OG-EVT-005: opsgenie.alert.escalated emits policy graph version [IP-006; IP-002].
- OG-EVT-006: opsgenie.alert.closed emits timeline checksum [IP-006; PRD].
- OG-EVT-007: opsgenie.team.mapped emits identity resolution status [IP-006; IP-002].
- OG-EVT-008: opsgenie.schedule.imported emits no-gap result [IP-006; IP-001].
- OG-EVT-009: opsgenie.policy.imported emits cycle result [IP-006; IP-002].
- OG-EVT-010: opsgenie.oec.bound emits credential-sidecar reference [IP-006; IP-009].
- OG-EVT-011: opsgenie.jira.linked emits external reference id [ADR-0321 D-085; PRD].
- OG-EVT-012: opsgenie.confluence.linked emits postmortem annotation id [ADR-0321 D-085; PRD].
- OG-EVT-013: opsgenie.statuspage.component.bound emits ADR-0168 component id [ADR-0168; IP-030].
- OG-EVT-014: opsgenie.statuspage.updated emits public supersession id [ADR-0168; IP-030].
- OG-EVT-015: opsgenie.shadow.started emits dual-route window [ADR-0321 D-085; IP-021].
- OG-EVT-016: opsgenie.cutover.completed emits SLO evidence [IP-021; IP-025].
- OG-EVT-017: opsgenie.vendor.sunset emits exit evidence [ADR-0321 D-085; IP-025].
- OG-EVT-018: opsgenie.rollback.executed emits compensation checksum [IP-016; IP-025].

## Cedar gates
- OG-CEDAR-001: alert import denies if tenant mismatch appears in source row [IP-002; ADR-0321 D-085].
- OG-CEDAR-002: alert ack denies if principal is not mapped responder [IP-002; ADR-0321 D-085].
- OG-CEDAR-003: alert escalate denies if policy graph is stale [IP-002; ADR-0321 D-085].
- OG-CEDAR-004: alert close denies if postmortem requirement is unresolved [IP-002; PRD].
- OG-CEDAR-005: team mapping denies if identity proof is absent [IP-002; ADR-0321 D-085].
- OG-CEDAR-006: schedule import denies if gap check fails [IP-002; IP-001].
- OG-CEDAR-007: policy import denies if cycle check fails [IP-002; ADR-0321 D-085].
- OG-CEDAR-008: OEC bind denies if credential-sidecar grant is absent [IP-002; IP-009].
- OG-CEDAR-009: Jira link denies if Atlassian bridge is not tenant-approved [IP-002; ADR-0321 D-085].
- OG-CEDAR-010: Confluence link denies if postmortem export is disabled [IP-002; ADR-0321 D-085].
- OG-CEDAR-011: Statuspage component bind denies if component namespace is not tenant-owned [IP-002; ADR-0168].
- OG-CEDAR-012: Statuspage update denies if disclosure policy is private [IP-002; ADR-0168].
- OG-CEDAR-013: Shadow route denies if SLO budget is burning [IP-002; IP-021].
- OG-CEDAR-014: Cutover denies if rollback bundle is absent [IP-002; IP-016].
- OG-CEDAR-015: Vendor sunset denies if active source dependency remains [IP-002; IP-025].
- OG-CEDAR-016: Run-OEC denies if raw source token appears in payload [IP-002; IP-009].
- OG-CEDAR-017: Cross-product link denies if Jira or Confluence reference crosses tenant boundary [IP-002; ADR-0321 D-085].
- OG-CEDAR-018: Public correction denies if supersession id is absent [IP-002; ADR-0168].

## Migration and cutover
- OG-CUT-001: Preview alert imports before responder notification [IP-016; ADR-0321 D-085].
- OG-CUT-002: Preview schedule imports before policy graph import [IP-001; IP-016].
- OG-CUT-003: Preview policy imports before escalation testing [IP-002; IP-016].
- OG-CUT-004: Bind OEC credentials before event replay [IP-009; ADR-0321 D-085].
- OG-CUT-005: Map Jira links before incident-room template activation [ADR-0321 D-085; PRD].
- OG-CUT-006: Map Confluence postmortem links before postmortem seal testing [ADR-0321 D-085; PRD].
- OG-CUT-007: Map Statuspage components before public update testing [ADR-0168; IP-030].
- OG-CUT-008: Run replay with live paging disabled [ADR-0169; IP-016].
- OG-CUT-009: Run OEC ingress comparison against local event rules [ADR-0321 D-085; IP-006].
- OG-CUT-010: Run ack-latency comparison against local SLO [IP-021; slos/local-page-to-acknowledge.openslo.yaml].
- OG-CUT-011: Run war-room comparison against local SLO [IP-021; slos/local-war-room-creation-latency.openslo.yaml].
- OG-CUT-012: Run statuspage sync comparison against ADR-0168 payloads [ADR-0168; IP-021].
- OG-CUT-013: Run postmortem seal comparison against local evidence bundle [PRD; IP-025].
- OG-CUT-014: Run 14-day shadow route when replacing live alerting [ADR-0321 D-084; IP-021].
- OG-CUT-015: Run cutover only after all critical integrations dual-route cleanly [ADR-0321 D-085; IP-021].
- OG-CUT-016: Run vendor sunset after source dependencies are absent [ADR-0321 D-085; IP-025].
- OG-CUT-017: Run rollback drill before final source deactivation [IP-016; IP-025].
- OG-CUT-018: Run audit export and tenant signoff after subscription sunset [PRD section E; IP-023].

## Observability
- OG-OBS-001: Count imported alerts by tenant, source checksum, and transform version [IP-011; IP-016].
- OG-OBS-002: Count dedup collisions by alias and local dedup key [ADR-0169; IP-011].
- OG-OBS-003: Count ack latency by responder and schedule version [IP-021; IP-011].
- OG-OBS-004: Count escalation outcomes by policy graph version [IP-002; IP-011].
- OG-OBS-005: Count Jira link failures by external reference type [ADR-0321 D-085; IP-011].
- OG-OBS-006: Count Confluence link failures by postmortem annotation type [ADR-0321 D-085; IP-011].
- OG-OBS-007: Count Statuspage component sync freshness [ADR-0168; IP-021].
- OG-OBS-008: Count OEC ingress refusals by credential-sidecar reason [IP-009; IP-011].
- OG-OBS-009: Count policy refusals by IncidentResponseAction verb [ADR-0321 D-084; IP-002].
- OG-OBS-010: Count shadow-route mismatches by alert alias [ADR-0321 D-085; IP-021].
- OG-OBS-011: Count cutover blockers by integration family [ADR-0321 D-085; IP-025].
- OG-OBS-012: Count rollback readiness by imported object type [IP-016; IP-025].
- OG-OBS-013: Count public update supersessions by component [ADR-0168; IP-030].
- OG-OBS-014: Count identity mapping gaps by source team [ADR-0321 D-085; IP-002].
- OG-OBS-015: Count schedule gaps discovered during import [IP-001; IP-011].
- OG-OBS-016: Count policy cycles discovered during import [IP-002; IP-011].
- OG-OBS-017: Count audit emission lag after import approval [IP-011; slos/audit-emission-lag.openslo.yaml].
- OG-OBS-018: Count tenant cost for dual-route alerting [IP-017; PRD].

## Rollback
- OG-RB-001: Roll back alert import by compensating local incident creation [IP-016; IP-025].
- OG-RB-002: Roll back schedule import by restoring previous rotation checksum [IP-001; IP-016].
- OG-RB-003: Roll back policy import by restoring previous graph checksum [IP-002; IP-016].
- OG-RB-004: Roll back OEC bind by revoking credential-sidecar reference [IP-009; IP-016].
- OG-RB-005: Roll back Jira link by marking external reference inactive [ADR-0321 D-085; PRD].
- OG-RB-006: Roll back Confluence link by marking annotation inactive [ADR-0321 D-085; PRD].
- OG-RB-007: Roll back Statuspage component by publishing supersession [ADR-0168; IP-030].
- OG-RB-008: Roll back public update by publishing correction [ADR-0168; IP-030].
- OG-RB-009: Roll back identity mapping by quarantining unmapped responder aliases [IP-002; IP-016].
- OG-RB-010: Roll back shadow route by returning alert delivery to source only with explicit permit [IP-002; IP-021].
- OG-RB-011: Roll back cutover by re-enabling source path before subscription sunset [ADR-0321 D-085; IP-016].
- OG-RB-012: Roll back vendor sunset only if source subscription and credentials remain active [ADR-0321 D-085; IP-025].
- OG-RB-013: Roll back postmortem source link without mutating sealed local document [PRD; IP-025].
- OG-RB-014: Roll back status subscriber migration without deleting public history [ADR-0168; IP-030].
- OG-RB-015: Roll back policy deploy by preserving refusal evidence [IP-002; IP-011].
- OG-RB-016: Roll back import batch by appending compensation records [IP-016; PRD].
- OG-RB-017: Roll back marketplace automation by preserving DealSet evidence [ADR-0314; IP-014].
- OG-RB-018: Roll back final claim by reporting this as IP substance only [task constraint; final evidence].

## Acceptance evidence
- OG-ACCEPT-001: File has >=200 lines by wc evidence [task constraint; verification].
- OG-ACCEPT-002: Numbered rows cite ADR-0321 D-085, ADR-0321 D-084, ADR-0168, PRD, or IP dependencies [task constraint; verification].
- OG-ACCEPT-003: OpsGenie-specific verbs include link-to-jira-issue, link-to-confluence-postmortem, configure-statuspage-component, and run-OEC [ADR-0321 D-085; rg].
- OG-ACCEPT-004: Atlassian cross-product bridge remains annotation-only, not authority [ADR-0321 D-085; PRD].
- OG-ACCEPT-005: Statuspage-compatible public component rows cite ADR-0168 [ADR-0168; rg].
- OG-ACCEPT-006: OEC rows cite credential-sidecar IP-009 [IP-009; rg].
- OG-ACCEPT-007: Schedule and policy import rows cite IP-001 and IP-002 [IP-001; IP-002].
- OG-ACCEPT-008: Backfill and replay rows cite IP-016 [IP-016; rg].
- OG-ACCEPT-009: Cutover and SLO rows cite IP-021 [IP-021; rg].
- OG-ACCEPT-010: Closeout rows cite IP-025 [IP-025; rg].
- OG-ACCEPT-011: No prohibited neighboring services are referenced as write targets [task constraint; git diff].
- OG-ACCEPT-012: No ADR files or manifests are modified [task constraint; git diff].
- OG-ACCEPT-013: No Atlassian-named microservice boundary is introduced [ADR-0321 D-085; PRD].
- OG-ACCEPT-014: Public correction rollback uses supersession instead of deletion [ADR-0168; IP-030].
- OG-ACCEPT-015: Identity-mapping rows prevent imported vendor admins from becoming authority [IP-002; ADR-0321 D-085].
- OG-ACCEPT-016: Source-vendor metadata remains migration evidence, not authorization context [IP-002; PRD].
- OG-ACCEPT-017: Subscription sunset appears only after rollback and SLO gates [ADR-0321 D-085; IP-025].
- OG-ACCEPT-018: Final report must include path, line count, citation count, and remaining thin IP backlog [task constraint; final evidence].
