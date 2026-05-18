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
acceptance_lanes: [cargo-nextest, oya-governance-layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: invitation-flow — kernel + domain + usecase + adapter + worker (RFC 5546 + RFC 6047 + RFC 6638)

## Intent

Implement the invitation-flow BC per PRD §"Bounded Contexts" row 5.
RFC 5546 iTIP REQUEST/REPLY/COUNTER/CANCEL/REFRESH; RFC 6047 iMIP
(mail bridge); RFC 6638 auto-scheduling polls.

## ChangeSet boundary

5 crates: `-kernel`, `-domain`, `-usecase`, `-adapter`, `-worker`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-invitation-flow-kernel/` | create | InvitationDispatcher + RsvpStateMachine port traits |
| `microservices/calendar/src/crates/oya-calendar-invitation-flow-domain/` | create | RFC 5546 + RFC 6638 state-machine invariants |
| `microservices/calendar/src/crates/oya-calendar-invitation-flow-usecase/` | create | send-invitation + receive-reply + run-poll orchestrators |
| `microservices/calendar/src/crates/oya-calendar-invitation-flow-adapter/` | create | delegates external delivery to mail µservice via Workflow |
| `microservices/calendar/src/crates/oya-calendar-invitation-flow-worker/` | create | scheduling-poll convergence worker; RSVP fanout |

## Acceptance Gates

```bash
cargo nextest run -p oya-calendar-invitation-flow-domain -- rfc_5546_state_machine
cargo nextest run -p oya-calendar-invitation-flow-domain -- rfc_6638_scheduling
cargo run -p oya-dev-cli -- gate validate slo --microservice calendar --slo rsvp-fanout-latency
cargo run -p oya-dev-cli -- gate validate slo --microservice calendar --slo scheduling-convergence-latency
```

## Test Plan

- RFC 5546 state-machine — all transitions tested.
- RFC 6638 scheduling polls — convergence + expiry + organiser
  override (Cases A-D per `runbooks/scheduling-poll-deadlock.md`).
- RSVP last-write-wins by `decided_at` (Hyrum #5).
- Loop-detection — empty REPLY refused (per `runbooks/calendar-
  bridge-mail-loop-detection.md`).

## Halt Conditions

- RFC 5546 state-machine test fails — block.
- RFC 6638 convergence deadlock detected — block.

## Next IP

[`IP-009-ics-import-export-and-caldav.md`](IP-009-ics-import-export-and-caldav.md)

## References

- RFC 5545; RFC 5546; RFC 6047; RFC 6638.
- `microservices/calendar/runbooks/scheduling-poll-deadlock.md`.
- `microservices/calendar/runbooks/calendar-bridge-mail-loop-detection.md`.
- `microservices/calendar/slos/rsvp-fanout-latency.openslo.yaml`.
- `microservices/calendar/slos/scheduling-convergence-latency.openslo.yaml`.
