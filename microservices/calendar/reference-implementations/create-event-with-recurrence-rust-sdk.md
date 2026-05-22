---
doc_class: ReferenceImplementation
microservice: calendar
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Create a recurring event + query FREEBUSY via the calendar Rust SDK

A runnable example that:

1. Authenticates as a tenant calendar_member principal.
2. Creates a calendar.
3. Creates an event with RFC 5545 RRULE.
4. Configures a FREEBUSY policy.
5. Issues a cross-tenant FREEBUSY grant.
6. Queries the FREEBUSY from the grantee's perspective.
7. Verifies audit-chain emissions.

## Cargo.toml

```toml
[package]
name = "calendar-recurring-event-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-calendar-client = { path = "../../../../crates/oya-calendar-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
icalendar = "0.16"
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.10"
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use oya_calendar_client::{
    CalendarClient, CalendarClientConfig,
    CalendarCreate, FreebusyPolicy, DisclosureMode,
    EventCreate, EventClass, EventTransparency,
    RecurrenceRule, RecurrenceFreq, RecurrenceByDay,
    FreebusyGrant, FreebusyQuery, FreebusyPurpose,
};
use oya_cedar_client::CedarPrincipal;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Construct the client.
    let principal = CedarPrincipal::from_env("CALENDAR_MEMBER_JWT")?;
    let client = CalendarClient::connect(CalendarClientConfig {
        cell_endpoint: std::env::var("CALENDAR_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 2. Create a calendar for Alice.
    let calendar = client.calendar_create(CalendarCreate {
        owner_principal: "u-alice@acme-corp.com".into(),
        calendar_id: "alice-primary".into(),
        display_name: "Alice's Primary Calendar".into(),
        timezone: "Australia/Sydney".into(),
        default_freebusy_policy: FreebusyPolicy {
            default_internal: DisclosureMode::LimitedDetails,
            default_external: DisclosureMode::BusyOnly,
            default_delegated: DisclosureMode::LimitedDetails,
        },
    }).await?;
    info!("Calendar created: {} ({})", calendar.calendar_id, calendar.display_name);

    // 3. Create a recurring event: daily standup, 09:00-09:15 Sydney time, weekdays.
    let sydney_tz: Tz = "Australia/Sydney".parse()?;
    let start = sydney_tz.with_ymd_and_hms(2026, 5, 21, 9, 0, 0).unwrap();
    let end = sydney_tz.with_ymd_and_hms(2026, 5, 21, 9, 15, 0).unwrap();

    let event = client.event_create(EventCreate {
        calendar_id: calendar.calendar_id.clone(),
        summary: "Team standup".into(),
        description: "Daily team sync".into(),
        location: "Sydney HQ — Meeting Room 4".into(),
        start: start.to_rfc3339(),
        end: end.to_rfc3339(),
        timezone: "Australia/Sydney".into(),
        class: EventClass::Public,
        transparency: EventTransparency::Opaque,
        attendees: vec![
            "u-alice@acme-corp.com".into(),
            "u-bob@acme-corp.com".into(),
            "u-charlie@acme-corp.com".into(),
        ],
        recurrence: Some(RecurrenceRule {
            freq: RecurrenceFreq::Weekly,
            by_day: Some(vec![
                RecurrenceByDay::Monday,
                RecurrenceByDay::Tuesday,
                RecurrenceByDay::Wednesday,
                RecurrenceByDay::Thursday,
                RecurrenceByDay::Friday,
            ]),
            count: Some(20),
            until: None,
            interval: 1,
        }),
        conference_url: Some("https://meet.acme-corp.com/standup".into()),
    }).await?;
    info!("Event created: {} (uid={}, sequence={})",
          event.event_id, event.uid, event.sequence);

    // 4. Issue a cross-tenant FREEBUSY grant to an external recruiter.
    let grant = client.freebusy_grant_create(FreebusyGrant {
        calendar_id: calendar.calendar_id.clone(),
        grantee_principal: "u-recruiter@betacorp-recruiting.com".into(),
        grantee_tenant: "betacorp-recruiting".into(),
        disclosure_mode: DisclosureMode::BusyOnly,
        window_start: "2026-05-25T00:00:00Z".into(),
        window_end: "2026-06-15T23:59:59Z".into(),
        expires_at: "2026-06-15T23:59:59Z".into(),
        reason: "Interview booking for backend engineer position".into(),
    }).await?;
    info!("FREEBUSY grant created: {}", grant.grant_id);

    // 5. Query FREEBUSY from the recruiter's perspective.
    let recruiter_principal = CedarPrincipal::from_env("RECRUITER_JWT")?;
    let recruiter_client = CalendarClient::connect(CalendarClientConfig {
        cell_endpoint: std::env::var("CALENDAR_ENDPOINT")?,
        tenant_id: "betacorp-recruiting".into(),
        principal: recruiter_principal,
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    let fb_response = recruiter_client.freebusy_query(FreebusyQuery {
        target_calendar: "acme-corp/alice-primary".into(),
        window_start: "2026-05-28T00:00:00+10:00".into(),
        window_end: "2026-05-30T23:59:59+10:00".into(),
        purpose: FreebusyPurpose::InterviewScheduling,
        requested_disclosure_mode: Some(DisclosureMode::BusyOnly),
    }).await?;
    info!("FREEBUSY response: {} busy slots, mode={:?}, cache_hit={}",
          fb_response.busy_slots.len(),
          fb_response.disclosure_mode,
          fb_response.cache_hit);

    for slot in &fb_response.busy_slots {
        info!("  BUSY: {} → {} ({})", slot.start, slot.end, slot.state);
    }

    Ok(())
}
```

## Expected output (against a paid-tier cell)

```
INFO Calendar created: alice-primary (Alice's Primary Calendar)
INFO Event created: e_acme_001 (uid=1f4a7c4a@calendar.acme-corp.oyatie.local, sequence=0)
INFO FREEBUSY grant created: fg_acme_001
INFO FREEBUSY response: 6 busy slots, mode=BusyOnly, cache_hit=false
  BUSY: 2026-05-28T09:00:00+10:00 → 2026-05-28T09:15:00+10:00 (BUSY)
  BUSY: 2026-05-28T11:00:00+10:00 → 2026-05-28T12:00:00+10:00 (BUSY)
  BUSY: 2026-05-28T14:00:00+10:00 → 2026-05-28T15:30:00+10:00 (BUSY)
  BUSY: 2026-05-29T09:00:00+10:00 → 2026-05-29T09:15:00+10:00 (BUSY)
  BUSY: 2026-05-29T10:00:00+10:00 → 2026-05-29T11:00:00+10:00 (BUSY)
  BUSY: 2026-05-30T09:00:00+10:00 → 2026-05-30T09:15:00+10:00 (BUSY)
```

## HTTP alternative (curl)

```sh
# 1. Create calendar
curl -X POST https://calendar.prod-syd-1.oyatie.local/v1/calendar/calendars \
    -H "Authorization: Bearer $CALENDAR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "owner_principal":"u-alice@acme-corp.com",
        "calendar_id":"alice-primary",
        "display_name":"Alice'\''s Primary Calendar",
        "timezone":"Australia/Sydney",
        "default_freebusy_policy":{
            "default_internal":"limited_details",
            "default_external":"busy_only",
            "default_delegated":"limited_details"
        }
    }'

# 2. Create event with RRULE
curl -X POST https://calendar.prod-syd-1.oyatie.local/v1/calendar/events \
    -H "Authorization: Bearer $CALENDAR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "calendar_id":"alice-primary",
        "summary":"Team standup",
        "start":"2026-05-21T09:00:00+10:00",
        "end":"2026-05-21T09:15:00+10:00",
        "timezone":"Australia/Sydney",
        "class":"PUBLIC",
        "transparency":"OPAQUE",
        "attendees":["u-alice@acme-corp.com","u-bob@acme-corp.com"],
        "recurrence":{
            "rrule":"FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR;COUNT=20"
        },
        "conference_url":"https://meet.acme-corp.com/standup"
    }'

# 3. Create FREEBUSY grant
curl -X POST https://calendar.prod-syd-1.oyatie.local/v1/calendar/calendars/alice-primary/freebusy-grants \
    -H "Authorization: Bearer $CALENDAR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "grantee_principal_id":"u-recruiter@betacorp-recruiting.com",
        "grantee_tenant_id":"betacorp-recruiting",
        "disclosure_mode":"busy_only",
        "window_start":"2026-05-25T00:00:00Z",
        "window_end":"2026-06-15T23:59:59Z",
        "expires_at":"2026-06-15T23:59:59Z",
        "reason":"Interview booking"
    }'

# 4. FREEBUSY query (from recruiter)
curl -X POST https://calendar.prod-syd-1.oyatie.local/v1/calendar/freebusy/query \
    -H "Authorization: Bearer $RECRUITER_JWT" \
    -H "X-Oya-Tenant-Id: betacorp-recruiting" \
    -H "Content-Type: application/json" \
    -d '{
        "target_calendar":"acme-corp/alice-primary",
        "window_start":"2026-05-28T00:00:00+10:00",
        "window_end":"2026-05-30T23:59:59+10:00",
        "purpose":"interview-scheduling"
    }'

# 5. ICS export (busy-only projection)
curl -X GET "https://calendar.prod-syd-1.oyatie.local/v1/calendar/events/e_acme_001/export-ics?projection=busy_only" \
    -H "Authorization: Bearer $CALENDAR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp"

# 6. CalDAV (RFC 4791) — for legacy clients
curl -X PROPFIND "https://caldav.prod-syd-1.oyatie.local/u-alice/alice-primary/" \
    --user u-alice@acme-corp.com:<password> \
    -H "Content-Type: application/xml" \
    -H "Depth: 1" \
    -d '<?xml version="1.0"?>
<propfind xmlns="DAV:">
  <prop><displayname/><resourcetype/></prop>
</propfind>'

# 7. JMAP-for-Calendars (paid; per ADR-CAL-0003)
curl -X POST https://calendar.prod-syd-1.oyatie.local/jmap/api \
    -H "Authorization: Bearer $CALENDAR_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "using":["urn:ietf:params:jmap:calendars"],
        "methodCalls":[
            ["Calendar/get", {"accountId":"acme-corp.u-alice"}, "0"]
        ]
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `calendar::event::create` |
| `recurrence_expansion_exceeds_threshold` | 422 | No | RRULE would expand to >10k instances; narrow the window or COUNT |
| `tzdb_version_mismatch` | 422 | No | Client's TZDB version differs significantly; refresh |
| `freebusy_grant_window_invalid` | 422 | No | Grant window doesn't fit pack policy |
| `cross_tenant_freebusy_deny` | 403 | No | Grant missing or expired |
| `ics_import_recurrence_bomb` | 422 | No | ICS file contains malicious recurrence; rejected |
| `pack_freebusy_residency_violation` | 403 | No | Pack requires details to stay in home jurisdiction |
| `event_conflict_resource` | 409 | No | Room/resource already booked |
| `cache_invalidation_pending` | 503 | Yes (auto, 1-2s backoff) | FREEBUSY cache being rebuilt |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `calendar_create` | `calendar.calendar.created.v1` |
| `event_create` | `calendar.event.created.v1` |
| `event_update` | `calendar.event.updated.v1` |
| `freebusy_policy_update` | `calendar.freebusy.policy.updated.v1` |
| `freebusy_grant_create` | `calendar.freebusy.grant.created.v1` |
| `freebusy_query` | `calendar.freebusy.disclosed.v1` |
| `ics_import` | `calendar.ics.imported.v1` |
| `tzdb_refresh` | `calendar.tzdb.refreshed.v1` |
| `room_booking` | `calendar.room.booked.v1` |
| `imip_sent` | `calendar.imip.sent.v1` |
| `imip_received` | `calendar.imip.received.v1` |
| Cedar deny anywhere | `calendar.cedar.denied.v1` |

## Where this file lives

`microservices/calendar/reference-implementations/create-event-with-recurrence-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/calendar/reference-implementations/event-create-example/` once `oya-calendar-client` ships.
