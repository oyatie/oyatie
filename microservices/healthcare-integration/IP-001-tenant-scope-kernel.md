# IP-001 Healthcare Integration Tenant Scope Kernel

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-001-tenant-scope-kernel.md
Batch: Batch C healthcare-integration IP deepening
Status: implementation-plan
Owner: axis-healthcare-integration + council-product
Primary layer: kernel
Primary bounded contexts: patient-record, fhir-resource, hl7-message, referral, clinical-consent
Primary capabilities: fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, patient-match-review
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md
Repo references: microservices/healthcare-integration/ARCHITECTURE.md
Repo references: microservices/healthcare-integration/manifest.json
Repo references: microservices/healthcare-integration/compliance.md
Repo references: microservices/healthcare-integration/capabilities/fhir-read.yaml
Repo references: microservices/healthcare-integration/capabilities/hl7-route.yaml
Repo references: microservices/healthcare-integration/capabilities/break-glass-authorize.yaml
Repo references: microservices/healthcare-integration/capabilities/consent-sync.yaml
Repo references: microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml
Repo references: microservices/healthcare-integration/capabilities/patient-match-review.yaml
Repo references: microservices/healthcare-integration/policy/clinical-interoperability-authorization.cedar
Repo references: microservices/healthcare-integration/policy/data-residency.md
Repo references: microservices/healthcare-integration/contracts/openapi-v1.yaml
Repo references: microservices/healthcare-integration/contracts/asyncapi-v1.yaml

## Objective
- Build the pure tenant-scope kernel for Healthcare Integration before adapters, transports, or provider-specific connectors.
- Treat tenant identity as the safety primitive, not as metadata added by controllers or source-system adapters.
- Make every clinical object addressable by tenant_id, principal_id, audience_type, home_cell, jurisdiction_code, data_class, purpose, and audit_event_class.
- Keep source-system identifiers as provenance fields; never allow Epic, Cerner, Redox, Rhapsody, or any other vendor identifier to become the Oyatie tenant boundary.
- Use ADR-0244 as the tenant-scoping authority and ADR-0242 as the reminder that Oyatie itself is a tenant-aware platform.
- Use ADR-0243 and ADR-0246 to ensure kernel outputs are policy-engine ready without embedding Cedar runtime mechanics in the kernel.
- Use ADR-0257 to keep ontology object references version-aware and deprecation-safe.
- Use ADR-0258 to keep API-visible identifiers stable across v1 contract evolution.
- Use ADR-0263 to require audit-ready correlation fields before mutation leaves the kernel.
- Use ADR-0314 to bind commercial DealSet settlement references to clinical integration actions without turning marketplace into the service boundary.
- Use ADR-0321 to meet B2B leader depth rather than merely naming Healthcare as a vertical.

## Tenant Scope Model
- TenantScope is a value object, not a loose request map.
- TenantScope requires tenant_id.
- TenantScope requires principal_id.
- TenantScope requires audience_type.
- TenantScope requires home_cell.
- TenantScope requires jurisdiction_code.
- TenantScope requires data_class.
- TenantScope requires purpose.
- TenantScope requires audit_event_class.
- TenantScope requires trace_context.
- TenantScope requires idempotency_key for every command-shaped operation.
- TenantScope carries deal_set_id when a marketplace commercial obligation is involved.
- TenantScope carries workflow_run_id when the action is orchestrated or replayable.
- TenantScope carries ontology_object_ref when projection already exists.
- TenantScope carries source_system_ref only as provenance.
- TenantScope carries provider_credential_ref only as an OpenBao reference.
- TenantScope does not carry raw secrets.
- TenantScope does not carry direct database table names.
- TenantScope does not permit absent tenant_id in emergency flows.
- TenantScope permits emergency audience only with explicit emergency_attestation.
- TenantScope treats break-glass as a scoped override, not as a tenant bypass.
- TenantScope rejects cross-tenant patient matching unless a referral or consent graph edge authorizes the relationship.
- TenantScope records tenant role, clinical role, and system role separately.
- TenantScope separates platform support operator authority from tenant clinical operator authority.
- TenantScope separates automated worker authority from human clinician authority.
- TenantScope keeps patient identity attributes out of metrics labels.
- TenantScope keeps raw PHI out of refusal messages.

## Kernel Types
- TenantId: canonical tenant slug or UUID owned by the tenant substrate.
- PrincipalId: caller identity after identity service authentication.
- AudienceType: HEALTHCARE_OPERATOR, AUDITOR, SUPPORT_OPERATOR, AUTOMATED_WORKER, EMERGENCY_SERVICES, or TENANT_ADMIN.
- HomeCell: tenant home cell selected by manifest cell eligibility.
- JurisdictionCode: pack resolver input for HIPAA, GDPR, KR-Medical-Devices, EU-MDR, and GxP.
- DataClass: fhir_resource, hl7_message, break_glass_event, clinical_consent, referral_record, provider_directory_record.
- Purpose: treatment, payment, operations, consent_reconciliation, referral_coordination, provenance_export, audit_review, incident_response.
- AuditEventClass: event class expected by audit-chain and ADR-0263.
- DealSetId: settlement reference required by ADR-0314 when integration is commercially mediated.
- WorkflowRunId: workflow-engine correlation for long-running import, route, replay, consent sync, or review flows.
- OntologyObjectRef: ontology projection pointer with type, version, and tenant partition.
- SourceSystemRef: vendor or facility identifier retained for provenance and reconciliation only.
- CredentialLeaseRef: OpenBao sidecar lease reference with TTL metadata, never the credential value.
- ResidencyLabel: computed from jurisdiction, pack, data class, and tenant home cell.
- ScopeDecision: accepted, rejected_missing_field, rejected_cross_tenant, rejected_residency, rejected_policy_prereq, degraded_most_restrictive.
- ScopeEvidence: signed digest, rejected field list, decision reason, and audit-chain target.

## Invariants
- A command without tenant_id is invalid before Cedar evaluation.
- A command without principal_id is invalid before Cedar evaluation.
- A command without audience_type is invalid before Cedar evaluation.
- A command without data_class is invalid before ontology projection.
- A command without purpose is invalid before provider adapter selection.
- A command without audit_event_class is invalid before async event publication.
- A command with source_system_ref but no tenant_id is rejected.
- A command with deal_set_id but no tenant_id is rejected.
- A command with workflow_run_id but no tenant_id is rejected.
- A command with emergency audience but no emergency_attestation is rejected.
- A command with support operator audience and PHI export purpose is rejected unless tenant policy grants it.
- A command with auditor audience can read evidence but cannot mutate clinical state.
- A command with automated worker audience must include workflow_run_id and idempotency_key.
- Patient identifiers never act as global primary keys.
- FHIR resource ids are nested under tenant and source-system provenance.
- HL7 message control ids are nested under tenant, facility, and source interface.
- Consent records bind tenant, patient, purpose, policy pack, and source provenance.
- Referral records bind both sending and receiving tenants when cross-tenant care coordination is allowed.
- Break-glass records bind tenant, emergency attestation, justification, reviewer, and expiry.
- Every accepted scope emits evidence suitable for audit-chain.
- Every rejected scope emits denial evidence without PHI.

## Competitor Displacement
- Redox displacement: Redox brokers network connectivity; this kernel owns tenant-scoped evidence before any network adapter is selected.
- Rhapsody displacement: Rhapsody routes interface messages; this kernel makes the route illegal unless tenant, purpose, pack, and audit fields are already complete.
- InterSystems IRIS for Health displacement: IRIS can centralize clinical data; this kernel prevents centralization from flattening tenant home-cell and pack residency.
- Lyniate/Corepoint displacement: Corepoint mapping strength is adapter-side; this kernel keeps mapping outputs subordinate to Oyatie tenant and ontology references.
- Mirth Connect displacement: Mirth channels often express transformation logic; this kernel refuses channel success as sufficient without audit, policy, and DealSet correlation.
- NextGate displacement: NextGate identity matching is useful but not authoritative; this kernel treats match confidence as evidence under tenant scope, not as cross-tenant identity collapse.
- Health Catalyst displacement: Health Catalyst analytics can aggregate healthcare facts; this kernel requires analytics extracts to start from scoped, policy-cleared, evidence-bearing records.
- Epic displacement: Epic FHIR app access is source-specific; this kernel normalizes Epic ids into tenant provenance rather than product boundary.
- Cerner displacement: Cerner Millennium integrations remain source adapters; this kernel keeps Millennium encounter ids inside tenant-scoped provenance.
- Allscripts displacement: Allscripts feed variance is handled as source-system drift, not as a separate service boundary.
- Veeva displacement: Veeva life-sciences workflows inform GxP and consent evidence, but tenant-scope remains common across all healthcare contexts.

## Implementation Slices
- Slice 1: define TenantScope value object in the kernel layer named by ADR-0105.
- Slice 2: define ScopeDecision and ScopeEvidence as pure domain-neutral outputs.
- Slice 3: define constructors that return rejection reasons instead of partial scope objects.
- Slice 4: define data_class enumeration aligned to PRD functional requirements.
- Slice 5: define audience_type enumeration aligned to policy/clinical-interoperability-authorization.cedar.
- Slice 6: define purpose enumeration aligned to treatment, payment, operations, consent, referral, and audit flows.
- Slice 7: define home_cell and jurisdiction validators aligned to manifest.json cell eligibility.
- Slice 8: define source_system_ref as opaque provenance with vendor, facility, interface, and source id.
- Slice 9: define ontology_object_ref as opaque type and version pointer.
- Slice 10: define deal_set_id validation as optional by action but mandatory when commercial integration is involved.
- Slice 11: define emergency_attestation validation without bypassing tenant_id.
- Slice 12: define support_operator restrictions at value-object construction time.
- Slice 13: define auditor restrictions at value-object construction time.
- Slice 14: define automated_worker requirements for workflow_run_id and idempotency_key.
- Slice 15: define PHI-safe display strings for refusal evidence.
- Slice 16: define deterministic digest input ordering for audit evidence.
- Slice 17: define residency label calculation as pack-resolver input, not as hard-coded law text.
- Slice 18: define degradation mode when tenant projection is stale.
- Slice 19: define most-restrictive fallback when pack overlays conflict.
- Slice 20: define property-test cases for missing, malformed, cross-tenant, emergency, and replay contexts.

## Integration Boundaries
- Identity service authenticates principal_id; this kernel validates presence and role compatibility.
- Compliance service owns pack interpretation; this kernel names pack inputs and receives residency labels.
- Consent graph owns consent relationships; this kernel requires consent references when purpose depends on consent.
- Workflow engine owns orchestration; this kernel requires workflow_run_id for async and replayable operations.
- Ontology owns object storage and projection; this kernel validates versioned references.
- Audit-chain owns signed evidence persistence; this kernel constructs evidence input.
- Marketplace owns DealSet settlement; this kernel requires deal_set_id where commercial obligations apply.
- Provider adapters own source protocols; this kernel accepts only opaque source_system_ref.

## Failure Modes
- Missing tenant_id: reject before policy and emit ScopeEvidence with rejected_missing_field.
- Missing principal_id: reject before policy and emit ScopeEvidence with rejected_missing_field.
- Missing data_class: reject before ontology projection and emit ScopeEvidence with rejected_missing_field.
- Cross-tenant patient match: reject unless consent/referral relationship is provided.
- Emergency attestation absent: reject even when audience_type is EMERGENCY_SERVICES.
- Support operator attempts PHI export: reject unless policy reference explicitly permits.
- Auditor attempts mutation: reject and emit refusal evidence.
- Automated worker lacks idempotency key: reject to prevent replay ambiguity.
- Source system claims tenant through adapter payload: ignore source tenant claim and use authenticated TenantScope only.
- Residency label cannot be computed: degrade to most restrictive and block export.
- Pack conflict: choose higher restriction and require compliance remediation.
- Stale tenant projection: allow read-only evidence review, block high-risk mutation.
- Credential lease absent: block provider adapter selection.
- Audit-chain target absent: block mutation and async publication.
- DealSet absent for billable connector route: block action acceptance.

## Tests and Evidence
- Test tenant scope construction accepts the complete fhir-read context in capabilities/fhir-read.yaml.
- Test tenant scope construction accepts the complete hl7-route context in capabilities/hl7-route.yaml.
- Test tenant scope construction accepts break-glass only with emergency_attestation.
- Test consent-sync rejects absent clinical_consent data class.
- Test ehr-provenance-seal requires source_system_ref and ontology_object_ref.
- Test patient-match-review rejects cross-tenant records without referral or consent relationship.
- Test support operator cannot mutate clinical state by default.
- Test auditor can read evidence and cannot invoke import/export mutations.
- Test automated worker requires workflow_run_id and idempotency_key.
- Test all rejections omit raw PHI in display messages.
- Test digest ordering is deterministic for audit evidence.
- Test pack conflict falls back to higher restriction.
- Test source-system ids never become TenantId.
- Test emergency services path still requires tenant_id.
- Test DealSet path requires deal_set_id when commercial obligation is present.
- Evidence file expectation: future implementation emits multispectrum evidence under evidence/multispectrum.
- Evidence event expectation: accepted scope prepares audit_event_class for audit-chain.
- Evidence contract expectation: openapi-v1.yaml request fields stay compatible with TenantScope.
- Evidence event expectation: asyncapi-v1.yaml required fields stay compatible with TenantScope.
- Evidence policy expectation: clinical-interoperability-authorization.cedar receives complete context.

## Acceptance Criteria
- AC-001: The kernel has no dependency on provider SDKs, interface engines, or source database schemas.
- AC-002: The kernel refuses to construct partial tenant scope.
- AC-003: Every accepted scope can produce audit evidence input.
- AC-004: Every rejected scope produces PHI-safe denial evidence.
- AC-005: Emergency flows cannot bypass tenant scope.
- AC-006: Cross-tenant flows require explicit consent/referral relationship evidence.
- AC-007: Source-system identifiers remain provenance, not tenancy authority.
- AC-008: Marketplace DealSet settlement remains a reference, not a service boundary.
- AC-009: ADR-0105 layer naming is preserved.
- AC-010: ADR-0244 tenant-as-universal-scoping primitive is satisfied.
- AC-011: ADR-0243 Cedar gate receives complete context.
- AC-012: ADR-0257 ontology references are versioned.
- AC-013: ADR-0258 API-facing ids are stable.
- AC-014: ADR-0263 audit emission prerequisites are present.
- AC-015: ADR-0314 DealSet obligations are scoped.
- AC-016: ADR-0321 B2B leader depth names explicit benchmark displacement.
- AC-017: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst are explicitly displaced.
- AC-018: The IP remains implementable without editing ADR-0321.
- AC-019: The IP cites only repo-local files and existing ADR ids.
- AC-020: TenantScope fixtures cover fhir-read, hl7-route, consent-sync, patient-match-review, and break-glass contexts with PHI-safe refusal evidence.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-001-tenant-scope-kernel.md:24` - Repo references: microservices/healthcare-integration/contracts/openapi-v1.yaml; `microservices/healthcare-integration/IP-001-tenant-scope-kernel.md:25` - Repo references: microservices/healthcare-integration/contracts/asyncapi-v1.yaml.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-001-tenant-scope-kernel.md:67` - - TenantScope keeps raw PHI out of refusal messages.; `microservices/healthcare-integration/IP-001-tenant-scope-kernel.md:76` - - Purpose: treatment, payment, operations, consent_reconciliation, referral_coordination, provenance_export, audit_review, incident_response..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-001-tenant-scope-kernel.md:208` - - AC-014: ADR-0263 audit emission prerequisites are present..
