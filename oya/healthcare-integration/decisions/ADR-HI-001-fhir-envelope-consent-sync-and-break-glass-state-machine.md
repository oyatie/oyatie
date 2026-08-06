---
id: ADR-HI-001
title: fhir-envelope-consent-sync-and-break-glass-state-machine
status: Proposed
date: 2026-05-20
microservice: healthcare-integration
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
decision_owner: healthcare-integration-platform-architecture
---

# ADR-HI-001: FHIR Envelope, Consent Sync, And Break-Glass State Machine

## Context

- Architectural pressure: Clinical Interoperability Safety Pressure.
- Healthcare Integration owns clinical interoperability, consent, break-glass, regulated health-record provenance, patient matching, FHIR resources, and HL7 message ingestion.
- The service must normalize clinical records without pretending that external EHR systems share the same identity, consent, or audit semantics.
- Clinical integrations need a stable envelope because FHIR resources, HL7 v2 messages, referral documents, and patient-match candidates carry different provenance shapes.
- Consent state can change outside Oyatie, and clinical access decisions must honor that change before downstream work sees protected health data.
- Emergency access must be available, but it has to be bounded, logged, reviewed, and separated from ordinary authorization.
- Patient identity matching must preserve candidate uncertainty because an incorrect merge can expose records across patients.
- The service must support Epic, Cerner, Allscripts, Veeva, SMART on FHIR, HL7 v2, and direct partner adapters without making any one vendor the internal model.
- Named constraint: PHI Minimum Necessary Constraint.
- The PHI Minimum Necessary Constraint requires every projection and API response to include only the clinical fields needed by the requesting workflow.
- Named constraint: Consent Freshness Constraint.
- The Consent Freshness Constraint requires cached consent to be no older than 60 seconds for read access and no older than 5 minutes for asynchronous workflow continuation.
- Named constraint: Break-Glass Accountability Constraint.
- The Break-Glass Accountability Constraint requires emergency access to name patient, reason, actor, duration, and post-event reviewer before data release.
- Named constraint: Source Provenance Constraint.
- The Source Provenance Constraint requires every normalized envelope to retain source_system, source_record_id, source_version, received_at, and transform_version.
- Named constraint: Patient Match Uncertainty Constraint.
- The Patient Match Uncertainty Constraint requires probabilistic matches below 0.98 confidence to remain candidate records until manual or rules-based adjudication.
- Named constraint: Jurisdictional Consent Constraint.
- The Jurisdictional Consent Constraint requires consent transitions to use tenant region pack, patient residency, care site jurisdiction, and consent purpose.
- Named constraint: Audit Chain Constraint.
- The Audit Chain Constraint requires access, transformation, disclosure, break-glass, and consent sync events to emit audit records before clinical data is returned.
- Named constraint: Adapter Isolation Constraint.
- The Adapter Isolation Constraint prevents vendor adapter retry, pagination, and mapping behavior from leaking into domain APIs.
- Existing service docs name patient-record, fhir-resource, hl7-message, referral, and clinical-consent as core bounded contexts.
- The service must let other Oyatie products request clinical facts while keeping healthcare-specific consent and provenance inside healthcare-integration.
- The decision must be specific enough to produce OpenAPI, AsyncAPI, protobuf, Cedar, dashboards, and runbooks.
- The design must not collapse FHIR resources into generic JSON blobs; domain evidence needs typed resource_kind and profile references.
- The design must not over-normalize clinical content into one universal table because FHIR profile variance is expected.
- The design needs deterministic replay for regulator, customer, and security investigations.
- Break-glass must be a state machine, not a boolean flag, because it has request, active, expired, revoked, and reviewed phases.
- Consent sync must tolerate external system lag and still expose the reason access is denied or stale.
- The service has to support batch imports and interactive point-of-care reads with different latency and completeness expectations.

## Decision

- We will implement the Clinical Record Envelope with Consent Sync and Break-Glass State Machine.
- The named pattern is canonical envelope plus resource projection plus consent-state ledger plus emergency-access state machine.
- The named technology choice is service-local Postgres for envelope and consent ledgers, FHIR profile-aware validation in the adapter boundary, Cedar for access decisions, and AsyncAPI CloudEvents for disclosure events.
- ClinicalRecordEnvelope will be the canonical unit of provenance for every FHIR resource, HL7 v2 message, referral document, and patient-match update.
- FhirResourceProjection will store searchable, minimum-necessary fields for approved resource kinds and profile versions.
- Raw clinical payloads will be encrypted by tenant and region pack and referenced by content_hash, not copied into every projection.
- Hl7MessageEnvelope will preserve message_control_id, sending_application, receiving_application, trigger_event, segment_hashes, and ack_state.
- ClinicalConsentGrant will represent jurisdiction-aware consent with subject_patient_id, purpose_of_use, actor_scope, data_category, source_system, effective_at, expires_at, and revocation_state.
- BreakGlassSession will represent requested, active, expired, revoked, and reviewed states.
- A break-glass session must expire within 30 minutes unless a jurisdiction pack sets a lower maximum.
- Break-glass access must be reviewed within 24 hours by a clinical_privacy_reviewer or it becomes overdue_review.
- PatientMatchCandidate confidence >= 0.98 may auto-link when two identifiers from different namespaces agree.
- PatientMatchCandidate confidence >= 0.92 and < 0.98 requires adjudication before record consolidation.
- PatientMatchCandidate confidence < 0.92 remains unlinked and visible only to matching operators.
- Consent cache freshness must be p95 under 60 seconds for interactive reads and p99 under 5 minutes for async workflow checks.
- FHIR bundle ingestion must validate resource references and profile conformance before projection.
- FHIR bundle ingestion must target p95 2 seconds for bundles under 100 resources and p99 15 seconds for bundles under 1000 resources.
- HL7 v2 inbound message acknowledgement must target p95 2 seconds and p99 10 seconds.
- Clinical read authorization must target p95 100 ms and p99 250 ms after consent cache lookup.
- Disclosure event publication must target p99 3 seconds after clinical data response.
- A data response is prohibited when consent is stale, revoked, absent, or incompatible with purpose_of_use unless an active break-glass session covers the request.
- A break-glass response must include a disclosure banner token, audit_event_id, session_id, reason_code, and expires_at.
- All data access APIs must evaluate Cedar using actor, patient, tenant, purpose_of_use, data_category, consent_state, break_glass_state, and region_pack.
- External EHR adapters must not return data directly to callers; they write envelopes and projections first, then domain APIs read projections.
- Consent sync events from external systems must be idempotent by source_system, source_consent_id, source_version, and normalized_consent_hash.
- The service will publish healthcare_integration.clinical_disclosure.v1, healthcare_integration.consent_state_changed.v1, healthcare_integration.break_glass_state_changed.v1, and healthcare_integration.patient_match_changed.v1 events.
- Each event must include traceparent, tenant_id, region_pack, patient_ref, consent_version, evidence_hash, and disclosure_scope.

## Alternatives Considered

- Alternative 1: Direct FHIR Passthrough.
- Pros: Preserves vendor payloads with minimal transformation.
- Pros: Allows FHIR clients to use familiar resource shapes.
- Cons: Consent, audit, minimum necessary filtering, and patient matching would be reimplemented by consumers.
- Cons: HL7 v2, referral documents, and non-FHIR evidence would remain second-class.
- Cons: Vendor-specific quirks would leak into Oyatie product contracts.
- Rejection reason: Healthcare Integration needs a canonical provenance envelope and access gate.

- Alternative 2: Fully Flattened Clinical Warehouse.
- Pros: Analytical queries and downstream joins could be faster.
- Pros: Reporting teams would see stable relational columns.
- Cons: FHIR profile evolution and extension fields would be lossy or brittle.
- Cons: Minimum-necessary access would be harder because all fields become broadly queryable.
- Cons: Interactive clinical reads would depend on warehouse freshness.
- Rejection reason: The service needs typed projections, not a single flattened warehouse.

- Alternative 3: Consent As External EHR Truth Only.
- Pros: Avoids maintaining consent state locally.
- Pros: Reduces risk of conflict with source-of-care systems.
- Cons: Every read would depend on external EHR availability and latency.
- Cons: Async workflows could run on stale assumptions without a local consent transition audit.
- Cons: Break-glass review and disclosure reporting would be fragmented.
- Rejection reason: Oyatie needs local consent-state evidence for safe access decisions.

- Alternative 4: Generic Emergency Override Flag.
- Pros: Simple to implement and easy for support teams to understand.
- Pros: Works for early incident response demos.
- Cons: It does not encode duration, reason, review, revocation, or patient scope.
- Cons: It cannot prove post-event accountability.
- Cons: It risks becoming a permanent bypass in production.
- Rejection reason: Break-glass must be a bounded auditable state machine.

- Alternative 5: Patient Merge As Immediate Mutation.
- Pros: Auto-merging high-confidence candidates simplifies downstream reads.
- Pros: Duplicate charts would be reduced quickly.
- Cons: Incorrect merges are high-impact privacy and clinical-safety incidents.
- Cons: It loses uncertainty evidence needed for audit and reversal.
- Cons: Jurisdiction-specific identity rules cannot be represented by one threshold.
- Rejection reason: Candidate uncertainty must remain explicit until thresholds and policies allow link.

## Consequences

- Positive consequence: Clinical access decisions are explainable through envelope provenance, consent state, and break-glass state.
- Positive consequence: External EHR adapters can vary without breaking Oyatie service contracts.
- Positive consequence: Consent sync lag becomes measurable and visible to operators.
- Positive consequence: Break-glass access is available for emergencies without becoming an untracked bypass.
- Positive consequence: Patient matching can improve over time while preserving uncertain candidate evidence.
- Positive consequence: FHIR profile validation happens before projection, reducing downstream data ambiguity.
- Positive consequence: Audit events can reconstruct what was disclosed, why, under which consent, and through which adapter.
- Negative consequence: Every clinical read now depends on consent projection health and Cedar policy latency.
- Negative consequence: FHIR profile validation and HL7 parsing increase adapter complexity.
- Negative consequence: Some downstream consumers will need to adapt to minimum-necessary response scopes.
- Negative consequence: Patient matching thresholds require careful tuning and recurring false-link review.
- Negative consequence: Break-glass review creates operational work for privacy reviewers.
- Neutral consequence: Raw payload retention remains adapter-specific but content hashes and envelope metadata are standardized.
- Neutral consequence: FHIR resources are still represented in FHIR terms, but Oyatie controls disclosure APIs.
- Neutral consequence: Batch imports and interactive reads use the same provenance model with different SLOs.
- Follow-up work: HI-FW-001 will define supported FHIR R4 profiles and extension handling.
- Follow-up work: HI-FW-002 will define HL7 v2 ACK and retry handling per source system.
- Follow-up work: HI-FW-003 will build the break-glass reviewer dashboard and overdue review alerts.
- Follow-up work: HI-FW-004 will tune patient-match thresholds against de-identified test corpora.
- Follow-up work: HI-FW-005 will add jurisdiction-pack consent transition tests.

## Implementation Notes

- Data shape: ClinicalRecordEnvelope.
- ClinicalRecordEnvelope fields: envelope_id, tenant_id, patient_ref, source_system, source_record_id, source_version, resource_kind.
- ClinicalRecordEnvelope fields: profile_url, payload_content_hash, transform_version, received_at, region_pack, traceparent.
- Data shape: FhirResourceProjection.
- FhirResourceProjection fields: projection_id, envelope_id, fhir_version, resource_type, profile_url, logical_id, patient_ref.
- FhirResourceProjection fields: effective_time, status, coded_summary, searchable_refs, projection_hash, projection_version.
- Data shape: Hl7MessageEnvelope.
- Hl7MessageEnvelope fields: message_id, tenant_id, source_system, message_control_id, trigger_event, sending_application.
- Hl7MessageEnvelope fields: receiving_application, segment_hashes, ack_state, ack_code, received_at, transformed_envelope_ids.
- Data shape: ClinicalConsentGrant.
- ClinicalConsentGrant fields: consent_id, tenant_id, patient_ref, source_system, source_consent_id, source_version.
- ClinicalConsentGrant fields: purpose_of_use, actor_scope, data_category, jurisdiction, effective_at, expires_at, revoked_at.
- Data shape: ConsentTransition.
- ConsentTransition fields: transition_id, consent_id, from_state, to_state, trigger_kind, source_event_id, evidence_hash, occurred_at.
- Data shape: BreakGlassSession.
- BreakGlassSession fields: session_id, tenant_id, patient_ref, actor_principal, reason_code, requested_at, activated_at.
- BreakGlassSession fields: expires_at, revoked_at, reviewed_at, reviewer_principal, review_outcome, disclosure_scope.
- Data shape: PatientMatchCandidate.
- PatientMatchCandidate fields: candidate_id, tenant_id, source_patient_ref, target_patient_ref, confidence, match_features_hash.
- PatientMatchCandidate fields: adjudication_state, adjudicated_by, adjudicated_at, linked_patient_ref.
- API endpoint: POST /v1/healthcare/fhir/bundles ingests validated FHIR bundles and returns envelope ids.
- API endpoint: POST /v1/healthcare/hl7/messages ingests HL7 v2 messages and returns ACK state.
- API endpoint: POST /v1/healthcare/consents/sync receives external consent transitions idempotently.
- API endpoint: GET /v1/healthcare/patients/{patient_ref}/records returns minimum-necessary clinical projections.
- API endpoint: POST /v1/healthcare/break-glass/sessions requests emergency access for a named patient and reason.
- API endpoint: POST /v1/healthcare/break-glass/sessions/{session_id}/review records privacy review outcome.
- API endpoint: GET /v1/healthcare/patient-matches/{candidate_id} returns match evidence to authorized operators.
- API endpoint: POST /v1/healthcare/patient-matches/{candidate_id}/adjudicate links, rejects, or defers a candidate.
- API endpoint: GET /v1/healthcare/disclosures/{audit_event_id} returns disclosure metadata, not raw PHI.
- Event: healthcare_integration.clinical_disclosure.v1 records every clinical data response.
- Event: healthcare_integration.consent_state_changed.v1 records consent grant, expiry, revocation, and source correction.
- Event: healthcare_integration.break_glass_state_changed.v1 records requested, active, expired, revoked, and reviewed states.
- Event: healthcare_integration.patient_match_changed.v1 records candidate creation, adjudication, link, and rejection.
- Cedar policy: clinical-interoperability-authorization.cedar permits reads only when purpose_of_use, actor_scope, and data_category match active consent.
- Cedar policy: emergency-services-bypass.cedar permits break-glass activation only for approved emergency roles, reason codes, and patient scope.
- Cedar policy: data-residency.md denies raw payload transfer outside region_pack unless the adapter has explicit cross-border authorization.
- Cedar policy: auditor-scope.cedar permits audit metadata read without returning raw PHI payloads.
- Cedar policy: abuse-defence.cedar rate-limits repeated patient search and broad disclosure attempts.
- Cedar policy: patient-match-adjudication.cedar restricts candidate link and reject actions to identity-resolution operators.
- Cedar policy: consent-sync-writer.cedar allows only trusted adapter principals to submit external consent transitions.
- Cedar policy: break-glass-review.cedar requires reviewer_principal to differ from actor_principal.
- SLO target: clinical read availability is 99.95 percent monthly.
- SLO target: clinical read authorization latency is p95 100 ms and p99 250 ms after consent cache lookup.
- SLO target: FHIR bundle ingestion under 100 resources is p95 2 seconds and p99 15 seconds.
- SLO target: FHIR bundle ingestion under 1000 resources is p95 15 seconds and p99 60 seconds.
- SLO target: HL7 ACK latency is p95 2 seconds and p99 10 seconds.
- SLO target: consent cache freshness is p95 60 seconds and p99 5 minutes.
- SLO target: disclosure event publication lag is p99 under 3 seconds.
- SLO target: break-glass overdue review count is zero after 24 hours.
- Dashboard: healthcare-integration-overview shows ingestion rates, clinical reads, consent denials, and break-glass sessions.
- Dashboard: consent-freshness shows external source lag, stale consent denials, and sync failures.
- Dashboard: break-glass-review shows active, expired, overdue, revoked, and reviewed sessions.
- Dashboard: patient-match-quality shows candidate counts, confidence bands, false link reversals, and adjudication latency.
- Dashboard: fhir-profile-conformance shows rejected bundles, unsupported profiles, and transform version drift.
- Runbook: break-glass-overdue-review describes reviewer paging, revocation, and audit packet export.
- Runbook: consent-sync-lag describes source connector health checks and stale-access denial handling.
- Runbook: patient-match-false-link describes link reversal, disclosure review, and affected patient audit.
- Runbook: fhir-profile-rejection describes profile registry update, adapter rollback, and replay.

## Verification

- Test: fhir_bundle_valid_profile_creates_envelopes ingests supported resources and asserts envelope plus projection rows.
- Test: fhir_bundle_invalid_profile_rejected rejects unsupported profile_url and emits no projection.
- Test: hl7_ack_idempotency_replays_message_control_id submits duplicate message and asserts one envelope set.
- Test: consent_revocation_denies_read creates active consent, revokes it, and asserts Cedar denies patient record read.
- Test: consent_stale_denies_interactive_read simulates consent age above 60 seconds and asserts denial reason stale_consent.
- Test: async_workflow_denies_after_five_minute_staleness simulates stale async consent and asserts workflow halt.
- Test: break_glass_activation_allows_emergency_read creates active session and asserts disclosure banner token.
- Test: break_glass_expires_after_30_minutes asserts access denied after session expiry.
- Test: break_glass_review_requires_distinct_reviewer asserts Cedar denies self-review.
- Test: patient_match_auto_link_threshold asserts confidence 0.99 with two identifiers links automatically.
- Test: patient_match_manual_threshold asserts confidence 0.94 remains pending_adjudication.
- Test: minimum_necessary_response_scope asserts restricted data_category omits unrelated clinical fields.
- Test: disclosure_event_before_response uses transaction hooks to assert audit event exists before data return.
- Test: data_residency_denies_raw_payload_export asserts adapter transfer denied outside region pack.
- Metric: healthcare_fhir_bundle_ingest_duration_seconds by source_system, resource_count_bucket, profile_url, and outcome.
- Metric: healthcare_hl7_ack_latency_seconds by source_system, trigger_event, and ack_code.
- Metric: healthcare_consent_freshness_seconds by source_system, jurisdiction, and tenant_id.
- Metric: healthcare_clinical_read_authorization_seconds by purpose_of_use and policy_pack_version.
- Metric: healthcare_break_glass_active_total by tenant_id, reason_code, actor_role, and patient_region.
- Metric: healthcare_break_glass_overdue_review_total by tenant_id and reviewer_team.
- Metric: healthcare_disclosure_event_lag_seconds by disclosure_scope and region_pack.
- Metric: healthcare_patient_match_candidate_total by confidence_bucket and adjudication_state.
- Metric: healthcare_patient_match_false_link_total by source_system and match_version.
- Dashboard: Clinical Access Safety shows consent denials, break-glass reads, stale consent, and disclosure lag.
- Dashboard: FHIR Adapter Health shows bundle failures, profile drift, transform errors, and replay backlog.
- Dashboard: HL7 Inbound Health shows ACK latency, duplicate message count, and segment parse failures.
- Dashboard: Consent Sync Freshness shows source lag and stale-access denial distribution.
- Dashboard: Break-Glass Accountability shows active sessions, overdue reviews, reviewer backlog, and revocations.
- Alert: HealthcareConsentFreshnessHigh fires when p95 freshness exceeds 60 seconds for 10 minutes.
- Alert: ClinicalDisclosureLagHigh fires when p99 disclosure publication exceeds 3 seconds for 10 minutes.
- Alert: HealthcareBreakGlassOverdue fires when any session remains unreviewed after 24 hours.
- Alert: HealthcareFhirProfileRejectionSpike fires when rejection rate exceeds 2 percent for one source system.
- Promotion gate: run FHIR profile conformance tests against supported profile registry.
- Promotion gate: run HL7 parser replay with at least 1000 de-identified messages.
- Promotion gate: run Cedar tests for ordinary read, denied read, break-glass activation, break-glass review, auditor scope, and residency.
- Promotion gate: run load test with 100 clinical reads per second, 20 FHIR bundle ingests per second, and 50 HL7 messages per second.

## References

- HL7, FHIR Release 4 specification.
- HL7, FHIR Release 5 specification.
- HL7, Version 2 Product Platform documentation.
- SMART Health IT, SMART on FHIR Authorization Guide.
- IHE IT Infrastructure, Patient Identifier Cross-reference and Patient Demographics Query profiles.
- U.S. Department of Health and Human Services, HIPAA Security Rule guidance.
- OAuth 2.0 Authorization Framework, RFC 6749.
- JSON Web Token Bearer Profile for OAuth 2.0, RFC 7523.
- OpenID Core 1.0.
- OpenAPI Specification 3.1.0.
- AsyncAPI Specification 3.0.0.
- CloudEvents Specification 1.0.2.
- RFC 9110, HTTP Semantics.
- W3C Trace Context Recommendation.
