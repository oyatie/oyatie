---
doc_class: CompliancePackOverlay
pack_id: HIPAA-2024
microservice: drive
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# drive HIPAA Compliance Pack Overlay

## Pack Identity
- Full pack name: HIPAA Administrative Simplification drive ePHI storage overlay.
- Citing jurisdiction: United States federal health information regime.
- Version: HIPAA-2024-v1.
- Canonical source URL: https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C
- Cited law: 45 CFR Parts 160, 162, and 164.
- Covered drive surface: files, versions, folders, share links, previews, OCR, DLP scans, WORM tiers, legal holds, exports, and deletion workflows.
- Pack activation means drive can store ePHI only in HIPAA-certified cells with BAA admission proof.
- The overlay treats file content, preview text, extracted OCR, and thumbnails as possible ePHI.
- Data classes added here include `DRIVE_FILE_PHI`, `DRIVE_PREVIEW_PHI`, `DRIVE_SHARE_PHI`, and `DRIVE_AUDIT_PHI`.
- Minimum necessary applies to reads, shares, previews, search, exports, and sync.
- ADR-0064 keeps drive canonical storage neutral while this pack adds policy, retention, and metadata deltas.
- ADR-0251 supplies cell certification, pack registry, breach workflow, and encryption requirements.
- ADR-0263 requires PHI-safe telemetry and audit ids for every file mutation.
- This overlay excludes PCI-DSS because drive stores tokenized payment documents only when a payments pack owns PCI scope.
- Any detected cardholder data is routed to DLP quarantine and payments compliance review.

## Data Model Deltas
- Add `file.phi_signal` as enum `none|possible|confirmed`.
- Add `file.phi_basis` as enum `treatment|payment|operations|patient_request|none`.
- Add `file.patient_context_id` as nullable opaque reference.
- Add `file.minimum_necessary_scope` as array of allowed roles.
- Add `file.encrypted_phi_blob_ref` for envelope-encrypted content.
- Add `file.openbao_dek_ref` for tenant key reference.
- Add `file.preview_phi_state` as enum `not_generated|safe|blocked|confirmed_phi`.
- Add `file.ocr_phi_state` as enum `not_run|safe|blocked|confirmed_phi`.
- Add `file.thumbnail_phi_blocked` boolean.
- Add `file.break_glass_reason_id` nullable.
- Add `file.disclosure_accounting_required` boolean.
- Add `file.retention_floor_iso8601` default `P6Y`.
- Add `version.phi_diff_hash` for PHI version comparison.
- Add `share_link.phi_external_recipient_baa_status`.
- Add `share_link.minimum_necessary_expires_at`.
- Add `folder.hipaa_case_context_id` nullable.
- Add `folder.inherited_phi_signal` rollup.
- Add `dlp_verdict.hipaa_rule_id`.
- Add `scan.phi_scanner_build_id`.
- Add `legal_hold.hipaa_hold_reason`.
- Add `export_job.phi_manifest_hash`.
- Add `sync_client.phi_offline_cache_allowed` default false.
- Add `audit_shadow.drive_phi_event_id`.
- Add `tenant_drive_config.hipaa_cell_certification` requiring `hipaa-certified`.

## Cedar Policy Deltas
- Policy `HIPAA-drive-upload-01`: permit upload when `tenant.has_pack("HIPAA-2024") && cell.certification.contains("hipaa-certified")`.
- Policy `HIPAA-drive-upload-02`: require PHI scan before file becomes shareable.
- Policy `HIPAA-drive-read-01`: permit read when `principal.role in file.minimum_necessary_scope`.
- Policy `HIPAA-drive-read-02`: forbid read when `file.phi_signal == "confirmed" && principal.acr < "elevated"`.
- Policy `HIPAA-drive-share-01`: forbid external share when recipient BAA is not verified.
- Policy `HIPAA-drive-share-02`: require expiry <= 7 days for external PHI share link.
- Policy `HIPAA-drive-preview-01`: forbid preview generation when OCR would expose confirmed PHI.
- Policy `HIPAA-drive-preview-02`: permit preview only through PHI-safe renderer.
- Policy `HIPAA-drive-download-01`: require purpose in `treatment|payment|operations`.
- Policy `HIPAA-drive-sync-01`: forbid offline cache for confirmed PHI unless managed device.
- Policy `HIPAA-drive-sync-02`: require device-bound key for PHI sync.
- Policy `HIPAA-drive-ocr-01`: forbid OCR model touch unless provider BYOK is active.
- Policy `HIPAA-drive-search-01`: restrict PHI search to case context.
- Policy `HIPAA-drive-retention-01`: forbid purge before six-year floor or active hold release.
- Policy `HIPAA-drive-worm-01`: require WORM immutability for designated clinical records.
- Policy `HIPAA-drive-breakglass-01`: permit emergency read only with reason id and TTL <= 1h.
- Policy `HIPAA-drive-export-01`: require privacy-office approval for PHI export.
- Policy `HIPAA-drive-export-02`: require manifest hash before download.
- Policy `HIPAA-drive-route-01`: forbid replication outside HIPAA-certified cell.
- Policy `HIPAA-drive-delete-01`: permit tombstone only after retention check.
- Policy `HIPAA-drive-link-01`: forbid public anonymous links for PHI.
- Policy `HIPAA-drive-folder-01`: inherited PHI signal controls child objects.
- Policy `HIPAA-drive-quarantine-01`: permit release only with DLP override approval.
- Policy `HIPAA-drive-pack-01`: defer deactivation while PHI files remain under retention.

## API Contract Deltas
- `POST /files` requires `X-Oyatie-Purpose` for HIPAA tenants.
- `POST /files` returns `phi_scan_required=true` before shareable state.
- `GET /files/{id}` requires elevated ACR for confirmed PHI.
- `GET /files/{id}/download` requires purpose header.
- `POST /files/{id}/share-links` rejects anonymous links for PHI.
- `POST /files/{id}/share-links` requires recipient BAA proof for external link.
- `POST /files/{id}/preview` refuses confirmed PHI unless PHI-safe renderer is available.
- `POST /files/{id}/ocr` requires provider BYOK mode.
- `POST /sync/sessions` requires managed-device proof for PHI offline cache.
- `POST /folders` accepts `hipaa_case_context_id`.
- `POST /exports` requires privacy-office approval id.
- `GET /exports/{id}` returns PHI manifest hash.
- `DELETE /files/{id}` returns retention conflict before six-year floor.
- `POST /legal-holds` requires HIPAA hold reason.
- `DELETE /legal-holds/{id}` requires compliance workflow completion.
- `POST /quarantine/{id}/release` requires DLP override reason.
- `POST /replication/plan` rejects non-certified target cells.
- `GET /audit/disclosures` returns PHI disclosure accounting.
- `PATCH /tenant-drive-config` requires BAA admission proof.
- `POST /pack/deactivate` returns retained PHI object count.

## Workflow Deltas
- Upload workflow scans content before indexing or sharing.
- Preview workflow blocks thumbnails when PHI is confirmed.
- OCR workflow uses BYOK provider path or fails closed.
- Share workflow checks BAA and minimum necessary.
- External share workflow writes disclosure accounting.
- Sync workflow checks managed-device and offline cache policy.
- Folder inheritance workflow rolls PHI signal to children.
- Legal hold workflow locks WORM tier before export.
- Deletion workflow checks HIPAA retention and active holds.
- Export workflow builds PHI manifest before object assembly.
- Break-glass workflow opens one-hour emergency access.
- Quarantine workflow isolates scan-failed files.
- DLP override workflow requires privacy-office approval.
- Replication workflow validates HIPAA-certified target cell.
- Search workflow indexes only PHI-safe tokens.
- Version workflow hashes PHI diff without storing raw diff in audit.
- Incident workflow marks suspected PHI exfiltration candidate.
- Pack activation workflow disables public links on inherited PHI folders.
- Pack deactivation workflow waits for retained PHI inventory.
- Audit publication workflow seals every PHI file state change.

## SLO Deltas
- PHI upload scan start p99 must complete <= 2 minutes.
- PHI DLP scan completion p99 target is <= 15 minutes.
- PHI read audit seal p99 must complete <= 1 second.
- External BAA lookup p99 must stay <= 200 ms.
- PHI share-link creation p99 must stay <= 500 ms.
- PHI preview block decision p99 must stay <= 300 ms.
- OCR BYOK admission check p99 must stay <= 200 ms.
- Managed-device sync check p99 must stay <= 200 ms.
- Disclosure accounting write p99 must complete <= 1 second.
- Export manifest generation p99 target is <= 30 minutes.
- Retention conflict response p99 must stay <= 300 ms.
- Quarantine placement p99 must complete <= 2 minutes.
- Break-glass workflow start p99 must complete <= 2 minutes.
- HIPAA breach candidate creation p99 target is <= 5 minutes.
- Route residency validation p99 must stay <= 100 ms.
- PHI inventory report lag target is <= 1 hour.

## Audit-event class additions
- `DrivePhiFileUploaded` records file id and scan state.
- `DrivePhiScanCompleted` records verdict and scanner build.
- `DrivePhiPreviewBlocked` records renderer reason.
- `DrivePhiOcrBlocked` records provider credential mode.
- `DrivePhiFileRead` records purpose and principal role.
- `DrivePhiDownloadIssued` records download ticket hash.
- `DrivePhiShareLinkBlocked` records BAA status.
- `DrivePhiShareLinkIssued` records expiry and recipient hash.
- `DrivePhiExternalDisclosureRecorded` records disclosure id.
- `DrivePhiSyncSessionDenied` records device state.
- `DrivePhiLegalHoldApplied` records hold reason.
- `DrivePhiLegalHoldReleased` records workflow id.
- `DrivePhiWormTierEntered` records retention floor.
- `DrivePhiExportManifestCreated` records manifest hash.
- `DrivePhiPurgeRefused` records retention conflict.
- `DrivePhiBreakGlassStarted` records reason id.
- `DrivePhiQuarantined` records file digest.
- `DrivePhiQuarantineReleased` records approver.
- `DrivePhiReplicationBlocked` records target cell.
- `DrivePhiPackDeactivationDeferred` records retained count.

## Failure Modes specific to this pack
- PHI scanner is unavailable; recovery is quarantine uploads and block shares.
- BAA registry lookup fails; recovery is block external share.
- Preview renderer leaks text; recovery is disable preview for PHI class.
- OCR provider is not BYOK; recovery is block OCR and preserve raw file.
- Managed-device proof is stale; recovery is deny offline sync.
- External recipient loses BAA status; recovery is revoke share links.
- Public link existed before activation; recovery is revoke and notify owner.
- Legal hold conflicts with deletion; recovery is hold lock wins.
- WORM entry fails; recovery is halt export and page storage owner.
- Export manifest mismatch appears; recovery is revoke bundle and rebuild.
- Search index contains raw PHI; recovery is drop shard and rebuild safe index.
- Replication planner chooses non-certified cell; recovery is reject plan.
- Break-glass TTL expires; recovery is revoke ticket and preserve audit.
- DLP false positive blocks urgent care file; recovery is privacy-office override.
- DLP false negative is discovered; recovery is retroactive accounting and incident review.
- Sync cache persists after revocation; recovery is remote wipe and rotate device key.
- File version diff exposes PHI in audit; recovery is purge diff payload and keep hash.
- Pack deactivation requested with retained PHI; recovery is defer.
- Attachment imported from mail lacks PHI state; recovery is rescan before indexing.
- Audit-chain backpressure appears; recovery is fail-closed for PHI mutations.

## Cross-µservice coordination
- `tenancy` must place HIPAA tenants in HIPAA-certified cells.
- `identity` provides elevated ACR and managed-device claims.
- `compliance` provides BAA proof, breach workflow, and privacy-office approvals.
- `audit-chain` seals every PHI file event.
- `observability` scrubs PHI from file telemetry.
- `mail` applies HIPAA overlay when attachments move between mail and drive.
- `workflow-engine` runs quarantine, legal hold, break-glass, and export workflows.
- `policy-engine` loads all `HIPAA-drive-*` fragments.
- `dlp-virus-scan` returns signed PHI verdicts.
- `search` indexes only PHI-safe tokens.
- `cloud-kms` or OpenBao provides tenant DEK references.
- `admin-console` displays BAA and PHI inventory state.
- `incident-response` consumes exfiltration candidates.
- `notification` avoids PHI in share notifications.
- `records` owns patient context references.
- `data-warehouse` receives aggregate PHI-free storage metrics.
- `support` uses break-glass workflow for PHI tickets.
- `legal` defines WORM and hold templates.
- `sync` enforces device-bound offline policy.
- `pack-registry` signs this HIPAA drive overlay.
