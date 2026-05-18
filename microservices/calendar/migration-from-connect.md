---
doc_class: MigrationGuide
template_id: TPL-MIGRATION-GUIDE
microservice: calendar
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-CALENDAR accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-CAL-0001, ADR-CAL-0002, ADR-CAL-0003, ADR-CAL-0004]
related_specs: [/specs/microservices/calendar.json, /specs/microservices/calendar/calendar.json]
owner_team: axis-calendar
date: 2026-05-17
doc_status: published
---

# Migration: `oya-connect-calendar-*` → `oya-calendar-*`

This document applies the Strangler Pattern from the agent-skills
`deprecation-and-migration` skill to the **calendar** µservice. It is the
consumer-facing companion to ADR-0134 (cross-µservice migration policy) and
ADR-0135 (target topology).

## Status

**Deprecated as of 2026-05-17 — replacement available and production-proven
in dev cluster.**

| Field | Value |
|---|---|
| Replacement | `oya-calendar-*` crate family under `microservices/calendar/src/crates/` |
| Removal date | **Advisory** — concrete target is HG-CALENDAR accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #3) |
| Reason | ADR-0132 no-suite forward-policy + ADR-0139 per-µservice SLO authority + ADR-0131 per-µservice flat layout + the 6-BC calendar surface (events / recurrence / availability / room-booking / invitation-flow / ics-import-export) is only addressable at µservice granularity, not at Connect-suite granularity |
| Migration owner (Churn Rule) | axis-calendar |
| Migration window | Phase 2 adapter + Phase 3 canary = ~5 months; Phase 5 removal sweep in month 6 (see ADR-0134) |

## Replacement

The 6 bounded-contexts of the `calendar` µservice live under
`microservices/calendar/src/crates/` per ADR-0131. Each legacy
`oya-connect-calendar-*` crate has a 1:1 replacement under the new prefix.

### Crate import-path map

| Legacy `oya-connect-calendar-*` path | New `oya-calendar-*` path |
|---|---|
| `oya-connect-calendar-domain` | (split per BC; see note below) |
| `oya-connect-calendar-event-kernel` | `oya-calendar-event-store-kernel` |
| `oya-connect-calendar-event-domain` | `oya-calendar-event-store-domain` |
| `oya-connect-calendar-event-usecase` | `oya-calendar-event-store-usecase` |
| `oya-connect-calendar-event-api` | `oya-calendar-event-store-api` |
| `oya-connect-calendar-event-adapter` | `oya-calendar-event-store-adapter` |
| `oya-connect-calendar-event-adapter-postgres` | `oya-calendar-event-store-adapter-postgres` |
| `oya-connect-calendar-event-rest` | `oya-calendar-event-store-rest` |
| `oya-connect-calendar-event-worker` | `oya-calendar-event-store-worker` |
| `oya-connect-calendar-event-sdk` | `oya-calendar-event-store-sdk` |
| `oya-connect-calendar-event-app` | `oya-calendar-event-store-app` |
| `oya-connect-calendar-recurrence-kernel` | `oya-calendar-recurrence-engine-kernel` |
| `oya-connect-calendar-recurrence-domain` | `oya-calendar-recurrence-engine-domain` |
| `oya-connect-calendar-recurrence-usecase` | `oya-calendar-recurrence-engine-usecase` |
| `oya-connect-calendar-recurrence-api` | `oya-calendar-recurrence-engine-api` |
| `oya-connect-calendar-recurrence-adapter` | `oya-calendar-recurrence-engine-adapter` |
| `oya-connect-calendar-recurrence-app` | `oya-calendar-recurrence-engine-app` |
| `oya-connect-calendar-availability-kernel` | `oya-calendar-availability-resolver-kernel` |
| `oya-connect-calendar-availability-domain` | `oya-calendar-availability-resolver-domain` |
| `oya-connect-calendar-availability-usecase` | `oya-calendar-availability-resolver-usecase` |
| `oya-connect-calendar-availability-api` | `oya-calendar-availability-resolver-api` |
| `oya-connect-calendar-availability-adapter` | `oya-calendar-availability-resolver-adapter` |
| `oya-connect-calendar-availability-adapter-redis` | `oya-calendar-availability-resolver-adapter-redis` |
| `oya-connect-calendar-availability-rest` | `oya-calendar-availability-resolver-rest` |
| `oya-connect-calendar-availability-worker` | `oya-calendar-availability-resolver-worker` |
| `oya-connect-calendar-availability-app` | `oya-calendar-availability-resolver-app` |
| `oya-connect-calendar-room-kernel` | `oya-calendar-room-booking-kernel` |
| `oya-connect-calendar-room-domain` | `oya-calendar-room-booking-domain` |
| `oya-connect-calendar-room-usecase` | `oya-calendar-room-booking-usecase` |
| `oya-connect-calendar-room-api` | `oya-calendar-room-booking-api` |
| `oya-connect-calendar-room-adapter` | `oya-calendar-room-booking-adapter` |
| `oya-connect-calendar-room-rest` | `oya-calendar-room-booking-rest` |
| `oya-connect-calendar-room-app` | `oya-calendar-room-booking-app` |
| `oya-connect-calendar-invitation-kernel` | `oya-calendar-invitation-flow-kernel` |
| `oya-connect-calendar-invitation-domain` | `oya-calendar-invitation-flow-domain` |
| `oya-connect-calendar-invitation-usecase` | `oya-calendar-invitation-flow-usecase` |
| `oya-connect-calendar-invitation-api` | `oya-calendar-invitation-flow-api` |
| `oya-connect-calendar-invitation-adapter` | `oya-calendar-invitation-flow-adapter` |
| `oya-connect-calendar-invitation-worker` | `oya-calendar-invitation-flow-worker` |
| `oya-connect-calendar-invitation-app` | `oya-calendar-invitation-flow-app` |
| `oya-connect-calendar-ics-kernel` | `oya-calendar-ics-import-export-kernel` |
| `oya-connect-calendar-ics-domain` | `oya-calendar-ics-import-export-domain` |
| `oya-connect-calendar-ics-usecase` | `oya-calendar-ics-import-export-usecase` |
| `oya-connect-calendar-ics-api` | `oya-calendar-ics-import-export-api` |
| `oya-connect-calendar-ics-adapter` | `oya-calendar-ics-import-export-adapter` |
| `oya-connect-calendar-ics-adapter-icalendar` | `oya-calendar-ics-import-export-adapter-icalendar` |
| `oya-connect-calendar-ics-adapter-caldav` | `oya-calendar-ics-import-export-adapter-caldav-radicale` (per ADR-CAL-0001 — backend-qualified) |
| `oya-connect-calendar-ics-rest` | `oya-calendar-ics-import-export-rest` |
| `oya-connect-calendar-ics-app` | `oya-calendar-ics-import-export-app` |

> **`oya-connect-calendar-domain` split.** The legacy bundled crate
> bundled events + recurrence + availability + rooms + invitations +
> ics-import-export into a single domain-layer crate. Per ADR-0131 +
> ADR-0105 (13-layer enum), the new layout splits the domain layer per
> bounded context. Migration imports from the legacy bundled
> `oya-connect-calendar-domain` must each pick the specific replacement
> BC; a one-line wholesale `use oya_calendar::*` import is not
> supported.

### Net-new boundaries (no legacy counterpart)

The new µservice introduces capabilities that did NOT exist in
`oya-connect-calendar-*`. They are therefore not part of the migration
surface — they are clean replacement-boundary features. Specifically:

- **`oya-calendar-ics-import-export-adapter-caldav-sabredav`** —
  secondary CalDAV backend per ADR-CAL-0001 (only enabled on
  `pack-us-healthcare`).
- **JMAP Calendars adapter** (`oya-calendar-ics-import-export-
  adapter-jmap`) — scheduled-for-distinct-tracked-work to M04 per ADR-CAL-0003.
- **`oya-calendar-tzdb-refresh-worker`** — automated IANA tzdb
  refresh per ADR-CAL-0004; the legacy connect-calendar surface had
  no tzdb refresh worker.
- **Cross-tenant availability with policy-bounded disclosure** — PRD
  FR-10 differentiator; legacy `oya-connect-calendar-*` had no
  Cedar-gated cross-tenant resolver.
- **Dual-context (Personal / Professional) structural isolation** —
  inherited from Bominal ADR-0208 but enforced in code (Cedar
  `policy/event-isolation.md`); the legacy surface only had policy-
  layer isolation, not code-layer.
- **Meeting-creation bridge to messenger huddles** —
  cross-µservice via Workflow per PRD §"Workflow events consumed";
  legacy surface had no messenger linkage.

### Concrete import migration recipes

```rust
// BEFORE
use oya_connect_calendar_event_kernel::{CalendarEvent, Attendee};
use oya_connect_calendar_event_usecase::CreateEvent;
use oya_connect_calendar_recurrence_kernel::RecurrenceRule;
use oya_connect_calendar_invitation_kernel::{Invitation, RsvpState};

// AFTER
use oya_calendar_event_store_kernel::{CalendarEvent, Attendee};
use oya_calendar_event_store_usecase::CreateEvent;
use oya_calendar_recurrence_engine_kernel::RecurrenceRule;
use oya_calendar_invitation_flow_kernel::{Invitation, RsvpState};
```

```toml
# BEFORE — Cargo.toml of a downstream consumer
[dependencies]
oya-connect-calendar-event-kernel = { workspace = true }
oya-connect-calendar-event-usecase = { workspace = true }
oya-connect-calendar-recurrence-kernel = { workspace = true }
oya-connect-calendar-invitation-kernel = { workspace = true }

# AFTER
[dependencies]
oya-calendar-event-store-kernel = { workspace = true }
oya-calendar-event-store-usecase = { workspace = true }
oya-calendar-recurrence-engine-kernel = { workspace = true }
oya-calendar-invitation-flow-kernel = { workspace = true }
```

## Reason

The legacy `oya-connect-calendar-*` family was authored before the
following ADRs crystallised:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a
   brand-layer concept and must not appear in crate names.
2. **ADR-0139 — per-µservice SLO authority.** Calendar needs
   independent SLO targets per surface (agenda-render latency,
   free/busy query, ics-import throughput, scheduling-convergence
   latency, CalDAV availability, notification-delivery freshness,
   RSVP fanout latency, room-conflict-detection correctness 100%,
   tzdb staleness bound). A `connect-*` umbrella SLO cannot honour
   those.
3. **ADR-0131 — per-µservice flat layout.** Calendar's IaC, runbooks,
   threat-model, DPIA, compliance, capacity-model, cost-budget,
   incident-response, failure-modes, multi-region all need to live
   under one folder (`microservices/calendar/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA +
   KR-FSS), pack-eu (GDPR Art. 17), pack-us (general), pack-us-
   healthcare (HIPAA appointment-data), pack-jp (APPI), pack-sg
   (PDPA), pack-au (Privacy Act), pack-in (DPDPA), pack-br (LGPD),
   pack-ae (UAE PDPL + Hijri overlay), pack-ksa (KSA PDPL + Hijri
   overlay) — each lives as `microservices/calendar/policy/pack-
   <region>/`. They cannot share a folder root with mail / messenger /
   community.
5. **ADR-CAL-0001 → ADR-CAL-0004** — calendar-specific decisions
   (CalDAV backend pick, RRULE engine choice, CalDAV-first frontend
   priority, tzdb refresh policy) need to live at per-µservice ADR
   granularity, not at the Connect suite level.

## Migration Guide (step-by-step)

For each consumer crate that imports `oya-connect-calendar-*`:

### Step 1 — Add the new dependency

```bash
# In your consumer crate's Cargo.toml, add the new mapped dependency.
# Keep the legacy dependency for now (Phase 2 adapter soak).
```

### Step 2 — Update imports per the import-path map above

```bash
# Use this command per file as a guided rewrite (review every hit;
# manual disambiguation needed for the `oya-connect-calendar-domain`
# split case):
rg -l "oya_connect_calendar_" --type rust path/to/your/crate
```

### Step 3 — Verify behavioural parity

```bash
# Inside your consumer crate:
cargo nextest run --features connect-calendar-strangler-canary
```

Run with the feature flag enabled to route through the new µservice;
run without to route through the legacy adapter. Compare:

- error variant ordering (Hyrum's Law — see surfaces below).
- p99 latency (must be ≤ legacy + 5% per ADR-0134 Phase 3 canary gate).
- log-line format (preserved verbatim during the canary; may be
  tightened in a successor-IP `feedback_no_silent_regression`-conforming
  ADR).
- RRULE expansion output (per ADR-CAL-0002 — full RFC 5545 conformance
  may emit DIFFERENT occurrences from the legacy engine for the named
  edge cases; see Hyrum's-Law surface #1 below).

### Step 4 — Remove the legacy dependency

Only after your consumer crate's tests pass against the new imports
AND the calendar µservice's Phase 3 canary reaches 100% traffic (per
ADR-0134), remove the legacy dependency from your `Cargo.toml`:

```toml
# Remove this line:
oya-connect-calendar-event-kernel = { workspace = true }
```

### Step 5 — Verify zero residual

```bash
# Per ADR-0134 Phase 4 verification:
cargo tree -e normal -p your-crate | grep oya-connect-calendar   # expect empty
rg "use oya_connect_calendar_" --type rust path/to/your/crate    # expect zero hits
```

## Configuration delta

| Configuration key | Legacy | New |
|---|---|---|
| Feature flag namespace | `connect.calendar.*` | `calendar.*` |
| OpenSLO file | bundled in `Connect.openslo.yaml` (umbrella) | `microservices/calendar/slos/*.openslo.yaml` (per-µservice, 9 files) |
| Helm chart values key | `.Values.connect.calendar.*` | `.Values.calendar.*` |
| K8s namespace | `connect` | `calendar` |
| Cedar policy fragment path | `policy/connect/calendar/*.cedar` | `microservices/calendar/policy/*.cedar` |
| pack-kr overlay path | `policy/connect/calendar/pack-kr/*` | `microservices/calendar/iac/kustomize/overlays/pack-kr/*` + per-pack section in `threat-model.md` / `dpia.md` / `compliance.md` / `multi-region.md` |
| Workflow event prefix | `connect.calendar.*` | `calendar.*` (e.g., `calendar.event.lifecycle.v1`, `calendar.invitation.rsvp.v1`, `calendar.room.booking.v1`) |
| Ontology type prefix | `Connect.Calendar.*` | `Calendar.*` (e.g., `Calendar.CalendarEvent`, `Calendar.Resource`, `Calendar.Booking`, `Calendar.Invitation`, `Calendar.LegalHold`) |
| Telemetry metric prefix | `oya_connect_calendar_*` | `oya_calendar_*` |
| Tracing span attribute namespace | `connect.calendar.*` | `calendar.*` |
| CalDAV chart name | `connect.caldav.*` | `calendar.iac.helm.radicale.*` (primary) + `calendar.iac.helm.sabredav.*` (us-healthcare) per ADR-CAL-0001 |
| tzdb engine choice | `chrono-tz` (cluster-default) | `chrono-tz` + per-tenant pin override per ADR-CAL-0004 |
| RRULE engine choice | (legacy in-house) | `rrule-rs` 0.13.x per ADR-CAL-0002 |

## Dual-context isolation invariant (preserved + strengthened)

The Personal ↔ Professional context isolation invariant from the
Bominal ADR-0208 dual-context inheritance is preserved verbatim in
`oya-calendar-event-store-kernel`. Specifically:

- The `EventContextBoundaryGuard` port trait keeps the same method
  signatures.
- Cross-context attempts (Professional → Personal event read) emit
  the same 403 + same audit-chain event variant
  (`CalendarCrossContextRefused`).
- The kernel-layer refusal (not adapter-layer) invariant is preserved.
- **Strengthened**: cross-context attempts are also refused at the
  Cedar policy layer per `policy/event-isolation.md`; the kernel
  refusal is the defence-in-depth backup.

This means downstream consumers that wrap the boundary guard via the
legacy import path will see identical refusal behaviour after
migration; no test rewrite needed for the isolation surface.

## Hyrum's-Law surfaces — explicit callouts

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law Makes
Removal Hard", these are the legacy calendar surfaces with observable
behaviour that may be depended on. Each is preserved verbatim during
the canary; consumers must re-test after Phase 5 removal in case they
had a long-tail dependency:

1. **RRULE iteration order**. The legacy in-house RRULE engine
   emitted occurrences in DTSTART-ascending order. `rrule-rs` (per
   ADR-CAL-0002) preserves DTSTART-ascending order. Consumers that
   pattern-match on the iteration order see no change. **BUT**: for
   the 7 named edge cases in ADR-CAL-0002 (BYSETPOS × EXDATE, etc.),
   the new engine may emit a DIFFERENT set of occurrences from the
   legacy engine because the new engine is RFC-5545-strict where the
   legacy was looser. Consumers using these edge cases MUST review
   their expected expansions during the canary.
2. **Free/busy slot boundary inclusivity.** Legacy emitted slots with
   `[start, end)` half-open boundary (start inclusive, end
   exclusive). New µservice preserves `[start, end)` — explicitly
   documented in `oya-calendar-availability-resolver-kernel`.
   Consumers that depended on end-inclusive boundaries see different
   slot counts at boundary times; pattern-match on `[start, end)`.
3. **ICS X-extension preservation.** Legacy preserved unknown
   `X-*` properties byte-for-byte across .ics import/export. New
   µservice preserves the same; verified by AC-04 round-trip test
   `tests/e2e/ics-roundtrip-x-extensions.rs`.
4. **Timezone-DB version pinning behaviour.** Legacy did not support
   per-tenant tzdb pin; the cluster default was the only tzdb visible.
   New µservice supports per-tenant pin per ADR-CAL-0004. Consumers
   that read historical appointments must NOT assume the latest tzdb
   is the rendering tzdb — they must consult `tzdb_release_in_use`
   on the appointment.
5. **RSVP race-condition tie-breaks.** Legacy used last-write-wins
   for concurrent RSVPs from the same attendee (e.g., user accepts
   then immediately declines from a different client). New µservice
   preserves last-write-wins, with `decided_at` as the deterministic
   tie-breaker. Consumers that pattern-match on the ordering see no
   change.
6. **Recurrence expansion bound at 5y horizon.** Legacy was unbounded;
   new µservice rejects > 5y horizon per PRD AC-10. Consumers that
   submitted long-horizon RRULEs to legacy and expected unbounded
   expansion will receive `MalformedRecurrence::BoundExceeded` on
   the new path. This is a deliberate strengthening; the legacy
   behaviour was a DoS surface.
7. **CalDAV ETag format.** Legacy used a per-event monotonic counter
   as the CalDAV ETag. New µservice uses an RFC-4791-conformant
   strong ETag (SHA-256 of the canonicalised iCalendar
   serialisation). Apple Calendar / Thunderbird / DAVx5 don't care
   about the ETag format; consumers that pattern-matched on the
   monotonic-counter shape will see a hash. Documented in
   `runbooks/caldav-sync-loop.md`.

## Runbook continuity table

| Legacy runbook (under `policy/connect/calendar/runbooks/`) | New runbook (under `microservices/calendar/runbooks/`) | Status |
|---|---|---|
| `room-booking-conflict.md` | `room-booking-conflict.md` | preserved verbatim |
| `ics-import-failure.md` | `ics-import-failure.md` | preserved verbatim |
| `calendar-restore.md` | `calendar-restore.md` | preserved verbatim |
| `availability-cache-rebuild.md` | `availability-cache-rebuild.md` | preserved verbatim |
| `timezone-db-refresh.md` | `timezone-db-refresh.md` | preserved + expanded with ADR-CAL-0004 refresh worker |
| `recurrence-storm.md` | `recurrence-storm.md` | preserved + expanded with ADR-CAL-0002 bound-exceeded refusal |
| (no legacy counterpart) | `caldav-sync-loop.md` | NEW per ADR-CAL-0001 + Hyrum surface #7 |
| (no legacy counterpart) | `scheduling-poll-deadlock.md` | NEW per RFC 6638 auto-scheduling |
| (no legacy counterpart) | `tzdb-rollback.md` | NEW per ADR-CAL-0004 rollback procedure |
| (no legacy counterpart) | `rsvp-storm-throttle.md` | NEW per RSVP fanout SLO |
| (no legacy counterpart) | `shared-cal-permission-drift.md` | NEW per calendar-sharing BC |
| (no legacy counterpart) | `calendar-bridge-mail-loop-detection.md` | NEW per mail-bridge invitation flow |

## Phases (per ADR-0134)

| Phase | Description | Status (calendar) | Exit condition |
|---|---|---|---|
| 1. Parallel ship | New µservice + legacy coexist | **active** | HG-CALENDAR passes at p99 SLOs in dev cluster sustained 7d |
| 2. Adapter soak | `oya-connect-calendar-migration-adapter` shims legacy symbols → new impl | pending | All consumers compile against adapter; 3-month soak elapses |
| 3. Feature-flagged canary | 10% → 50% → 100% traffic shift over 6 weeks | pending | New µservice carries 100% traffic for 7 consecutive days |
| 4. Zero-active-usage verification | Dependency-graph + telemetry + grep all clean | pending | Verification commands all exit 0 |
| 5. Code removal sweep | Delete legacy crates + Cargo.toml entries + spec pointers | pending | `cargo build --workspace` exits 0; no `oya_connect_calendar_*` symbol resolves |
| 6. Umbrella retirement | Conditional on all 8 sub-µservices reaching their own Phase 5 | pending | All 8 HG-<MS> gates green at p99 SLO sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

Per the deprecation-and-migration skill, every deprecation closeout must
satisfy these checks. Each is gated by a concrete command:

- [ ] **Replacement is production-proven and covers all critical use cases.**
  ```bash
  cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice calendar
  # expect: HG-CALENDAR accepts at p99 SLOs sustained 30d
  ```
- [ ] **Migration guide exists with concrete steps and examples.**
  ```bash
  test -f microservices/calendar/migration-from-connect.md   # this file
  ```
- [ ] **All active consumers have been migrated** (per Phase 4):
  ```bash
  cargo tree -e normal -p oya-connect-calendar-domain --invert    | grep -v 'oya-connect-calendar-migration-adapter' | wc -l   # expect 0
  rg "use oya_connect_calendar_" --type rust    | rg -v "migration-adapter|legacy_in_process|tests/"    | wc -l   # expect 0
  ```
- [ ] **Old code, tests, documentation, configuration removed** (per Phase 5):
  ```bash
  find crates -maxdepth 1 -type d -name "oya-connect-calendar-*" | wc -l   # expect 0
  test ! -f /specs/microservices/calendar.json                          # expect file absent
  ```
- [ ] **No references to the deprecated system remain in the codebase**
  (excluding historical ADR / RETIRED.md / git-log surfaces):
  ```bash
  rg "oya_connect_calendar" --type rust    | rg -v "docs/decisions/|RETIRED.md|tests/golden/"    | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed (they served their purpose)** (per
  Phase 5):
  ```bash
  test ! -f microservices/calendar/deprecation-notice.md          # expect file absent
  test ! -f microservices/calendar/migration-from-connect.md      # expect file absent (this file removes itself in Phase 5)
  ```

## Breaking changes (flagged per `feedback_no_silent_regression`)

This migration is **NOT a breaking change** during Phases 1–4 for the
core symbol surface: the adapter preserves the legacy symbol surface
verbatim, including error variant ordering and timing characteristics
within the +5% canary tolerance.

**There ARE three behavioural strengthenings** that may visibly differ
from legacy and are NOT preserved by the adapter (per
`feedback_no_silent_regression`):

1. **RFC 5545 RRULE conformance edge cases** (per ADR-CAL-0002).
   Consumers depending on the legacy RRULE engine's looser
   interpretation of BYSETPOS × EXDATE, BYDAY=-1MO, UNTIL+COUNT, DST
   transitions, leap seconds, and floating times will see strictly-
   conformant expansions instead. This is a deliberate strengthening;
   consumers may need to update their expected expansions.
2. **Recurrence horizon bounded at 5y** (per PRD AC-10).
   Consumers that submitted unbounded RRULEs to the legacy engine and
   expected unbounded expansion will receive
   `MalformedRecurrence::BoundExceeded`. This is a deliberate
   strengthening; the legacy behaviour was a DoS surface.
3. **CalDAV strong-ETag format** (per ADR-CAL-0001 + Hyrum surface
   #7). Consumers that pattern-matched on the monotonic-counter ETag
   shape will see a SHA-256 hash. The CalDAV protocol itself doesn't
   care; the change is invisible at protocol level.

Phase 5 (code removal) **IS a breaking change** for any consumer that
did not migrate during the 5-month adapter+canary window. Per
`feedback_no_silent_regression`:

- Sunset schedule (advisory): 6 months from this document's
  `deprecation_date` (2026-05-17), so a target advisory removal date
  of **2026-11-17** (subject to the HG-CALENDAR retirement trigger
  gating).
- Owning axis (axis-calendar) ships migration ChangeSets for every
  internal consumer per the Churn Rule before Phase 5.
- External consumers (reading `/specs/microservices/calendar.json`)
  receive a 6-month sunset window from this notice; the spec file's
  `deprecated: true` + `replacement_path:
  /specs/microservices/calendar/calendar.json` fields render in the
  agent-coordination dashboard.

## References

- ADR-0135: Connect super-app expansion into 8 flat µservices.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-suite forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0134: Connect dissolution Strangler migration (operational policy).
- ADR-CAL-0001: CalDAV server backend selection.
- ADR-CAL-0002: Recurrence engine RFC 5545 conformance.
- ADR-CAL-0003: JMAP vs CalDAV frontend priority.
- ADR-CAL-0004: IANA tzdb refresh + pinning policy.
- RFC 5545 — iCalendar.
- RFC 5546 — iTIP.
- RFC 6047 — iMIP.
- RFC 4791 — CalDAV.
- RFC 6638 — CalDAV Scheduling Extensions.
- RFC 7953 — VAVAILABILITY.
- RFC 8984 — JSCalendar.
- draft-ietf-jmap-calendars — JMAP Calendars.
- `microservices/calendar/PRD.md` — full target-state product definition.
- `microservices/calendar/PHASE-01-CALENDAR-FOUNDATION.md` — phase plan.
- `microservices/calendar/deprecation-notice.md` — formal deprecation notice.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern + Adapter Pattern + Churn Rule + Verification.
