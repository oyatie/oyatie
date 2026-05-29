---
doc_class: CompliancePackOverlay
pack_id: EU-GDPR-2018-baseline
microservice: compliance
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# compliance GDPR Compliance Pack Overlay

## Pack Identity
- Full pack name: EU GDPR compliance-pack registry and rights overlay.
- Citing jurisdiction: European Union and EEA personal-data regime.
- Version: EU-GDPR-2018-baseline-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2016/679/oj
- Cited law: Regulation (EU) 2016/679.
- Covered compliance surface: processing register, lawful basis, DPA registry, DPIA, DSAR, erasure, restriction, portability, transfer mechanisms, and breach workflow.
- Pack activation means compliance becomes the canonical controller and processor evidence hub for GDPR obligations.
- The overlay stores evidence, hashes, and case metadata, not raw personal data payloads by default.
- Data classes include `COMPLIANCE_GDPR_PROCESSING_RECORD`, `COMPLIANCE_GDPR_DSAR_CASE`, `COMPLIANCE_GDPR_DPIA`, and `COMPLIANCE_GDPR_BREACH_CASE`.
- Article 33 breach readiness is a primary SLO and deadline driver.
- ADR-0064 keeps GDPR obligations in a pack overlay.
- ADR-0251 supplies pack schema, admission, breach workflow, and regulator citations.
- ADR-0263 supplies evidence emission and audit linkage.
- PCI-DSS is omitted because cardholder-data compliance is activated only where payment data flows.
- GDPR and PCI may both be active, but this document governs personal-data obligations.

## Data Model Deltas
- Add `processing_record.processing_purpose_id`.
- Add `processing_record.lawful_basis`.
- Add `processing_record.article_9_condition`.
- Add `processing_record.data_subject_category`.
- Add `processing_record.recipient_category`.
- Add `processing_record.retention_schedule_version`.
- Add `processing_record.transfer_mechanism`.
- Add `processing_record.dpa_ref`.
- Add `processing_record.controller_processor_role`.
- Add `dpia.dpia_id`.
- Add `dpia.risk_outcome`.
- Add `dpia.approved_by_dpo_at`.
- Add `dsar.case_id`.
- Add `dsar.request_type` as enum `access|erasure|restriction|portability|rectification|objection`.
- Add `dsar.subject_hash`.
- Add `dsar.deadline_at`.
- Add `dsar.extension_reason`.
- Add `breach_case.article33_clock_started_at`.
- Add `breach_case.regulator_notification_due_at`.
- Add `breach_case.data_subject_notice_required`.
- Add `transfer_record.scc_module`.
- Add `audit_shadow.compliance_gdpr_event_id`.
- Add `tenant_compliance_config.eu_dpa_version`.
- Add `tenant_compliance_config.dpo_contact_ref`.

## Cedar Policy Deltas
- Policy `GDPR-compliance-admission-01`: require DPO contact before GDPR pack activation.
- Policy `GDPR-compliance-processing-01`: require lawful basis for processing record.
- Policy `GDPR-compliance-processing-02`: require Article 9 condition for special-category data.
- Policy `GDPR-compliance-dpa-01`: require DPA ref for processor relationship.
- Policy `GDPR-compliance-dpia-01`: require DPIA for high-risk processing.
- Policy `GDPR-compliance-dpia-02`: forbid high-risk launch until DPO approval.
- Policy `GDPR-compliance-dsar-01`: permit DSAR case creation for verified subject.
- Policy `GDPR-compliance-dsar-02`: restrict DSAR case read to DPO and assigned processor.
- Policy `GDPR-compliance-erasure-01`: require legal hold conflict review before refusal.
- Policy `GDPR-compliance-portability-01`: require machine-readable export manifest.
- Policy `GDPR-compliance-transfer-01`: require transfer mechanism for non-EEA recipient.
- Policy `GDPR-compliance-transfer-02`: forbid expired SCC mechanism.
- Policy `GDPR-compliance-breach-01`: start Article 33 clock when candidate is confirmed.
- Policy `GDPR-compliance-breach-02`: require regulator notification decision before close.
- Policy `GDPR-compliance-retention-01`: forbid indefinite retention without legal basis.
- Policy `GDPR-compliance-export-01`: require DPO approval for evidence export.
- Policy `GDPR-compliance-admin-01`: require elevated ACR for case mutation.
- Policy `GDPR-compliance-support-01`: require DPO-visible case view for support.
- Policy `GDPR-compliance-webhook-01`: require DPA for external evidence sink.
- Policy `GDPR-compliance-objection-01`: propagate objection to dependent services.
- Policy `GDPR-compliance-consent-01`: require consent snapshot when basis is consent.
- Policy `GDPR-compliance-index-01`: require erasure propagation evidence before DSAR close.
- Policy `GDPR-compliance-pack-01`: defer deactivation while DSAR or breach cases are open.
- Policy `GDPR-compliance-audit-01`: require audit seal for every case transition.

## API Contract Deltas
- `POST /packs/EU-GDPR-2018-baseline/admit` requires DPO contact and DPA version.
- `POST /processing-records` requires lawful basis and purpose id.
- `POST /processing-records` requires Article 9 condition for special-category data.
- `POST /dpia` records high-risk processing assessment.
- `PATCH /dpia/{id}/approve` requires DPO role.
- `POST /dsar` requires verified subject identity.
- `PATCH /dsar/{id}/extend` requires extension reason.
- `PATCH /dsar/{id}/close` requires propagation evidence.
- `POST /transfers` requires mechanism and recipient category.
- `PATCH /transfers/{id}` validates SCC expiry.
- `POST /breach-cases` starts Article 33 clock.
- `PATCH /breach-cases/{id}/notification-decision` records regulator decision.
- `POST /evidence` requires GDPR control or case id.
- `POST /exports/auditor` requires DPO approval and redaction manifest.
- `POST /objections` propagates processing objection.
- `POST /consent-snapshots` stores consent text hash.
- `GET /deadlines` returns DSAR and breach deadlines.
- `PATCH /tenant-compliance-config` records DPO contact.
- `POST /support/case-view` requires DPO-visible reason.
- `POST /pack/deactivate` refuses open DSAR or breach cases.

## Workflow Deltas
- Pack admission workflow verifies DPO contact and DPA version.
- Processing-register workflow records purpose, lawful basis, and retention.
- DPIA workflow routes high-risk processing to DPO approval.
- DSAR workflow verifies subject and starts statutory timer.
- DSAR export workflow collects downstream service manifests.
- Erasure workflow waits for propagation evidence from services.
- Restriction workflow sends dependent service restrictions.
- Portability workflow assembles machine-readable manifests.
- Objection workflow propagates stop-processing signals.
- Consent workflow stores consent and withdrawal evidence.
- Transfer workflow validates SCC, adequacy, or derogation.
- Breach workflow starts Article 33 regulator deadline.
- Breach decision workflow records notification and subject notice decisions.
- Evidence export workflow redacts personal data.
- Retention workflow rejects indefinite retention without basis.
- Support case workflow creates DPO-visible view.
- External webhook workflow validates processor DPA.
- Pack deactivation waits for open DSAR, DPIA, transfer, and breach cases.
- Audit bundle workflow seals every case transition.
- Deadline monitor pages DPO before statutory deadlines.

## SLO Deltas
- GDPR breach regulator-readiness p99 target is <= 60 hours.
- Article 33 clock creation p99 target is <= 5 minutes.
- DSAR case creation p99 target is <= 5 minutes after verification.
- DSAR first response target is <= 7 days.
- DSAR completion target is <= 30 days unless extension is recorded.
- Erasure propagation evidence collection target is <= 72 hours after approval.
- Objection propagation p99 target is <= 30 minutes.
- DPA lookup p99 must stay <= 200 ms.
- DPIA DPO routing p99 target is <= 2 minutes.
- Transfer mechanism validation p99 must stay <= 200 ms.
- Consent snapshot write p99 must stay <= 300 ms.
- Deadline dashboard lag target is <= 5 minutes.
- Evidence export manifest p99 target is <= 30 minutes.
- Support case view audit p99 must complete <= 1 second.
- Retention conflict response p99 must stay <= 300 ms.
- Processing-register freshness target is <= 24 hours.

## Audit-event class additions
- `ComplianceGdprPackAdmissionStarted` records tenant id and pack version.
- `ComplianceGdprPackAdmissionApproved` records DPO and DPA refs.
- `ComplianceGdprProcessingRecordCreated` records purpose and basis.
- `ComplianceGdprDpiaStarted` records assessment id.
- `ComplianceGdprDpiaApproved` records DPO id.
- `ComplianceGdprDsarCreated` records request type.
- `ComplianceGdprDsarExtended` records reason.
- `ComplianceGdprDsarClosed` records propagation manifest.
- `ComplianceGdprTransferMechanismRecorded` records mechanism.
- `ComplianceGdprTransferExpired` records transfer id.
- `ComplianceGdprBreachClockStarted` records candidate id.
- `ComplianceGdprBreachNotificationDecisionRecorded` records decision.
- `ComplianceGdprConsentSnapshotStored` records consent hash.
- `ComplianceGdprObjectionPropagated` records service count.
- `ComplianceGdprEvidenceAccepted` records case id.
- `ComplianceGdprAuditorExportCreated` records manifest hash.
- `ComplianceGdprDeadlineWarningIssued` records deadline id.
- `ComplianceGdprSupportCaseViewed` records reason id.
- `ComplianceGdprRetentionRejected` records schedule id.
- `ComplianceGdprPackDeactivationDeferred` records open cases.

## Failure Modes specific to this pack
- DPO contact missing; recovery is reject pack admission.
- Lawful basis missing; recovery is reject processing record.
- Article 9 condition missing; recovery is block special-category processing.
- DPIA approval missing; recovery is block high-risk launch.
- DSAR subject verification fails; recovery is deny case creation.
- DSAR propagation evidence incomplete; recovery is keep case open and page owner.
- Erasure conflicts with legal hold; recovery is restrict processing and record refusal basis.
- SCC expires; recovery is block transfer and notify tenant.
- Breach clock not started; recovery is retroactive event and page DPO.
- Regulator notification decision missing near deadline; recovery is escalate.
- Consent snapshot hash mismatch; recovery is disable consent-based processing.
- Objection propagation fails; recovery is retry and block downstream processing.
- Auditor export contains personal data beyond scope; recovery is revoke and rebuild.
- Retention schedule indefinite; recovery is reject schedule.
- Support view lacks DPO-visible reason; recovery is deny view.
- Webhook lacks DPA; recovery is disable sink.
- Pack deactivation requested with open cases; recovery is defer.
- Deadline monitor lags; recovery is page compliance on stale dashboard.
- Audit-chain backpressure appears; recovery is fail-closed for case transitions.
- Processing register drift found; recovery is reconcile from service manifests.

## Cross-µservice coordination
- `tenancy` enforces GDPR pack activation and EU cell placement.
- `identity` verifies subject identity and DPO roles.
- `audit-chain` seals processing, DSAR, transfer, and breach events.
- `observability` emits personal-data-safe compliance evidence.
- `policy-engine` loads all `GDPR-compliance-*` fragments.
- `workflow-engine` runs DSAR, erasure, restriction, breach, and DPIA workflows.
- `mail` returns DSAR and erasure propagation manifests.
- `drive` returns DSAR and erasure propagation manifests.
- `calendar` returns DSAR and erasure propagation manifests.
- `incident-response` sends confirmed personal-data candidates to compliance.
- `admin-console` renders GDPR processing and deadline state.
- `legal` supplies DPA, SCC, and DPIA review content.
- `support` uses DPO-visible case view.
- `data-warehouse` receives aggregate compliance metrics only.
- `notification` routes DPO deadline warnings.
- `localization` provides EU privacy notices.
- `connector` validates processor DPA references.
- `search` confirms erasure propagation where it indexes services.
- `release-engine` gates high-risk processing on DPIA.
- `pack-registry` signs this GDPR compliance overlay.
