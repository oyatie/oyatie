# IP-026 ITSM itil-process-normalizer

Service: itsm
ChangeSet scope: microservices/itsm/IP-026-itil-process-normalizer.md
Benchmarks displaced: ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, Freshservice
Binding ADRs: ADR-0105, ADR-0131, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0258, ADR-0263, ADR-0314, ADR-0316, ADR-0321

## Objective
- Objective 001: Build an ITIL process normalizer that translates vendor process vocabulary into canonical Oyatie incident, problem, change, catalog, and CMDB semantics.
- Objective 002: Preserve vendor provenance while preventing ServiceNow, Jira Service Management, BMC Helix, Ivanti Neurons, or Freshservice labels from becoming canonical type names.
- Objective 003: Produce deterministic normalized process records that downstream tenant scope, Cedar policy, ontology projection, workflow templates, REST contracts, AsyncAPI events, and replay workers can consume.
- Objective 004: Make migration and greenfield operation share the same normalized process grammar so imported vendor data does not receive a weaker path.
- Objective 005: Attach tenant_id, home_cell_id, data_class, source_system_kind, process_family, process_stage, and audit preimage id to every normalized record.
- Objective 006: Make the normalizer intern-buildable under docs/standards/documentation-rigor.md section 1.1 with no private product notes.

## Problem framing
- Problem 001: ServiceNow ITSM uses table and workflow names that look canonical inside ServiceNow but must be reduced to source provenance in Oyatie.
- Problem 002: Jira Service Management uses issue types, request types, queues, and SLAs that often mix requester and operator concerns.
- Problem 003: BMC Helix ITSM uses form and support-group vocabulary that can hide tenant, role, and approval semantics.
- Problem 004: Ivanti Neurons uses discovery, automation, and device-state vocabulary that can overstate confidence in CMDB relations.
- Problem 005: Freshservice uses requester-friendly catalog and asset terms that can blur service catalog request, catalog publication, and CMDB write boundaries.
- Problem 006: Without a normalizer, every adapter would reimplement vendor mapping and create inconsistent policy, ontology, and workflow inputs.
- Problem 007: Without deterministic process stages, replay cannot prove imported vendor history reaches the same command state on every run.
- Problem 008: Without data_class preservation, compliance pack overlays cannot narrow or reject process transitions.

## Canonical process families
- Process family 001: `incident_management` handles incident open, triage, assignment, resolution, closure, SLA evidence, and major incident bridge request.
- Process family 002: `problem_management` handles problem candidate, correlation, root-cause hypothesis, workaround, known error, remediation, and closeout.
- Process family 003: `change_enablement` handles standard, normal, emergency, freeze override, approval, implementation, verification, rollback, and post-change review.
- Process family 004: `service_request_management` handles catalog request, entitlement, approval, fulfillment, requester notification, cancellation, and closeout.
- Process family 005: `service_catalog_management` handles catalog draft, schema review, entitlement review, approval, publication, revocation, and marketplace settlement.
- Process family 006: `configuration_management` handles CI discovery, CI ownership, lifecycle state, relation confidence, relation write, drift repair, and verification.
- Process family 007: `knowledge_management` handles workaround publication and knowledge link evidence when problem or incident flows publish operator guidance.
- Process family 008: `sla_management` handles clock start, pause, resume, breach, recompute, seal, and evidence export.

## Canonical process stages
- Process stage 001: `received` accepts externally sourced or native input after tenant context validation.
- Process stage 002: `classified` assigns process family, data_class, source_system_kind, and vendor provenance.
- Process stage 003: `authorized` records Cedar default-deny allow evidence for the requested transition.
- Process stage 004: `projected` records ontology object or edge creation where the process needs durable graph state.
- Process stage 005: `routed` records resolver, approver, catalog owner, CMDB owner, or responder bridge target.
- Process stage 006: `approved` records approval, separation-of-duty, freeze-window, breakglass, or entitlement decision.
- Process stage 007: `executing` records workflow run id, worker dispatch id, idempotency key, and progress surface.
- Process stage 008: `verified` records completion checks, evidence attachments, and expected state validation.
- Process stage 009: `sealed` records audit-chain completion and immutable evidence refs.
- Process stage 010: `rolled_back` records rollback run, preserved audit evidence, and compensation outcome.
- Process stage 011: `denied` records policy, validation, pack overlay, confidence, source mapping, or tenant-scope refusal.
- Process stage 012: `replayed` records deterministic replay completion and payload digest comparison.

## Vendor mapping rules
- Vendor mapping 001: ServiceNow `incident` table maps to `incident_management` with incident_ticket data class.
- Vendor mapping 002: ServiceNow `problem` table maps to `problem_management` with problem_record data class.
- Vendor mapping 003: ServiceNow `change_request` table maps to `change_enablement` with change_request data class.
- Vendor mapping 004: ServiceNow `sc_cat_item` maps to `service_catalog_management` and never bypasses entitlement policy.
- Vendor mapping 005: ServiceNow `cmdb_ci` maps to `configuration_management` with cmdb_ci data class and confidence metadata.
- Vendor mapping 006: Jira Service Management issue type `Incident` maps to `incident_management` only after request-type validation.
- Vendor mapping 007: Jira Service Management issue type `Change` maps to `change_enablement` only after approval semantics are explicit.
- Vendor mapping 008: Jira queues map to routing metadata, not canonical process families.
- Vendor mapping 009: Jira SLA fields map to `sla_management` with clock state and breach evidence.
- Vendor mapping 010: BMC Helix incident forms map to `incident_management` with support-group provenance.
- Vendor mapping 011: BMC Helix problem investigation forms map to `problem_management` with known-error evidence.
- Vendor mapping 012: BMC Helix change forms map to `change_enablement` with approval and status reason fields.
- Vendor mapping 013: Ivanti Neurons device inventory maps to `configuration_management` with discovery source and confidence.
- Vendor mapping 014: Ivanti Neurons automation recommendation maps to `change_enablement` as suggested change input, not execution.
- Vendor mapping 015: Ivanti Neurons relation evidence maps to CMDB relation only after endpoint tenant equality passes.
- Vendor mapping 016: Freshservice ticket maps to incident or service request based on request type and fulfillment fields.
- Vendor mapping 017: Freshservice service item maps to `service_catalog_management` with requester and fulfiller field separation.
- Vendor mapping 018: Freshservice asset maps to `configuration_management` only with ownership, lifecycle, and discovery provenance.
- Vendor mapping 019: Unknown vendor object kind maps to `denied` with unsupported mapping evidence.
- Vendor mapping 020: Ambiguous vendor object kind maps to `classified_pending_review` and cannot execute a mutating workflow.

## Normalized record schema
- Schema 001: `normalized_process_id` is deterministic from tenant_id, source_system_kind, source_system_ref, process_family, and version.
- Schema 002: `tenant_id` is mandatory and never inferred from source_system_ref.
- Schema 003: `home_cell_id` is mandatory for residency and replay routing.
- Schema 004: `principal_id` is mandatory for native operations and migration worker actor identity.
- Schema 005: `audience_type` is mandatory and defaults are forbidden.
- Schema 006: `delegated_admin_grant_id` is optional but required for managed-service-provider tenant actions.
- Schema 007: `source_system_kind` is enum-backed and includes ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, Freshservice, and native.
- Schema 008: `source_system_ref` is provenance and cannot become primary authorization key.
- Schema 009: `process_family` uses the canonical family roster in this IP.
- Schema 010: `process_stage` uses the canonical stage roster in this IP.
- Schema 011: `data_class` uses incident_ticket, problem_record, change_request, service_catalog_item, cmdb_ci, cmdb_relation, knowledge_record, or sla_evidence.
- Schema 012: `vendor_payload_digest` supports replay drift detection.
- Schema 013: `mapping_version` supports deterministic remapping across contract releases.
- Schema 014: `audit_preimage_id` ties the normalized record to ADR-0263 audit evidence.
- Schema 015: `policy_decision_id` ties the normalized record to Cedar default-deny evidence.
- Schema 016: `ontology_object_ref` is nullable until projection succeeds.
- Schema 017: `workflow_run_id` is nullable until workflow dispatch succeeds.
- Schema 018: `deal_set_id` is nullable except marketplace-backed catalog processes.
- Schema 019: `compliance_pack_set` records active pack overlays at normalization time.
- Schema 020: `rollback_plan_id` is nullable until a mutating workflow creates compensatable effects.

## Implementation sequence
- Implementation 001: Add source-system enum and canonical process-family enum in the ITSM kernel layer.
- Implementation 002: Add process-stage enum with transition validation helper.
- Implementation 003: Add normalized record struct with all mandatory context and provenance fields.
- Implementation 004: Add vendor mapper interface with one mapper per displaced benchmark source.
- Implementation 005: Add ServiceNow mapper fixtures for incident, problem, change, catalog, and CMDB examples.
- Implementation 006: Add Jira Service Management mapper fixtures for incident, change, queue, request type, and SLA examples.
- Implementation 007: Add BMC Helix mapper fixtures for incident, problem investigation, change, support group, and status reason examples.
- Implementation 008: Add Ivanti Neurons mapper fixtures for device, relation, recommendation, owner, and confidence examples.
- Implementation 009: Add Freshservice mapper fixtures for ticket, service item, asset, requester, and approval examples.
- Implementation 010: Add unknown-object mapper that returns denied evidence rather than fallback success.
- Implementation 011: Add ambiguous-object mapper that returns review-needed evidence rather than executing mutation.
- Implementation 012: Add replay digest comparison and deterministic normalized id generation.
- Implementation 013: Add REST and worker integration points without allowing adapters to bypass the normalizer.
- Implementation 014: Add AsyncAPI event for `itsm.process.normalized.v1`.
- Implementation 015: Add audit event class references for normalized, denied, replay-drift, and rollback-ready states.

## Test matrix
- Test matrix 001: Unit test maps ServiceNow incident to incident_management and incident_ticket.
- Test matrix 002: Unit test maps ServiceNow change to change_enablement and change_request.
- Test matrix 003: Unit test maps Jira queue metadata without changing process family.
- Test matrix 004: Unit test maps Jira SLA fields to sla_management.
- Test matrix 005: Unit test maps BMC problem investigation to problem_management.
- Test matrix 006: Unit test maps BMC support group as routing metadata only.
- Test matrix 007: Unit test maps Ivanti device to configuration_management and cmdb_ci.
- Test matrix 008: Unit test maps Ivanti recommendation to suggested change input.
- Test matrix 009: Unit test maps Freshservice service item to service_catalog_management.
- Test matrix 010: Unit test maps Freshservice asset to configuration_management only with owner and confidence fields.
- Test matrix 011: Negative test rejects missing tenant_id for every vendor mapper.
- Test matrix 012: Negative test rejects source ref only for every vendor mapper.
- Test matrix 013: Negative test rejects unknown vendor object kind with denied evidence.
- Test matrix 014: Negative test holds ambiguous vendor object kind for review without mutation.
- Test matrix 015: Property test proves normalized ids differ across tenants for identical vendor source refs.
- Test matrix 016: Property test proves mapping version changes do not rewrite historical normalized records.
- Test matrix 017: Replay test proves same vendor payload digest maps to same normalized record.
- Test matrix 018: Replay test detects digest drift and emits replay-drift evidence.
- Test matrix 019: Contract test verifies normalized event carries tenant_id, data_class, process_family, and source_system_kind.
- Test matrix 020: Audit test verifies normalized and denied outcomes both emit ADR-0263 evidence.

## Operational behavior
- Operational behavior 001: Normalization runs before policy action execution when imported vendor data enters a mutable workflow.
- Operational behavior 002: Normalization can run in dry-run mode to preview migration gaps.
- Operational behavior 003: Normalization can run in replay mode to prove deterministic mapping after schema changes.
- Operational behavior 004: Normalization never calls external vendor APIs after receiving the input payload.
- Operational behavior 005: Normalization writes no business state until tenant validation and audit preimage creation succeed.
- Operational behavior 006: Normalization denies or holds ambiguous payloads instead of picking the closest canonical type.
- Operational behavior 007: Normalization emits metrics by source_system_kind, process_family, data_class, outcome, and mapping_version.
- Operational behavior 008: Normalization emits no raw tenant names or secret-bearing vendor payload fields in logs.
- Operational behavior 009: Normalization stores payload digest, not the entire vendor payload, unless retention policy permits payload storage.
- Operational behavior 010: Normalization exposes remediation hints for unsupported mapping, missing tenant, missing data class, and ambiguous process family.

## Rollback and replay
- Rollback 001: Rollback uses normalized_process_id to find compensatable effects.
- Rollback 002: Rollback preserves the original audit_preimage_id and policy_decision_id.
- Rollback 003: Rollback can tombstone normalized records created in error without deleting sealed evidence.
- Rollback 004: Rollback cannot remap a vendor object to a different process family without a new mapping_version and audit event.
- Rollback 005: Replay compares vendor_payload_digest before attempting any mutation.
- Rollback 006: Replay refuses to run when mapping_version is missing or unknown.
- Rollback 007: Replay emits drift evidence when canonical output changes unexpectedly.
- Rollback 008: Replay can run tenant-by-tenant and source-system-by-source-system.
- Rollback 009: Replay metrics feed data-warehouse only through tenant-scoped aggregate events.
- Rollback 010: Replay completion returns normalized record refs, denied refs, drift refs, and rollback-ready refs.

## Acceptance criteria
- Acceptance 001: An intern can implement the source-system enum, process-family enum, process-stage enum, and normalized record struct from this IP.
- Acceptance 002: An intern can write one mapper per displaced benchmark vendor without copying product-specific types into canonical names.
- Acceptance 003: An intern can explain why unknown and ambiguous mappings are denied or held rather than guessed.
- Acceptance 004: An intern can implement deterministic id generation and replay drift detection.
- Acceptance 005: An intern can implement ServiceNow, Jira Service Management, BMC Helix, Ivanti Neurons, and Freshservice fixtures.
- Acceptance 006: An intern can wire normalizer output into tenant scope, Cedar, ontology, workflow, REST, AsyncAPI, and audit paths.
- Acceptance 007: An intern can build dry-run, replay, rollback, and metrics behavior without private notes.
- Acceptance 008: An intern can prove vendor source refs are provenance and never authorization keys.
- Acceptance 009: An intern can produce migration gap reports that include unsupported and ambiguous mappings.
- Acceptance 010: An intern can produce a PR with kernel types, mappers, fixtures, contract events, audit evidence, metrics, replay, and rollback tests.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 defines the intern-buildability bar for this IP.
- Citation 002: microservices/itsm/manifest.json supplies the benchmark roster, layer conformance, audience type, and pack roster.
- Citation 003: microservices/itsm/PRD.md supplies incident, problem, change, service request, and configuration item bounded contexts.
- Citation 004: microservices/itsm/contracts/openapi-v1.yaml supplies the REST action surface consumed after normalization.
- Citation 005: ADR-0105 defines where normalizer kernel, usecase, worker, rest, adapter, and governance code belongs.
- Citation 006: ADR-0131 defines ontology projection expectations consumed by normalized output.
- Citation 007: ADR-0244 defines Cedar default-deny obligations that consume normalized action and resource semantics.
- Citation 008: ADR-0246 defines library-first dispatch expectations so vendor mapping logic is shared and tested.
- Citation 009: ADR-0258 defines versioning expectations for normalizer contracts and mapping versions.
- Citation 010: ADR-0263 defines audit-chain event discipline for normalized, denied, replayed, and rolled-back states.
- Citation 011: ADR-0314 defines DealSet evidence fields needed by marketplace-backed catalog normalization.
- Citation 012: ADR-0316 prevents benchmark product labels from becoming canonical process boundaries.
- Citation 013: ADR-0321 defines the B2B leader parity bar that this ITIL normalizer satisfies.

## Additional implementation guardrails
- Guardrail 001: The normalizer must not parse markdown runbooks as mapping configuration; mapping tables live in typed code or typed fixtures.
- Guardrail 002: The normalizer must not accept adapter-provided canonical process_family values without recomputing them.
- Guardrail 003: The normalizer must not allow source_system_kind to be free-form text in persisted records.
- Guardrail 004: The normalizer must not coerce a vendor severity into Oyatie impact or urgency without an explicit mapping table.
- Guardrail 005: The normalizer must not coerce a vendor status into Oyatie process_stage without a transition-validity check.
- Guardrail 006: The normalizer must not treat ServiceNow assignment group, Jira project role, BMC support group, Ivanti owner, or Freshservice agent role as authority.
- Guardrail 007: The normalizer must not write ontology refs directly; it returns normalized input for the projection packet.
- Guardrail 008: The normalizer must not publish audit completion events for downstream steps it did not execute.
- Guardrail 009: The normalizer must not collapse incident and service-request paths solely because a vendor models both as tickets.
- Guardrail 010: The normalizer must not collapse catalog publication and catalog request paths solely because a vendor models both under service catalog.
- Guardrail 011: The normalizer must not collapse CI ownership and CI discovery confidence into one field.
- Guardrail 012: The normalizer must not retry indefinitely; retry policy must include capped attempts, tenant hash, vendor kind, and mapping gap reason.
- Guardrail 013: The normalizer must not emit raw vendor payloads to logs, metrics, or audit-chain events.
- Guardrail 014: The normalizer must not accept mapping_version downgrade during replay.
- Guardrail 015: The normalizer must not promote dry-run output to mutable workflow input without a fresh tenant and policy evaluation.
- Guardrail 016: The normalizer must not broaden compliance pack behavior when a benchmark fixture lacks a pack-specific field.
- Guardrail 017: The normalizer must not create a new microservice dependency; it stays inside ITSM and consumes existing substrate services.
- Guardrail 018: The normalizer must not modify manifests, journeys, ADR-0321, ERP, or other B2B leader batches as part of this IP.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
