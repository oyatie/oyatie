---
doc_class: CompliancePackOverlay
pack_id: SOC2-T2
microservice: calendar
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# calendar SOC 2 Compliance Pack Overlay

## Pack Identity
- Full pack name: SOC 2 Type II Trust Services Criteria calendar control overlay.
- Citing jurisdiction: AICPA attestation framework for service organizations.
- Version: SOC2-T2-2017-TSC-2022-POF-v1.
- Canonical source URL: https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022
- Cited framework: 2017 Trust Services Criteria with Revised Points of Focus 2022.
- Covered calendar surface: event writes, room bookings, invitation delivery, CalDAV sync, backups, restore drills, admin access, access reviews, and audit exports.
- Pack activation means calendar emits auditor-ready evidence for selected TSC categories.
- SOC 2 adds control proof; it does not create statutory privacy rights.
- Data classes include `CALENDAR_SOC2_EVIDENCE`, `CALENDAR_CONTROL_EXCEPTION`, and `CALENDAR_ACCESS_REVIEW_RECORD`.
- Type II evidence must prove control operation across the period.
- ADR-0064 keeps controls in an overlay.
- ADR-0251 supplies pack signature and evidence retention.
- ADR-0263 supplies SLO evidence and audit linkage.
- Raw event details are redacted from auditor exports by default.
- This overlay excludes PCI-DSS because calendar is not cardholder-data processing.

## Data Model Deltas
- Add `calendar_control.control_period_id`.
- Add `calendar_control.trust_service_categories`.
- Add `calendar_control.cc_mapping`.
- Add `calendar_control.owner_team`.
- Add `calendar_control.test_frequency`.
- Add `calendar_control.last_tested_at`.
- Add `calendar_control.exception_state`.
- Add `calendar_control.exception_ticket_id`.
- Add `calendar_control.evidence_hash`.
- Add `calendar_control.sample_selection_seed`.
- Add `event.delivery_integrity_hash`.
- Add `event.access_review_cycle_id`.
- Add `event.backup_snapshot_id`.
- Add `event.restore_drill_id`.
- Add `event.invitation_delivery_evidence_id`.
- Add `event.room_conflict_control_verdict_id`.
- Add `event.caldav_sync_control_state`.
- Add `room.booking_review_due_at`.
- Add `recurrence.processing_integrity_hash`.
- Add `caldav_route.config_change_id`.
- Add `admin_action.approval_chain_hash`.
- Add `export_job.auditor_redaction_profile`.
- Add `tenant_calendar_config.soc2_audit_scope`.
- Add `availability_cache.integrity_verified_at`.

## Cedar Policy Deltas
- Policy `SOC2-calendar-admin-01`: require approved case for privileged event access.
- Policy `SOC2-calendar-admin-02`: forbid admin action when review is overdue.
- Policy `SOC2-calendar-room-01`: require conflict control verdict for room booking.
- Policy `SOC2-calendar-invite-01`: require delivery evidence for external invitations.
- Policy `SOC2-calendar-export-01`: require redaction profile for auditor export.
- Policy `SOC2-calendar-export-02`: forbid raw event export unless tenant approves.
- Policy `SOC2-calendar-control-01`: require evidence hash for control test completion.
- Policy `SOC2-calendar-control-02`: forbid exception closure without mitigation.
- Policy `SOC2-calendar-change-01`: require change ticket for CalDAV route update.
- Policy `SOC2-calendar-backup-01`: require backup snapshot evidence.
- Policy `SOC2-calendar-restore-01`: permit restore drill in audit window or incident.
- Policy `SOC2-calendar-integrity-01`: require processing integrity hash for recurrence rebuild.
- Policy `SOC2-calendar-availability-01`: require availability SLO evidence for freebusy cache.
- Policy `SOC2-calendar-privacy-01`: require privacy evidence when Privacy TSC selected.
- Policy `SOC2-calendar-confidentiality-01`: require encryption proof for confidential exports.
- Policy `SOC2-calendar-vendor-01`: require external CalDAV provider evidence.
- Policy `SOC2-calendar-sample-01`: permit auditor sample only through redacted view.
- Policy `SOC2-calendar-retention-01`: forbid retention change without approval.
- Policy `SOC2-calendar-access-01`: require unique principal id for event operations.
- Policy `SOC2-calendar-session-01`: require MFA for admin calendar changes.
- Policy `SOC2-calendar-monitoring-01`: require alert route for freebusy SLO breach.
- Policy `SOC2-calendar-delegate-01`: require periodic delegate review.
- Policy `SOC2-calendar-pack-01`: forbid pack deactivation during audit period.
- Policy `SOC2-calendar-sync-01`: require sync integrity evidence for CalDAV replay.

## API Contract Deltas
- `POST /admin/events/{id}/access` requires support case or change ticket.
- `POST /rooms/{id}/book` records conflict control verdict.
- `POST /auditor/exports` requires redaction profile.
- `GET /auditor/exports/{id}` returns evidence hash and sample seed.
- `POST /controls/tests` records selected TSC category.
- `PATCH /controls/exceptions/{id}` requires mitigation or acceptance.
- `POST /caldav/routes` requires change ticket.
- `POST /restore-drills` requires audit window id.
- `POST /recurrence/rebuild` records integrity hash.
- `POST /invitations/{id}/delivery-evidence` stores delivery evidence.
- `GET /access-reviews/{id}` returns delegate and admin review status.
- `POST /incidents/{id}/calendar-evidence` links evidence by hash.
- `GET /availability/evidence` returns freebusy SLO data.
- `GET /privacy/evidence` returns Privacy TSC evidence when enabled.
- `POST /retention/rules` requires approval id.
- `GET /vendor/caldav/evidence` returns provider evidence.
- `GET /admin/actions` requires MFA-authenticated caller.
- `PATCH /tenant-calendar-config` records SOC 2 audit scope.
- `POST /pack/deactivate` refuses active audit window.
- `GET /sync/evidence` returns CalDAV replay proof.

## Workflow Deltas
- Quarterly access review enumerates calendar admins and delegates.
- Privileged event access creates support-case evidence.
- Room booking workflow records conflict-control verdict.
- Invitation delivery workflow stores delivery evidence.
- CalDAV route change workflow requires approval and rollback proof.
- Recurrence rebuild workflow verifies processing integrity.
- Restore drill workflow proves backup availability.
- Auditor export workflow defaults to redacted event metadata.
- Control exception workflow tracks mitigation and acceptance.
- Incident linkage workflow preserves evidence hash.
- External CalDAV provider review refreshes vendor evidence.
- Retention-rule change workflow requires owner approval.
- Availability review consumes freebusy cache metrics.
- Processing Integrity review checks recurrence and invite state.
- Confidentiality review verifies encryption posture.
- Privacy review verifies subject-rights calendar evidence.
- Common Criteria review checks unique user and MFA evidence.
- Audit period close freezes sample seed.
- Pack deactivation waits for audit-period close.
- Evidence bundle publication signs manifest into audit-chain.

## SLO Deltas
- Privileged event access audit p99 must complete <= 1 second.
- Access review evidence freshness target is <= 24 hours.
- Room conflict verdict storage p99 must complete <= 1 second.
- Invitation delivery evidence p99 must complete <= 5 seconds.
- Recurrence integrity verification p99 target is <= 5 minutes.
- Restore drill evidence publication target is <= 24 hours.
- Auditor redacted export p99 target is <= 4 hours.
- Control exception creation p99 must complete <= 2 minutes.
- CalDAV route change evidence target is <= 15 minutes.
- Vendor CalDAV evidence refresh cadence is monthly.
- Access review cadence is quarterly.
- Delegate review cadence is quarterly.
- Sample seed publication target is <= 1 hour after period close.
- Incident evidence linkage p99 must complete <= 10 minutes.
- Availability SLO dashboard lag target is <= 15 minutes.
- SOC 2 calendar dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `CalendarSoc2PrivilegedAccessRequested` records case id.
- `CalendarSoc2PrivilegedAccessGranted` records MFA and TTL.
- `CalendarSoc2AccessReviewStarted` records cycle id.
- `CalendarSoc2AccessReviewCompleted` records exceptions count.
- `CalendarSoc2DelegateReviewed` records delegate id.
- `CalendarSoc2RoomConflictVerdictStored` records booking id.
- `CalendarSoc2InvitationDeliveryEvidenceStored` records invite id.
- `CalendarSoc2CaldavRouteChanged` records change ticket.
- `CalendarSoc2RecurrenceIntegrityVerified` records recurrence id.
- `CalendarSoc2RestoreDrillCompleted` records snapshot id.
- `CalendarSoc2AuditorExportCreated` records redaction profile.
- `CalendarSoc2ControlExceptionOpened` records criterion id.
- `CalendarSoc2ControlExceptionClosed` records mitigation.
- `CalendarSoc2IncidentEvidenceLinked` records incident id.
- `CalendarSoc2VendorCaldavReviewed` records provider id.
- `CalendarSoc2RetentionRuleApproved` records approval id.
- `CalendarSoc2SampleSeedFrozen` records audit period.
- `CalendarSoc2EvidenceBundleSigned` records bundle hash.
- `CalendarSoc2PackDeactivationDeferred` records audit period.
- `CalendarSoc2AvailabilityBreachRecorded` records SLO id.

## Failure Modes specific to this pack
- Auditor export includes raw event detail; recovery is revoke and regenerate redacted bundle.
- Access review overdue; recovery is freeze new delegate grants.
- Admin access lacks case id; recovery is terminate session and open exception.
- Room conflict control missing; recovery is block booking.
- Invitation delivery evidence missing; recovery is retry and mark control degraded.
- CalDAV route changed without ticket; recovery is rollback route.
- Recurrence rebuild lacks integrity hash; recovery is halt rebuild.
- Restore drill fails; recovery is open availability exception.
- Vendor CalDAV evidence expires; recovery is disable provider for scoped tenants.
- Control exception has no owner; recovery is assign calendar control owner.
- Evidence hash mismatch appears; recovery is rebuild from audit-chain.
- Sample seed changes after freeze; recovery is void sample.
- Pack deactivation requested mid-period; recovery is defer.
- Delegate owner leaves tenant; recovery is force delegate review.
- MFA status missing for admin action; recovery is deny.
- Privacy TSC selected but DSAR evidence absent; recovery is page compliance.
- Availability metric missing tenant label; recovery is reject emission.
- Incident evidence references tombstoned event; recovery is use audit-chain tombstone.
- Retention change approval expires; recovery is keep prior rule.
- Sync replay proof missing; recovery is suspend CalDAV replay.

## Cross-µservice coordination
- `tenancy` provides tenant pack roster and audit-period scope.
- `identity` provides unique principal, MFA, and access-review subjects.
- `compliance` owns SOC 2 control catalog, exceptions, and auditor requests.
- `audit-chain` signs evidence hashes and period manifests.
- `observability` provides SLO evidence for freebusy and sync criteria.
- `mail` coordinates invitation delivery evidence.
- `drive` stores redacted auditor export bundles.
- `workflow-engine` runs access review, exception, restore drill, and evidence workflows.
- `policy-engine` loads all `SOC2-calendar-*` fragments.
- `incident-response` provides incident ids for linked calendar evidence.
- `admin-console` renders scoped evidence without raw event content.
- `secrets` provides encryption proof for exports.
- `support` supplies approved case ids for privileged access.
- `data-warehouse` receives aggregate control metrics.
- `legal` defines auditor redaction profiles.
- `notification` routes review reminders.
- `vendor-management` supplies external CalDAV evidence.
- `release-engine` records calendar service change evidence.
- `meet` or room service provides room conflict evidence.
- `pack-registry` signs this SOC 2 calendar overlay.
