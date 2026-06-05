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
acceptance_lanes: [cargo-nextest, room-conflict-test, oya-governance-layer-correctness]
---

# IP-007: Room booking

## A. Problem
Enterprise calendar parity requires resource booking with deterministic double-booking refusal, not only user events.

## B. Approach
Implement the `oya-calendar-room-booking-kernel` bounded context and its planned domain/usecase/adapter/rest layers using the crate names already specified by the PRD/IP. Room booking reads resource state through ports, writes booking decisions through event-store transactions, and emits room conflict events.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-calendar-room-booking-kernel.yaml` | Existing kernel catalog anchor. |
| `src/crates/oya-calendar-room-booking-kernel/` | Planned crate path named by manifest/catalog. |
| `src/crates/oya-calendar-room-booking-{domain,usecase,adapter,rest}/` | Planned paths already named by this IP and PRD. |
| `slos/room-conflict-detection-correctness.openslo.yaml` | Correctness SLO for refusal behavior. |
| `runbooks/room-booking-conflict.md` | Operational closure for booking conflicts. |

## D. Ordered implementation steps
1. Define `Resource`, `Booking`, `ConflictDecision`, and booking-window types.
2. Implement conflict detection for one-off and recurring bookings.
3. decisions to event-store transaction boundaries.
4. Emit `RoomBooked` and `RoomBookingConflict` events from `contracts/asyncapi/calendar-events.yaml`.
5. Add tests for simultaneous contenders and recurring room reservations.
6. Add REST contract tests against `contracts/openapi/calendar.yaml`.
7. Wire SLO metrics and room-conflict runbook triggers.

## E. Acceptance
- `cargo nextest run -p oya-calendar-room-booking-kernel` passes.
- Room double-booking tests prove one winner and deterministic losers under concurrency.
- `buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice calendar` passes.
- `slos/room-conflict-detection-correctness.openslo.yaml` resolves.
- `runbooks/room-booking-conflict.md` includes rollback and operator evidence steps.

## F. Evidence
- PRD FR-04 and room performance target: `microservices/calendar/PRD.md`.
- Contracts: `contracts/openapi/calendar.yaml`, `contracts/asyncapi/calendar-events.yaml`.
- Catalog: `catalog/oya-calendar-room-booking-kernel.yaml`.
- Benchmark: `benchmarks/gcal-outlook-calendly-vs-oyatie.md`.

## G. Counterpart comparison
Outlook room resources and Google resource calendars are the main counterparts. Calendly and Cal.com book time but do not provide enterprise resource graphs. Oyatie must match Outlook/Google conflict behavior and make the proof tenant-visible through the room-conflict correctness SLO.

## H. Foundation delivery expansion
- Deliverable detail: define resource identity, room capacity, amenity tags, location, and booking policy fields.
- Deliverable detail: represent conflict decisions with deterministic winner/loser evidence.
- Deliverable detail: handle one-off and recurring reservations through event-store transaction boundaries.
- Deliverable detail: emit room booking and conflict events with idempotency keys.
- Deliverable detail: expose room availability through policy-filtered contract responses.
- Deliverable detail: support operator-visible repair for stuck or duplicated booking rows.
- Deliverable detail: wire conflict counters to the room correctness SLO.
- Deliverable detail: Slack room and huddle integrations are comparison pressure for collaboration-adjacent scheduling.

## I. Acceptance expansion
- Acceptance detail: double-booking tests must prove a single winner under concurrent contenders.
- Acceptance detail: recurring room tests must detect conflicts across expanded occurrence windows.
- Acceptance detail: capacity and amenity tests must reject incompatible room assignments.
- Acceptance detail: event-store transaction tests must prove booking and lifecycle event stay atomic.
- Acceptance detail: REST contract tests must include successful booking and conflict refusal responses.
- Acceptance detail: SLO resolution must include conflict correctness, not only latency.
- Acceptance detail: runbook evidence must show operator rollback and communication steps.
- Acceptance detail: Slack/Google/Outlook comparisons must be about resource booking and collaboration-room parity.

## J. Evidence expansion
- Evidence detail: capture nextest output for room-booking kernel and related crates.
- Evidence detail: capture concurrency fixture names and winner/loser assertions.
- Evidence detail: capture OpenAPI validation for room booking endpoints.
- Evidence detail: cite `runbooks/room-booking-conflict.md` for conflict operations.
- Evidence detail: cite `slos/room-conflict-detection-correctness.openslo.yaml` for promotion criteria.
- Evidence detail: cite `benchmarks/gcal-outlook-calendly-vs-oyatie.md` for resource-booking comparison.
- Evidence detail: cite Slack as collaboration-suite pressure, not as the primary room-resource protocol.
