---
doc_class: CompliancePackOverlay
pack_id: EU-GDPR-2018-baseline
microservice: drive
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# drive GDPR Compliance Pack Overlay

## Pack Identity
- Full pack name: EU GDPR drive file privacy and portability overlay.
- Citing jurisdiction: European Union and EEA personal-data regime.
- Version: EU-GDPR-2018-baseline-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2016/679/oj
- Cited law: Regulation (EU) 2016/679.
- Covered drive surface: files, folders, versions, previews, OCR text, share links, search index, sync cache, exports, legal holds, and erasure.
- Pack activation means drive treats user-authored and uploaded content as personal data unless classified otherwise.
- The overlay distinguishes personal data, special-category data, and business-confidential non-personal files.
- Data classes include `DRIVE_PERSONAL_DATA_EU`, `DRIVE_SPECIAL_CATEGORY_EU`, and `DRIVE_PORTABILITY_EXPORT_EU`.
- DSAR portability is file-native: original file, metadata manifest, and machine-readable folder map.
- ADR-0064 keeps EU-specific retention and rights behavior in the pack.
- ADR-0251 supplies pack admission, cell eligibility, and breach workflow.
- ADR-0263 requires PII scrubbing before telemetry leaves drive.
- This overlay excludes PCI-DSS because drive does not own payment authorization.
- Detected cardholder data is quarantined and handed to payments compliance if needed.

## Data Model Deltas
- Add `file.eu_personal_data_signal` as enum `none|personal|special_category`.
- Add `file.lawful_basis` for processing and indexing.
- Add `file.lawful_basis_evidence_id` nullable.
- Add `file.data_subject_ids_hash` for DSAR enumeration.
- Add `file.erasure_state` as enum `active|restricted|erasure_pending|tombstoned`.
- Add `file.restriction_reason` for Article 18.
- Add `file.portability_included` boolean.
- Add `file.transfer_mechanism` as enum `none|scc|adequacy|derogation`.
- Add `file.eu_residency_cell` from tenancy placement.
- Add `file.preview_personal_data_scrubbed_at`.
- Add `file.ocr_personal_data_scrubbed_at`.
- Add `folder.eu_subject_scope_hash` rollup.
- Add `folder.dsar_export_cursor`.
- Add `share_link.eu_transfer_risk` as enum `none|internal|external_eea|external_non_eea`.
- Add `share_link.processor_dpa_ref`.
- Add `version.erasure_superseded_by` for tombstone lineage.
- Add `search_index.eu_erasure_rebuild_required` boolean.
- Add `sync_cache.eu_remote_wipe_required` boolean.
- Add `legal_hold.erasure_conflict_reason`.
- Add `export_job.gdpr_manifest_hash`.
- Add `export_job.portability_format_set` default `original,metadata,json`.
- Add `audit_shadow.gdpr_event_id`.
- Add `tenant_drive_config.eu_dpa_version`.
- Add `tenant_drive_config.eu_retention_schedule_version`.

## Cedar Policy Deltas
- Policy `GDPR-drive-read-01`: permit read only for declared processing purpose.
- Policy `GDPR-drive-read-02`: forbid read when file is restricted and caller is not DPO.
- Policy `GDPR-drive-upload-01`: require lawful basis for indexing uploaded content.
- Policy `GDPR-drive-share-01`: forbid non-EEA share without transfer mechanism.
- Policy `GDPR-drive-share-02`: require processor DPA for external processor share.
- Policy `GDPR-drive-export-01`: permit DSAR export for verified subject or DPO.
- Policy `GDPR-drive-export-02`: require manifest hash before export release.
- Policy `GDPR-drive-erasure-01`: permit tombstone when no legal hold conflict exists.
- Policy `GDPR-drive-erasure-02`: forbid purge when statutory retention applies.
- Policy `GDPR-drive-restrict-01`: restrict serving during accuracy dispute.
- Policy `GDPR-drive-portability-01`: require original format plus metadata manifest.
- Policy `GDPR-drive-preview-01`: forbid preview for restricted files.
- Policy `GDPR-drive-ocr-01`: forbid OCR model touch without lawful basis.
- Policy `GDPR-drive-search-01`: require index rebuild after erasure.
- Policy `GDPR-drive-sync-01`: require remote wipe after erasure on synced devices.
- Policy `GDPR-drive-link-01`: require expiration for external personal-data links.
- Policy `GDPR-drive-breach-01`: start Article 33 clock on confirmed exfiltration.
- Policy `GDPR-drive-admin-01`: require DPO-visible audit for admin file access.
- Policy `GDPR-drive-retention-01`: forbid blanket indefinite retention.
- Policy `GDPR-drive-import-01`: require origin transfer mechanism for imports.
- Policy `GDPR-drive-ai-01`: require model-touch consent for AI tagging.
- Policy `GDPR-drive-route-01`: require EU residency unless transfer mechanism permits.
- Policy `GDPR-drive-webhook-01`: forbid file webhook without DPA reference.
- Policy `GDPR-drive-pack-01`: defer pack deactivation with open DSAR cases.

## API Contract Deltas
- `POST /files` requires `lawful_basis` for EU tenants.
- `POST /files` accepts `lawful_basis_evidence_id`.
- `GET /files/{id}` masks restricted files for non-DPO roles.
- `POST /files/{id}/share-links` requires transfer mechanism for non-EEA access.
- `POST /files/{id}/preview` refuses restricted files.
- `POST /files/{id}/ocr` requires lawful basis.
- `POST /dsar/export` starts file and folder portability export.
- `GET /dsar/export/{id}` returns manifest hash and original formats.
- `POST /dsar/erasure` starts tombstone and index rebuild workflow.
- `POST /dsar/restrict` blocks serving while accuracy is disputed.
- `POST /sync/remote-wipe` records erasure-driven wipe task.
- `POST /search/rebuild` requires erasure reason.
- `POST /webhooks` requires processor DPA reference.
- `POST /files/import` records origin transfer mechanism.
- `POST /ai/tag` requires model-touch lawful basis.
- `POST /retention/rules` requires purpose-bounded schedule.
- `POST /breach-candidates` starts GDPR breach clock.
- `GET /audit/admin-access` returns DPO-visible access events.
- `PATCH /tenant-drive-config` records EU DPA version.
- `POST /pack/deactivate` refuses open DSAR cases.

## Workflow Deltas
- Upload workflow records lawful basis before indexing.
- Special-category classifier runs before preview and share.
- DSAR export enumerates files by subject hash.
- Portability export includes original file and metadata JSON.
- Erasure workflow tombstones content and versions.
- Search index rebuild runs after tombstone.
- Sync remote wipe runs after erasure approval.
- Restriction workflow blocks file serving.
- Legal hold conflict routes to DPO review.
- External share workflow validates DPA and transfer mechanism.
- OCR workflow checks lawful basis before model touch.
- AI tagging workflow records model-touch consent or lawful basis.
- Import workflow records origin transfer mechanism.
- Breach candidate workflow starts Article 33 clock.
- Admin access workflow creates DPO-visible review event.
- Retention schedule workflow records processing purpose.
- Preview cache purge runs after erasure or restriction.
- Pack activation workflow scans inherited public links.
- Pack deactivation waits for open DSAR work.
- Audit bundle workflow signs export manifests.

## SLO Deltas
- GDPR breach regulator-readiness p99 target is <= 60 hours.
- Breach clock creation p99 target is <= 5 minutes.
- DSAR file enumeration first response target is <= 7 days.
- Full portability export target is <= 30 days.
- Erasure tombstone p99 target is <= 72 hours after approval.
- Search index rebuild after erasure target is <= 24 hours.
- Sync remote wipe task creation p99 target is <= 30 minutes.
- Restriction activation p99 must complete <= 15 minutes.
- Lawful-basis upload preflight p99 must stay <= 500 ms.
- EU route validation p99 must stay <= 100 ms.
- External DPA lookup p99 must stay <= 200 ms.
- Admin access audit p99 must complete <= 1 second.
- Preview cache purge p99 target is <= 2 hours.
- Portability manifest generation p99 target is <= 4 hours.
- Special-category classifier review cadence is daily.
- DPO dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `DriveGdprLawfulBasisRecorded` records file id and basis.
- `DriveGdprSpecialCategoryDetected` records classifier version.
- `DriveGdprShareTransferChecked` records mechanism.
- `DriveGdprDsarExportStarted` records case id.
- `DriveGdprDsarExportCompleted` records manifest hash.
- `DriveGdprFileTombstoned` records file id.
- `DriveGdprVersionTombstoned` records version id.
- `DriveGdprRestrictionApplied` records reason.
- `DriveGdprRestrictionReleased` records reviewer.
- `DriveGdprIndexRebuilt` records shard id.
- `DriveGdprRemoteWipeRequested` records device id hash.
- `DriveGdprPreviewCachePurged` records file id.
- `DriveGdprWebhookDpaRejected` records destination id.
- `DriveGdprAiTaggingConsentChecked` records model id.
- `DriveGdprBreachClockStarted` records candidate id.
- `DriveGdprAdminAccessReviewed` records review id.
- `DriveGdprImportTransferRecorded` records origin.
- `DriveGdprRetentionScheduleChanged` records schedule.
- `DriveGdprPublicLinkRevoked` records link id.
- `DriveGdprPackDeactivationDeferred` records open cases.

## Failure Modes specific to this pack
- Lawful basis missing on upload; recovery is hold file unindexed.
- DSAR subject hash misses shared folder; recovery is rerun from audit-chain.
- Export manifest omits version; recovery is revoke export and rebuild.
- Erasure tombstone fails on object store; recovery is deny reads and retry purge.
- Search shard still serves erased content; recovery is remove shard and rebuild.
- Sync client remains online with erased file; recovery is remote wipe and rotate token.
- External share lacks DPA; recovery is revoke link.
- Transfer mechanism expires; recovery is block non-EEA access.
- Special-category classifier is unavailable; recovery is fail-closed for sharing.
- OCR ran without lawful basis; recovery is delete OCR text and open incident.
- AI tag retained personal data; recovery is purge tag and evidence.
- Preview cache leaks restricted file; recovery is purge cache and disable preview.
- Admin access lacks DPO case; recovery is revoke access and open exception.
- Retention schedule is indefinite; recovery is reject schedule.
- Public link existed before activation; recovery is revoke and notify owner.
- Legal hold blocks erasure; recovery is restrict processing and route review.
- Pack deactivation requested with open DSAR; recovery is defer.
- Import lacks origin transfer proof; recovery is quarantine import.
- EU cell outage suggests non-EU failover; recovery is queue operations.
- Breach clock fails to start; recovery is create retroactive audit event and page compliance.

## Cross-µservice coordination
- `tenancy` provides EU cell placement and active pack roster.
- `identity` verifies data subject, DPO role, and device owner.
- `compliance` owns DSAR, processing register, DPA, and breach cases.
- `audit-chain` seals file lifecycle, DSAR, and erasure events.
- `observability` scrubs file names and personal data from telemetry.
- `mail` applies GDPR overlay when attachments move between mail and drive.
- `workflow-engine` runs DSAR, erasure, restriction, and breach workflows.
- `policy-engine` loads all `GDPR-drive-*` fragments.
- `search` rebuilds personal-data indexes after erasure.
- `sync` performs remote wipe for erased synced files.
- `dlp-virus-scan` classifies special-category content.
- `notification` avoids personal data in share previews.
- `admin-console` renders EU DPA and retention configuration.
- `incident-response` consumes file exfiltration candidates.
- `legal` resolves legal hold conflicts.
- `data-warehouse` receives aggregate storage metrics only.
- `support` uses DPO-visible access path.
- `connector` validates external processor DPA references.
- `localization` provides EU language notices.
- `pack-registry` signs this GDPR drive overlay.
