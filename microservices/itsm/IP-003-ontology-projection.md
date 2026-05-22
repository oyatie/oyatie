# IP-003 ITSM ontology-projection

Service: itsm
ChangeSet scope: microservices/itsm/IP-003-ontology-projection.md
Benchmarks: ServiceNow ITSM, Jira Service Management, BMC Remedy, Zendesk Support, Freshdesk
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- ontology-projection-objective 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- ontology-projection-objective 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- ontology-projection-objective 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- ontology-projection-objective 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- ontology-projection-objective 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- ontology-projection-objective 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Prerequisites
- ontology-projection-prerequisites 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- ontology-projection-prerequisites 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- ontology-projection-prerequisites 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- ontology-projection-prerequisites 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- ontology-projection-prerequisites 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- ontology-projection-prerequisites 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Implementation steps
- ontology-projection-implementation-steps 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- ontology-projection-implementation-steps 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- ontology-projection-implementation-steps 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- ontology-projection-implementation-steps 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- ontology-projection-implementation-steps 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- ontology-projection-implementation-steps 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Tests and evidence
- ontology-projection-tests-and-evidence 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- ontology-projection-tests-and-evidence 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- ontology-projection-tests-and-evidence 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- ontology-projection-tests-and-evidence 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- ontology-projection-tests-and-evidence 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- ontology-projection-tests-and-evidence 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Rollback
- ontology-projection-rollback 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- ontology-projection-rollback 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- ontology-projection-rollback 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- ontology-projection-rollback 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- ontology-projection-rollback 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- ontology-projection-rollback 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Acceptance criteria
- ontology-projection-acceptance-criteria 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- ontology-projection-acceptance-criteria 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- ontology-projection-acceptance-criteria 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- ontology-projection-acceptance-criteria 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- ontology-projection-acceptance-criteria 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- ontology-projection-acceptance-criteria 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Batch B substance audit
- Substance status 001: the pre-pass ontology IP was a 55-line capability-name stamp and did not define node, edge, projection, or rollback behavior.
- Substance status 002: this packet defines ITSM ontology projection for incident, problem, change, service catalog, and CMDB evidence.
- Substance status 003: displaced benchmarks are ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, and Freshservice.
- Substance status 004: ServiceNow table names, Jira request types, BMC forms, Ivanti discovery classes, and Freshservice catalog categories remain source metadata, not ontology type names.
- Substance status 005: docs/standards/documentation-rigor.md section 1.1 requires this IP to let an intern implement ontology projection without private architecture context.
- Substance status 006: ADR-0131, ADR-0246, ADR-0263, ADR-0316, and ADR-0321 jointly require canonical projection, library-first dispatch, audit evidence, capability-tier boundaries, and B2B depth.
- Substance status 007: projection must produce tenant-scoped, pack-aware, versioned ontology objects that survive vendor migration and replay.
- Substance status 008: projection failure must not mutate workflow state or mark policy decisions as accepted.

## Canonical ontology objects
- Object 001: `ItsmIncidentTicket` represents a tenant-scoped service interruption, request failure, or supportable incident record.
- Object 002: `ItsmIncidentTicket` carries source_system_kind, source_system_ref, severity, impact, urgency, assignment_group_ref, and requester_visibility_class.
- Object 003: `ItsmProblemRecord` represents root-cause analysis and known-error evidence linked to one or more incident tickets.
- Object 004: `ItsmProblemRecord` carries cause_hypothesis, workaround_state, known_error_flag, related_incident_refs, and remediation_owner_ref.
- Object 005: `ItsmChangeRequest` represents planned operational change with approval, risk, freeze, implementation, and rollback evidence.
- Object 006: `ItsmChangeRequest` carries risk_tier, change_window, approver_refs, separation_of_duty_status, and rollback_plan_ref.
- Object 007: `ItsmServiceCatalogItem` represents a requestable service offering with entitlement, form schema, approval, fulfillment, and data-class metadata.
- Object 008: `ItsmServiceCatalogItem` carries requester_fields, fulfiller_fields, entitlement_policy_ref, approval_template_ref, and deal_set_id when marketplace-backed.
- Object 009: `ItsmConfigurationItem` represents a tenant-scoped asset, service component, or dependency endpoint under CMDB governance.
- Object 010: `ItsmConfigurationItem` carries ci_type, lifecycle_state, source_confidence, discovery_source, owner_ref, and residency_zone.
- Object 011: `ItsmCmdbRelation` represents a typed relation between two configuration items with provenance and confidence.
- Object 012: `ItsmCmdbRelation` carries relation_type, source_ci_ref, target_ci_ref, confidence_score, discovered_at, verified_by_principal_id, and tenant_id.
- Object 013: `ItsmMajorIncidentBridge` represents the handoff from ITSM incident evidence into incident-management responder orchestration.
- Object 014: `ItsmMajorIncidentBridge` carries ITSM incident ref, incident-management incident ref, bridge_reason, stakeholder_scope, and audit event ids.
- Object 015: `ItsmSlaEvidence` represents SLA target, breach, recompute, pause, resume, and evidence-seal state.
- Object 016: `ItsmSlaEvidence` carries policy_ref, objective_ref, clock_state, breach_state, recompute_reason, and sealed_event_ref.

## Canonical ontology edges
- Edge 001: `incident_has_problem` links `ItsmIncidentTicket` to `ItsmProblemRecord` after tenant and home-cell equivalence are verified.
- Edge 002: `incident_affected_ci` links `ItsmIncidentTicket` to `ItsmConfigurationItem` with source confidence and impact role.
- Edge 003: `problem_caused_by_ci` links `ItsmProblemRecord` to `ItsmConfigurationItem` and records confidence, evidence source, and reviewer.
- Edge 004: `change_modifies_ci` links `ItsmChangeRequest` to `ItsmConfigurationItem` and requires rollback plan evidence.
- Edge 005: `change_resolves_problem` links `ItsmChangeRequest` to `ItsmProblemRecord` and records known-error closure evidence.
- Edge 006: `catalog_item_fulfills_request` links `ItsmServiceCatalogItem` to request workflow instances without exposing requester private data.
- Edge 007: `cmdb_relation_connects_ci` links `ItsmCmdbRelation` to both endpoint CIs and rejects cross-tenant endpoints.
- Edge 008: `incident_bridged_to_major_incident` links ITSM incident evidence to incident-management orchestration only after bidirectional audit preimages exist.
- Edge 009: `sla_evidence_measures_ticket` links `ItsmSlaEvidence` to incidents and service requests with immutable clock state.
- Edge 010: `catalog_item_requires_change` links catalog fulfillment to change approval where fulfillment mutates operational infrastructure.
- Edge 011: `problem_has_workaround` links problem evidence to knowledge records when knowledge publish policy has passed.
- Edge 012: `change_blocked_by_freeze` links change request to freeze-window evidence without rewriting original approval evidence.

## Projection inputs
- Projection input 001: REST action response includes ontology_object_ref only after projection succeeds.
- Projection input 002: Async command accepted event includes tenant_id, data_class, source_system_kind, and projection intent.
- Projection input 003: gRPC internal calls pass the same tenant context fields as REST to avoid projection drift.
- Projection input 004: ServiceNow import supplies table, sys_id, number, assignment group, state, priority, and update timestamp.
- Projection input 005: Jira Service Management import supplies project key, request type, issue key, queue, participant set, and SLA clock.
- Projection input 006: BMC Helix import supplies form name, request id, support group, problem investigation, change id, and status reason.
- Projection input 007: Ivanti Neurons import supplies device id, relation discovery source, confidence, owner, lifecycle state, and endpoint identifiers.
- Projection input 008: Freshservice import supplies ticket id, asset id, requester, service item, approval workflow, and fulfillment state.
- Projection input 009: Marketplace-backed catalog items supply deal_set_id, license terms, entitlement tier, and revocation behavior.
- Projection input 010: Replay jobs supply original audit preimage so projection can remain deterministic across retries.

## Projection algorithm
- Projection algorithm 001: Validate tenant context before reading vendor payload fields.
- Projection algorithm 002: Normalize source_system_kind through the manifest-approved benchmark enum.
- Projection algorithm 003: Map source vendor object kind to canonical ITSM bounded context.
- Projection algorithm 004: Reject any vendor object kind that lacks a canonical object mapping and explicit data_class.
- Projection algorithm 005: Compute canonical object ref from tenant_id, bounded_context, normalized local id, and version.
- Projection algorithm 006: Preserve source_system_ref as provenance metadata, not primary key.
- Projection algorithm 007: Build canonical object payload with tenant_id, home_cell_id, data_class, compliance packs, source provenance, and audit preimage id.
- Projection algorithm 008: Validate relation endpoints before creating any edge.
- Projection algorithm 009: Refuse CMDB relation projection when either endpoint is absent, wrong tenant, wrong home cell, or below confidence threshold.
- Projection algorithm 010: Refuse incident-problem links that create graph loops or cross-tenant correlation leakage.
- Projection algorithm 011: Emit projection-started audit evidence before write and projection-completed evidence after write.
- Projection algorithm 012: Emit projection-denied evidence when validation or policy rejects the input.
- Projection algorithm 013: Store projection version so later schema changes can replay deterministically.
- Projection algorithm 014: Store source payload digest so tampering or replay drift is visible.
- Projection algorithm 015: Store rollback reference for newly created nodes and edges.
- Projection algorithm 016: Return ontology object refs only after audit evidence and storage write both succeed.

## Implementation sequence
- Implementation 001: Add canonical object structs in the ITSM domain or kernel layer selected by ADR-0105 boundaries.
- Implementation 002: Add source-system mapping functions for ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, and Freshservice.
- Implementation 003: Add object-ref constructor that rejects missing tenant_id, bounded_context, local_id, and version.
- Implementation 004: Add edge constructor that requires both endpoint refs plus tenant equality validation.
- Implementation 005: Add projection command type with tenant context, source payload digest, projection version, and audit preimage id.
- Implementation 006: Add projection result type with object refs, edge refs, audit event ids, denied reason, and rollback ref.
- Implementation 007: Add repository interface for idempotent upsert of ontology nodes and edges.
- Implementation 008: Add replay mode that compares payload digest and projection version before writing.
- Implementation 009: Add rollback mode that deletes or tombstones projected nodes while preserving sealed audit events.
- Implementation 010: Add metrics for projection_count, projection_denied_count, projection_latency, relation_confidence_failures, and replay_drift.
- Implementation 011: Add OpenAPI examples that show ontology_object_ref for incident, change, catalog, and CMDB actions.
- Implementation 012: Add AsyncAPI examples for projection-started, projection-completed, projection-denied, and projection-rollback events.
- Implementation 013: Add proto messages for projection command and result with explicit validation tests.
- Implementation 014: Add SDK affordance to fetch projection result by audit_event_id and object_ref.
- Implementation 015: Add CLI fixture command that loads benchmark payload examples and prints canonical object refs.

## Test matrix
- Test matrix 001: Unit test maps ServiceNow incident table payload to `ItsmIncidentTicket`.
- Test matrix 002: Unit test maps Jira Service Management request payload to `ItsmIncidentTicket` or `ItsmServiceCatalogItem` based on request type.
- Test matrix 003: Unit test maps BMC Helix problem investigation payload to `ItsmProblemRecord`.
- Test matrix 004: Unit test maps BMC Helix change payload to `ItsmChangeRequest`.
- Test matrix 005: Unit test maps Ivanti Neurons device payload to `ItsmConfigurationItem`.
- Test matrix 006: Unit test maps Ivanti relation payload to `ItsmCmdbRelation` when endpoints share tenant.
- Test matrix 007: Unit test maps Freshservice service item payload to `ItsmServiceCatalogItem`.
- Test matrix 008: Negative test rejects ServiceNow table payload with missing tenant context.
- Test matrix 009: Negative test rejects Jira request payload that attempts requester visibility escalation.
- Test matrix 010: Negative test rejects BMC support group as authorization source.
- Test matrix 011: Negative test rejects Ivanti relation with endpoint tenant mismatch.
- Test matrix 012: Negative test rejects Freshservice catalog publish lacking entitlement policy ref.
- Test matrix 013: Property test proves source ids cannot collide across tenants.
- Test matrix 014: Property test proves relation edges cannot cross tenant boundary.
- Test matrix 015: Replay test proves same payload digest and projection version produce same object refs.
- Test matrix 016: Replay test detects changed payload digest and emits drift evidence.
- Test matrix 017: Rollback test removes projected node write effect while preserving audit-chain evidence.
- Test matrix 018: Contract test verifies REST response returns ontology_object_ref only on successful projection.
- Test matrix 019: AsyncAPI test verifies projection-denied event includes refusal reason and policy decision id.
- Test matrix 020: Metrics test verifies projection labels use tenant hash, source_system_kind, bounded_context, and outcome.

## Failure handling
- Failure handling 001: If tenant validation fails, projection does not inspect vendor payload fields.
- Failure handling 002: If source-system mapping fails, projection emits denied evidence with unmapped source kind.
- Failure handling 003: If data_class mapping fails, projection emits denied evidence and prevents ontology write.
- Failure handling 004: If relation endpoint lookup fails, projection creates no partial relation edge.
- Failure handling 005: If confidence score is below threshold, projection records rejected relation evidence without creating an edge.
- Failure handling 006: If audit-start emission fails, projection halts before storage write.
- Failure handling 007: If storage write fails, projection emits failed evidence and leaves workflow result pending remediation.
- Failure handling 008: If audit-completed emission fails after storage write, the remediation path reconciles object ref and missing event.
- Failure handling 009: If replay drift is detected, replay halts and emits drift evidence without rewriting object refs.
- Failure handling 010: If rollback fails, rollback runbook receives object ref, edge ref, projection version, and audit event ids.

## Acceptance expansion
- Acceptance expansion 001: An intern can identify every canonical ITSM ontology object and edge required by this packet.
- Acceptance expansion 002: An intern can implement source-vendor mapping without copying ServiceNow, Jira, BMC, Ivanti, or Freshservice object names into canonical type names.
- Acceptance expansion 003: An intern can explain why source ids are provenance metadata rather than primary keys.
- Acceptance expansion 004: An intern can explain why CMDB relation endpoints require tenant and home-cell equality.
- Acceptance expansion 005: An intern can explain how major incident bridge projection differs from incident-management paging orchestration.
- Acceptance expansion 006: An intern can implement projection-denied evidence without guessing audit event fields.
- Acceptance expansion 007: An intern can implement deterministic replay and drift detection from payload digest and projection version.
- Acceptance expansion 008: An intern can implement rollback without deleting sealed audit evidence.
- Acceptance expansion 009: An intern can write benchmark fixtures for all five displaced vendors.
- Acceptance expansion 010: An intern can produce a PR with domain structs, mapping functions, repository interface, contract examples, tests, metrics, and rollback fixtures.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 supplies the intern-buildability quality gate for this rewrite.
- Citation 002: microservices/itsm/manifest.json supplies the ITSM bounded contexts, benchmark roster, layer conformance, and compliance packs.
- Citation 003: microservices/itsm/PRD.md defines incident-ticket, problem, change, service-request, and configuration-item as canonical operational concerns.
- Citation 004: microservices/itsm/contracts/openapi-v1.yaml defines ontology_object_ref in accepted action responses.
- Citation 005: ADR-0105 defines which layers may own projection domain types, usecases, REST examples, workers, and adapters.
- Citation 006: ADR-0131 defines ontology projection expectations inherited by ITSM object modeling.
- Citation 007: ADR-0246 defines library-first dispatch expectations so projection mapping is reusable and testable.
- Citation 008: ADR-0258 defines contract versioning expectations for OpenAPI, AsyncAPI, proto, SDK, and CLI projection surfaces.
- Citation 009: ADR-0263 defines audit-chain event discipline for projection-started, completed, denied, replayed, and rolled back states.
- Citation 010: ADR-0314 defines DealSet evidence when service catalog projection involves marketplace-backed content.
- Citation 011: ADR-0316 prevents vendor product labels from becoming ontology type boundaries.
- Citation 012: ADR-0321 defines B2B leader parity expectations and rejects shallow benchmark mapping.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-003-ontology-projection.md` matched [`emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-003-ontology-projection.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
