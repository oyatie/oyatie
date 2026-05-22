---
doc_class: MigrationPlaybook
microservice: calendar
vendor: Google Calendar (Personal + Workspace Business/Enterprise)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Google Calendar → oyatie calendar

Audience: a team running Google Calendar (Workspace Business or Enterprise) for organizational scheduling. Drivers: per-tenant four-disclosure-mode FREEBUSY + cross-tenant FREEBUSY grants + work-personal dual-context + sovereign-pack residency + cryptographic audit-chain + ~ 10% TCO reduction at 10k+ seats vs Google Workspace Business when calendar-specific.

## Why this migration matters

Google Calendar is excellent at:

- Best-in-class mobile UX.
- Tight integration with Gmail + Meet + Drive.
- AI-assisted features (suggested times, smart notifications).
- Free tier for individuals.
- Tight integration with Gmail iMIP.

oyatie calendar adds:

- **Four FREEBUSY disclosure modes** (none / busy_only / availability_class / limited_details; per ADR-CAL-001). Google has 2 effective modes (free/busy or full details).
- **Cross-tenant FREEBUSY grants with expiration + reason** (Google requires sharing the entire calendar).
- **Work-personal dual-context aggregation** (per ADR-CAL-001 Constraint CAL-C8). Google requires separate accounts; no aggregated dual-view.
- **Cryptographic audit-chain on disclosures** (Google's audit log is server-mutable).
- **Sovereign-pack residency**.
- **Per-tenant scheduling-link enclave** (each tenant gets its own Calendly-equivalent at paid tier).
- **Recurrence-bomb defense** (10 000 instance cap per query; Google's limit is unclear).
- **Per-FREEBUSY-response audit-chain** at compliance_pack-bound paid.
- **Direct CalDAV access** (Google deprecated CalDAV API in 2023).

Trade-off: Google's AI-assisted scheduling (suggested times, schedule analysis) is mature; oyatie's equivalent uses the `intelligence` µservice which is pack-gated for EU-AI-Act tenants. Calendar AI features may have reduced functionality for regulated tenants.

## Step 1 — Inventory the Google Calendar estate (≤ 1 week)

```bash
# Google Workspace Admin → Calendar Audit Log → Export to BigQuery
# Or use Google Calendar API for programmatic export
python3 -m google_calendar_export \
    --service-account-key ./sa-key.json \
    --customer-id C03dasdf12 \
    --output ./gcal-export/

# Per-user ICS export (also available via Calendar UI):
for user in $(cat ./users.txt); do
    gam export calendar $user --format ical --output ./gcal-export/$user.ics
done
```

Document:

- User count + per-user calendar count (typical: 2-10 calendars per user).
- Total event count (typical: 50k-500k events for 10k-user org).
- Shared calendars + their permissions.
- Resource calendars (rooms + equipment).
- iCalendar webhooks (Push notifications subscriptions).
- Google Workspace Marketplace apps with Calendar access.
- Domain-wide Delegation grants.
- Calendar settings: working hours, time zone defaults, notification preferences.

Typical mid-size: 1k-10k users, 5k-50k calendars, 500k-5M events.

## Step 2 — Map Google Calendar concepts to oyatie calendar (≤ 1 week)

| Google Calendar concept | oyatie calendar equivalent |
|---|---|
| User primary calendar | Per-user primary calendar with default FREEBUSY policy |
| Secondary calendar (e.g., "Personal", "Work") | Additional per-user calendars; per-calendar ACL |
| Shared calendar | Tenant-level calendar with Cedar role-mapped permissions |
| Resource calendar (room/equipment) | Room resource principal with availability ACL |
| Google Meet conferencing | RFC 7986 CONFERENCE URL (preserve Meet URLs; or replace with `meet`/`messenger` huddles) |
| Sharing settings (Public/See free/busy/See all event details) | Four-mode FREEBUSY ACL (none/busy_only/availability_class/limited_details) |
| Calendar invitations (Google iMIP via Gmail) | iMIP via `mail` µservice |
| Appointment slots (Workspace Plus+ feature) | Scheduling link (per ADR-CAL-001 §; per-tenant enclave at paid) |
| Working hours | Working-hours metadata on user calendar |
| Out of office | First-class OOO event class (paid with usage-sensitive billing_components) |
| Tasks | `tasks` µservice integration (separate from calendar events) |

## Step 3 — Data migration (≤ 2-6 weeks per 1M events)

```sh
oya calendar migrate import-google-calendar \
    --tenant acme-corp \
    --gcal-export-dir ./gcal-export/ \
    --map-user-calendars true \
    --map-shared-calendars-to-tenant-calendars true \
    --map-resource-calendars-to-rooms true \
    --preserve-uids true \
    --preserve-sequence-numbers true \
    --throttle-rate 5000-events-per-sec
```

The migration:

1. Creates oyatie tenants from Google Workspace domains.
2. Creates oyatie principals from Google users (preserve email + display name).
3. Imports calendars per user (primary + secondary).
4. Imports shared calendars as tenant-level calendars.
5. Imports resource calendars as room principals.
6. Imports events preserving UID + sequence (so cross-system threading is preserved).
7. Re-parses RRULE strings against RFC 5545.
8. Recomputes future occurrences against current TZDB; preserves past occurrences as-is.
9. Maps Google's "See free/busy" sharing to oyatie's `busy_only` mode.
10. Maps "See all event details" to `limited_details` mode.

Backfill rate ~ 5k events/sec at paid. 1M events → ~ 4 hours.

Verify post-import counts:

```sh
oya calendar tenant stats --tenant acme-corp
# Output:
#   total_calendars: 12 480
#   total_events: 1 247 821
#   total_rooms: 248
#   total_recurrence_rules: 84 213
#   imported_from: google-calendar
```

## Step 4 — Reconfigure DNS for CalDAV/JMAP endpoint (≤ 1 day)

```
# Add SRV records for CalDAV discovery (RFC 6764)
_caldavs._tcp.acme-corp.example.   SRV 0 1 443 caldav.prod-us-east-1.oyatie.local.
_carddavs._tcp.acme-corp.example.  SRV 0 1 443 caldav.prod-us-east-1.oyatie.local.

# JMAP discovery (per ADR-CAL-0003)
.well-known/jmap → https://calendar.prod-us-east-1.oyatie.local/jmap
```

## Step 5 — SSO + identity migration (≤ 1 week)

```sh
oya identity oidc-federation configure \
    --tenant acme-corp \
    --idp google-workspace \
    --client-id <google-oauth-client-id>
```

Users continue to sign in with Google → oyatie receives the OIDC claim + issues its own session.

## Step 6 — Shadow run + cutover (≤ 4-8 weeks)

Phase 1 (weeks 1-2): Read-only Google Calendar; new events go to oyatie. Users use both apps.
Phase 2 (weeks 3-4): Migrate one team at a time. Their calendar views shift to oyatie.
Phase 3 (weeks 5-6): Migrate user secondary calendars.
Phase 4 (weeks 7-8): DNS-flip CalDAV/JMAP discovery.

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.calendar_substrate.cut_over \
    --payload '{"from":"google-calendar","to":"oyatie","cutover_at":"2026-08-15T14:00:00Z"}'
```

## Step 7 — Google Calendar decommission (≤ 90-180 d post-cutover)

After ≥ 90 d:

- Export final calendar state for archival.
- Cancel Google Workspace Calendar component (if separable from broader Workspace).
- Retain ICS archive for legal-hold duration.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| iCalendar Webhook subscriptions break | High | Re-issue oyatie webhook subscriptions; provide compatibility shim during shadow phase |
| Marketplace apps with Calendar access | High | Pre-audit; port top-10 apps; defer long-tail to community plugin SDK |
| Google Meet conference URLs in events | Medium | Preserve URLs (RFC 7986 CONFERENCE); or replace with `meet`/`messenger` huddles via post-migration script |
| Recurrence rule edge cases (RRULE with custom extensions) | Medium | RFC 5545 strict parse; flag non-conformant for manual review |
| Time zone changes mid-migration | Medium | TZDB pinning per event preserves past; future occurrences recalculated against latest TZDB |
| Mobile app transition (Google Calendar mobile vs oyatie mobile) | Medium | Side-by-side iOS/Android available; rolling cutover |
| Smart suggestions (Google's AI features) | Medium | oyatie has `intelligence` µservice scheduling assist (pack-gated for EU-AI-Act) |
| Working hours not 1:1 (per-user vs per-calendar) | Low | Map Google's per-user → oyatie per-calendar; default to per-user equivalent |
| Out of office events | Low | Map to OOO event class (paid with usage-sensitive billing_components) or status |
| Quick add feature (natural language event creation) | Low | oyatie has equivalent via `intelligence` µservice NLU |
| Tasks integration | Medium | Tasks separated to `tasks` µservice; calendar shows tasks as overlay |
| Apple Calendar sync (uses CalDAV) | Low | oyatie CalDAV at all tiers; Apple Calendar works natively |
| Outlook calendar sync (Outlook can use CalDAV via add-on) | Low | oyatie CalDAV at all tiers |
| Calendar discovery (Google account-driven autoconfig) | Medium | SRV records published per Step 4; clients re-discover on first sync |
| Cross-org sharing (Google Workspace external sharing) | Medium | Replay as cross-tenant FREEBUSY grants per ADR-CAL-001 |
| Calendar API call volume from third-party tools | Medium | oyatie REST + JMAP supports equivalent rate limits; tenants may need to update SDK URLs |
| Google Calendar embed widgets on websites | Medium | oyatie provides equivalent embed widget; URL pattern changes |
| Workspace Marketplace integrations (booking systems, etc.) | Medium | Per-integration replacement plan; oyatie's plugin SDK can host most |
| Resource (room) calendars + booking | Medium | Map to oyatie room principals; auto-RSVP based on availability |
| Calendar settings per-user (notifications, reminders) | Low | Map to oyatie user calendar preferences |
| Smart Compose for event descriptions | Low | oyatie's `intelligence` µservice (pack-gated) |
| Public calendars (read-only embed via URL) | Low | oyatie equivalent via signed share-link with `viewer` permission |
