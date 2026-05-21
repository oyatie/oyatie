# IP-028 Healthcare Integration break-glass justification review

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-028-break-glass-justification-review.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Status: authoring-ready
Owner: axis-healthcare-integration
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/capabilities/break-glass-authorize.yaml; microservices/healthcare-integration/policies/local-breakglass-access-control.cedar; microservices/healthcare-integration/policy/emergency-services-bypass.cedar; microservices/healthcare-integration/runbooks/local-breakglass-audit-review.md; microservices/healthcare-integration/runbooks/break-glass-audit-review.md; microservices/healthcare-integration/slos/local-hipaa-access-review-latency.openslo.yaml; microservices/healthcare-integration/dashboards/local-policy-decisions.json
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Define the implementation plan for break-glass access justification and post-event review.
- Keep emergency access challenge-free for clean emergency-services paths while preserving audit evidence.
- Require reviewer adjudication after emergency access, not before life-safety access.
- Bind every break-glass event to Cedar, tenant scope, audit-chain evidence, and pack rules.
- Separate emergency access from standing privilege escalation.
- Prevent support operators from using break-glass as a routine bypass.
- Keep this IP documentation-only and limited to the assigned file.

## Break-glass request fields
- Field 001: tenant_id identifies the tenant context.
- Field 002: principal_id identifies the requesting clinician, operator, or worker.
- Field 003: audience_type identifies emergency-services, healthcare operator, support operator, auditor, or automated worker.
- Field 004: purpose identifies emergency-treatment, continuity-of-care, public-health, or approved pack purpose.
- Field 005: data_class identifies break_glass_event, fhir_resource, clinical_consent, or hl7_message.
- Field 006: patient_scope_id identifies the patient or MPI candidate set.
- Field 007: emergency_context_id identifies the incident or care context.
- Field 008: justification_code identifies the allowed reason.
- Field 009: free_text_justification is retained but never used as policy authority.
- Field 010: requested_scope identifies resources, routes, and time window.
- Field 011: max_duration_minutes caps emergency access.
- Field 012: home_cell identifies enforcement locality.
- Field 013: jurisdiction_code identifies pack and regulator overlays.
- Field 014: source_system_id identifies the clinical system.
- Field 015: consent_override_reason identifies why consent cannot be checked first.
- Field 016: reviewer_queue_id identifies post-event review ownership.
- Field 017: audit_event_class identifies ADR-0263 event family.
- Field 018: workflow_run_id identifies review workflow.
- Field 019: dealset_reference identifies billable vendor access where applicable.
- Field 020: notification_plan_id identifies patient or regulator notification obligations.

## Pre-access decision rules
- Rule 001: allow clean emergency-services path without bot challenge.
- Rule 002: require tenant_id and principal_id even for emergency access.
- Rule 003: require audience_type to be approved for break-glass.
- Rule 004: require purpose to be emergency or explicitly pack-approved.
- Rule 005: require requested_scope to be narrower than tenant-wide access.
- Rule 006: require max_duration_minutes within pack cap.
- Rule 007: require source-system credential sidecar before source fetch.
- Rule 008: require audit-chain write before protected resource delivery.
- Rule 009: require Cedar permit before domain command.
- Rule 010: deny access if policy fragment is not soaked per ADR-0294.
- Rule 011: deny access if data_class is outside route or resource scope.
- Rule 012: deny access if home_cell or jurisdiction forbids movement.
- Rule 013: deny access if reviewer_queue_id cannot be assigned.
- Rule 014: deny support-operator access without additional tenant-admin approval.
- Rule 015: deny broad export during emergency unless regulator pack permits it.
- Rule 016: allow read-only emergency FHIR access where treatment requires it.
- Rule 017: allow ACK route continuation where blocking would harm care.
- Rule 018: record consent override reason but preserve consent review follow-up.
- Rule 019: create DealSet hold for billable emergency vendor exchange.
- Rule 020: emit refusal evidence for every deny.

## Post-event review states
- State 001: opened means access was granted or denied and review record exists.
- State 002: evidence-collected means audit, route, FHIR, consent, and source records are attached.
- State 003: reviewer-assigned means an eligible reviewer owns the case.
- State 004: awaiting-justification means required structured reason is incomplete.
- State 005: awaiting-patient-notice means pack requires notice before closure.
- State 006: accepted means reviewer confirmed necessity and scope.
- State 007: narrowed means reviewer accepts emergency need but finds overbroad scope.
- State 008: rejected means reviewer finds unjustified access.
- State 009: escalated means compliance or security review is required.
- State 010: tenant-admin-notified means tenant owner received required notice.
- State 011: regulator-notice-required means pack deadline is active.
- State 012: regulator-notice-sent means notice evidence is attached.
- State 013: reimbursement-hold means DealSet settlement remains blocked.
- State 014: reimbursement-released means DealSet hold cleared.
- State 015: remediation-open means training, policy, or access adjustment is required.
- State 016: remediation-closed means corrective action is complete.
- State 017: superseded means newer evidence changed the review.
- State 018: archived means retention and export controls are active.
- State 019: appealed means tenant or principal challenges decision.
- State 020: final means all mandatory review evidence is closed.

## Review evidence requirements
- Evidence 001: original break-glass request.
- Evidence 002: Cedar decision id.
- Evidence 003: audit event id for grant or denial.
- Evidence 004: resource ids or route ids accessed.
- Evidence 005: requested_scope and actual_scope comparison.
- Evidence 006: source_system_id and credential sidecar handle class.
- Evidence 007: consent status at access time.
- Evidence 008: consent override reason.
- Evidence 009: patient_scope_id or MPI candidate id.
- Evidence 010: reviewer identity and role.
- Evidence 011: notification plan.
- Evidence 012: regulator pack obligations.
- Evidence 013: DealSet hold or release.
- Evidence 014: transport protocol and downgrade state.
- Evidence 015: workflow_run_id.
- Evidence 016: dashboard incident link.
- Evidence 017: runbook invocation id.
- Evidence 018: remediation tasks.
- Evidence 019: appeal status.
- Evidence 020: final export hash.

## Break-Glass Justification Benchmark Displacement
- Displacement claim: this IP measures competitors against temporary-access justification, bounded scope, expiry, post-event reviewer decision, and regulator-ready evidence.
- Non-generic rule: a vendor comparison that does not distinguish emergency reason, patient scope, closeout state, and abuse review does not satisfy this IP.
- Redox displacement: Redox emergency connectivity is displaced by tenant-scoped break-glass review with signed evidence.
- Redox proof: access is not accepted until post-event review state is closed.
- Rhapsody displacement: route-console overrides are displaced by Cedar-governed emergency route continuation.
- Rhapsody proof: ACK and route evidence attach to the review packet.
- InterSystems IRIS for Health displacement: suite-level emergency controls are displaced by flat healthcare-integration review workflows.
- InterSystems proof: PRD, architecture, and capability record keep ownership local.
- Lyniate/Corepoint displacement: operator channel overrides are displaced by reviewer-owned justification states.
- Lyniate/Corepoint proof: runbooks own review and remediation.
- Mirth Connect displacement: script toggles are displaced by structured request fields and Cedar gates.
- Mirth proof: free text is retained as evidence but never policy authority.
- NextGate displacement: identity matching confidence is not emergency authorization.
- NextGate proof: patient_scope_id must still attach to scope and post-event review.
- Health Catalyst displacement: analytics-based anomaly review is displaced by operational break-glass evidence and regulator-ready export.
- Health Catalyst proof: every review packet includes event, metric, trace, log, and audit ids.

## Failure modes
- Failure 001: missing tenant_id denies access.
- Failure 002: missing principal_id denies access.
- Failure 003: unsupported audience_type denies access.
- Failure 004: excessive requested_scope narrows or denies access.
- Failure 005: audit-chain unavailable blocks protected resource delivery.
- Failure 006: credential sidecar unavailable blocks source fetch.
- Failure 007: reviewer queue unavailable grants only if emergency path requires it and opens escalation.
- Failure 008: consent unavailable grants only under emergency purpose and opens consent review.
- Failure 009: break-glass timer expiry revokes temporary access.
- Failure 010: repeated unjustified use opens remediation and policy review.
- Failure 011: support-operator misuse opens security incident.
- Failure 012: pack notification deadline missed opens compliance incident.
- Failure 013: DealSet hold not released blocks vendor settlement.
- Failure 014: transport downgrade without disclosure denies access.
- Failure 015: local reference missing in review packet returns REVISE.

## Capacity and performance
- Capacity 001: pre-access emergency decision target must preserve life-safety path.
- Capacity 002: post-event review latency follows local-hipaa-access-review-latency.openslo.yaml.
- Capacity 003: review queues partition by tenant, cell, pack, severity, and reviewer role.
- Capacity 004: metrics avoid raw patient identifiers.
- Capacity 005: emergency access counter dimensions include reason_code, audience_type, pack, and cell.
- Capacity 006: repeated-principal detection uses privacy-preserving tenant-local aggregation.
- Capacity 007: audit export jobs are idempotent.
- Capacity 008: notification deadlines are scheduled per pack.
- Capacity 009: reviewer overload triggers escalation before deadlines burn.
- Capacity 010: break-glass access duration caps are enforced by timer jobs.
- Capacity 011: source-system outages are recorded separately from policy denials.
- Capacity 012: emergency traffic does not bypass residency.
- Capacity 013: support access has stricter limits than clinician emergency access.
- Capacity 014: appeal workflows do not reopen resource access automatically.
- Capacity 015: reimbursement hold release is asynchronous and auditable.

## Observability
- Event `oya.healthcare.integration.breakglass.requested` records request.
- Event `oya.healthcare.integration.breakglass.granted` records grant.
- Event `oya.healthcare.integration.breakglass.denied` records denial.
- Event `oya.healthcare.integration.breakglass.review.opened` records review creation.
- Event `oya.healthcare.integration.breakglass.review.closed` records final review.
- Metric `healthcare_integration_breakglass_access_total` dimensions: decision, reason_code, audience_type, cell, pack.
- Metric `healthcare_integration_breakglass_review_latency_seconds` dimensions: status, reviewer_role, pack.
- Metric `healthcare_integration_breakglass_scope_narrowed_total` dimensions: reason_code, pack, cell.
- Trace span `healthcare.breakglass.evaluate` wraps policy, audit, resource access, and review creation.
- Log schema includes request_id, decision_id, audit_event_id, reviewer_queue_id, workflow_run_id, and notification_plan_id.
- Dashboard reference: dashboards/local-policy-decisions.json.
- Runbook reference: runbooks/local-breakglass-audit-review.md.
- Runbook reference: runbooks/break-glass-audit-review.md.
- Policy reference: policies/local-breakglass-access-control.cedar.
- Policy reference: policy/emergency-services-bypass.cedar.

## Implementation steps
- Step 001: Add break-glass request value object.
- Step 002: Add emergency decision usecase.
- Step 003: Add review aggregate and state transitions.
- Step 004: Add duration cap enforcement worker.
- Step 005: Add notification scheduling worker.
- Step 006: Add DealSet hold integration for billable access.
- Step 007: Add Cedar policy tests for allowed and denied paths.
- Step 008: Add audit-chain event emission for request, grant, deny, review, and closure.
- Step 009: Add dashboard panels for review latency and denials.
- Step 010: Add runbook links for audit review.
- Step 011: Add contract examples for granted, denied, narrowed, and rejected review.
- Step 012: Add property tests for state transitions.
- Step 013: Add replay tests for review packet export.
- Step 014: Add pack tests for notification deadlines.
- Step 015: Add benchmark displacement evidence to review packet.

## Tests and evidence
- Test 001: line count for this IP is at least 200.
- Test 002: ADR scan finds the full binding ADR list.
- Test 003: benchmark scan finds all seven named competitors.
- Test 004: local reference scan finds break-glass-authorize.yaml.
- Test 005: local reference scan finds both break-glass Cedar policy references.
- Test 006: local reference scan finds both break-glass runbooks.
- Test 007: local reference scan finds local-hipaa-access-review-latency.openslo.yaml.
- Test 008: review confirms emergency path is challenge-free when clean but still audited.
- Test 009: review confirms ADR-0321 was not edited.
- Test 010: review confirms no oya vcs verify, done, or promote was run.

## Rollback
- Rollback 001: disable new review workflow for affected tenant only.
- Rollback 002: keep stricter prior break-glass policy active.
- Rollback 003: revoke temporary access when timer or review fails.
- Rollback 004: retain original grant and denial events.
- Rollback 005: do not delete review packets.
- Rollback 006: keep DealSet holds until review evidence is accepted.
- Rollback 007: restore prior Cedar fragment only after soak-window rules permit.
- Rollback 008: notify tenant admin when rollback changes access posture.
- Rollback 009: export affected evidence packets for audit.
- Rollback 010: open remediation for repeated unjustified access.

## Acceptance criteria
- AC01: Emergency access requires tenant, principal, audience, purpose, scope, duration, and audit class.
- AC02: Clean emergency-services access avoids bot friction but not audit.
- AC03: Every granted access opens post-event review.
- AC04: Review packets include request, decision, resource scope, consent state, reviewer, notice, and settlement evidence.
- AC05: Free text is evidence only and never policy authority.
- AC06: Support-operator break-glass is stricter than clinician emergency access.
- AC07: DealSet holds remain until review permits release.
- AC08: Pack notification deadlines are explicit.
- AC09: All seven named benchmarks are explicitly displaced.
- AC10: This plan remains scoped to the assigned IP file.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-028-break-glass-justification-review.md:187` - - Step 008: Add audit-chain event emission for request, grant, deny, review, and closure..
