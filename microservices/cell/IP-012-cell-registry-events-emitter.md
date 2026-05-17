---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-012-cell-registry-events-emitter
status: pending
owner: axis-cell-substrate
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, asyncapi-conformance]
---

# IP-012: AsyncAPI event surface emission (Workflow bus integration)

## Intent

Wire `CellEventEmitter` port impls to emit events conforming to `contracts/asyncapi/cell-events.yaml` to the workflow bus. Adds audit-chain Ed25519 signing on each event.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-cell-registry-adapter/src/event_emitter.rs` | create |
| Event-signing helper (shared crate or inline) | create |
| Catalog row update | update |
| AsyncAPI lane check spec | update |

## Code Shape

```rust
// adapter/src/event_emitter.rs
pub struct WorkflowBusEventEmitter {
    bus: amqp::Channel,
    signing_key: Ed25519SigningKey,
}

#[async_trait]
impl CellEventEmitter for WorkflowBusEventEmitter {
    async fn emit_cell_assigned(&self, payload: &CellAssignedPayload) -> Result<(), KernelError> {
        let mut signed = payload.clone();
        signed.signature = self.signing_key.sign(&serde_json::to_vec(&payload)?)?;
        self.bus.basic_publish(
            "workflow-events",
            "cell.assigned",
            amqp::BasicPublishOptions::default(),
            &serde_json::to_vec(&signed)?,
            amqp::BasicProperties::default(),
        ).await?;
        Ok(())
    }
    // ... emit_cell_rebalanced, emit_cell_lifecycle_transition, etc.
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-cell-cell-registry-adapter --test event_emitter
cargo run -p oya-dev-cli -- gate validate asyncapi-conformance --microservice cell
```

## Test Plan

- Unit: each event payload schema-conforms to `cell-events.yaml`.
- Integration: emit + consume cycle on test RabbitMQ; verify Ed25519 signature.
- E2E: tenancy + observability consume events end-to-end.

## Halt Conditions

- Event payload diverges from AsyncAPI schema — fix.
- Unsigned event emitted — block.

## Next IP

[`IP-013-observability-slo-manifests.md`](IP-013-observability-slo-manifests.md)

## References

- `microservices/cell/contracts/asyncapi/cell-events.yaml`.
- Bominal ADR-0028 (Ed25519 audit-chain).
- AsyncAPI 3.x spec — `asyncapi.com`.
