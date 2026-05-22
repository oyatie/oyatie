---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-calendar
microservice: calendar
status: Accepted
sales_segment: shared-substrate + suite-app
tier: tenant-facing
milestone_first_ship: M02-product-tier-foundation
bominal_source: [ADR-0208-connect-dual-context-unified-channel-hub, ADR-0215-connect-retention-legal-hold-dual-context]
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145), ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/microservices/calendar.json, /specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-calendar
doc_status: published
---

# PRD-calendar: Calendar + Scheduling µservice

## Purpose

The `calendar` µservice is oyatie's native calendar, meeting-scheduling, invitation, cross-tenant availability, and room-booking substrate. Per ADR-0132 (product-suite + bundle dissolution) and parallel-session ADR-0135 (Connect unbundle), calendar is a standalone tenant-facing µservice — no longer part of a Connect suite — owning: event/meeting scheduling; invitations (RFC 5545 iCalendar + CalDAV compatible); recurring events; cross-tenant availability lookups; meeting rooms / resources; time-zone handling (ICU/IANA tzdata); .ics import/export.

The µservice carries dual-context (Personal / Professional) per parallel ADR-0238; details never cross context boundaries except via explicit invitation or policy-bound projection.

Bominal inheritance: ADR-0208 dual-context unified-channel hub + ADR-0215 retention + legal-hold overlays are inherited 1:1 per `feedback_bominal_inheritance_precedence.md`; oyatie additions captured below.

## Tenant Value

- **Tenant Outcome 1 — Scheduling without third-party dependency.** Tenants do not need Google Calendar / Outlook / Calendly / Cal.com / Fantastical accounts; the µservice is a native first-party scheduling substrate.
- **Tenant Outcome 2 — Cross-tenant availability with policy-bounded disclosure.** Two tenants who explicitly opt-in via invitation flow share only the minimum-necessary free/busy projection; details never leak across tenants without explicit grant.
- **Tenant Outcome 3 — Meeting-room and resource scheduling.** Per-tenant resource graph (rooms, devices, vehicles); recurring booking; conflict resolution.
- **Tenant Outcome 4 — Recurring events at scale.** Bounded recurrence-expansion (RFC 5545 RRULE/EXDATE/RDATE); no unbounded materialisation.
- **Tenant Outcome 5 — iCalendar + CalDAV import/export.** Cleanly migrate from / coexist with Google / Outlook / Apple Calendar via CalDAV adapter (read + write) and .ics import/export.
- **Internal Outcome 6 — Dual-context separation.** Personal vs Professional events isolated at the data-class + Cedar-policy boundary; cross-context inference is structurally impossible.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant operator | to create an event with title, start/end, attendees, location, time-zone | meetings can be scheduled | event-store | Must |
| FR-02 | event author | to attach an RFC 5545 RRULE for recurrence | weekly / monthly / custom recurrence works | recurrence-engine | Must |
| FR-03 | scheduler | to query free/busy across attendees in <500ms p99 | I can pick a slot interactively | availability-resolver | Must |
| FR-04 | scheduler | to book a meeting room from the tenant resource graph | room conflicts are resolved at booking time | room-booking | Must |
| FR-05 | event organiser | to send RFC 5545 ITIP invitations to internal + external attendees | invitations work across calendar systems | invitation-flow | Must |
| FR-06 | external invitee | to accept / decline / counter-propose via RFC 5546 reply | attendance state stays in sync | invitation-flow | Must |
| FR-07 | tenant operator | to import a .ics file (RFC 5545) | migration from another calendar is one-shot | ics-import-export | Must |
| FR-08 | tenant operator | to export a calendar to .ics for backup / portability | data-portability (GDPR Art. 20) honoured | ics-import-export | Must |
| FR-09 | CalDAV client | to read / write events over CalDAV (RFC 4791) | Apple Calendar / Thunderbird / native clients work | ics-import-export (CalDAV adapter) | Must |
| FR-10 | cross-tenant scheduler | to query free/busy of an attendee in a different tenant (opted-in) | external scheduling works without raw-event leak | availability-resolver | Must |
| FR-11 | event author | to attach time-zone metadata (IANA tz, e.g., Asia/Seoul) | DST + jurisdiction rules apply correctly | event-store + recurrence-engine | Must |
| FR-12 | event author | to delegate event ownership / chair | meeting governance works | event-store | Should |
| FR-13 | tenant compliance officer | to put a professional event under legal hold | event + attendee history + invitations preserved past retention | event-store | Must |
| FR-14 | tenant operator | to receive a webhook on event-state change | downstream Workflow can react | (cross-cutting) | Must |
| FR-15 | analytics consumer | to receive a per-tenant utilisation aggregate (room occupancy %, busy %) | capacity planning is data-driven | (cross-cutting, anonymised) | Should |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Single event fetch | ≤30ms | ≤200ms | ≤500ms | Postgres read; cache-hit ratio >80% |
| Cross-tenant availability lookup | ≤100ms | ≤500ms | ≤1s | up to 100 attendees; Valkey cache |
| Recurrence expansion (single RRULE; 1y horizon) | ≤200ms | ≤1s | ≤3s | bounded window |
| Event write (with invitation fanout) | ≤80ms | ≤300ms | ≤800ms | sync write; async fanout |
| .ics import (10k events) | — | ≤60s | ≤120s | streaming parse |
| .ics export (10k events) | — | ≤30s | ≤60s | streaming write |
| Room conflict check | ≤20ms | ≤100ms | ≤300ms | resource-graph index |
| CalDAV PROPFIND (calendar collection) | ≤80ms | ≤400ms | ≤1s | per RFC 4791 |

### Security

- All event payloads encrypted-at-rest under tenant-DEK (per Bominal ADR-0111 envelope encryption) when in Professional context; Personal context uses E2E where the tenant has declared E2E.
- Cross-tenant availability lookups return only free/busy projection; raw event fields (title, description, attendees, location) NEVER cross tenant boundary without explicit invitation accept.
- All CalDAV endpoints are mTLS + per-tenant API key + RBAC.
- All .ics imports are parsed by a hardened RFC 5545 parser with input bounds (max events / file ≤ 100k; max recurrence horizon ≤ 5y); reject malformed input rather than auto-repair.

### Audit + Compliance

- Every `EventCreated / EventUpdated / EventCanceled / InvitationAcceptedDeclined / RoomBooked` emits an audit-chain record (Merkle + Ed25519 per Bominal ADR-0028).
- Legal-hold preserves event + attendee state + invitation chain past retention expiry.
- Per-jurisdiction retention (KR PIPA / EU GDPR / US sector-specific) computed per ADR-0140 Cedar pack overlay.

### Availability + SLO

- Availability target: 99.95 % monthly for event-read path; 99.9 % for write path.
- RTO ≤ 15 min; RPO ≤ 60 s (Postgres logical replication).
- Cross-tenant availability resolver: degrades to "unknown — please contact attendee" rather than block on remote-tenant outage.

### Data residency

- Tenant data pinned to the tenant's region per ADR-0117 + ADR-0140; cross-region replication forbidden by default; SCC-gated when activated.

### DR posture (per ADR-0343)

- Manifest target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, `replication_shape=active-active-multi-az-cross-region-warm`. The older 15-minute/60-second objective remains a stretch SLO, not the ADR-0343 manifest contract.
- Applicable pack floors from `specs/compliance-pack-floors.json`: HIPAA-2024 `3600s/300s` with multi-region required; KR-PIPA-2023 default `14400s/900s`; SOC2-T2 `14400s/900s`; ISO27001-2022 `14400s/3600s`; KR-CSAP-v3.1 `3600s/900s` with multi-region required. The effective maximum pack floor is ISO27001 `14400s/3600s`; calendar keeps the stricter scheduling target.
- `failover_runbook=runbooks/dr-failover.md`, resolved at `microservices/calendar/runbooks/dr-failover.md`; backup substrates are `postgres_wal_g`, `valkey`, `object_storage_versioned`, and `audit_chain_merkle_seal`.
- `multi_region_active_active=true` only inside activated pack-pinned region pairs; default data-residency still forbids unconstrained cross-region replication.
- Why: calendar failover is tenant-visible as missed meetings, healthcare appointments, interviews, and room conflicts; a restored but stale schedule would create operational and compliance harm.

### Capacity model (per ADR-0340)

- Per-tenant baseline: `0.08 vCPU`, `192 MiB RAM`, `3 GiB storage`, `connections_per_tenant={valkey:3, postgres:3, outbound_http:4}`.
- Scaling dimension: `per_request` for availability, free/busy, recurrence, room booking, RSVP fanout, CalDAV/ICS sync, and tzdb refresh.
- Cell placement class: `Tier-3` with manifest `pod_runtime_tier=2`; calendar scales with schedule queries and event mutations rather than tenant seat count alone.
- Autoscaling boundaries: min `3` api/availability replicas per tenant-cell, max `48` before event-store shard split; recurrence and import workers are separately capped to prevent large .ics imports from starving free/busy checks.
- Why: calendar workloads combine latency-sensitive free/busy queries with bursty imports and invitation storms; the model preserves live scheduling while bounding batch work.

### Sustainability + cost attribution (per ADR-0344)

- Every event write, free/busy query, recurrence expansion, room-booking decision, invitation, CalDAV call, and .ics import/export audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider routing is carbon-aware for large .ics import/export jobs, tzdb refresh, recurrence materialization backfill, and room-utilization analytics; it is not carbon-routed for live free/busy, invitation acceptance, HIPAA emergency scheduling, or legal-hold operations.
- Tenant cost transparency surface: calendar admin shows scheduling volume, free/busy query load, room/resource utilization cost, import/export job cost, and CalDAV bridge cost; finops-portal handles tenant and compliance-pack rollups.
- Why: scheduling appears lightweight but can produce heavy recurrence and availability compute, so CSRD, SB-253, and SEC climate-disclosure reporting needs the cost/emission trail tied to calendar capabilities.

### API versioning posture (per ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet using `Oyatie-Version` header, `/v/YYYY-MM-DD/` REST/CalDAV extension prefix, and proto3 field `string oyatie_version = 8001` for public events/contracts.
- SDK semver model: calendar SDKs publish `major.minor.patch`; RFC 5545/4791 compatibility is preserved while oyatie-specific behavior is governed by date carrier.
- Support window: last `N=3` public versions for at least `180` days after deprecation.
- Per-tenant pinning: yes for regulated tenants, CalDAV bridge migrations, scheduling partners, and embedded resource-booking clients.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC over HTTP/3 remains tag-compatible and exempt from public carrier routing.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates). Six primary BCs.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `event-store` | `oya-calendar-event-store-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Event persistence; attendee state; legal hold; tenant-DEK encryption | `CalendarEvent`, `Attendee`, `RetentionPolicyRef`, `LegalHoldRef` |
| `recurrence-engine` | `oya-calendar-recurrence-engine-{kernel,domain,usecase,api,adapter,app}` | RFC 5545 RRULE / EXDATE / RDATE expansion; bounded materialisation | `RecurrenceRule`, `OccurrenceWindow`, `ExpandedOccurrence` |
| `availability-resolver` | `oya-calendar-availability-resolver-{kernel,domain,usecase,api,adapter,adapter-valkey,rest,worker,app}` | Free/busy projection; cross-tenant resolver; cache | `FreeBusyProjection`, `AvailabilityWindow`, `CrossTenantInviteGrant` |
| `room-booking` | `oya-calendar-room-booking-{kernel,domain,usecase,api,adapter,rest,app}` | Resource graph; conflict resolution; recurring booking | `Resource`, `Booking`, `ConflictDecision` |
| `invitation-flow` | `oya-calendar-invitation-flow-{kernel,domain,usecase,api,adapter,worker,app}` | RFC 5545 ITIP / RFC 5546 reply flow; external delivery via mail µservice | `Invitation`, `RsvpState`, `CounterProposal` |
| `ics-import-export` | `oya-calendar-ics-import-export-{kernel,domain,usecase,api,adapter,adapter-icalendar,adapter-caldav,rest,app}` | RFC 5545 .ics parse/emit; RFC 4791 CalDAV adapter | `IcsDocument`, `CalDavCollection`, `ImportJob`, `ExportJob` |

Naming justification (one of six; same shape applies to others) — `event-store`:

```
NAME: oya-calendar-event-store-<layer>
JUSTIFICATION:
- microservice = calendar: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. No shared|vertical bisection.
- bc-tokens = event-store: primary BC for event persistence; siblings (recurrence-engine,
  availability-resolver, room-booking, invitation-flow, ics-import-export) justify
  explicit BC token per ADR-0056 v4.1 BC-optionality rule.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (CalendarEvent, Attendee, RetentionPolicyRef,
    LegalHoldRef, EventContext{Personal|Professional}). Zero I/O. data_class annotations.
  - domain: pure event-invariant math (overlap, ordering, time-zone arithmetic, hold
    coverage).
  - usecase (per ADR-0106): orchestrators (create-event, update-event, cancel-event,
    apply-legal-hold, expire-retention) reading via ports.
  - api: protocol-neutral typed contracts.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-postgres: backend-qualified adapter (per ADR-0105 Amendment 3
    *-adapter-<backend> pattern); implements EventRepository against Postgres with RLS.
  - rest: HTTP handler/route layer.
  - worker: long-lived background workers (retention sweep, hold cascade).
  - sdk: client library for tenants + workflow consumers.
  - app: composition root binary.
- exemptions claimed: none.
```

(Equivalent justifications recorded for the other five BCs at `microservices/calendar/specs/naming-justification.md`.)

Layer mapping table per BC (13-layer enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-valkey | adapter-icalendar | adapter-caldav | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `event-store` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | ✓ | ✓ |
| `recurrence-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | ✓ |
| `availability-resolver` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | ✓ | ✓ | — | ✓ |
| `room-booking` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | ✓ | — | — | ✓ |
| `invitation-flow` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | — | ✓ |
| `ics-import-export` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | ✓ | — | — | ✓ |

Total crates introduced by this µservice: **44**.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `EventRepository` | `oya-calendar-event-store-kernel` | `-adapter-postgres` | `PERSONAL_EVENT_CONTENT` + `PROFESSIONAL_EVENT_CONTENT` (per-context envelope encryption) |
| `RecurrenceExpander` | `oya-calendar-recurrence-engine-kernel` | `-adapter` (rrule-rs based) | `INTERNAL_ONLY` |
| `FreeBusyProjector` | `oya-calendar-availability-resolver-kernel` | `-adapter-valkey` | `BEHAVIORAL_TENANT_PRODUCT` (free/busy projection only) |
| `CrossTenantInviteResolver` | `oya-calendar-availability-resolver-kernel` | `-adapter` | `SENSITIVE_PIPA_ART23` (tenant identifier mapping) |
| `ResourceRepository` | `oya-calendar-room-booking-kernel` | `-adapter-postgres` (subsumed under event-store -adapter-postgres crate) | `BEHAVIORAL_TENANT_PRODUCT` |
| `InvitationDispatcher` | `oya-calendar-invitation-flow-kernel` | `-adapter` (delegates to `mail` µservice via Workflow) | `PII_IDENTIFYING` (attendee email) |
| `IcsParser` / `IcsEmitter` | `oya-calendar-ics-import-export-kernel` | `-adapter-icalendar` | `PERSONAL_EVENT_CONTENT` + `PROFESSIONAL_EVENT_CONTENT` |
| `CalDavBackend` | `oya-calendar-ics-import-export-kernel` | `-adapter-caldav` | `PERSONAL_EVENT_CONTENT` + `PROFESSIONAL_EVENT_CONTENT` |
| `TimeZoneResolver` | `oya-calendar-event-store-kernel` | `-adapter` (chrono-tz / ICU) | `INTERNAL_ONLY` |
| `RetentionPolicyResolver` | `oya-calendar-event-store-kernel` | `-adapter` (resolves to `tenancy` µservice via Workflow) | `AUDIT` |
| `LegalHoldStore` | `oya-calendar-event-store-kernel` | `-adapter-postgres` | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `calendar` MUST NOT import another product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). Consumed µservices: `tenancy` (tenant + identity resolution), `audit-chain` (seal emission), `mail` (invitation delivery), `messenger` (room-channel binding for event-bound chat), `observability` (telemetry). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice calendar`
- `oya gate validate lean-a2 --microservice calendar`
- `oya gate validate port-location --microservice calendar`
- `oya gate validate layer-correctness --microservice calendar`
- `oya gate validate per-microservice-layout --microservice calendar`
- `oya gate validate statelessness --microservice calendar`
- `oya gate validate shardability --microservice calendar`
- `oya gate validate hyperscaler-maturity --microservice calendar`
- `oya gate validate rfc-5545-conformance --microservice calendar` (NEW)
- `oya gate validate rfc-4791-conformance --microservice calendar` (NEW; CalDAV)

## Integration via Workflow + Ontology

### Workflow events produced

| Event | Topic | Trigger | Consumed by | Idempotency key |
|---|---|---|---|---|
| `EventCreated` | `calendar.event.lifecycle.v1` | new event written | mail (invitation), audit-chain, workflow-engine (triggers) | `event_id` |
| `EventUpdated` | `calendar.event.lifecycle.v1` | event mutation | mail (update invitations), audit-chain | `event_id + version` |
| `EventCanceled` | `calendar.event.lifecycle.v1` | cancellation | mail (cancellation), audit-chain | `event_id + canceled_at` |
| `InvitationAcceptedDeclined` | `calendar.invitation.rsvp.v1` | RSVP received | event-store, audit-chain | `invitation_id + state` |
| `RoomBooked` | `calendar.room.booking.v1` | room reserved | audit-chain, observability | `booking_id` |
| `RoomBookingConflict` | `calendar.room.booking.v1` | conflict detected | observability, requester | `booking_id` |
| `RecurrenceWindowExpanded` | `calendar.recurrence.v1` | bounded expansion done | observability | `event_id + window_hash` |
| `LegalHoldApplied` / `LegalHoldReleased` | `audit.calendar.legal_hold.v1` | hold transition | audit-chain, governance | `event_id + hold_id` |

### Workflow events consumed

| Event | Producer | Handler | Action |
|---|---|---|---|
| `TenantOnboarded` | `tenancy` | event-store usecase | provision tenant-DEK; create default resource collection |
| `TenantOffboarded` | `tenancy` | event-store usecase | mark events for retention sweep / legal-hold scan |
| `MailDeliveryFailed` | `mail` | invitation-flow usecase | retry / surface "delivery-failed" recipient card |
| `MessengerRoomCreated` | `messenger` | event-store usecase | bind room-channel to event when requested |
| `WorkflowTrigger` | `workflow-engine` | event-store usecase | event-bound automation (e.g., "auto-cancel if RSVPs < 3") |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit |
|---|---|---|---|
| `CalendarEvent{event_id, tenant, context, starts_at, ends_at, ...}` | `events→Tenant`, `events→User(attendee)` | `event-store` | Ed25519 |
| `Resource{resource_id, kind, location, tenant}` | `resource_of→Tenant` | `room-booking` | Ed25519 |
| `Booking{booking_id, resource_id, event_id, period}` | `books→Resource`, `books→CalendarEvent` | `room-booking` | Ed25519 |
| `Invitation{invitation_id, event_id, recipient_ref, state}` | `invites→CalendarEvent`, `invites→User` | `invitation-flow` | Ed25519 |
| `LegalHold{hold_id, event_id, opened_by, opened_at}` | `holds→CalendarEvent` | `event-store` | Ed25519 |

### Ontology reads

| Object | Read by | Query shape |
|---|---|---|
| `User` (tenant directory) | `event-store`, `invitation-flow`, `availability-resolver` | by `(tenant_id, user_id)` |
| `Tenant` | `event-store`, `availability-resolver` (cross-tenant) | by `tenant_id` |
| `RetentionPolicy` | `event-store` | by `(tenant_id, pack)` |

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| Google Calendar | Workspace Calendar | recurrence; room booking; cross-org invites; .ics; CalDAV | `developers.google.com/calendar` |
| Microsoft Outlook Calendar | Microsoft 365 Calendar | recurrence; resource booking; ITIP invites; Exchange ActiveSync | `learn.microsoft.com/graph/api/resources/event` |
| Apple Calendar | iCloud Calendar | CalDAV; .ics; native time-zone | `developer.apple.com/documentation/eventkit` |
| Calendly | Scheduling links | external scheduling; round-robin; availability windows | `developer.calendly.com` |
| Cal.com | Open-source Calendly | self-hosted; webhook; teams | `cal.com/docs/api-reference` |
| Fantastical | Mac/iOS calendar app | natural-language event input; weather; conferencing | (consumer app; no public API) |
| Naver Works Calendar | Korean enterprise | KR-locale; KR holidays; group calendars | `developers.worksmobile.com/kr` |

Key parity gaps to close (ordered):

1. **Cross-tenant availability with policy-bounded disclosure** — none of the competitors gate cross-org free/busy with Cedar-policy + audit-chain. **Differentiator.**
2. **Dual-context (Personal / Professional) isolation enforced structurally** — no competitor enforces context-separation in code; tenant-policy only. **Differentiator.**
3. **CalDAV (RFC 4791) read+write parity** — Apple/Google parity required for Apple/Thunderbird/Evolution clients.
4. **Naver Works KR-locale + Korean holidays** — pack-kr launch gate.
5. **RFC 5545 RRULE conformance** — must pass the [RFC 5545 test corpus](https://github.com/libical/libical/tree/master/test-data).
6. **.ics import / export at scale (≥10k events)** — Google parity; Calendly does not support .ics import.

## Performance Targets (canonical bench surface)

| Metric | Target | Verification |
|---|---|---|
| Event-fetch p99 | ≤ 200ms | `cargo bench -p oya-calendar-event-store-adapter-postgres -- event_fetch` |
| Cross-tenant availability p99 | ≤ 500ms | `cargo bench -p oya-calendar-availability-resolver-usecase -- cross_tenant` |
| Recurrence expansion p99 (1y horizon, complex RRULE) | ≤ 1s | `cargo bench -p oya-calendar-recurrence-engine-domain -- rrule_expand` |
| Event-write p99 | ≤ 300ms | `cargo bench -p oya-calendar-event-store-usecase -- write` |
| .ics import (10k events) p99 | ≤ 60s | `cargo bench -p oya-calendar-ics-import-export-adapter-icalendar -- import_10k` |
| Room conflict-check p99 | ≤ 100ms | `cargo bench -p oya-calendar-room-booking-usecase -- conflict_check` |

Error budget: monthly 99.95% availability → ~22 min/month.

## Horizontal Scalability

State strategy (per Bominal ADR-0019): `mixed`. Postgres (event-store; per-tenant RLS); Valkey (availability-resolver cache; per-tenant key prefix); stateless workers for invitation-fanout + retention-sweep + recurrence-expansion + ics-import.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active tenant calendars | 100k | 1M | Postgres connection pool > 70% |
| Events/s write | 1k | 10k | event-store-rest p99 > 200ms |
| Cross-tenant availability lookup/s | 5k | 50k | availability-resolver-rest p99 > 400ms |
| Recurrence expansion/s | 100 | 1k | recurrence worker queue depth > 60s of cadence |
| Active CalDAV sessions | 10k | 100k | rest pod CPU > 70% |

Scale-out policy:
- Kubernetes HPA: rest pods scale on CPU > 70%; min 3, max 100.
- Postgres: per-tenant logical shard; cross-cell replication-factor 3 with Patroni.
- Valkey: cluster mode; per-tenant key prefix; eviction policy `allkeys-lru` for free/busy cache.
- Pre-warmed pool: 5 standby pods; cold-start ≤ 700ms.

Cross-region: M02 launches in KR (ap-seoul-1); M03 expands to EU + US per ADR-0117 jurisdiction pack.

Sharding: events partitioned by `(tenant_id, starts_at_year_month)`; resources partitioned by `tenant_id`; invitations partitioned by `event_id`.

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Event create with attendees + invitation fanout completes within p99 ≤ 300ms | `cargo bench` |
| AC-02 | Cross-tenant availability lookup returns only free/busy projection (no titles / attendees / locations) | `cargo nextest -p oya-calendar-availability-resolver-domain -- cross_tenant_minimum_necessary` |
| AC-03 | RFC 5545 RRULE conformance test corpus 100% pass | `cargo nextest -p oya-calendar-recurrence-engine-domain -- rfc_5545_corpus` |
| AC-04 | RFC 4791 CalDAV PROPFIND / REPORT / PUT / DELETE end-to-end against Apple Calendar + Thunderbird + Evolution | E2E test suite `tests/e2e/caldav-clients.rs` |
| AC-05 | .ics import of 10k events completes within p99 ≤ 60s without parse errors | `cargo bench -p oya-calendar-ics-import-export-adapter-icalendar` |
| AC-06 | Legal-hold preserves event + attendee history + invitation chain past retention expiry | `cargo nextest -p oya-calendar-event-store-domain -- legal_hold` |
| AC-07 | Personal-context details NEVER appear in Professional-context availability queries | `cargo nextest -p oya-calendar-availability-resolver-domain -- context_isolation` |
| AC-08 | Tenant-DEK envelope encryption applied to Professional event content; verified at rest | `tests/e2e/encryption-at-rest.rs` |
| AC-09 | Room conflict detection prevents double-booking at write time | `cargo nextest -p oya-calendar-room-booking-domain -- conflict` |
| AC-10 | Recurrence expansion bounded; > 5y horizon rejected | `cargo nextest -p oya-calendar-recurrence-engine-domain -- unbounded_rejection` |
| AC-11 | Audit-chain seal emitted for every event lifecycle + invitation RSVP + room booking | `cargo nextest -p oya-calendar-event-store-app -- audit_chain_emission` |
| AC-12 | `oya gate validate per-microservice-layout --microservice calendar` exit 0 | ADR-0131 lane |

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | Should we ship native conferencing (oyatie-Connect-Conference) bound to events, or rely on Workflow trigger to external Zoom/Meet? | council-product | subsequent-to-M03-completion |
| 2 | Time-zone source-of-truth: chrono-tz (Rust) vs ICU4X — chrono-tz chosen for now; revisit if ICU4X offers better calendar-locale coverage | axis-calendar | ADR successor-IP |
| 3 | Calendar federation (Google/Outlook source-of-truth coexistence mode vs migration-only) — current scope is migration via CalDAV only | council-product | subsequent-to-M03-completion |
| 4 | RRULE BYSETPOS interaction with EXDATE — defer to RFC 5545 corpus | axis-calendar | resolved by corpus pass |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application→usecase | layer rename |
| ADR-0117 | Cloud-native infrastructure | data residency |
| ADR-0135 | Connect unbundle (parallel session) | dual-context inheritance |
| ADR-0139 | Agentic SLO-gated promotion | gate authority |
| ADR-0131 | Per-microservice flat layout | layout authority |
| ADR-0132 | Product-suite + bundle dissolution | µservice independence |
| ADR-0133 | Industry-best-practice conformance | hyperscaler-grade bar |
| ADR-0140 | Cedar policy enforcement | policy substrate |
| Bominal ADR-0208 | Connect dual-context unified-channel hub | inherited 1:1 |
| Bominal ADR-0215 | Connect retention + legal-hold dual-context | inherited 1:1 |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `calendar` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `calendar` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 2 module pin(s) across 1 context(s).
- Scaling input: `per_request` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
