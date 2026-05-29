---
id: ADR-TASKS-0003
status: Accepted
date: 2026-05-17
microservice: tasks
deciders: axis-tasks, council-architecture, axis-calendar
owner: axis-tasks + axis-calendar
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-CAL-0002
related_artifacts:
  - microservices/tasks/PRD.md (FR-05; AC-03)
  - microservices/tasks/IP-008-recurring-task-engine.md
  - microservices/calendar/decisions/ADR-CAL-0002-recurrence-engine-rfc-conformance.md
purpose: |
  Pick the recurrence engine + RFC 5545 RRULE conformance contract for the
  tasks µservice and align 1:1 with the calendar µservice so that recurring
  tasks and recurring events share semantics, corpus, and library pin.
---

# ADR-TASKS-0003: Recurring task engine — RFC 5545 RRULE subset aligned with calendar ADR-CAL-0002 (rrule-rs 0.13.x); bounded materialisation (5y horizon)

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD-tasks §FR-05 mandates recurring tasks via an RFC 5545 RRULE subset.
Three problems collide:

1. **RFC 5545 fidelity**. RRULE has 14 base components (FREQ, INTERVAL,
   COUNT, UNTIL, BYDAY, BYMONTHDAY, BYMONTH, BYWEEKNO, BYHOUR, BYMINUTE,
   BYSECOND, BYSETPOS, WKST, EXDATE). Edge cases (BYSETPOS + EXDATE
   interaction, DST transitions, leap years, week-number ISO-8601
   variance) are notorious.
2. **Materialisation horizon**. An RRULE with no UNTIL + no COUNT can
   produce unbounded occurrences. PRD §"Performance" requires a
   bounded horizon for materialisation; calendar ADR-CAL-0002 chose 5
   years.
3. **Alignment with the calendar µservice**. The calendar µservice
   already shipped `rrule-rs 0.13.x` LTS via ADR-CAL-0002. If tasks
   ships a different engine, recurring task ↔ recurring event bindings
   (PRD §FR-12 calendar-bridge) could diverge — a recurring task
   bound to a recurring event would render different occurrences
   between the two surfaces, a bug that observability cannot detect
   ahead-of-time.

Candidates:

1. **`rrule-rs 0.13.x`** (active Rust crate; aligned with calendar's
   ADR-CAL-0002 pick). Pros: zero divergence risk with calendar;
   already vetted through calendar's libical conformance corpus; LTS
   pinning policy already in place.
2. **`chrono-rrule`**. Alternative Rust crate; smaller surface; less
   active maintenance.
3. **Hand-rolled engine**. Build a minimal RRULE parser + expander
   covering only the subset of FREQ ∈ {DAILY, WEEKLY, MONTHLY, YEARLY}
   plus INTERVAL + COUNT + UNTIL.

## Decision

The tasks µservice ships **`rrule-rs 0.13.x` LTS, aligned 1:1 with
calendar ADR-CAL-0002**. Supported subset:

- `FREQ ∈ {SECONDLY (forbidden by domain validator), MINUTELY (forbidden), HOURLY (forbidden), DAILY, WEEKLY, MONTHLY, YEARLY}`. The sub-daily kinds are rejected at the domain layer because tasks have no use case for sub-daily recurrence (calendar permits HOURLY for shift-work but tasks does not).
- `INTERVAL`, `COUNT`, `UNTIL`, `BYDAY`, `BYMONTHDAY`, `BYMONTH`,
  `BYWEEKNO`, `BYHOUR`, `BYMINUTE`, `BYSETPOS`, `WKST`, `EXDATE` —
  full support.
- Bounded materialisation horizon = **5 years** from the rule's
  `start_at`. Rules yielding beyond 5y refuse to materialise the tail
  occurrences and emit a `RecurrenceHorizonTruncated` marker.
- DST + IANA tzdb handling delegated to the timezone resolver shared
  with calendar (ADR-CAL-0004 tzdb refresh policy).

The recurrence worker (`oya-tasks-recurrence-worker`) materialises a
rolling 24-hour window of upcoming task instances; the AC-03
conformance corpus (libical-derived; shared with calendar) is the
release gate.

## Alternatives Considered

### Alternative 1 — `chrono-rrule` alternative crate

- Pros:
  - Smaller surface; easier to audit.
- Cons:
  - Different parser + different edge-case handling vs calendar's
    `rrule-rs`. Task ↔ event divergence in occurrence sets.
  - Less active maintenance (last release > 18 months at decision
    time).
- Rejected because: divergence with calendar is a P0 design risk
  for the cross-µservice bridge (FR-12).

### Alternative 2 — Hand-rolled minimal RRULE parser

- Pros:
  - Total control over edge cases.
- Cons:
  - The set of edge cases is enormous; conformance corpus tests
    document hundreds of test vectors; building from scratch wastes
    eng-time that could be spent on tasks-specific features.
  - Future RFC 5545bis (draft-ietf-calext-icalendar-bis) refinement
    requires us to track changes; rrule-rs upstream absorbs this work.
- Rejected because: NIH cost > divergence cost.

### Alternative 3 — Outsource recurrence to the workflow-engine

- Pros:
  - workflow-engine already has a durable scheduler.
- Cons:
  - workflow-engine isn't an RRULE engine; it'd need an RRULE adapter
    anyway.
  - Adds an inter-µservice dependency for the materialisation path
    that the calendar µservice doesn't have (calendar runs its own
    recurrence in-µservice).
- Rejected because: same-shape boundary already drawn by calendar;
  asymmetry would cost more than it saves.

## Consequences

### Consequence 1 — Conformance corpus is shared

The libical RRULE corpus lives at
`microservices/calendar/tests/rrule-corpus/` and is imported by
`microservices/tasks/tests/rrule-corpus/` as a git submodule or path
dependency. Both µservices' AC-03 lanes consume the same vectors.
If calendar updates a corpus expectation, tasks updates in lockstep.

### Consequence 2 — IANA tzdb refresh is shared

Per ADR-CAL-0004, calendar runs a tzdb refresh CronJob. The tasks
µservice subscribes to the same tzdb dataset via the substrate
`tzdb-bridge` pattern. A tzdb update applies to both µservices
simultaneously. Per-tenant tz pinning (regulated sectors) is honoured
via the shared resolver.

### Consequence 3 — Sub-daily forbid at domain layer

The domain validator refuses `FREQ ∈ {SECONDLY, MINUTELY, HOURLY}`.
A future product decision could relax this (e.g., a 6-hour security-
patrol task in physical-security packs); doing so requires an ADR
amendment + a corresponding tenant_class and paid billing-component review per ADR-0329, ADR-0330, ADR-0331, and ADR-TASKS-0006.

## References

- ADR-CAL-0002 (calendar RRULE conformance).
- ADR-CAL-0004 (tzdb refresh).
- ADR-TASKS-0002 (dependency graph); ADR-TASKS-0005 (workflow bridge).
- RFC 5545 — `tools.ietf.org/html/rfc5545`.
- RFC 5545bis draft — `tools.ietf.org/wg/calext/`.
- rrule-rs 0.13 — `github.com/fmeringdal/rust-rrule`.
- libical — `github.com/libical/libical`.
- PRD-tasks §FR-05 + AC-03.
