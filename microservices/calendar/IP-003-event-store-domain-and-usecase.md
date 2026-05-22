---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-003-event-store-domain-and-usecase
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, oya-governance-layer-correctness, oya-governance-legal-hold]
---

# IP-003: Event-store domain and usecase

## A. Problem
Calendar needs event mutation rules that are testable without Postgres or REST. Without a pure domain/usecase split, Google/Outlook-style event CRUD can leak tenant data, shift retained events, or bypass legal hold.

## B. Approach
Implement `oya-calendar-event-store-domain` and `oya-calendar-event-store-usecase` as the invariant and orchestration layer over the kernel ports. Domain owns overlap, attendee-state transition, legal-hold, recurrence-reference, and context-boundary rules; usecase owns create/update/cancel/apply-hold/expire-retention workflows.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-calendar-event-store-domain.yaml` | Existing domain catalog record. |
| `catalog/oya-calendar-event-store-usecase.yaml` | Existing usecase catalog record. |
| `src/crates/oya-calendar-event-store-domain/` | Planned pure-rule crate named by manifest/catalog. |
| `src/crates/oya-calendar-event-store-usecase/` | Planned orchestrator crate named by manifest/catalog. |
| `contracts/asyncapi/calendar-events.yaml` | Event lifecycle output contract for usecase emissions. |

## D. Ordered implementation steps
1. Create overlap and event-time invariants using explicit IANA time-zone input.
2. Implement attendee-state transition rules for invited, accepted, declined, tentative, and counter-proposed states.
3. Add legal-hold mutation refusal and retention-expiry protections.
4. Implement usecases through kernel ports only.
5. Emit lifecycle events with idempotency keys matching `calendar.event.lifecycle.v1`.
6. Add tests for legal-hold refusal, context isolation, attendee transition legality, and idempotent cancellation.
7. Run layer-correctness and cross-product dependency gates before promotion.

## E. Acceptance
- `cargo nextest run -p oya-calendar-event-store-domain` passes.
- `cargo nextest run -p oya-calendar-event-store-usecase` passes.
- `cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice calendar` passes.
- `cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice calendar` passes.
- Legal-hold behavior matches `policy/data-residency.md` and `compliance.md`.

## F. Evidence
- PRD FR-01, FR-12, FR-13, and workflow event table: `microservices/calendar/PRD.md`.
- AsyncAPI lifecycle topics: `microservices/calendar/contracts/asyncapi/calendar-events.yaml`.
- Runbooks: `runbooks/calendar-restore.md`, `runbooks/shared-cal-permission-drift.md`.
- SLOs: `slos/agenda-render-latency.openslo.yaml`, `slos/notification-delivery-freshness.openslo.yaml`.

## G. Counterpart comparison
Google Calendar and Outlook match event CRUD depth, while Apple and Fastmail prove strict recurrence and shared calendar expectations. Oyatie's domain/usecase layer must meet that baseline and exceed it with legal-hold, audit-chain idempotency, and dual-context refusal tests that counterparts expose only as administrative policy.

## H. Foundation delivery expansion
- Deliverable detail: domain owns overlap, time-window, attendee transition, cancellation, and hold invariants.
- Deliverable detail: usecase owns create, update, cancel, invite-state apply, retention-expire, and legal-hold workflows.
- Deliverable detail: every command carries tenant, context, actor, idempotency key, and audit correlation.
- Deliverable detail: recurrence references remain references; expansion stays in the recurrence engine.
- Deliverable detail: lifecycle events map to `calendar.event.lifecycle.v1` examples.
- Deliverable detail: cancellation and legal-hold paths include refusal reasons safe for audit logs.
- Deliverable detail: tests cover organizer, attendee, external invitee, auditor, and CI actors.
- Deliverable detail: Slack meeting-channel workflows are comparison pressure for collaboration state transitions.

## I. Acceptance expansion
- Acceptance detail: overlap tests must cover DST transitions and explicit IANA zone input.
- Acceptance detail: legal-hold tests must prove updates and deletes are refused while reads remain policy-scoped.
- Acceptance detail: idempotency tests must show duplicate create/update/cancel commands return stable results.
- Acceptance detail: attendee-state tests must reject impossible transitions such as declined-to-countered without new invite.
- Acceptance detail: emitted events must validate against AsyncAPI examples.
- Acceptance detail: lean/layer gates must prove no adapter imports in domain/usecase.
- Acceptance detail: dual-context tests must prove personal and professional events never share mutable state.
- Acceptance detail: Slack, Google, and Outlook comparisons must be backed by event-state tests.

## J. Evidence expansion
- Evidence detail: capture nextest output for domain and usecase crates.
- Evidence detail: capture AsyncAPI validation for lifecycle event examples.
- Evidence detail: capture legal-hold refusal fixture names in the evidence bundle.
- Evidence detail: cite `runbooks/calendar-restore.md` for event replay expectations.
- Evidence detail: cite `runbooks/shared-cal-permission-drift.md` for permission correction behavior.
- Evidence detail: cite SLO files when lifecycle command latency is measured.
- Evidence detail: cite Slack as the collaboration workflow counterpart that increases RSVP/channel handoff pressure.
