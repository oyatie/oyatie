---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-007-room-booking
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, oya-governance-layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: room-booking — kernel + domain + usecase + adapter + rest

## Intent

Implement the room-booking BC per PRD §"Bounded Contexts" row 4 +
PRD AC-09 (100% double-booking refusal at write time). Resource
graph queries; conflict resolution; recurring booking.

## ChangeSet boundary

5 crates: `-kernel`, `-domain`, `-usecase`, `-adapter`, `-rest`.
Storage is subsumed under the event-store `-adapter-postgres` crate
per PRD §Bounded Contexts table footnote.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-room-booking-kernel/` | create | ResourceRepository + ConflictDecision port traits |
| `microservices/calendar/src/crates/oya-calendar-room-booking-domain/` | create | conflict invariant (PRD AC-09: 100% double-booking refusal) |
| `microservices/calendar/src/crates/oya-calendar-room-booking-usecase/` | create | book-room orchestrator |
| `microservices/calendar/src/crates/oya-calendar-room-booking-adapter/` | create | thin adapter over event-store-adapter-postgres |
| `microservices/calendar/src/crates/oya-calendar-room-booking-rest/` | create | REST handler |

## Acceptance Gates

```bash
cargo nextest run -p oya-calendar-room-booking-domain -- conflict
cargo run -p oya-dev-cli -- gate validate slo --microservice calendar --slo room-conflict-detection-correctness
```

## Test Plan

- PRD AC-09 — concurrent booking writes never produce conflicting rows.
- Recurring booking expansion + conflict check across the expansion window.
- Performance: room conflict check p99 ≤ 100ms.

## Halt Conditions

- PRD AC-09 test fails — Sev-1 block; correctness invariant.

## Next IP

[`IP-008-invitation-flow.md`](IP-008-invitation-flow.md)

## References

- PRD-calendar AC-09.
- `microservices/calendar/slos/room-conflict-detection-correctness.openslo.yaml`.
