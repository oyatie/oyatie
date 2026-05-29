# IP-005 Healthcare Integration REST Contract Surface

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-005-rest-contract-surface.md
Batch: Batch C healthcare-integration IP deepening
Status: implementation-plan
Owner: axis-healthcare-integration + council-api
Primary layer: api + rest
Primary contract: microservices/healthcare-integration/contracts/openapi-v1.yaml
Primary transport: HTTP/3 default, HTTP/2 and HTTP/1.1 fallback, strict TLS 1.3, ECH, PQC hybrid where negotiated
Primary bounded contexts: patient-record, fhir-resource, hl7-message, referral, clinical-consent
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md
Repo references: microservices/healthcare-integration/ARCHITECTURE.md
Repo references: microservices/healthcare-integration/manifest.json
Repo references: microservices/healthcare-integration/contracts/openapi-v1.yaml
Repo references: microservices/healthcare-integration/contracts/local-openapi-v1.yaml
Repo references: microservices/healthcare-integration/contracts/healthcare-integration-v1.proto
Repo references: microservices/healthcare-integration/contracts/local-operations-v1.proto
Repo references: microservices/healthcare-integration/sdk-plan.md
Repo references: microservices/healthcare-integration/slos/read-latency.openslo.yaml
Repo references: microservices/healthcare-integration/slos/write-latency.openslo.yaml
Repo references: microservices/healthcare-integration/slos/policy-decision-latency.openslo.yaml
Repo references: docs/decisions/ADR-0253-amendment-http3-fallback-strict-tls-ech-pqc.md
Repo references: docs/decisions/ADR-0258-api-versioning-model.md

## Objective
- Define the v1 REST contract surface for Healthcare Integration without letting REST become the source of clinical truth.
- Make REST a command/query front door into tenant scope, Cedar policy, workflow templates, ontology projection, and async event publication.
- Preserve ADR-0258 versioning discipline across route names, request fields, response fields, and deprecation signals.
- Preserve ADR-0253-amendment transport posture across HTTP/3, fallback, strict TLS, ECH, and PQC negotiation.
- Preserve ADR-0244 tenant scope on every request.
- Preserve ADR-0243 Cedar gate before provider or storage access.
- Preserve ADR-0263 audit event prerequisites in all accepted responses.
- Preserve ADR-0314 DealSet obligation references where commercial connector routes are invoked.
- Preserve ADR-0321 B2B leader substance through explicit healthcare integration endpoints.

## REST Surface Shape
- Base path is /healthcare-integration/v1.
- OpenAPI authority is contracts/openapi-v1.yaml.
- Local deployment projection is contracts/local-openapi-v1.yaml.
- Internal synchronous shape is contracts/healthcare-integration-v1.proto when REST delegates to gRPC.
- Local internal operations shape is contracts/local-operations-v1.proto.
- listHealthcareIntegrationCapabilities returns tenant-scoped capability availability.
- invokeHealthcareIntegrationAction accepts Cedar-gated actions and returns 202 for async-safe work.
- getHealthcareIntegrationActionStatus returns workflow progress and evidence links.
- listHealthcareIntegrationEvidence returns PHI-safe evidence summaries.
- getHealthcareIntegrationEvidencePacket returns regulator/tenant evidence when policy permits.
- previewHealthcareIntegrationTransform returns transform preview without canonical mutation.
- approveHealthcareIntegrationReview records human review outcomes.
- revokeHealthcareIntegrationTemporaryAccess expires break-glass or temporary grants.
- retryHealthcareIntegrationAction retries idempotent failed actions.
- cancelHealthcareIntegrationAction cancels pending workflow when rollback bundle exists.
- exportHealthcareIntegrationRollbackBundle exports rollback metadata when policy permits.

## Required Headers
- Authorization is required.
- X-Oyatie-Tenant-Id is required.
- X-Oyatie-Principal-Id is required.
- X-Oyatie-Audience-Type is required.
- X-Oyatie-Home-Cell is required.
- X-Oyatie-Jurisdiction-Code is required.
- X-Oyatie-Trace-Context is required.
- Idempotency-Key is required for POST, PATCH, DELETE, retry, and cancel.
- X-Oyatie-DealSet-Id is required when commercial route or marketplace settlement applies.
- X-Oyatie-Policy-Pack-Ids is required for compliance-sensitive operations.
- X-Oyatie-Api-Version is required and must match v1 compatibility.
- X-Oyatie-Transport-Profile advertises h3-h2-h1-strict-tls13-ech-pqc.
- X-Oyatie-Client-Capability identifies SDK or tenant integration client.
- X-Oyatie-Emergency-Attestation is required for emergency services paths.
- X-Oyatie-Source-System-Ref is required for source-system access.
- X-Oyatie-Workflow-Run-Id is returned after accepted async work.

## Request Fields
- tenant_id must match authenticated tenant header.
- principal_id must match authenticated principal header.
- audience_type must match authenticated audience header.
- purpose must be explicit.
- data_class must be explicit.
- action_id must be one of fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, patient-match-review, referral-handoff, provider-directory-sync, bulk-backfill-replay, or evidence-export.
- source_system_ref is required for adapter work.
- ontology_object_ref is optional on first import and required on update/export where projection exists.
- workflow_template_id is required for async template invocation.
- deal_set_id is required when marketplace settlement applies.
- emergency_attestation is required for emergency services.
- break_glass_justification is required for break-glass.
- reviewer_ref is required for review outcome endpoints.
- consent_ref is required when purpose depends on consent.
- referral_ref is required when cross-tenant referral context applies.
- residency_label is supplied by service after pack evaluation, not trusted from clients.
- credential_lease_ref is never supplied by external clients.

## Response Fields
- request_id is returned on every response.
- tenant_id is returned on every success response.
- action_id is returned on action responses.
- workflow_run_id is returned for async work.
- audit_event_class is returned for accepted work.
- audit_evidence_ref is returned when evidence is sealed or pending.
- cedar_decision_id is returned for accepted and denied decisions when safe.
- ontology_object_ref is returned when projection exists.
- status is returned with accepted, running, blocked, completed, denied, failed_reversible, failed_quarantined, cancelled, or rolled_back.
- denial_code is returned for policy, tenant, pack, residency, idempotency, or validation failures.
- denial_message is PHI-safe.
- rollback_bundle_ref is returned before externally visible mutation completes.
- retry_after is returned for transient backpressure.
- pack_decision_ref is returned when pack overlays affect behavior.
- deal_set_id is returned when settlement applies.
- transport_profile is returned for diagnostics.

## Endpoint Commitments
- GET /healthcare-integration/v1/capabilities lists capability records for the authenticated tenant.
- POST /healthcare-integration/v1/actions/{action_id} invokes an action through TenantScope and Cedar.
- GET /healthcare-integration/v1/actions/{action_id}/runs/{workflow_run_id} reads status without exposing PHI.
- POST /healthcare-integration/v1/actions/{action_id}/runs/{workflow_run_id}/retry retries idempotent failures.
- POST /healthcare-integration/v1/actions/{action_id}/runs/{workflow_run_id}/cancel cancels pending work with rollback evidence.
- POST /healthcare-integration/v1/transforms/preview previews source-to-ontology mapping.
- POST /healthcare-integration/v1/reviews/{review_id}/approve records human approval.
- POST /healthcare-integration/v1/reviews/{review_id}/reject records human rejection.
- POST /healthcare-integration/v1/break-glass/{event_id}/revoke revokes temporary access.
- GET /healthcare-integration/v1/evidence lists evidence summaries.
- GET /healthcare-integration/v1/evidence/{evidence_id} returns policy-cleared evidence detail.
- POST /healthcare-integration/v1/evidence/export creates an evidence export workflow.
- GET /healthcare-integration/v1/rollback/{rollback_bundle_id} returns rollback metadata when policy permits.
- GET /healthcare-integration/v1/provider-directory searches provider metadata without granting PHI access.
- GET /healthcare-integration/v1/openapi returns current contract metadata and deprecation status.

## Versioning Rules
- v1 request fields cannot be removed without ADR-0258 deprecation.
- v1 response fields cannot be narrowed without ADR-0258 deprecation.
- New required fields require minor compatibility plan and dual-read period.
- New action ids require capability record and policy fragment.
- New data classes require TenantScope, Cedar, ontology, event, SLO, and evidence updates.
- Deprecated fields return warnings before enforcement.
- Deprecated action ids return sunset metadata.
- Contract examples must include tenant_id, principal_id, purpose, data_class, audit_event_class, and idempotency.
- SDK generation follows sdk-plan.md and cannot hide policy or audit fields.
- Local contract cannot weaken production contract requirements.
- Proto projection cannot add privileged fields absent from REST contract authority.

## Competitor Displacement
- Redox displacement: Redox API aggregation is displaced by a tenant-scoped REST contract that returns policy, workflow, ontology, and audit references.
- Rhapsody displacement: Rhapsody route operations are displaced by REST commands that expose idempotency, status, rollback, and evidence.
- InterSystems IRIS for Health displacement: IRIS REST access is displaced by Oyatie v1 contract versioning and cell-aware tenancy.
- Lyniate/Corepoint displacement: Corepoint management APIs are displaced by governed action/status/review endpoints.
- Mirth displacement: Mirth channel HTTP listeners are displaced by an API-first surface with Cedar and audit in the contract.
- NextGate displacement: NextGate matching APIs are displaced by review-bound patient-match endpoints that do not auto-merge tenant identity.
- Health Catalyst displacement: Health Catalyst data access APIs are displaced by policy-cleared evidence export endpoints.
- Epic displacement: Epic FHIR APIs become source_system_ref targets behind common action contracts.
- Cerner displacement: Cerner APIs become source adapters, not Oyatie REST authority.
- Allscripts displacement: Allscripts endpoint variance becomes transform preview and source provenance.
- Veeva displacement: Veeva regulated workflow APIs become pack-aware action templates behind common contract.

## Implementation Slices
- Slice 1: expand contracts/openapi-v1.yaml route list.
- Slice 2: add header schema components.
- Slice 3: add TenantScope request schema.
- Slice 4: add ActionRequest schema per capability family.
- Slice 5: add ActionAccepted schema with workflow and audit fields.
- Slice 6: add ActionStatus schema.
- Slice 7: add EvidenceSummary schema.
- Slice 8: add EvidenceExportRequest schema.
- Slice 9: add TransformPreviewRequest schema.
- Slice 10: add ReviewDecisionRequest schema.
- Slice 11: add RollbackBundle schema.
- Slice 12: add DenialResponse schema.
- Slice 13: add version/deprecation metadata.
- Slice 14: add transport profile extension.
- Slice 15: add DealSet extension.
- Slice 16: add pack overlay extension.
- Slice 17: add PHI-safe examples.
- Slice 18: add local-openapi projection parity.
- Slice 19: add SDK generation expectations.
- Slice 20: add contract tests.

## Failure Modes
- Missing tenant header: 400 with PHI-safe validation denial.
- Tenant header mismatches token: 403 with Cedar/identity denial evidence.
- Missing idempotency key on command: 400 and no workflow start.
- Unsupported action id: 404 or 410 when sunset.
- Deprecated field used after sunset: 422 with migration guidance.
- Source-system ref absent for adapter action: 400.
- Credential lease unavailable: 503 with retry_after only when safe.
- Cedar deny: 403 with PHI-safe denial code.
- Residency blocks export: 409 with pack decision ref.
- Audit-chain unavailable: 503 for mutation and read-only status remains available.
- Workflow backlog: 202 accepted only when queue admission is safe.
- Replay conflict: 409 with rollback bundle ref.
- Break-glass missing justification: 400.
- Patient match review missing reviewer: 409 review_required.
- Evidence export attempts raw PHI beyond permit: 403.

## Tests and Evidence
- Test OpenAPI validates every required header.
- Test ActionRequest requires tenant_id, principal_id, purpose, data_class, and deal_set_id where applicable.
- Test accepted action returns workflow_run_id and audit_event_class.
- Test denied action returns PHI-safe denial response.
- Test status endpoint returns no raw PHI.
- Test transform preview does not mutate ontology.
- Test review approval requires reviewer_ref.
- Test break-glass revoke expires temporary access.
- Test evidence export respects residency_label.
- Test retry requires original idempotency key.
- Test cancel requires rollback bundle availability.
- Test provider directory search cannot grant PHI access.
- Test local-openapi-v1.yaml does not weaken production required fields.
- Test proto contracts do not add privileged REST-bypassing fields.
- Test API deprecation metadata follows ADR-0258.
- Test transport profile records ADR-0253-amendment posture.
- Test DealSet fields appear in commercial action examples.
- Test Redox-like API client cannot omit Cedar context.
- Test Mirth-like webhook cannot bypass idempotency.
- Test Health Catalyst-like export cannot bypass evidence policy.

## Acceptance Criteria
- AC-001: REST v1 exposes action, status, preview, review, evidence, rollback, and provider directory surfaces.
- AC-002: Every command-shaped endpoint requires tenant, principal, purpose, data class, and idempotency.
- AC-003: Every accepted async command returns workflow_run_id.
- AC-004: Every accepted mutation returns audit_event_class or audit evidence reference.
- AC-005: Denial responses are PHI-safe.
- AC-006: REST cannot grant provider credentials directly.
- AC-007: REST cannot bypass Cedar, ontology, workflow, or audit.
- AC-008: REST versioning follows ADR-0258.
- AC-009: Transport posture follows ADR-0253-amendment.
- AC-010: DealSet fields are visible where settlement applies.
- AC-011: Local contracts do not weaken production contracts.
- AC-012: SDK generation preserves safety fields.
- AC-013: Provider-specific APIs remain behind source_system_ref.
- AC-014: Evidence export is policy-cleared and residency-aware.
- AC-015: ADR-0321 industry-leader depth includes explicit benchmark displacement.
- AC-016: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst are explicitly displaced.
- AC-017: The IP remains implementable without editing ADR-0321.
- AC-018: The IP cites repo-local contract, SLO, and SDK references.
- AC-019: REST is a controlled surface, not canonical healthcare state.
- AC-020: REST examples cover fhir-read, hl7-route, consent-sync, break-glass, patient-match-review, and provenance export request-response evidence.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-005-rest-contract-surface.md:9` - Primary contract: microservices/healthcare-integration/contracts/openapi-v1.yaml; `microservices/healthcare-integration/IP-005-rest-contract-surface.md:16` - Repo references: microservices/healthcare-integration/contracts/openapi-v1.yaml.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-005-rest-contract-surface.md:47` - - listHealthcareIntegrationEvidence returns PHI-safe evidence summaries.; `microservices/healthcare-integration/IP-005-rest-contract-surface.md:104` - - denial_message is PHI-safe..
