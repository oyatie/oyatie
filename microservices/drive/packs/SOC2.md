---
doc_class: CompliancePackOverlay
pack_id: SOC2-T2
microservice: drive
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# drive SOC 2 Compliance Pack Overlay

## Pack Identity
- Full pack name: SOC 2 Type II Trust Services Criteria drive control overlay.
- Citing jurisdiction: AICPA attestation framework for service organizations.
- Version: SOC2-T2-2017-TSC-2022-POF-v1.
- Canonical source URL: https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022
- Cited framework: 2017 Trust Services Criteria with Revised Points of Focus 2022.
- Covered drive surface: file access, storage durability, backups, restore drills, share links, DLP, WORM, admin access, change control, and audit exports.
- Pack activation means drive emits auditor-ready evidence for selected Trust Services Criteria.
- SOC 2 adds control evidence; it does not replace legal privacy packs.
- Data classes include `DRIVE_SOC2_EVIDENCE`, `DRIVE_CONTROL_EXCEPTION`, and `DRIVE_ACCESS_REVIEW_RECORD`.
- Type II operation requires evidence across the audit period, not only point-in-time settings.
- ADR-0064 keeps control evidence in an overlay.
- ADR-0251 supplies pack signature and evidence retention.
- ADR-0263 supplies telemetry and audit linkage for control proof.
- Raw file content is not exposed to auditors unless the tenant approves.
- This overlay excludes PCI-DSS because drive is not the cardholder-data environment.

## Data Model Deltas
- Add `drive_control.control_period_id`.
- Add `drive_control.trust_service_categories`.
- Add `drive_control.cc_mapping`.
- Add `drive_control.owner_team`.
- Add `drive_control.test_frequency`.
- Add `drive_control.last_tested_at`.
- Add `drive_control.exception_state`.
- Add `drive_control.exception_ticket_id`.
- Add `drive_control.evidence_hash`.
- Add `drive_control.sample_selection_seed`.
- Add `file.access_review_cycle_id`.
- Add `file.integrity_hash_verified_at`.
- Add `file.backup_snapshot_id`.
- Add `file.restore_drill_id`.
- Add `file.dlp_control_verdict_id`.
- Add `file.share_link_review_due_at`.
- Add `file.worm_control_state`.
- Add `folder.privileged_access_reason`.
- Add `share_link.soc2_review_state`.
- Add `storage_route.config_change_id`.
- Add `object_store.durability_evidence_id`.
- Add `admin_action.approval_chain_hash`.
- Add `export_job.auditor_redaction_profile`.
- Add `tenant_drive_config.soc2_audit_scope`.

## Cedar Policy Deltas
- Policy `SOC2-drive-admin-01`: require approved case for privileged file access.
- Policy `SOC2-drive-admin-02`: forbid privileged access when review is overdue.
- Policy `SOC2-drive-share-01`: require periodic review for external share links.
- Policy `SOC2-drive-share-02`: forbid public link for confidential files.
- Policy `SOC2-drive-export-01`: require redaction profile for auditor export.
- Policy `SOC2-drive-export-02`: forbid raw file export unless tenant approval exists.
- Policy `SOC2-drive-control-01`: require evidence hash for completed control test.
- Policy `SOC2-drive-control-02`: forbid exception closure without mitigation.
- Policy `SOC2-drive-change-01`: require change ticket for storage route change.
- Policy `SOC2-drive-backup-01`: require backup snapshot evidence for scoped files.
- Policy `SOC2-drive-restore-01`: permit restore drill in audit window or incident.
- Policy `SOC2-drive-dlp-01`: require DLP verdict before external confidential share.
- Policy `SOC2-drive-integrity-01`: require integrity hash before object promotion.
- Policy `SOC2-drive-worm-01`: require WORM evidence when immutability control selected.
- Policy `SOC2-drive-availability-01`: require queue and object-store SLO evidence.
- Policy `SOC2-drive-privacy-01`: require privacy evidence when Privacy TSC selected.
- Policy `SOC2-drive-confidentiality-01`: require encryption proof for confidential export.
- Policy `SOC2-drive-vendor-01`: require object-store vendor evidence if external.
- Policy `SOC2-drive-sample-01`: permit auditor sample only through redacted view.
- Policy `SOC2-drive-retention-01`: forbid retention change without approval.
- Policy `SOC2-drive-access-01`: require unique principal id for file operations.
- Policy `SOC2-drive-session-01`: require MFA for admin file changes.
- Policy `SOC2-drive-monitoring-01`: require alert route for storage SLO breach.
- Policy `SOC2-drive-pack-01`: forbid pack deactivation during audit period.

## API Contract Deltas
- `POST /admin/files/{id}/access` requires support case or change ticket.
- `POST /share-links/{id}/review` records review decision.
- `POST /auditor/exports` requires redaction profile.
- `GET /auditor/exports/{id}` returns evidence hash and sample seed.
- `POST /controls/tests` records selected TSC category.
- `PATCH /controls/exceptions/{id}` requires mitigation or acceptance.
- `POST /storage-routes` requires change ticket.
- `POST /restore-drills` requires audit window id.
- `POST /integrity/verify` records hash verification.
- `POST /dlp/verdicts` stores control verdict id.
- `GET /access-reviews/{id}` returns share and admin review status.
- `POST /incidents/{id}/drive-evidence` links file evidence by hash.
- `GET /availability/evidence` returns storage SLO data.
- `GET /privacy/evidence` returns Privacy TSC evidence when enabled.
- `POST /retention/rules` requires approval id.
- `GET /vendor/object-store/evidence` returns provider evidence.
- `GET /admin/actions` requires MFA-authenticated caller.
- `PATCH /tenant-drive-config` records SOC 2 audit scope.
- `POST /pack/deactivate` refuses active audit window.
- `GET /worm/evidence` returns immutability proof.

## Workflow Deltas
- Quarterly access review enumerates drive admins and share links.
- Privileged file access creates support-case evidence.
- Storage route change workflow requires approval and rollback proof.
- DLP operating-effectiveness test samples external shares.
- Integrity workflow verifies object hashes before promotion.
- Restore drill workflow proves backup availability.
- Auditor export workflow defaults to redacted metadata.
- Control exception workflow tracks mitigation and acceptance.
- Incident linkage workflow preserves evidence hash.
- Object-store vendor review refreshes durability evidence.
- Retention-rule change workflow requires owner approval.
- WORM control workflow records immutability evidence.
- Confidentiality review verifies encryption posture.
- Privacy review verifies subject-rights file handling evidence.
- Processing Integrity review checks no duplicate or corrupt versions.
- Availability review consumes storage and restore metrics.
- Common Criteria review checks unique user and MFA evidence.
- Audit period close freezes sample-selection seed.
- Pack deactivation waits for audit-period close.
- Evidence bundle publication signs manifest into audit-chain.

## SLO Deltas
- Privileged file access audit p99 must complete <= 1 second.
- Access review evidence freshness target is <= 24 hours.
- DLP verdict storage p99 must complete <= 2 seconds.
- Integrity hash verification p99 target is <= 5 minutes for normal files.
- Restore drill evidence publication target is <= 24 hours.
- Auditor redacted export p99 target is <= 4 hours.
- Control exception creation p99 must complete <= 2 minutes.
- Storage route change evidence target is <= 15 minutes.
- Vendor object-store evidence refresh cadence is monthly.
- Access review cadence is quarterly.
- Share-link review cadence is quarterly.
- Sample seed publication target is <= 1 hour after period close.
- Incident evidence linkage p99 must complete <= 10 minutes.
- Confidential export encryption proof p99 must complete <= 5 minutes.
- Object promotion integrity proof p99 must stay <= 500 ms.
- SOC 2 drive dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `DriveSoc2PrivilegedAccessRequested` records case id.
- `DriveSoc2PrivilegedAccessGranted` records MFA and TTL.
- `DriveSoc2AccessReviewStarted` records cycle id.
- `DriveSoc2AccessReviewCompleted` records exceptions count.
- `DriveSoc2ShareLinkReviewed` records link id.
- `DriveSoc2StorageRouteChanged` records change ticket.
- `DriveSoc2DlpControlVerdictStored` records verdict id.
- `DriveSoc2IntegrityHashVerified` records file id.
- `DriveSoc2RestoreDrillCompleted` records snapshot id.
- `DriveSoc2AuditorExportCreated` records redaction profile.
- `DriveSoc2ControlExceptionOpened` records criterion id.
- `DriveSoc2ControlExceptionClosed` records mitigation.
- `DriveSoc2IncidentEvidenceLinked` records incident id.
- `DriveSoc2VendorStoreReviewed` records vendor id.
- `DriveSoc2RetentionRuleApproved` records approval id.
- `DriveSoc2WormEvidenceCaptured` records file id.
- `DriveSoc2SampleSeedFrozen` records audit period.
- `DriveSoc2EvidenceBundleSigned` records bundle hash.
- `DriveSoc2PackDeactivationDeferred` records audit period.
- `DriveSoc2AvailabilityBreachRecorded` records SLO id.

## Failure Modes specific to this pack
- Auditor export includes raw file; recovery is revoke export and regenerate redacted bundle.
- Access review is overdue; recovery is freeze new external share grants.
- Admin access lacks case id; recovery is terminate session and open exception.
- Storage route changed without ticket; recovery is rollback route.
- Integrity hash mismatch appears; recovery is quarantine object version.
- Restore drill fails; recovery is open availability exception.
- DLP verdict stream delayed; recovery is mark control degraded and block confidential share.
- Vendor object-store evidence expires; recovery is disable provider for scoped tenants.
- Control exception has no owner; recovery is assign drive control owner.
- Evidence hash mismatch appears; recovery is rebuild from audit-chain.
- Sample seed changes after freeze; recovery is void sample and freeze new seed.
- Pack deactivation requested mid-period; recovery is defer.
- Shared file owner leaves tenant; recovery is force ownership review.
- MFA status missing for admin action; recovery is deny action.
- Privacy TSC selected but DSAR evidence absent; recovery is page compliance.
- Availability metric missing tenant label; recovery is reject emission per ADR-0263.
- Confidential export lacks encryption proof; recovery is quarantine export.
- Incident evidence references tombstoned file; recovery is use audit-chain tombstone.
- Retention change approval expires; recovery is keep prior rule.
- WORM evidence missing; recovery is block immutability claim.

## Cross-µservice coordination
- `tenancy` provides tenant pack roster and audit-period scope.
- `identity` provides unique principal, MFA, and access-review subjects.
- `compliance` owns SOC 2 control catalog, exceptions, and auditor requests.
- `audit-chain` signs evidence hashes and control-period manifests.
- `observability` provides SLO evidence for availability criteria.
- `mail` coordinates attachment transfer evidence.
- `workflow-engine` runs access review, exception, restore drill, and evidence publication workflows.
- `policy-engine` loads all `SOC2-drive-*` fragments.
- `incident-response` provides incident ids for linked drive evidence.
- `admin-console` renders scoped evidence without raw files.
- `dlp-virus-scan` provides operating-effectiveness verdicts.
- `secrets` or OpenBao provides encryption proof.
- `storage` provides object-store durability evidence.
- `support` supplies approved case ids for privileged access.
- `data-warehouse` receives aggregate control metrics.
- `legal` defines auditor redaction profiles.
- `notification` routes review reminders.
- `vendor-management` supplies external object-store evidence.
- `release-engine` records drive service change evidence.
- `pack-registry` signs this SOC 2 drive overlay.
