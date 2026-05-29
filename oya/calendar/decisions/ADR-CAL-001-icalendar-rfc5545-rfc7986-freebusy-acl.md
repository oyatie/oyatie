---
id: ADR-CAL-001
title: RFC 5545 iCalendar, RFC 7986 Extensions, and Per-Tenant FREEBUSY ACL Semantics
status: Accepted
date: 2026-05-20
microservice: calendar
related_oyatie_adrs:
  - docs/decisions/ADR-0002-tenant-and-identity-kernel.md
  - docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md
  - docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - docs/decisions/ADR-0008-data-use-boundary.md
  - docs/decisions/ADR-0005-eventing-backbone-outbox-pattern.md
decision_owner: axis-calendar
---

# ADR-CAL-001: RFC 5545 iCalendar, RFC 7986 Extensions, and Per-Tenant FREEBUSY ACL Semantics

## Context

- Calendar owns event storage, recurrence, availability resolution, room booking, invitations, ICS import/export, CalDAV, and timezone refresh.
- Existing IPs already name `IP-005-recurrence-engine`, `IP-006-availability-resolver`, `IP-008-invitation-flow`, and `IP-009-ics-import-export-and-caldav`.
- Existing architecture names data classes `INTERNAL_ONLY`, `AUDIT`, and `PII_QUASI`.
- Existing SLOs include agenda render latency, CalDAV availability, freebusy query latency, and ICS import throughput.
- Named precedent: Google Calendar and Microsoft Exchange treat iCalendar as the interchange format while maintaining richer internal event models.
- Named precedent: CalDAV servers use VFREEBUSY as an interoperability contract but apply server-specific ACLs before disclosure.
- Named precedent: AWS IAM resource policies inspire tenant-level availability delegation boundaries.
- Constraint CAL-C1: tenant and principal identity must come from ADR-0002, not from organizer email alone.
- Constraint CAL-C2: invite import, RSVP, freebusy disclosure, ACL changes, and recurrence exceptions must emit audit evidence per ADR-0003.
- Constraint CAL-C3: Cedar must gate event read, busy disclosure, detail disclosure, room booking, delegation, and external invite import per ADR-0007.
- Constraint CAL-C4: event summary, location, attendees, and attachments must respect ADR-0008 data-use classes.
- Constraint CAL-C5: recurrence and RSVP updates must publish through the canonical eventing backbone per ADR-0005.
- Constraint CAL-C6: FREEBUSY must reveal the least data necessary and must not leak private event title, location, attendee list, or health/legal context.
- Constraint CAL-C7: cross-tenant interview booking and vendor scheduling need limited availability disclosure without shared calendar membership.
- Constraint CAL-C8: personal and work dual-context calendars must not leak personal private blocks into employer detail views.
- Constraint CAL-C9: recurring events must remain compatible with RFC 5545 RRULE, EXDATE, RDATE, and RECURRENCE-ID.
- Constraint CAL-C10: newer RFC 7986 fields such as COLOR, IMAGE, CONFERENCE, and NAME can be preserved without becoming authorization inputs.
- Constraint CAL-C11: timezone updates must not rewrite historical event meaning silently.
- Constraint CAL-C12: ICS import must tolerate hostile payloads and resource-exhaustion recurrence storms.
- Constraint CAL-C13: calendar must interoperate with mail via iMIP without making mail the source of calendar truth.
- Constraint CAL-C14: calendar must support regulated packs where attendee identity is sensitive or safety-critical.
- The architecture must distinguish busy blocks, tentative holds, out-of-office, focus time, and confidential holds.
- The architecture must support "show only availability" and "show limited details" as first-class ACL modes.
- The architecture must make FREEBUSY cache invalidation deterministic after event or ACL changes.
- The architecture must preserve imported vendor fields where safe, but not trust them for policy.
- The architecture must keep cold-reader buildability: an implementer can map RFC fields to storage and policy checks.

## Decision

- Use RFC 5545 iCalendar as the canonical external interchange model for events, recurrence, VTIMEZONE, and VFREEBUSY.
- Use RFC 7986 extensions for display metadata, but treat them as optional projection fields.
- Maintain an internal `CalendarEvent` model that preserves RFC fields while adding tenant, principal, cell, data-class, and audit fields.
- Store recurrence as parsed structured data plus original RFC line for round-trip fidelity.
- Evaluate FREEBUSY requests through Cedar before any availability computation.
- Define four disclosure modes: `none`, `busy_only`, `availability_class`, and `limited_details`.
- Make `busy_only` the default for cross-tenant scheduling.
- Make `limited_details` require explicit tenant or user grant.
- Treat `TRANSP:TRANSPARENT` events as not blocking availability unless pack policy overrides.
- Treat `CLASS:PRIVATE` as never detail-disclosable outside the owner context.
- Treat `CLASS:CONFIDENTIAL` as busy-only outside an explicit privileged context.
- Preserve RFC 7986 `CONFERENCE` but gate join URLs as event details, not freebusy data.
- Preserve RFC 7986 `COLOR`, `IMAGE`, and `NAME` only in detail projections.
- Use per-tenant `FreebusyPolicy` objects to define default disclosure for internal, external, delegated, and public callers.
- Use per-calendar ACL entries for exceptions.
- Use per-event ACL overrides only when a user explicitly shares event details.
- Use eventing to publish `calendar.event.changed.v1`, `calendar.acl.changed.v1`, and `calendar.freebusy.cache.invalidated.v1`.
- Cache FREEBUSY results by tenant, calendar id, principal, disclosure mode, time window, and policy hash.
- Set FREEBUSY cache TTL to 60 seconds for normal calendars and 10 seconds for high-risk packs.
- Reject unbounded recurrence expansion at import and query time.
- Cap recurrence expansion to a requested window plus one look-ahead period.
- Pin TZDB version on each event occurrence calculation.
- Recalculate future instances after TZDB upgrade; never rewrite past instance timestamps without explicit migration evidence.
- Use mail iMIP only as an invitation transport; calendar event-store remains the source of truth after import.
- Make ICS export policy-bound; export can omit details and preserve VFREEBUSY only.

## Alternatives Considered

### Store Raw ICS Only

- Pros: maximum round-trip fidelity.
- Pros: simple import pipeline.
- Pros: easy external export.
- Cons: recurrence queries become expensive and error-prone.
- Cons: Cedar cannot easily gate structured fields.
- Cons: FREEBUSY computation would depend on repeated parsing.
- Rejected because calendar needs typed recurrence, tenant scope, and policy-safe availability resolution.

### Use a Proprietary Event Model and Export ICS as Projection

- Pros: clean internal domain model.
- Pros: easier to optimize for Oyatie features.
- Pros: fewer RFC corner cases in storage.
- Cons: round-trip loss with external clients.
- Cons: CalDAV and iMIP interoperability regress.
- Cons: importing RRULE and VTIMEZONE semantics becomes ambiguous.
- Rejected because interoperable calendars require RFC-grounded semantics.

### Global Freebusy Visibility by Default

- Pros: easiest scheduling experience.
- Pros: matches many consumer calendar defaults.
- Pros: lower support cost for finding meeting times.
- Cons: leaks work patterns, health appointments, legal meetings, and safety-sensitive events.
- Cons: incompatible with personal/work dual-context rules.
- Cons: violates least-disclosure expectations for cross-tenant scheduling.
- Rejected because busy disclosure must be policy-bound and tenant-scoped.

### Per-Event Detail ACL Only

- Pros: fine-grained control.
- Pros: simple mental model for private events.
- Cons: too many ACL rows for normal calendars.
- Cons: default behavior becomes hard to audit.
- Cons: freebusy cache invalidation becomes noisy.
- Rejected in favor of tenant and calendar ACL defaults with event override only for explicit exceptions.

## Consequences

- Positive: ICS import/export stays interoperable with Google Calendar, Exchange, Apple Calendar, and CalDAV clients.
- Positive: FREEBUSY disclosure has a concrete Cedar-governed policy surface.
- Positive: personal/work calendar boundaries are enforceable without special-case UI logic.
- Positive: recurrence can be tested against RFC fixtures and stored as typed data.
- Positive: RFC 7986 fields are preserved without becoming accidental policy inputs.
- Positive: cache invalidation follows event, ACL, and policy hashes.
- Positive: timezone behavior is explainable during TZDB upgrades.
- Positive: mail can deliver invitations without becoming calendar authority.
- Negative: parsing and preserving full iCalendar semantics is more complex than a simple event table.
- Negative: external clients may expect detail visibility that policy denies.
- Negative: recurrence expansion can be costly for pathological RRULEs.
- Negative: FREEBUSY cache keys have high dimensionality.
- Negative: preserving original RFC lines adds storage overhead.
- Neutral: public calendars can still expose details through explicit public-read policy.
- Neutral: room resources are treated as principals with specific availability ACLs.
- Neutral: tentative holds can be visible as `availability_class` without exposing title.
- Neutral: imports from mail and CalDAV use the same normalizer.
- Neutral: pack overlays can tighten disclosure without changing RFC compatibility.

## Implementation Notes

- Data shape `CalendarEvent`: `{tenant_id, calendar_id, event_id, uid, sequence, dtstart, dtend, timezone_ref, recurrence_ref, class, transparency, data_class, policy_hash}`.
- Data shape `RecurrenceRule`: `{tenant_id, event_id, rrule, rdate[], exdate[], recurrence_id, tzdb_version, original_lines_hash}`.
- Data shape `FreebusyPolicy`: `{tenant_id, calendar_id, default_internal, default_external, default_delegated, pack_overrides, updated_at}`.
- Data shape `FreebusyGrant`: `{tenant_id, calendar_id, grantee_principal_id, grantee_tenant_id, disclosure_mode, expires_at, reason}`.
- Data shape `FreebusyQuery`: `{requesting_tenant_id, requesting_principal_id, target_calendar_id, window_start, window_end, purpose, disclosure_mode}`.
- REST endpoint `POST /v1/calendar/events/import-ics` imports RFC 5545 text with policy checks.
- REST endpoint `GET /v1/calendar/events/{event_id}/export-ics` exports event or busy-only projection.
- REST endpoint `POST /v1/calendar/freebusy/query` returns VFREEBUSY-compatible results.
- REST endpoint `PUT /v1/calendar/calendars/{calendar_id}/freebusy-policy` updates defaults.
- REST endpoint `POST /v1/calendar/calendars/{calendar_id}/freebusy-grants` creates scoped grants.
- REST endpoint `POST /v1/calendar/tzdb/refresh` starts controlled TZDB update.
- AsyncAPI channel `calendar.event.changed.v1` publishes event changes.
- AsyncAPI channel `calendar.acl.changed.v1` publishes ACL and freebusy policy changes.
- AsyncAPI channel `calendar.freebusy.cache.invalidated.v1` publishes cache invalidation.
- gRPC method `CalendarAvailability.QueryFreebusy` supports low-latency internal scheduling.
- gRPC method `CalendarInterop.NormalizeIcs` supports mail iMIP handoff.
- Cedar permit `calendar::freebusy::query` requires valid purpose and disclosure mode.
- Cedar forbid `calendar::event::detail_read` when `resource.class in ["PRIVATE", "CONFIDENTIAL"]` and no explicit grant exists.
- Cedar permit `calendar::freebusy::limited_details` requires `FreebusyGrant.disclosure_mode == "limited_details"`.
- Cedar forbid `calendar::ics::import` when recurrence expansion estimate exceeds pack threshold.
- Audit event `EVT-CAL-ICS-IMPORTED` includes source, uid, sequence, and parser profile.
- Audit event `EVT-CAL-FREEBUSY-DISCLOSED` includes disclosure mode, window, and policy hash.
- Audit event `EVT-CAL-FREEBUSY-GRANT-CHANGED` includes grantee and expiry.
- Audit event `EVT-CAL-TZDB-REFRESHED` includes old and new TZDB versions.
- Metric `calendar_freebusy_query_latency_ms` tracks p50, p95, and p99 by disclosure mode.
- Metric `calendar_recurrence_expansion_count` tracks expansion count per query.
- Metric `calendar_ics_import_reject_total` tracks malformed and hostile imports.
- Metric `calendar_freebusy_cache_hit_ratio` tracks cache effectiveness by tenant and mode.
- Capacity math: if p95 freebusy query target is 50 ms and peak is 2,000 queries/s, Little's Law gives 100 in-flight queries; provision 1,000 slots with cache hit ratio >=80 percent.
- Capacity math: cap recurrence expansion at 10,000 instances per query to bound CPU under pathological RRULEs.
- Rollback path: freebusy policy update stores previous policy hash and can revert pointer without mutating events.
- Rollback path: TZDB refresh can pin affected future calculations to prior TZDB version during incident.
- Multi-region path: freebusy queries execute in target calendar home cell; remote callers receive policy-filtered results.
- Sovereign path: regulated packs prohibit detail disclosure outside home jurisdiction even if busy-only can cross cell.
- Versioning: ICS import profile is versioned as `calendar-ics-profile-v1`.
- Deprecation: unsupported RFC extensions are preserved as opaque properties for 180 days before rejection policy changes.

## Verification

- Unit test `private_event_never_discloses_details_without_grant` covers `CLASS:PRIVATE`.
- Unit test `confidential_event_returns_busy_only_for_external` covers confidential holds.
- Unit test `transparent_event_does_not_block_freebusy` covers `TRANSP:TRANSPARENT`.
- Unit test `rfc7986_conference_url_is_detail_not_busy` protects join URLs.
- Unit test `freebusy_query_requires_cedar_purpose` proves policy gating.
- Property test `rrule_round_trip_preserves_original_lines_hash` covers import/export fidelity.
- Property test `recurrence_expansion_is_window_bounded` generates hostile RRULEs.
- Fuzz test `ics_parser_rejects_control_character_header_injection` covers malicious imports.
- Integration test `mail_imip_import_creates_calendar_event_store_record` verifies mail handoff.
- Integration test `cross_tenant_interview_booking_busy_only` verifies default disclosure.
- Integration test `freebusy_policy_change_invalidates_cache` verifies invalidation channel.
- Integration test `tzdb_refresh_preserves_past_occurrences` verifies historical stability.
- Load test `freebusy_2000_qps_80_percent_cache_hit` keeps p99 below 100 ms.
- Load test `ics_import_100mb_rejected_before_expansion` covers resource exhaustion.
- Chaos test `eventing_backpressure_blocks_acl_change` proves evidence-first behavior.
- Chaos test `tzdb_refresh_partial_failure_reverts_pointer` proves rollback.
- Metric SLO: `calendar_freebusy_query_latency_ms` p95 below 50 ms for cached paths.
- Metric SLO: `calendar_recurrence_expansion_count` p99 below 10,000 instances per query.
- Metric SLO: `calendar_freebusy_cache_hit_ratio` above 80 percent for workday scheduling.
- Audit check: every FREEBUSY response has one `EVT-CAL-FREEBUSY-DISCLOSED`.
- Audit check: every grant change has before and after disclosure mode.
- Static check: FREEBUSY code path cannot read event description or conference URL unless mode is `limited_details`.
- Static check: ICS parser runs before domain storage writes and after size checks.
- Contract check: OpenAPI documents the four disclosure modes and their defaults.

