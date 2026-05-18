---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-013-workflow-handoff
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + axis-workflow
acceptance_lanes: [cargo-nextest, oya-governance-cross-product-isolation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: Workflow + Ontology handoff (cross-µservice integration)

## Intent

Wire calendar's Workflow event production + Ontology entity writes
per PRD §"Workflow events produced" + §"Ontology writes". Calendar
MUST NOT directly import another product µservice crate; all
cross-µservice flows go through Workflow (events) or Ontology
(entity reads/writes).

## ChangeSet boundary

Per-BC Workflow event emit calls + Ontology entity writes; no new
crates (handoff is a wiring layer inside existing usecase + worker
crates).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/src/workflow_emit.rs` | create | emit calendar.event.lifecycle.v1 events |
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/src/ontology_write.rs` | create | write Calendar.CalendarEvent + Calendar.LegalHold entities |
| `microservices/calendar/src/crates/oya-calendar-invitation-flow-worker/src/workflow_emit.rs` | create | emit calendar.invitation.rsvp.v1 events |
| `microservices/calendar/src/crates/oya-calendar-room-booking-usecase/src/workflow_emit.rs` | create | emit calendar.room.booking.v1 events |
| `microservices/calendar/tests/cross-product-isolation.rs` | create | LEAN-A2 cross-product refusal coverage |

## Acceptance Gates

```bash
cargo nextest run -p tests --test cross_product_isolation
cargo run -p oya-dev-cli -- gate validate cross-product-isolation --microservice calendar
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice calendar
```

## Test Plan

- Cross-product import refusal — calendar crates do NOT import
  oya-mail-*, oya-messenger-*, etc. directly.
- Workflow event coverage — every PRD §"Workflow events produced"
  row has a corresponding emit call.
- Ontology entity coverage — every PRD §"Ontology writes" row has
  a corresponding entity write.

## Halt Conditions

- Any direct cross-product crate import — block.
- Any PRD-listed Workflow event without an emit call — block.

## Next IP

[`IP-014-hg-calendar-authority-cohesion.md`](IP-014-hg-calendar-authority-cohesion.md)

## References

- `feedback_workflow_objectgraph_adapter_layer.md` — Workflow +
  Ontology = adapter layer.
- ADR-0131 (per-µservice flat layout); ADR-0132.
- LEAN-A2 cross-product CI lane.
