---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: calendar
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-calendar + gtm-customer-success
deciders: axis-calendar, council-architecture
related_adrs: [ADR-0131, ADR-0132, ADR-0133, ADR-CAL-0001, ADR-CAL-0003]
related_artifacts:
  - microservices/calendar/contracts/openapi/calendar.yaml
  - microservices/calendar/contracts/proto/calendar.proto
  - microservices/calendar/contracts/asyncapi/calendar-events.yaml
  - microservices/calendar/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (calendar µservice)

## Purpose

Tenants integrate with calendar via three primary surfaces: standard
calendar protocols (CalDAV / iCalendar / iTIP / iMIP), the REST facade,
and programmatic SDKs. This document specifies the SDK strategy.

## Surface choice (first decision for tenants)

| Surface | Use when | Authority |
|---|---|---|
| CalDAV (RFC 4791) | Tenant uses Apple Calendar, Thunderbird, Evolution, DAVx5, GNOME / KDE clients | RFC 4791 + RFC 6638 + RFC 7953 |
| iCalendar (.ics) import/export | Tenant migrates from another calendar, or needs portable backup | RFC 5545 |
| iMIP / iTIP (mail-based invitations) | Cross-org invitations via standard mail | RFC 5546 + RFC 6047 |
| REST facade (calendar.yaml) | Tenant writes a custom calendar app or backend pipeline | OpenAPI 3.1.0 |
| gRPC (calendar.proto) | Tenant runs a backend service; wants strongly-typed contracts | proto3 |
| JMAP Calendars (M04-onward) | Modern JSON-over-HTTP; Fastmail-style integrations | draft-ietf-jmap-calendars (M04 once IETF stabilises per ADR-CAL-0003) |
| Per-language SDK | Tenant wants ergonomic auth + tenant binding + retry | this plan |

## Launch order (per ADR-CAL-0003)

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M03 (oyatie's own language) | First-party authored `oya-calendar-<bc>-sdk` crates per BC | axis-calendar |
| **TypeScript** | M03 (Node + Browser) | OpenAPI-generated baseline + first-party CalDAV client wrapper; published to npm | axis-calendar + gtm |
| **Python** | M03-onward1 (data-pipeline + scripting tenants) | OpenAPI-generated; published to PyPI; pairs with `caldav` reference lib | axis-calendar + gtm |
| **Swift** | M03-onward1 (iOS / macOS partner-app integrators) | thin wrapper over CalDAV + EventKit-shaped API; eventual JMAP switch at M04 | axis-calendar |
| **Go** | M04 (backend services + ops tools) | gRPC-generated baseline + ergonomic wrappers | axis-calendar + gtm |
| **JVM (Kotlin / Java)** | M04 (enterprise tenants) | gRPC-generated baseline; Maven Central | axis-calendar + gtm |
| **C# / .NET** | M05 (Microsoft-ecosystem tenants) | OpenAPI-generated; NuGet | axis-calendar + gtm |

Per ADR-CAL-0003: CalDAV (RFC 4791) ships first at M03; JMAP Calendars
scheduled-for-distinct-tracked-work to M04 once the IETF draft stabilises. M03 SDKs accordingly
ship CalDAV-wrapping clients; M04 SDKs gain JMAP Calendars support
additively (no breaking change to M03 surface).

## Generation strategy

### Rust SDKs (first-party)

Per-BC under `microservices/calendar/src/crates/oya-calendar-<bc>-sdk/`:

- `oya-calendar-event-store-sdk`: read events; write events; legal-hold; tenant-DEK envelope helper
- `oya-calendar-recurrence-engine-sdk`: client-side RRULE expansion via `rrule-rs` (mirrors server engine; consumers can pre-compute)
- `oya-calendar-availability-resolver-sdk`: free/busy queries; cross-tenant invite helper
- `oya-calendar-room-booking-sdk`: resource graph queries; booking + conflict
- `oya-calendar-invitation-flow-sdk`: send/receive iTIP REPLY; RSVP state helpers
- `oya-calendar-ics-import-export-sdk`: .ics parse/emit; CalDAV client wrapper

Common shape:
- `Client::new(opts)` with OIDC token provider closure.
- `Client` bound to tenant + calendar-context at construction;
  `X-Tenant-Id` + `X-Calendar-Context` headers automatic.
- Built-in exponential backoff for 5xx + 429.
- gRPC streaming where applicable (event lifecycle subscription).
- Re-exports types from corresponding `-kernel` crate.
- `#![deny(unsafe_code)]`.

### Generated SDKs

Pipeline (lives in `microservices/calendar/sdk-generation/`, future IP):

1. Source of truth: `contracts/openapi/calendar.yaml` + `contracts/proto/calendar.proto` + `contracts/asyncapi/calendar-events.yaml`.
2. OpenAPI → language: `openapi-generator-cli` 7.x with language profile.
3. Proto → language: `protoc` + language plugin.
4. AsyncAPI → language: `asyncapi-generator` 2.x for typed event subscription clients.
5. Ergonomic wrapper hand-authored on top: auth helpers, tenant-context binding, retry policy + circuit-breaker matching Rust SDK behavior.
6. Per-language CI lane: build + lint + integration-test against staging calendar cluster.

### CalDAV-compatible libraries (consume, don't re-author)

For CalDAV client integration, leverage upstream libraries:

- TypeScript: `dav` (Fastmail-maintained); wrap in ergonomic shim.
- Python: `caldav` (reference impl); wrap.
- Swift: native CalDAV via EventKit; thin wrapper exposing oyatie-specific tenant + context binding.
- Apple Calendar / Thunderbird / Evolution / DAVx5: use the standard CalDAV protocol against our Radicale backend; no SDK needed.

## Public surface (across SDKs)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| List calendars (by context) | `listCalendars(context)` | `Calendar[]` |
| Read calendar | `getCalendar(id)` | `Calendar` |
| List events (paginated; window-bounded) | `listEvents(calendar, window, cursor)` | `EventPage` |
| Read event | `getEvent(calendar, id)` | `CalendarEvent` |
| Create event | `createEvent(req)` | `CalendarEvent` |
| Update event | `updateEvent(req)` | `CalendarEvent` |
| Cancel event | `cancelEvent(id)` | `CalendarEvent` |
| Query free/busy | `queryFreeBusy(attendees, window)` | `FreeBusyProjection[]` |
| Book a room | `bookRoom(req)` | `Booking` |
| Send invitation | `sendInvitation(event_id, attendees)` | `InvitationDispatchReceipt` |
| Accept / decline / counter | `respondToInvitation(invitation_id, response)` | `RsvpReceipt` |
| .ics import | `importIcs(blob)` | `ImportJob` |
| .ics export | `exportIcs(calendar_id, window)` | `ExportJob` |
| Subscribe to events | `streamEventLifecycle()` | streaming events |

Helper utilities:
- Client-side RRULE expansion helper — Rust + TS + Python (mirrors server `rrule-rs` 0.13.x per ADR-CAL-0002).
- iCalendar canonicalisation helper — Rust + TS + Python (for strong-ETag computation per ADR-CAL-0001).
- tz-aware datetime builder — Rust + TS + Python (mirrors server chrono-tz behaviour per ADR-CAL-0004).

## Tenant SDK onboarding

| Step | Owner |
|---|---|
| Issue OIDC + per-tenant DEK reference via OpenBao | ops-security |
| Provide tenant onboarding doc + SDK quick-start (per language) | gtm-customer-success |
| Provide sample workflow: how to subscribe to `EventCreated` in tenant pipeline | axis-calendar |
| Provide CalDAV client tutorial (Apple / Thunderbird / DAVx5) | gtm + axis-calendar |
| Quarterly SDK update notifications (breaking changes 6mo advance) | axis-calendar |

## Sunset policy

| SDK | Sunset trigger | Window |
|---|---|---|
| Any SDK with < 1% tenant usage for ≥ 12mo | underused | 6mo advance + migration help |
| Generator lib upstream-deprecated | dep-deprecated | 12mo + auto-migrate where possible |
| Breaking API change in calendar µservice | per-release | major version bump in SDK; backwards-adapter for 1 prior major |

Per `agent-skills:deprecation-and-migration`: every sunset emits an ADR-shaped notice + deprecation-warning in SDK + tenant comms.

## Versioning

- calendar µservice: semver.
- SDK per language: matches calendar major.minor; SDK patch independent.
- Compat matrix per language; CI lane verifies SDK against current + 1 prior major.

## Open-source decision

Defer per-SDK OSS decision until API stable in production ≥ 6mo. Default: closed-source until tenant-driven request or competitive consideration. Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compat test: SDK version N+1 against calendar versions N-1, N, N+1.
- Annual SDK telemetry review per language; underused sunsetted.

## References

- `microservices/calendar/contracts/openapi/calendar.yaml`
- `microservices/calendar/contracts/proto/calendar.proto`
- `microservices/calendar/contracts/asyncapi/calendar-events.yaml`
- ADR-0105 (13-layer enum; `sdk` is canonical)
- ADR-CAL-0001 (CalDAV backend selection — Radicale primary)
- ADR-CAL-0002 (RRULE engine — `rrule-rs` 0.13.x)
- ADR-CAL-0003 (CalDAV at M03; JMAP Calendars at M04)
- ADR-CAL-0004 (IANA tzdb pin policy)
- OpenAPI Generator — `openapi-generator.tech`
- gRPC — `grpc.io`
- `caldav` (Python) — `github.com/python-caldav/caldav`
- `dav` (TypeScript) — `github.com/lambdabaa/dav`
- EventKit (Swift) — `developer.apple.com/documentation/eventkit`
- Stripe SDK precedent — `stripe.com/docs/libraries`
- `microservices/mail/sdk-plan.md` — sibling reference.
