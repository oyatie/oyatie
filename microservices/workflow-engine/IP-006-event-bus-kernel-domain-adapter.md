---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-006-event-bus-kernel-domain-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-workflow-engine-event-bus-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis}

## Intent

Typed workflow-event publish/subscribe + outbox + replay-from-offset + backpressure. Postgres outbox = durable; Redis = ephemeral subscription state. Kernel: port traits + entities. Domain: pure event serialization + offset arithmetic.

## ChangeSet boundary

7 new crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-event-bus-kernel/{...}` | create | `WorkflowEvent`, `EventOffset`, `Subscription` entities + `EventBus`, `OutboxRelay` port traits |
| `src/crates/oya-workflow-engine-event-bus-domain/{...}` | create | pure serialization + offset arithmetic + Ed25519 envelope signing |
| `src/crates/oya-workflow-engine-event-bus-usecase/{...}` | create | publish + subscribe orchestrators |
| `src/crates/oya-workflow-engine-event-bus-api/{...}` | create | typed I/O |
| `src/crates/oya-workflow-engine-event-bus-adapter/{...}` | create | protocol-neutral impls |
| `src/crates/oya-workflow-engine-event-bus-adapter-postgres/{Cargo.toml,src/lib.rs,migrations/V1__outbox_schema.sql}` | create | Outbox pattern: append-only outbox table; INSERT trigger; UPDATE/DELETE refused |
| `src/crates/oya-workflow-engine-event-bus-adapter-redis/{Cargo.toml,src/lib.rs}` | create | Subscription registry + delivery state |
| `microservices/workflow-engine/catalog/oya-workflow-engine-event-bus-*.yaml` | create | 7 catalog rows |
| `Cargo.toml` (workspace) | update | register |

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait EventBus: Send + Sync + Sealed {
    async fn publish(&self, event: &WorkflowEvent, opts: PublishOpts) -> Result<EventId, BusError>;
    async fn subscribe(&self, filter: EventFilter, opts: SubscribeOpts) -> Result<EventStream, BusError>;
    async fn replay_from_offset(&self, sub_id: &SubId, from: EventOffset, to: EventOffset)
        -> Result<(), BusError>;
}

#[async_trait]
pub trait OutboxRelay: Send + Sync + Sealed {
    async fn next_batch(&self, max_size: usize) -> Result<Vec<OutboxRow>, BusError>;
    async fn mark_published(&self, offset: EventOffset) -> Result<(), BusError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-workflow-engine-event-bus-domain --all-features
cargo nextest run -p oya-workflow-engine-event-bus-adapter-postgres --all-features
cargo run -p oya-dev-cli -- gate validate outbox-append-only --crate oya-workflow-engine-event-bus-adapter-postgres
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_event_envelope_serde_roundtrip` | canonicalized form preserved |
| `test_envelope_signature_tampering_detected` | Ed25519 mismatch refused |
| `test_offset_monotonic_per_tenant_run` | sequence per (tenant, run) increases |
| `test_outbox_insert_only_no_update` | Postgres trigger refuses UPDATE/DELETE |
| `test_replay_idempotency` | same offset range replayed twice → subscriber sees consistent set |
| `test_subscription_tenant_isolation` | tenant-A subscription cannot read tenant-B events |

## Next IP

[`IP-007-event-bus-rest-worker-sdk-app.md`](IP-007-event-bus-rest-worker-sdk-app.md)

## References

- PRD FR-03, FR-04, FR-13
- Postgres outbox pattern — `microservices.io/patterns/data/transactional-outbox.html`
- AsyncAPI contracts at `contracts/asyncapi/workflow-events.yaml`
