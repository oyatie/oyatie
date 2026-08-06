---
id: ADR-0599
title: "Commission the comms calendar capability-first move + cloud-agnostic core slice (comms-calendar-domain/api/usecase)"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-22
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0015, ADR-0510, ADR-0536, ADR-0538, ADR-0555, ADR-0562, ADR-0563]
related: [ADR-0029, ADR-0083, ADR-0105, ADR-0131, ADR-0139, ADR-0245, ADR-0280, ADR-0512]
related_specs:
  - /specs/capability-registry.json
  - /specs/reachability-registry.json
milestone: W2
---

# ADR-0599: Commission the comms calendar capability-first slice

## Status

**Proposed - 2026-06-22 (authored for founder sign-off; door: two-way — an additive capability-first
relocation of one existing domain crate plus two new pure-application crates behind already-accepted
ports/clean-arch seams; removable by deleting the new crates and reverting the move without unwinding
any SSOT; the accounting producer remains the sole face generator).**

## Context

The capability-first reorg (ADR-0562) homes each product capability under
`<capability>/{core,ports,adapters,facade}/`. The `comms` capability tree was established by the
twelfth strangler move (ADR-0562 §10.16, mail + messenger + meet + contact-center) under the
existing `comms/*/*` glob-only workspace membership (ADR-0538), and its ownership/reachability seed
is `comms/OWNERS` (breadth-unlimited, ADR-0555).

The workspace calendar kernel still lived OUTSIDE its capability home at
`oya/calendar/crates/oya-calendar-domain` — the 430-line W-Workspace calendar domain (ADR-0029,
typed `Classified<…>` records: `Calendar`, `CalendarEvent`, `Attendee`, `RecurrenceRule`,
`CalendarSlot`, the `SlotPicker` seam, and the CalDAV STAGING surface). Its ONLY Cargo dependency is
`libs/oya-data-boundary-kernel`; the meet coupling is a single domain error variant
(`InvalidMeetSessionId`), NOT a build-time crate dependency — so calendar relocates independently of
meet. No workspace crate depends on `oya-calendar-domain`, so the move is import-rewrite-free.

Beyond the domain kernel, the calendar capability had NO cloud-agnostic application layer: no port
defining the persistence/scheduling seams, and no usecase composing the event/recurrence/attendee
invariants the kernel cannot enforce alone. CalDAV and persistence adapters were (correctly) absent
— CalDAV stays out of the kernel per ADR-0015 — but so was the clean-arch boundary that lets those
adapters be wired LATER without touching the domain.

## Decision

### D1 — MOVE the domain into its capability home (de-branded, glob-covered)

Relocate `oya/calendar/crates/oya-calendar-domain` to `comms/core/calendar-domain` via the
deterministic reorg codemod (ADR-0563), de-branding the cargo name
`oya-calendar-domain` → `comms-calendar-domain` (drop the vendor prefix; path-tail == cargo name;
face dir not in the name). The crate lands under the existing `comms/*/*` glob, so the root
workspace `members` array is NOT edited (ADR-0538 glob-only contract;
`root_workspace_changed = false`). The gate-bound catalog record is re-keyed
`registry/catalog/oya-calendar-domain.yaml` → `registry/catalog/comms-calendar-domain.yaml` to track
the live crate id (catalog-liveness keys on the file stem). The committed move plan rotates the
predecessor `intelligence-move-plan.json` out, keeping exactly one plan in `specs/reorg/` (the
single-plan invariant the codemod enforces fail-closed). The `oya/calendar/slos/*` SLO subtree is
NOT co-moved: those SLOs gate the unbuilt event-store / availability-resolver / recurrence-engine /
ICS / CalDAV adapters + workers, not the domain crate that moves; they home when the calendar
product fully lands (the intelligence-move precedent co-moves only the SLOs of the crates that move).

### D2 — BUILD the cloud-agnostic core slice (clean-arch, owned-stack shape)

Add the port `comms/ports/calendar-api` (`comms-calendar-api`) and the usecase
`comms/core/calendar-usecase` (`comms-calendar-usecase`):

- The port defines the trait seams concrete adapters implement LATER: `CalendarStore` (a
  tenant-scoped persistence seam) and `FreeBusyResolver` (the free/busy + slot-finding seam,
  mirroring the domain `SlotPicker` at the boundary). The trait shapes model the W5 owned-stack
  destination; adapters absorb Postgres/Valkey/CalDAV. CalDAV stays OUT of the port and the kernel
  per ADR-0015.
- The port also defines the fail-closed `AuthorizedCalendarContext`: default-deny by construction
  (no anonymous constructor), admitting a call ONLY when a verified principal, a `tenant:`-prefixed
  scope, a non-empty Cedar policy-decision ref, and an audit-correlation id are ALL present. Any
  future HTTP/gRPC facade MUST present a valid context before it touches tenant data (the new-HTTP-
  surfaces default-deny doctrine), mirroring `comms-mail-mailbox-api`.
- The usecase composes the event/recurrence/attendee invariants over the domain + port:
  fail-closed authz at every entrypoint, a tenant-isolation guard across the request/aggregate
  boundary (defense in depth, independent of any backend RLS), attendee well-formedness (delegated
  to the kernel `Attendee::new`), RFC-5545 RRULE `FREQ`-subset validation before building the kernel
  `RecurrenceRule`, and a scheduling composition (`schedule_event`: first free slot → validated
  event). It is pure application logic — NO persistence, cloud, identity, or CalDAV backend; those
  are DEFERRED behind the port traits.

### D3 — Adapters DEFERRED behind the ports

The cloud/persistence/identity adapters (a Postgres `CalendarStore` with FORCE-RLS tenant isolation,
a Valkey/availability `FreeBusyResolver`, and any CalDAV/ICS protocol adapter) are intentionally OUT
of this slice. They are commissioned later behind the unchanged D2 ports, so the domain and usecase
never change at adapter-wiring or owned-stack cutover.

## Consequences

The calendar capability gains its capability-first home and a tested, cloud-agnostic core
(domain + port + usecase) with fail-closed authorization wired at the application boundary, while
deferring every transient-infra concern behind clean-arch ports. The move is byte-deterministic
(codemod + producer-regenerated faces) and adds zero new acyclicity/membership/total-accounting
debt: the moved domain is baseline-relabeled by the move-manifest (ADR-0563), and the new crates +
catalog records + move plan are justified by THIS ADR (the producer derives `justification_ref:
ADR-0599` from the paths named below), owned by `comms/OWNERS` / `registry/catalog/OWNERS` /
`specs/reorg/OWNERS`, and reachable via the existing `comms/OWNERS`, `registry/catalog/`, and
`specs/reorg/` reachability anchors.

## Files

This ADR commissions and justifies the following born paths (the producer's justification resolver
maps each tracked path mentioned here to `ADR-0599`):

`comms/ports/calendar-api/BUCK`,
`comms/ports/calendar-api/Cargo.toml`,
`comms/ports/calendar-api/src/lib.rs`,
`comms/core/calendar-usecase/BUCK`,
`comms/core/calendar-usecase/Cargo.toml`,
`comms/core/calendar-usecase/src/lib.rs`,
`registry/catalog/comms-calendar-api.yaml`,
`registry/catalog/comms-calendar-usecase.yaml`,
`specs/reorg/calendar-move-plan.json`.

The relocated domain crate `comms/core/calendar-domain/{BUCK,Cargo.toml,src/lib.rs}` and its re-keyed
catalog record `registry/catalog/comms-calendar-domain.yaml` are rename-relabeled in the firewall
baseline by the move-manifest (ADR-0563), not re-justified here.

This ADR also justifies the bounded calendar workplace contract replay evidence slice that keeps
PRD-CALENDAR AC-01..AC-05 source-locked without claiming runtime promotion, deployment, UI
readiness, production/GA readiness, or customer availability. The producer's justification resolver
maps each tracked path mentioned here to `ADR-0599`:

`specs/fixtures/calendar-prd/calendar_prd_replay_check.py`,
`specs/fixtures/calendar-prd/red-fixtures.json`,
`specs/fixtures/calendar-prd/replay/ac/calendar-ac01-work-event-org-pillar-audit.fixture.json`,
`specs/fixtures/calendar-prd/replay/ac/calendar-ac02-personal-detail-projection.fixture.json`,
`specs/fixtures/calendar-prd/replay/ac/calendar-ac03-action-card-workflow-handoff.fixture.json`,
`specs/fixtures/calendar-prd/replay/ac/calendar-ac04-legal-hold-preservation.fixture.json`,
`specs/fixtures/calendar-prd/replay/ac/calendar-ac05-jurisdiction-retention-ux.fixture.json`,
`specs/fixtures/calendar-prd/replay/asyncapi/calendar-asyncapi-v1-replay.fixture.json`,
`specs/fixtures/calendar-prd/replay/authority/calendar-inventory-provenance-rejection.fixture.json`,
`specs/fixtures/calendar-prd/replay/authority/calendar-prd-authority-source-lock.fixture.json`,
`specs/fixtures/calendar-prd/replay/boundary/calendar-personal-work-pillar-boundary.fixture.json`,
`specs/fixtures/calendar-prd/replay/contracts/calendar-produced-contracts.fixture.json`,
`specs/fixtures/calendar-prd/replay/openapi/calendar-openapi-v1-replay.fixture.json`,
`specs/fixtures/calendar-prd/replay/policy/calendar-build-parentage.fixture.json`,
`specs/fixtures/calendar-prd/replay/proto/calendar-proto-v1-replay.fixture.json`,
`specs/fixtures/calendar-prd/replay/ux/calendar-browser-accessibility-evidence.fixture.json`.
