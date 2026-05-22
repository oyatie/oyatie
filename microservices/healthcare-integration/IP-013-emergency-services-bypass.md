# IP-013 Healthcare Integration Emergency Services Bypass

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-013-emergency-services-bypass.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Date: 2026-05-20
Owner: axis-healthcare-integration
Capability focus: break-glass-authorize, fhir-read, hl7-route, consent-sync, ehr-provenance-seal, patient-match-review
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Primary local citations:
- microservices/healthcare-integration/PRD.md
- microservices/healthcare-integration/ARCHITECTURE.md
- microservices/healthcare-integration/policy/emergency-services-bypass.cedar
- microservices/healthcare-integration/capabilities/break-glass-authorize.yaml
- microservices/healthcare-integration/capabilities/fhir-read.yaml
- microservices/healthcare-integration/capabilities/hl7-route.yaml
- microservices/healthcare-integration/capabilities/consent-sync.yaml
- microservices/healthcare-integration/runbooks/emergency-services-chaos.md
- microservices/healthcare-integration/runbooks/break-glass-audit-review.md
- microservices/healthcare-integration/dashboards/abuse-defence-outcomes.json
- microservices/healthcare-integration/dashboards/local-policy-decisions.json
- microservices/healthcare-integration/slos/local-hipaa-access-review-latency.openslo.yaml
- microservices/healthcare-integration/incident-response.md
- docs/standards/documentation-rigor.md
- specs/root-hub-pointers.json
- specs/master-plan-sequencing.json

## 1. Executive Intent
- This IP turns emergency access from a generic break-glass checkbox into a governed emergency-services bypass path.
- The bypass is only for clinically urgent external emergency responders, disaster-response agencies, and tenant-approved emergency workflows.
- It does not grant anonymous read access.
- It does not disable tenant scoping.
- It does not disable Cedar.
- It does not bypass audit-chain evidence.
- It does not bypass data residency.
- It does not bypass marketplace DealSet obligations when a commercial exchange is created.
- It narrows the PRD break-glass promise into an enforceable emergency attestation, policy, evidence, and review flow.
- It binds the architecture requirement that healthcare-integration owns clinical interoperability, consent, break-glass, and regulated health-record provenance.
- It uses ADR-0105 layering so emergency routing remains outside domain invariants until policy and application gates succeed.
- It uses ADR-0131 documentation rigor so the bypass has concrete acceptance evidence rather than a broad aspiration.
- It uses ADR-0243 and ADR-0244 tenant and policy controls before provider access.
- It uses ADR-0253-amendment transport posture for HTTP/3, ECH, and PQC negotiation where exposed.
- It uses ADR-0257 ontology read rules so emergency projections are library-first and traceable.
- It uses ADR-0263 audit-event routing for emergency, abuse, and insider-risk evidence.
- It uses ADR-0314 DealSet settlement only after authorization, not as a gate around lifesaving access.
- It uses ADR-0321 as the healthcare-integration documentation-depth anchor without changing ADR-0321.

## 2. B2B Leader Problem
- Hospital networks need external emergency responders to obtain critical records without waiting for a normal administrator workflow.
- Regional healthcare tenants need the same emergency path across EHR, lab, referral, and consent feeds.
- Enterprise buyers reject a platform that treats break-glass as an afterthought because emergency access is a compliance, safety, and incident-response surface.
- Small B2B tenants need emergency access that does not require a dedicated compliance engineering team.
- Marketplace partners need proof that emergency access did not create hidden entitlement or settlement leakage.
- Auditors need to reconstruct why a responder accessed PHI, which tenant authorized the policy, what data classes moved, and who reviewed the event afterward.
- SREs need emergency access to degrade gracefully during regional incidents without cross-cell residency violations.
- Security teams need bot, abuse, and fraud controls that do not add friction to validated emergency services.
- Product leaders need a story that beats integration middleware by combining clinical urgency, Cedar evidence, tenant scope, and post-event review.

## 3. Scope
- Build the emergency-services bypass request model.
- Build emergency responder attestation validation.
- Bind bypass to `break-glass-authorize`.
- Bind emergency reads to `fhir-read`.
- Bind emergency HL7 routing to `hl7-route`.
- Bind consent conflict handling to `consent-sync`.
- Bind source-system provenance to `ehr-provenance-seal`.
- Bind duplicate/ambiguous patient resolution to `patient-match-review`.
- Emit audit events for allow, deny, degrade, revoke, review-opened, review-closed, and replayed decisions.
- Add operator review workflow hooks to the runbooks and dashboards already cited by this IP.
- Keep all data movements within tenant, home-cell, jurisdiction, and pack constraints.
- Keep the emergency exception narrow enough to be explainable to a regulator.

## 4. Non-Goals
- Do not create a vendor-named service boundary.
- Do not implement a full emergency dispatch product.
- Do not let emergency responders self-register without tenant-approved authority.
- Do not treat emergency status as a permanent principal attribute.
- Do not store raw clinical payloads in audit metrics.
- Do not weaken `policy/emergency-services-bypass.cedar` default-deny requirements.
- Do not bypass `policy/data-residency.md`.
- Do not replace normal consent workflows.
- Do not edit ADR-0321.
- Do not run Oya VCS verify, done, or promote for this authoring pass.

## 5. Domain Model
- `EmergencyBypassRequest` is the command envelope.
- `tenant_id` is required.
- `principal_id` is required.
- `audience_type` must be `EMERGENCY_SERVICES`.
- `emergency_attestation` must be `jurisdiction_registered` or stronger.
- `jurisdiction_code` is required.
- `incident_id` is required.
- `clinical_reason_code` is required.
- `requested_data_classes` must be enumerated.
- `minimum_necessary_profile` maps each reason to allowable resources.
- `patient_match_context` carries local identifiers and confidence evidence.
- `source_system_id` names the EHR, lab, referral, or HIE feed.
- `residency_pack_id` names the active pack overlay.
- `dealset_context_id` is optional until a commercial exchange is created.
- `audit_event_class` must be `AbuseDefenceEmergencyServiceBypass`.
- `expires_at` is required and short-lived.
- `review_due_at` is required.
- `revocation_token` is generated at authorization time.

## 6. Policy Gates
- Gate 1 validates the principal and resource tenant match.
- Gate 2 validates the emergency audience type.
- Gate 3 validates jurisdiction registration.
- Gate 4 validates purpose and clinical reason.
- Gate 5 validates minimum necessary resource class.
- Gate 6 validates home-cell and residency constraints.
- Gate 7 validates consent conflict handling.
- Gate 8 validates bot score and abuse-defence state.
- Gate 9 validates OpenBao sidecar credential mode when a provider credential is required.
- Gate 10 validates audit-chain availability or bounded evidence buffering.
- Gate 11 validates post-event review assignment.
- Gate 12 validates TTL and revocation token generation.
- Gate 13 validates marketplace settlement deferral rules.
- Gate 14 validates replay and appeal packet readiness.
- Gate 15 denies any missing required context before provider access.

## 7. Implementation Steps
- Add application command handler `AuthorizeEmergencyServicesBypass`.
- Keep the handler in the ADR-0105 application/usecase layers.
- Keep policy-port traits in kernel/domain layers.
- Keep source-system adapters in adapter/worker layers.
- Evaluate Cedar before resolving provider credentials.
- Read provider credentials through OpenBao sidecar with short TTL.
- Materialize a signed policy decision id.
- Call patient-match review before returning resources when identity confidence is below tenant threshold.
- Return only the minimum necessary FHIR resources for the reason code.
- For HL7, route only the matching message class and segment projection needed for the incident.
- For consent conflicts, mark emergency override as temporary, reviewable, and reversible.
- Seal every retrieved bundle with ehr provenance before handoff.
- Emit `emergency_bypass.authorized` after policy permit and before provider read.
- Emit `emergency_bypass.denied` on every failed gate.
- Emit `emergency_bypass.degraded` when audit buffering or regional failover alters normal handling.
- Emit `emergency_bypass.revoked` when TTL, operator action, or tenant policy closes the path.
- Emit `emergency_bypass.review_opened` automatically.
- Emit `emergency_bypass.review_closed` only after auditor or tenant reviewer decision.
- Add dashboard dimensions for policy result, reason code, jurisdiction, pack id, source system, and review status.
- Add runbook steps for emergency-services chaos drills.
- Add replay fixture coverage for allow, deny, stale consent, regional outage, and compromised credential scenarios.

## 8. Data Residency and Consent Handling
- Residency packs decide where the emergency read is executed.
- Metadata can cross cells only when the pack allows metadata-only failover.
- PHI payload remains in the tenant home cell unless the active pack permits emergency cross-cell movement.
- Consent denial does not erase emergency authority; it changes evidence and review obligations.
- Consent conflict evidence includes consent source, policy version, override reason, and reviewer queue.
- The bypass may access a consent-restricted record only when policy maps the reason to an emergency exception.
- Every exception requires review by the tenant or delegated compliance reviewer.
- Review outcome can uphold, revoke, narrow, or flag for incident response.
- Revoked events trigger downstream invalidation and export packet regeneration.
- Cross-border emergency routing must cite the active pack overlay.

## 9. Observability and Audit
- Metric `healthcare_emergency_bypass_requests_total` counts attempts by decision and reason bucket.
- Metric `healthcare_emergency_bypass_latency_ms` tracks policy-to-bundle completion.
- Metric `healthcare_emergency_review_overdue_total` tracks missed review deadlines.
- Metric `healthcare_emergency_denial_total` tracks failed gates.
- Metric labels must avoid raw tenant id cardinality.
- Audit payload includes tenant id, principal id, policy decision id, incident id, jurisdiction, data class, and source system.
- Trace spans link policy evaluation, patient match, provider read, provenance seal, and audit-chain write.
- Logs redact PHI and include only stable evidence identifiers.
- Dashboards must show clean path, suspicious path, denial path, and degraded path separately.
- Alerting fires when emergency bypass volume deviates from tenant baseline.
- Alerting fires when review overdue count is non-zero.
- Alerting fires when bot controls deny non-emergency traffic while emergency traffic rises.

## 10. Benchmark Displacement
- Redox displacement: Redox gives integration network reach; this IP adds tenant-owned Cedar decisions, emergency TTL, signed audit replay, and minimum-necessary resource narrowing.
- Rhapsody displacement: Rhapsody routes clinical messages well; this IP adds product-level emergency authority, consent conflict evidence, and first-class post-event review.
- InterSystems IRIS for Health displacement: IRIS provides strong data platform capabilities; this IP keeps emergency access in a flat microservice with explicit policy, provenance, and pack overlays instead of suite-coupled state.
- Lyniate/Corepoint displacement: Corepoint-style interface engines handle interface workflows; this IP binds break-glass to tenant scope, responder attestation, and regulator-ready event trails.
- Mirth Connect displacement: Mirth enables flexible channel scripting; this IP avoids ad hoc channel logic and requires typed commands, Cedar policy, and deterministic replay evidence.
- NextGate displacement: NextGate focuses identity resolution; this IP uses patient-match-review as a gate within a broader emergency authorization chain.
- Health Catalyst displacement: Health Catalyst emphasizes analytics and population data; this IP treats emergency access as live operational control with immediate revocation and audit closure.
- Combined displacement: competitors solve pieces of interoperability, identity, routing, or analytics; this IP combines emergency authority, policy, tenant isolation, residency, provenance, and evidence in one governed path.

## 11. Security and Abuse Controls
- Bot score above the emergency threshold denies non-emergency traffic.
- Emergency traffic still requires jurisdiction registration.
- Suspicious emergency volume creates an incident-response event.
- Principal credential compromise revokes active tokens and replays affected decisions.
- Source-system credential failure degrades to denial, not broad read fallback.
- Audit-chain outage pauses high-risk mutation and buffers bounded read evidence.
- Data exfiltration suspicion narrows the response to identity and allergy-critical resources only.
- Reviewer assignment failure blocks closure and pages compliance operations.
- Policy mismatch rolls back to the last soaked policy fragment.
- Any anonymous audience type is forbidden.

## 12. Rollback
- Roll back by disabling the emergency bypass capability flag at tenant pack level.
- Keep normal `break-glass-authorize` available for tenant-internal operators if separately configured.
- Revoke active emergency tokens.
- Mark all in-flight requests as revoked.
- Emit revocation events.
- Regenerate export packets for affected incidents.
- Re-run audit completeness checks.
- Replay denied and revoked requests through a dry-run worker.
- Notify tenant reviewers for any event that completed before rollback.
- Preserve evidence for regulator review.

## 13. Acceptance Evidence
- The IP cites `policy/emergency-services-bypass.cedar`.
- The IP cites `capabilities/break-glass-authorize.yaml`.
- The IP cites `PRD.md` and `ARCHITECTURE.md`.
- The IP cites emergency and break-glass runbooks.
- The policy denies missing tenant, principal, audience, cell tier, purpose, and data class.
- The policy permits only emergency audience with jurisdiction registration.
- The implementation plan includes responder attestation.
- The implementation plan includes minimum necessary data.
- The implementation plan includes TTL and revocation.
- The implementation plan includes consent conflict treatment.
- The implementation plan includes residency constraints.
- The implementation plan includes audit events for allow, deny, degrade, revoke, review open, and review close.
- The implementation plan includes dashboard and SLO hooks.
- The implementation plan includes all seven named benchmark families.
- The implementation plan keeps ADR-0321 referenced but unmodified.

## 14. Done Criteria
- `AuthorizeEmergencyServicesBypass` has a contract fixture.
- Cedar policy tests cover allow, deny, missing context, suspicious bot, stale consent, and emergency attestation failure.
- OpenAPI examples include an emergency request and a denial packet.
- AsyncAPI examples include authorized, denied, revoked, and review events.
- Proto examples include internal policy decision and revocation shapes.
- Runbook references include chaos drill and audit review.
- Dashboard references include policy decisions and abuse-defence outcomes.
- SLO references include access review latency.
- Evidence packet includes policy id, trace id, audit event id, and review id.
- No other file is required for this IP deepening pass.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-013-emergency-services-bypass.md:55` - - Auditors need to reconstruct why a responder accessed PHI, which tenant authorized the policy, what data classes moved, and who reviewed the event afterward.; `microservices/healthcare-integration/IP-013-emergency-services-bypass.md:149` - - PHI payload remains in the tenant home cell unless the active pack permits emergency cross-cell movement..
