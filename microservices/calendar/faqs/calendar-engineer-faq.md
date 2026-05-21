---
doc_class: FAQ
microservice: calendar
persona: calendar-engineer + scheduling-engineer + caldav-jmap-engineer
related_adrs: [ADR-CAL-001, ADR-CAL-0001, ADR-CAL-0002, ADR-CAL-0003, ADR-CAL-0004]
date: 2026-05-20
doc_status: published
---

# Calendar Engineer FAQ — calendar

## Why RFC 5545 iCalendar + RFC 7986 instead of a proprietary model?

Per ADR-CAL-001 § Alternatives Considered. RFC 5545 is:

1. **Universal interchange standard**: every major calendar client (Apple Calendar, Outlook, Google Calendar, Mozilla Thunderbird, KOrganizer) consumes RFC 5545 ICS files.
2. **Stable since 2009** (with iCal2.0 effectively unchanged since 1998).
3. **Recurrence semantics are well-defined**: RRULE, EXDATE, RDATE, RECURRENCE-ID have precise meanings.
4. **Round-trip-safe**: an event exported as ICS can be re-imported with no loss.

Proprietary models would force every external client to use oyatie-specific tooling. RFC 7986 (extensions for COLOR, IMAGE, CONFERENCE, NAME) adds modern UX without breaking backward compatibility.

The internal `CalendarEvent` model preserves RFC fields + adds tenant, principal, cell, data-class, and audit fields (per ADR-CAL-001 § Decision).

## What's the four-disclosure-mode model?

Per ADR-CAL-001 § Decision. FREEBUSY responses come in four modes:

1. **none**: no data returned. Requester learns nothing about the calendar.
2. **busy_only**: time blocks marked BUSY (no titles, no participants, no locations). Standard cross-tenant default.
3. **availability_class**: BUSY + work-personal-class indicator (e.g., "BUSY: WORK", "BUSY: PERSONAL"). Used in dual-context.
4. **limited_details**: BUSY + summary + location (no participants, no description). Requires explicit grant.

Why four modes:

- One-size-fits-all leaks too much (Google's default shows full titles to anyone with access).
- Pack-bound tenants need stricter modes (HIPAA appointments → `busy_only` even within tenant).
- Cross-tenant interviews need `busy_only` (Calendly's model).
- Internal team scheduling needs `limited_details` (so Bob can see "Strategy review with CTO" before booking conflict).

## How does cross-tenant FREEBUSY work?

Per ADR-CAL-001 § Decision + IP-journey-j132 + j144 + j56. Three pathways:

1. **Pre-grant via `FreebusyGrant`**: tenant A creates explicit grant for tenant B's user X. Grant has scope (calendar, disclosure_mode, expiration, reason). Stored in `FreebusyGrant` table.
2. **Scheduling-link** (Calendly-style): user generates a short-lived FREEBUSY URL. External requester (anonymous or signed) gets `busy_only` view for the link's scope window.
3. **iMIP invitation** (per ADR-CAL-001 § Decision): incoming invitation via mail; calendar imports + creates RSVP flow. Mail is not the source of truth; calendar event-store is.

Cedar gate: `calendar::freebusy::query` must pass for every cross-tenant request. Audit-chain `EVT-CAL-FREEBUSY-DISCLOSED` emits the disclosure mode + window + policy hash.

## Why is the FREEBUSY cache TTL 60 s default and 10 s for high-risk packs?

Per ADR-CAL-001 § Decision. Trade-off between:

- **Cache hit ratio**: high TTL → higher hit ratio → lower query cost.
- **Staleness window**: low TTL → fresher data → less risk of disclosing already-cancelled events.

60 s is the sweet spot for default tenants (most calendar changes are not urgent). 10 s for high-risk packs (HIPAA, FedRAMP-High, KR-PIPA) because:

- Confidential appointments may be quickly rescheduled.
- 10-second-stale data still risks leaking that an appointment EXISTED (even if it moved).

Cache invalidation is also event-driven: `calendar.event.changed.v1` or `calendar.acl.changed.v1` immediately invalidates affected cache entries.

## How does work-personal dual-context FREEBUSY work?

Per ADR-CAL-001 Constraint CAL-C8 + IP-journey-j27 + j35. A principal has TWO calendars on different `audience_type`:

- `audience_type=work`: employer-tenant calendar (e.g., u-alice@acme-corp.com).
- `audience_type=personal`: personal-tenant calendar (e.g., u-alice@personal-tenant).

Aggregated FREEBUSY:

- Employer view: shows BUSY blocks from both calendars, classified by `availability_class` (so employer sees "BUSY: PERSONAL" without knowing what).
- Personal view: full visibility into both calendars.
- Personal-only events NEVER leak detail to employer.

This is enforced by Cedar gates that check `principal.audience_type` vs `resource.calendar.audience_type`.

## What's the recurrence engine throughput limit?

Per ADR-CAL-001 § Decision verification target. Hard caps:

- Recurrence expansion ≤ 10 000 instances per query.
- Recurrence storage: parsed RRULE + original RFC line preserved.
- TZDB pinning per event occurrence calculation.

This protects against:

- **Recurrence bombs**: `FREQ=SECONDLY;COUNT=1000000000000` would consume 1 TB+ RAM if naively expanded.
- **DoS**: malicious ICS imports trying to exhaust CPU.
- **Performance regression**: queries with overly-wide windows.

If a query genuinely needs >10k instances (e.g., 20-year historical view of a daily standup), the query must paginate or narrow the window.

## How is TZDB updated without rewriting history?

Per ADR-CAL-001 Constraint CAL-C11 + ADR-CAL-0004. Two principles:

1. **TZDB pinning per occurrence**: each event occurrence records `tzdb_version` at creation. Past occurrences NEVER change timestamps when TZDB updates.
2. **Recalculate future only**: TZDB refresh recalculates future occurrences. If a country changes its DST rule, future Daylight Saving transitions reflect the new rule.

Example: in 2024, Lebanon delayed DST start by a month. Events created before the change had timestamps based on old DST rule. After TZDB update, future occurrences reflect the new rule; past occurrences stay as they were.

This is critical for audit + legal evidence ("the meeting WAS at 14:00 UTC on that date, no matter what TZDB says now").

## How does iMIP work with mail?

Per ADR-CAL-001 § Decision. iMIP (Mail-based iCalendar) is RFC 6047. Flow:

1. Sender's calendar creates event with recipients.
2. Calendar µservice emits iMIP message (ICS in mail body).
3. Recipient's mail µservice receives → identifies iMIP → forwards to calendar µservice via `CalendarInterop.NormalizeIcs` gRPC.
4. Calendar µservice creates event in recipient's calendar with RSVP=NEEDS-ACTION.
5. Recipient RSVPs → calendar updates → iMIP REPLY sent back via mail.

Critical: mail is just transport. Calendar event-store is the source of truth. The recipient's local event reflects RSVPs + recurrence-exception updates regardless of mail server state.

## What's the CalDAV vs JMAP priority (ADR-CAL-0003)?

Per ADR-CAL-0003 § Decision. Two API surfaces:

- **CalDAV RFC 4791**: mature standard; supported by Apple Calendar, Mozilla Thunderbird, KOrganizer, many corporate clients. Required at all tiers.
- **JMAP-for-Calendars (draft-ietf-jmap-calendars)**: modern JSON-RPC; better mobile push; lower bandwidth; better tenant isolation. paid priority.

Migration story: CalDAV for legacy clients (Apple/Thunderbird/Outlook-via-add-on). JMAP for first-party + modern web clients. Both APIs serve the same `CalendarEvent` model — clients can mix-and-match.

## How does the per-tenant scheduling-link enclave work (paid)?

Per paid tier capability. Each tenant gets its own scheduling-link domain (e.g., `meet.acme-corp.com`). Configuration:

```yaml
tenant: acme-corp
scheduling_link_domain: meet.acme-corp.com
default_disclosure_mode: busy_only
default_expiry_days: 30
require_authenticated_requester: false  # or true for B2B-only
watermark_policy: email-tagged
```

Each user can generate a scheduling link:

```sh
oya calendar scheduling-link create \
    --tenant acme-corp \
    --user u-alice@acme-corp.com \
    --duration-minutes 30 \
    --available-windows weekdays:09:00-17:00 \
    --buffer-minutes 10 \
    --advance-notice-hours 4
# Returns: https://meet.acme-corp.com/u-alice/30min
```

External requesters visit the link → see FREEBUSY in `busy_only` mode → book a slot → event created on Alice's calendar with iMIP invite sent.

## What's the room booking model?

Per IP-007-room-booking + IP-008-invitation-flow. Room resources are principals with `principal_type=resource`. Each room has:

- Display name, capacity, location, features (whiteboard, video conferencing, etc.).
- ACL (which users can book; which can override; admin role).
- Booking policy (advance notice, max duration, buffer).

When a user adds a room as an attendee, the room's calendar auto-RSVPs based on availability. Conflict detection happens at event-create time.

Per IP-journey-j113 (shift scheduling): rooms can be assigned to shift-based teams with per-shift booking quotas.

## How does FREEBUSY handle confidential events?

Per ADR-CAL-001 § Decision. Per RFC 5545:

- `CLASS:PUBLIC` (default): full visibility per ACL.
- `CLASS:PRIVATE`: NEVER detail-disclosable outside owner. Even `limited_details` mode shows BUSY-only.
- `CLASS:CONFIDENTIAL`: BUSY-only outside explicit privileged context (e.g., legal hold + court order).

`TRANSP:TRANSPARENT` events (e.g., focus time, OOO) do not block FREEBUSY by default but can be configured per tenant policy.

## What's the multi-region FREEBUSY behavior?

Per ADR-CAL-001 § Implementation Notes multi-region path. FREEBUSY queries execute in the target calendar's **home cell**. Remote callers receive policy-filtered results.

Why: keeps the policy evaluation co-located with the data; reduces cross-region round-trips; ensures pack residency rules are enforced at the source.

For compliance_pack-bound paid sovereign packs: details may not cross jurisdiction even if BUSY-only can.

## How does the calendar µservice differ from `meet`?

- `calendar`: scheduling, events, recurrence, FREEBUSY. The event-store.
- `meet`: video conferencing (similar to messenger huddles but standalone). Owns SFU + media pipeline.

A calendar event can have a `CONFERENCE` URL pointing to a Meet room. Calendar creates the event; Meet creates the room; the integration is via Cedar grant + iCalendar `CONFERENCE` field per RFC 7986.

## How is migration from Google Calendar handled?

See `migration-playbooks/from-google-calendar.md`. Short version:

1. Export Google Calendar via ICS export (Workspace Admin).
2. Run `oya calendar migrate import-google-calendar`.
3. Map Google Calendar IDs → oyatie calendar IDs (preserving owner).
4. Re-issue all sharing links (Google's URLs differ).
5. Reconfigure room resources.
6. Migrate iCalendar webhooks if any.
7. Shadow period 30-60 d; cutover via DNS-flip on CalDAV endpoint.
