---
doc_class: CompliancePackOverlay
pack_id: KR-PIPA-2023-amendment
microservice: mail
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# mail KR-PIPA Compliance Pack Overlay

## Pack Identity
- Full pack name: Korea Personal Information Protection Act mail overlay.
- Citing jurisdiction: Republic of Korea personal information regime.
- Version: KR-PIPA-2023-amendment-v1.
- Canonical source URL: https://law.go.kr/LSW/lsInfoP.do?lsId=011357
- Cited law: 개인정보 보호법, Act No. 17799 baseline with current consolidation at law.go.kr.
- Covered mail surface: Korean tenant mailboxes, Korean-language notices, recipient consent, resident-registration-number detection, attachments, search, retention, breach workflow, and cross-border transfer.
- Pack activation means mail captures 동의 (consent), 보존 (retention), 국외이전 (cross-border transfer), and 처리위탁 (processor delegation) evidence.
- This overlay treats Korean resident registration numbers and sensitive information as high-risk fields.
- Data classes include `MAIL_PI_KR`, `MAIL_SENSITIVE_PI_KR`, `MAIL_RRN_KR`, and `MAIL_CONSENT_LEDGER_KR`.
- Korean-language privacy notices are required for user-facing consent and breach flows.
- ADR-0064 keeps Korean semantics in the pack overlay while preserving canonical mail base.
- ADR-0251 supplies pack admission, retention, cell, and breach workflow hooks.
- ADR-0263 requires tenant-aware PII scrubbing before observability storage.
- The overlay excludes PCI-DSS because mail does not process card authorization.
- Payment references in mail are tokenized and routed to payments if PCI scope is detected.

## Data Model Deltas
- Add `message.kr_pi_signal` as enum `none|personal|sensitive|rrn`.
- Add `message.kr_consent_id` for 동의 ledger linkage.
- Add `message.kr_processing_purpose_id` for stated processing purpose.
- Add `message.kr_retention_basis_id` for 보존 proof.
- Add `message.kr_retention_until` timestamp.
- Add `message.kr_cross_border_transfer_id` for 국외이전 proof.
- Add `message.kr_processor_delegation_id` for 처리위탁 proof.
- Add `message.kr_notice_language` default `ko-KR`.
- Add `message.kr_subject_rights_case_id` nullable.
- Add `message.kr_breach_clock_started_at` nullable.
- Add `attachment.kr_rrn_detected` boolean.
- Add `attachment.kr_sensitive_pi_verdict` as enum `clear|possible|confirmed|scan_failed`.
- Add `mailbox.kr_local_residency_cell` copied from tenancy.
- Add `mailbox.kr_dpo_contact_ref`.
- Add `mailbox.kr_minor_guardian_consent_required` boolean.
- Add `mailbox.kr_retention_schedule_version`.
- Add `mailbox.kr_marketing_consent_required` boolean.
- Add `alias.kr_real_name_exposure_level` as enum `none|display|legal`.
- Add `search_index.kr_erasure_rebuild_required` boolean.
- Add `consent_snapshot.kr_consent_text_hash`.
- Add `consent_snapshot.kr_consent_captured_at`.
- Add `audit_shadow.kr_pipa_event_id`.
- Add `export_job.kr_subject_rights_manifest_hash`.
- Add `tenant_mail_config.kr_pipa_notice_version`.

## Cedar Policy Deltas
- Policy `KRPIPA-mail-send-01`: require consent when `message.kr_processing_purpose_id.requires_consent == true`.
- Policy `KRPIPA-mail-send-02`: forbid RRN in subject or preview.
- Policy `KRPIPA-mail-send-03`: forbid outbound sensitive PI without approved purpose.
- Policy `KRPIPA-mail-transfer-01`: forbid cross-border route without `kr_cross_border_transfer_id`.
- Policy `KRPIPA-mail-transfer-02`: require Korean-language transfer notice before external processor route.
- Policy `KRPIPA-mail-retention-01`: require 보존 ledger entry for retention beyond default.
- Policy `KRPIPA-mail-erasure-01`: permit erasure when retention basis expired.
- Policy `KRPIPA-mail-erasure-02`: forbid erasure when legal hold or statutory retention is active.
- Policy `KRPIPA-mail-consent-01`: require 동의 text hash before marketing mail processing.
- Policy `KRPIPA-mail-consent-02`: forbid processing after consent withdrawal.
- Policy `KRPIPA-mail-minor-01`: require guardian consent for minor mailbox marketing.
- Policy `KRPIPA-mail-rrn-01`: require privacy officer approval for RRN attachment release.
- Policy `KRPIPA-mail-processor-01`: require 처리위탁 registry entry for delegated processing.
- Policy `KRPIPA-mail-admin-01`: require Korean DPO-visible audit for admin read.
- Policy `KRPIPA-mail-ai-01`: require explicit consent before AI draft touches Korean PI.
- Policy `KRPIPA-mail-search-01`: restrict sensitive PI search to approved purpose.
- Policy `KRPIPA-mail-export-01`: permit subject-rights export only through verified identity.
- Policy `KRPIPA-mail-breach-01`: start Korean breach workflow on confirmed PI leak.
- Policy `KRPIPA-mail-alias-01`: limit real-name exposure by mailbox setting.
- Policy `KRPIPA-mail-webhook-01`: forbid processor webhook without delegation registry.
- Policy `KRPIPA-mail-route-01`: require KR resident mail storage in approved KR cell.
- Policy `KRPIPA-mail-preview-01`: scrub Korean PI from notification previews.
- Policy `KRPIPA-mail-import-01`: require source consent proof for imported mailbox.
- Policy `KRPIPA-mail-pack-01`: defer deactivation while consent or retention cases are open.

## API Contract Deltas
- `POST /messages/send` requires `kr_processing_purpose_id` for KR tenants.
- `POST /messages/send` requires `kr_consent_id` when purpose is consent-based.
- `POST /messages/send` rejects RRN in subject and preview.
- `POST /messages/send` rejects sensitive PI without approved purpose.
- `POST /messages/{id}/forward` requires cross-border transfer id for non-KR route.
- `POST /consent/capture` stores Korean notice text hash.
- `POST /consent/withdraw` stops consent-based mail processing.
- `POST /retention/rules` requires 보존 basis id.
- `POST /dsr/export` requires verified identity and Korean notice version.
- `POST /dsr/erasure` starts Korean erasure workflow.
- `POST /attachments/{id}/release` requires privacy officer approval for RRN.
- `POST /webhooks` requires processor delegation id.
- `POST /ai/draft` requires Korean PI model-touch consent.
- `GET /privacy-notices/kr/mail` returns Korean-language notice.
- `POST /mailboxes/import` requires source consent proof.
- `GET /audit/admin-access` exposes DPO-visible Korean events.
- `POST /breach-candidates` starts KR breach workflow.
- `PATCH /tenant-mail-config` requires KR PIPA notice version.
- `POST /aliases` requires real-name exposure level.
- `GET /exports/{id}` returns Korean subject-rights manifest.

## Workflow Deltas
- Send preflight classifies Korean PI and RRN before routing.
- Consent capture stores 동의 text hash and timestamp.
- Consent withdrawal disables marketing and AI model touch.
- Retention change records 보존 basis and expiry.
- Cross-border forwarding displays Korean-language transfer notice.
- Processor webhook setup verifies 처리위탁 registry.
- RRN attachment release requires privacy officer approval.
- Minor mailbox marketing checks guardian consent.
- Korean subject-rights export verifies identity before enumeration.
- Erasure workflow tombstones message body and rebuilds index.
- Breach candidate workflow starts KR notification timeline.
- DPO admin-read review is created for support access.
- AI draft workflow requires explicit Korean PI consent.
- Import workflow validates source consent and transfer basis.
- Notification workflow scrubs Korean PI from previews.
- KR cell migration workflow refuses non-KR destination for resident PI.
- Alias workflow limits real-name exposure.
- Search workflow restricts sensitive PI queries by purpose.
- Pack deactivation waits for consent and retention ledgers.
- Audit bundle publication signs Korean manifest to audit-chain.

## SLO Deltas
- KR breach workflow creation p99 target is <= 5 minutes.
- Korean DPO notification p99 target is <= 24 hours for confirmed leak.
- Consent capture p99 must stay <= 300 ms.
- Consent withdrawal propagation p99 target is <= 30 minutes.
- RRN detection p99 target is <= 2 minutes after attachment ingest.
- RRN release approval workflow start p99 target is <= 2 minutes.
- Korean subject-rights export target is <= 10 days internal.
- Erasure tombstone p99 target is <= 72 hours after approval.
- KR route residency check p99 must stay <= 100 ms.
- Processor delegation lookup p99 must stay <= 200 ms.
- Korean notice retrieval p99 must stay <= 150 ms.
- Retention ledger write p99 must stay <= 500 ms.
- Admin-read audit emission p99 must complete <= 1 second.
- Index rebuild after erasure p99 target is <= 24 hours.
- Marketing objection enforcement p99 target is <= 10 minutes.
- Korean privacy dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `MailKrPipaConsentCaptured` records consent id and text hash.
- `MailKrPipaConsentWithdrawn` records consent id.
- `MailKrPipaPurposeChecked` records purpose id.
- `MailKrPipaRrnDetected` records attachment digest.
- `MailKrPipaRrnReleaseApproved` records privacy officer id.
- `MailKrPipaSensitivePiBlocked` records policy id.
- `MailKrPipaRetentionLedgerWritten` records 보존 basis.
- `MailKrPipaCrossBorderNoticeShown` records transfer id.
- `MailKrPipaProcessorDelegationChecked` records delegation id.
- `MailKrPipaSubjectRightsExportStarted` records case id.
- `MailKrPipaSubjectRightsExportCompleted` records manifest hash.
- `MailKrPipaErasureTombstoned` records message id.
- `MailKrPipaAdminAccessReviewed` records DPO review id.
- `MailKrPipaAiDraftConsentChecked` records model surface.
- `MailKrPipaMinorGuardianConsentRequired` records mailbox id.
- `MailKrPipaAliasDisclosureChanged` records exposure level.
- `MailKrPipaBreachWorkflowStarted` records candidate id.
- `MailKrPipaKrCellRouteBlocked` records target cell.
- `MailKrPipaNoticeVersionChanged` records notice version.
- `MailKrPipaPackDeactivationDeferred` records open ledger count.

## Failure Modes specific to this pack
- Consent ledger is unavailable; recovery is block consent-based processing.
- Korean notice text hash mismatches; recovery is disable affected consent capture.
- RRN appears in subject; recovery is reject send and suggest safe wording.
- RRN scanner times out; recovery is quarantine attachment.
- Cross-border transfer id is missing; recovery is block route.
- Processor delegation registry is stale; recovery is suspend webhook.
- Consent withdrawal races queued mail; recovery is cancel queued mail.
- Retention basis expires during legal hold; recovery is keep restriction and route review.
- Subject identity verification fails; recovery is deny export and record attempt.
- Erasure index rebuild fails; recovery is remove shard from serving.
- KR cell outage suggests non-KR failover; recovery is queue mail in KR boundary.
- AI draft attempts Korean PI without consent; recovery is block model touch.
- Minor guardian consent missing; recovery is disable marketing processing.
- Admin read lacks DPO case; recovery is revoke support session.
- Breach workflow deadline clock fails to start; recovery is create retroactive event and page compliance.
- Alias exposes legal name unexpectedly; recovery is revert to display-only and audit.
- Imported mailbox lacks consent proof; recovery is quarantine import.
- Korean-language notice unavailable; recovery is fail-closed for new processing.
- Pack deactivation requested with open ledgers; recovery is defer.
- Notification preview leaks Korean PI; recovery is disable previews and open incident.

## Cross-µservice coordination
- `tenancy` provides KR cell placement and active KR-PIPA pack roster.
- `identity` verifies subject identity, DPO roles, and minor guardian status.
- `compliance` owns 동의, 보존, 국외이전, and 처리위탁 ledgers.
- `audit-chain` seals Korean PI events and subject-rights manifests.
- `observability` scrubs Korean PI before telemetry storage.
- `drive` applies KR-PIPA overlay for mail attachments saved to drive.
- `workflow-engine` runs consent, erasure, breach, and RRN release workflows.
- `policy-engine` loads all `KRPIPA-mail-*` fragments.
- `localization` provides Korean privacy notice text.
- `notification` removes Korean PI from message previews.
- `dlp-virus-scan` returns RRN and sensitive PI verdicts.
- `admin-console` surfaces KR-PIPA mail configuration.
- `incident-response` consumes Korean breach candidates.
- `search` rebuilds indexes after Korean erasure.
- `legal` defines Korean retention and hold exceptions.
- `support` requires DPO-visible access path.
- `connector` validates delegated processor webhooks.
- `data-warehouse` receives only aggregate KR mail metrics.
- `billing` receives no Korean PI mail metadata.
- `pack-registry` signs this KR-PIPA mail overlay.
