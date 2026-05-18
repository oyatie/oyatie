---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-007-supervision-event-bus
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1]
depends_on: [IP-002, IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: supervision-event-bus BC (7 crates)

## Intent

AMQP + Redis Streams substrate; per-event Ed25519 signature; subscriber registration. 7 crates: kernel, usecase, api, adapter, worker, sdk, app.

## Concrete File Targets

Crates at `microservices/foundry/src/crates/oya-foundry-supervisor-supervision-event-bus-{layer}/`.

## Key code

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait SupervisionEventPublisher: Send + Sync + Sealed {
    async fn publish(&self, event: &SupervisionEvent) -> Result<EventId, KernelError>;
}

#[async_trait]
pub trait SupervisionEventSubscriber: Send + Sync + Sealed {
    async fn subscribe(&self, topic: &str, group: &str) -> Result<Box<dyn Stream<Item = Result<SupervisionEvent, KernelError>>>, KernelError>;
}
```

```rust
// adapter/src/redis_streams.rs
// Implements both Publisher (XADD with Ed25519-signed payload) and Subscriber
// (XREADGROUP with at-least-once delivery).
```

## Acceptance Gates

Standard per-crate gates. Plus:

```bash
# End-to-end: publish synthetic event; verify foundry-evidence sealed; verify audit-chain Merkle.
cargo nextest run -p oya-foundry-supervisor-supervision-event-bus-worker --test e2e_publish_and_seal
```

## Halt Conditions

- Event missing Ed25519 signature.
- At-least-once delivery semantics violated.

## Next IP

[`IP-008-kill-switch-engage-state.md`](IP-008-kill-switch-engage-state.md)

## References

- PRD FR-06; `contracts/asyncapi/foundry-supervisor-events.yaml`.
- ADR-0028 (audit-chain).
- Redis Streams — `redis.io/docs/data-types/streams/`.
