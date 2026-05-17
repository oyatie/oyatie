---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: calendar
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-calendar + ops-security
deciders: council-architecture, ops-security, axis-calendar, council-privacy
methodology: STRIDE + LINDDUN + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140]
review_cadence: quarterly + on every BC architectural change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1-CC6.8, CC7.1-CC7.5, CC8.1"
  - "ISO 27001:2022: A.5.7-A.5.34, A.8.2-A.8.34"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR-ISMS-P §2.1-2.12", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "eIDAS 910/2014", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2/27"]
doc_status: published
---

# Threat Model: calendar µservice

## Purpose

Identify, classify, and mitigate threats to the calendar µservice's confidentiality, integrity, availability, and privacy posture. Calendar carries dual-context PII (personal + professional events), attendee identities, organisational schedules, and cross-tenant invitation flows; a compromise here cascades into operational privacy, productivity-data exposure, and potential surveillance. This document is the canonical security artifact reviewed by SOC 2 / ISO 27001 / GDPR DPAs.

## Scope

### In-scope

All components introduced for the calendar µservice across the six bounded contexts (event-store, recurrence-engine, availability-resolver, room-booking, invitation-flow, ics-import-export), deployed in the tenant workload cluster:

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres (event store) | `oya-calendar-event-store-*` (10 crates) |
| Redis (availability cache) | `oya-calendar-recurrence-engine-*` (6 crates) |
| chrono-tz / IANA tzdata | `oya-calendar-availability-resolver-*` (9 crates) |
| `icalendar-rs` (vetted fork, RFC 5545 parse/emit) | `oya-calendar-room-booking-*` (7 crates) |
| in-house CalDAV (RFC 4791) adapter | `oya-calendar-invitation-flow-*` (7 crates) |
| | `oya-calendar-ics-import-export-*` (9 crates) |

### Out-of-scope

- Underlying Kubernetes / IaaS layer (owned by `cloud-k8s`).
- Mail delivery (owned by `mail` µservice).
- Tenancy / identity (owned by `tenancy` µservice).
- Audit-chain seal infrastructure (owned by `audit-chain` µservice).
- Observability collectors (owned by `observability` µservice).

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Tenant operators        Customer apps        External attendees          │
│         │                       │                       │                  │
│         │ (HTTPS+OIDC+MFA)      │ (per-tenant API key)  │ (RFC 5546 reply) │
│         ▼                       ▼                       ▼                  │
│  ┌─ Public ingress (Envoy + WAF + DDoS) ──────────────────────────────┐    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Tenant workload cluster ──────────────────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → REST + CalDAV ingress                        │
│  ┌─ calendar-event-store-rest ─┐ ┌─ calendar-availability-resolver-rest ─┐ │
│  │ OIDC + RLS                  │ │ OIDC + Cedar pillar guard             │ │
│  └─────────────────────────────┘ └───────────────────────────────────────┘ │
│  ┌─ calendar-room-booking-rest ─┐ ┌─ calendar-ics-import-export-rest ────┐ │
│  │ tenant-scoped + RBAC         │ │ .ics parse + CalDAV + import/export  │ │
│  └──────────────────────────────┘ └──────────────────────────────────────┘ │
│                                                                            │
│  Trust boundary 2: REST → Postgres (per-tenant RLS + tenant-DEK)           │
│  ┌─ Postgres (event-store; per-tenant RLS) ─────────────────────────┐      │
│  │  Row-level security; encryption-at-rest; tenant-DEK envelope     │      │
│  └──────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 3: REST → Redis (availability cache, per-tenant key prefix)│
│                                                                            │
│  Trust boundary 4: Cross-tenant availability resolver → remote tenant      │
│       (over mTLS internal mesh; Cedar `cross-tenant-grant` policy)         │
│                                                                            │
│  Trust boundary 5: invitation-flow → mail µservice (Workflow event)        │
│       (per-event lifecycle; mail µservice owns delivery + spam reputation) │
│                                                                            │
│  Trust boundary 6: CalDAV ingress → ics-import-export-adapter-caldav       │
│       (per-tenant API key over Basic Auth + HTTPS; RFC 4791)               │
│                                                                            │
│  Trust boundary 7: Workers (retention sweep + recurrence expand) → DB      │
│       (SPIFFE-identity bound; not user-callable)                           │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Seven trust boundaries.

## Assets & Data Classification

Per Bominal ADR-0028 + `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Event title + description + location (Professional context) | `PROFESSIONAL_EVENT_CONTENT` (tenant-DEK encrypted) | Critical | per jurisdiction + legal hold | Postgres |
| Event title + description + location (Personal context) | `PERSONAL_EVENT_CONTENT` (E2E where tenant declares) | Critical | per jurisdiction + legal hold | Postgres |
| Attendee list (emails + display names) | `PII_IDENTIFYING` | High | per event retention | Postgres + audit-chain |
| RSVP state | `PII_IDENTIFYING` | Medium | per event retention | Postgres |
| Cross-tenant availability projection (free/busy only) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | cache TTL ≤ 60s | Redis |
| Cross-tenant invitation grant | `AUDIT` + `SENSITIVE_PIPA_ART23` | High | append-only | Postgres + audit-chain |
| Resource (room) graph | `BEHAVIORAL_TENANT_PRODUCT` | Medium | per resource retention | Postgres |
| Bookings | `BEHAVIORAL_TENANT_PRODUCT` | Medium | per event retention | Postgres |
| Legal-hold records | `AUDIT` | Critical | append-only; preserved past retention | Postgres + audit-chain |
| Tenant-DEK | `SECRET` | Critical | OpenBao 90d rotation | OpenBao |
| CalDAV per-tenant API key | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| .ics import payloads in flight | `PERSONAL_EVENT_CONTENT` / `PROFESSIONAL_EVENT_CONTENT` (per source) | Critical | transient (parsed + dropped) | tmpfs |
| Audit-chain seal records | `AUDIT` | High | append-only | audit-chain µservice |
| IANA tzdata (system-shared) | `INTERNAL_ONLY` | Low | hourly refresh | shared volume |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| Tenant operator (human) | Untrusted external | OIDC + MFA | RW own tenant's events / resources |
| Customer app (machine) | Untrusted external | per-tenant API key (30d rotation) | RW own tenant's calendar via SDK / REST |
| External attendee | Untrusted external | none (cookie-bound RSVP token + RFC 5546 reply) | RSVP only on invitations they received |
| CalDAV client (Apple Calendar / Thunderbird / Evolution) | Untrusted external | per-tenant API key over Basic Auth + HTTPS | RW own tenant's calendar via RFC 4791 |
| Remote tenant (cross-tenant availability query) | Semi-trusted | mTLS internal mesh + Cedar grant | free/busy projection only |
| Workflow µservice | Trusted internal | mTLS + SPIFFE | trigger event-bound automation |
| Mail µservice | Trusted internal | mTLS + SPIFFE | invitation delivery on behalf of calendar |
| Tenancy µservice | Trusted internal | mTLS + SPIFFE | identity resolution |
| Audit-chain µservice | Trusted internal | mTLS + SPIFFE | seal emission |
| Worker (retention sweep / recurrence expand) | Trusted internal | SPIFFE + OpenBao SA token | RW on event-store |
| Council-architecture / ops-security | Trusted internal | OIDC + MFA + JIT | admin-level access |
| External auditor (SOC 2 / ISO 27001) | Read-only time-boxed | OIDC + MFA + JIT ≤ 4h | read-only |
| Attacker (opportunistic / targeted) | Untrusted | none | — |
| Insider (accidental / malicious) | Trusted internal | OIDC + MFA | mitigated via PR review + LEAN gates + audit-chain |

## STRIDE Threat Catalog

Each threat: ID; asset; description; likelihood (L/M/H); impact (L/M/H); risk; mitigations; owner; residual; framework controls.

### Spoofing

**T-S-01 — External attendee impersonates another invitee via crafted RFC 5546 reply**
- Asset: invitation-flow RSVP intake
- L M / I H / Risk H
- Mitigations:
  - RSVP reply must carry a single-use HMAC-signed token bound to `(invitation_id, recipient_email)`; token rotated on every state transition.
  - Mail µservice signs outbound invitations with DKIM + SPF + DMARC; replies validated against original `Message-ID` + token signature.
  - Counter-proposal flow requires re-confirmation by event organiser before persisting.
- Owner: axis-calendar + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC7.1; ISO 27001 A.5.15, A.8.5, A.8.7; GDPR Art. 32(1)(b)

**T-S-02 — Attacker forges cross-tenant availability grant**
- Asset: `CrossTenantInviteGrant` audit chain
- L M / I H / Risk H
- Mitigations:
  - Grant write requires Ed25519 signature from both tenants' tenancy-µservice SPIFFE identity.
  - Cedar policy `cross-tenant-availability-grant` (NEW) refuses grant unless both tenants' onboarding-state + jurisdiction-pack permit cross-pack disclosure.
  - LEAN check `oya-check-cross-tenant-grant-signature` verifies signature chain at PR time.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.15, A.8.7; GDPR Art. 32; KR PIPA Art. 23-2

**T-S-03 — CalDAV client impersonates another tenant via stolen API key**
- Asset: CalDAV REST
- L M / I H / Risk H
- Mitigations:
  - Per-tenant API key bound to `(tenant_id, device_id)`; rotation 30d; revocation on suspicion.
  - CalDAV requests carry tenant claim non-modifiable (server-side mapping; not client header).
  - Rate-limit + anomaly detection on per-key access patterns; suspicious patterns trigger forced re-auth.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.5.17, A.8.5

**T-S-04 — Attacker forges room-booking on behalf of another user**
- Asset: room-booking REST
- L L / I M / Risk L-M
- Mitigations:
  - Booking write requires OIDC token with `calendar:room:book` scope; booker identity recorded.
  - Cedar policy refuses booking when booker is not in the target resource's allow-list.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3

### Tampering

**T-T-01 — Recurrence rule (RRULE) tampering to generate spam**
- Asset: event recurrence
- L M / I H / Risk H
- Mitigations:
  - RRULE horizon hard-capped at 5y; INTERVAL+COUNT validated.
  - `oya-check-rrule-bounds` LEAN lane (NEW) refuses build if API accepts unbounded RRULE.
  - Per-tenant rate limit on event creation.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6, A.8.32; GDPR Art. 32(1)(c)

**T-T-02 — .ics injection attack (CRLF / vulnerability via crafted payload)**
- Asset: .ics import path
- L H / I H / Risk H
- Mitigations:
  - Hardened RFC 5545 parser (vetted fork of `icalendar-rs`) with input bounds: max events/file ≤ 100k, max line length ≤ 8KB, max recurrence horizon ≤ 5y.
  - Parser rejects malformed input rather than auto-repair; no fallback to lax parse.
  - Fuzzing: `cargo fuzz` corpus + RFC 5545 + RFC 5546 known-bad inputs.
  - Per-tenant rate limit on .ics imports; max 10/hour.
  - `oya-check-ics-parser-conformance` LEAN lane (NEW) validates parser against RFC 5545 corpus + OWASP injection corpus.
- Owner: axis-calendar + ops-security
- Residual: M (fuzz corpus baseline)
- Frameworks: SOC 2 CC6.7, CC7.1; ISO 27001 A.8.28; GDPR Art. 32; OWASP Top 10 A03:2021 (Injection)

**T-T-03 — Tenant-DEK substitution on event read (downgrade attack)**
- Asset: tenant-DEK envelope encryption
- L L / I H / Risk M
- Mitigations:
  - Envelope encryption per Bominal ADR-0111; ciphertext records carry a binding to the DEK ID + signed integrity check.
  - DEK rotation event re-encrypts; old DEKs maintained for read-only past-record decryption only.
  - LEAN check `oya-check-dek-binding-integrity` validates ciphertext binding.
- Owner: ops-security + cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.8.24, A.8.25; GDPR Art. 32(1)(a)

**T-T-04 — Recurrence-engine cache poisoning leads to false occurrence times**
- Asset: recurrence expansion cache
- L L / I M / Risk L-M
- Mitigations:
  - Cache key is `(event_id, version, window_hash)`; cache invalidates on event version increment.
  - Cache stored in Redis with per-tenant prefix; cross-tenant read forbidden by Redis ACL.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.21

**T-T-05 — Audit-chain seal omission for event update**
- Asset: audit emission
- L L / I H / Risk M
- Mitigations:
  - Every event write path emits via `audit-chain` µservice port; LEAN check `oya-check-audit-emission-coverage` refuses build if any usecase mutating events skips emission.
  - Audit-chain µservice acks emission; missing acks trigger `held` SLO state via observability.
- Owner: audit-chain + axis-calendar
- Residual: L
- Frameworks: SOC 2 CC4.1, CC7.2, CC8.1; ISO 27001 A.5.28, A.8.15; GDPR Art. 5(2), Art. 30

### Repudiation

**T-R-01 — Event organiser denies sending a meeting invitation**
- Asset: invitation chain
- L L / I M / Risk L-M
- Mitigations:
  - InvitationDispatch carries organiser SPIFFE-identity + Ed25519 audit-chain seal.
  - Mail µservice's outbound spool retains the signed payload for 90d.
- Owner: axis-calendar + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.28, A.8.15

**T-R-02 — Attendee disputes RSVP state ("I never declined")**
- Asset: RSVP state
- L M / I M / Risk M
- Mitigations:
  - Every RSVP transition emits audit record with HMAC-signed reply-token; ledger replayable.
  - Receipt of reply (mail-delivery proof) cross-correlates.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.15; GDPR Art. 5(2)

**T-R-03 — Room-booking actor denies booking authorship**
- Asset: booking audit chain
- L L / I M / Risk L
- Mitigations:
  - Booking writes carry booker OIDC subject + Ed25519 audit-chain seal.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.15

### Information Disclosure

**T-I-01 — Cross-tenant availability projection leaks event details (title / attendees / location)**
- Asset: cross-tenant availability projection
- L M / I H / Risk H
- Mitigations:
  - Projection is type-narrowed (Rust type system + Cedar policy) to `{starts_at, ends_at, busy: bool}` only.
  - LEAN check `oya-check-cross-tenant-availability-projection` (NEW) refuses build if projection includes raw fields.
  - Penetration test against projection boundary annually + on every BC change.
  - Threat hunt: weekly query: any cross-tenant lookup returning > 3 fields = alarm.
- Owner: axis-calendar + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.15, A.8.12; GDPR Arts. 5(1)(f), 25, 32; KR PIPA Art. 23

**T-I-02 — Personal-context event leaks into Professional context query**
- Asset: dual-context isolation
- L M / I H / Risk H
- Mitigations:
  - Context field is non-nullable + immutable post-creation; Cedar policy `event-isolation.cedar` refuses cross-context read.
  - Rust type system: separate types `PersonalEvent` vs `ProfessionalEvent`; no shared parent struct that allows leakage.
  - LEAN check `oya-check-context-isolation` (NEW) validates no usecase reads both contexts in same query.
- Owner: axis-calendar + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3; GDPR Art. 5(1)(b), 25

**T-I-03 — Attendee enumeration via timing side-channel**
- Asset: invitation existence
- L M / I M / Risk M
- Mitigations:
  - RSVP endpoint returns constant-time response whether invitation exists or not.
  - Per-IP rate limit on RSVP endpoint; anomaly detection on enumeration patterns.
- Owner: ops-security
- Residual: M (timing side-channel is hard to fully eliminate)
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.5

**T-I-04 — .ics export contains attendee emails beyond minimum-necessary**
- Asset: .ics export path
- L M / I M / Risk M
- Mitigations:
  - Export filters attendee emails based on requestor role; non-organisers receive only their own RSVP record.
  - Public CalDAV PROPFIND on shared collections strips attendee details by default; tenant can opt-in to include.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12; GDPR Art. 5(1)(c)

**T-I-05 — Room booking exposes confidential meeting subject via room calendar**
- Asset: room calendar view
- L M / I M / Risk M
- Mitigations:
  - Room calendar by default shows only `{starts_at, ends_at, booker}`; subject visible only to booker + organisation admin.
  - Cedar policy `room-calendar-projection.cedar`.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12

**T-I-06 — Tenant-DEK leaked via log emission**
- Asset: encryption keys
- L M / I H / Risk H
- Mitigations:
  - DEK wrapped in `Secret<T>` type with stripped `Debug` impl; never serializable.
  - Secret-scanner CI lane (`oya-foundry-fitness-evidence-secret-scan`) scans every commit + log emission.
  - Rotation: 90d for tenant-DEK; rotation event re-encrypts active records.
- Owner: ops-security + cloud-secrets
- Residual: M (human-error baseline)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32

### Denial of Service

**T-D-01 — Recurrence storm: malicious tenant submits 1000s of complex RRULEs**
- Asset: recurrence engine
- L H / I H / Risk H
- Mitigations:
  - RRULE complexity bound at API: max INTERVAL=1, max COUNT=10000, max horizon 5y.
  - Worker rate-limit per tenant: max 100 RRULE expansions/min.
  - Worker queue depth alarm; burst capacity is bounded with backpressure.
  - Per-tenant cost-meter: cumulative expansion seconds budgeted; excess returns 429.
- Owner: ops-sre-reliability + axis-calendar
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Availability resolver cache-miss storm**
- Asset: Redis availability cache
- L M / I H / Risk H
- Mitigations:
  - Cache TTL ≤ 60s + jitter ± 5s prevents synchronized expiry.
  - Per-tenant rate limit on availability lookups.
  - Stampede protection: single-flight per `(tenant, attendees, window)`.
  - When cache fully unavailable, resolver degrades to "unknown" rather than blocking.
- Owner: axis-calendar + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-03 — Time-zone DB stale → wrong DST transitions**
- Asset: IANA tzdata
- L L / I H / Risk M
- Mitigations:
  - Hourly tzdata refresh job; alert on > 6h staleness.
  - Pre-deploy gate refuses promotion if tzdata > 30d stale.
  - chrono-tz pinned version + CI lane validates against upstream IANA.
- Owner: ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6

**T-D-04 — Room-booking race condition double-books**
- Asset: room availability
- L M / I M / Risk M
- Mitigations:
  - Postgres SELECT … FOR UPDATE on resource row before INSERT booking; serialized at DB level.
  - Idempotency key on booking request prevents retry storms.
  - Booking conflict yields structured error + suggested alternative slots.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.14

**T-D-05 — .ics import fails on malformed input → tenant locked out**
- Asset: import job
- L M / I M / Risk M
- Mitigations:
  - Import job runs async with progress reporting; malformed input does NOT block tenant's other operations.
  - Per-import-job size limit (100k events max); excess paginated.
  - Parser timeout: max 5 min/job; exceeding kills job with status `aborted`.
  - Per-event error reporting: partial-success import allowed (parsable events imported; unparsable reported).
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.14, A.8.16

**T-D-06 — Invitation fanout flood (large attendee list)**
- Asset: mail µservice + invitation dispatcher
- L M / I M / Risk M
- Mitigations:
  - Max attendees/event = 1000 (hard); soft 200; > soft triggers explicit confirmation.
  - Invitation dispatch is async + rate-limited at mail µservice boundary.
  - Per-tenant invitation budget (daily); excess delayed.
- Owner: mail + axis-calendar
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6

### Elevation of Privilege

**T-E-01 — Attendee escalates to organiser via crafted RSVP**
- Asset: event ownership
- L L / I H / Risk M
- Mitigations:
  - Event ownership transfer requires organiser explicit action; never inferred from RSVP.
  - Cedar policy `event-ownership-transfer.cedar` refuses transfer without organiser OIDC token.
- Owner: axis-calendar + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3

**T-E-02 — CalDAV client uses HTTP PUT to overwrite another user's calendar**
- Asset: CalDAV write
- L L / I H / Risk M
- Mitigations:
  - CalDAV collection scoped by `(tenant_id, user_id)`; cross-user write returns 403.
  - PUT path validation against client's bound calendar.
- Owner: axis-calendar
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.3

**T-E-03 — Worker SA token leaked → arbitrary event writes**
- Asset: worker ServiceAccount
- L L / I H / Risk M
- Mitigations:
  - SA token bound to pod identity; rotation 24h.
  - Network policy: worker → DB only; not user-facing.
  - Worker writes are scoped to system-emitted events (audit + retention sweep); user-facing writes go via REST.
- Owner: ops-security + axis-calendar
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.7

**T-E-04 — Legal-hold bypass via raw DB access**
- Asset: legal-hold preservation
- L L / I H / Risk M
- Mitigations:
  - Postgres role for application has no DELETE permission; only soft-delete via row column.
  - Hard-delete restricted to a `purge-with-2-person-rule` admin script audited via audit-chain.
  - Periodic integrity scan: compare hold-set vs Postgres rows; mismatch alerts.
- Owner: ops-security + compliance
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.27, A.8.4; GDPR Art. 17 (right to erasure carve-outs)

## LINDDUN Privacy Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | event attendee list | repeated meetings link individuals into a social graph | tenant-DEK + access controls; cross-tenant aggregations require explicit grant + audit | M (legitimate use case) |
| T-L-02 | Identifiability | event location strings | "Room 314, 5th Floor" + "AM Meeting with Tax Attorney" identifies individual | redaction in cross-tenant projection; per-event privacy classification | L |
| T-L-03 | Non-repudiation | RSVP state | end-user disputes RSVP authorship | HMAC-signed reply token + audit chain | L |
| T-L-04 | Detectability | meeting timing | burst of events correlates with business events (M&A diligence pattern) | reasonable disclosure (tenant onboarding); no broader mitigation possible | M |
| T-L-05 | Disclosure | external CalDAV exposure | CalDAV is internet-accessible; tenant misconfiguration may expose | per-tenant API key required; default private; LEAN check on public-collection drift | L |
| T-L-06 | Unawareness | end-user (the tenant's user) of cross-tenant availability sharing | end-user may not know their availability is shared externally | tenant DPA mandates upstream disclosure; default opt-out | M-H (joint controllership) |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | erasure of an attendee identifier across many events | DSR cascade: scan all events for the identifier; tombstone the attendee record; preserve event minus the identifier; legal hold may override | M (best-effort within hold) |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres per-tenant RLS | Preventive | axis-calendar | `oya-check-rls-coverage` LEAN lane |
| Tenant-DEK envelope encryption | Preventive | cloud-secrets | DEK binding integrity check |
| Cedar `event-isolation.cedar` | Preventive | ops-security | policy unit-tests |
| Cross-tenant projection type-narrowing | Preventive | axis-calendar | LEAN check + pen-test |
| RFC 5545 hardened parser (vetted fork) | Preventive | axis-calendar | fuzz corpus |
| RRULE bounds enforcement | Preventive | axis-calendar | LEAN check |
| RSVP HMAC-signed reply token | Preventive (S+R) | axis-calendar | audit-chain replayability |
| Ed25519 audit-chain seal | Detective + non-repudiation | audit-chain | per-event emission |
| Per-tenant rate limits (events / availability / .ics / invitations) | Preventive (DoS) | ops-sre-reliability | metrics |
| Hourly tzdata refresh + staleness gate | Preventive | ops-sre-reliability | freshness metric |
| Postgres FOR UPDATE on room rows | Preventive (race) | axis-calendar | concurrency tests |
| SA-token rotation 24h, DEK 90d, API key 30d | Preventive | cloud-secrets | OpenBao audit |
| 2-person rule on hard-delete | Preventive (insider) | ops-security | OpenBao JIT |
| DSR cascade runner | Compliance | council-privacy | DSR queue SLO |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review |
|---|---|---|---|
| T-T-02 (.ics injection) | M | fuzz corpus baseline; never fully eliminable | Quarterly |
| T-I-03 (timing side-channel) | M | inherent network-timing characteristic | Annually |
| T-I-06 (DEK leak via logs) | M | human-error baseline | Quarterly |
| T-L-01 (linkability) | M | legitimate calendar use case | Annually |
| T-L-04 (detectability via timing) | M | tenant business reality | Annually |
| T-L-06 (joint-controllership unawareness) | M-H | tenant-of-tenant disclosure responsibility | Annually |
| T-L-07 (right-to-erasure best-effort) | M | hold-vs-erasure tension | Annually |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlays

### pack-kr (KR PIPA + ISMS-P + 전자문서법)

- **PIPA Art. 23 (sensitive personal information)**: meeting subjects may contain Art. 23 categories (health, political, sexual orientation, etc.) when discussed. Mitigation: per-tenant Cedar policy can mark certain rooms / event types as "sensitive"; flagged events get additional access restrictions.
- **PIPA Art. 23-2 (cross-border)**: cross-tenant availability across jurisdictions requires SCC-equivalent at tenant-DPA level.
- **PIPA Art. 29 (technical safeguards)**: every STRIDE mitigation maps to one of the 12 prescribed safeguards.
- **전자문서법 Art. 5**: audit-chain Ed25519 seal satisfies electronic-document integrity requirement.
- **ISMS-P §2.5 + §2.7**: 2-person rule + JIT elevation map directly.
- **Korean holidays**: pack-kr overlay ships Korean lunar-calendar holiday set; events touching public holidays auto-flag for reschedule confirmation.

### pack-us-healthcare (HIPAA)

- **§164.502 (Minimum Necessary)**: cross-tenant projection enforces minimum-necessary at the type level.
- **§164.312 (Technical Safeguards)**: per-tenant RLS + tenant-DEK + audit-chain satisfy access-control + audit-log + integrity-verification + encryption-and-decryption.
- **§164.316 (Documentation)**: this threat model + DPIA + compliance.md retained ≥ 6y.
- **§164.502(e) (BAA)**: BAA required pre-tenant; clinical-scheduling tenants pinned to pack-us-healthcare region.
- **45 CFR Part 164 Subpart D (Breach Notification)**: integrated into `incident-response.md`.
- **HIPAA-class retention**: ≥ 6y for clinical events; longer than default; cost-budget reflects.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS)

- **GDPR Art. 25**: privacy-by-design baked into Rust type system (Personal vs Professional context separation).
- **GDPR Art. 35**: DPIA in `dpia.md` satisfies high-risk processing.
- **GDPR Art. 28 (processor)**: tenant DPA template + sub-processor list.
- **GDPR Art. 32**: every "T-*-NN" mitigation contributes.
- **GDPR Arts. 44–50 (transfers)**: pack-eu Postgres cluster EU-resident; cross-region replication forbidden by default.
- **NIS2**: incident-response timelines (24h+72h+1mo) when oyatie crosses thresholds.
- **eIDAS 910/2014**: audit-chain Ed25519 seals are AdES.

### pack-jp (APPI)

- **APPI Art. 17 (purpose)**: declared at tenant onboarding.
- **APPI Art. 21 (cross-border)**: pack-jp JP-resident.
- **APPI Art. 23 (joint use)**: tenant-of-tenant disclosure obligation.
- **APPI Art. 27 (cross-border consent)**: explicit consent on tenant onboarding for cross-pack.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/calendar-overlay.md`.

## Re-review Triggers

- Any change to dual-context isolation invariant.
- Any RFC 5545 / RFC 4791 / RFC 5546 spec revision.
- Any new pack activation.
- Quarterly scheduled.
- Post-incident.
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519).
- ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140.
- `microservices/calendar/PRD.md`, `dpia.md`, `compliance.md`, `policy/*.cedar`.
- RFC 5545 (iCalendar), RFC 5546 (iTIP), RFC 4791 (CalDAV).
- libical conformance corpus.
- Microsoft Threat Modeling (STRIDE), LINDDUN privacy.
- NIST SP 800-154.
