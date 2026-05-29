# IP-003 Healthcare Integration Ontology Projection

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-003-ontology-projection.md
Batch: Batch C healthcare-integration IP deepening
Status: implementation-plan
Owner: axis-healthcare-integration + axis-ontology
Primary layer: application + usecase + domain
Primary projection mode: library-first ontology read/write path
Primary bounded contexts: patient-record, fhir-resource, hl7-message, referral, clinical-consent
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md
Repo references: microservices/healthcare-integration/ARCHITECTURE.md
Repo references: microservices/healthcare-integration/manifest.json
Repo references: microservices/healthcare-integration/capabilities/fhir-read.yaml
Repo references: microservices/healthcare-integration/capabilities/hl7-route.yaml
Repo references: microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml
Repo references: microservices/healthcare-integration/contracts/openapi-v1.yaml
Repo references: microservices/healthcare-integration/contracts/asyncapi-v1.yaml
Repo references: docs/decisions/ADR-0257-amendment-library-first-ontology-read-path.md
Repo references: docs/decisions/ADR-0257-ontology-object-type-versioning-deprecation-handshake.md
Repo references: specs/microservices/ontology.json
Repo references: registry/knowledge-graph-kinetic.json
Repo references: registry/knowledge-graph-dynamic.json

## Objective
- Define how Healthcare Integration projects clinical interoperability facts into the shared ontology without creating a healthcare data silo.
- Keep source-system payloads outside ontology authority until tenant, Cedar, provenance, and pack checks pass.
- Represent FHIR, HL7, consent, referral, provenance, provider directory, and patient-match review as typed ontology objects or edges.
- Preserve ADR-0257 versioning and deprecation handshake for every projected object type.
- Preserve ADR-0244 tenant scope on every object and edge.
- Preserve ADR-0243 Cedar decision ids on every mutation path.
- Preserve ADR-0263 audit emission prerequisites for every projection change.
- Preserve ADR-0314 DealSet references for commercially mediated connector operations.
- Preserve ADR-0321 industry-leader depth by displacing vendor hub models with tenant-governed projection.

## Projection Principles
- Source payload is evidence, not canonical ontology state.
- Tenant scope is part of the projection key.
- Source system id is part of provenance, not object identity.
- FHIR logical id is nested under tenant, source_system_ref, resource_type, and version.
- HL7 message control id is nested under tenant, facility, sending application, receiving application, and message timestamp.
- Consent id is nested under tenant, patient scope, purpose, source, status, and pack.
- Referral id is nested under sending tenant, receiving tenant when permitted, patient scope, care context, and status.
- Patient match candidate id is nested under tenant and review workflow, not global patient identity.
- Provider directory id is network-scoped only when policy permits network-level lookup.
- Break-glass event id is nested under tenant, patient scope, attestation, justification, expiry, and reviewer.
- Projection stores normalized object refs and provenance digests, not raw source payloads.
- Raw payload retention follows pack policy and storage tier decisions outside this IP.
- Projection failure cannot be silently retried without audit evidence.
- Projection replay uses idempotency key, source digest, and prior ontology object version.
- Projection deprecation uses ADR-0257 handshake before object type removal or semantic narrowing.
- Projection read path is library-first, not direct database read.
- Projection write path records Cedar decision id and audit event class.
- Projection read model excludes raw PHI from metrics and dashboards.
- Projection exposes conflict evidence when FHIR and HL7 sources disagree.
- Projection exposes provenance chain when a clinical record is amended, corrected, or sealed.

## Object Types
- HealthcarePatientRecord: tenant-scoped clinical document summary with provenance links.
- HealthcareFhirResource: normalized FHIR resource pointer with resource_type and version.
- HealthcareHl7Message: normalized HL7 message envelope with route and ack status.
- HealthcareClinicalConsent: consent grant, denial, expiry, source, purpose, and pack overlay.
- HealthcareReferral: care coordination object with relationship authorization.
- HealthcareBreakGlassEvent: emergency or break-glass access record with expiry and review status.
- HealthcareEhrProvenanceSeal: digest and source chain for imported or exported clinical records.
- HealthcarePatientMatchReview: human-reviewable candidate match bundle.
- HealthcareProviderDirectoryEntry: scoped provider/facility/network metadata.
- HealthcareAuditProjection: non-PHI projection of policy, event, and evidence references.
- HealthcareReplayBatch: import/replay unit with source digest set and idempotency scope.
- HealthcareRouteDecision: HL7/FHIR route decision with Cedar and pack context.
- HealthcareDataResidencyConstraint: computed residency label attached to projected object.
- HealthcareDealSetObligation: commercial settlement pointer attached to mediated route.
- HealthcarePackOverlay: pack-specific retention, export, residency, and approval overlay.

## Edge Types
- patient-record HAS_FHIR_RESOURCE fhir-resource.
- patient-record DERIVED_FROM_HL7_MESSAGE hl7-message.
- fhir-resource SEALED_BY ehr-provenance-seal.
- hl7-message ROUTED_BY route-decision.
- clinical-consent AUTHORIZES_PURPOSE patient-record.
- clinical-consent LIMITS_EXPORT fhir-resource.
- referral CONNECTS_TENANT sending-tenant to receiving-tenant.
- patient-match-review CANDIDATE_FOR patient-record.
- break-glass-event ELEVATES_ACCESS_TO patient-record.
- provider-directory-entry PARTICIPATES_IN referral.
- replay-batch REPLAYS source-system-ref.
- route-decision SETTLED_BY deal-set-obligation.
- pack-overlay CONSTRAINS data-residency-constraint.
- audit-projection EVIDENCES ontology-object-ref.
- tenant OWNS every healthcare ontology object.

## Projection Fields
- tenant_id is required on every object.
- principal_id is required on every write.
- audience_type is required on every write.
- home_cell is required on every object.
- jurisdiction_code is required on every object.
- data_class is required on every object.
- purpose is required on every write.
- audit_event_class is required on every write.
- cedar_decision_id is required on every write.
- source_system_ref is required for imported facts.
- source_digest is required for imported facts.
- ontology_object_type is required on every object.
- ontology_object_version is required on every object.
- projection_version is required on every object.
- deprecation_status is required on every object type.
- workflow_run_id is required for async projection.
- idempotency_key is required for replayable writes.
- deal_set_id is required for commercially mediated actions.
- residency_label is required before export.
- pack_ids are required for compliance-sensitive facts.
- phi_redaction_profile is required for evidence and dashboards.

## Competitor Displacement
- Redox displacement: Redox can normalize APIs, but Oyatie projection owns tenant-scoped ontology semantics and audit evidence.
- Rhapsody displacement: Rhapsody maps and routes interfaces, but Oyatie projection models durable clinical objects and replayable provenance.
- InterSystems IRIS for Health displacement: IRIS stores and queries health data; Oyatie keeps healthcare projection under platform-wide ontology versioning and tenant cells.
- Lyniate/Corepoint displacement: Corepoint transformations feed source evidence, while Oyatie ontology controls canonical object and edge shape.
- Mirth displacement: Mirth channel outputs are treated as source events, not canonical semantic state.
- NextGate displacement: NextGate identity resolution becomes PatientMatchReview evidence; it cannot merge ontology persons across tenants by itself.
- Health Catalyst displacement: Health Catalyst analytics depends on modeled facts; Oyatie projection records governed, pack-aware source-to-ontology lineage first.
- Epic displacement: Epic FHIR resources are projected as HealthcareFhirResource with Epic as provenance.
- Cerner displacement: Cerner encounter facts are projected with source_system_ref and cannot override tenant object identity.
- Allscripts displacement: Allscripts feed variants map through route decisions and conflict evidence.
- Veeva displacement: Veeva regulated records inform GxP overlays and provenance seals without creating a separate life-sciences ontology silo.

## Implementation Slices
- Slice 1: define object type registry entries for the Healthcare object types listed above.
- Slice 2: define edge type registry entries for the Healthcare edge types listed above.
- Slice 3: define FHIR-to-ontology mapper contracts for patient, encounter, observation, condition, medication, document reference, and provenance resources.
- Slice 4: define HL7-to-ontology mapper contracts for ADT, ORU, ORM, MDM, SIU, and ACK envelopes.
- Slice 5: define consent-to-ontology mapper for grant, deny, revoke, expire, and conflict states.
- Slice 6: define referral-to-ontology mapper for sending tenant, receiving tenant, provider, patient scope, and status.
- Slice 7: define break-glass-to-ontology mapper for attestation, justification, expiry, reviewer, and posthoc review.
- Slice 8: define provenance seal mapper for source digest, import digest, export digest, and replay digest.
- Slice 9: define patient match review mapper for candidate set, confidence, reviewer, decision, and non-merge result.
- Slice 10: define data residency constraint mapper from pack resolver output.
- Slice 11: define DealSet obligation mapper for mediated connector actions.
- Slice 12: define projection conflict model when FHIR, HL7, and manual correction disagree.
- Slice 13: define projection replay semantics by source digest and idempotency key.
- Slice 14: define projection version increment rules.
- Slice 15: define object type deprecation handshake per ADR-0257.
- Slice 16: define PHI-safe projection summaries for audit and dashboards.
- Slice 17: define projection test fixtures for each capability record.
- Slice 18: define projection fixture parity for Redox-like normalized API input.
- Slice 19: define projection fixture parity for Rhapsody/Corepoint/Mirth-like HL7 channel output.
- Slice 20: define projection fixture parity for NextGate-like patient match evidence.

## Failure Modes
- Source payload lacks tenant scope: reject before projection.
- Source payload tenant conflicts with authenticated tenant: reject and emit conflict evidence.
- FHIR id collides across source systems: preserve both source refs and require review before merge.
- HL7 duplicate message control id appears: use tenant, facility, interface, timestamp, and digest to disambiguate.
- Consent status conflicts: apply most restrictive status and open consent remediation workflow.
- Referral crosses tenants without relationship evidence: reject edge creation.
- Break-glass projection lacks expiry: reject.
- Patient match confidence is high but reviewer absent: create review object, do not merge records.
- Provenance digest mismatch: quarantine projection and open replay remediation.
- Ontology object version is deprecated: use ADR-0257 migration handshake.
- Pack overlay conflicts with export: block export edge and emit residency evidence.
- DealSet id mismatches tenant: reject commercial edge.
- Audit-chain emission unavailable: pause projection writes.
- Workflow replay creates divergent projection: retain prior projection and emit divergence evidence.
- Provider directory entry claims network access: require policy before network edge creation.

## Tests and Evidence
- Test FHIR Patient projects into HealthcarePatientRecord under tenant scope.
- Test FHIR Observation projects into HealthcareFhirResource with resource_type.
- Test HL7 ADT projects into HealthcareHl7Message and derived patient-record edge.
- Test HL7 ACK updates route decision without mutating clinical facts.
- Test consent grant authorizes purpose edge.
- Test consent revoke blocks export edge.
- Test referral creates cross-tenant edge only with relationship evidence.
- Test break-glass event creates expiry-bound edge.
- Test provenance seal includes source digest and audit event class.
- Test patient match review does not merge cross-tenant identity.
- Test provider directory entry cannot grant PHI access by itself.
- Test replay by idempotency key is stable.
- Test deprecated object version requires migration path.
- Test projection conflict emits PHI-safe evidence.
- Test pack residency blocks export edge.
- Test DealSet obligation links only to matching tenant route decision.
- Test Redox-like input cannot bypass tenant scope.
- Test Rhapsody-like route output cannot become canonical state alone.
- Test Mirth-like transform output requires Cedar decision id.
- Test NextGate-like match evidence remains review-bound.

## Acceptance Criteria
- AC-001: Projection is library-first and does not read ontology storage directly from adapters.
- AC-002: Every projected object carries tenant_id.
- AC-003: Every projected write carries Cedar decision id.
- AC-004: Every projected write carries audit_event_class.
- AC-005: Every imported fact carries source_system_ref and source_digest.
- AC-006: Every object type carries version and deprecation status.
- AC-007: Cross-tenant edges require explicit relationship evidence.
- AC-008: Patient matching creates review evidence, not automatic identity collapse.
- AC-009: Pack overlays can block export or replication.
- AC-010: DealSet obligations are represented as references.
- AC-011: ADR-0257 versioning and deprecation handshake are satisfied.
- AC-012: ADR-0244 tenant scoping is satisfied.
- AC-013: ADR-0263 audit emission prerequisites are satisfied.
- AC-014: ADR-0314 marketplace settlement linkage is satisfied.
- AC-015: ADR-0321 industry-leader depth includes explicit benchmark displacement.
- AC-016: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst are explicitly displaced.
- AC-017: The IP remains implementable without editing ADR-0321.
- AC-018: The IP cites repo-local ontology and healthcare references.
- AC-019: The IP keeps canonical state out of provider adapters.
- AC-020: Ontology projection fixtures bind FHIR, HL7, consent, referral, and MPI records to versioned tenant-local object references.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-003-ontology-projection.md:18` - Repo references: microservices/healthcare-integration/contracts/openapi-v1.yaml; `microservices/healthcare-integration/IP-003-ontology-projection.md:19` - Repo references: microservices/healthcare-integration/contracts/asyncapi-v1.yaml.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-003-ontology-projection.md:55` - - Projection read model excludes raw PHI from metrics and dashboards.; `microservices/healthcare-integration/IP-003-ontology-projection.md:69` - - HealthcareAuditProjection: non-PHI projection of policy, event, and evidence references..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-003-ontology-projection.md:33` - - Preserve ADR-0263 audit emission prerequisites for every projection change.; `microservices/healthcare-integration/IP-003-ontology-projection.md:164` - - Audit-chain emission unavailable: pause projection writes..
