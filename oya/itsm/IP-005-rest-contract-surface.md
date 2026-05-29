# IP-005 ITSM rest-contract-surface

Service: itsm
ChangeSet scope: microservices/itsm/IP-005-rest-contract-surface.md
Benchmarks: ServiceNow ITSM, Jira Service Management, BMC Remedy, Zendesk Support, Freshdesk
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- rest-contract-surface-objective 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- rest-contract-surface-objective 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- rest-contract-surface-objective 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- rest-contract-surface-objective 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- rest-contract-surface-objective 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- rest-contract-surface-objective 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Prerequisites
- rest-contract-surface-prerequisites 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- rest-contract-surface-prerequisites 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- rest-contract-surface-prerequisites 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- rest-contract-surface-prerequisites 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- rest-contract-surface-prerequisites 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- rest-contract-surface-prerequisites 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Implementation steps
- rest-contract-surface-implementation-steps 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- rest-contract-surface-implementation-steps 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- rest-contract-surface-implementation-steps 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- rest-contract-surface-implementation-steps 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- rest-contract-surface-implementation-steps 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- rest-contract-surface-implementation-steps 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Tests and evidence
- rest-contract-surface-tests-and-evidence 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- rest-contract-surface-tests-and-evidence 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- rest-contract-surface-tests-and-evidence 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- rest-contract-surface-tests-and-evidence 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- rest-contract-surface-tests-and-evidence 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- rest-contract-surface-tests-and-evidence 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Rollback
- rest-contract-surface-rollback 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- rest-contract-surface-rollback 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- rest-contract-surface-rollback 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- rest-contract-surface-rollback 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- rest-contract-surface-rollback 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- rest-contract-surface-rollback 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Acceptance criteria
- rest-contract-surface-acceptance-criteria 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- rest-contract-surface-acceptance-criteria 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- rest-contract-surface-acceptance-criteria 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- rest-contract-surface-acceptance-criteria 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- rest-contract-surface-acceptance-criteria 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- rest-contract-surface-acceptance-criteria 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Batch B substance audit
- Substance status 001: the seed REST IP was 55 lines and did not specify endpoints, schemas, validation order, examples, or error semantics.
- Substance status 002: this packet deepens the ITSM REST surface for incident, problem, change, service catalog, CMDB, and bridge operations.
- Substance status 003: displaced benchmarks are ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, and Freshservice.
- Substance status 004: REST must expose canonical Oyatie contracts rather than product-specific vendor URLs, table names, or screen actions.
- Substance status 005: docs/standards/documentation-rigor.md section 1.1 requires an intern to build OpenAPI paths, schemas, tests, and examples from this packet.
- Substance status 006: ADR-0258 requires versioned contract evolution; this IP therefore uses `/itsm/v1` and forbids unversioned endpoints.
- Substance status 007: ADR-0253-amendment requires HTTP/3, TLS 1.3, ECH, and PQC posture to be visible in server and deployment assumptions.
- Substance status 008: ADR-0314 requires DealSet settlement fields for marketplace-backed catalog actions.

## Endpoint inventory
- Endpoint 001: `GET /itsm/v1/capabilities` lists enabled ITSM capabilities for the authenticated tenant and pack set.
- Endpoint 002: `POST /itsm/v1/incidents` opens an incident ticket with tenant context, requester visibility, impact, urgency, and source provenance.
- Endpoint 003: `PATCH /itsm/v1/incidents/{incident_id}` amends incident details without changing sealed audit evidence.
- Endpoint 004: `POST /itsm/v1/incidents/{incident_id}/major-bridge` requests handoff to incident-management and returns bridge evidence.
- Endpoint 005: `POST /itsm/v1/problems` opens a problem record with related incident refs and root-cause hypothesis.
- Endpoint 006: `POST /itsm/v1/problems/{problem_id}/links` links incidents, problems, workarounds, and known errors after graph checks pass.
- Endpoint 007: `POST /itsm/v1/changes` opens a standard, normal, or emergency change request.
- Endpoint 008: `POST /itsm/v1/changes/{change_id}/approve` records an approval decision with separation-of-duty evidence.
- Endpoint 009: `POST /itsm/v1/changes/{change_id}/implement` records implementation start, verification, and rollback plan refs.
- Endpoint 010: `POST /itsm/v1/catalog/items` creates or drafts a service catalog item.
- Endpoint 011: `POST /itsm/v1/catalog/items/{item_id}/publish` publishes a catalog item after entitlement, approval, and DealSet checks.
- Endpoint 012: `POST /itsm/v1/catalog/items/{item_id}/revoke` revokes a catalog item and starts consumer notification workflow.
- Endpoint 013: `POST /itsm/v1/cmdb/items` creates or imports a configuration item with discovery provenance.
- Endpoint 014: `POST /itsm/v1/cmdb/relations` writes a CMDB relation after endpoint tenant and confidence checks.
- Endpoint 015: `POST /itsm/v1/replay/vendor-events` replays imported vendor events through canonical guards.
- Endpoint 016: `GET /itsm/v1/audit/{audit_event_id}` returns tenant-scoped audit evidence refs and never raw secret payloads.

## Shared request envelope
- Request envelope 001: Every mutating request requires tenant_id and principal_id even when bearer token already contains claims.
- Request envelope 002: Every mutating request requires purpose and data_class.
- Request envelope 003: Every mutating request requires idempotency_key and traceparent.
- Request envelope 004: Every request touching marketplace-backed catalog content requires deal_set_id.
- Request envelope 005: Every request sourced from ServiceNow, Jira, BMC, Ivanti, or Freshservice requires source_system_kind and source_system_ref.
- Request envelope 006: Every request touching delegated administration requires delegated_admin_grant_id.
- Request envelope 007: Every request touching compliance packs requires compliance_pack_set as an explicit array.
- Request envelope 008: Every request touching workflow execution requires workflow_template_id or a server-selected template result.
- Request envelope 009: Every request touching ontology projection requires prior object ref or projection intent.
- Request envelope 010: Every request touching emergency behavior requires emergency_bypass_reason and bypass_expiry.
- Request envelope 011: Every request body must reject unknown top-level fields unless endpoint-specific extension schema permits them.
- Request envelope 012: Every request must preserve tenant id through logs as hash only, not raw tenant display name.

## Shared response envelope
- Response envelope 001: Accepted mutation responses return status `202` when workflow execution continues asynchronously.
- Response envelope 002: Synchronous validation failures return `400` with validation_error_code and field path.
- Response envelope 003: Policy denials return `403` with policy_decision_id, refusal reason, and audit_event_id when emitted.
- Response envelope 004: Missing tenant or wrong home cell returns `409` when authenticated identity is valid but request scope conflicts.
- Response envelope 005: Duplicate idempotency key returns `200` or `202` with the original command result and idempotency replay marker.
- Response envelope 006: Unsupported source vendor mapping returns `422` with source_system_kind and mapping gap code.
- Response envelope 007: Pack overlay conflicts return `422` with pack id, narrowed behavior, and required remediation.
- Response envelope 008: Accepted responses include audit_event_class, audit_event_id, workflow_run_id when created, policy_decision_id, and ontology_object_ref when projected.
- Response envelope 009: Rollback responses include rollback_plan_id, rollback_run_id, original_audit_event_id, and preserved_evidence_refs.
- Response envelope 010: Error responses include remediation_hint_slug that maps to a runbook or implementation test.

## Schema details
- Schema 001: `TenantContext` includes tenant_id, principal_id, audience_type, home_cell_id, delegated_admin_grant_id, compliance_pack_set, and purpose.
- Schema 002: `SourceSystemRef` includes source_system_kind, source_system_ref, source_payload_digest, imported_at, and source_confidence.
- Schema 003: `IncidentOpenRequest` includes summary, description, impact, urgency, requester_ref, affected_ci_refs, and source provenance.
- Schema 004: `IncidentAmendRequest` includes patch operations and reason but excludes fields that would rewrite sealed breach evidence.
- Schema 005: `MajorBridgeRequest` includes bridge_reason, stakeholder_scope, incident_management_policy_ref, and status update intent.
- Schema 006: `ProblemOpenRequest` includes related_incident_refs, cause_hypothesis, workaround_text, and evidence refs.
- Schema 007: `ProblemLinkRequest` includes link_type, source_ref, target_ref, reviewer, and cycle_check_result.
- Schema 008: `ChangeOpenRequest` includes change_type, risk_tier, affected_ci_refs, schedule_window, approver_refs, and rollback_plan.
- Schema 009: `ChangeApprovalRequest` includes approval_decision, approver_ref, separation_of_duty_evidence, and freeze_window_result.
- Schema 010: `CatalogItemRequest` includes item schema, requester_fields, fulfiller_fields, entitlement_policy_ref, approval_template_ref, and deal_set_id.
- Schema 011: `CatalogPublishRequest` includes release channel, publish window, entitlement evidence, and revocation plan.
- Schema 012: `CmdbItemRequest` includes ci_type, lifecycle_state, owner_ref, discovery_source, source_confidence, and residency zone.
- Schema 013: `CmdbRelationRequest` includes relation_type, source_ci_ref, target_ci_ref, confidence_score, and verification principal.
- Schema 014: `VendorReplayRequest` includes source_system_kind, batch_id, cursor, projection_version, and dry_run flag.
- Schema 015: `ItsmActionAccepted` includes audit_event_class, audit_event_id, workflow_run_id, ontology_object_ref, policy_decision_id, and rollback_plan_id.

## Vendor displacement examples
- Vendor example 001: ServiceNow incident import uses `source_system_kind=servicenow_itsm` but creates canonical `/incidents` payload, not `/table/incident`.
- Vendor example 002: ServiceNow change import maps `change_request` fields into canonical `ChangeOpenRequest`.
- Vendor example 003: Jira Service Management issue import uses project key and request type only as provenance fields.
- Vendor example 004: Jira queue actions map to incident triage fields and do not create queue-specific endpoints.
- Vendor example 005: BMC Helix problem investigation import maps support group and problem id into `ProblemOpenRequest`.
- Vendor example 006: BMC Helix change approval import maps status reason into canonical approval decision.
- Vendor example 007: Ivanti Neurons device import maps discovered asset into `CmdbItemRequest` with confidence and owner provenance.
- Vendor example 008: Ivanti Neurons relationship import maps endpoint ids into `CmdbRelationRequest`.
- Vendor example 009: Freshservice ticket import maps requester and service item references into incident or catalog request payloads.
- Vendor example 010: Freshservice catalog publication maps service item schema into `CatalogItemRequest` plus DealSet evidence when marketplace-backed.

## Implementation sequence
- Implementation 001: Update `contracts/openapi-v1.yaml` path inventory with explicit ITSM endpoints.
- Implementation 002: Add shared request envelope schemas before endpoint-specific schemas.
- Implementation 003: Add shared response envelope schemas before endpoint-specific accepted and error payloads.
- Implementation 004: Add vendor replay request and response schemas with dry-run behavior.
- Implementation 005: Add examples for all five displaced vendors and native Oyatie.
- Implementation 006: Add schema validation tests for required tenant context fields.
- Implementation 007: Add schema validation tests for data_class per endpoint.
- Implementation 008: Add status-code tests for validation, policy denial, wrong tenant, pack overlay conflict, unsupported mapping, duplicate idempotency, and accepted mutation.
- Implementation 009: Add OpenAPI lint expectations for versioned paths, operation ids, tags, security schemes, and examples.
- Implementation 010: Add REST handler stubs only after contract tests fail for missing routes.
- Implementation 011: Add generated SDK snapshot updates only after OpenAPI contract stabilizes.
- Implementation 012: Add docs cross-links to runbooks for each remediation_hint_slug.

## Test matrix
- Test matrix 001: Contract test validates every mutating endpoint requires tenant_id.
- Test matrix 002: Contract test validates every mutating endpoint requires principal_id.
- Test matrix 003: Contract test validates every mutating endpoint requires purpose.
- Test matrix 004: Contract test validates every mutating endpoint requires data_class.
- Test matrix 005: Contract test validates every mutating endpoint requires idempotency_key.
- Test matrix 006: Contract test validates every source-vendor request requires source_system_kind and source_system_ref.
- Test matrix 007: Contract test validates ServiceNow incident example resolves to canonical incident endpoint.
- Test matrix 008: Contract test validates Jira request example does not create Jira-specific endpoint.
- Test matrix 009: Contract test validates BMC problem example maps to canonical problem schema.
- Test matrix 010: Contract test validates Ivanti CMDB relation example requires endpoint equality fields.
- Test matrix 011: Contract test validates Freshservice catalog example requires entitlement and approval fields.
- Test matrix 012: Handler test rejects missing tenant context before usecase invocation.
- Test matrix 013: Handler test returns policy_decision_id on Cedar denial.
- Test matrix 014: Handler test returns ontology_object_ref only after successful projection.
- Test matrix 015: Handler test returns workflow_run_id for asynchronous templates.
- Test matrix 016: Handler test returns original result on idempotency replay.
- Test matrix 017: Handler test returns pack overlay conflict code with pack id.
- Test matrix 018: Handler test returns unsupported mapping code for unknown vendor object.
- Test matrix 019: Security test confirms bearer auth is required for every endpoint.
- Test matrix 020: Observability test confirms traceparent and audit ids propagate to response evidence.

## Acceptance expansion
- Acceptance expansion 001: An intern can add concrete OpenAPI paths rather than leaving a generic action endpoint as the only contract.
- Acceptance expansion 002: An intern can implement request envelope, response envelope, examples, and endpoint-specific schemas.
- Acceptance expansion 003: An intern can explain why vendor table URLs do not become Oyatie REST endpoints.
- Acceptance expansion 004: An intern can explain why source refs are provenance and not route identity.
- Acceptance expansion 005: An intern can implement required status codes and error payloads without guessing remediation shape.
- Acceptance expansion 006: An intern can add examples for ServiceNow, Jira Service Management, BMC Helix, Ivanti Neurons, and Freshservice.
- Acceptance expansion 007: An intern can tie OpenAPI responses to workflow, policy, ontology, and audit evidence.
- Acceptance expansion 008: An intern can test HTTP/3 and TLS posture through deployment metadata and contract server declarations.
- Acceptance expansion 009: An intern can generate SDK updates after contract tests pass.
- Acceptance expansion 010: An intern can produce a PR with OpenAPI changes, schema tests, handler validation tests, examples, SDK snapshots, and runbook links.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 sets the intern-buildability quality gate for this REST packet.
- Citation 002: microservices/itsm/manifest.json supplies benchmark roster, audience type, layer conformance, and pack roster.
- Citation 003: microservices/itsm/PRD.md supplies ITSM bounded contexts and acceptance expectations.
- Citation 004: microservices/itsm/contracts/openapi-v1.yaml is the contract file this IP directly deepens.
- Citation 005: ADR-0105 defines REST as one layer and prevents it from owning domain validation alone.
- Citation 006: ADR-0244 requires policy denial semantics before mutating usecase execution.
- Citation 007: ADR-0253-amendment defines HTTP/3, ECH, PQC, and strict TLS assumptions for exposed REST services.
- Citation 008: ADR-0258 defines OpenAPI versioning, deprecation, and compatibility expectations.
- Citation 009: ADR-0263 defines audit-chain evidence fields returned by accepted, denied, replayed, and rolled-back operations.
- Citation 010: ADR-0314 defines DealSet fields for marketplace-backed ITSM catalog flows.
- Citation 011: ADR-0316 prevents vendor URL or product names from becoming canonical REST boundaries.
- Citation 012: ADR-0321 defines B2B leader parity depth for the ITSM REST contract surface.
- Citation 013: docs/AGENTS.md supplies the Oya VCS claim, verification, completion, and promotion lifecycle that this contract packet must satisfy.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
