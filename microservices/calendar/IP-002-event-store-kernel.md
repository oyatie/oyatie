---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-002-event-store-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-port-location, oya-governance-data-class-coverage]
---

# IP-002: event-store kernel — CalendarEvent + Attendee + RetentionPolicyRef + LegalHoldRef + port traits

## Intent

Author the event-store BC's kernel layer per ADR-0105 13-layer enum.
Defines the canonical types and port traits with zero I/O and zero
business logic. Annotates every field with `#[data_class(...)]` per
the LEAN data-class lane.

## ChangeSet boundary

1 crate (`oya-calendar-event-store-kernel`); ~12 type definitions +
6 port traits + Cedar entity-shape declarations.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-event-store-kernel/Cargo.toml` | create | crate manifest; deps: chrono, serde, ed25519-dalek |
| `microservices/calendar/src/crates/oya-calendar-event-store-kernel/src/lib.rs` | create | crate root + re-exports |
| `microservices/calendar/src/crates/oya-calendar-event-store-kernel/src/entity.rs` | create | `CalendarEvent`, `Attendee`, `EventContext{Personal,Professional}`, `RetentionPolicyRef`, `LegalHoldRef` |
| `microservices/calendar/src/crates/oya-calendar-event-store-kernel/src/ports.rs` | create | `EventRepository`, `ResourceRepository`, `LegalHoldStore`, `RetentionPolicyResolver`, `TimeZoneResolver`, `EventContextBoundaryGuard` |
| `microservices/calendar/src/crates/oya-calendar-event-store-kernel/src/error.rs` | create | `EventStoreError` variant enum (preserve order per Hyrum #1 in `migration-from-connect.md`) |
| `microservices/calendar/src/crates/oya-calendar-event-store-kernel/src/data_class.rs` | create | `#[data_class]` macro re-export from `oya-shared-data-class` |

## Crate Naming

`oya-calendar-event-store-kernel` — per PRD §"Bounded Contexts" row 1
+ ADR-0056 v4.1 + ADR-0105.

## Code Shape

```rust
// src/entity.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub event_id: EventId,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub tenant_id: TenantId,
    pub context: EventContext,
    #[data_class(PERSONAL_EVENT_CONTENT, PROFESSIONAL_EVENT_CONTENT)]
    pub title: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub tz: chrono_tz::Tz,
    pub rrule: Option<RecurrenceRuleRef>,
    pub attendees: Vec<Attendee>,
    pub retention_policy_ref: RetentionPolicyRef,
    pub legal_hold_ref: Option<LegalHoldRef>,
}

// src/ports.rs
pub trait EventRepository: Send + Sync {
    fn create(&self, event: CalendarEvent) -> Result<EventId, EventStoreError>;
    fn update(&self, event: CalendarEvent) -> Result<EventId, EventStoreError>;
    fn cancel(&self, event_id: EventId) -> Result<(), EventStoreError>;
    fn read(&self, event_id: EventId) -> Result<CalendarEvent, EventStoreError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-calendar-event-store-kernel
cargo clippy -p oya-calendar-event-store-kernel -- -D warnings
cargo nextest run -p oya-calendar-event-store-kernel
cargo run -p oya-dev-cli -- gate validate port-location --microservice calendar
cargo run -p oya-dev-cli -- gate validate data-class-coverage --microservice calendar
```

## Test Plan

- Property tests on `EventContext` discriminated-union exhaustiveness.
- Trait object compile-checks: every port trait is `dyn`-safe.
- Data-class annotation coverage: every field on every type has a
  `#[data_class(...)]` annotation or is documented as
  `INTERNAL_ONLY`.

## Halt Conditions

- Any port trait imports an I/O or data-layer crate — fail
  `port-location` gate.
- Any field lacks `#[data_class]` — fail `data-class-coverage` gate.

## Next IP

[`IP-003-event-store-domain-and-usecase.md`](IP-003-event-store-domain-and-usecase.md)

## References

- ADR-0105 (13-layer enum); ADR-0106 (usecase rename); ADR-0131.
- ADR-CAL-0002 (RRULE engine; references `RecurrenceRuleRef`).
- PRD-calendar §"Bounded Contexts" + §"Port traits declared in each kernel".
