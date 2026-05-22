---
doc_class: CompliancePackOverlay
pack_id: SOC2-T2
microservice: mail
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# mail SOC 2 Compliance Pack Overlay

## Pack Identity
- Full pack name: SOC 2 Type II Trust Services Criteria mail control overlay.
- Citing jurisdiction: AICPA attestation framework for service organizations.
- Version: SOC2-T2-2017-TSC-2022-POF-v1.
- Canonical source URL: https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022
- Cited framework: 2017 Trust Services Criteria with Revised Points of Focus 2022.
- Covered mail surface: mail delivery, mailbox administration, DLP, retention, access reviews, change evidence, incident evidence, and customer audit exports.
- Pack activation means mail produces auditor-ready evidence for Security, Availability, Confidentiality, Processing Integrity, and Privacy categories as selected by the tenant.
- SOC 2 does not alter statutory rights by itself; it adds control evidence and operating-effectiveness proof.
- Data classes include `MAIL_SOC2_EVIDENCE`, `MAIL_CONTROL_EXCEPTION`, and `MAIL_ACCESS_REVIEW_RECORD`.
- Type II evidence is sampled across time, so this overlay stores control-period metadata on mail events.
- ADR-0064 keeps SOC 2 as an overlay rather than a separate mail implementation.
- ADR-0251 supplies pack signature, evidence retention, and cell eligibility.
- ADR-0263 supplies metric, log, trace, exemplar, and audit-id evidence.
- Mail must preserve control evidence without exposing message contents to auditors by default.
- This overlay excludes PCI-DSS because SOC 2 audit evidence is not cardholder-data processing.

## Data Model Deltas
- Add `mail_control.control_period_id` to group evidence by audit window.
- Add `mail_control.trust_service_categories` as array of selected TSC categories.
- Add `mail_control.cc_mapping` for Common Criteria linkage.
- Add `mail_control.owner_team` for evidence accountability.
- Add `mail_control.test_frequency` as enum `continuous|daily|weekly|monthly|quarterly`.
- Add `mail_control.last_tested_at` timestamp.
- Add `mail_control.exception_state` as enum `none|open|mitigated|accepted`.
- Add `mail_control.exception_ticket_id` for remediation tracking.
- Add `mail_control.evidence_hash` for immutable evidence bundles.
- Add `mail_control.sample_selection_seed` for auditor sampling.
- Add `mailbox.access_review_cycle_id` for quarterly review evidence.
- Add `mailbox.privileged_access_reason` for admin access.
- Add `mailbox.delegation_review_due_at` for shared mailbox delegation.
- Add `message.delivery_integrity_hash` for processing integrity proof.
- Add `message.retention_policy_version` for confidentiality and privacy controls.
- Add `message.dlp_control_verdict_id` for DLP operating evidence.
- Add `message.incident_linked` as boolean when incident evidence references mail.
- Add `message.backup_snapshot_id` for availability restore proof.
- Add `message.queue_delay_bucket` for availability SLO evidence.
- Add `dkim_key.rotation_evidence_id` for change-control proof.
- Add `smtp_route.config_change_id` for route change audit.
- Add `admin_action.approval_chain_hash` for privileged operations.
- Add `export_job.auditor_redaction_profile` to avoid raw message exposure.
- Add `tenant_mail_config.soc2_audit_scope` for selected criteria.

## Cedar Policy Deltas
- Policy `SOC2-mail-admin-01`: permit privileged mailbox access only with approved change or support case.
- Policy `SOC2-mail-admin-02`: forbid admin action when access review is overdue.
- Policy `SOC2-mail-delegation-01`: require quarterly review for shared mailbox delegation.
- Policy `SOC2-mail-export-01`: permit auditor export only with redaction profile.
- Policy `SOC2-mail-export-02`: forbid raw body export unless tenant explicitly approves.
- Policy `SOC2-mail-control-01`: require evidence hash for control test completion.
- Policy `SOC2-mail-control-02`: forbid closing exception without mitigation or risk acceptance.
- Policy `SOC2-mail-change-01`: require change ticket for SMTP route updates.
- Policy `SOC2-mail-change-02`: require DKIM rotation evidence before key retirement.
- Policy `SOC2-mail-incident-01`: permit incident linkage when incident id exists.
- Policy `SOC2-mail-dlp-01`: require DLP verdict before confidential outbound send.
- Policy `SOC2-mail-backup-01`: require backup snapshot before mailbox migration.
- Policy `SOC2-mail-restore-01`: permit restore drill only in audit test window or incident.
- Policy `SOC2-mail-availability-01`: forbid planned maintenance outside approved window.
- Policy `SOC2-mail-processing-01`: require delivery integrity hash for queue replay.
- Policy `SOC2-mail-privacy-01`: require privacy category evidence when tenant selects Privacy TSC.
- Policy `SOC2-mail-confidentiality-01`: require encryption proof for confidential mailbox exports.
- Policy `SOC2-mail-monitoring-01`: require alert route for queue delay breach.
- Policy `SOC2-mail-vendor-01`: require vendor evidence for external SMTP relay.
- Policy `SOC2-mail-sample-01`: permit auditor sample read only through redacted view.
- Policy `SOC2-mail-retention-01`: forbid retention-rule change without approval.
- Policy `SOC2-mail-access-01`: require unique principal id for mailbox operations.
- Policy `SOC2-mail-session-01`: require MFA for admin console mail changes.
- Policy `SOC2-mail-pack-01`: forbid pack deactivation during open audit period.

## API Contract Deltas
- `POST /admin/mailboxes/{id}/access` requires `support_case_id` or `change_ticket_id`.
- `POST /shared-mailboxes/{id}/delegates` returns access-review cycle id.
- `POST /auditor/exports` requires `redaction_profile`.
- `GET /auditor/exports/{id}` returns evidence hash and sample seed.
- `POST /controls/tests` records selected TSC category.
- `PATCH /controls/exceptions/{id}` requires mitigation or acceptance reason.
- `POST /smtp-routes` requires change ticket.
- `PATCH /dkim-keys/{id}` requires rotation evidence.
- `POST /restore-drills` requires audit window id.
- `POST /queue/replay` requires delivery integrity hash.
- `POST /dlp/verdicts` stores control verdict id.
- `GET /access-reviews/{id}` returns delegate and admin review status.
- `POST /incidents/{id}/mail-evidence` links mail evidence by hash.
- `GET /availability/evidence` returns queue delay buckets.
- `GET /privacy/evidence` returns Privacy TSC evidence when enabled.
- `POST /retention/rules` requires approval id.
- `GET /vendor/smtp-relays/evidence` returns relay compliance evidence.
- `GET /admin/actions` requires MFA-authenticated caller.
- `PATCH /tenant-mail-config` records SOC 2 audit scope.
- `POST /pack/deactivate` refuses during active audit window.

## Workflow Deltas
- Quarterly access review enumerates mail admins and shared mailbox delegates.
- Privileged mailbox access creates support-case evidence.
- DKIM key rotation workflow stores before-and-after proofs.
- SMTP route change workflow requires approval and rollback proof.
- DLP operating-effectiveness test samples outbound messages.
- Queue replay workflow records delivery integrity hashes.
- Restore drill workflow proves backup availability.
- Auditor export workflow defaults to redacted message metadata.
- Control exception workflow tracks mitigation and acceptance.
- Incident linkage workflow preserves evidence hash instead of raw bodies.
- Vendor relay review workflow refreshes external SMTP evidence.
- Retention-rule change workflow requires owner approval.
- Confidentiality review verifies encryption posture for exports.
- Privacy review verifies subject-rights handling evidence.
- Processing Integrity review verifies no duplicate message delivery.
- Availability review consumes queue delay and restore metrics.
- Common Criteria review checks unique user and MFA evidence.
- Audit period close workflow freezes sample-selection seed.
- Pack deactivation workflow waits for audit-period close.
- Evidence bundle publication signs manifest into audit-chain.

## SLO Deltas
- Privileged mail access audit emission p99 must complete <= 1 second.
- Access review evidence freshness target is <= 24 hours.
- DLP control verdict storage p99 must complete <= 2 seconds.
- Queue delay control metric lag target is <= 5 minutes.
- Restore drill evidence publication target is <= 24 hours.
- Auditor redacted export p99 target is <= 4 hours.
- Control exception creation p99 must complete <= 2 minutes.
- DKIM rotation evidence publication target is <= 15 minutes.
- SMTP route change evidence target is <= 15 minutes.
- Vendor relay evidence refresh cadence is monthly.
- Access review cadence is quarterly.
- Control sampling seed publication target is <= 1 hour after audit period close.
- Incident evidence linkage p99 must complete <= 10 minutes.
- Confidential export encryption proof p99 must complete <= 5 minutes.
- Mail delivery integrity hash creation p99 must stay <= 100 ms.
- SOC 2 dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `MailSoc2PrivilegedAccessRequested` records case and approver.
- `MailSoc2PrivilegedAccessGranted` records MFA and TTL.
- `MailSoc2AccessReviewStarted` records cycle id.
- `MailSoc2AccessReviewCompleted` records exceptions count.
- `MailSoc2SharedMailboxDelegationChanged` records review due date.
- `MailSoc2DkimRotationEvidenceCaptured` records key id.
- `MailSoc2SmtpRouteChanged` records change ticket.
- `MailSoc2DlpControlVerdictStored` records verdict id.
- `MailSoc2QueueReplayVerified` records integrity hash.
- `MailSoc2RestoreDrillCompleted` records snapshot id.
- `MailSoc2AuditorExportCreated` records redaction profile.
- `MailSoc2ControlExceptionOpened` records criterion id.
- `MailSoc2ControlExceptionClosed` records mitigation.
- `MailSoc2IncidentEvidenceLinked` records incident id.
- `MailSoc2VendorRelayReviewed` records vendor id.
- `MailSoc2RetentionRuleApproved` records approval id.
- `MailSoc2SampleSeedFrozen` records audit period.
- `MailSoc2EvidenceBundleSigned` records bundle hash.
- `MailSoc2PackDeactivationDeferred` records audit period.
- `MailSoc2AvailabilityBreachRecorded` records SLO id.

## Failure Modes specific to this pack
- Auditor export includes raw body accidentally; recovery is revoke export and regenerate redacted bundle.
- Access review is overdue; recovery is freeze new delegated mailbox grants.
- Admin access lacks case id; recovery is terminate session and open exception.
- DKIM rotation evidence missing; recovery is pause key retirement.
- SMTP route changed without ticket; recovery is rollback route and open control exception.
- Queue replay lacks integrity hash; recovery is halt replay and regenerate from immutable log.
- Restore drill fails; recovery is open availability exception and rerun after fix.
- DLP verdict stream is delayed; recovery is mark control degraded and block confidential sends if needed.
- Vendor relay evidence expires; recovery is disable relay for SOC 2 scoped tenants.
- Control exception has no owner; recovery is assign to mail control owner automatically.
- Evidence hash mismatch appears; recovery is rebuild bundle from audit-chain source.
- Audit sample seed changes after freeze; recovery is void sample and freeze new signed seed.
- Pack deactivation requested mid-period; recovery is defer until audit window closes.
- Shared mailbox owner leaves; recovery is force delegation review.
- MFA status missing for admin action; recovery is deny and require fresh step-up.
- Privacy TSC selected but DSAR evidence absent; recovery is page compliance owner.
- Availability metric missing tenant label; recovery is reject emission per ADR-0263.
- Confidentiality export lacks encryption proof; recovery is quarantine export.
- Incident evidence references deleted message; recovery is use audit-chain tombstone proof.
- Retention change approval expires; recovery is keep prior rule.

## Cross-µservice coordination
- `tenancy` provides tenant pack roster and audit-period scope.
- `identity` provides unique principal, MFA, and access-review subject data.
- `compliance` owns SOC 2 control catalog, exceptions, and auditor export requests.
- `audit-chain` signs evidence hashes and control-period manifests.
- `observability` provides SLO evidence for availability and alerting criteria.
- `drive` stores redacted auditor export bundles under evidence retention.
- `workflow-engine` runs access review, exception, restore drill, and evidence publication workflows.
- `policy-engine` loads all `SOC2-mail-*` fragments atomically.
- `incident-response` provides incident ids for linked mail evidence.
- `admin-console` renders scoped SOC 2 evidence without raw message bodies.
- `dlp-virus-scan` provides operating-effectiveness verdict evidence.
- `secrets` or OpenBao provides DKIM key rotation proofs.
- `cloud-network` provides SMTP route change evidence.
- `support` supplies approved case ids for privileged access.
- `data-warehouse` receives only aggregate control metrics.
- `legal` defines auditor redaction profiles.
- `notification` routes access review reminders.
- `vendor-management` supplies external SMTP relay evidence.
- `release-engine` records mail service change evidence.
- `pack-registry` signs this SOC 2 mail overlay.
