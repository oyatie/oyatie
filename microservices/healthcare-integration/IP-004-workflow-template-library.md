# IP-004 Healthcare Integration Workflow Template Library

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-004-workflow-template-library.md
Batch: Batch C healthcare-integration IP deepening
Status: implementation-plan
Owner: axis-healthcare-integration + axis-workflow
Primary layer: usecase + worker
Primary dependency: workflow-engine
Primary bounded contexts: patient-record, fhir-resource, hl7-message, referral, clinical-consent
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md
Repo references: microservices/healthcare-integration/ARCHITECTURE.md
Repo references: microservices/healthcare-integration/backfill-replay.md
Repo references: microservices/healthcare-integration/failure-modes.md
Repo references: microservices/healthcare-integration/runbooks/consent-sync-conflict.md
Repo references: microservices/healthcare-integration/runbooks/ehr-provenance-gap.md
Repo references: microservices/healthcare-integration/runbooks/patient-match-duplicate.md
Repo references: microservices/healthcare-integration/runbooks/break-glass-audit-review.md
Repo references: microservices/healthcare-integration/runbooks/hl7-queue-backlog.md
Repo references: microservices/healthcare-integration/contracts/asyncapi-v1.yaml
Repo references: specs/microservices/workflow.json
Repo references: docs/decisions/ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md

## Objective
- Define reusable workflow templates for Healthcare Integration operations that cannot be safely modeled as one-shot REST calls.
- Keep workflow orchestration separate from clinical kernel decisions.
- Require tenant scope, Cedar decision, ontology projection, audit event, and rollback evidence for every template.
- Provide templates for import, route, consent reconciliation, break-glass review, provenance sealing, patient match review, and replay.
- Make workflow_run_id mandatory for async workers and event publication.
- Make templates pack-aware and data-residency-aware.
- Make templates benchmark-displacing rather than vendor-copying.
- Keep Workflow Studio/no-code UX downstream of governed templates, not upstream of safety rules.
- Keep ADR-0321 depth visible through concrete healthcare B2B workflows.

## Template Catalog
- Template HI-FHIR-READ-IMPORT imports FHIR resources from a source system into tenant-scoped projection.
- Template HI-HL7-ROUTE routes HL7 messages with ACK tracking and replay controls.
- Template HI-BREAK-GLASS-REVIEW authorizes, records, expires, and reviews emergency access.
- Template HI-CONSENT-SYNC reconciles consent grants, denials, revocations, and conflicts.
- Template HI-EHR-PROVENANCE-SEAL seals source, import, export, and replay digests.
- Template HI-PATIENT-MATCH-REVIEW coordinates match candidate review without identity collapse.
- Template HI-REFERRAL-HANDOFF coordinates cross-tenant referral when relationship evidence exists.
- Template HI-PROVIDER-DIRECTORY-SYNC refreshes provider/facility/network metadata without PHI access.
- Template HI-BULK-BACKFILL-REPLAY replays historical payloads under idempotency and pack controls.
- Template HI-EXPORT-EVIDENCE-PACKET exports regulator or tenant audit evidence with residency checks.

## Shared Inputs
- tenant_id is required.
- principal_id is required.
- audience_type is required.
- home_cell is required.
- jurisdiction_code is required.
- data_class is required.
- purpose is required.
- audit_event_class is required.
- cedar_decision_id is required.
- workflow_run_id is assigned at start.
- idempotency_key is required for every command step.
- source_system_ref is required for source-system work.
- credential_lease_ref is required for provider access.
- ontology_object_ref is required after projection starts.
- deal_set_id is required for mediated commercial connector actions.
- pack_ids are required.
- residency_label is required before export, replication, or evidence packet generation.
- trace_context is required for every step.
- rollback_bundle_ref is required before destructive or externally visible transition.
- operator_review_ref is required for patient match, break-glass, and consent conflict decisions.

## Shared States
- created: template instance accepted with tenant scope.
- policy_checked: Cedar permit recorded.
- credential_bound: sidecar lease posture recorded when provider access is needed.
- source_discovered: source objects listed with provenance digests.
- transform_previewed: proposed mapping shown without canonical mutation.
- projection_pending: ontology mutation prepared.
- projection_written: ontology object or edge written.
- event_published: AsyncAPI event emitted.
- audit_sealed: audit-chain evidence emitted.
- review_required: human review required before continuation.
- blocked_by_pack: pack or residency conflict blocks continuation.
- retry_scheduled: transient retry scheduled with idempotency key.
- replaying: deterministic replay in progress.
- completed: all required evidence and rollback paths present.
- failed_reversible: failed with rollback bundle intact.
- failed_quarantined: failed with source payload or projection quarantined.
- expired: break-glass or temporary permit expired.
- rolled_back: prior state restored or compensating event emitted.

## HI-FHIR-READ-IMPORT
- Step 1: construct TenantScope from IP-001 fields.
- Step 2: evaluate Cedar fhir-read permit from IP-002.
- Step 3: bind provider credential lease when source access is needed.
- Step 4: discover FHIR resources by tenant-approved source query.
- Step 5: compute source digest before transform.
- Step 6: preview FHIR-to-ontology projection.
- Step 7: check consent and pack overlays.
- Step 8: write ontology projection from IP-003.
- Step 9: publish accepted/completed event from IP-006.
- Step 10: seal audit evidence.
- Step 11: retain rollback bundle.
- Step 12: expose progress to REST action status from IP-005.

## HI-HL7-ROUTE
- Step 1: accept HL7 envelope with tenant, facility, interface, and message control id.
- Step 2: compute source digest before parsing.
- Step 3: evaluate hl7-route Cedar permit.
- Step 4: bind credential lease for outbound route when needed.
- Step 5: resolve route decision with pack and residency labels.
- Step 6: publish route accepted event.
- Step 7: invoke adapter outside domain boundary.
- Step 8: record ACK or NACK with digest and timestamp.
- Step 9: project route result into ontology route decision.
- Step 10: handle duplicate message id through idempotency key.
- Step 11: quarantine malformed message with PHI-safe evidence.
- Step 12: route backlog alerts to runbooks/hl7-queue-backlog.md.

## HI-BREAK-GLASS-REVIEW
- Step 1: require tenant_id even for emergency audience.
- Step 2: require emergency_attestation or tenant break-glass permit.
- Step 3: require break_glass_justification.
- Step 4: set expiry before permit is used.
- Step 5: emit audit event at authorization time.
- Step 6: project break-glass event into ontology.
- Step 7: publish emergency access event.
- Step 8: schedule posthoc review task.
- Step 9: expire access automatically.
- Step 10: record reviewer outcome.
- Step 11: escalate missing review through runbooks/break-glass-audit-review.md.
- Step 12: preserve evidence packet for HIPAA and local packs.

## HI-CONSENT-SYNC
- Step 1: ingest consent source under tenant scope.
- Step 2: evaluate consent-sync Cedar permit.
- Step 3: compute consent source digest.
- Step 4: map grant, denial, revoke, expire, and conflict.
- Step 5: apply most restrictive status on conflict.
- Step 6: project clinical consent object.
- Step 7: update consent graph references.
- Step 8: publish consent changed event.
- Step 9: pause dependent exports when consent narrows.
- Step 10: open remediation workflow for conflicts.
- Step 11: use runbooks/consent-sync-conflict.md for operators.
- Step 12: seal audit evidence.

## HI-EHR-PROVENANCE-SEAL
- Step 1: require source_system_ref and source digest.
- Step 2: require ontology_object_ref.
- Step 3: evaluate ehr-provenance-seal permit.
- Step 4: bind replay batch when historical import is involved.
- Step 5: compute canonical projection digest.
- Step 6: compare source, import, export, and replay digests.
- Step 7: publish provenance seal event.
- Step 8: project provenance seal.
- Step 9: quarantine mismatched digest.
- Step 10: open runbooks/ehr-provenance-gap.md if digest gap persists.
- Step 11: preserve rollback bundle.
- Step 12: emit regulator-ready evidence.

## HI-PATIENT-MATCH-REVIEW
- Step 1: collect candidate records under one tenant unless referral relation exists.
- Step 2: require patient-match-review Cedar permit.
- Step 3: compute candidate evidence without merging identity.
- Step 4: present PHI-minimized review bundle.
- Step 5: require human reviewer for match decision.
- Step 6: project PatientMatchReview object.
- Step 7: update linkage only inside authorized tenant or referral scope.
- Step 8: publish match review event.
- Step 9: retain rejected candidate evidence.
- Step 10: route duplicates to runbooks/patient-match-duplicate.md.
- Step 11: block cross-tenant merge by default.
- Step 12: seal audit evidence.

## Competitor Displacement
- Redox displacement: Redox workflows are network exchange oriented; Oyatie templates bind workflow, policy, ontology, event, and evidence in one controlled path.
- Rhapsody displacement: Rhapsody route workflows become adapter steps inside Oyatie HL7 route templates, not the governing workflow.
- InterSystems IRIS for Health displacement: IRIS process orchestration is replaced by platform workflow templates that preserve tenant cells and pack controls.
- Lyniate/Corepoint displacement: Corepoint interface work is only the adapter segment after policy and before evidence sealing.
- Mirth Connect displacement: Mirth channel pipelines are displaced by workflow states with explicit rollback, replay, audit, and review gates.
- NextGate displacement: NextGate matching is displaced by a patient-match review workflow that refuses automatic cross-tenant identity collapse.
- Health Catalyst displacement: Health Catalyst analytics pipelines are displaced by evidence-first export workflows for governed extracts.
- Epic displacement: Epic import becomes a FHIR import workflow, not a suite-specific privileged path.
- Cerner displacement: Cerner feed routing becomes HL7 route workflow under common policy.
- Allscripts displacement: Allscripts feed quirks become transform preview and quarantine cases.
- Veeva displacement: Veeva regulated flow patterns become GxP pack overlays inside common templates.

## Implementation Slices
- Slice 1: define template metadata schema.
- Slice 2: define shared input contract.
- Slice 3: define shared state machine.
- Slice 4: define HI-FHIR-READ-IMPORT template.
- Slice 5: define HI-HL7-ROUTE template.
- Slice 6: define HI-BREAK-GLASS-REVIEW template.
- Slice 7: define HI-CONSENT-SYNC template.
- Slice 8: define HI-EHR-PROVENANCE-SEAL template.
- Slice 9: define HI-PATIENT-MATCH-REVIEW template.
- Slice 10: define HI-REFERRAL-HANDOFF template.
- Slice 11: define HI-PROVIDER-DIRECTORY-SYNC template.
- Slice 12: define HI-BULK-BACKFILL-REPLAY template.
- Slice 13: define HI-EXPORT-EVIDENCE-PACKET template.
- Slice 14: define rollback bundle requirements.
- Slice 15: define retry and replay semantics.
- Slice 16: define human review gates.
- Slice 17: define pack conflict pause semantics.
- Slice 18: define event publication hooks.
- Slice 19: define runbook links.
- Slice 20: define fixture set for each template.

## Tests and Evidence
- Test every template rejects absent tenant_id.
- Test every template records Cedar decision id before mutation.
- Test every template records workflow_run_id.
- Test every async step records idempotency_key.
- Test FHIR import can be replayed without duplicate projection.
- Test HL7 route duplicate is idempotent.
- Test break-glass expiry happens automatically.
- Test consent conflict pauses dependent exports.
- Test provenance digest mismatch quarantines workflow.
- Test patient match review requires human decision.
- Test referral handoff requires relationship evidence.
- Test provider directory sync cannot read PHI.
- Test bulk replay respects pack residency.
- Test evidence export blocks on residency conflict.
- Test rollback bundle exists before externally visible transition.
- Test event publication uses asyncapi-v1.yaml fields.
- Test REST status uses openapi-v1.yaml action status contract.
- Test runbook links resolve for operational failures.
- Test Redox/Rhapsody/Mirth-style connector outputs cannot skip workflow gates.
- Test Health Catalyst-style export starts from evidence packet workflow.

## Acceptance Criteria
- AC-001: Each template requires TenantScope fields.
- AC-002: Each template records Cedar decision before mutation.
- AC-003: Each template records ontology projection or explicit no-projection reason.
- AC-004: Each template records audit_event_class.
- AC-005: Each template has rollback or quarantine semantics.
- AC-006: Each template has replay semantics when async or source-driven.
- AC-007: Human review gates exist for patient match, consent conflict, and break-glass.
- AC-008: Pack conflict can pause workflows.
- AC-009: Workflow states are explicit and machine-checkable.
- AC-010: Runbook references exist for major operational failures.
- AC-011: ADR-0035 workflow hybrid model is respected.
- AC-012: ADR-0105 layer separation is respected.
- AC-013: ADR-0244 tenant scope is respected.
- AC-014: ADR-0263 audit emission is respected.
- AC-015: ADR-0321 industry-leader depth includes explicit benchmark displacement.
- AC-016: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst are explicitly displaced.
- AC-017: The IP remains implementable without editing ADR-0321.
- AC-018: The IP cites repo-local workflow, runbook, and service references.
- AC-019: Workflow templates do not embed provider-specific tenancy.
- AC-020: Workflow template fixtures cover import, route, consent reconciliation, MPI adjudication, break-glass review, and provenance export states.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-004-workflow-template-library.md:21` - Repo references: microservices/healthcare-integration/contracts/asyncapi-v1.yaml; `microservices/healthcare-integration/IP-004-workflow-template-library.md:225` - - Test event publication uses asyncapi-v1.yaml fields..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-004-workflow-template-library.md:44` - - Template HI-PROVIDER-DIRECTORY-SYNC refreshes provider/facility/network metadata without PHI access.; `microservices/healthcare-integration/IP-004-workflow-template-library.md:115` - - Step 11: quarantine malformed message with PHI-safe evidence..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-004-workflow-template-library.md:245` - - AC-014: ADR-0263 audit emission is respected..
