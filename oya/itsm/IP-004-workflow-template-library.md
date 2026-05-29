# IP-004 ITSM workflow-template-library

Service: itsm
ChangeSet scope: microservices/itsm/IP-004-workflow-template-library.md
Benchmarks: ServiceNow ITSM, Jira Service Management, BMC Remedy, Zendesk Support, Freshdesk
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- workflow-template-library-objective 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- workflow-template-library-objective 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- workflow-template-library-objective 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- workflow-template-library-objective 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- workflow-template-library-objective 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- workflow-template-library-objective 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Prerequisites
- workflow-template-library-prerequisites 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- workflow-template-library-prerequisites 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- workflow-template-library-prerequisites 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- workflow-template-library-prerequisites 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- workflow-template-library-prerequisites 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- workflow-template-library-prerequisites 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Implementation steps
- workflow-template-library-implementation-steps 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- workflow-template-library-implementation-steps 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- workflow-template-library-implementation-steps 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- workflow-template-library-implementation-steps 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- workflow-template-library-implementation-steps 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- workflow-template-library-implementation-steps 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Tests and evidence
- workflow-template-library-tests-and-evidence 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- workflow-template-library-tests-and-evidence 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- workflow-template-library-tests-and-evidence 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- workflow-template-library-tests-and-evidence 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- workflow-template-library-tests-and-evidence 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- workflow-template-library-tests-and-evidence 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Rollback
- workflow-template-library-rollback 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- workflow-template-library-rollback 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- workflow-template-library-rollback 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- workflow-template-library-rollback 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- workflow-template-library-rollback 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- workflow-template-library-rollback 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Acceptance criteria
- workflow-template-library-acceptance-criteria 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- workflow-template-library-acceptance-criteria 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- workflow-template-library-acceptance-criteria 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- workflow-template-library-acceptance-criteria 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- workflow-template-library-acceptance-criteria 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- workflow-template-library-acceptance-criteria 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Batch B substance audit
- Substance status 001: the seed file was 55 lines and did not define any workflow template inputs, states, guards, compensation steps, or evidence outputs.
- Substance status 002: this packet defines the ITSM workflow template library used by incident, problem, change, service catalog, CMDB, and major-incident bridge flows.
- Substance status 003: displaced benchmarks are ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, and Freshservice.
- Substance status 004: workflow templates must expose ITIL-grade operator controls without copying vendor suite state machines or hidden approval shortcuts.
- Substance status 005: docs/standards/documentation-rigor.md section 1.1 requires the template descriptions to be implementable without reverse-engineering product screens.
- Substance status 006: ADR-0105 keeps template storage, execution, worker dispatch, policy, and contract layers separated.
- Substance status 007: ADR-0316 keeps vendor labels as capability tiers and prevents a ServiceNow-style suite boundary from becoming the template owner.
- Substance status 008: ADR-0321 requires B2B leader parity at a domain-specific surface level, not just generic workflow invocation.

## Template inventory
- Template 001: `itsm.incident.triage.v1` opens or updates an incident ticket, assigns impact and urgency, computes priority, and records requester visibility.
- Template 002: `itsm.incident.sla-clock.v1` starts, pauses, resumes, breaches, and recomputes SLA evidence without rewriting sealed breach history.
- Template 003: `itsm.problem.investigation.v1` links incidents, records root-cause hypotheses, publishes workarounds, and closes known-error evidence.
- Template 004: `itsm.change.standard.v1` handles pre-approved standard changes with limited risk, fixed implementation window, and rollback checklist.
- Template 005: `itsm.change.normal.v1` handles normal changes with risk assessment, approval routing, freeze-window check, implementation, and verification.
- Template 006: `itsm.change.emergency.v1` handles emergency changes with breakglass reason, expedited approval, rollback evidence, and post-change review.
- Template 007: `itsm.catalog.request.v1` handles requester submission, entitlement check, approval, fulfillment, completion, and cancellation.
- Template 008: `itsm.catalog.publish.v1` handles draft, schema review, entitlement review, approval, release, and revocation of service catalog items.
- Template 009: `itsm.cmdb.discovery-sync.v1` handles imported CI discovery, confidence gating, relation validation, and projection into ontology.
- Template 010: `itsm.cmdb.relation-repair.v1` handles relation drift review, tenant boundary check, remediation, and audit closeout.
- Template 011: `itsm.major-incident.bridge.v1` hands ITSM incident evidence into incident-management while preserving bidirectional audit preimages.
- Template 012: `itsm.vendor-migration.replay.v1` replays imported ServiceNow, Jira, BMC, Ivanti, or Freshservice events through canonical guards.

## Common template inputs
- Input 001: tenant context from IP-001 is mandatory for every template instance.
- Input 002: Cedar policy decision id from IP-002 is mandatory before any mutating step executes.
- Input 003: ontology object ref from IP-003 is mandatory where a ticket, change, problem, catalog item, CI, or relation already exists.
- Input 004: source_system_kind distinguishes ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, Freshservice, and native Oyatie.
- Input 005: source_system_ref is retained as provenance and never used as the workflow run id.
- Input 006: workflow_template_id must include semantic version so replay and rollback can choose the original path.
- Input 007: compliance_pack_set controls step additions, step removals, retention, egress, approval, and evidence seal requirements.
- Input 008: deal_set_id is mandatory for marketplace-backed service catalog templates.
- Input 009: requester_visibility_class controls which fields can be shown to non-ITIL requesters.
- Input 010: operator_visibility_class controls which fields can be shown to ITIL operators, auditors, and delegated administrators.
- Input 011: data_class controls residency and audit behavior for incident_ticket, change_request, problem_record, service_catalog_item, cmdb_ci, and cmdb_relation.
- Input 012: idempotency_key prevents duplicate ticket opens, duplicated approvals, repeated catalog publication, and repeated CMDB writes.

## State machine contracts
- State machine 001: Incident triage starts at `received`, moves through `classified`, `assigned`, `in_progress`, `resolved`, and `closed`.
- State machine 002: Incident triage can move to `major_bridge_requested` only when bridge policy and incident-management handoff checks pass.
- State machine 003: SLA clock starts at `not_started`, moves through `running`, `paused`, `breached`, `met`, and `sealed`.
- State machine 004: SLA clock cannot move from `breached` back to `running`; recompute creates separate evidence instead.
- State machine 005: Problem investigation starts at `candidate`, moves through `correlated`, `hypothesized`, `workaround_published`, `root_cause_confirmed`, and `closed`.
- State machine 006: Problem investigation rejects a transition that would create a problem-link graph cycle.
- State machine 007: Standard change starts at `draft`, moves through `preapproved`, `scheduled`, `implemented`, `verified`, and `closed`.
- State machine 008: Normal change starts at `draft`, moves through `risk_assessed`, `approval_pending`, `approved`, `scheduled`, `implemented`, `verified`, and `closed`.
- State machine 009: Emergency change starts at `breakglass_requested`, moves through `expedited_approved`, `implemented`, `stabilized`, `reviewed`, and `closed`.
- State machine 010: Service catalog request starts at `submitted`, moves through `entitlement_checked`, `approval_pending`, `fulfillment_running`, `fulfilled`, and `closed`.
- State machine 011: Catalog publication starts at `draft`, moves through `schema_reviewed`, `entitlement_reviewed`, `approved`, `published`, and `revoked`.
- State machine 012: CMDB sync starts at `discovered`, moves through `confidence_checked`, `relation_validated`, `projected`, and `verified`.

## Step guards
- Guard 001: Every first mutating step calls tenant validation from IP-001.
- Guard 002: Every mutating step calls Cedar default-deny evaluation from IP-002.
- Guard 003: Every projection step calls ontology object and edge validation from IP-003.
- Guard 004: Every approval step checks separation-of-duty requirements under active pack overlays.
- Guard 005: Every change scheduling step checks freeze windows and active maintenance windows.
- Guard 006: Every catalog step checks requester entitlement and fulfiller authority separately.
- Guard 007: Every CMDB relation step checks endpoint tenant equality and confidence threshold.
- Guard 008: Every major-incident bridge step requires incident-management acceptance and status update policy.
- Guard 009: Every marketplace-backed catalog step checks DealSet settlement evidence under ADR-0314.
- Guard 010: Every source-vendor replay step checks payload digest, projection version, and original audit preimage.
- Guard 011: Every rollback step checks that sealed audit-chain events remain immutable.
- Guard 012: Every egress step checks data residency and pack-specific disclosure rules.

## Vendor displacement behavior
- Vendor displacement 001: ServiceNow flow designer imports are converted to template metadata and cannot execute as unreviewed scripts.
- Vendor displacement 002: ServiceNow incident and change workflows map to canonical templates only after table and state normalization passes.
- Vendor displacement 003: Jira Service Management automation rules map to guards and transition actions, not direct database writes.
- Vendor displacement 004: Jira queue and SLA behavior maps to explicit SLA clock templates and requester visibility classes.
- Vendor displacement 005: BMC Helix workflow forms map to canonical state machines with declared data classes.
- Vendor displacement 006: BMC Helix support-group routing maps to resolver policies and approval steps.
- Vendor displacement 007: Ivanti Neurons automation recommendations map to suggested change templates and cannot bypass approval.
- Vendor displacement 008: Ivanti discovery sync maps to CMDB confidence gates and ontology relation projection.
- Vendor displacement 009: Freshservice workflow automators map to catalog request templates with entitlement and approval separation.
- Vendor displacement 010: Freshservice asset workflows map to CMDB sync templates only when tenant relation validation passes.

## Implementation sequence
- Implementation 001: Create template ids and metadata structs in the usecase layer, with version, owner, data_class, and pack applicability.
- Implementation 002: Add state enum and transition enum for each of the twelve template families.
- Implementation 003: Add guard registry that names tenant, policy, ontology, approval, freeze, catalog, CMDB, bridge, settlement, replay, rollback, and egress guards.
- Implementation 004: Add template validation that refuses unversioned ids and unknown guard names.
- Implementation 005: Add workflow-engine adapter inputs without making workflow-engine own ITSM domain decisions.
- Implementation 006: Add template examples to contracts so REST, AsyncAPI, proto, SDK, and CLI share naming.
- Implementation 007: Add replay fixtures for each displaced benchmark source.
- Implementation 008: Add rollback fixtures for incident close, SLA recompute, problem link, change approval, catalog publish, CMDB relation, and bridge handoff.
- Implementation 009: Add metrics for template_run_count, guard_denial_count, state_transition_latency, rollback_count, and replay_drift_count.
- Implementation 010: Add audit event classes for template selected, guard denied, transition accepted, transition failed, rollback started, and rollback completed.
- Implementation 011: Add documentation cross-links from runbooks to the template ids they remediate.
- Implementation 012: Add catalog entries that bind template library crate, workflow adapter crate, worker crate, SDK crate, and test crate.

## Test matrix
- Test matrix 001: Unit test validates all twelve template ids include semantic version suffix.
- Test matrix 002: Unit test validates every template declares data_class coverage.
- Test matrix 003: Unit test validates every template declares tenant and Cedar guards before first mutation.
- Test matrix 004: Unit test validates ServiceNow imported workflow maps to incident triage or change template, not arbitrary script execution.
- Test matrix 005: Unit test validates Jira automation import maps to guard plus transition action.
- Test matrix 006: Unit test validates BMC form workflow maps to canonical state transition names.
- Test matrix 007: Unit test validates Ivanti recommendation maps to suggested change, not direct implementation.
- Test matrix 008: Unit test validates Freshservice automator maps to catalog request or publish template.
- Test matrix 009: Property test generates invalid transition orders and proves the state machine denies them.
- Test matrix 010: Property test generates vendor workflow ids and proves they cannot become canonical template ids without mapping.
- Test matrix 011: Integration test runs normal change from draft to closed with approval, freeze check, implementation, verification, and rollback evidence.
- Test matrix 012: Integration test runs emergency change with breakglass, expedited approval, stabilization, review, and audit evidence.
- Test matrix 013: Integration test runs CMDB sync with confidence pass and relation projection.
- Test matrix 014: Negative integration test rejects CMDB sync with cross-tenant relation endpoint.
- Test matrix 015: Integration test runs service catalog publish with DealSet settlement evidence.
- Test matrix 016: Negative integration test rejects catalog publish without entitlement policy.
- Test matrix 017: Integration test bridges a major incident only after incident-management accepts the handoff.
- Test matrix 018: Replay test reruns a ServiceNow incident import and reaches identical template state.
- Test matrix 019: Rollback test reverts catalog publication while preserving audit event ids.
- Test matrix 020: Metrics test verifies guard_denial_count labels include template id, tenant hash, data class, and source vendor.

## Acceptance expansion
- Acceptance expansion 001: An intern can list every required ITSM workflow template and its canonical states.
- Acceptance expansion 002: An intern can implement state transition validation without seeing vendor product screens.
- Acceptance expansion 003: An intern can explain how ServiceNow flow designer, Jira automation, BMC workflows, Ivanti recommendations, and Freshservice automators are displaced.
- Acceptance expansion 004: An intern can wire tenant, policy, ontology, approval, freeze, catalog, CMDB, bridge, settlement, replay, rollback, and egress guards in the right order.
- Acceptance expansion 005: An intern can implement normal, standard, and emergency change templates as separate state machines.
- Acceptance expansion 006: An intern can explain why SLA recompute creates new evidence rather than rewriting sealed breach state.
- Acceptance expansion 007: An intern can explain why workflow-engine executes orchestration but ITSM owns domain states and guards.
- Acceptance expansion 008: An intern can build benchmark replay fixtures for all five displaced vendors.
- Acceptance expansion 009: An intern can build rollback fixtures that preserve audit-chain evidence under ADR-0263.
- Acceptance expansion 010: An intern can produce a PR with template metadata, state machines, guard registry, fixtures, metrics, contracts, and runbook links.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 sets the intern-buildability gate for this workflow-template packet.
- Citation 002: microservices/itsm/manifest.json supplies ITSM bounded contexts, benchmark roster, layer conformance, and pack roster.
- Citation 003: microservices/itsm/PRD.md supplies incident, problem, change, service-request, and configuration-item user stories.
- Citation 004: microservices/itsm/contracts/openapi-v1.yaml supplies the action invocation contract that references workflow_run_id.
- Citation 005: ADR-0105 defines separation between ITSM domain templates and workflow-engine execution mechanics.
- Citation 006: ADR-0244 defines default-deny guard requirements before mutating transitions.
- Citation 007: ADR-0258 defines versioned template id and contract evolution requirements.
- Citation 008: ADR-0263 defines audit event discipline for transitions, denial, replay, and rollback.
- Citation 009: ADR-0314 defines DealSet evidence for marketplace-backed service catalog templates.
- Citation 010: ADR-0316 prevents vendor workflow labels from becoming Oyatie service or template boundaries.
- Citation 011: ADR-0321 defines B2B leader parity expectations for ITSM workflow depth.
- Citation 012: docs/AGENTS.md supplies the Oya VCS lifecycle that requires this template-library packet to remain tied to claim and promotion evidence.
- Citation 013: microservices/itsm/runbooks/change-freeze-override.md anchors the operational remediation path for freeze-window template failures.
- Citation 014: microservices/itsm/runbooks/local-cmdb-relation-drift.md anchors the operational remediation path for CMDB relation template failures.
- Citation 015: microservices/itsm/runbooks/service-catalog-publish-failure.md anchors the operational remediation path for catalog publication template failures.
- Citation 016: microservices/itsm/runbooks/major-incident-backlog.md anchors the operational remediation path for major incident bridge template failures.
- Citation 017: microservices/itsm/slos/local-mttr-objective.openslo.yaml anchors incident triage latency targets for the template run metrics.
- Citation 018: microservices/itsm/slos/local-change-failure-rate.openslo.yaml anchors change template quality gates.
- Citation 019: microservices/itsm/slos/local-cmdb-relation-freshness.openslo.yaml anchors CMDB sync freshness gates.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
