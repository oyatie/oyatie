# IP-028 ITSM service-catalog-entitlement-orchestrator

Service: itsm
ChangeSet scope: microservices/itsm/IP-028-service-catalog-entitlement-orchestrator.md
Benchmarks displaced: ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, Freshservice
Binding ADRs: ADR-0105, ADR-0244, ADR-0246, ADR-0258, ADR-0263, ADR-0314, ADR-0316, ADR-0321

## Objective
- Objective 001: Build the ITSM service catalog entitlement orchestrator for catalog item visibility, request eligibility, approval routing, fulfillment authorization, and revocation.
- Objective 002: Displace ServiceNow Service Catalog, Jira Service Management portal request types, BMC Digital Workplace, Ivanti service catalog, and Freshservice service catalog.
- Objective 003: Keep requester data, fulfiller data, entitlement data, approval evidence, DealSet evidence, and audit events separated.
- Objective 004: Make marketplace-backed catalog content enforce ADR-0314 DealSet settlement before publication and fulfillment.
- Objective 005: Make the implementation buildable from this IP under docs/standards/documentation-rigor.md section 1.1.
- Objective 006: Avoid touching manifests, journeys, ERP, ADR-0321, or other B2B-leader batches.

## Domain entities
- Entity 001: `CatalogItemDraft` is a tenant-scoped draft containing item schema, requester fields, fulfiller fields, data_class roster, owner, and approval template.
- Entity 002: `CatalogEntitlementPolicy` defines who can view, request, approve, fulfill, revoke, and audit a catalog item.
- Entity 003: `CatalogRequest` records a requester-visible request instance with tenant, principal, item, form answers, SLA target, and status.
- Entity 004: `CatalogFulfillmentPlan` records operator-only fulfillment steps, dependencies, workflow template, and rollback plan.
- Entity 005: `CatalogApprovalEvidence` records approver, decision, separation-of-duty result, policy decision id, and audit event id.
- Entity 006: `CatalogDealSetBinding` records marketplace provider, deal_set_id, license terms, usage limits, revocation behavior, and settlement status.
- Entity 007: `CatalogPublication` records publication version, channel, target tenant cohort, release window, and revocation state.
- Entity 008: `CatalogRevocation` records reason, affected request count, consumer notice workflow, rollback run, and evidence refs.
- Entity 009: `CatalogRequesterView` exposes only fields permitted for requesters and hides fulfiller-only and audit-only data.
- Entity 010: `CatalogOperatorView` exposes fulfillment details only after Cedar permits the operator action.

## Entitlement semantics
- Entitlement 001: View entitlement answers whether a principal can see an item in the portal.
- Entitlement 002: Request entitlement answers whether a principal can submit an item request.
- Entitlement 003: Approve entitlement answers whether a principal can approve a submitted request.
- Entitlement 004: Fulfill entitlement answers whether a principal can execute fulfillment steps.
- Entitlement 005: Publish entitlement answers whether a principal can publish or update a catalog item.
- Entitlement 006: Revoke entitlement answers whether a principal can revoke an item or published version.
- Entitlement 007: Audit entitlement answers whether a principal can inspect sealed request, approval, and fulfillment evidence.
- Entitlement 008: Marketplace entitlement answers whether a tenant can consume the provider-backed item under DealSet terms.
- Entitlement 009: Pack entitlement answers whether active compliance packs narrow fields, approvals, retention, egress, or availability.
- Entitlement 010: Delegated-admin entitlement answers whether a managed-service provider can request or fulfill on behalf of a client tenant.

## Vendor displacement behavior
- Vendor behavior 001: ServiceNow catalog categories map to display taxonomy and cannot grant view entitlement by themselves.
- Vendor behavior 002: ServiceNow catalog item variables map to requester or fulfiller fields after data-class classification.
- Vendor behavior 003: ServiceNow flow approvals map to canonical approval evidence and separation-of-duty checks.
- Vendor behavior 004: Jira Service Management request types map to catalog item schemas, not canonical endpoint names.
- Vendor behavior 005: Jira portal customers map to requester visibility and cannot become fulfillers automatically.
- Vendor behavior 006: BMC Digital Workplace service bundles map to catalog item groups with explicit entitlement policies.
- Vendor behavior 007: BMC approval chains map to approval evidence and cannot bypass Cedar default-deny.
- Vendor behavior 008: Ivanti catalog automations map to fulfillment plans and cannot execute without approval and policy evidence.
- Vendor behavior 009: Ivanti device-targeted requests require CMDB relation validation before fulfillment.
- Vendor behavior 010: Freshservice service items map to catalog items with requester and fulfiller field separation.
- Vendor behavior 011: Freshservice requester groups map to view or request entitlement only after tenant-scoped grant mapping.
- Vendor behavior 012: Vendor catalog item ids remain source refs and never become canonical catalog ids.

## Orchestration sequence
- Sequence 001: Validate tenant context before catalog item, request, or publication parsing.
- Sequence 002: Normalize source vendor object through IP-026 process normalizer when imported.
- Sequence 003: Classify requester fields and fulfiller fields by data_class.
- Sequence 004: Validate item schema against allowed data classes and active pack overlays.
- Sequence 005: Evaluate publish entitlement before creating or updating a catalog item draft.
- Sequence 006: Evaluate DealSet binding when a marketplace provider backs the item.
- Sequence 007: Publish item version only after policy, approval, DealSet, and audit-start evidence exist.
- Sequence 008: Evaluate view entitlement before displaying the item.
- Sequence 009: Evaluate request entitlement before accepting a request.
- Sequence 010: Evaluate approval entitlement before recording approval decision.
- Sequence 011: Evaluate fulfill entitlement before dispatching workflow template.
- Sequence 012: Project request and fulfillment objects into ontology after accepted state changes.
- Sequence 013: Emit audit events for draft, published, requested, approved, fulfilled, revoked, denied, and rolled_back.
- Sequence 014: Revoke item by version and cohort, not by deleting historical request evidence.
- Sequence 015: Roll back fulfillment effects through the stored fulfillment rollback plan.

## Data model
- Data model 001: `catalog_item_id` is deterministic from tenant_id, item_slug, publication_version, and source_system_kind.
- Data model 002: `catalog_item_version` increments on schema, entitlement, workflow, DealSet, or pack-overlay changes.
- Data model 003: `requester_field_schema` stores fields requesters can see and submit.
- Data model 004: `fulfiller_field_schema` stores operator-only fields and secrets references.
- Data model 005: `data_class_roster` records all data classes the request can collect or mutate.
- Data model 006: `entitlement_policy_ref` points to Cedar policy and local entitlement config.
- Data model 007: `approval_template_ref` points to ITSM workflow template library.
- Data model 008: `deal_set_id` is mandatory when provider content carries marketplace terms.
- Data model 009: `publication_channel` records tenant cohort, portal, SDK, CLI, or managed-service surface.
- Data model 010: `revocation_plan_id` records notification and rollback behavior.
- Data model 011: `request_id` is tenant-scoped and idempotency-key protected.
- Data model 012: `fulfillment_run_id` ties catalog request to workflow execution.
- Data model 013: `audit_event_id` ties every state transition to ADR-0263 evidence.
- Data model 014: `policy_decision_id` ties every entitlement decision to Cedar evidence.
- Data model 015: `rollback_plan_id` ties every fulfillment mutation to compensation.

## Implementation sequence
- Implementation 001: Add domain structs for catalog item draft, entitlement policy, request, approval evidence, fulfillment plan, DealSet binding, publication, and revocation.
- Implementation 002: Add schema classifier that marks requester, fulfiller, and audit-only fields.
- Implementation 003: Add entitlement evaluator that calls Cedar default-deny for view, request, approve, fulfill, publish, revoke, audit, marketplace, pack, and delegated-admin actions.
- Implementation 004: Add DealSet validator for marketplace-backed catalog items.
- Implementation 005: Add publication service with idempotent item versioning.
- Implementation 006: Add request service with view and request entitlement checks.
- Implementation 007: Add approval service with separation-of-duty evidence.
- Implementation 008: Add fulfillment dispatcher that uses ITSM workflow templates.
- Implementation 009: Add revocation service with cohort targeting and notification workflow.
- Implementation 010: Add ontology projection for catalog item, request, fulfillment, and revocation refs.
- Implementation 011: Add audit events for every accepted, denied, and rolled-back state.
- Implementation 012: Add metrics for view_denied, request_denied, publish_denied, fulfill_denied, DealSet_missing, revocation_count, and fulfillment_latency.
- Implementation 013: Add REST examples for catalog item create, publish, request, approve, fulfill, revoke, and audit lookup.
- Implementation 014: Add AsyncAPI events for catalog published, request accepted, approval denied, fulfillment completed, and item revoked.
- Implementation 015: Add SDK builder that requires tenant context and item id before request submission.

## Test matrix
- Test 001: Unit test separates requester fields from fulfiller fields.
- Test 002: Unit test rejects catalog item schema with undeclared data class.
- Test 003: Unit test rejects publish without publish entitlement.
- Test 004: Unit test rejects marketplace-backed publish without deal_set_id.
- Test 005: Unit test rejects requester view when view entitlement fails.
- Test 006: Unit test rejects request submission when request entitlement fails.
- Test 007: Unit test rejects approval when approver is requester under separation-of-duty pack.
- Test 008: Unit test rejects fulfillment when operator lacks fulfill entitlement.
- Test 009: Unit test rejects revocation when actor lacks revoke entitlement.
- Test 010: Unit test permits audit read without mutation entitlement.
- Test 011: ServiceNow fixture maps catalog item variables into requester and fulfiller schemas.
- Test 012: Jira fixture maps request type fields without creating Jira endpoint names.
- Test 013: BMC fixture maps service bundle to item group and entitlement policy.
- Test 014: Ivanti fixture maps device-targeted request to CMDB relation pre-check.
- Test 015: Freshservice fixture maps service item to catalog item and requester group to entitlement evidence.
- Test 016: Property test proves item ids differ across tenants for the same source vendor item.
- Test 017: Replay test proves publication version is deterministic under the same mapping version.
- Test 018: Rollback test revokes item version without deleting historical request evidence.
- Test 019: Contract test validates OpenAPI request and response examples.
- Test 020: Audit test validates draft, published, requested, approved, fulfilled, revoked, denied, and rolled_back events.

## Failure handling
- Failure 001: Missing tenant context returns validation error before schema classification.
- Failure 002: Missing entitlement policy returns denied evidence and remediation hint.
- Failure 003: Missing DealSet for marketplace item returns publish denial.
- Failure 004: Unknown requester field data class returns schema denial.
- Failure 005: Approval conflict returns separation-of-duty denial.
- Failure 006: Fulfillment dispatch failure leaves request in approved pending-remediation state.
- Failure 007: Revocation notification failure records partial revocation evidence and runbook pointer.
- Failure 008: Ontology projection failure blocks accepted response until remediation or compensation path is recorded.
- Failure 009: Replay drift blocks publication update and emits drift evidence.
- Failure 010: Rollback failure preserves original publication and fulfillment audit events.

## Acceptance criteria
- Acceptance 001: An intern can implement catalog item, entitlement, request, approval, fulfillment, DealSet, publication, and revocation structures.
- Acceptance 002: An intern can implement field classification and explain requester versus fulfiller separation.
- Acceptance 003: An intern can implement entitlement evaluation for view, request, approve, fulfill, publish, revoke, audit, marketplace, pack, and delegated-admin actions.
- Acceptance 004: An intern can explain how ServiceNow, Jira Service Management, BMC Digital Workplace, Ivanti, and Freshservice catalogs are displaced.
- Acceptance 005: An intern can implement DealSet-gated marketplace publication under ADR-0314.
- Acceptance 006: An intern can implement publication versioning, request idempotency, and revocation by version.
- Acceptance 007: An intern can implement benchmark fixtures and negative entitlement tests.
- Acceptance 008: An intern can implement REST, AsyncAPI, SDK, metrics, audit, replay, and rollback evidence.
- Acceptance 009: An intern can avoid changing manifests, journeys, ERP, ADR-0321, or other B2B leader batches.
- Acceptance 010: An intern can produce a single-PR deliverable with tests and runbook pointers.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 defines the intern-buildability bar.
- Citation 002: microservices/itsm/manifest.json defines ITSM audience, benchmark roster, compliance packs, and layer conformance.
- Citation 003: microservices/itsm/PRD.md defines service-request and service catalog expectations.
- Citation 004: microservices/itsm/contracts/openapi-v1.yaml defines action invocation and accepted response evidence fields.
- Citation 005: microservices/itsm/runbooks/service-catalog-publish-failure.md anchors publication remediation.
- Citation 006: ADR-0105 defines layer boundaries for catalog domain, usecase, REST, worker, adapter, and governance logic.
- Citation 007: ADR-0244 defines default-deny entitlement evaluation.
- Citation 008: ADR-0246 defines reusable entitlement library expectations.
- Citation 009: ADR-0258 defines catalog contract versioning and deprecation requirements.
- Citation 010: ADR-0263 defines audit-chain event discipline for catalog state transitions.
- Citation 011: ADR-0314 defines DealSet settlement evidence for marketplace-backed catalog content.
- Citation 012: ADR-0316 prevents vendor catalog product labels from becoming Oyatie service boundaries.
- Citation 013: ADR-0321 defines B2B leader parity expectations for ITSM catalog depth.

## Detailed build checklist
- Build checklist 001: Add fixture `servicenow_catalog_laptop_request_valid.json`.
- Build checklist 002: Add fixture `servicenow_catalog_admin_role_not_publish.json`.
- Build checklist 003: Add fixture `jira_request_type_hardware_valid.json`.
- Build checklist 004: Add fixture `jira_customer_not_fulfiller_denied.json`.
- Build checklist 005: Add fixture `bmc_digital_workplace_bundle_valid.json`.
- Build checklist 006: Add fixture `bmc_approval_chain_separation_denied.json`.
- Build checklist 007: Add fixture `ivanti_device_request_cmdb_precheck_valid.json`.
- Build checklist 008: Add fixture `ivanti_automation_without_approval_denied.json`.
- Build checklist 009: Add fixture `freshservice_service_item_valid.json`.
- Build checklist 010: Add fixture `freshservice_requester_group_not_publish_denied.json`.
- Build checklist 011: Add canonicalen request body for catalog item publish.
- Build checklist 012: Add canonicalen request body for catalog item request.
- Build checklist 013: Add canonicalen request body for catalog approval.
- Build checklist 014: Add canonicalen request body for fulfillment dispatch.
- Build checklist 015: Add canonicalen request body for catalog revocation.
- Build checklist 016: Add canonicalen response body for view entitlement denial.
- Build checklist 017: Add canonicalen response body for request entitlement denial.
- Build checklist 018: Add canonicalen response body for marketplace DealSet denial.
- Build checklist 019: Add canonicalen response body for accepted publication.
- Build checklist 020: Add canonicalen response body for accepted fulfillment.
- Build checklist 021: Add AsyncAPI message fixture for `itsm.catalog.published.v1`.
- Build checklist 022: Add AsyncAPI message fixture for `itsm.catalog.requested.v1`.
- Build checklist 023: Add AsyncAPI message fixture for `itsm.catalog.approved.v1`.
- Build checklist 024: Add AsyncAPI message fixture for `itsm.catalog.fulfilled.v1`.
- Build checklist 025: Add AsyncAPI message fixture for `itsm.catalog.revoked.v1`.
- Build checklist 026: Add AsyncAPI message fixture for `itsm.catalog.denied.v1`.
- Build checklist 027: Add Grafana panel query for catalog view denial rate.
- Build checklist 028: Add Grafana panel query for catalog request denial rate.
- Build checklist 029: Add Grafana panel query for DealSet settlement missing rate.
- Build checklist 030: Add Grafana panel query for fulfillment latency by item version.
- Build checklist 031: Add runbook link for service catalog publish failure.
- Build checklist 032: Add runbook link for dealset support entitlement hold.
- Build checklist 033: Add runbook link for delegated admin lockout.
- Build checklist 034: Add rollback test for revocation without evidence deletion.
- Build checklist 035: Add final verification command for line count and citation density.
- Build checklist 036: Add PR summary line naming this as net-new IP-028.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
