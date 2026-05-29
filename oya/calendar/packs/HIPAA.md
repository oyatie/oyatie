---
doc_class: CompliancePackOverlay
pack_id: HIPAA-2024
microservice: calendar
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# calendar HIPAA Compliance Pack Overlay

## Pack Identity
- Full pack name: HIPAA Administrative Simplification calendar ePHI scheduling overlay.
- Citing jurisdiction: United States federal health information regime.
- Version: HIPAA-2024-v1.
- Canonical source URL: https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C
- Cited law: 45 CFR Parts 160, 162, and 164.
- Covered calendar surface: events, attendees, invitations, freebusy, rooms, recurrence, reminders, CalDAV/ICS import, scheduling links, and audit exports.
- Pack activation means calendar may hold ePHI only when tenant has BAA proof and HIPAA-certified cell placement.
- Appointment title, description, location, attendee list, reminder text, and availability metadata can all leak PHI.
- Data classes include `CALENDAR_EVENT_PHI`, `CALENDAR_FREEBUSY_PHI`, `CALENDAR_ATTENDEE_PHI`, and `CALENDAR_AUDIT_PHI`.
- Minimum necessary applies to freebusy disclosure and invitation detail.
- ADR-0064 keeps scheduling base neutral while this pack adds PHI policy.
- ADR-0251 supplies cell eligibility, breach workflow, and pack retention.
- ADR-0263 requires PHI-safe telemetry and audit ids on state changes.
- This overlay excludes PCI-DSS because calendar does not process payment authorization.
- Payment meeting references are treated as confidential text unless payments service activates PCI scope.

## Data Model Deltas
- Add `event.phi_signal` as enum `none|possible|confirmed`.
- Add `event.phi_basis` as enum `treatment|payment|operations|patient_request|none`.
- Add `event.patient_context_id` as nullable opaque reference.
- Add `event.minimum_necessary_visibility` as enum `busy_only|limited|full`.
- Add `event.title_phi_blocked` boolean.
- Add `event.location_phi_blocked` boolean.
- Add `event.description_phi_encrypted` boolean.
- Add `event.reminder_phi_scrubbed` boolean.
- Add `event.break_glass_reason_id` nullable.
- Add `event.disclosure_accounting_required` boolean.
- Add `event.retention_floor_iso8601` default `P6Y`.
- Add `attendee.baa_status_snapshot`.
- Add `attendee.phi_role_scope`.
- Add `freebusy.phi_safe_projection` boolean.
- Add `room.hipaa_certified_room` boolean for physical/virtual rooms.
- Add `recurrence.phi_exception_count`.
- Add `invite.phi_detail_level` as enum `none|limited|full`.
- Add `ics_import.phi_scan_verdict`.
- Add `caldav_sync.phi_route_cell`.
- Add `schedule_link.external_baa_required` boolean.
- Add `workflow_handoff.hipaa_case_context_id`.
- Add `audit_shadow.calendar_phi_event_id`.
- Add `export_job.phi_calendar_manifest_hash`.
- Add `tenant_calendar_config.hipaa_cell_certification`.

## Cedar Policy Deltas
- Policy `HIPAA-calendar-create-01`: require HIPAA-certified cell for PHI event creation.
- Policy `HIPAA-calendar-create-02`: forbid PHI in event title.
- Policy `HIPAA-calendar-create-03`: require encrypted description for confirmed PHI.
- Policy `HIPAA-calendar-read-01`: permit full read only for roles in minimum necessary scope.
- Policy `HIPAA-calendar-freebusy-01`: expose busy-only projection to external attendees.
- Policy `HIPAA-calendar-freebusy-02`: forbid diagnostic detail in freebusy payload.
- Policy `HIPAA-calendar-invite-01`: require BAA for external recipient receiving PHI detail.
- Policy `HIPAA-calendar-invite-02`: downgrade invite detail when recipient BAA is absent.
- Policy `HIPAA-calendar-room-01`: require HIPAA-certified virtual room for PHI appointment.
- Policy `HIPAA-calendar-reminder-01`: scrub PHI from reminder notifications.
- Policy `HIPAA-calendar-caldav-01`: require PHI-safe CalDAV adapter for sync.
- Policy `HIPAA-calendar-ics-01`: quarantine import when PHI scan fails.
- Policy `HIPAA-calendar-search-01`: restrict PHI event search to case context.
- Policy `HIPAA-calendar-breakglass-01`: permit emergency read only with reason id and TTL <= 1h.
- Policy `HIPAA-calendar-retention-01`: forbid purge before six-year floor.
- Policy `HIPAA-calendar-export-01`: require privacy-office approval for PHI calendar export.
- Policy `HIPAA-calendar-route-01`: forbid replication outside HIPAA-certified cell.
- Policy `HIPAA-calendar-reschedule-01`: retain disclosure accounting across event moves.
- Policy `HIPAA-calendar-delegate-01`: require covered workforce role for delegate scheduling.
- Policy `HIPAA-calendar-link-01`: forbid anonymous scheduling link for PHI appointments.
- Policy `HIPAA-calendar-webhook-01`: require BAA proof for external workflow handoff.
- Policy `HIPAA-calendar-ai-01`: forbid AI scheduling suggestions on PHI unless BYOK provider active.
- Policy `HIPAA-calendar-audit-01`: require audit seal for every PHI event mutation.
- Policy `HIPAA-calendar-pack-01`: defer deactivation while PHI events remain retained.

## API Contract Deltas
- `POST /events` requires `X-Oyatie-Purpose` for HIPAA tenants.
- `POST /events` rejects PHI in title.
- `POST /events` requires encrypted description flag for confirmed PHI.
- `GET /events/{id}` requires elevated ACR for full PHI detail.
- `GET /freebusy` returns busy-only projection for external callers.
- `POST /events/{id}/invite` checks recipient BAA status.
- `POST /events/{id}/rooms` requires HIPAA-certified room for PHI.
- `POST /reminders` refuses PHI reminder text.
- `POST /ics/import` returns quarantine state on scan failure.
- `POST /caldav/sync` requires PHI-safe route cell.
- `POST /schedule-links` refuses anonymous PHI scheduling links.
- `POST /delegates` requires covered workforce role mapping.
- `POST /exports` requires privacy-office approval id.
- `GET /exports/{id}` returns PHI manifest hash.
- `DELETE /events/{id}` returns retention conflict before six-year floor.
- `POST /events/{id}/break-glass` requires reason id.
- `POST /workflow-handoffs` requires BAA proof for external destination.
- `POST /ai/suggest-time` requires BYOK provider admission.
- `PATCH /tenant-calendar-config` requires BAA admission proof.
- `POST /pack/deactivate` returns retained PHI event count.

## Workflow Deltas
- Event create preflight classifies title, description, location, and attendees.
- Confirmed PHI event stores detail encrypted.
- External invite downgrades detail when BAA is absent.
- Freebusy workflow emits busy-only projection by default.
- Room booking checks HIPAA-certified room or meeting bridge.
- Reminder workflow scrubs PHI from notifications.
- CalDAV sync uses PHI-safe adapter path.
- ICS import quarantines suspected PHI until review.
- Break-glass read opens one-hour emergency review.
- Reschedule workflow preserves disclosure accounting.
- Delegate workflow checks covered workforce role.
- Scheduling link workflow disables anonymous link mode.
- Search workflow indexes only PHI-safe tokens.
- Export workflow builds PHI manifest before release.
- Retention workflow prevents purge before six-year floor.
- Webhook handoff verifies BAA destination.
- AI suggestion workflow uses BYOK provider or blocks.
- Replication workflow validates certified target cell.
- Incident workflow creates suspected PHI leak candidate.
- Pack deactivation waits for retained event inventory.

## SLO Deltas
- PHI event create preflight p99 must stay <= 500 ms.
- PHI freebusy projection p99 must stay <= 300 ms.
- BAA attendee lookup p99 must stay <= 200 ms.
- PHI event mutation audit seal p99 must complete <= 1 second.
- Reminder scrub p99 must stay <= 100 ms.
- HIPAA room certification lookup p99 must stay <= 200 ms.
- ICS PHI scan start p99 must complete <= 2 minutes.
- CalDAV route validation p99 must stay <= 100 ms.
- Break-glass workflow start p99 must complete <= 2 minutes.
- Export manifest generation p99 target is <= 15 minutes.
- Retention conflict response p99 must stay <= 300 ms.
- Suspected breach candidate creation p99 target is <= 5 minutes.
- AI provider BYOK check p99 must stay <= 200 ms.
- Disclosure accounting write p99 must complete <= 1 second.
- PHI event inventory report lag target is <= 1 hour.
- HIPAA calendar dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `CalendarPhiEventPreflighted` records event id and verdict.
- `CalendarPhiTitleBlocked` records policy id.
- `CalendarPhiEventCreated` records detail level.
- `CalendarPhiEventUpdated` records version.
- `CalendarPhiEventCanceled` records retention state.
- `CalendarPhiFreebusyProjected` records projection type.
- `CalendarPhiInviteDowngraded` records recipient class.
- `CalendarPhiExternalInviteBlocked` records BAA status.
- `CalendarPhiRoomCertificationChecked` records room id.
- `CalendarPhiReminderScrubbed` records reminder id.
- `CalendarPhiIcsImportQuarantined` records import id.
- `CalendarPhiCaldavRouteBlocked` records target cell.
- `CalendarPhiBreakGlassStarted` records reason id.
- `CalendarPhiDisclosureRecorded` records attendee hash.
- `CalendarPhiExportManifestCreated` records manifest hash.
- `CalendarPhiPurgeRefused` records retention floor.
- `CalendarPhiWebhookRefused` records destination id.
- `CalendarPhiAiSuggestionBlocked` records provider mode.
- `CalendarPhiAuditBackpressureClosed` records queue depth.
- `CalendarPhiPackDeactivationDeferred` records retained count.

## Failure Modes specific to this pack
- PHI appears in title; recovery is reject event and offer safe title.
- BAA lookup times out; recovery is downgrade invite to busy-only.
- Freebusy exposes diagnosis detail; recovery is invalidate projection cache.
- Reminder contains PHI; recovery is scrub and regenerate notification.
- Room certification expires; recovery is move event to certified room or block.
- CalDAV adapter cannot enforce PHI policy; recovery is suspend sync.
- ICS import scan fails; recovery is quarantine import.
- External scheduling link exists before activation; recovery is revoke link.
- Break-glass TTL expires; recovery is revoke read token.
- Legal hold conflicts with deletion; recovery is hold lock wins.
- AI suggestion tries platform default provider; recovery is block suggestion.
- Replication planner chooses uncertified cell; recovery is reject plan.
- Export manifest mismatch appears; recovery is revoke export and rebuild.
- Search index contains PHI text; recovery is drop shard and rebuild safe tokens.
- Attendee BAA revoked after invite; recovery is cancel or downgrade future updates.
- Delegate loses covered workforce role; recovery is revoke delegate grant.
- Audit-chain backpressure appears; recovery is fail-closed for PHI mutations.
- Pack deactivation requested with retained PHI events; recovery is defer.
- Patient context mapping stale; recovery is block full-detail disclosure.
- Webhook destination certificate changes; recovery is suspend handoff.

## Cross-µservice coordination
- `tenancy` must place HIPAA tenants in HIPAA-certified cells.
- `identity` provides elevated ACR and workforce role claims.
- `compliance` provides BAA proof and breach workflow.
- `audit-chain` seals every PHI calendar event.
- `observability` scrubs PHI from event telemetry.
- `mail` applies HIPAA overlay when sending invitations.
- `drive` applies HIPAA overlay when event attachments are stored.
- `workflow-engine` runs break-glass, export, and breach workflows.
- `policy-engine` loads all `HIPAA-calendar-*` fragments.
- `meet` or room service provides certified virtual room proof.
- `notification` scrubs reminder previews.
- `search` indexes only PHI-safe event tokens.
- `dlp-virus-scan` scans ICS imports and descriptions.
- `records` owns patient context references.
- `admin-console` displays BAA and event inventory state.
- `incident-response` consumes calendar leak candidates.
- `support` uses break-glass workflow for PHI tickets.
- `legal` defines retention and hold templates.
- `data-warehouse` receives aggregate PHI-free scheduling metrics.
- `pack-registry` signs this HIPAA calendar overlay.
