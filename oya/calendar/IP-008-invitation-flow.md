---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-008-invitation-flow
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, rfc-5546-conformance, mail-handoff-test]
---

# IP-008: Invitation flow

## A. Problem
Calendar invitations must interoperate with external calendar systems while preserving Oyatie tenant policy, RSVP state, audit-chain evidence, and mail-service handoff boundaries.

## B. Approach
Implement `oya-calendar-invitation-flow-kernel` and the planned domain/usecase/adapter/worker layers already named by this IP. The worker emits and consumes RFC 5546 RSVP state changes through Workflow and delegates delivery to the mail microservice rather than importing mail crates.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-calendar-invitation-flow-kernel.yaml` | Existing catalog anchor. |
| `src/crates/oya-calendar-invitation-flow-kernel/` | Planned kernel path named by manifest/catalog. |
| `src/crates/oya-calendar-invitation-flow-{domain,usecase,adapter,worker}/` | Planned paths named by this IP/PRD. |
| `contracts/asyncapi/calendar-events.yaml` | RSVP and lifecycle event contract. |
| `slos/rsvp-fanout-latency.openslo.yaml` | Invitation/RSVP fanout SLO. |

## D. Ordered implementation steps
1. Define invitation, recipient, RSVP, counter-proposal, and delivery-attempt types.
2. Implement RFC 5546 state transitions with idempotent invitation IDs.
3. Bind mail handoff through a Workflow event or port, not a product crate import.
4. Add retry, dead-letter, and audit-correlation behavior in the worker.
5. Test accept, decline, tentative, counter-propose, cancellation, and duplicate reply handling.
6. Add mail-delivery-failed consumer behavior from the PRD workflow table.
7. Wire SLO and runbook evidence for RSVP storms.

## E. Acceptance
- `cargo nextest run -p oya-calendar-invitation-flow-kernel` passes.
- RFC 5546/iTIP fixtures pass for accept, decline, counter, and cancel.
- `cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice calendar` passes.
- `slos/rsvp-fanout-latency.openslo.yaml` resolves.
- `runbooks/rsvp-storm-throttle.md` covers throttle and recovery.

## F. Evidence
- PRD FR-05 and FR-06: `microservices/calendar/PRD.md`.
- AsyncAPI: `contracts/asyncapi/calendar-events.yaml`.
- Policy: `policy/tenant-scope.cedar`.
- Runbooks: `runbooks/rsvp-storm-throttle.md`, `runbooks/calendar-bridge-mail-loop-detection.md`.

## G. Counterpart comparison
Google, Outlook, Apple, Fastmail, and Proton all support invitation and RSVP flows; Calendly and Cal.com cover booking confirmations but not full iTIP semantics. Oyatie's target is full interop plus audit-chain and mail-loop protection, not a narrower booking notification.

## H. Foundation delivery expansion
- Deliverable detail: define invitation, recipient, organizer, RSVP, counter-proposal, and delivery-attempt records.
- Deliverable detail: use RFC 5546/iTIP transitions for accept, decline, tentative, counter, and cancel.
- Deliverable detail: delegate outbound delivery to mail/workflow ports instead of importing mail crates.
- Deliverable detail: persist delivery attempt idempotency and dead-letter metadata.
- Deliverable detail: consume mail failure signals without creating a calendar-mail retry loop.
- Deliverable detail: emit RSVP lifecycle events with audit correlation.
- Deliverable detail: expose throttling metrics for RSVP storms.
- Deliverable detail: Slack meeting reminders and channel RSVP conventions are collaboration pressure for fanout behavior.

## I. Acceptance expansion
- Acceptance detail: fixture tests must cover accept, decline, tentative, counter-propose, cancel, and duplicate reply.
- Acceptance detail: idempotency tests must prove duplicate RSVP messages do not duplicate lifecycle events.
- Acceptance detail: mail-loop tests must prove calendar does not re-trigger failed mail endlessly.
- Acceptance detail: fanout SLO checks must include delivered, delayed, failed, and dead-lettered counts.
- Acceptance detail: layer gates must prove no direct mail product crate import.
- Acceptance detail: policy tests must preserve tenant/context boundaries for external invitees.
- Acceptance detail: runbook coverage must include RSVP storm throttle and bridge-loop detection.
- Acceptance detail: Slack/Google/Outlook comparisons must be supported by full RSVP state tests.

## J. Evidence expansion
- Evidence detail: capture nextest output for invitation-flow kernel and worker crates.
- Evidence detail: capture AsyncAPI validation for RSVP events.
- Evidence detail: capture mail-loop fixture output.
- Evidence detail: cite `runbooks/rsvp-storm-throttle.md` for burst response.
- Evidence detail: cite `runbooks/calendar-bridge-mail-loop-detection.md` for mail-loop safety.
- Evidence detail: cite `slos/rsvp-fanout-latency.openslo.yaml` for fanout criteria.
- Evidence detail: cite Slack as collaboration-calendar notification pressure alongside Google and Outlook RSVP behavior.
