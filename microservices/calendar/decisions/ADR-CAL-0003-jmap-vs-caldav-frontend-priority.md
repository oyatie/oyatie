---
id: ADR-CAL-0003
status: Accepted
date: 2026-05-17
microservice: calendar
deciders: axis-calendar, council-architecture, gtm-customer-success
owner: axis-calendar + gtm-customer-success
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-CAL-0001
  - ADR-MAIL-0003
related_artifacts:
  - microservices/calendar/PRD.md (FR-09; §Competitive Benchmark)
  - microservices/calendar/sdk-plan.md
  - microservices/calendar/decisions/ADR-CAL-0001-caldav-server-backend-selection.md
purpose: |
  Decide which calendar wire protocol ships first at M03. Two protocols
  serve the same set of clients but at different maturity levels: CalDAV
  (RFC 4791, stable since 2007, supported by every desktop+mobile
  calendar client) vs JMAP Calendars (draft-ietf-jmap-calendars,
  IETF-WG-draft, supported only by Fastmail today). This ADR mirrors
  ADR-MAIL-0003 (SDK launch order) on the calendar side.
---

# ADR-CAL-0003: CalDAV (RFC 4791) ships first at M03; JMAP Calendars (draft-ietf-jmap-calendars) ships M04 once IETF draft stabilises

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The calendar µservice has two reasonable wire protocols to offer
external clients:

1. **CalDAV (RFC 4791, 2007)** — XML-over-WebDAV calendar protocol;
   universally supported by Apple Calendar, Mozilla Thunderbird/
   Lightning, GNOME Evolution, KDE KOrganizer, DAVx5 (Android), iOS
   Calendar app, macOS Calendar app, every enterprise CalDAV gateway.
   Stable RFC. Hyrum's-Law-bound to a fault — clients have been
   depending on its exact semantics for ~20 years.
2. **JMAP Calendars (draft-ietf-jmap-calendars)** — JSON-over-HTTP
   modern calendar protocol; IETF WG draft (latest revision -16 as of
   2026-Q1, not yet RFC). Supported in production today only by
   Fastmail (which authored the spec). Apple has signaled M5+ interest
   per WWDC 2024 sessions; nothing else implements it. Pairs with JMAP
   Mail (RFC 8620 + RFC 8621) for unified mail+calendar client
   experiences.

PRD-calendar FR-09 makes CalDAV a "Must"; the same PRD §Competitive
Benchmark row notes Apple/Google parity requires CalDAV. JMAP Calendars
is not listed in FR-* as a Must — but ADR-MAIL-0003 (SDK launch order)
ships JMAP Mail first for Swift + TypeScript; this ADR resolves whether
calendar mirrors that decision.

The sibling ADR-MAIL-0003 chose **JMAP-first for Swift, JMAP-jam wrapper
for TypeScript, IMAP4rev2 as fallback after JMAP feature-parity**. This
suggests calendar should consider mirroring with JMAP Calendars first.
However, three asymmetries argue against straight mirroring:

- **JMAP Calendars is a DRAFT, not an RFC.** Shipping a draft as a
  primary frontend creates breaking-change risk on every IETF revision.
- **CalDAV client market is ~100×** the JMAP Calendars client market.
  Apple Calendar + Thunderbird + DAVx5 are mainstream; JMAP Calendars
  has only Fastmail's first-party clients today.
- **`rrule-rs` recurrence engine semantics (per ADR-CAL-0002)** were
  selected for RFC 5545 conformance, which both protocols consume —
  so the engine choice is protocol-independent.

PRD AC-04 makes CalDAV end-to-end interop a release gate; there is no
equivalent JMAP Calendars gate in the M03 PRD.

## Decision

The calendar µservice ships **CalDAV (RFC 4791) as the primary
external wire protocol at M03**, with JMAP Calendars deferred to **M04
or later, contingent on the IETF draft reaching RFC status**.

Concrete bindings:

1. **M03 ship surface**:
   - REST facade (per existing `contracts/openapi/calendar.yaml`).
   - gRPC (per existing `contracts/proto/calendar.proto`).
   - **CalDAV (RFC 4791)** via the `oya-calendar-ics-import-export-
     adapter-caldav-radicale` adapter (per ADR-CAL-0001).
   - .ics import/export per RFC 5545 (existing `runbooks/ics-import-
     failure.md`).
   - Workflow events (per existing `contracts/asyncapi/calendar-
     events.yaml`).

2. **M04 (or later) ship surface adds**:
   - **JMAP Calendars** (then-current IETF status; promoted to RFC by
     M04 or shipped with a "draft-N" compatibility disclaimer).
   - Implementation crate: `oya-calendar-ics-import-export-adapter-
     jmap` (no `-backend` qualifier — JMAP is the protocol itself, not
     a backend choice).

3. **SDK launch order** (mirrors ADR-MAIL-0003 where applicable):
   - **Rust SDK** at M03 — first-party authored `oya-calendar-<bc>-sdk`
     crates per BC; CalDAV + REST + gRPC client.
   - **TypeScript SDK** at M03 — OpenAPI-generated baseline + first-
     party CalDAV client wrapper; published to npm.
   - **Python SDK** at M03+1 — OpenAPI-generated; published to PyPI;
     pairs with `caldav` reference lib.
   - **Swift SDK** at M03+1 — native EventKit-shaped wrapper over our
     CalDAV; eventual JMAP Calendars switch when M04 ships.
   - **Go / JVM / C# SDKs** at M04+ — gRPC baseline + ergonomic
     wrappers.

4. **Tenant-visible surface table**:

   | Client | M03 protocol | M04+ protocol |
   |---|---|---|
   | Apple Calendar / iCloud | CalDAV | CalDAV (no change; JMAP Cal optional) |
   | Mozilla Thunderbird / Lightning | CalDAV | CalDAV (no change) |
   | GNOME Evolution / KDE KOrganizer | CalDAV | CalDAV |
   | DAVx5 (Android) | CalDAV | CalDAV |
   | iOS Calendar app | CalDAV | CalDAV (no change) |
   | Outlook desktop (.ics import) | .ics import | .ics import + opt-in CalDAV |
   | Fastmail-style integrations | (none) | JMAP Calendars |
   | Custom oyatie web client | REST + gRPC | REST + gRPC + JMAP Calendars |

5. **CalDAV adapter SLOs** are wired in `slos/caldav-availability.
   openslo.yaml` (per the OpenSLO manifests added in this same
   ChangeSet); JMAP Calendars SLOs ship in M04.

## Alternatives Considered

### A. JMAP Calendars first; CalDAV second

- **Pros**:
  - Mirrors ADR-MAIL-0003 (JMAP Mail first) symmetrically.
  - Modern protocol — JSON over HTTP, single-round-trip for many ops,
    push notifications via JMAP Event Source.
  - Pairs cleanly with the JMAP Mail SDK story.
- **Cons**:
  - JMAP Calendars is an IETF draft, not an RFC. Breaking changes
    arrive on draft revisions; consumers depending on a specific
    draft version will break.
  - Client population is ~Fastmail-only today; the protocol-population
    asymmetry is roughly 100:1 in CalDAV's favour.
  - PRD FR-09 + AC-04 already make CalDAV a release gate; deferring
    CalDAV to M04 would break the release gate.
- **Rejected** because shipping a draft as the primary frontend
  violates the no-silent-regression principle (every draft revision
  is a silent regression for consumers).

### B. CalDAV + JMAP Calendars simultaneously at M03

- **Pros**:
  - Maximum client coverage on day one.
  - No M04 follow-up effort.
- **Cons**:
  - Doubles the M03 surface area; doubles the conformance test matrix
    (both protocols against the same RRULE engine).
  - JMAP Calendars draft instability means M03 ships a known-to-
    change interface; sunset cost amplifies the cost-of-removal under
    the deprecation-and-migration skill.
  - HG-CALENDAR gate (per ADR-0123) demands SLO history before
    promotion; doubling the SLO surface at M03 likely fails the gate
    on coverage.
- **Rejected** because the marginal client coverage from JMAP at M03
  is tiny (Fastmail's clients), while the architectural cost
  (instability) is large.

### C. CalDAV only; never JMAP Calendars

- **Pros**:
  - Smallest possible long-term surface area.
  - Aligns with PRD FR-* where JMAP Calendars is not a Must.
- **Cons**:
  - Cedes the JMAP-Mail-paired modern client integration to Fastmail
    and (eventually) Apple.
  - Forfeits the architectural symmetry with ADR-MAIL-0003.
  - Closes a planned M04 enhancement preemptively.
- **Rejected** — JMAP Calendars is the most plausible future protocol;
  closing the door is wrong.

### D. CalDAV at M03; JMAP Calendars at M04 once IETF draft stabilises  ← **CHOSEN**

- **Pros**:
  - PRD release gate (AC-04) met at M03 with the universally-supported
    CalDAV.
  - JMAP Calendars deferred until draft becomes RFC (or settles enough
    for a stable ship); avoids no-silent-regression violation.
  - Mirrors ADR-MAIL-0003 staging (JMAP first, IMAP fallback) at a
    protocol cadence appropriate to each surface's maturity.
  - One protocol per ship-cycle keeps SLO + conformance + interop
    surfaces bounded.
- **Cons**:
  - M03 ships without the SDK symmetry of mail (which gets JMAP-first);
    Swift SDK at M03+1 will explicitly note the CalDAV-only posture
    until M04.
  - M04 effort is non-trivial (~3 engineer-months for the JMAP
    Calendars adapter + SDK wrappers).
- **Accepted** — meets the release gate; defers the experimental
  protocol; preserves the architectural symmetry as a staged rollout.

## Consequences

### Positive

- **AC-04 (CalDAV end-to-end interop) met at M03 by construction.**
  CalDAV against Apple Calendar / Thunderbird / Evolution / DAVx5
  passes the public CalDAV interop matrix.
- **No silent regression on draft revisions.** JMAP Calendars is
  deferred until the draft stabilises, so consumers depending on a
  draft-N revision don't face breaking changes on each IETF revision.
- **Bounded ship surface.** One protocol per milestone is the right
  cadence for SLO + conformance + interop coverage.
- **Symmetry with ADR-MAIL-0003 preserved as a staged rollout.** The
  calendar SDK launch order (Rust + TS at M03; Python + Swift at M03+1)
  mirrors mail's pattern with one milestone of stagger.

### Negative

- **No JMAP Calendars at M03.** Fastmail-style integrations + the
  "modern protocol" demo story are absent at M03; gtm-customer-success
  notes this in the M03 pitch.
- **M04 JMAP Calendars effort is non-trivial.** ~3 engineer-months for
  the adapter + SDK wrappers; ~1 engineer-month for the OpenSLO
  manifests + dashboards.
- **Swift SDK lacks EventKit-JMAP path at M03+1.** Swift SDK ships a
  CalDAV-wrapping EventKit-shape; the JMAP Calendars Swift path comes
  with M04. Documented in `sdk-plan.md`.

### Operational

- **CalDAV CI lane** `oya-governance-rfc-4791-conformance` BLOCKER
  from M03 (per PRD §"CI lanes that must green").
- **JMAP Calendars CI lane** `oya-governance-jmap-calendars-draft-
  conformance` shipped as REPORT-ONLY at M03 (the lane exists; it
  validates against the current draft); promoted to BLOCKER at M04.
- **Per-protocol SLOs** in `slos/caldav-availability.openslo.yaml`
  (this ChangeSet); `slos/jmap-calendars-availability.openslo.yaml`
  added at M04.
- **Sunset for the M03 "CalDAV-only" SDK posture**: when M04 ships,
  the M03 SDKs get a minor-version bump that adds JMAP Calendars
  support as an opt-in client; no breaking change.

### Regulatory

- **GDPR Art. 20** (data portability): CalDAV at M03 + .ics
  import/export already satisfies; JMAP Calendars at M04 adds a more
  modern format.
- **KR PIPA Art. 36** (right to provide data): same.
- **HIPAA 45 CFR §164.522(b)** (right to inspect appointment data):
  CalDAV inspection adequate.
- **EU AI Act**: out of scope.

## Verification

- [ ] **CalDAV (RFC 4791) end-to-end interop passes at M03** —
  `cargo nextest run -p tests --test e2e_caldav_clients`.
- [ ] **CalDAV availability SLO authored** — `slos/caldav-availability.
  openslo.yaml` exists.
- [ ] **JMAP Calendars CI lane scaffolded as REPORT-ONLY at M03** —
  `oya gate validate jmap-calendars-draft-conformance --microservice
  calendar --report-only` exits 0.
- [ ] **SDK launch order documented** — `sdk-plan.md` table matches
  the table in this ADR.

## References

- RFC 4791 — Calendaring Extensions to WebDAV (CalDAV).
- RFC 5545 — iCalendar.
- RFC 6638 — Scheduling Extensions to CalDAV.
- RFC 7953 — Calendar Availability (VAVAILABILITY).
- RFC 8620 — JMAP Core.
- RFC 8621 — JMAP Mail.
- RFC 8984 — JSCalendar (data model used by JMAP Calendars).
- draft-ietf-jmap-calendars-16 — JMAP Calendars (IETF WG draft).
- Fastmail JMAP Calendars implementation — `jmap.io`.
- Apple WWDC 2024 EventKit notes on JMAP — public sessions.
- ADR-MAIL-0003 — SDK launch order (paired sibling decision).
- ADR-0131; ADR-0132; ADR-0133.
- ADR-CAL-0001 (CalDAV backend selection).
- ADR-CAL-0002 (RRULE conformance engine).
- `microservices/calendar/PRD.md` FR-09 + AC-04 + §Competitive Benchmark.
- `microservices/calendar/sdk-plan.md`.
