---
doc_class: DeprecationNotice
template_id: TPL-DEPRECATION-NOTICE
microservice: calendar
deprecated_artifact: oya-calendar-* crate family
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-CALENDAR accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-CAL-0001, ADR-CAL-0002, ADR-CAL-0003, ADR-CAL-0004]
related_specs: [/specs/microservices/calendar.json]
owner_team: axis-calendar
date: 2026-05-17
doc_status: published
---

# Deprecation Notice: `oya-calendar-*` crate family

> Formal deprecation notice in the format prescribed by the agent-skills
> `deprecation-and-migration` skill SKILL.md §"Step 2: Announce and Document".

## Status

**Deprecated as of 2026-05-17.**

## Replacement

`oya-calendar-*` crate family under `microservices/calendar/src/crates/`
per ADR-0131. See **`microservices/calendar/migration-from-connect.md`**
for the full import-path map (47 crate mappings), Hyrum's-Law-bound
surface callouts, configuration delta table, runbook continuity table
(6 preserved + 6 net-new), and step-by-step migration guide.

## Removal date

**Advisory — no hard deadline.** Concrete removal target is HG-CALENDAR
accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #3).
Following the 5-month Strangler window in ADR-0134 (Phase 2 adapter soak +
Phase 3 canary), the indicative advisory removal date is **2026-11-17**,
gated on the SLO trigger.

## Reason

The legacy `oya-calendar-*` family was authored before the
following ADRs crystallised; each ADR makes the legacy shape non-
conforming:

1. **ADR-0132 — no-grouping forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a
   brand-layer concept and must not appear in crate names.
2. **ADR-0139 — agentic SLO-gated promotion.** Calendar needs
   independent SLO targets per surface (agenda-render latency, free/
   busy query, ics-import throughput, scheduling-convergence latency,
   CalDAV availability, notification-delivery freshness, RSVP fanout
   latency, room-conflict-detection correctness 100%, tzdb staleness
   bound); a `connect-*` umbrella SLO cannot serve them.
3. **ADR-0131 — per-µservice flat layout.** Calendar's IaC, runbooks,
   threat-model, DPIA, compliance, capacity-model, cost-budget all
   need to live under one folder (`microservices/calendar/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA), pack-eu
   (GDPR Art. 17), pack-us-healthcare (HIPAA), pack-jp, pack-sg, etc.
   need to live at per-µservice overlay granularity.
5. **ADR-CAL-0001 → ADR-CAL-0004** — calendar-specific decisions
   (CalDAV backend, RRULE engine, frontend priority, tzdb refresh)
   need to live at per-µservice ADR granularity, not at the Connect
   suite level.

## Migration Guide pointer

→ **`microservices/calendar/migration-from-connect.md`**

Includes: 1:1 import-path map (47 mappings); net-new-boundary
features (sabredav backend, JMAP Calendars adapter at M04, tzdb
refresh worker, cross-tenant resolver, meeting-creation bridge);
concrete `use` and `Cargo.toml` rewrites; configuration delta table;
dual-context isolation invariant preservation; Hyrum's-Law surface
callouts (RRULE iteration, slot boundary inclusivity, ICS X-extension
preservation, tzdb pin behaviour, RSVP race tie-breaks, recurrence
horizon bound at 5y, CalDAV strong-ETag format); runbook continuity
table (6 preserved + 6 net-new); 5-step migration recipe; 6-phase
Strangler timeline; verification checklist.

## Affected packages enumerated

Per `find crates -maxdepth 1 -type d -name 'oya-calendar-*'`
(2026-05-17 workspace state):

| Currently extant in `crates/` | Mapped replacement |
|---|---|
| `oya-calendar-domain` | split per BC → `oya-calendar-{event-store,recurrence-engine,availability-resolver,room-booking,invitation-flow,ics-import-export}-domain` |

Plus all `oya-calendar-{kernel,usecase,api,adapter*,rest,
worker,sdk,app}-*` crates scaffolded during Phase 2 adapter authoring.

## Breaking changes flagged per `feedback_no_silent_regression`

| Change | Phase | Breaking? | Sunset notice |
|---|---|---|---|
| New `oya-calendar-*` crates ship in parallel | 1 | No (additive) | — |
| New `oya-calendar-tzdb-refresh-worker` (ADR-CAL-0004) | 1 | No (net-new; no legacy counterpart) | — |
| New CalDAV backend-qualified adapters (ADR-CAL-0001) | 1 | No (replaces a less-typed legacy adapter; preserved during canary) | — |
| `rrule-rs` engine replaces legacy in-house RRULE | 1 | **Behaviourally divergent** for 7 named edge cases per ADR-CAL-0002 | adapter does NOT mask divergence; documented in migration guide Hyrum #1 |
| Recurrence horizon bounded at 5y (PRD AC-10) | 1 | **Behaviourally divergent** for unbounded legacy RRULEs | adapter does NOT mask; documented Hyrum #6 |
| CalDAV strong-ETag format | 1 | **Format-divergent** | invisible at CalDAV protocol level; documented Hyrum #7 |
| `oya-calendar-migration-adapter` shim authored | 2 | No (preserves legacy symbol surface) | — |
| Feature-flagged canary 10→50→100% | 3 | No (additive, gated) | — |
| Zero-usage verification | 4 | No (observability only) | — |
| **`oya-calendar-*` crates removed from workspace** | **5** | **YES — breaking** | **6-mo advisory sunset from 2026-05-17** |
| `microservices/connector/` umbrella folder removed | 6 | No | — |

Per `feedback_no_silent_regression.md`, the Phase 5 breaking change carries:

- **This deprecation notice** (renders the change loud + immediate +
  CI-detectable).
- **ADR-0134** (carries the migration policy decision).
- **ADR-CAL-0002** (specifically documents the RRULE behavioural
  strengthening as a deliberate, owner-authored design choice — NOT a
  silent regression).
- **Version bump.** The `Cargo.toml` of every consumer crate is bumped
  per semver when its legacy imports are removed (treating the
  `oya-calendar-*` re-export as the public contract).
- **Sunset schedule.** 6-month advisory window from this notice; concrete
  date 2026-11-17 contingent on the HG-CALENDAR SLO trigger.
- **Owning-axis migration ChangeSets.** axis-calendar ships migration
  ChangeSets for every known internal consumer per the Churn Rule
  before Phase 5.

## Verification (per skill SKILL.md §"Verification")

- [ ] Replacement is production-proven and covers all critical use cases —
  HG-CALENDAR gate at p99 SLO sustained 30d.
- [ ] Migration guide exists with concrete steps and examples —
  `migration-from-connect.md`.
- [ ] All active consumers have been migrated — verified by Phase 4
  commands (see ADR-0134 §Phase 4).
- [ ] Old code, tests, documentation, configuration removed — Phase 5
  commands.
- [ ] No references to the deprecated system remain — `rg
  "oya_connect_calendar" --type rust` produces zero hits outside
  historical surfaces.
- [ ] Deprecation notices removed — this notice deletes itself in Phase 5.

## References

- ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- ADR-CAL-0001 (CalDAV backend selection — `connect.caldav.*` →
  `calendar.radicale.*` + healthcare `calendar.sabredav.*`).
- ADR-CAL-0002 (RRULE engine `rrule-rs` 0.13.x — full RFC 5545
  conformance).
- ADR-CAL-0003 (CalDAV at M03; JMAP Calendars at M04).
- ADR-CAL-0004 (IANA tzdb refresh + per-tenant pin policy).
- `microservices/calendar/migration-from-connect.md` — full migration guide.
- `microservices/calendar/PRD.md` — target-state product definition.
- `microservices/calendar/runbooks/*.md` — 12 runbooks (6 preserved + 6 new).
- `feedback_no_silent_regression.md`.
- agent-skills deprecation-and-migration SKILL.md.
- RFC 5545 — iCalendar.
- RFC 4791 — CalDAV.
- RFC 6638 — CalDAV Scheduling Extensions.
