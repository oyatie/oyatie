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
acceptance_lanes: [asyncapi-validate, ontology-link-test, oya-governance-cross-product-boundary]
---

# IP-013: Workflow and Ontology handoff

## A. Problem
Calendar integrates with tenancy, mail, messenger, workflow-engine, audit-chain, and observability, but direct product-crate imports would violate the microservice boundary.

## B. Approach
Use AsyncAPI events and Ontology object/link writes as the only handoff surfaces. Calendar emits lifecycle, RSVP, room, recurrence, and legal-hold events, consumes tenant/mail/messenger/workflow signals, and writes calendar objects with audit correlation.

## C. Deliverables
| Artifact | Role |
|---|---|
| `contracts/asyncapi/calendar-events.yaml` | Produced/consumed workflow event source. |
| `contracts/proto/calendar.proto` | Shared typed payload source. |
| `manifest.json` | Dependency and contract registry. |
| `backfill-replay.md` | Replay and catch-up operational rule. |
| `runbooks/calendar-bridge-mail-loop-detection.md` | Mail handoff loop guard. |

## D. Ordered implementation steps
1. Map every PRD produced and consumed event to an AsyncAPI topic.
2. Add idempotency keys and audit-correlation fields to event examples.
3. Document ontology writes for `CalendarEvent`, `Resource`, `Booking`, `Invitation`, and `LegalHold`.
4. Test event consumers with synthetic tenancy, mail failure, and messenger room creation inputs.
5. Add cross-product dependency scan proving no direct product crate imports.
6. Add backfill replay evidence for missed lifecycle events.
7. Tie observability metrics to SLO dashboards.

## E. Acceptance
- AsyncAPI validation passes for `contracts/asyncapi/calendar-events.yaml`.
- `buck2 build //:quality-lane-registry-authority-check # lane=lean-a2 --microservice calendar` passes.
- `buck2 build //:quality-lane-registry-authority-check # lane=ontology-link-consistency --microservice calendar` passes.
- Replay instructions in `backfill-replay.md` cover idempotent lifecycle events.
- Mail-loop detection runbook remains linked.

## F. Evidence
- PRD workflow and ontology sections: `microservices/calendar/PRD.md`.
- Contracts: `contracts/asyncapi/calendar-events.yaml`, `contracts/proto/calendar.proto`.
- Runbooks: `backfill-replay.md`, `runbooks/calendar-bridge-mail-loop-detection.md`, `runbooks/scheduling-poll-deadlock.md`.
- Manifest dependencies: `manifest.json`.

## G. Counterpart comparison
Google and Microsoft expose webhooks/subscriptions, and Cal.com/Calendly expose booking webhooks. Oyatie must match event-driven integration while adding typed ontology links, audit-chain correlation, and cross-product import refusal as governance evidence.

## H. Foundation delivery expansion
- Deliverable detail: produced events include event lifecycle, RSVP, room booking, recurrence refusal, legal hold, and import job state.
- Deliverable detail: consumed events include tenant changes, mail delivery state, messenger room creation, and workflow retry signals.
- Deliverable detail: ontology writes include `CalendarEvent`, `Resource`, `Booking`, `Invitation`, and `LegalHold` links.
- Deliverable detail: idempotency keys are stable across replay and backfill.
- Deliverable detail: audit correlation survives consumer retries and dead-letter movement.
- Deliverable detail: direct product crate imports are forbidden; only contracts and ports are allowed.
- Deliverable detail: dashboards distinguish produced, consumed, replayed, and dead-lettered events.
- Deliverable detail: Slack workflow handoff expectations are counterpart pressure for event clarity and retry semantics.

## I. Acceptance expansion
- Acceptance detail: AsyncAPI validation must cover every produced and consumed topic.
- Acceptance detail: replay tests must prove idempotent lifecycle reconstruction.
- Acceptance detail: ontology-link tests must prove object/link names match the PRD.
- Acceptance detail: cross-product dependency scan must fail on direct mail, messenger, workflow, or ontology crate imports.
- Acceptance detail: mail failure consumer tests must avoid calendar-mail retry loops.
- Acceptance detail: backfill documentation must name checkpoint, resume, and duplicate-handling behavior.
- Acceptance detail: SLO dashboard checks must include event lag and dead-letter count.
- Acceptance detail: Slack, GitHub, and Linear-style workflow consumers must integrate through events rather than private imports.

## J. Evidence expansion
- Evidence detail: capture AsyncAPI validator output.
- Evidence detail: capture ontology-link consistency gate output.
- Evidence detail: capture cross-product dependency scan output.
- Evidence detail: cite `backfill-replay.md` for replay controls.
- Evidence detail: cite `runbooks/calendar-bridge-mail-loop-detection.md` for loop prevention.
- Evidence detail: cite `runbooks/scheduling-poll-deadlock.md` for workflow degradation.
- Evidence detail: cite Slack as workflow/collaboration counterpart pressure for typed handoff contracts.
