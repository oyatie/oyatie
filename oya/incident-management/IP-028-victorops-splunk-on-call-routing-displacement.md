# IP-028 Incident Management victorops-splunk-on-call-routing-displacement

Service: incident-management
ChangeSet scope: microservices/incident-management/IP-028-victorops-splunk-on-call-routing-displacement.md
Benchmark displacement: VictorOps / Splunk On-Call alert routing, escalation, timeline, and Splunk event-source bridge
Primary sources: ADR-0321 D-084; ADR-0169; ADR-0263; ADR-0003; PRD-incident-management
Related IPs: IP-001, IP-002, IP-006, IP-009, IP-011, IP-016, IP-021
Non-goals: no SIEM service edit; no observability manifest edit; no Splunk-named boundary
Acceptance floor: >=200 lines; numbered rows cite local authority

## Objective
- VO-OBJ-001: Displace VictorOps alert routing with tenant-owned event rules [ADR-0321 D-084; IP-006].
- VO-OBJ-002: Displace Splunk On-Call escalation with Cedar-gated escalation-evaluate [ADR-0321 D-084; IP-002].
- VO-OBJ-003: Displace Splunk incident timeline with incident_timeline audit events [PRD; IP-011].
- VO-OBJ-004: Displace source routing keys with credential-sidecar references [IP-009; ADR-0321 D-084].
- VO-OBJ-005: Displace alert dedup behavior with ADR-0169 idempotency envelopes [ADR-0169; IP-006].
- VO-OBJ-006: Displace chatops timeline notes with tenant-local incident-room entries [PRD; IP-001].
- VO-OBJ-007: Displace routing rules with event_orchestration transforms [ADR-0321 D-084; IP-004].
- VO-OBJ-008: Displace alert recovery events with resolve command [ADR-0321 D-084; IP-005].
- VO-OBJ-009: Displace notification policies with comms rail preference projection [PRD; competitor-parity].
- VO-OBJ-010: Displace Splunk source role with source_vendor metadata only [IP-002; ADR-0263].
- VO-OBJ-011: Displace raw log authority with normalized observability emission [ADR-0263; IP-011].
- VO-OBJ-012: Displace vendor audit logs with audit-chain evidence [ADR-0003; PRD].
- VO-OBJ-013: Displace external schedule gaps with local no-gap validation [IP-001; ADR-0321 D-084].
- VO-OBJ-014: Displace external escalation cycles with local graph cycle refusal [IP-002; ADR-0321 D-084].
- VO-OBJ-015: Displace event-source retries with replay-safe webhook handling [ADR-0169; IP-016].
- VO-OBJ-016: Displace live migration risk with shadow route and SLO promotion [IP-021; ADR-0321 D-084].
- VO-OBJ-017: Displace rollback uncertainty with compensating command bundles [IP-016; IP-025].
- VO-OBJ-018: Displace subscription lock-in with source sunset evidence [ADR-0321 D-084; PRD].

## Source export intake
- VO-INTAKE-001: Export routing keys with checksum and credential binding target [IP-009; IP-016].
- VO-INTAKE-002: Export alert rules with matching criteria and target escalation [ADR-0321 D-084; IP-006].
- VO-INTAKE-003: Export incidents with source incident id and timeline rows [PRD; IP-016].
- VO-INTAKE-004: Export acknowledgements with responder aliases and timestamps [IP-001; IP-016].
- VO-INTAKE-005: Export recoveries with source close reason [ADR-0321 D-084; IP-005].
- VO-INTAKE-006: Export schedules with rotations and overrides [IP-001; ADR-0321 D-084].
- VO-INTAKE-007: Export escalation policies with levels and delays [IP-002; ADR-0321 D-084].
- VO-INTAKE-008: Export teams with identity aliases only [IP-002; PRD].
- VO-INTAKE-009: Export notification preferences as comms rail candidates [PRD; competitor-parity].
- VO-INTAKE-010: Export Splunk saved searches as event-source references [ADR-0263; IP-006].
- VO-INTAKE-011: Export Splunk alert payload samples for replay fixtures [ADR-0169; IP-016].
- VO-INTAKE-012: Export webhook retry metadata for idempotency validation [ADR-0169; IP-006].
- VO-INTAKE-013: Export chat transcripts as incident-room annotations [PRD; IP-011].
- VO-INTAKE-014: Export runbook links as workflow template references [IP-004; PRD].
- VO-INTAKE-015: Export audit log rows where available [ADR-0003; IP-023].
- VO-INTAKE-016: Export source account status for sunset checklist [ADR-0321 D-084; IP-025].
- VO-INTAKE-017: Export integration state as migration metadata, not authority [IP-002; PRD].
- VO-INTAKE-018: Export alert volume histogram for capacity admission [IP-018; IP-017].

## Transform and ontology
- VO-ONTO-001: Map source incident to incident.incident with tenant timeline [ADR-0321 D-084; PRD].
- VO-ONTO-002: Map routing key to event_rule credential binding [IP-009; IP-006].
- VO-ONTO-003: Map matching rule to incident.event_rule [ADR-0321 D-084; IP-003].
- VO-ONTO-004: Map escalation policy to incident.escalation_policy [ADR-0321 D-084; IP-002].
- VO-ONTO-005: Map schedule to incident.on_call_schedule [ADR-0321 D-084; IP-001].
- VO-ONTO-006: Map team to incident.team after identity proof [IP-002; PRD].
- VO-ONTO-007: Map user to principal alias, never direct principal [IP-002; IP-001].
- VO-ONTO-008: Map acknowledgement to page ack command [ADR-0321 D-084; IP-005].
- VO-ONTO-009: Map recovery to resolve command [ADR-0321 D-084; IP-005].
- VO-ONTO-010: Map chat note to incident timeline annotation [PRD; IP-011].
- VO-ONTO-011: Map Splunk saved search to observability source reference [ADR-0263; IP-011].
- VO-ONTO-012: Map Splunk payload fields through emission contract vocabulary [ADR-0263; IP-003].
- VO-ONTO-013: Map dedup field to idempotency key [ADR-0169; IP-006].
- VO-ONTO-014: Map severity to tenant severity policy [PRD; IP-001].
- VO-ONTO-015: Map runbook link to workflow template id [IP-004; PRD].
- VO-ONTO-016: Map source audit row to audit-chain provenance [ADR-0003; IP-011].
- VO-ONTO-017: Map source token to credential-sidecar reference [IP-009; IP-024].
- VO-ONTO-018: Map migration batch to rollback bundle [IP-016; IP-025].

## Command contracts
- VO-CMD-001: routing-key.import-preview validates source checksum [IP-016; IP-009].
- VO-CMD-002: routing-key.bind creates credential-sidecar reference [IP-009; IP-005].
- VO-CMD-003: alert-rule.import-preview validates event rule transform [IP-006; IP-016].
- VO-CMD-004: alert.ingest accepts idempotency_key and dedup_key [ADR-0169; IP-005].
- VO-CMD-005: alert.route invokes page-dispatch [ADR-0321 D-084; IP-005].
- VO-CMD-006: alert.acknowledge invokes page acknowledgement [IP-005; IP-001].
- VO-CMD-007: alert.escalate invokes escalation-evaluate [IP-005; IP-002].
- VO-CMD-008: alert.recover invokes resolve command [ADR-0321 D-084; IP-005].
- VO-CMD-009: timeline.note records incident-room annotation [PRD; IP-005].
- VO-CMD-010: schedule.import-preview validates no gaps [IP-001; IP-016].
- VO-CMD-011: policy.import-preview validates no cycles [IP-002; IP-016].
- VO-CMD-012: saved-search.bind records observability source reference [ADR-0263; IP-011].
- VO-CMD-013: replay.fixture-run disables live paging [ADR-0169; IP-016].
- VO-CMD-014: shadow-route.start starts dual-route comparison [IP-021; ADR-0321 D-084].
- VO-CMD-015: cutover.complete records SLO gate success [IP-021; IP-025].
- VO-CMD-016: vendor-sunset.record records source retirement [ADR-0321 D-084; IP-025].
- VO-CMD-017: rollback.execute appends compensation record [IP-016; PRD].
- VO-CMD-018: evidence.export returns tenant-readable provenance [ADR-0003; IP-023].

## Async events
- VO-EVT-001: routing_key.previewed emits checksum and tenant [IP-016; IP-006].
- VO-EVT-002: routing_key.bound emits credential reference [IP-009; IP-006].
- VO-EVT-003: alert_rule.previewed emits transform version [IP-016; IP-006].
- VO-EVT-004: alert.ingested emits dedup and idempotency keys [ADR-0169; IP-006].
- VO-EVT-005: alert.routed emits page id and policy version [IP-006; IP-002].
- VO-EVT-006: alert.acknowledged emits ack latency [IP-021; IP-006].
- VO-EVT-007: alert.escalated emits graph checksum [IP-002; IP-006].
- VO-EVT-008: alert.recovered emits resolution state [PRD; IP-006].
- VO-EVT-009: timeline.note.recorded emits timeline checksum [IP-011; IP-006].
- VO-EVT-010: schedule.imported emits no-gap result [IP-001; IP-006].
- VO-EVT-011: policy.imported emits cycle result [IP-002; IP-006].
- VO-EVT-012: saved_search.bound emits observability source [ADR-0263; IP-011].
- VO-EVT-013: replay.fixture.completed emits side-effect-free result [ADR-0169; IP-016].
- VO-EVT-014: shadow_route.started emits dual-route window [IP-021; ADR-0321 D-084].
- VO-EVT-015: cutover.completed emits SLO evidence [IP-021; IP-025].
- VO-EVT-016: vendor_sunset.recorded emits exit evidence [IP-025; PRD].
- VO-EVT-017: rollback.executed emits compensation checksum [IP-016; IP-025].
- VO-EVT-018: evidence.exported emits audit-chain bundle id [ADR-0003; IP-023].

## Cedar gates
- VO-CEDAR-001: routing-key bind denies without credential-sidecar grant [IP-002; IP-009].
- VO-CEDAR-002: alert ingest denies without tenant match [IP-002; IP-001].
- VO-CEDAR-003: alert route denies without event_rule ownership [IP-002; IP-006].
- VO-CEDAR-004: alert ack denies without responder mapping [IP-002; IP-001].
- VO-CEDAR-005: alert escalate denies on stale policy graph [IP-002; ADR-0321 D-084].
- VO-CEDAR-006: alert recover denies without open timeline [IP-002; PRD].
- VO-CEDAR-007: schedule import denies on coverage gap [IP-002; IP-001].
- VO-CEDAR-008: policy import denies on cycle [IP-002; ADR-0321 D-084].
- VO-CEDAR-009: saved-search bind denies raw token payloads [IP-002; IP-009].
- VO-CEDAR-010: replay denies live page side effects [IP-002; IP-016].
- VO-CEDAR-011: shadow route denies during SLO burn [IP-002; IP-021].
- VO-CEDAR-012: cutover denies without rollback bundle [IP-002; IP-016].
- VO-CEDAR-013: vendor sunset denies with active source dependency [IP-002; IP-025].
- VO-CEDAR-014: evidence export denies non-auditor audience [IP-002; IP-023].
- VO-CEDAR-015: timeline note denies cross-tenant room id [IP-002; IP-001].
- VO-CEDAR-016: runbook link denies without template approval [IP-002; IP-004].
- VO-CEDAR-017: public note denies unless stakeholder disclosure is approved [IP-002; IP-030].
- VO-CEDAR-018: source-vendor role denies as standalone authority [IP-002; PRD].

## Migration and cutover
- VO-CUT-001: Stage routing keys before alert-rule replay [IP-009; IP-016].
- VO-CUT-002: Stage alert rules before live dual-route [IP-006; IP-016].
- VO-CUT-003: Stage schedules before escalation policies [IP-001; IP-016].
- VO-CUT-004: Stage policies before ack and escalation drills [IP-002; IP-021].
- VO-CUT-005: Stage saved search references before Splunk payload replay [ADR-0263; IP-016].
- VO-CUT-006: Stage identity mappings before responder drills [IP-002; IP-001].
- VO-CUT-007: Run replay fixtures without live pages [ADR-0169; IP-016].
- VO-CUT-008: Run routing comparison by dedup key [ADR-0169; IP-021].
- VO-CUT-009: Run ack drill against local SLO [IP-021; slos/local-page-to-acknowledge.openslo.yaml].
- VO-CUT-010: Run room drill against local SLO [IP-021; slos/local-war-room-creation-latency.openslo.yaml].
- VO-CUT-011: Run policy refusal drill for stale graph [IP-002; IP-021].
- VO-CUT-012: Run schedule gap drill for import preview [IP-001; IP-021].
- VO-CUT-013: Run observability emission check against ADR-0263 [ADR-0263; IP-011].
- VO-CUT-014: Run audit-chain check against ADR-0003 [ADR-0003; IP-011].
- VO-CUT-015: Run capacity check for alert storm volume [IP-018; IP-012].
- VO-CUT-016: Run cost check for dual-route notification fanout [IP-017; PRD].
- VO-CUT-017: Run cutover after green SLO promotion [IP-021; IP-025].
- VO-CUT-018: Run vendor sunset only after rollback evidence remains valid [IP-025; ADR-0321 D-084].

## Observability
- VO-OBS-001: Metric alert.ingest.count includes tenant, source_vendor, and routing key hash [ADR-0263; IP-011].
- VO-OBS-002: Metric alert.route.latency includes event_rule_id and policy_version [IP-011; IP-006].
- VO-OBS-003: Metric page.ack.latency links to page-to-ack SLO [IP-021; IP-011].
- VO-OBS-004: Metric escalation.cycle.refusal counts failed policy imports [IP-002; IP-011].
- VO-OBS-005: Metric schedule.gap.refusal counts failed schedule imports [IP-001; IP-011].
- VO-OBS-006: Metric webhook.retry.count follows ADR-0169 dimensions [ADR-0169; IP-011].
- VO-OBS-007: Metric splunk.payload.replay.count tracks fixture coverage [ADR-0263; IP-016].
- VO-OBS-008: Metric credential.bind.refusal counts raw token failures [IP-009; IP-011].
- VO-OBS-009: Metric audit.emission.lag links to audit SLO [ADR-0003; slos/audit-emission-lag.openslo.yaml].
- VO-OBS-010: Metric shadow.route.diff counts source/local mismatches [IP-021; ADR-0321 D-084].
- VO-OBS-011: Metric cutover.blocker.count groups by integration family [IP-025; PRD].
- VO-OBS-012: Metric rollback.ready.count groups by object type [IP-016; IP-025].
- VO-OBS-013: Metric alert.storm.throttle links to abuse defense [IP-012; IP-018].
- VO-OBS-014: Metric tenant.cost.dualroute links to cost budget [IP-017; PRD].
- VO-OBS-015: Metric policy.decision.count links to local policy dashboard [IP-002; dashboards/local-policy-decisions.json].
- VO-OBS-016: Metric timeline.note.count tracks room annotation import [PRD; IP-011].
- VO-OBS-017: Metric evidence.bundle.complete tracks audit packet readiness [IP-023; PRD].
- VO-OBS-018: Metric source.sunset.ready tracks final vendor retirement [IP-025; ADR-0321 D-084].

## Rollback
- VO-RB-001: Roll back routing key bind by revoking local credential reference [IP-009; IP-016].
- VO-RB-002: Roll back alert rule by disabling local event rule [IP-006; IP-016].
- VO-RB-003: Roll back incident import by compensating local timeline creation [IP-016; PRD].
- VO-RB-004: Roll back schedule import by restoring layer checksum [IP-001; IP-016].
- VO-RB-005: Roll back policy import by restoring graph checksum [IP-002; IP-016].
- VO-RB-006: Roll back saved search bind by disabling source reference [ADR-0263; IP-011].
- VO-RB-007: Roll back live page route by returning to source with explicit permit [IP-002; IP-021].
- VO-RB-008: Roll back public stakeholder note by supersession [IP-030; PRD].
- VO-RB-009: Roll back chat annotation by marking source note inactive [PRD; IP-011].
- VO-RB-010: Roll back replay batch by appending compensation records [IP-016; ADR-0169].
- VO-RB-011: Roll back cutover by restoring dual-route phase [IP-021; IP-025].
- VO-RB-012: Roll back vendor sunset only when source account remains active [IP-025; ADR-0321 D-084].
- VO-RB-013: Roll back source token quarantine by rotating local secret [IP-009; IP-024].
- VO-RB-014: Roll back policy bundle by preserving refusal evidence [IP-002; IP-011].
- VO-RB-015: Roll back SLO promotion by marking gate failed [IP-021; IP-025].
- VO-RB-016: Roll back audit export by issuing corrected evidence packet [IP-023; ADR-0003].
- VO-RB-017: Roll back cost attribution by recalculating dual-route window [IP-017; PRD].
- VO-RB-018: Roll back final claim by reporting this IP as implementation plan scope only [task constraint; final evidence].

## Acceptance evidence
- VO-ACCEPT-001: File line count is >=200 [task constraint; wc].
- VO-ACCEPT-002: Citation density includes local authority refs on numbered rows [task constraint; grep].
- VO-ACCEPT-003: VictorOps/Splunk displacement is expressed as incident-management surfaces only [ADR-0321 D-084; PRD].
- VO-ACCEPT-004: Splunk source bridge cites ADR-0263 observability emission contract [ADR-0263; rg].
- VO-ACCEPT-005: Audit replacement cites ADR-0003 audit-chain [ADR-0003; rg].
- VO-ACCEPT-006: Retry replacement cites ADR-0169 [ADR-0169; rg].
- VO-ACCEPT-007: Credential replacement cites IP-009 [IP-009; rg].
- VO-ACCEPT-008: Schedule replacement cites IP-001 [IP-001; rg].
- VO-ACCEPT-009: Policy replacement cites IP-002 [IP-002; rg].
- VO-ACCEPT-010: Event replacement cites IP-006 [IP-006; rg].
- VO-ACCEPT-011: Backfill replacement cites IP-016 [IP-016; rg].
- VO-ACCEPT-012: SLO promotion cites IP-021 [IP-021; rg].
- VO-ACCEPT-013: No SIEM, observability manifest, ADR, journey, or neighboring service file is edited [task constraint; git diff].
- VO-ACCEPT-014: No Splunk-named service boundary is introduced [PRD; competitor-parity].
- VO-ACCEPT-015: Live replay is disabled until approved cutover [ADR-0169; IP-016].
- VO-ACCEPT-016: Source-vendor metadata remains migration evidence only [IP-002; PRD].
- VO-ACCEPT-017: Rollback preserves timeline and audit history [ADR-0003; IP-016].
- VO-ACCEPT-018: Final report includes path, line count, citation count, and blockers [task constraint; final evidence].

## Wave 15 counterpart anchor
- Counterpart baseline: PagerDuty, OpsGenie, xMatters, FireHydrant, ServiceNow, and Slack define the incident-management parity envelope; this displacement IP must close its slice with tenant-scoped policy, audit, and rollback evidence.
