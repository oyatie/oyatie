---
doc_class: CompliancePackOverlay
pack_id: KR-PIPA-2023-amendment
microservice: compliance
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# compliance KR-PIPA Compliance Pack Overlay

## Pack Identity
- Full pack name: Korea Personal Information Protection Act compliance registry overlay.
- Citing jurisdiction: Republic of Korea personal information regime.
- Version: KR-PIPA-2023-amendment-v1.
- Canonical source URL: https://law.go.kr/LSW/lsInfoP.do?lsId=011357
- Cited law: 개인정보 보호법, Act No. 17799 baseline with current consolidation at law.go.kr.
- Covered compliance surface: 동의 ledger, 보존 ledger, 국외이전 records, 처리위탁 registry, subject-rights cases, breach workflow, Korean notices, and DPO evidence.
- Pack activation means compliance is the canonical source for Korean privacy evidence and downstream service gates.
- The overlay stores evidence, hashes, and case metadata, not raw Korean personal information by default.
- Data classes include `COMPLIANCE_KR_PIPA_CONSENT`, `COMPLIANCE_KR_PIPA_RETENTION`, `COMPLIANCE_KR_PIPA_TRANSFER`, and `COMPLIANCE_KR_PIPA_BREACH`.
- Korean-language notice versions are mandatory for subject-facing workflows.
- ADR-0064 keeps Korean privacy behavior in the pack overlay.
- ADR-0251 supplies pack schema, retention, and breach workflow.
- ADR-0263 supplies evidence emission and audit linkage.
- PCI-DSS is omitted because payment compliance is separate where card data flows.
- KR-PIPA may coexist with KR financial packs, but this overlay governs personal information.

## Data Model Deltas
- Add `consent.kr_consent_id`.
- Add `consent.kr_consent_text_hash`.
- Add `consent.kr_consent_captured_at`.
- Add `consent.kr_consent_withdrawn_at`.
- Add `consent.kr_processing_purpose_id`.
- Add `retention.kr_retention_basis_id`.
- Add `retention.kr_retention_until`.
- Add `retention.kr_retention_notice_version`.
- Add `transfer.kr_cross_border_transfer_id`.
- Add `transfer.kr_transfer_notice_hash`.
- Add `transfer.recipient_country_code`.
- Add `processor.kr_processor_delegation_id`.
- Add `processor.delegation_notice_hash`.
- Add `subject_rights.kr_case_id`.
- Add `subject_rights.subject_hash`.
- Add `subject_rights.deadline_at`.
- Add `breach_case.kr_breach_clock_started_at`.
- Add `breach_case.pipc_notification_due_at`.
- Add `breach_case.data_subject_notice_required`.
- Add `rrn_control.rrn_processing_approval_id`.
- Add `notice.kr_pipa_notice_version`.
- Add `audit_shadow.compliance_kr_pipa_event_id`.
- Add `tenant_compliance_config.kr_dpo_contact_ref`.
- Add `tenant_compliance_config.kr_pack_version`.

## Cedar Policy Deltas
- Policy `KRPIPA-compliance-admission-01`: require KR DPO contact before pack activation.
- Policy `KRPIPA-compliance-consent-01`: require 동의 text hash for consent record.
- Policy `KRPIPA-compliance-consent-02`: forbid consent-based processing after withdrawal.
- Policy `KRPIPA-compliance-retention-01`: require 보존 basis for retained data.
- Policy `KRPIPA-compliance-retention-02`: forbid retention past expiry without legal basis.
- Policy `KRPIPA-compliance-transfer-01`: require 국외이전 notice for cross-border transfer.
- Policy `KRPIPA-compliance-transfer-02`: forbid expired transfer notice.
- Policy `KRPIPA-compliance-processor-01`: require 처리위탁 registry for delegated processor.
- Policy `KRPIPA-compliance-processor-02`: require Korean processor notice hash.
- Policy `KRPIPA-compliance-subject-01`: permit subject-rights case creation for verified subject.
- Policy `KRPIPA-compliance-subject-02`: restrict case read to KR DPO and assignee.
- Policy `KRPIPA-compliance-erasure-01`: require retention conflict review before refusal.
- Policy `KRPIPA-compliance-breach-01`: start KR breach workflow when candidate is confirmed.
- Policy `KRPIPA-compliance-breach-02`: require PIPC notification decision before close.
- Policy `KRPIPA-compliance-rrn-01`: require approval id for RRN processing.
- Policy `KRPIPA-compliance-notice-01`: require Korean notice version for subject-facing action.
- Policy `KRPIPA-compliance-export-01`: require DPO approval for evidence export.
- Policy `KRPIPA-compliance-admin-01`: require elevated ACR for case mutation.
- Policy `KRPIPA-compliance-support-01`: require DPO-visible case view for support.
- Policy `KRPIPA-compliance-webhook-01`: require processor delegation for evidence sink.
- Policy `KRPIPA-compliance-route-01`: require KR cell for Korean resident evidence.
- Policy `KRPIPA-compliance-audit-01`: require audit seal for every ledger transition.
- Policy `KRPIPA-compliance-pack-01`: defer deactivation while ledgers or cases are open.
- Policy `KRPIPA-compliance-minimum-01`: restrict evidence fields by purpose and role.

## API Contract Deltas
- `POST /packs/KR-PIPA-2023-amendment/admit` requires KR DPO contact.
- `POST /consent` requires Korean notice text hash.
- `POST /consent/{id}/withdraw` records withdrawal timestamp.
- `POST /retention` requires 보존 basis id and notice version.
- `POST /transfers` requires 국외이전 notice hash.
- `PATCH /transfers/{id}` validates notice expiry.
- `POST /processors` requires 처리위탁 notice hash.
- `POST /subject-rights` requires verified subject identity.
- `PATCH /subject-rights/{id}/close` requires propagation evidence.
- `POST /rrn-approvals` records RRN processing approval.
- `POST /breach-cases` starts KR breach workflow.
- `PATCH /breach-cases/{id}/notification-decision` records PIPC decision.
- `POST /notices` records Korean notice version.
- `POST /evidence` requires KR ledger or case id.
- `POST /exports/auditor` requires DPO approval and redaction manifest.
- `GET /deadlines/kr-pipa` returns subject-rights and breach deadlines.
- `PATCH /tenant-compliance-config` records KR DPO contact.
- `POST /support/case-view` requires DPO-visible reason.
- `DELETE /evidence/{id}` returns retention conflict.
- `POST /pack/deactivate` refuses open ledgers or cases.

## Workflow Deltas
- Pack admission workflow verifies KR DPO contact and notice baseline.
- Consent workflow stores 동의 text hash and timestamp.
- Consent withdrawal workflow propagates stop-processing signals.
- Retention workflow records 보존 basis and expiry.
- Transfer workflow records 국외이전 notice and recipient country.
- Processor delegation workflow records 처리위탁 notice and processor scope.
- Subject-rights workflow verifies identity and starts deadline.
- Subject-rights export workflow collects service manifests.
- Erasure workflow waits for propagation evidence.
- RRN approval workflow gates resident-registration-number processing.
- Breach workflow starts Korean notification timeline.
- Breach decision workflow records PIPC and subject notice decisions.
- Notice workflow publishes Korean-language notice version.
- Evidence export workflow redacts Korean PI.
- Support case workflow creates DPO-visible view.
- Webhook workflow validates processor delegation.
- KR cell workflow validates evidence storage location.
- Pack deactivation waits for consent, retention, transfer, processor, and breach ledgers.
- Audit bundle workflow seals every ledger transition.
- Deadline monitor pages KR DPO before due time.

## SLO Deltas
- KR breach workflow creation p99 target is <= 5 minutes.
- Korean DPO notification p99 target is <= 24 hours for confirmed leak.
- Consent ledger write p99 must stay <= 300 ms.
- Consent withdrawal propagation p99 target is <= 30 minutes.
- Retention ledger write p99 must stay <= 500 ms.
- Transfer notice validation p99 must stay <= 200 ms.
- Processor delegation lookup p99 must stay <= 200 ms.
- Subject-rights case creation p99 target is <= 5 minutes after verification.
- Korean subject-rights completion internal target is <= 10 days.
- Erasure propagation evidence target is <= 72 hours after approval.
- RRN approval lookup p99 must stay <= 200 ms.
- Korean notice retrieval p99 must stay <= 150 ms.
- Deadline dashboard lag target is <= 5 minutes.
- Evidence export manifest p99 target is <= 30 minutes.
- Support case view audit p99 must complete <= 1 second.
- Korean compliance dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `ComplianceKrPipaPackAdmissionStarted` records tenant id and version.
- `ComplianceKrPipaPackAdmissionApproved` records DPO ref.
- `ComplianceKrPipaConsentCaptured` records consent id and text hash.
- `ComplianceKrPipaConsentWithdrawn` records consent id.
- `ComplianceKrPipaRetentionLedgerWritten` records 보존 basis.
- `ComplianceKrPipaTransferNoticeRecorded` records transfer id.
- `ComplianceKrPipaProcessorDelegationRecorded` records delegation id.
- `ComplianceKrPipaSubjectRightsCaseCreated` records request type.
- `ComplianceKrPipaSubjectRightsCaseClosed` records propagation manifest.
- `ComplianceKrPipaRrnApprovalRecorded` records approval id.
- `ComplianceKrPipaBreachWorkflowStarted` records candidate id.
- `ComplianceKrPipaNotificationDecisionRecorded` records decision.
- `ComplianceKrPipaNoticeVersionPublished` records notice version.
- `ComplianceKrPipaEvidenceAccepted` records ledger id.
- `ComplianceKrPipaAuditorExportCreated` records manifest hash.
- `ComplianceKrPipaDeadlineWarningIssued` records deadline id.
- `ComplianceKrPipaSupportCaseViewed` records reason id.
- `ComplianceKrPipaStorageRouteBlocked` records target cell.
- `ComplianceKrPipaRetentionRejected` records retention id.
- `ComplianceKrPipaPackDeactivationDeferred` records open ledger count.

## Failure Modes specific to this pack
- KR DPO contact missing; recovery is reject pack admission.
- Consent text hash missing; recovery is reject consent record.
- Consent withdrawal propagation fails; recovery is retry and block downstream processing.
- Retention basis missing; recovery is reject retention record.
- Transfer notice expires; recovery is block cross-border transfer.
- Processor delegation registry missing; recovery is block delegated processing.
- Subject identity verification fails; recovery is deny case creation.
- Erasure propagation evidence incomplete; recovery is keep case open.
- RRN approval missing; recovery is block RRN processing.
- Breach workflow clock fails to start; recovery is retroactive event and page DPO.
- PIPC notification decision missing near deadline; recovery is escalate.
- Korean notice unavailable; recovery is fail-closed for subject-facing workflows.
- Evidence export contains Korean PI beyond scope; recovery is revoke and rebuild.
- Support view lacks DPO-visible reason; recovery is deny view.
- Webhook lacks processor delegation; recovery is disable sink.
- KR cell route unavailable; recovery is buffer or reject evidence writes.
- Pack deactivation requested with open ledgers; recovery is defer.
- Deadline monitor lags; recovery is page compliance owner.
- Audit-chain backpressure appears; recovery is fail-closed for ledger transitions.
- Service manifest drift found; recovery is reconcile from downstream signed manifests.

## Cross-µservice coordination
- `tenancy` enforces KR-PIPA pack activation and KR cell placement.
- `identity` verifies subject identity and KR DPO roles.
- `audit-chain` seals consent, retention, transfer, processor, and breach events.
- `observability` emits Korean PI-safe compliance evidence.
- `policy-engine` loads all `KRPIPA-compliance-*` fragments.
- `workflow-engine` runs consent, subject-rights, erasure, and breach workflows.
- `mail` consumes consent, retention, transfer, and processor ledgers.
- `drive` consumes consent, retention, transfer, and processor ledgers.
- `calendar` consumes consent, retention, transfer, and processor ledgers.
- `incident-response` sends confirmed Korean PI candidates to compliance.
- `admin-console` renders Korean compliance state.
- `legal` supplies notice text and processor delegation language.
- `support` uses DPO-visible case view.
- `data-warehouse` receives aggregate compliance metrics only.
- `notification` routes Korean DPO deadline warnings.
- `localization` provides Korean notice text.
- `connector` validates delegated processor references.
- `search` confirms erasure propagation for indexed services.
- `release-engine` gates new Korean PI processing on ledger availability.
- `pack-registry` signs this KR-PIPA compliance overlay.
