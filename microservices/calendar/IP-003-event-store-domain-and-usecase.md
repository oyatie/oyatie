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
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-layer-correctness, oya-governance-port-location]
---

# IP-003: event-store domain + usecase — invariants and orchestrators

## Intent

Author the event-store BC's `domain` (pure invariant math: overlap
checking, time-zone arithmetic, legal-hold coverage, retention
boundary calculations) and `usecase` (orchestrators: create-event,
update-event, cancel-event, apply-legal-hold, expire-retention) per
ADR-0105 13-layer + ADR-0106 usecase rename.

## ChangeSet boundary

2 crates (`-domain` + `-usecase`); domain has zero I/O; usecase
reads via ports only.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-event-store-domain/Cargo.toml` | create | crate manifest |
| `microservices/calendar/src/crates/oya-calendar-event-store-domain/src/lib.rs` | create | crate root |
| `microservices/calendar/src/crates/oya-calendar-event-store-domain/src/overlap.rs` | create | half-open `[start, end)` overlap invariant (Hyrum #2) |
| `microservices/calendar/src/crates/oya-calendar-event-store-domain/src/legal_hold.rs` | create | legal-hold coverage invariants (PRD AC-06) |
| `microservices/calendar/src/crates/oya-calendar-event-store-domain/src/retention.rs` | create | per-pack retention floor invariants |
| `microservices/calendar/src/crates/oya-calendar-event-store-domain/src/context_isolation.rs` | create | dual-context invariant (Personal ↮ Professional) |
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/Cargo.toml` | create | crate manifest |
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/src/lib.rs` | create | crate root |
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/src/create_event.rs` | create | CreateEvent orchestrator |
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/src/update_event.rs` | create | UpdateEvent orchestrator |
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/src/cancel_event.rs` | create | CancelEvent orchestrator |
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/src/apply_legal_hold.rs` | create | ApplyLegalHold orchestrator |
| `microservices/calendar/src/crates/oya-calendar-event-store-usecase/src/expire_retention.rs` | create | ExpireRetention orchestrator |

## Acceptance Gates

```bash
cargo nextest run -p oya-calendar-event-store-domain
cargo nextest run -p oya-calendar-event-store-usecase
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice calendar
```

## Test Plan

- Property tests on `overlap.rs` — half-open `[start, end)` is
  associative + commutative.
- Named legal-hold tests per PRD AC-06.
- Context isolation tests per PRD AC-07: Personal-context attempt
  to read Professional-context event must fail with `403 + audit-
  emit` (matches `oya-mail-dual-context-isolation` test pattern).

## Halt Conditions

- Any domain function does I/O — fail `layer-correctness` gate.
- Any usecase imports a non-port type from an adapter crate — fail
  `port-location` gate.

## Next IP

[`IP-004-event-store-adapter-postgres.md`](IP-004-event-store-adapter-postgres.md)

## References

- ADR-0105 (13-layer enum); ADR-0106 (usecase rename).
- PRD-calendar AC-06 + AC-07.
- `microservices/mail/IP-002-mailbox-store-kernel.md` — sibling reference.
