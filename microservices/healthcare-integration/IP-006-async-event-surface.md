# IP-006 Healthcare Integration Async Event Surface

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-006-async-event-surface.md
Batch: Batch C healthcare-integration IP deepening
Status: implementation-plan
Owner: axis-healthcare-integration + council-platform-events
Primary layer: worker + adapter + governance
Primary contract: microservices/healthcare-integration/contracts/asyncapi-v1.yaml
Primary topic family: healthcare-integration.events.v1
Primary bounded contexts: patient-record, fhir-resource, hl7-message, referral, clinical-consent
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md
Repo references: microservices/healthcare-integration/ARCHITECTURE.md
Repo references: microservices/healthcare-integration/contracts/asyncapi-v1.yaml
Repo references: microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml
Repo references: microservices/healthcare-integration/slos/audit-emission-lag.openslo.yaml
Repo references: microservices/healthcare-integration/slos/replay-freshness.openslo.yaml
Repo references: microservices/healthcare-integration/slos/local-audit-completeness.openslo.yaml
Repo references: microservices/healthcare-integration/dashboards/slo-and-error-budget.json
Repo references: microservices/healthcare-integration/dashboards/local-domain-throughput.json
Repo references: microservices/healthcare-integration/backfill-replay.md
Repo references: microservices/healthcare-integration/runbooks/hl7-queue-backlog.md
Repo references: microservices/healthcare-integration/runbooks/fhir-endpoint-degradation.md
Repo references: docs/decisions/ADR-0005-eventing-backbone-outbox-pattern.md
Repo references: docs/decisions/ADR-0153-outbox-pattern.md
Repo references: docs/decisions/ADR-0154-event-schema-versioning.md

## Objective
- Define the Healthcare Integration async event surface for accepted actions, workflow progress, projection changes, audit evidence, replay, and failure handling.
- Make events tenant-scoped, policy-visible, ontology-aware, replayable, and PHI-minimized.
- Use AsyncAPI 3.1.0 as the contract authority.
- Use the outbox pattern for exactly-once effect approximation and retry safety.
- Use event schema versioning to avoid breaking downstream consumers.
- Use audit emission lag and replay freshness SLOs as operating evidence.
- Use local AsyncAPI projection for local deployment without weakening production fields.
- Keep ADR-0321 B2B leader substance by displacing healthcare interface queues with governed platform events.
- Keep source-system messages as payload evidence, not as direct event authority.

## Event Families
- HealthcareIntegrationActionAccepted records REST or workflow acceptance.
- HealthcareIntegrationActionDenied records PHI-safe policy or validation denial.
- HealthcareIntegrationWorkflowStarted records workflow template instance start.
- HealthcareIntegrationWorkflowProgressed records state transition.
- HealthcareIntegrationWorkflowBlocked records pack, policy, residency, review, or backpressure block.
- HealthcareIntegrationWorkflowCompleted records completion with audit and rollback evidence.
- HealthcareIntegrationWorkflowFailed records reversible or quarantined failure.
- HealthcareIntegrationFhirResourceImported records source-to-ontology FHIR import success.
- HealthcareIntegrationHl7MessageRouted records route decision and ACK/NACK outcome.
- HealthcareIntegrationConsentChanged records grant, deny, revoke, expire, or conflict.
- HealthcareIntegrationBreakGlassAuthorized records emergency or tenant break-glass access.
- HealthcareIntegrationBreakGlassExpired records automatic expiry.
- HealthcareIntegrationBreakGlassReviewed records posthoc review result.
- HealthcareIntegrationEhrProvenanceSealed records source/import/export/replay digest seal.
- HealthcareIntegrationPatientMatchReviewRequired records candidate set requiring review.
- HealthcareIntegrationPatientMatchReviewed records reviewer decision.
- HealthcareIntegrationReferralHandedOff records authorized referral relationship flow.
- HealthcareIntegrationProviderDirectorySynced records provider metadata update without PHI grant.
- HealthcareIntegrationReplayStarted records backfill or replay batch start.
- HealthcareIntegrationReplayCompleted records replay completion with divergence summary.
- HealthcareIntegrationProjectionConflictDetected records source-to-ontology disagreement.
- HealthcareIntegrationEvidenceExported records policy-cleared evidence export.
- HealthcareIntegrationResidencyBlocked records pack or home-cell export block.
- HealthcareIntegrationCredentialLeaseRejected records provider adapter credential posture failure.

## Required Envelope
- event_id is required.
- event_type is required.
- event_version is required.
- tenant_id is required.
- principal_id is required when human or authenticated system action triggered the event.
- audience_type is required.
- home_cell is required.
- jurisdiction_code is required.
- data_class is required.
- purpose is required.
- audit_event_class is required.
- event_time is required.
- trace_context is required.
- idempotency_key is required for command-derived events.
- workflow_run_id is required for workflow-derived events.
- cedar_decision_id is required for accepted, denied, blocked, and mutation events.
- ontology_object_ref is required when projection exists.
- source_system_ref is required for source-system-derived events.
- source_digest is required for source-system-derived events.
- deal_set_id is required when commercial route or marketplace settlement applies.
- pack_ids are required for compliance-sensitive events.
- residency_label is required before export or replication events.
- phi_redaction_profile is required for events visible to dashboards or external consumers.
- schema_ref is required.
- producer is required.
- outbox_record_id is required.

## Payload Rules
- Payloads do not include raw PHI by default.
- Payloads include references to sealed evidence rather than source payloads.
- Payloads include source digests for replay and audit.
- Payloads include denial_code and PHI-safe denial_message for denial events.
- Payloads include workflow state for progress events.
- Payloads include rollback_bundle_ref before externally visible mutation completion.
- Payloads include projection_version when ontology state changes.
- Payloads include previous_projection_version when an update supersedes prior projection.
- Payloads include conflict_summary for projection conflicts.
- Payloads include ack_status for HL7 route outcomes.
- Payloads include consent_status for consent changes.
- Payloads include break_glass_expiry for break-glass authorization.
- Payloads include reviewer_ref for human review outcomes.
- Payloads include replay_batch_id for replay events.
- Payloads include divergence_count for replay completion.
- Payloads include evidence_packet_ref for evidence export.
- Payloads include pack_decision_ref for residency or pack blocks.
- Payloads include credential_lease_ref only as lease metadata, never secret values.
- Payloads include deal_set_id where settlement applies.
- Payloads include no patient global identifier outside tenant scope.

## Topics and Partitions
- Topic healthcare-integration.events.v1 carries domain events.
- Topic healthcare-integration.audit.v1 carries audit-forwarding envelopes when separated.
- Topic healthcare-integration.replay.v1 carries replay and backfill progress.
- Topic healthcare-integration.deadletter.v1 carries quarantined events with PHI-safe metadata.
- Partition key uses tenant_id plus bounded_context plus workflow_run_id when available.
- Patient identifiers are never partition keys.
- Source-system identifiers are never sole partition keys.
- DealSet identifiers are never sole partition keys.
- Emergency events use tenant_id plus break_glass_event_id.
- Replay events use tenant_id plus replay_batch_id.
- Evidence export events use tenant_id plus evidence_packet_ref.
- Deadletter events use tenant_id plus original_event_id.
- Cross-tenant referral events emit separate tenant-visible events when policy permits.
- Metrics labels avoid raw tenant_id and use aggregated dimensions where required.
- Audit-chain evidence stores tenant id in signed evidence, not high-cardinality metrics.

## Event Versioning
- event_version starts at 1.0.0 for this v1 surface.
- Additive optional fields require minor version.
- New required fields require dual-publish or compatibility window.
- Field removal requires ADR-0154 deprecation path.
- Semantic narrowing requires ADR-0258 API/version compatibility review.
- Event type removal requires sunset metadata.
- Local AsyncAPI projection cannot remove production-required fields.
- Consumers must reject unknown critical fields only when marked critical.
- Producers must include schema_ref for each event.
- Replay must preserve original event_version and include replay_envelope_version.
- Deadletter must preserve original event_type and event_version.

## Competitor Displacement
- Redox displacement: Redox event delivery is displaced by tenant-scoped outbox events with policy, ontology, workflow, and audit references.
- Rhapsody displacement: Rhapsody queues are displaced by governed topic families with replay, deadletter, SLO, and schema versioning.
- InterSystems IRIS for Health displacement: IRIS interoperability messages are displaced by platform events that preserve tenant cell and pack evidence.
- Lyniate/Corepoint displacement: Corepoint route outputs become source evidence behind Oyatie events, not the event contract itself.
- Mirth Connect displacement: Mirth channels are displaced by AsyncAPI-defined events with PHI-minimized payload and outbox identity.
- NextGate displacement: NextGate match notifications become review-required events that cannot merge patient identity automatically.
- Health Catalyst displacement: Health Catalyst analytics feeds are displaced by evidence-cleared export events and replay freshness controls.
- Epic displacement: Epic FHIR subscription or polling output becomes source-system-derived events under common schema.
- Cerner displacement: Cerner HL7/FHIR feed events become source provenance behind tenant-scoped Oyatie events.
- Allscripts displacement: Allscripts interface events become route/projection events with conflict evidence.
- Veeva displacement: Veeva regulated notifications become pack-aware events with audit and GxP evidence.

## Implementation Slices
- Slice 1: expand contracts/asyncapi-v1.yaml message catalog.
- Slice 2: add required envelope schema.
- Slice 3: add PHI-minimized payload schemas.
- Slice 4: add ActionAccepted and ActionDenied messages.
- Slice 5: add workflow started/progressed/blocked/completed/failed messages.
- Slice 6: add FHIR import message.
- Slice 7: add HL7 route message.
- Slice 8: add consent changed message.
- Slice 9: add break-glass authorized/expired/reviewed messages.
- Slice 10: add provenance sealed message.
- Slice 11: add patient match review required/reviewed messages.
- Slice 12: add referral handoff message.
- Slice 13: add provider directory sync message.
- Slice 14: add replay started/completed messages.
- Slice 15: add projection conflict message.
- Slice 16: add evidence exported message.
- Slice 17: add residency blocked message.
- Slice 18: add credential lease rejected message.
- Slice 19: add deadletter envelope.
- Slice 20: add local-asyncapi parity checks.

## Outbox and Delivery Semantics
- Every event is written to outbox inside the same transaction boundary as state transition where applicable.
- Outbox record id is included in event envelope.
- Producer retries are idempotent by event_id.
- Consumer processing is idempotent by tenant_id, event_type, event_id, and event_version.
- Audit-critical events block mutation completion until outbox write succeeds.
- Non-critical progress events may be retried with bounded delay.
- Deadletter requires PHI-safe metadata only.
- Replay consumes original event and records replay envelope.
- Backfill replay uses source_digest and idempotency_key.
- Event publication records trace context.
- Event publication records schema_ref.
- Event publication records producer version.
- Event publication records workflow_run_id when applicable.
- Event publication records rollback_bundle_ref when applicable.
- Event publication records pack decision reference when applicable.

## Failure Modes
- Outbox write fails: block audit-critical mutation.
- Broker publish fails: keep outbox pending and alert burn budget.
- Consumer rejects schema: route to deadletter with schema_ref.
- Event missing tenant_id: reject before outbox.
- Event includes raw PHI in dashboard-visible field: fail contract test.
- Replay divergence detected: emit ProjectionConflictDetected.
- Deadletter backlog grows: trigger hl7-queue-backlog or endpoint degradation runbook.
- Audit emission lag SLO burns: trigger audit emission alert.
- Replay freshness SLO burns: trigger replay remediation.
- Credential lease rejected: emit credential lease event without secret.
- Residency block occurs: emit ResidencyBlocked and pause export.
- Break-glass expiry event fails: fail closed and revoke active temporary access.
- Consent revocation event delayed: pause dependent exports.
- Cross-tenant referral event leaks receiving tenant details: fail contract test.
- Local AsyncAPI omits production field: fail parity check.

## Tests and Evidence
- Test every message requires event_id, event_type, event_version, tenant_id, audit_event_class, event_time, and schema_ref.
- Test command-derived events require idempotency_key.
- Test workflow-derived events require workflow_run_id.
- Test source-derived events require source_system_ref and source_digest.
- Test accepted events include cedar_decision_id.
- Test denied events include PHI-safe denial_code.
- Test projection events include ontology_object_ref and projection_version.
- Test HL7 route event includes ack_status.
- Test consent event includes consent_status.
- Test break-glass event includes expiry.
- Test patient match event includes review_required or reviewer_ref.
- Test replay completion includes divergence_count.
- Test evidence export includes evidence_packet_ref.
- Test residency block includes pack_decision_ref.
- Test credential lease rejection excludes secret value.
- Test deadletter preserves original event type and version.
- Test local-asyncapi-v1.yaml does not weaken production envelope.
- Test audit-emission-lag SLO names event publication behavior.
- Test replay-freshness SLO names replay event behavior.
- Test Redox/Rhapsody/Mirth-style events cannot omit tenant or audit fields.

## Acceptance Criteria
- AC-001: AsyncAPI v1 covers action, workflow, FHIR, HL7, consent, break-glass, provenance, match, replay, conflict, export, residency, credential, and deadletter events.
- AC-002: Every event is tenant-scoped.
- AC-003: Every accepted or denied decision event carries Cedar decision context when safe.
- AC-004: Every event is PHI-minimized by default.
- AC-005: Source payloads are referenced by digest and evidence ref, not copied into events.
- AC-006: Outbox semantics are required for state-changing events.
- AC-007: Replay semantics preserve original event version.
- AC-008: Deadletter semantics preserve original event type and version.
- AC-009: Local AsyncAPI cannot weaken production envelope.
- AC-010: SLOs cover audit emission lag and replay freshness.
- AC-011: Events do not use patient ids as partition keys.
- AC-012: DealSet obligations appear where settlement applies.
- AC-013: Pack and residency blocks are first-class events.
- AC-014: ADR-0154 event schema versioning is respected.
- AC-015: ADR-0321 industry-leader depth includes explicit benchmark displacement.
- AC-016: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst are explicitly displaced.
- AC-017: The IP remains implementable without editing ADR-0321.
- AC-018: The IP cites repo-local AsyncAPI, SLO, dashboard, replay, and runbook references.
- AC-019: Async events are governed platform events, not provider queue pass-through.
- AC-020: Async examples cover route accepted, route refused, consent changed, MPI adjudication opened, break-glass reviewed, and provenance sealed events.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-006-async-event-surface.md:9` - Primary contract: microservices/healthcare-integration/contracts/asyncapi-v1.yaml; `microservices/healthcare-integration/IP-006-async-event-surface.md:15` - Repo references: microservices/healthcare-integration/contracts/asyncapi-v1.yaml.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-006-async-event-surface.md:31` - - Make events tenant-scoped, policy-visible, ontology-aware, replayable, and PHI-minimized.; `microservices/healthcare-integration/IP-006-async-event-surface.md:35` - - Use audit emission lag and replay freshness SLOs as operating evidence..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-006-async-event-surface.md:17` - Repo references: microservices/healthcare-integration/slos/audit-emission-lag.openslo.yaml; `microservices/healthcare-integration/IP-006-async-event-surface.md:35` - - Use audit emission lag and replay freshness SLOs as operating evidence..
