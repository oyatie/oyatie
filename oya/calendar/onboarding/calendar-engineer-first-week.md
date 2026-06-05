---
doc_class: Onboarding
microservice: calendar
persona: calendar-engineer + scheduling-engineer + caldav-jmap-engineer
related_adrs: [ADR-CAL-001, ADR-CAL-0001, ADR-CAL-0002, ADR-CAL-0003, ADR-CAL-0004]
date: 2026-05-20
doc_status: published
---

# Calendar Engineer onboarding — first 5 working days on `calendar`

Audience: a new calendar engineer, scheduling engineer, or CalDAV/JMAP engineer joining the `calendar` rotation. By Day-5 they will have: bootstrapped a demo_trial cell, created their first event with recurrence, configured per-tenant FREEBUSY policy, exercised cross-tenant FREEBUSY grant (paid shadow), imported an ICS file, walked TZDB refresh + recurrence-storm runbooks.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 40 min). Note the four-vendor displacement + four-disclosure-mode doctrine.
2. Read `ARCHITECTURE.md` § event-store + § recurrence-engine + § freebusy-acl + § dual-context-isolation (∼ 60 min).
3. Read `decisions/ADR-CAL-001-icalendar-rfc5545-rfc7986-freebusy-acl.md` end-to-end (∼ 50 min). Binding architecture.
4. Read `decisions/ADR-CAL-0001..0004` (CalDAV backend, recurrence RFC conformance, JMAP-vs-CalDAV priority, TZDB refresh) (∼ 35 min total).
5. Read RFC 5545 §§ 1-3, RFC 7986 §§ 1-5, RFC 8607 (CalDAV scheduling extensions) section overviews (∼ 90 min).
6. Open the Grafana folder `calendar`. baseline boards: `calendar-freebusy-query-latency`, `calendar-recurrence-expansion-count`, `calendar-ics-import-reject-total`, `calendar-freebusy-cache-hit-ratio`, `calendar-tzdb-refresh-latency`, `calendar-cross-tenant-freebusy-grant-active-total`.
7. Walk `runbooks/README.md`. The on-call runbooks: `freebusy-cache-stale.md`, `recurrence-expansion-storm.md`, `tzdb-refresh-failed.md`, `caldav-sync-stuck.md`, `cross-tenant-freebusy-deny.md`, `ics-import-recurrence-bomb.md`, `dual-context-leak.md`, `room-booking-double-allocation.md`.
8. Sit in on the Wednesday calendar-substrate handoff.

Acceptance: you can sketch the FREEBUSY query path: requester → calendar-api → Cedar `calendar::freebusy::query` → check `FreebusyPolicy` + `FreebusyGrant` for (target_calendar, requester) → select disclosure mode → recurrence expansion (capped at 10k instances) → cache lookup (Valkey; TTL by pack) → audit-chain `EVT-CAL-FREEBUSY-DISCLOSED` → response. And the event create path: principal → Cedar `calendar::event::create` → recurrence parse → store `CalendarEvent` + `RecurrenceRule` rows → invalidate FREEBUSY cache for affected tenants/calendars → Kafka emit `calendar.event.changed.v1`.

## Day 2 — demo_trial cell bootstrap + first event

```text
Native operation: calendar bootstrap
Route: cloud control-plane operation ledger (not local retired CLI/raw Cargo)
Required evidence:
- Buck2 target(s) for the changed contract/runtime
- Prow/Kubernetes-native `oya-ci-required` job URL
- operation ledger id and emitted audit-chain event ids
```

Expected runtime: ≤ 12 min. Verify:

```sh
oya calendar health --cell drill-syd-1
# Expected:
#   postgres.calendar-events: up (lag_ms=14)
#   valkey.freebusy-cache: up (hit_ratio_24h=82%)
#   kafka.calendar-events: connected
#   radicale.caldav: up (port 5232)
#   tzdb_version: 2026a (current IANA release)
#   audit-chain.emit: up
```

Create a tenant + calendar + first event:

```sh
oya calendar tenant create \
    --cell drill-syd-1 \
    --tenant-id drill-acme \
    --display-name "ACME Calendar"

oya calendar calendar create \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --calendar-id alice-primary \
    --display-name "Alice's Primary Calendar" \
    --default-freebusy-policy busy_only

oya calendar event create \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --calendar alice-primary \
    --summary "Team standup" \
    --description "Daily team sync" \
    --start "2026-05-21T09:00:00+10:00" \
    --end "2026-05-21T09:15:00+10:00" \
    --class PUBLIC \
    --transparency OPAQUE
# Output:
#   event_id: e_drill_001
#   uid: 1f4a7c4a@calendar.drill-acme.oyatie.local
#   sequence: 0
#   audit_event_id: ae_cal_event_created_001
```

Add a recurrence rule:

```sh
oya calendar event recurrence-add \
    --tenant drill-acme \
    --event e_drill_001 \
    --rrule "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR;COUNT=20" \
    --tzid "Australia/Sydney"
# Output:
#   recurrence_rule_id: rr_drill_001
#   parsed_rrule: FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR;COUNT=20
#   tzdb_version: 2026a
#   estimated_expansion: 20 instances
#   audit_event_id: ae_cal_recurrence_added_001
```

Verify the event via CalDAV:

```sh
curl -X PROPFIND http://drill-radicale-syd-1:5232/u-alice/alice-primary/ \
    --user u-alice@drill.test:<password> \
    -H "Content-Type: application/xml" \
    -H "Depth: 1" \
    -d '<?xml version="1.0"?>
<propfind xmlns="DAV:">
  <prop><displayname/><resourcetype/></prop>
</propfind>'
# Response includes the calendar resource with displayname "Alice's Primary Calendar"
```

Acceptance: cell bootstrap; tenant + calendar + event + recurrence rule round-trip; CalDAV reachable.

## Day 3 — FREEBUSY policy + query (cross-user same-tenant)

Configure the default FREEBUSY policy:

```sh
oya calendar freebusy-policy update \
    --tenant drill-acme \
    --calendar alice-primary \
    --default-internal busy_only \
    --default-external none \
    --default-delegated limited_details
# Output:
#   policy_hash: ph_drill_acme_alice_001
#   audit_event_id: ae_cal_freebusy_policy_updated_001
```

Now query FREEBUSY from another user in the same tenant:

```sh
oya calendar freebusy query \
    --requesting-tenant drill-acme \
    --requesting-user u-bob@drill.test \
    --target-calendar drill-acme/alice-primary \
    --window-start "2026-05-21T00:00:00+10:00" \
    --window-end "2026-05-25T23:59:59+10:00" \
    --purpose "team-scheduling"
# Cedar evaluates:
#   - principal in same tenant as target ✓
#   - valid purpose ✓
#   - default_internal = busy_only → mode = busy_only
# Output:
#   freebusy_response:
#     - start: 2026-05-21T09:00:00+10:00
#       end: 2026-05-21T09:15:00+10:00
#       state: BUSY
#     - start: 2026-05-22T09:00:00+10:00
#       end: 2026-05-22T09:15:00+10:00
#       state: BUSY
#     ... (recurrence-expanded; capped at window)
#   disclosure_mode: busy_only
#   summary_disclosed: NO (mode is busy_only)
#   policy_hash: ph_drill_acme_alice_001
#   cache_hit: false (first query)
#   audit_event_id: ae_cal_freebusy_disclosed_001
```

Repeat the query — should hit cache:

```sh
oya calendar freebusy query ... [same params]
# Output: cache_hit: true; latency 8ms (vs 42ms first time)
```

Acceptance: FREEBUSY policy + query verified; cache hit verified.

## Day 4 — Cross-tenant FREEBUSY grant (paid shadow) + ICS import

Cross-tenant FREEBUSY grant (paid feature; shadowed at demo_trial):

```sh
# Alice grants a specific external user limited disclosure
oya calendar freebusy-grant create \
    --tenant drill-acme \
    --calendar alice-primary \
    --grantee-principal u-recruiter@drill-betta.test \
    --grantee-tenant drill-betta \
    --disclosure-mode busy_only \
    --expires-at 2026-06-20T00:00:00Z \
    --reason "Interview scheduling with drill-betta"
# Cedar drive::freebusy::grant ✓
# Output:
#   grant_id: fg_drill_001
#   audit_event_id: ae_cal_freebusy_grant_created_001
```

Now the external recruiter can query Alice's FREEBUSY:

```sh
oya calendar freebusy query \
    --requesting-tenant drill-betta \
    --requesting-user u-recruiter@drill-betta.test \
    --target-calendar drill-acme/alice-primary \
    --window-start "2026-05-25T00:00:00+10:00" \
    --window-end "2026-05-30T23:59:59+10:00" \
    --purpose "interview-scheduling"
# Cedar evaluates:
#   - cross-tenant: grant exists ✓
#   - disclosure_mode: busy_only
# Output: returns busy slots only, no titles, no participants
```

Import an ICS file (per ADR-CAL-001 § Decision):

```sh
# Create a sample ICS file
cat > ./meeting.ics <<EOF
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Test//Sample//EN
BEGIN:VEVENT
UID:imported-001@external.example
DTSTART:20260601T140000Z
DTEND:20260601T150000Z
SUMMARY:External meeting
LOCATION:Online
END:VEVENT
END:VCALENDAR
EOF

oya calendar event import-ics \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --calendar alice-primary \
    --ics-file ./meeting.ics
# Cedar calendar::ics::import ✓
# Recurrence expansion estimate within pack threshold ✓
# Output:
#   events_imported: 1
#   ics_uid: imported-001@external.example
#   internal_event_id: e_drill_002
#   parser_profile: calendar-ics-profile-v1
#   audit_event_id: ae_cal_ics_imported_001
```

Try to import a hostile ICS (per ADR-CAL-001 Constraint CAL-C12: "ICS import must tolerate hostile payloads"):

```sh
# This ICS has a pathological RRULE that would expand to 1 trillion instances
cat > ./hostile.ics <<EOF
BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
UID:bomb@external.example
DTSTART:20200101T000000Z
DTEND:20200101T010000Z
RRULE:FREQ=SECONDLY;COUNT=1000000000000
SUMMARY:Recurrence bomb
END:VEVENT
END:VCALENDAR
EOF

oya calendar event import-ics \
    --tenant drill-acme \
    --user u-alice@drill.test \
    --calendar alice-primary \
    --ics-file ./hostile.ics
# Expected: 422 Unprocessable Entity
# Reason: recurrence_expansion_estimate_exceeds_threshold (cap is 10k instances per query)
```

Acceptance: cross-tenant FREEBUSY grant works; ICS import works; recurrence bomb rejected.

## Day 5 — TZDB refresh + recurrence-storm runbook

TZDB refresh (per ADR-CAL-0004):

```sh
# Trigger controlled TZDB update
oya calendar tzdb refresh \
    --cell drill-syd-1 \
    --target-version 2026b \
    --dry-run true
# Output (dry-run):
#   from_version: 2026a
#   to_version: 2026b
#   affected_calendars: 14 (have events with timezones changed)
#   affected_events: 1 247
#   recalculation_estimate: 8s
#   PROCEED? (review affected events first)

# Actual refresh
oya calendar tzdb refresh \
    --cell drill-syd-1 \
    --target-version 2026b
# Output:
#   tzdb_version: 2026b
#   affected_future_instances_recalculated: 1 247
#   past_occurrences_preserved: true (per ADR-CAL-001 § Decision)
#   audit_event_id: ae_cal_tzdb_refreshed_001
```

Walk the recurrence-storm runbook. Read `runbooks/recurrence-expansion-storm.md`. Scenario: a poorly-formed RRULE causes a query to consume disproportionate CPU. Runbook covers:

1. Identify from `calendar-recurrence-expansion-count` panel (p99 spike).
2. Find the offending event via Postgres query.
3. Verify the recurrence cap (10k instances) is enforced.
4. If cap was bypassed (CVE scenario): patch + flag the event for re-validation.
5. Audit-chain emit incident report.

Acceptance: TZDB refresh + runbook walked.

## What you've learned

- demo_trial bootstrap + tenant + calendar + event + recurrence.
- FREEBUSY policy + four disclosure modes.
- Cross-user same-tenant FREEBUSY query.
- Cross-tenant FREEBUSY grant with expiration + reason.
- ICS import with recurrence-bomb defense.
- TZDB refresh with future-recalculation + past-preservation.

Next week: paid promotion (JMAP-for-Calendars conformance + cross-tenant FREEBUSY grants at scale + iMIP cross-tenant invites), paid tour (work-personal dual-context FREEBUSY + regulated calendar packs + per-tenant scheduling-link enclaves), compliance_pack-bound paid tour (per-pack residency enforcement + per-FREEBUSY-response audit-chain), and your first production shadow on cross-tenant FREEBUSY grant approval.
