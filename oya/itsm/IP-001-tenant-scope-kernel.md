# IP-001 ITSM tenant-scope-kernel

Service: itsm
ChangeSet scope: microservices/itsm/IP-001-tenant-scope-kernel.md
Benchmarks: ServiceNow ITSM, Jira Service Management, BMC Remedy, Zendesk Support, Freshdesk
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- tenant-scope-kernel-objective 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- tenant-scope-kernel-objective 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- tenant-scope-kernel-objective 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- tenant-scope-kernel-objective 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- tenant-scope-kernel-objective 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- tenant-scope-kernel-objective 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Prerequisites
- tenant-scope-kernel-prerequisites 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- tenant-scope-kernel-prerequisites 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- tenant-scope-kernel-prerequisites 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- tenant-scope-kernel-prerequisites 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- tenant-scope-kernel-prerequisites 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- tenant-scope-kernel-prerequisites 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Implementation steps
- tenant-scope-kernel-implementation-steps 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- tenant-scope-kernel-implementation-steps 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- tenant-scope-kernel-implementation-steps 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- tenant-scope-kernel-implementation-steps 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- tenant-scope-kernel-implementation-steps 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- tenant-scope-kernel-implementation-steps 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Tests and evidence
- tenant-scope-kernel-tests-and-evidence 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- tenant-scope-kernel-tests-and-evidence 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- tenant-scope-kernel-tests-and-evidence 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- tenant-scope-kernel-tests-and-evidence 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- tenant-scope-kernel-tests-and-evidence 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- tenant-scope-kernel-tests-and-evidence 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Rollback
- tenant-scope-kernel-rollback 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- tenant-scope-kernel-rollback 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- tenant-scope-kernel-rollback 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- tenant-scope-kernel-rollback 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- tenant-scope-kernel-rollback 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- tenant-scope-kernel-rollback 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Acceptance criteria
- tenant-scope-kernel-acceptance-criteria 001: ITSM binds incident-open to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.
- tenant-scope-kernel-acceptance-criteria 002: ITSM binds change-approve to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Jira Service Management plus BMC Remedy.
- tenant-scope-kernel-acceptance-criteria 003: ITSM binds problem-link to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=service_catalog_item, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against BMC Remedy plus Zendesk Support.
- tenant-scope-kernel-acceptance-criteria 004: ITSM binds service-catalog-publish to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=cmdb_relation, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Zendesk Support plus Freshdesk.
- tenant-scope-kernel-acceptance-criteria 005: ITSM binds cmdb-sync to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=incident_ticket, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Freshdesk plus ServiceNow ITSM.
- tenant-scope-kernel-acceptance-criteria 006: ITSM binds major-incident-bridge to tenant_id, principal_id, audience_type=ITIL_OPERATOR, data_class=change_request, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ServiceNow ITSM plus Jira Service Management.

## Batch B substance audit
- Substance status 001: the pre-pass file was 55 lines, so this IP required a direct substance rewrite rather than a cosmetic citation sweep.
- Substance status 002: the implementation target is the tenant-scope kernel for ITSM, not generic ticketing, CRM support, or incident-response paging.
- Substance status 003: the displaced benchmark set is ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, and Freshservice.
- Substance status 004: BMC Remedy, Zendesk Support, and Freshdesk mentions in the stamped seed remain historical scaffold residue and are superseded by this benchmark roster.
- Substance status 005: intern-buildability is assessed against docs/standards/documentation-rigor.md section 1.1 and the PR-143 microservice artifact floor.
- Substance status 006: the IP is buildable only if an implementer can create tenant isolation, policy hooks, audit events, rollback paths, and test fixtures without private context.
- Substance status 007: the claim boundary is microservices/itsm and this IP does not authorize changes to adjacent B2B-leader batches.
- Substance status 008: the packet is single-PR sized when limited to the ITSM tenant-scope kernel crate, its tests, and the local manifest references.

## Domain problem framing
- Problem framing 001: ITSM state is long lived and operator-controlled, so tenant identity must be attached before any incident, change, problem, catalog, or CMDB mutation is accepted.
- Problem framing 002: ServiceNow-style tables encourage broad cross-module joins; Oyatie must instead keep tenant scope explicit at every bounded-context edge.
- Problem framing 003: Jira Service Management projects encourage queue-specific permissions; Oyatie maps that concern to tenant_id, principal_id, audience_type, and policy decision evidence.
- Problem framing 004: BMC Helix ITSM automation depends on assignment groups; Oyatie translates assignment groups into tenant-scoped resolver policies and audited ownership transitions.
- Problem framing 005: Ivanti Neurons discovery creates device-context risk; Oyatie treats every configuration item and relation as tenant data with provenance, confidence, and residency tags.
- Problem framing 006: Freshservice service catalog flows mix requester and fulfiller data; Oyatie separates requester-visible fields from ITIL operator evidence with data_class annotations.
- Problem framing 007: The kernel must reject any action where tenant_id is inferred from route, session, object id, or source connector alone.
- Problem framing 008: The kernel must require caller, subject tenant, home cell, compliance pack, data class, purpose, idempotency key, and audit target before command construction.
- Problem framing 009: The kernel must make cross-tenant managed-service-provider work possible through explicit delegated administration grants, not shared queue visibility.
- Problem framing 010: The kernel must preserve pack overlays so GDPR, KR-PIPA, SOC-2, ISO-27001, ITIL, and FedRAMP-High can alter retention or disclosure without changing command code.

## Scope and non-scope
- Scope 001: Define the tenant-scoped command envelope for incident-open, change-approve, problem-link, service-catalog-publish, cmdb-sync, and major-incident-bridge.
- Scope 002: Define the immutable tenant context record used by REST, gRPC, worker, policy, and audit layers.
- Scope 003: Define validation failures for missing tenant context, mismatched home cell, expired delegated-admin grant, and forbidden pack overlay.
- Scope 004: Define how source-vendor identifiers are normalized without becoming authorization authorities.
- Scope 005: Define how tenant-scoped object refs flow into ontology projection, workflow execution, and audit-chain sealing.
- Scope 006: Define test fixtures covering one direct enterprise tenant, one managed-service-provider delegated tenant, and one rejected cross-tenant attempt.
- Scope 007: Define rollback fixtures that can erase staged tenant-scope rows without deleting sealed audit evidence.
- Scope 008: Out of scope: replacing incident-management paging semantics; this IP owns ITSM ticket and configuration evidence, not responder escalation.
- Scope 009: Out of scope: user-facing workflow studio screens; this IP exposes domain primitives consumed by later UX packets.
- Scope 010: Out of scope: vendor connector import jobs; this IP defines the envelope that connector jobs must use.

## Required primitives
- Primitive 001: `ItsmTenantContext` contains tenant_id, principal_id, audience_type, home_cell_id, delegated_admin_grant_id, compliance_pack_set, and request_purpose.
- Primitive 002: `ItsmTenantContext` contains source_system_kind so ServiceNow, Jira Service Management, BMC Helix, Ivanti, and Freshservice imports remain traceable.
- Primitive 003: `ItsmTenantContext` contains source_system_ref but the ref is never accepted as a substitute for tenant_id.
- Primitive 004: `ItsmTenantContext` contains data_class with values incident_ticket, change_request, problem_record, service_catalog_item, cmdb_ci, and cmdb_relation.
- Primitive 005: `ItsmTenantContext` contains idempotency_key with tenant_id embedded in the idempotency namespace.
- Primitive 006: `ItsmTenantContext` contains traceparent and audit_event_target for observability and audit-chain correlation.
- Primitive 007: `ItsmCommandEnvelope` wraps the domain command plus `ItsmTenantContext`, validation result, and policy decision id.
- Primitive 008: `ItsmTenantScopeError` enumerates missing_tenant, invalid_principal, wrong_home_cell, expired_delegation, pack_overlay_forbidden, and source_ref_without_tenant.
- Primitive 009: `ItsmObjectRef` contains tenant_id, bounded_context, local_id, source_system_kind, source_system_ref, and version.
- Primitive 010: `ItsmMutationGuard` exposes `validate_context`, `authorize_command`, `seal_preimage`, `dispatch`, and `seal_result`.
- Primitive 011: `ItsmTenantPartitionKey` uses tenant_id plus home_cell_id plus bounded_context to prevent cross-tenant queue coalescing.
- Primitive 012: `ItsmReplayCursor` stores tenant_id and source_system_kind so replay cannot traverse another tenant's imported ticket stream.

## Implementation sequence
- Implementation 001: Add the tenant context type in the ITSM kernel layer named by ADR-0105 and keep it free of HTTP, database, and vendor adapter concerns.
- Implementation 002: Add constructors that require every field explicitly; do not provide a default tenant or default principal.
- Implementation 003: Add parser helpers for REST and gRPC layers that convert authenticated claims into `ItsmTenantContext`.
- Implementation 004: Add validator logic that rejects audience types other than ITIL_OPERATOR or delegated managed-service operator types declared in manifest.json.
- Implementation 005: Add home-cell validation using the manifest's tier-1, tier-2, and tier-3 eligibility model.
- Implementation 006: Add source-system enum values for ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, and Freshservice.
- Implementation 007: Add an explicit `OtherVendor` variant only if it carries a catalog registration id and cannot bypass benchmark-specific fixtures.
- Implementation 008: Add data-class enum coverage for all ITSM object families listed in the PRD and contract files.
- Implementation 009: Add idempotency namespace construction that includes tenant_id, bounded_context, command_kind, and source_system_kind.
- Implementation 010: Add a no-tenant rejection path before policy evaluation so unauthorized data never reaches Cedar input assembly.
- Implementation 011: Add Cedar entity assembly only after tenant context validation succeeds.
- Implementation 012: Add audit preimage creation before dispatch so failed authorization still produces refusal evidence where policy requires it.
- Implementation 013: Add command-envelope serialization for worker handoff with tenant context preserved as immutable metadata.
- Implementation 014: Add a replay-safe cursor that refuses cursor tokens lacking tenant_id and source_system_kind.
- Implementation 015: Add rollback fixture constructors that recreate the tenant context from the original audit preimage.
- Implementation 016: Add OpenAPI schema references for tenant context fields without duplicating domain validation in the REST layer.
- Implementation 017: Add proto message fields for tenant context and mark all fields required through validation tests because proto3 itself lacks required semantics.
- Implementation 018: Add AsyncAPI message headers for tenant_id, home_cell_id, data_class, and audit_event_target.
- Implementation 019: Add internal metrics labels tenant_hash, bounded_context, source_system_kind, data_class, and outcome while avoiding raw tenant names.
- Implementation 020: Add fixture records for one imported ServiceNow incident, one Jira change, one BMC problem, one Ivanti CI relation, and one Freshservice catalog item.

## Policy and authorization
- Policy 001: Cedar input principal must include principal_id, audience_type, tenant_id, delegated_admin_grant_id, and compliance pack set.
- Policy 002: Cedar input action must include bounded_context, command_kind, data_class, and source_system_kind.
- Policy 003: Cedar input resource must include tenant_id, home_cell_id, object_ref, source_system_ref, and version.
- Policy 004: Cedar input context must include request_purpose, trace_id, idempotency_key, and requested_retention_class.
- Policy 005: Default deny applies before any ITSM action because ADR-0244 requires denied-by-default policy posture.
- Policy 006: A delegated administrator can operate only if grant tenant, subject tenant, source system, and command kind match.
- Policy 007: A managed-service provider cannot list another tenant's tickets by guessing source-system refs.
- Policy 008: A catalog publisher cannot change CMDB relation state unless the CMDB write permission is independently granted.
- Policy 009: An incident bridge can open a major incident record only after tenant scope and command purpose are preserved in the workflow input.
- Policy 010: Emergency bypass paths must preserve tenant scope and mark the audit event as breakglass rather than erasing policy context.
- Policy 011: Pack overlays can narrow command availability; they cannot broaden access beyond the base tenant-scope decision.
- Policy 012: Refusal evidence must include tenant context hashes, source-system kind, policy id, and bounded context without leaking raw secrets.

## Data model details
- Data model 001: Store tenant-scoped ITSM objects in partitions by tenant_id and home_cell_id before status or vendor type.
- Data model 002: Store source-system ids in a secondary index because vendor ids are not globally unique across tenants.
- Data model 003: Store object version per bounded context so incident, problem, change, catalog, and CMDB histories do not share sequence counters.
- Data model 004: Store source-system kind as an enum and source-system ref as a string with normalization metadata.
- Data model 005: Store data_class on every object and relation, even when the vendor object name appears self-evident.
- Data model 006: Store provenance for import source, manual operator action, workflow action, marketplace action, and replay action.
- Data model 007: Store `created_by_principal_id` and `last_mutated_by_principal_id` separately so ownership transfer is auditable.
- Data model 008: Store `delegated_admin_grant_id` only when a delegated actor is present; never synthesize a grant for direct tenant staff.
- Data model 009: Store compliance pack set at mutation time because later pack activation cannot rewrite historical context.
- Data model 010: Store audit-chain event ids for accepted, rejected, replayed, and rolled-back actions.
- Data model 011: Store rollback plan id beside each command envelope so cleanup can operate without searching free-form logs.
- Data model 012: Store workflow_run_id only after workflow dispatch; failed policy checks must not pretend a workflow existed.

## Contract obligations
- Contract 001: REST requests must include tenant_id, principal_id, purpose, data_class, deal_set_id when applicable, and idempotency_key.
- Contract 002: REST responses must return audit_event_class, workflow_run_id when created, ontology_object_ref when projected, and policy_decision_id.
- Contract 003: gRPC messages must include the same tenant context fields and use explicit validation errors for omitted fields.
- Contract 004: Async events must carry tenant_id and data_class headers so consumers can enforce residency before reading payloads.
- Contract 005: SDK clients must force tenant context construction before allowing a command builder to be finalized.
- Contract 006: CLI commands must require tenant id flags even for local fixture runs because fixtures train agent behavior.
- Contract 007: Catalog records must name the kernel crate that owns tenant context and the adapter crates that must consume it.
- Contract 008: OpenAPI examples must include one ServiceNow import, one Jira change approval, and one Ivanti CMDB relation write.
- Contract 009: AsyncAPI examples must include accepted, denied, replay-started, replay-completed, and rollback-completed event classes.
- Contract 010: Proto examples must include home cell and delegated grant fields so internal service calls do not lose scope.

## Tests and fixtures
- Test 001: Unit test `tenant_context_requires_tenant_id` rejects blank, missing, and whitespace tenant ids.
- Test 002: Unit test `tenant_context_rejects_source_ref_only` proves ServiceNow sys_id is not authorization scope.
- Test 003: Unit test `tenant_context_preserves_home_cell` proves home cell is part of the immutable context.
- Test 004: Unit test `tenant_context_requires_data_class` covers all six ITSM data classes.
- Test 005: Property test generates vendor source refs and proves none can cross tenant partition keys.
- Test 006: Authorization test denies a Jira change approval attempted by a ServiceNow-only delegated grant.
- Test 007: Authorization test denies a Freshservice catalog publish attempted without catalog publisher scope.
- Test 008: Authorization test permits a ServiceNow incident open for a direct ITIL operator with correct tenant context.
- Test 009: Authorization test permits a BMC Helix problem link only when incident and problem share tenant id.
- Test 010: Authorization test permits an Ivanti CMDB relation write only when both endpoints are tenant-scoped.
- Test 011: Replay test proves cursor tokens cannot be reused across tenants.
- Test 012: Replay test proves cursor tokens cannot switch source-system kind midway.
- Test 013: Worker test proves command envelopes keep tenant context through queue serialization.
- Test 014: REST contract test rejects missing tenant_id before command handler execution.
- Test 015: gRPC contract test rejects missing data_class before usecase invocation.
- Test 016: Async contract test verifies tenant_id and data_class headers are emitted.
- Test 017: Audit test verifies denied commands emit refusal evidence with tenant hash and policy decision id.
- Test 018: Rollback test verifies rollback command loads original tenant context from audit preimage.
- Test 019: Metrics test verifies labels use tenant hash and never raw tenant name.
- Test 020: Fixture test verifies ServiceNow, Jira Service Management, BMC Helix, Ivanti Neurons, and Freshservice examples are present.

## Migration and rollout
- Rollout 001: Start with kernel-only tenant context types and tests; do not wire REST until context validation passes.
- Rollout 002: Wire REST command parsing behind existing ITSM action endpoints after test fixtures are green.
- Rollout 003: Wire gRPC and worker serialization after REST and kernel agree on field names.
- Rollout 004: Wire Cedar input assembly only after command envelope immutability is proven.
- Rollout 005: Wire audit preimage emission before accepted-result emission.
- Rollout 006: Wire replay cursor enforcement before any vendor import job can use the ITSM pipeline.
- Rollout 007: Wire rollback fixtures before enabling production replay or migration operations.
- Rollout 008: Gate promotion on line-count, citation-density, fixture, contract, policy, and audit evidence checks.
- Rollout 009: Use cell canaries by tenant cohort; never use a global ITSM canary because tenant scope is the property under test.
- Rollout 010: Disable only the new command-envelope path on rollback; do not disable sealed audit-chain evidence.

## Acceptance expansion
- Acceptance expansion 001: An intern can identify the owner type, source file family, command envelope fields, and validation sequence from this IP alone.
- Acceptance expansion 002: An intern can implement the tenant context type without guessing whether vendor ids are authorization authorities.
- Acceptance expansion 003: An intern can build the first tests without searching Slack, old tickets, or private notes.
- Acceptance expansion 004: An intern can explain how ServiceNow, Jira Service Management, BMC Helix, Ivanti Neurons, and Freshservice are displaced.
- Acceptance expansion 005: An intern can explain why incident-management remains separate from ITSM even when major incidents overlap.
- Acceptance expansion 006: An intern can explain why Freshservice-style catalog request data is separated from CMDB relation evidence.
- Acceptance expansion 007: An intern can explain why delegated administration requires a grant id instead of shared project membership.
- Acceptance expansion 008: An intern can explain why audit preimage exists before workflow dispatch.
- Acceptance expansion 009: An intern can explain why compliance packs can narrow access but cannot broaden it.
- Acceptance expansion 010: An intern can produce a PR that includes kernel code, policy tests, contract examples, and rollback fixtures.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 sets the intern-buildability bar applied to this rewrite.
- Citation 002: docs/AGENTS.md requires Oya VCS claim, verify, done, and promote lifecycle discipline for scoped changes.
- Citation 003: microservices/itsm/manifest.json defines audience_type tenant-b2b-it, layer conformance, and compliance pack roster.
- Citation 004: microservices/itsm/PRD.md defines incident-ticket, problem, change, service-request, and configuration-item as the bounded contexts.
- Citation 005: microservices/itsm/contracts/openapi-v1.yaml defines the initial REST action surface and data_class enum.
- Citation 006: ADR-0105 supplies the layer enum boundary between kernel, domain, usecase, application, rest, worker, adapter, and governance layers.
- Citation 007: ADR-0244 supplies default-deny authorization expectations inherited by ITSM policy packets.
- Citation 008: ADR-0253-amendment supplies HTTP/3, ECH, and PQC transport expectations for service contracts.
- Citation 009: ADR-0258 supplies contract versioning and deprecation expectations for OpenAPI, AsyncAPI, proto, SDK, and CLI surfaces.
- Citation 010: ADR-0263 supplies audit-event registry discipline for accepted, denied, replayed, and rolled-back ITSM mutations.
- Citation 011: ADR-0314 supplies marketplace DealSet settlement requirements for commercial ITSM actions.
- Citation 012: ADR-0316 supplies the capability-tier doctrine that prevents vendor labels from becoming service boundaries.
- Citation 013: ADR-0321 supplies B2B leader microservice parity expectations and prevents shallow benchmark name-dropping.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/itsm/IP-001-tenant-scope-kernel.md` matched [`emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/itsm/IP-001-tenant-scope-kernel.md`, `microservices/itsm/manifest.json`, `microservices/itsm/capacity-model.md`, `microservices/itsm/compliance.md`, `microservices/itsm/ARCHITECTURE.md`].
