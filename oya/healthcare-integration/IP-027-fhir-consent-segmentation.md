# IP-027 Healthcare Integration FHIR consent segmentation

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-027-fhir-consent-segmentation.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Status: authoring-ready
Owner: axis-healthcare-integration
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/capabilities/fhir-read.yaml; microservices/healthcare-integration/capabilities/consent-sync.yaml; microservices/healthcare-integration/policies/local-fhir-exchange-consent.cedar; microservices/healthcare-integration/policies/local-patient-consent-sync.cedar; microservices/healthcare-integration/runbooks/local-consent-sync-lag.md; microservices/healthcare-integration/runbooks/local-fhir-bundle-failure.md; microservices/healthcare-integration/slos/local-consent-sync-freshness.openslo.yaml; microservices/healthcare-integration/slos/local-fhir-bundle-success.openslo.yaml
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Define an atomic implementation plan for FHIR resource segmentation by consent state.
- Bind FHIR read, bundle, export, and replay paths to consent-sync evidence.
- Prevent resource delivery when consent is missing, stale, revoked, region-incompatible, or purpose-incompatible.
- Keep consent logic Cedar-gated and tenant-scoped.
- Preserve FHIR resource provenance and ontology versioning under ADR-0257.
- Keep contract versioning and deprecation under ADR-0258.
- Keep this IP documentation-only and limited to the assigned file.

## Segmentation model
- Segment field 001: tenant_id identifies the tenant that owns the FHIR exchange.
- Segment field 002: principal_id identifies the actor or automated worker requesting access.
- Segment field 003: audience_type identifies healthcare operator, auditor, patient delegate, or permitted worker.
- Segment field 004: purpose binds the request to treatment, operations, payment, patient-access, public-health, or pack-specific purpose.
- Segment field 005: data_class identifies fhir_resource or clinical_consent.
- Segment field 006: patient_scope_id links the request to the tenant MPI projection.
- Segment field 007: consent_record_id links to the consent-sync record.
- Segment field 008: consent_version prevents stale decisions.
- Segment field 009: consent_status captures active, revoked, limited, expired, disputed, or emergency.
- Segment field 010: consent_scope captures resource types, compartments, and encounter bounds.
- Segment field 011: jurisdiction_code captures residency and regulator overlays.
- Segment field 012: home_cell captures cell-local enforcement.
- Segment field 013: source_system_id captures the EHR or registry endpoint.
- Segment field 014: fhir_resource_type captures Patient, Observation, Encounter, Medication, DocumentReference, or related type.
- Segment field 015: fhir_profile_version captures implementation guide compatibility.
- Segment field 016: ontology_object_type_version captures ADR-0257 projection version.
- Segment field 017: pack_overlay captures HIPAA, GDPR, KR-Medical-Devices, EU-MDR, GxP, or other active pack.
- Segment field 018: audit_event_class captures ADR-0263 event family.
- Segment field 019: workflow_run_id captures consent adjudication or sync workflow.
- Segment field 020: dealset_reference captures marketplace settlement when a vendor exchange is billable.

## Consent state transitions
- Transition 001: unknown to pending-sync when the source consent record has not been imported.
- Transition 002: pending-sync to active when consent-sync validates source, tenant, purpose, and scope.
- Transition 003: pending-sync to denied when source consent is malformed or missing.
- Transition 004: active to limited when patient or policy narrows resource type, purpose, or time range.
- Transition 005: active to revoked when revocation arrives from source or tenant workflow.
- Transition 006: active to expired when validity window closes.
- Transition 007: limited to active only through a new consent_version and reviewer evidence.
- Transition 008: revoked to active only through a new consent_record_id or explicit reinstatement evidence.
- Transition 009: disputed to active only after adjudication workflow completes.
- Transition 010: active to emergency when IP-028 break-glass evidence permits temporary access.
- Transition 011: emergency to review-required when emergency timer expires.
- Transition 012: review-required to active when reviewer confirms justification and patient notification rules.
- Transition 013: review-required to revoked when reviewer rejects justification.
- Transition 014: any state to quarantined when consent source integrity fails.
- Transition 015: quarantined to active only through replay and signed source revalidation.
- Transition 016: any state to superseded when a newer consent_version replaces it.
- Transition 017: superseded states remain retained for audit and replay.
- Transition 018: expired states remain queryable to auditors but not usable for new access.
- Transition 019: denied states emit refusal evidence and safe user explanation.
- Transition 020: every transition emits ADR-0263 audit-chain evidence.

## FHIR access gates
- Gate 001: deny when tenant_id is absent.
- Gate 002: deny when principal_id is absent.
- Gate 003: deny when audience_type is not permitted for the resource type.
- Gate 004: deny when purpose is not included in consent_scope.
- Gate 005: deny when data_class is not fhir_resource or clinical_consent.
- Gate 006: deny when consent_status is revoked.
- Gate 007: deny when consent_status is expired.
- Gate 008: deny when consent_version is stale relative to source_system_id.
- Gate 009: deny when jurisdiction_code conflicts with pack overlay.
- Gate 010: deny when home_cell would require forbidden cross-region PHI movement.
- Gate 011: deny when FHIR compartment membership conflicts with patient_scope_id.
- Gate 012: deny when resource type is outside consent_scope.
- Gate 013: deny when break-glass claim lacks IP-028 justification.
- Gate 014: deny when source credential sidecar cannot provide short-lived access.
- Gate 015: deny when audit-chain evidence cannot be written.
- Gate 016: deny when ADR-0294 soak requirements are unmet for the active Cedar fragment.
- Gate 017: deny when ADR-0257 object-type version is deprecated beyond sunset.
- Gate 018: deny when ADR-0258 contract version has passed sunset.
- Gate 019: deny when DealSet hold requires blocked vendor release.
- Gate 020: emit refusal evidence for every deny.

## Bundle segmentation rules
- Bundle rule 001: patient compartment entries require active consent.
- Bundle rule 002: encounter compartment entries require encounter-bounded consent or approved purpose.
- Bundle rule 003: observation entries require category and code filtering.
- Bundle rule 004: medication entries require purpose-specific consent where pack requires it.
- Bundle rule 005: document references require redaction state before export.
- Bundle rule 006: provenance resources are included when export requires evidence.
- Bundle rule 007: audit event references are included in regulator export manifests.
- Bundle rule 008: revoked resources are excluded from ordinary bundles.
- Bundle rule 009: expired resources are excluded from new treatment bundles unless policy permits historical read.
- Bundle rule 010: emergency resources are marked review-required.
- Bundle rule 011: disputed consent blocks export until adjudication.
- Bundle rule 012: limited consent restricts resource type, date range, purpose, and recipient.
- Bundle rule 013: source-system tags are retained but not treated as authority.
- Bundle rule 014: ontology projection version is written per resource.
- Bundle rule 015: deprecation status is written per profile version.
- Bundle rule 016: redaction is deterministic and replayable.
- Bundle rule 017: redaction evidence includes rule id and consent_version.
- Bundle rule 018: export manifest includes resource count by type, not patient identifiers.
- Bundle rule 019: export hash includes canonicalized bundle and redaction manifest.
- Bundle rule 020: replay preserves original denied-resource explanation.

## FHIR Consent Segmentation Benchmark Displacement
- Displacement claim: this IP measures competitors against per-resource consent segmentation, not generic FHIR connectivity or channel transformation.
- Non-generic rule: a vendor comparison must name consent_version, resource type, purpose, redaction manifest, projection version, and denial explanation before it counts as evidence.
- Redox displacement: Redox-style FHIR connectivity is displaced by consent_version-gated resource segmentation.
- Redox proof: fhir-read and consent-sync capabilities are both required before resource delivery.
- Rhapsody displacement: Rhapsody-style route transformation is displaced by policy-first bundle segmentation.
- Rhapsody proof: Cedar decision id and consent scope drive route output.
- InterSystems IRIS for Health displacement: IRIS-style shared clinical data platform is displaced by flat service ownership and explicit object-type versions.
- InterSystems proof: ADR-0257 and ADR-0258 references define version handling.
- Lyniate/Corepoint displacement: channel operations are displaced by consent workflow, audit evidence, and runbook-controlled sync.
- Lyniate/Corepoint proof: local-consent-sync-lag.md owns remediation.
- Mirth displacement: script-level filters are displaced by declarative Cedar gates and deterministic redaction manifests.
- Mirth proof: every bundle rule is testable without channel script authority.
- NextGate displacement: MPI identity confidence does not grant consent; identity and consent are separate gates.
- NextGate proof: patient_scope_id only selects candidate data after consent is active.
- Health Catalyst displacement: analytic cohort permissions are displaced by per-resource purpose, consent, and provenance evidence.
- Health Catalyst proof: aggregate analytics cannot override patient-level consent state.

## Failure modes
- Failure 001: missing consent emits denial and no FHIR resource payload.
- Failure 002: stale consent emits denial and sync lag runbook trigger.
- Failure 003: revoked consent emits denial and patient-notification evidence where pack requires it.
- Failure 004: disputed consent emits denial and adjudication workflow.
- Failure 005: emergency consent emits temporary access plus IP-028 review requirement.
- Failure 006: source consent feed outage uses last valid state only when pack permits and emits degraded evidence.
- Failure 007: ontology version deprecation blocks projection after sunset.
- Failure 008: profile mismatch emits bundle failure and remediation.
- Failure 009: cross-cell residency conflict blocks export.
- Failure 010: redaction manifest mismatch blocks delivery.
- Failure 011: audit-chain outage blocks mutation and export.
- Failure 012: credential sidecar outage blocks source fetch without credential leakage.
- Failure 013: contract version sunset blocks deprecated client access.
- Failure 014: DealSet hold blocks vendor delivery.
- Failure 015: reviewer override without signed decision is denied.

## Capacity and performance
- Capacity 001: consent freshness follows slos/local-consent-sync-freshness.openslo.yaml.
- Capacity 002: FHIR bundle success follows slos/local-fhir-bundle-success.openslo.yaml.
- Capacity 003: consent cache keys include tenant_id, patient_scope_id, consent_record_id, consent_version, purpose, and pack.
- Capacity 004: consent cache entries expire before source validity windows.
- Capacity 005: bundle segmentation work partitions by tenant, resource type, source_system_id, and home_cell.
- Capacity 006: high-volume Observation bundles use streaming redaction.
- Capacity 007: export manifests are built asynchronously for large bundles.
- Capacity 008: patient identifiers do not appear in metric labels.
- Capacity 009: resource counts use type-level metrics only.
- Capacity 010: replay budget separates sync replay from access replay.
- Capacity 011: tail latency budget separates policy, consent lookup, source fetch, redaction, and audit write.
- Capacity 012: tenant-local consent outage cannot starve other tenants.
- Capacity 013: source-system bursts require backpressure.
- Capacity 014: emergency access bypasses friction but not audit.
- Capacity 015: regulator exports use idempotent job ids.

## Observability
- Event `oya.healthcare.integration.fhir.consent.evaluated` records permit or deny.
- Event `oya.healthcare.integration.fhir.consent.revoked` records revocation.
- Event `oya.healthcare.integration.fhir.bundle.segmented` records bundle segmentation.
- Event `oya.healthcare.integration.fhir.bundle.rejected` records safe rejection.
- Event `oya.healthcare.integration.fhir.export.redacted` records deterministic redaction.
- Metric `healthcare_integration_consent_freshness_seconds` dimensions: cell, pack, source_type.
- Metric `healthcare_integration_fhir_bundle_rejection_total` dimensions: reason_code, resource_type, pack.
- Metric `healthcare_integration_fhir_redaction_total` dimensions: rule_id, resource_type, pack.
- Trace span `healthcare.fhir.consent.segment` wraps policy, consent lookup, bundle filter, redaction, and audit.
- Log schema includes consent_record_id, consent_version, resource_type, decision_id, audit_event_id, and workflow_run_id.
- Dashboard reference: dashboards/local-audit-completeness.json.
- Dashboard reference: dashboards/compliance-pack-health.json.
- Runbook reference: runbooks/local-consent-sync-lag.md.
- Runbook reference: runbooks/local-fhir-bundle-failure.md.
- Policy references: local-fhir-exchange-consent.cedar and local-patient-consent-sync.cedar.

## Implementation steps
- Step 001: Add consent segmentation value objects in kernel.
- Step 002: Add consent state aggregate rules in domain.
- Step 003: Add FHIR access usecase that requires consent decision.
- Step 004: Add bundle segmentation worker for async exports.
- Step 005: Add Cedar policy integration through library-first evaluator.
- Step 006: Add consent-sync replay hook.
- Step 007: Add redaction manifest model.
- Step 008: Add audit-chain event emission for each transition.
- Step 009: Add OpenAPI examples for permitted, denied, revoked, and emergency access.
- Step 010: Add AsyncAPI events for consent evaluated, bundle segmented, and export redacted.
- Step 011: Add proto fields for consent_version and redaction manifest id.
- Step 012: Add property tests for consent state transitions.
- Step 013: Add contract tests for profile version sunset.
- Step 014: Add replay tests for deterministic redaction.
- Step 015: Add benchmark displacement evidence to review packet.

## Tests and evidence
- Test 001: line count for this IP is at least 200.
- Test 002: ADR scan finds the full binding ADR list.
- Test 003: benchmark scan finds all seven named competitors.
- Test 004: local reference scan finds fhir-read.yaml and consent-sync.yaml.
- Test 005: local reference scan finds both consent Cedar policies.
- Test 006: local reference scan finds both FHIR and consent SLO files.
- Test 007: local reference scan finds both runbooks.
- Test 008: review confirms consent and MPI are separate gates.
- Test 009: review confirms ADR-0321 was not edited.
- Test 010: review confirms no oya vcs verify, done, or promote was run.

## Rollback
- Rollback 001: disable new consent segment rule for affected tenant only.
- Rollback 002: fall back to stricter existing consent behavior.
- Rollback 003: retain denied access evidence.
- Rollback 004: retain original redaction manifests.
- Rollback 005: replay affected exports idempotently after fix.
- Rollback 006: do not delete consent history.
- Rollback 007: do not rewrite prior audit events.
- Rollback 008: restore prior Cedar fragment only after soak-window rules permit.
- Rollback 009: keep DealSet holds until export evidence is accepted.
- Rollback 010: notify tenant admin through workflow remediation.

## Acceptance criteria
- AC01: Every FHIR access decision carries tenant, principal, audience, purpose, data class, patient scope, consent id, and consent version.
- AC02: Revoked, expired, disputed, stale, and missing consent deny delivery.
- AC03: Emergency access routes through IP-028 review.
- AC04: MPI identity confidence never replaces consent.
- AC05: Redaction is deterministic and replayable.
- AC06: Bundle manifests exclude patient identifiers from metrics.
- AC07: ADR-0257 and ADR-0258 version rules are explicit.
- AC08: DealSet holds are respected for billable vendor exchange.
- AC09: All seven named benchmarks are explicitly displaced.
- AC10: This plan remains scoped to the assigned IP file.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-027-fhir-consent-segmentation.md:26` - - Segment field 004: purpose binds the request to treatment, operations, payment, patient-access, public-health, or pack-specific purpose.; `microservices/healthcare-integration/IP-027-fhir-consent-segmentation.md:76` - - Gate 010: deny when home_cell would require forbidden cross-region PHI movement..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-027-fhir-consent-segmentation.md:187` - - Step 008: Add audit-chain event emission for each transition..
