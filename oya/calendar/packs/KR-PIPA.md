---
doc_class: CompliancePackOverlay
pack_id: KR-PIPA-2023-amendment
microservice: calendar
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# calendar KR-PIPA Compliance Pack Overlay

## Pack Identity
- Full pack name: Korea Personal Information Protection Act calendar overlay.
- Citing jurisdiction: Republic of Korea personal information regime.
- Version: KR-PIPA-2023-amendment-v1.
- Canonical source URL: https://law.go.kr/LSW/lsInfoP.do?lsId=011357
- Cited law: 개인정보 보호법, Act No. 17799 baseline with current consolidation at law.go.kr.
- Covered calendar surface: events, attendees, availability, reminders, rooms, scheduling links, ICS imports, CalDAV sync, retention, and subject-rights workflows.
- Pack activation means calendar captures 동의, 보존, 국외이전, and 처리위탁 evidence for Korean personal information.
- Korean resident registration numbers are forbidden in event title, location, reminder, and freebusy payloads.
- Data classes include `CALENDAR_PI_KR`, `CALENDAR_SENSITIVE_PI_KR`, `CALENDAR_RRN_KR`, and `CALENDAR_CONSENT_LEDGER_KR`.
- Korean-language notices are required for consent, transfer, breach, and subject-rights calendar flows.
- ADR-0064 keeps Korean scheduling semantics in the overlay.
- ADR-0251 supplies pack admission, breach workflow, and retention ledger hooks.
- ADR-0263 requires Korean PI scrubbing in calendar telemetry.
- PCI-DSS is omitted because calendar does not process card authorization.
- Payment event notes are confidential text and routed to DLP if card-like data appears.

## Data Model Deltas
- Add `event.kr_pi_signal` as enum `none|personal|sensitive|rrn`.
- Add `event.kr_consent_id` for 동의 ledger linkage.
- Add `event.kr_processing_purpose_id`.
- Add `event.kr_retention_basis_id` for 보존 proof.
- Add `event.kr_retention_until` timestamp.
- Add `event.kr_cross_border_transfer_id`.
- Add `event.kr_processor_delegation_id`.
- Add `event.kr_notice_language` default `ko-KR`.
- Add `event.kr_subject_rights_case_id`.
- Add `event.kr_breach_clock_started_at`.
- Add `event.kr_title_rrn_blocked` boolean.
- Add `attendee.kr_subject_hash`.
- Add `freebusy.kr_projection_level`.
- Add `reminder.kr_pi_scrubbed` boolean.
- Add `room.kr_processor_delegation_id`.
- Add `recurrence.kr_erasure_exception_map`.
- Add `ics_import.kr_source_consent_proof`.
- Add `caldav_sync.kr_processor_ref`.
- Add `search_index.kr_erasure_rebuild_required`.
- Add `consent_snapshot.kr_consent_text_hash`.
- Add `export_job.kr_subject_rights_manifest_hash`.
- Add `audit_shadow.kr_pipa_event_id`.
- Add `tenant_calendar_config.kr_pipa_notice_version`.
- Add `tenant_calendar_config.kr_retention_schedule_version`.

## Cedar Policy Deltas
- Policy `KRPIPA-calendar-create-01`: require processing purpose for KR personal events.
- Policy `KRPIPA-calendar-create-02`: require consent when purpose requires 동의.
- Policy `KRPIPA-calendar-create-03`: forbid RRN in title, location, and reminders.
- Policy `KRPIPA-calendar-read-01`: restrict sensitive PI read to approved purpose.
- Policy `KRPIPA-calendar-freebusy-01`: expose minimal freebusy for external attendees.
- Policy `KRPIPA-calendar-transfer-01`: forbid cross-border invite without transfer id.
- Policy `KRPIPA-calendar-transfer-02`: require Korean transfer notice for non-KR attendee.
- Policy `KRPIPA-calendar-retention-01`: require 보존 ledger entry for retention.
- Policy `KRPIPA-calendar-erasure-01`: permit erasure after retention basis expires.
- Policy `KRPIPA-calendar-erasure-02`: forbid erasure while statutory retention or hold exists.
- Policy `KRPIPA-calendar-consent-01`: require consent text hash for consent-based processing.
- Policy `KRPIPA-calendar-consent-02`: forbid processing after consent withdrawal.
- Policy `KRPIPA-calendar-processor-01`: require 처리위탁 registry for delegated room or CalDAV provider.
- Policy `KRPIPA-calendar-admin-01`: require DPO-visible audit for admin event read.
- Policy `KRPIPA-calendar-ai-01`: require explicit consent before AI scheduling touches Korean PI.
- Policy `KRPIPA-calendar-search-01`: restrict sensitive PI search by purpose.
- Policy `KRPIPA-calendar-export-01`: require verified identity for subject-rights export.
- Policy `KRPIPA-calendar-breach-01`: start Korean breach workflow on confirmed calendar PI leak.
- Policy `KRPIPA-calendar-link-01`: require expiry for public scheduling links.
- Policy `KRPIPA-calendar-route-01`: require KR resident storage in approved KR cell.
- Policy `KRPIPA-calendar-preview-01`: scrub Korean PI from notification previews.
- Policy `KRPIPA-calendar-import-01`: require source consent proof for imported ICS.
- Policy `KRPIPA-calendar-pack-01`: defer deactivation while ledgers are open.
- Policy `KRPIPA-calendar-room-01`: require delegation proof for external room provider.

## API Contract Deltas
- `POST /events` requires `kr_processing_purpose_id` for KR tenants.
- `POST /events` requires `kr_consent_id` when purpose is consent-based.
- `POST /events` rejects RRN in title, location, and reminders.
- `GET /freebusy` defaults to minimal projection.
- `POST /events/{id}/invite` requires transfer notice for non-KR attendees.
- `POST /rooms/{id}/book` requires processor delegation id when external.
- `POST /consent/capture` stores Korean notice text hash.
- `POST /consent/withdraw` stops consent-based scheduling processing.
- `POST /retention/rules` requires 보존 basis id.
- `POST /dsr/export` requires verified identity.
- `POST /dsr/erasure` starts Korean erasure workflow.
- `POST /search/rebuild` requires erasure reason.
- `POST /ics/import` requires source consent proof.
- `POST /caldav/sync` requires processor delegation id.
- `POST /ai/suggest-time` requires Korean PI model-touch consent.
- `GET /privacy-notices/kr/calendar` returns Korean notice.
- `GET /audit/admin-access` exposes DPO-visible events.
- `POST /breach-candidates` starts KR breach workflow.
- `PATCH /tenant-calendar-config` requires KR notice version.
- `POST /pack/deactivate` returns open ledger count.

## Workflow Deltas
- Event create preflight classifies Korean PI and RRN.
- Consent capture stores 동의 text hash and timestamp.
- Consent withdrawal disables AI scheduling and marketing events.
- Retention change records 보존 basis and expiry.
- Cross-border invitation displays Korean transfer notice.
- External room or CalDAV provider verifies 처리위탁 registry.
- Freebusy workflow emits minimal projection for external attendees.
- Korean subject-rights export verifies identity before enumeration.
- Erasure workflow tombstones event and recurrence exceptions.
- Search index rebuild runs after erasure.
- Breach candidate workflow starts KR notification timeline.
- DPO admin-read review is created for support access.
- AI scheduling workflow requires explicit Korean PI consent.
- ICS import validates source consent and transfer basis.
- Reminder workflow scrubs Korean PI from notification previews.
- KR cell migration refuses non-KR destination for resident PI.
- Scheduling link expiry enforcement runs at creation.
- Room booking workflow stores processor delegation proof.
- Pack deactivation waits for consent and retention ledgers.
- Audit bundle publication signs Korean manifest.

## SLO Deltas
- KR breach workflow creation p99 target is <= 5 minutes.
- Korean DPO notification p99 target is <= 24 hours for confirmed leak.
- Consent capture p99 must stay <= 300 ms.
- Consent withdrawal propagation p99 target is <= 30 minutes.
- RRN event detection p99 target is <= 500 ms.
- Freebusy minimal projection p99 must stay <= 200 ms.
- Korean subject-rights export target is <= 10 days internal.
- Erasure tombstone p99 target is <= 72 hours after approval.
- Search index rebuild target is <= 24 hours.
- KR route residency check p99 must stay <= 100 ms.
- Processor delegation lookup p99 must stay <= 200 ms.
- Korean notice retrieval p99 must stay <= 150 ms.
- Retention ledger write p99 must stay <= 500 ms.
- Admin-read audit emission p99 must complete <= 1 second.
- Reminder scrub p99 must stay <= 100 ms.
- Korean calendar dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `CalendarKrPipaConsentCaptured` records consent id and text hash.
- `CalendarKrPipaConsentWithdrawn` records consent id.
- `CalendarKrPipaPurposeChecked` records purpose id.
- `CalendarKrPipaRrnBlocked` records event field.
- `CalendarKrPipaFreebusyProjected` records projection level.
- `CalendarKrPipaSensitivePiBlocked` records policy id.
- `CalendarKrPipaRetentionLedgerWritten` records 보존 basis.
- `CalendarKrPipaCrossBorderNoticeShown` records transfer id.
- `CalendarKrPipaProcessorDelegationChecked` records delegation id.
- `CalendarKrPipaSubjectRightsExportStarted` records case id.
- `CalendarKrPipaSubjectRightsExportCompleted` records manifest hash.
- `CalendarKrPipaErasureTombstoned` records event id.
- `CalendarKrPipaReminderScrubbed` records reminder id.
- `CalendarKrPipaAdminAccessReviewed` records DPO review id.
- `CalendarKrPipaAiSchedulingConsentChecked` records model surface.
- `CalendarKrPipaBreachWorkflowStarted` records candidate id.
- `CalendarKrPipaKrCellRouteBlocked` records target cell.
- `CalendarKrPipaNoticeVersionChanged` records notice version.
- `CalendarKrPipaRoomDelegationChecked` records provider id.
- `CalendarKrPipaPackDeactivationDeferred` records open ledger count.

## Failure Modes specific to this pack
- Consent ledger unavailable; recovery is block consent-based scheduling.
- Korean notice hash mismatches; recovery is disable affected consent capture.
- RRN appears in event title; recovery is reject event.
- Freebusy leaks attendee detail; recovery is invalidate projection cache.
- Cross-border transfer id missing; recovery is downgrade invite or block.
- Processor delegation registry stale; recovery is suspend external room provider.
- Consent withdrawal races queued invitation; recovery is cancel queued invite.
- Retention basis expires during legal hold; recovery is restrict and route review.
- Subject identity verification fails; recovery is deny export.
- Erasure index rebuild fails; recovery is remove shard from serving.
- KR cell outage suggests non-KR failover; recovery is queue operations.
- AI scheduling attempts Korean PI without consent; recovery is block model touch.
- Admin read lacks DPO case; recovery is revoke support session.
- Breach workflow clock fails to start; recovery is create retroactive event.
- Scheduling link lacks expiry; recovery is revoke link.
- Imported ICS lacks consent proof; recovery is quarantine import.
- Korean-language notice unavailable; recovery is fail-closed for new processing.
- Pack deactivation requested with open ledgers; recovery is defer.
- Reminder preview leaks Korean PI; recovery is disable previews.
- External CalDAV sync stores data outside KR cell; recovery is suspend sync.

## Cross-µservice coordination
- `tenancy` provides KR cell placement and active KR-PIPA roster.
- `identity` verifies subject identity and DPO roles.
- `compliance` owns 동의, 보존, 국외이전, and 처리위탁 ledgers.
- `audit-chain` seals Korean calendar PI events.
- `observability` scrubs Korean PI before telemetry storage.
- `mail` applies KR-PIPA overlay for invitations.
- `drive` applies KR-PIPA overlay for event attachments.
- `workflow-engine` runs consent, erasure, breach, and subject-rights workflows.
- `policy-engine` loads all `KRPIPA-calendar-*` fragments.
- `localization` provides Korean privacy notices.
- `notification` removes Korean PI from reminders.
- `dlp-virus-scan` classifies RRN and sensitive event content.
- `search` rebuilds indexes after erasure.
- `admin-console` surfaces KR calendar configuration.
- `incident-response` consumes Korean breach candidates.
- `legal` defines Korean retention and hold exceptions.
- `support` requires DPO-visible access path.
- `connector` validates delegated room and CalDAV providers.
- `data-warehouse` receives only aggregate KR calendar metrics.
- `pack-registry` signs this KR-PIPA calendar overlay.
