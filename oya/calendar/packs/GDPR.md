---
doc_class: CompliancePackOverlay
pack_id: EU-GDPR-2018-baseline
microservice: calendar
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# calendar GDPR Compliance Pack Overlay

## Pack Identity
- Full pack name: EU GDPR calendar scheduling privacy overlay.
- Citing jurisdiction: European Union and EEA personal-data regime.
- Version: EU-GDPR-2018-baseline-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2016/679/oj
- Cited law: Regulation (EU) 2016/679.
- Covered calendar surface: events, attendees, availability, freebusy, reminders, locations, rooms, imports, exports, and erasure workflows.
- Pack activation means calendar treats attendee participation and scheduling metadata as personal data.
- Freebusy disclosure is personal data processing even when title is hidden.
- Data classes include `CALENDAR_PERSONAL_DATA_EU`, `CALENDAR_SPECIAL_CATEGORY_EU`, and `CALENDAR_PORTABILITY_EXPORT_EU`.
- DSAR export includes event data, attendee status, recurrence exceptions, and reminder metadata.
- ADR-0064 keeps EU rights behavior in the pack overlay.
- ADR-0251 supplies pack admission, cell eligibility, and breach workflow.
- ADR-0263 requires personal-data scrubbing before telemetry storage.
- This overlay excludes PCI-DSS because calendar does not own payment authorization.
- Calendar events referencing invoices remain ordinary confidential text unless payments service activates PCI scope.

## Data Model Deltas
- Add `event.eu_personal_data_signal` as enum `none|personal|special_category`.
- Add `event.lawful_basis`.
- Add `event.lawful_basis_evidence_id`.
- Add `event.data_subject_ids_hash`.
- Add `event.erasure_state` as enum `active|restricted|erasure_pending|tombstoned`.
- Add `event.restriction_reason`.
- Add `event.portability_included`.
- Add `event.transfer_mechanism`.
- Add `event.eu_residency_cell`.
- Add `event.visibility_privacy_level` as enum `private|busy_only|limited|full`.
- Add `attendee.dsar_subject_hash`.
- Add `attendee.rsvp_personal_data_state`.
- Add `freebusy.eu_projection_level`.
- Add `reminder.personal_data_scrubbed`.
- Add `location.special_category_risk`.
- Add `room.processor_dpa_ref`.
- Add `recurrence.erasure_exception_map`.
- Add `ics_import.origin_transfer_mechanism`.
- Add `caldav_sync.eu_processor_ref`.
- Add `search_index.eu_erasure_rebuild_required`.
- Add `export_job.gdpr_calendar_manifest_hash`.
- Add `audit_shadow.gdpr_event_id`.
- Add `tenant_calendar_config.eu_dpa_version`.
- Add `tenant_calendar_config.eu_retention_schedule_version`.

## Cedar Policy Deltas
- Policy `GDPR-calendar-read-01`: permit read only for declared processing purpose.
- Policy `GDPR-calendar-read-02`: forbid read when event is restricted and caller is not DPO.
- Policy `GDPR-calendar-create-01`: require lawful basis for event creation.
- Policy `GDPR-calendar-freebusy-01`: expose minimal freebusy unless full detail permitted.
- Policy `GDPR-calendar-freebusy-02`: forbid special-category detail in availability.
- Policy `GDPR-calendar-invite-01`: require transfer mechanism for non-EEA attendee route.
- Policy `GDPR-calendar-room-01`: require processor DPA for external room provider.
- Policy `GDPR-calendar-export-01`: permit DSAR export for verified subject or DPO.
- Policy `GDPR-calendar-erasure-01`: permit tombstone when no legal hold conflict exists.
- Policy `GDPR-calendar-erasure-02`: forbid purge when statutory retention applies.
- Policy `GDPR-calendar-restrict-01`: restrict serving during accuracy dispute.
- Policy `GDPR-calendar-portability-01`: require ICS plus metadata JSON for Article 20.
- Policy `GDPR-calendar-reminder-01`: scrub personal data from notification previews.
- Policy `GDPR-calendar-search-01`: require index rebuild after erasure.
- Policy `GDPR-calendar-caldav-01`: require processor DPA for CalDAV sync provider.
- Policy `GDPR-calendar-ics-01`: require origin transfer mechanism for imports.
- Policy `GDPR-calendar-breach-01`: start Article 33 clock on confirmed calendar leak.
- Policy `GDPR-calendar-admin-01`: require DPO-visible audit for admin event access.
- Policy `GDPR-calendar-retention-01`: forbid blanket indefinite retention.
- Policy `GDPR-calendar-ai-01`: require model-touch consent for scheduling suggestions.
- Policy `GDPR-calendar-route-01`: require EU residency unless transfer proof permits.
- Policy `GDPR-calendar-webhook-01`: forbid workflow webhook without DPA reference.
- Policy `GDPR-calendar-objection-01`: disable direct-marketing event processing after objection.
- Policy `GDPR-calendar-pack-01`: defer pack deactivation with open DSAR cases.

## API Contract Deltas
- `POST /events` requires `lawful_basis` for EU tenants.
- `POST /events` accepts `lawful_basis_evidence_id`.
- `GET /events/{id}` masks restricted events for non-DPO roles.
- `GET /freebusy` accepts `privacy_projection=minimal|busy_only|limited`.
- `POST /events/{id}/invite` requires transfer mechanism for non-EEA attendees.
- `POST /rooms/{id}/book` requires processor DPA reference when external.
- `POST /dsar/export` starts event and availability export.
- `GET /dsar/export/{id}` returns ICS and metadata manifest hash.
- `POST /dsar/erasure` starts event tombstone workflow.
- `POST /dsar/restrict` blocks event serving while accuracy is disputed.
- `POST /search/rebuild` requires erasure reason.
- `POST /ics/import` records origin transfer mechanism.
- `POST /caldav/sync` requires processor DPA reference.
- `POST /ai/suggest-time` requires model-touch lawful basis.
- `POST /retention/rules` requires purpose-bounded schedule.
- `POST /breach-candidates` starts GDPR breach clock.
- `GET /audit/admin-access` returns DPO-visible events.
- `PATCH /tenant-calendar-config` records EU DPA version.
- `POST /workflow-handoffs` requires processor DPA reference.
- `POST /pack/deactivate` refuses open DSAR cases.

## Workflow Deltas
- Event create workflow records lawful basis.
- Special-category classifier runs on title, description, location, and attendees.
- Freebusy workflow defaults to minimal projection.
- DSAR export enumerates events by subject hash.
- Portability export includes ICS and metadata JSON.
- Erasure workflow tombstones event and recurrence exceptions.
- Restriction workflow blocks normal event serving.
- Search index rebuild runs after erasure.
- Reminder workflow scrubs personal data from notification previews.
- Non-EEA invite workflow validates transfer mechanism.
- External room provider workflow validates DPA.
- CalDAV sync workflow checks processor DPA.
- ICS import records origin transfer mechanism.
- AI scheduling workflow records model-touch lawful basis.
- Breach candidate workflow starts Article 33 clock.
- Admin event access creates DPO-visible review event.
- Retention schedule workflow records processing purpose.
- Direct-marketing objection disables campaign events.
- Pack activation scans public scheduling links.
- Pack deactivation waits for DSAR completion.

## SLO Deltas
- GDPR breach regulator-readiness p99 target is <= 60 hours.
- Breach clock creation p99 target is <= 5 minutes.
- DSAR event enumeration first response target is <= 7 days.
- Full calendar portability export target is <= 30 days.
- Erasure tombstone p99 target is <= 72 hours after approval.
- Search index rebuild target is <= 24 hours.
- Restriction activation p99 must complete <= 15 minutes.
- Lawful-basis event preflight p99 must stay <= 300 ms.
- Freebusy minimal projection p99 must stay <= 200 ms.
- EU route validation p99 must stay <= 100 ms.
- DPA lookup p99 must stay <= 200 ms.
- Admin access audit p99 must complete <= 1 second.
- Reminder scrub p99 must stay <= 100 ms.
- Portability manifest generation p99 target is <= 4 hours.
- Special-category classifier review cadence is daily.
- DPO calendar dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `CalendarGdprLawfulBasisRecorded` records event id and basis.
- `CalendarGdprSpecialCategoryDetected` records classifier version.
- `CalendarGdprFreebusyProjected` records projection level.
- `CalendarGdprInviteTransferChecked` records mechanism.
- `CalendarGdprDsarExportStarted` records case id.
- `CalendarGdprDsarExportCompleted` records manifest hash.
- `CalendarGdprEventTombstoned` records event id.
- `CalendarGdprRecurrenceTombstoned` records recurrence id.
- `CalendarGdprRestrictionApplied` records reason.
- `CalendarGdprRestrictionReleased` records reviewer.
- `CalendarGdprIndexRebuilt` records shard id.
- `CalendarGdprReminderScrubbed` records reminder id.
- `CalendarGdprCaldavDpaChecked` records provider id.
- `CalendarGdprAiSuggestionConsentChecked` records model id.
- `CalendarGdprBreachClockStarted` records candidate id.
- `CalendarGdprAdminAccessReviewed` records review id.
- `CalendarGdprIcsImportTransferRecorded` records origin.
- `CalendarGdprRetentionScheduleChanged` records schedule.
- `CalendarGdprPublicLinkRevoked` records link id.
- `CalendarGdprPackDeactivationDeferred` records open cases.

## Failure Modes specific to this pack
- Lawful basis missing on event creation; recovery is reject event.
- Freebusy leaks event title; recovery is invalidate cache and rebuild minimal projection.
- DSAR subject hash misses attendee; recovery is rerun from audit-chain.
- ICS export omits recurrence exception; recovery is revoke export and rebuild.
- Erasure tombstone fails on recurrence; recovery is restrict series until fixed.
- Search shard serves erased event; recovery is remove shard and rebuild.
- External attendee lacks transfer mechanism; recovery is send busy-only placeholder.
- External room provider DPA expires; recovery is disable provider for EU tenants.
- Special-category classifier is unavailable; recovery is fail-closed for sharing.
- CalDAV provider lacks DPA; recovery is suspend sync.
- AI suggestion ran without lawful basis; recovery is delete suggestion and open incident.
- Reminder preview leaks personal data; recovery is disable previews and audit.
- Admin access lacks DPO case; recovery is revoke access.
- Retention schedule is indefinite; recovery is reject schedule.
- Public scheduling link existed before activation; recovery is revoke link.
- Legal hold blocks erasure; recovery is restrict processing and route review.
- Pack deactivation requested with open DSAR; recovery is defer.
- Import lacks origin transfer proof; recovery is quarantine import.
- EU cell outage suggests non-EU failover; recovery is queue operations.
- Breach clock fails to start; recovery is create retroactive event and page compliance.

## Cross-µservice coordination
- `tenancy` provides EU cell placement and active pack roster.
- `identity` verifies data subject and DPO role.
- `compliance` owns DSAR, processing register, DPA, and breach cases.
- `audit-chain` seals event lifecycle and DSAR events.
- `observability` scrubs personal data from scheduling telemetry.
- `mail` applies GDPR overlay when sending invitations.
- `drive` applies GDPR overlay for event attachments and exports.
- `workflow-engine` runs DSAR, erasure, restriction, and breach workflows.
- `policy-engine` loads all `GDPR-calendar-*` fragments.
- `search` rebuilds indexes after erasure.
- `notification` avoids personal data in reminders.
- `dlp-virus-scan` classifies special-category event content.
- `admin-console` renders EU DPA and retention configuration.
- `incident-response` consumes calendar leak candidates.
- `legal` resolves legal hold conflicts.
- `data-warehouse` receives aggregate scheduling metrics only.
- `support` uses DPO-visible access path.
- `connector` validates external room and CalDAV processors.
- `localization` provides EU language notices.
- `pack-registry` signs this GDPR calendar overlay.
