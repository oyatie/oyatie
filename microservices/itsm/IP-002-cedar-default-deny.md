# IP-002 ITSM cedar-default-deny

Service: itsm
ChangeSet scope: microservices/itsm/IP-002-cedar-default-deny.md
Benchmarks: ServiceNow ITSM, Jira Service Management, BMC Remedy, Zendesk Support, Freshdesk
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- cedar-default-deny-objective 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- cedar-default-deny-objective 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- cedar-default-deny-objective 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- cedar-default-deny-objective 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- cedar-default-deny-objective 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- cedar-default-deny-objective 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Prerequisites
- cedar-default-deny-prerequisites 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- cedar-default-deny-prerequisites 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- cedar-default-deny-prerequisites 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- cedar-default-deny-prerequisites 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- cedar-default-deny-prerequisites 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- cedar-default-deny-prerequisites 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Implementation steps
- cedar-default-deny-implementation-steps 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- cedar-default-deny-implementation-steps 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- cedar-default-deny-implementation-steps 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- cedar-default-deny-implementation-steps 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- cedar-default-deny-implementation-steps 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- cedar-default-deny-implementation-steps 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Tests and evidence
- cedar-default-deny-tests-and-evidence 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- cedar-default-deny-tests-and-evidence 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- cedar-default-deny-tests-and-evidence 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- cedar-default-deny-tests-and-evidence 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- cedar-default-deny-tests-and-evidence 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- cedar-default-deny-tests-and-evidence 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Rollback
- cedar-default-deny-rollback 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- cedar-default-deny-rollback 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- cedar-default-deny-rollback 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- cedar-default-deny-rollback 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- cedar-default-deny-rollback 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- cedar-default-deny-rollback 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Acceptance criteria
- cedar-default-deny-acceptance-criteria 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- cedar-default-deny-acceptance-criteria 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- cedar-default-deny-acceptance-criteria 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- cedar-default-deny-acceptance-criteria 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- cedar-default-deny-acceptance-criteria 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- cedar-default-deny-acceptance-criteria 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Batch B substance audit
- Substance status 001: the pre-pass file was 55 lines and repeated the same six capability sentences across every section.
- Substance status 002: this rewrite makes Cedar default-deny concrete for ITSM rather than treating authorization as a generic B2B concern.
- Substance status 003: displaced benchmarks are ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, and Freshservice.
- Substance status 004: the policy packet replaces product-suite permission assumptions with explicit tenant, principal, action, resource, context, pack, and audit inputs.
- Substance status 005: docs/standards/documentation-rigor.md section 1.1 is satisfied only if an intern can implement the deny matrix and tests from this file.
- Substance status 006: ADR-0244 and ADR-0321 make default-deny and B2B leader parity mandatory; optional policy wiring is not accepted.
- Substance status 007: the packet owns ITSM service authorization only and does not grant incident-management responder paging, ERP approval, or marketing automation access.
- Substance status 008: success means denied requests never reach workflow dispatch, ontology projection, connector replay, or marketplace settlement side effects.

## Authorization model
- Authorization model 001: Principal shape includes principal_id, tenant_id, audience_type, delegated_admin_grant_id, assurance_level, and compliance_pack_memberships.
- Authorization model 002: Action shape includes `itsm.incident.open`, `itsm.change.approve`, `itsm.problem.link`, `itsm.catalog.publish`, `itsm.cmdb.sync`, and `itsm.major_incident.bridge`.
- Authorization model 003: Resource shape includes tenant_id, bounded_context, object_ref, data_class, source_system_kind, home_cell_id, and retention_class.
- Authorization model 004: Context shape includes request_purpose, trace_id, idempotency_key, source_system_ref, emergency_bypass_flag, deal_set_id, and workflow_template_id.
- Authorization model 005: Every policy must deny when principal.tenant_id and resource.tenant_id differ unless delegated_admin_grant_id is present and valid.
- Authorization model 006: Every policy must deny when source_system_ref is present without source_system_kind and source_system_kind-specific fixture coverage.
- Authorization model 007: Every policy must deny when the action attempts to mutate a data_class outside the action's declared class roster.
- Authorization model 008: Every policy must deny when compliance packs require higher assurance than the principal presents.
- Authorization model 009: Every policy must deny when request_purpose is absent, generic, or not mapped to the ITIL operator reason catalog.
- Authorization model 010: Every policy must deny when deal_set_id is required by marketplace flow but absent from context.
- Authorization model 011: Every policy must deny when emergency_bypass_flag is true but the breakglass action class is not the emergency services bypass packet.
- Authorization model 012: Every policy must deny when workflow_template_id references a template outside the tenant's enabled catalog.

## Vendor displacement rules
- Vendor rule 001: ServiceNow roles and groups are imported as evidence attributes, not as permit sources.
- Vendor rule 002: ServiceNow table ACL semantics are mapped into tenant-scoped Cedar entities with explicit object refs.
- Vendor rule 003: Jira Service Management project roles are mapped into tenant-scoped queue grants and cannot grant CMDB relation writes.
- Vendor rule 004: Jira Service Management request participants are requester visibility fields, not fulfiller authorization fields.
- Vendor rule 005: BMC Helix support groups are mapped into resolver groups with tenant and data_class constraints.
- Vendor rule 006: BMC Helix problem investigations require incident and problem resources to share tenant_id and home_cell_id.
- Vendor rule 007: Ivanti Neurons discovered devices are resources with provenance and confidence; discovery does not imply operator authority.
- Vendor rule 008: Ivanti Neurons remediation recommendations are context fields and must not bypass change approval.
- Vendor rule 009: Freshservice requester roles are mapped to service-catalog visibility and cannot publish catalog items.
- Vendor rule 010: Freshservice asset assignment can update CMDB only when the actor holds explicit CMDB write scope.
- Vendor rule 011: Source-vendor admin status is never equivalent to Oyatie tenant owner status.
- Vendor rule 012: Imported vendor permissions are quarantined until the migration replay proves one-to-one Cedar policy mappings.

## Policy files and ownership
- Policy ownership 001: `microservices/itsm/policy/service-management-authorization.cedar` owns baseline ITSM allow rules.
- Policy ownership 002: `microservices/itsm/policy/abuse-defence.cedar` owns throttle, spray, and suspicious automation forbids.
- Policy ownership 003: `microservices/itsm/policy/auditor-scope.cedar` owns read-only audit access and cannot permit mutation actions.
- Policy ownership 004: `microservices/itsm/policy/ci-scope.cedar` owns CI fixture permissions and must be impossible to activate in production cells.
- Policy ownership 005: `microservices/itsm/policy/emergency-services-bypass.cedar` owns breakglass-only exceptions and must cite audit event classes.
- Policy ownership 006: `microservices/itsm/policy/data-residency.md` describes region and pack residency constraints consumed by policy tests.
- Policy ownership 007: `microservices/itsm/policies/local-incident-ticket-scope.cedar` narrows incident visibility and mutation rules.
- Policy ownership 008: `microservices/itsm/policies/local-change-approval-window.cedar` narrows change approval by freeze window and risk tier.
- Policy ownership 009: `microservices/itsm/policies/local-problem-link-control.cedar` prevents problem-link loops and cross-tenant correlation leaks.
- Policy ownership 010: `microservices/itsm/policies/local-service-catalog-publish-approval.cedar` requires catalog publisher authority and approval evidence.
- Policy ownership 011: `microservices/itsm/policies/local-cmdb-relation-write.cedar` requires both CMDB relation endpoints to belong to the same tenant scope.
- Policy ownership 012: `microservices/itsm/policies/local-sla-recompute-guard.cedar` prevents SLA recompute abuse from rewriting breach evidence.

## Implementation sequence
- Implementation 001: Add Cedar entity builder tests before changing policy fragments so expected entity shape is locked.
- Implementation 002: Encode principal entity with tenant, audience, assurance, delegated grant, and pack memberships.
- Implementation 003: Encode action entity with command kind, bounded context, mutability, and required data class.
- Implementation 004: Encode resource entity with tenant, home cell, object ref, data class, source-system kind, and version.
- Implementation 005: Encode context with purpose, trace, idempotency, deal set, workflow template, source ref, and emergency flag.
- Implementation 006: Add a no-policy-default-deny test proving an unknown ITSM action is denied when no rule matches.
- Implementation 007: Add a wrong-tenant test for every ITSM action kind.
- Implementation 008: Add a missing-purpose test for every ITSM action kind.
- Implementation 009: Add a wrong-data-class test for every ITSM action kind.
- Implementation 010: Add a delegated-admin grant test for managed-service-provider operation against a client tenant.
- Implementation 011: Add a delegated-admin expiry test that denies after validity window closes.
- Implementation 012: Add a source-vendor admin test proving imported ServiceNow admin evidence does not grant Oyatie mutation rights.
- Implementation 013: Add pack overlay tests for GDPR, KR-PIPA, FedRAMP-High, SOC-2, ISO-27001, and ITIL.
- Implementation 014: Add emergency bypass tests proving breakglass requires its own policy path and audit event class.
- Implementation 015: Add refusal evidence assembly after Cedar deny and before response serialization.
- Implementation 016: Add policy decision id propagation into REST, gRPC, AsyncAPI, workflow, and audit evidence.
- Implementation 017: Add a property test that randomly combines tenants, data classes, vendors, and actions and denies illegal combinations.
- Implementation 018: Add snapshot tests for Cedar fragments to prevent accidental broadening of permits.
- Implementation 019: Add negative fixtures for ServiceNow group import, Jira project admin, BMC support group, Ivanti discovery owner, and Freshservice requester.
- Implementation 020: Add positive fixtures only for explicitly mapped tenant-scoped roles.

## Deny matrix
- Deny matrix 001: Deny incident-open when principal lacks ITIL_OPERATOR or delegated service desk operator audience.
- Deny matrix 002: Deny incident-open when data_class is not incident_ticket.
- Deny matrix 003: Deny incident-open when source ref belongs to a different tenant import batch.
- Deny matrix 004: Deny change-approve when the actor is the requester and separation-of-duty pack overlay is active.
- Deny matrix 005: Deny change-approve when change freeze is active and no breakglass evidence exists.
- Deny matrix 006: Deny change-approve when risk tier exceeds the actor's approval ceiling.
- Deny matrix 007: Deny problem-link when incident and problem resources have different tenant_id or home_cell_id.
- Deny matrix 008: Deny problem-link when the link would create a cycle in the problem relation graph.
- Deny matrix 009: Deny problem-link when either side is under litigation hold and actor lacks auditor scope.
- Deny matrix 010: Deny service-catalog-publish when item owner, approver, and publisher are the same principal under SOC-2 overlay.
- Deny matrix 011: Deny service-catalog-publish when request form collects data classes not declared in the catalog item schema.
- Deny matrix 012: Deny service-catalog-publish when deal_set_id is missing for marketplace-provided catalog content.
- Deny matrix 013: Deny cmdb-sync when a relation endpoint lacks provenance or confidence score.
- Deny matrix 014: Deny cmdb-sync when a discovered asset attempts to overwrite a manually locked configuration item.
- Deny matrix 015: Deny cmdb-sync when an Ivanti relation crosses tenant or cell boundaries without approved replication policy.
- Deny matrix 016: Deny major-incident-bridge when incident-management has not accepted the bridge handoff.
- Deny matrix 017: Deny major-incident-bridge when stakeholder update egress policy forbids the target audience.
- Deny matrix 018: Deny major-incident-bridge when status page publication is requested without statuspage-sync authority.
- Deny matrix 019: Deny SLA recompute when recompute would remove already-sealed breach evidence.
- Deny matrix 020: Deny audit export when requested retention class conflicts with active pack overlay.

## Allow matrix
- Allow matrix 001: Allow direct tenant ITIL operator to open an incident ticket in the tenant's home cell with incident_ticket data class.
- Allow matrix 002: Allow delegated managed-service operator to triage an incident only when delegated grant includes incident-open.
- Allow matrix 003: Allow change approver to approve a low-risk change outside freeze windows when separation of duties passes.
- Allow matrix 004: Allow problem manager to link incident and problem records that share tenant, home cell, and permissible data class.
- Allow matrix 005: Allow catalog publisher to publish a catalog item after approval workflow id and deal set id are present.
- Allow matrix 006: Allow CMDB operator to sync an Ivanti relation when both endpoints share tenant and confidence threshold passes.
- Allow matrix 007: Allow major incident bridge when ITSM and incident-management both record the handoff and audit preimages align.
- Allow matrix 008: Allow auditor to read sealed policy decisions without granting mutation actions.
- Allow matrix 009: Allow CI harness to execute fixture-only permissions in non-production cells.
- Allow matrix 010: Allow emergency breakglass only through the emergency-services bypass packet with explicit reason and expiry.

## Evidence requirements
- Evidence 001: Every deny response contains policy_decision_id, policy_fragment_id, tenant hash, action, data_class, and refusal reason.
- Evidence 002: Every allow response contains policy_decision_id, audit_event_target, workflow_run_id when created, and ontology object ref when projected.
- Evidence 003: Every ServiceNow fixture includes source table, sys_id, imported group evidence, and mapped Cedar entities.
- Evidence 004: Every Jira fixture includes project key, request type, role evidence, and mapped Cedar entities.
- Evidence 005: Every BMC Helix fixture includes support group, problem investigation id, and mapped Cedar entities.
- Evidence 006: Every Ivanti fixture includes discovery source, device confidence, relation endpoints, and mapped Cedar entities.
- Evidence 007: Every Freshservice fixture includes requester role, catalog item, approval workflow, and mapped Cedar entities.
- Evidence 008: Every compliance pack fixture includes expected policy narrowing and explicit non-broadening assertion.
- Evidence 009: Every emergency fixture includes breakglass reason, expiry, approver, and audit-chain event id.
- Evidence 010: Every rollback fixture proves policy decisions remain sealed after command effects are reversed.

## Acceptance expansion
- Acceptance expansion 001: An intern can implement Cedar entity builders without guessing which fields belong to principal, action, resource, or context.
- Acceptance expansion 002: An intern can identify every policy file that must change and every policy file that must stay read-only for this packet.
- Acceptance expansion 003: An intern can write negative tests for all five displaced vendor permission models.
- Acceptance expansion 004: An intern can explain why imported vendor permissions are evidence, not authority.
- Acceptance expansion 005: An intern can explain why default-deny happens before workflow dispatch and ontology projection.
- Acceptance expansion 006: An intern can explain why breakglass is a separate policy path rather than an allow exception in every rule.
- Acceptance expansion 007: An intern can produce a policy PR with entity builders, Cedar fragments, denial tests, allow tests, refusal evidence, and rollback evidence.
- Acceptance expansion 008: An intern can verify that no action is permitted solely because ServiceNow, Jira, BMC, Ivanti, or Freshservice called the user an administrator.
- Acceptance expansion 009: An intern can verify that pack overlays narrow policy scope and never broaden it.
- Acceptance expansion 010: An intern can verify that every policy decision is traceable to audit-chain evidence under ADR-0263.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 sets the intern-buildability test for this policy implementation plan.
- Citation 002: microservices/itsm/manifest.json defines ITSM audience, compliance packs, layer conformance, and benchmark roster.
- Citation 003: microservices/itsm/PRD.md defines incident, problem, change, service catalog, and CMDB operational concerns.
- Citation 004: microservices/itsm/contracts/openapi-v1.yaml defines action invocation and action response evidence fields.
- Citation 005: ADR-0105 defines the policy boundary between kernel, domain, usecase, application, rest, worker, adapter, and governance layers.
- Citation 006: ADR-0244 defines default-deny authorization posture and forbids permissive fallback.
- Citation 007: ADR-0258 defines contract versioning obligations when policy fields appear in API, event, proto, SDK, and CLI surfaces.
- Citation 008: ADR-0263 defines audit-chain event discipline for policy allow, deny, breakglass, replay, and rollback.
- Citation 009: ADR-0314 defines DealSet settlement evidence required by commercial catalog and marketplace ITSM flows.
- Citation 010: ADR-0316 prevents ServiceNow, Jira, BMC, Ivanti, and Freshservice product labels from becoming authorization boundaries.
- Citation 011: ADR-0321 defines B2B leader parity expectations and forces policy depth beyond benchmark name repetition.
- Citation 012: docs/AGENTS.md supplies the Oya VCS lifecycle requirement that keeps this policy packet tied to claim, verify, done, and promote evidence.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
