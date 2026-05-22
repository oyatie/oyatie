---
doc_class: CompliancePackOverlay
pack_id: SOC2-T2
microservice: audit-chain
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# audit-chain SOC 2 Compliance Pack Overlay

## Pack Identity
- Full pack name: SOC 2 Type II Trust Services Criteria audit-chain control overlay.
- Citing jurisdiction: AICPA attestation framework for service organizations.
- Version: SOC2-T2-2017-TSC-2022-POF-v1.
- Canonical source URL: https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022
- Cited framework: 2017 Trust Services Criteria with Revised Points of Focus 2022.
- Covered audit-chain surface: event immutability, seal verification, key rotation, retention, replay, evidence exports, access reviews, and control-period manifests.
- Pack activation means audit-chain produces auditor-ready proof for its own controls and downstream service evidence.
- The overlay maps Merkle, signature, retention, and verification evidence to TSC criteria.
- Data classes include `AUDIT_CHAIN_SOC2_EVIDENCE`, `AUDIT_CHAIN_CONTROL_EXCEPTION`, and `AUDIT_CHAIN_SEAL_PROOF`.
- Type II evidence must be sampled and immutable across the audit period.
- ADR-0064 keeps SOC 2 behavior in an overlay.
- ADR-0251 supplies pack signature and retention.
- ADR-0263 supplies event emission linkage.
- Raw tenant event payloads are not auditor-exported by default.
- This overlay excludes PCI-DSS because SOC 2 proof is not cardholder-data processing.

## Data Model Deltas
- Add `audit_control.control_period_id`.
- Add `audit_control.trust_service_categories`.
- Add `audit_control.cc_mapping`.
- Add `audit_control.owner_team`.
- Add `audit_control.test_frequency`.
- Add `audit_control.last_tested_at`.
- Add `audit_control.exception_state`.
- Add `audit_control.exception_ticket_id`.
- Add `audit_control.evidence_hash`.
- Add `audit_control.sample_selection_seed`.
- Add `seal_batch.control_period_id`.
- Add `seal_batch.verification_evidence_id`.
- Add `seal_batch.integrity_hash`.
- Add `signature.key_rotation_evidence_id`.
- Add `retention_profile.control_state`.
- Add `replay_job.processing_integrity_hash`.
- Add `export_job.auditor_redaction_profile`.
- Add `query_session.access_review_cycle_id`.
- Add `storage_segment.durability_evidence_id`.
- Add `tamper_check.control_verdict_id`.
- Add `admin_action.approval_chain_hash`.
- Add `audit_shadow.audit_chain_soc2_event_id`.
- Add `tenant_audit_chain_config.soc2_audit_scope`.
- Add `tenant_audit_chain_config.control_period_lock`.

## Cedar Policy Deltas
- Policy `SOC2-audit-chain-admin-01`: require approved case for privileged event query.
- Policy `SOC2-audit-chain-admin-02`: forbid admin query when access review is overdue.
- Policy `SOC2-audit-chain-seal-01`: require verification evidence for seal batch.
- Policy `SOC2-audit-chain-key-01`: require key rotation evidence before key retirement.
- Policy `SOC2-audit-chain-retention-01`: require retention control state before profile change.
- Policy `SOC2-audit-chain-replay-01`: require processing integrity hash for replay job.
- Policy `SOC2-audit-chain-export-01`: require redaction profile for auditor export.
- Policy `SOC2-audit-chain-export-02`: forbid raw payload export unless tenant approves.
- Policy `SOC2-audit-chain-control-01`: require evidence hash for control test completion.
- Policy `SOC2-audit-chain-control-02`: forbid exception closure without mitigation.
- Policy `SOC2-audit-chain-period-01`: forbid evidence mutation after control-period lock.
- Policy `SOC2-audit-chain-sample-01`: require sample seed for auditor sampling.
- Policy `SOC2-audit-chain-storage-01`: require durability evidence for storage segment.
- Policy `SOC2-audit-chain-tamper-01`: require tamper verdict on control cadence.
- Policy `SOC2-audit-chain-vendor-01`: require HSM or storage vendor evidence if external.
- Policy `SOC2-audit-chain-access-01`: require unique principal id for audit operations.
- Policy `SOC2-audit-chain-session-01`: require MFA for admin changes.
- Policy `SOC2-audit-chain-monitoring-01`: require alert route for seal verification breach.
- Policy `SOC2-audit-chain-change-01`: require change ticket for schema or retention changes.
- Policy `SOC2-audit-chain-compaction-01`: require lineage proof for segment compaction.
- Policy `SOC2-audit-chain-backfill-01`: require backfill evidence and replay hash.
- Policy `SOC2-audit-chain-pack-01`: forbid pack deactivation during audit period.
- Policy `SOC2-audit-chain-evidence-01`: require signed control-period manifest.
- Policy `SOC2-audit-chain-support-01`: require support case for evidence inspection.

## API Contract Deltas
- `POST /admin/query` requires support case or change ticket.
- `POST /seal-batches/{id}/verify` records verification evidence.
- `POST /keys/{id}/rotate` records key rotation evidence.
- `POST /retention-profiles` requires control state.
- `POST /replay` requires processing integrity hash.
- `POST /auditor/exports` requires redaction profile.
- `GET /auditor/exports/{id}` returns evidence hash and sample seed.
- `POST /controls/tests` records selected TSC category.
- `PATCH /controls/exceptions/{id}` requires mitigation or acceptance.
- `PATCH /control-periods/{id}/lock` freezes evidence mutation.
- `POST /samples` stores sample seed.
- `POST /storage-segments/{id}/durability-evidence` records durability proof.
- `POST /tamper-checks` records control verdict.
- `GET /vendor/hsm/evidence` returns provider evidence.
- `GET /admin/actions` requires MFA-authenticated caller.
- `POST /schema-changes` requires change ticket.
- `POST /compaction` requires lineage proof.
- `POST /backfill` requires backfill evidence.
- `PATCH /tenant-audit-chain-config` records SOC 2 audit scope.
- `POST /pack/deactivate` refuses active audit window.

## Workflow Deltas
- Quarterly access review enumerates audit-chain admins and evidence viewers.
- Privileged event query creates support-case evidence.
- Seal verification workflow stores tamper verdict.
- Key rotation workflow stores before-and-after evidence.
- Retention profile workflow records control state.
- Replay workflow verifies processing integrity hash.
- Auditor export workflow defaults to redacted event metadata.
- Control exception workflow tracks mitigation and acceptance.
- Control-period workflow locks evidence mutation.
- Sample workflow freezes deterministic sample seed.
- Storage durability workflow records segment proof.
- HSM or storage vendor review refreshes evidence.
- Schema change workflow requires ticket and rollback plan.
- Compaction workflow records lineage proof.
- Backfill workflow records replay and integrity evidence.
- Alert workflow pages on seal verification breach.
- Common Criteria review checks unique user and MFA evidence.
- Audit period close signs control-period manifest.
- Pack deactivation waits for audit-period close.
- Evidence bundle publication signs manifest into audit-chain itself.

## SLO Deltas
- Privileged audit query audit p99 must complete <= 1 second.
- Access review evidence freshness target is <= 24 hours.
- Seal verification evidence p99 target is <= 5 minutes after batch close.
- Key rotation evidence publication target is <= 15 minutes.
- Retention control state publication target is <= 15 minutes.
- Replay integrity verification p99 target is <= 10 minutes.
- Auditor redacted export p99 target is <= 4 hours.
- Control exception creation p99 must complete <= 2 minutes.
- Control-period lock p99 target is <= 5 minutes.
- Sample seed publication target is <= 1 hour after period close.
- Storage durability evidence refresh cadence is daily.
- Tamper control verdict cadence is hourly.
- Vendor evidence refresh cadence is monthly.
- Schema change evidence propagation p99 target is <= 5 minutes.
- Seal breach alert target is <= 5 minutes.
- SOC 2 audit-chain dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `AuditChainSoc2PrivilegedQueryRequested` records case id.
- `AuditChainSoc2PrivilegedQueryGranted` records MFA and TTL.
- `AuditChainSoc2AccessReviewStarted` records cycle id.
- `AuditChainSoc2AccessReviewCompleted` records exceptions count.
- `AuditChainSoc2SealBatchVerified` records verification id.
- `AuditChainSoc2KeyRotationEvidenceStored` records key id.
- `AuditChainSoc2RetentionControlStateStored` records profile id.
- `AuditChainSoc2ReplayIntegrityVerified` records replay id.
- `AuditChainSoc2AuditorExportCreated` records redaction profile.
- `AuditChainSoc2ControlExceptionOpened` records criterion id.
- `AuditChainSoc2ControlExceptionClosed` records mitigation.
- `AuditChainSoc2ControlPeriodLocked` records period id.
- `AuditChainSoc2SampleSeedFrozen` records audit period.
- `AuditChainSoc2StorageDurabilityStored` records segment id.
- `AuditChainSoc2TamperVerdictStored` records verdict id.
- `AuditChainSoc2VendorEvidenceStored` records vendor id.
- `AuditChainSoc2SchemaChangeRecorded` records change ticket.
- `AuditChainSoc2CompactionLineageProved` records segment id.
- `AuditChainSoc2BackfillEvidenceStored` records backfill id.
- `AuditChainSoc2PackDeactivationDeferred` records audit period.

## Failure Modes specific to this pack
- Auditor export includes raw payload; recovery is revoke and regenerate redacted bundle.
- Access review overdue; recovery is freeze new evidence-viewer grants.
- Admin query lacks case id; recovery is terminate session and open exception.
- Seal verification evidence missing; recovery is rerun verification and mark control degraded.
- Key rotation evidence missing; recovery is pause key retirement.
- Retention control state missing; recovery is reject profile change.
- Replay lacks integrity hash; recovery is halt replay.
- Evidence mutation attempted after period lock; recovery is reject mutation.
- Sample seed missing; recovery is block auditor sampling.
- Storage durability evidence stale; recovery is mark availability control degraded.
- Tamper verdict overdue; recovery is page audit-chain owner.
- Vendor evidence expires; recovery is mark external reliance degraded.
- Schema change lacks ticket; recovery is rollback change.
- Compaction lineage proof missing; recovery is block compaction.
- Backfill evidence missing; recovery is block backfill.
- Pack deactivation requested mid-period; recovery is defer.
- MFA status missing for admin action; recovery is deny.
- Evidence hash mismatch appears; recovery is rebuild from immutable segments.
- Seal breach alert route missing; recovery is page SRE through fallback route.
- Audit-chain self-seal recursion fails; recovery is write emergency local seal and reconcile.

## Cross-µservice coordination
- `tenancy` provides tenant pack roster and audit-period scope.
- `identity` provides unique principal, MFA, and access-review subjects.
- `compliance` owns SOC 2 control catalog, exceptions, and auditor requests.
- `observability` provides SLO and tamper alert evidence.
- `policy-engine` loads all `SOC2-audit-chain-*` fragments.
- `workflow-engine` runs access review, exception, and evidence workflows.
- `storage` provides segment durability evidence.
- `cloud-kms` or OpenBao provides key rotation proof.
- `incident-response` consumes seal breach incidents.
- `admin-console` renders scoped evidence without raw payloads.
- `support` supplies approved case ids for privileged queries.
- `data-warehouse` receives aggregate audit health metrics.
- `legal` defines auditor redaction profiles.
- `notification` routes control and seal alerts.
- `vendor-management` supplies HSM and storage provider evidence.
- `release-engine` records schema and retention change evidence.
- `mail` relies on audit-chain SOC 2 seal evidence.
- `drive` relies on audit-chain SOC 2 seal evidence.
- `calendar` relies on audit-chain SOC 2 seal evidence.
- `pack-registry` signs this SOC 2 audit-chain overlay.
