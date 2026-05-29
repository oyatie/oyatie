---
doc_class: CompliancePackOverlay
pack_id: KR-PIPA-2023-amendment
microservice: drive
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# drive KR-PIPA Compliance Pack Overlay

## Pack Identity
- Full pack name: Korea Personal Information Protection Act drive overlay.
- Citing jurisdiction: Republic of Korea personal information regime.
- Version: KR-PIPA-2023-amendment-v1.
- Canonical source URL: https://law.go.kr/LSW/lsInfoP.do?lsId=011357
- Cited law: 개인정보 보호법, Act No. 17799 baseline with current consolidation at law.go.kr.
- Covered drive surface: files, folders, versions, share links, previews, OCR, search, sync cache, retention, subject-rights export, and breach workflow.
- Pack activation means drive captures 동의, 보존, 국외이전, and 처리위탁 evidence for Korean personal information.
- Korean resident registration numbers are high-risk and blocked from previews, search snippets, and notifications.
- Data classes include `DRIVE_PI_KR`, `DRIVE_SENSITIVE_PI_KR`, `DRIVE_RRN_KR`, and `DRIVE_CONSENT_LEDGER_KR`.
- Korean-language notices are required for consent, cross-border transfer, breach, and subject-rights workflows.
- ADR-0064 keeps Korean field behavior in the pack overlay.
- ADR-0251 supplies pack admission, breach workflow, and retention ledgers.
- ADR-0263 requires Korean PI scrubbing at telemetry boundaries.
- PCI-DSS is omitted because drive is not payment authorization.
- Detected PAN-like content is quarantined and routed to DLP review.

## Data Model Deltas
- Add `file.kr_pi_signal` as enum `none|personal|sensitive|rrn`.
- Add `file.kr_consent_id` for 동의 ledger linkage.
- Add `file.kr_processing_purpose_id`.
- Add `file.kr_retention_basis_id` for 보존 proof.
- Add `file.kr_retention_until` timestamp.
- Add `file.kr_cross_border_transfer_id`.
- Add `file.kr_processor_delegation_id`.
- Add `file.kr_notice_language` default `ko-KR`.
- Add `file.kr_subject_rights_case_id`.
- Add `file.kr_breach_clock_started_at`.
- Add `attachment.kr_rrn_detected` boolean.
- Add `preview.kr_rrn_preview_blocked` boolean.
- Add `ocr.kr_sensitive_pi_verdict`.
- Add `folder.kr_subject_scope_hash`.
- Add `folder.kr_local_residency_cell`.
- Add `share_link.kr_transfer_notice_id`.
- Add `share_link.kr_expiry_required` default true.
- Add `sync_cache.kr_remote_wipe_required`.
- Add `search_index.kr_erasure_rebuild_required`.
- Add `consent_snapshot.kr_consent_text_hash`.
- Add `export_job.kr_subject_rights_manifest_hash`.
- Add `audit_shadow.kr_pipa_event_id`.
- Add `tenant_drive_config.kr_pipa_notice_version`.
- Add `tenant_drive_config.kr_retention_schedule_version`.

## Cedar Policy Deltas
- Policy `KRPIPA-drive-upload-01`: require processing purpose for KR personal files.
- Policy `KRPIPA-drive-upload-02`: require consent when purpose requires 동의.
- Policy `KRPIPA-drive-read-01`: restrict sensitive PI read to approved purpose.
- Policy `KRPIPA-drive-share-01`: forbid RRN share without privacy officer approval.
- Policy `KRPIPA-drive-share-02`: require Korean transfer notice for non-KR recipient.
- Policy `KRPIPA-drive-transfer-01`: forbid cross-border route without transfer id.
- Policy `KRPIPA-drive-retention-01`: require 보존 ledger entry for retention.
- Policy `KRPIPA-drive-erasure-01`: permit erasure after retention basis expires.
- Policy `KRPIPA-drive-erasure-02`: forbid erasure while statutory retention or hold exists.
- Policy `KRPIPA-drive-consent-01`: require consent text hash for consent-based processing.
- Policy `KRPIPA-drive-consent-02`: forbid processing after consent withdrawal.
- Policy `KRPIPA-drive-rrn-01`: block preview and snippets for RRN files.
- Policy `KRPIPA-drive-rrn-02`: require approval for RRN attachment release.
- Policy `KRPIPA-drive-processor-01`: require 처리위탁 registry for delegated processing.
- Policy `KRPIPA-drive-admin-01`: require DPO-visible audit for admin file read.
- Policy `KRPIPA-drive-ai-01`: require explicit consent before AI OCR touches Korean PI.
- Policy `KRPIPA-drive-search-01`: restrict sensitive PI search by purpose.
- Policy `KRPIPA-drive-export-01`: require verified identity for subject-rights export.
- Policy `KRPIPA-drive-breach-01`: start Korean breach workflow on confirmed PI leak.
- Policy `KRPIPA-drive-link-01`: require expiry for external KR PI share links.
- Policy `KRPIPA-drive-route-01`: require KR resident storage in approved KR cell.
- Policy `KRPIPA-drive-preview-01`: scrub Korean PI from preview thumbnails.
- Policy `KRPIPA-drive-import-01`: require source consent proof for imported files.
- Policy `KRPIPA-drive-pack-01`: defer deactivation while ledgers are open.

## API Contract Deltas
- `POST /files` requires `kr_processing_purpose_id` for KR tenants.
- `POST /files` requires `kr_consent_id` when purpose is consent-based.
- `POST /files` returns RRN quarantine state when detected.
- `GET /files/{id}/preview` blocks RRN preview.
- `POST /files/{id}/share-links` requires transfer notice for non-KR recipients.
- `POST /files/{id}/share-links` requires expiry.
- `POST /consent/capture` stores Korean notice text hash.
- `POST /consent/withdraw` stops consent-based drive processing.
- `POST /retention/rules` requires 보존 basis id.
- `POST /dsr/export` requires verified identity.
- `POST /dsr/erasure` starts Korean erasure workflow.
- `POST /search/rebuild` requires erasure reason.
- `POST /sync/remote-wipe` records erasure-driven wipe task.
- `POST /webhooks` requires processor delegation id.
- `POST /ai/ocr` requires Korean PI model-touch consent.
- `GET /privacy-notices/kr/drive` returns Korean-language notice.
- `POST /files/import` requires source consent proof.
- `GET /audit/admin-access` exposes DPO-visible events.
- `POST /breach-candidates` starts KR breach workflow.
- `PATCH /tenant-drive-config` requires KR notice version.

## Workflow Deltas
- Upload preflight classifies Korean PI and RRN.
- Consent capture stores 동의 text hash and timestamp.
- Consent withdrawal disables AI OCR and sharing.
- Retention change records 보존 basis and expiry.
- Cross-border share displays Korean transfer notice.
- Processor webhook setup verifies 처리위탁 registry.
- RRN file release requires privacy officer approval.
- Korean subject-rights export verifies identity before enumeration.
- Erasure workflow tombstones file and versions.
- Search index rebuild runs after erasure.
- Sync remote wipe runs after erasure.
- Breach candidate workflow starts KR notification timeline.
- DPO admin-read review is created for support access.
- AI OCR workflow requires explicit Korean PI consent.
- Import workflow validates source consent and transfer basis.
- Preview workflow blocks RRN thumbnails and snippets.
- KR cell migration refuses non-KR destination for resident PI.
- Share link expiry enforcement runs at creation.
- Pack deactivation waits for consent and retention ledgers.
- Audit bundle publication signs Korean manifest.

## SLO Deltas
- KR breach workflow creation p99 target is <= 5 minutes.
- Korean DPO notification p99 target is <= 24 hours for confirmed leak.
- Consent capture p99 must stay <= 300 ms.
- Consent withdrawal propagation p99 target is <= 30 minutes.
- RRN detection p99 target is <= 2 minutes after file ingest.
- RRN release workflow start p99 target is <= 2 minutes.
- Korean subject-rights export target is <= 10 days internal.
- Erasure tombstone p99 target is <= 72 hours after approval.
- Search index rebuild target is <= 24 hours.
- Sync remote wipe task creation p99 target is <= 30 minutes.
- KR route residency check p99 must stay <= 100 ms.
- Processor delegation lookup p99 must stay <= 200 ms.
- Korean notice retrieval p99 must stay <= 150 ms.
- Retention ledger write p99 must stay <= 500 ms.
- Admin-read audit emission p99 must complete <= 1 second.
- Korean privacy dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `DriveKrPipaConsentCaptured` records consent id and text hash.
- `DriveKrPipaConsentWithdrawn` records consent id.
- `DriveKrPipaPurposeChecked` records purpose id.
- `DriveKrPipaRrnDetected` records file digest.
- `DriveKrPipaRrnPreviewBlocked` records file id.
- `DriveKrPipaRrnReleaseApproved` records privacy officer id.
- `DriveKrPipaSensitivePiBlocked` records policy id.
- `DriveKrPipaRetentionLedgerWritten` records 보존 basis.
- `DriveKrPipaCrossBorderNoticeShown` records transfer id.
- `DriveKrPipaProcessorDelegationChecked` records delegation id.
- `DriveKrPipaSubjectRightsExportStarted` records case id.
- `DriveKrPipaSubjectRightsExportCompleted` records manifest hash.
- `DriveKrPipaErasureTombstoned` records file id.
- `DriveKrPipaRemoteWipeRequested` records device hash.
- `DriveKrPipaAdminAccessReviewed` records DPO review id.
- `DriveKrPipaAiOcrConsentChecked` records model surface.
- `DriveKrPipaBreachWorkflowStarted` records candidate id.
- `DriveKrPipaKrCellRouteBlocked` records target cell.
- `DriveKrPipaNoticeVersionChanged` records notice version.
- `DriveKrPipaPackDeactivationDeferred` records open ledger count.

## Failure Modes specific to this pack
- Consent ledger is unavailable; recovery is block consent-based drive processing.
- Korean notice text hash mismatches; recovery is disable affected consent capture.
- RRN appears in preview; recovery is purge preview cache and block renderer.
- RRN scanner times out; recovery is quarantine file.
- Cross-border transfer id is missing; recovery is block share.
- Processor delegation registry is stale; recovery is suspend webhook.
- Consent withdrawal races queued share email; recovery is revoke link.
- Retention basis expires during legal hold; recovery is restrict and route review.
- Subject identity verification fails; recovery is deny export.
- Erasure index rebuild fails; recovery is remove shard from serving.
- KR cell outage suggests non-KR failover; recovery is queue operations.
- AI OCR attempts Korean PI without consent; recovery is block model touch.
- Admin read lacks DPO case; recovery is revoke support session.
- Breach workflow clock fails to start; recovery is create retroactive event.
- External share link lacks expiry; recovery is revoke link.
- Imported file lacks consent proof; recovery is quarantine import.
- Korean-language notice unavailable; recovery is fail-closed for new processing.
- Pack deactivation requested with open ledgers; recovery is defer.
- Notification preview leaks Korean PI; recovery is disable previews.
- Sync cache survives erasure; recovery is remote wipe and token rotation.

## Cross-µservice coordination
- `tenancy` provides KR cell placement and active KR-PIPA roster.
- `identity` verifies subject identity, DPO roles, and device ownership.
- `compliance` owns 동의, 보존, 국외이전, and 처리위탁 ledgers.
- `audit-chain` seals Korean PI events and export manifests.
- `observability` scrubs Korean PI before telemetry storage.
- `mail` applies KR-PIPA overlay for share notifications and attachment moves.
- `workflow-engine` runs consent, erasure, breach, and RRN release workflows.
- `policy-engine` loads all `KRPIPA-drive-*` fragments.
- `localization` provides Korean privacy notices.
- `notification` removes Korean PI from share previews.
- `dlp-virus-scan` returns RRN and sensitive PI verdicts.
- `search` rebuilds indexes after Korean erasure.
- `sync` executes remote wipe.
- `admin-console` surfaces KR drive configuration.
- `incident-response` consumes Korean breach candidates.
- `legal` defines Korean retention and hold exceptions.
- `support` requires DPO-visible access path.
- `connector` validates delegated processor webhooks.
- `data-warehouse` receives only aggregate KR drive metrics.
- `pack-registry` signs this KR-PIPA drive overlay.
