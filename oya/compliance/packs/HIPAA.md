---
doc_class: CompliancePackOverlay
pack_id: HIPAA-2024
microservice: compliance
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# compliance HIPAA Compliance Pack Overlay

## Pack Identity
- Full pack name: HIPAA Administrative Simplification compliance-pack registry overlay.
- Citing jurisdiction: United States federal health information regime.
- Version: HIPAA-2024-v1.
- Canonical source URL: https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C
- Cited law: 45 CFR Parts 160, 162, and 164.
- Covered compliance surface: pack admission, BAA registry, evidence collection, risk assessments, breach workflow, disclosure accounting, retention rules, and attestation exports.
- Pack activation means compliance becomes the source of truth for HIPAA pack admission and downstream service gates.
- The overlay stores PHI policy metadata but does not store raw PHI.
- Data classes include `COMPLIANCE_HIPAA_BAA`, `COMPLIANCE_HIPAA_EVIDENCE`, `COMPLIANCE_HIPAA_BREACH_CASE`, and `COMPLIANCE_HIPAA_DISCLOSURE`.
- Minimum necessary applies to compliance evidence views and support access.
- ADR-0064 keeps compliance base generic while this pack defines HIPAA-specific control rows.
- ADR-0251 supplies pack schema, registry, cell eligibility, and bundle signature.
- ADR-0263 supplies evidence emission and audit linkage rules.
- This overlay excludes PCI-DSS because PCI has its own payment-specific compliance pack where card data flows.
- HIPAA and PCI may both be active for one tenant, but this document governs only HIPAA compliance metadata.

## Data Model Deltas
- Add `pack_admission.hipaa_baa_status` as enum `missing|pending|verified|revoked`.
- Add `pack_admission.hipaa_baa_document_hash`.
- Add `pack_admission.hipaa_covered_entity_type`.
- Add `pack_admission.hipaa_business_associate_scope`.
- Add `pack_admission.hipaa_cell_certification_required` default `hipaa-certified`.
- Add `risk_assessment.hipaa_security_rule_review_id`.
- Add `risk_assessment.hipaa_privacy_rule_review_id`.
- Add `risk_assessment.hipaa_breach_rule_review_id`.
- Add `evidence_item.hipaa_control_id`.
- Add `evidence_item.phi_free_evidence_hash`.
- Add `evidence_item.minimum_necessary_scope`.
- Add `disclosure_accounting.disclosure_id`.
- Add `disclosure_accounting.recipient_org_hash`.
- Add `disclosure_accounting.patient_context_id_hash`.
- Add `disclosure_accounting.purpose`.
- Add `breach_case.hipaa_clock_started_at`.
- Add `breach_case.hipaa_notification_due_at`.
- Add `breach_case.risk_assessment_outcome`.
- Add `retention_rule.hipaa_floor_iso8601` default `P6Y`.
- Add `training_record.hipaa_workforce_training_version`.
- Add `sanction_record.hipaa_policy_violation_id`.
- Add `audit_shadow.compliance_hipaa_event_id`.
- Add `tenant_compliance_config.hipaa_pack_version`.
- Add `tenant_compliance_config.hipaa_privacy_officer_ref`.

## Cedar Policy Deltas
- Policy `HIPAA-compliance-admission-01`: permit pack activation only when BAA status is verified.
- Policy `HIPAA-compliance-admission-02`: require HIPAA-certified cell before activation.
- Policy `HIPAA-compliance-baa-01`: forbid downstream BAA proof read by non-compliance roles.
- Policy `HIPAA-compliance-evidence-01`: permit PHI-free evidence upload only when control id exists.
- Policy `HIPAA-compliance-evidence-02`: forbid raw PHI in evidence item payload.
- Policy `HIPAA-compliance-disclosure-01`: permit disclosure accounting read by privacy officer.
- Policy `HIPAA-compliance-disclosure-02`: require purpose for each external disclosure row.
- Policy `HIPAA-compliance-risk-01`: require annual security rule review.
- Policy `HIPAA-compliance-risk-02`: require privacy rule review before new PHI service activation.
- Policy `HIPAA-compliance-breach-01`: start breach case when downstream candidate confirmed.
- Policy `HIPAA-compliance-breach-02`: require risk assessment outcome before notification close.
- Policy `HIPAA-compliance-retention-01`: forbid evidence purge before six-year floor.
- Policy `HIPAA-compliance-training-01`: require workforce training before covered role grant.
- Policy `HIPAA-compliance-sanction-01`: require sanction record for confirmed policy violation.
- Policy `HIPAA-compliance-export-01`: require privacy-office approval for HIPAA evidence export.
- Policy `HIPAA-compliance-export-02`: require redaction manifest for auditor export.
- Policy `HIPAA-compliance-admin-01`: require elevated ACR for compliance case mutation.
- Policy `HIPAA-compliance-support-01`: require break-glass reason for support case view.
- Policy `HIPAA-compliance-webhook-01`: require BAA proof for external evidence sink.
- Policy `HIPAA-compliance-pack-01`: defer deactivation while HIPAA evidence is retained.
- Policy `HIPAA-compliance-revoke-01`: revoke dependent service pack gates when BAA is revoked.
- Policy `HIPAA-compliance-route-01`: require HIPAA-certified storage for evidence bundles.
- Policy `HIPAA-compliance-audit-01`: require audit seal for every admission and breach transition.
- Policy `HIPAA-compliance-minimum-01`: restrict evidence fields by minimum necessary scope.

## API Contract Deltas
- `POST /packs/HIPAA-2024/admit` requires verified BAA document hash.
- `POST /packs/HIPAA-2024/admit` requires HIPAA-certified cell proof.
- `GET /packs/HIPAA-2024/status` returns BAA and admission state.
- `POST /baa` stores document hash, not raw legal document body.
- `PATCH /baa/{id}/revoke` emits dependent pack revocation event.
- `POST /evidence` requires `hipaa_control_id`.
- `POST /evidence` rejects raw PHI payload markers.
- `GET /evidence/{id}` filters fields by minimum necessary scope.
- `POST /disclosures` requires purpose and recipient organization hash.
- `GET /disclosures` requires privacy officer role.
- `POST /risk-assessments/security-rule` records annual review.
- `POST /breach-cases` starts HIPAA breach workflow.
- `PATCH /breach-cases/{id}/risk-outcome` records assessment outcome.
- `POST /training-records` records workforce HIPAA training.
- `POST /sanctions` records confirmed policy violation response.
- `POST /exports/auditor` requires redaction manifest.
- `DELETE /evidence/{id}` returns retention conflict before six-year floor.
- `PATCH /tenant-compliance-config` records privacy officer ref.
- `POST /support/break-glass` requires reason id.
- `POST /pack/deactivate` returns retained evidence count.

## Workflow Deltas
- Pack admission workflow verifies BAA and cell certification.
- BAA revocation workflow revokes dependent service gates.
- Security rule review workflow runs annually.
- Privacy rule review workflow runs before new PHI service activation.
- Evidence ingestion workflow rejects raw PHI.
- Disclosure accounting workflow stores recipient and purpose hashes.
- Breach candidate workflow promotes confirmed candidates to HIPAA case.
- Breach notification workflow tracks 60-day outer deadline.
- Breach risk assessment workflow records low-probability determination.
- Workforce training workflow gates covered workforce roles.
- Sanction workflow records policy violation response.
- Auditor export workflow builds redacted evidence manifest.
- Evidence retention workflow blocks purge before six-year floor.
- Support break-glass workflow grants short case view.
- Webhook setup workflow validates BAA destination.
- Storage migration workflow validates HIPAA-certified evidence cell.
- Pack deactivation waits for retained evidence inventory.
- Dependent service sync workflow publishes admission state to pack-aware services.
- Audit bundle workflow seals every admission and breach transition.
- Minimum necessary workflow filters evidence views.

## SLO Deltas
- HIPAA pack admission decision p99 target is <= 15 minutes after complete evidence.
- BAA status lookup p99 must stay <= 200 ms.
- Dependent service gate propagation p99 target is <= 5 minutes.
- Evidence PHI rejection p99 must stay <= 500 ms.
- Disclosure accounting write p99 must complete <= 1 second.
- Breach case creation p99 target is <= 5 minutes.
- HIPAA notification workflow supports 60-day outer deadline.
- Privacy officer internal notification p99 target is <= 24 hours.
- Annual risk review reminder target is 30 days before due date.
- Training gate propagation p99 target is <= 15 minutes.
- Evidence export manifest p99 target is <= 30 minutes.
- Retention conflict response p99 must stay <= 300 ms.
- Support break-glass start p99 target is <= 2 minutes.
- BAA revocation propagation p99 target is <= 5 minutes.
- HIPAA compliance dashboard lag target is <= 15 minutes.
- Evidence integrity verification cadence is daily.

## Audit-event class additions
- `ComplianceHipaaPackAdmissionStarted` records tenant id and pack version.
- `ComplianceHipaaPackAdmissionApproved` records BAA hash and cell proof.
- `ComplianceHipaaPackAdmissionRejected` records reason.
- `ComplianceHipaaBaaVerified` records document hash.
- `ComplianceHipaaBaaRevoked` records revocation reason.
- `ComplianceHipaaEvidenceAccepted` records control id.
- `ComplianceHipaaEvidenceRejectedPhi` records detector verdict.
- `ComplianceHipaaDisclosureRecorded` records recipient hash.
- `ComplianceHipaaSecurityRuleReviewed` records review id.
- `ComplianceHipaaPrivacyRuleReviewed` records review id.
- `ComplianceHipaaBreachCaseStarted` records candidate id.
- `ComplianceHipaaRiskAssessmentCompleted` records outcome.
- `ComplianceHipaaTrainingRecorded` records training version.
- `ComplianceHipaaSanctionRecorded` records violation id.
- `ComplianceHipaaAuditorExportCreated` records manifest hash.
- `ComplianceHipaaEvidencePurgeRefused` records retention floor.
- `ComplianceHipaaSupportBreakGlassStarted` records reason id.
- `ComplianceHipaaDependentGateRevoked` records service id.
- `ComplianceHipaaStorageRouteBlocked` records target cell.
- `ComplianceHipaaPackDeactivationDeferred` records retained count.

## Failure Modes specific to this pack
- BAA document hash missing; recovery is reject pack admission.
- HIPAA cell proof missing; recovery is block activation.
- BAA revoked but dependent gates remain active; recovery is force revoke and page compliance.
- Evidence upload contains raw PHI; recovery is reject and quarantine.
- Disclosure accounting misses recipient purpose; recovery is reject row.
- Breach candidate not promoted; recovery is retroactive case creation.
- Risk assessment outcome missing near deadline; recovery is page privacy officer.
- Workforce training expires; recovery is revoke covered role.
- Auditor export includes raw PHI; recovery is revoke and rebuild redacted export.
- Evidence retention purge requested early; recovery is refuse purge.
- Support break-glass lacks reason; recovery is deny case view.
- External evidence sink lacks BAA; recovery is disable webhook.
- Annual risk review overdue; recovery is block new PHI service activation.
- Privacy officer ref missing; recovery is reject tenant config.
- Evidence hash mismatch appears; recovery is rebuild from audit-chain.
- Storage route chooses uncertified cell; recovery is block migration.
- Pack deactivation requested with retained evidence; recovery is defer.
- Minimum necessary scope too broad; recovery is restrict fields and open review.
- Audit-chain backpressure appears; recovery is fail-closed for admission and breach transitions.
- Dependent service state drift appears; recovery is republish signed pack state.

## Cross-µservice coordination
- `tenancy` enforces HIPAA pack activation and cell placement.
- `identity` provides privacy officer and covered workforce roles.
- `audit-chain` seals admission, evidence, disclosure, and breach events.
- `observability` emits PHI-free compliance SLO evidence.
- `policy-engine` loads all `HIPAA-compliance-*` fragments.
- `workflow-engine` runs admission, breach, training, and export workflows.
- `mail` consumes BAA status and disclosure accounting rules.
- `drive` consumes BAA status and retention rules.
- `calendar` consumes BAA status and room certification rules.
- `records` owns patient context references.
- `incident-response` sends confirmed PHI candidates to compliance.
- `admin-console` renders HIPAA admission state.
- `legal` supplies BAA templates and review status.
- `support` uses break-glass case view.
- `data-warehouse` receives aggregate compliance metrics only.
- `notification` routes privacy officer deadlines.
- `storage` provides HIPAA-certified evidence backend proof.
- `release-engine` gates new PHI services on privacy review.
- `dlp-virus-scan` screens evidence uploads for raw PHI.
- `pack-registry` signs this HIPAA compliance overlay.
