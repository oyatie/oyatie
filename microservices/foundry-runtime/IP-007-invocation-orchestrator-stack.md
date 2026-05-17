---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-007-invocation-orchestrator-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

# IP-007: oya-foundry-runtime-invocation-orchestrator stack

## Intent

The full invocation-orchestrator BC: kernel + domain + usecase + api + adapter + worker + app. Implements `LifecycleStore` + `EventEmitter` + `TimeoutClock` + `CancellationSignal`. Drives one invocation start → complete with idempotency keys; emits AsyncAPI events; enforces timeout per descriptor.

## ChangeSet boundary

7 new Rust crates.

## Concrete File Targets

Per layer crate at `microservices/foundry-runtime/src/crates/oya-foundry-runtime-invocation-orchestrator-<layer>/`:
- kernel: entities (InvocationLifecycle, OrchestratorVerdict, CancellationToken) + ports + errors
- domain: lifecycle-state-machine validity (pending → running → completed | failed | cancelled) + deadline arithmetic
- usecase: orchestrator + idempotency-key dedup + cancellation propagation
- api: typed contracts
- adapter: in-memory lifecycle store backed by Redis hot index + Postgres durable + AsyncAPI emitter
- worker: timeout monitor + idempotent re-emission for downstream delivery guarantees
- app: composition root

## Crate Naming

All crates follow `oya-foundry-runtime-invocation-orchestrator-<layer>`.

## Code Shape

```rust
// usecase/src/orchestrator.rs
pub struct InvocationOrchestrator<L, E, T, C> {
    lifecycle_store: L,
    event_emitter: E,
    timeout_clock: T,
    cancellation_signal: C,
}

impl<L: LifecycleStore, E: EventEmitter, T: TimeoutClock, C: CancellationSignal>
    InvocationOrchestrator<L, E, T, C>
{
    pub async fn execute(&self, invocation: Invocation, timeout: Duration) -> Result<InvocationLifecycle, OrchestratorError> {
        // Check idempotency key
        if let Some(existing) = self.lifecycle_store.lookup_by_idempotency(&invocation.idempotency_key).await? {
            return Ok(existing);
        }

        let lifecycle = self.lifecycle_store.create(&invocation).await?;
        self.event_emitter.emit_invocation_started(&lifecycle).await?;

        // Race: dispatch vs timeout vs cancellation
        let result = tokio::select! {
            result = self.dispatch(&lifecycle) => result,
            _ = self.timeout_clock.sleep(timeout) => Err(OrchestratorError::Timeout),
            _ = self.cancellation_signal.wait(&lifecycle.invocation_id) => Err(OrchestratorError::Cancelled),
        };

        // Emit terminal lifecycle event
        match &result {
            Ok(invocation) => self.event_emitter.emit_invocation_completed(invocation).await?,
            Err(e) => self.event_emitter.emit_invocation_failed(&lifecycle, e).await?,
        }
        Ok(lifecycle)
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-invocation-orchestrator-{kernel,domain,usecase,api,adapter,worker,app}
cargo nextest run -p oya-foundry-runtime-invocation-orchestrator-{kernel,domain,usecase}
cargo nextest run -p oya-foundry-runtime-invocation-orchestrator-worker --features testcontainers
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_lifecycle_state_machine_validity` | only valid transitions accepted |
| `test_idempotency_key_dedup` | repeated key → existing lifecycle returned |
| `test_timeout_emits_failed` | timeout → InvocationFailed{reason=timeout} |
| `test_cancellation_propagates` | cancel signal → InvocationCancelled |
| `test_event_emission_idempotent` | re-running on crashed orchestrator does not double-emit |
| `test_deadline_arithmetic_pure` | domain layer math correctness |

## Halt Conditions

- Lifecycle state machine permits invalid transitions — refactor.
- Event emission not idempotent — refactor (could cause duplicate downstream effects).

## Next IP

[`IP-008-runtime-pool-stack.md`](IP-008-runtime-pool-stack.md)

## References

- ADR-0025; ADR-0105.
- `contracts/asyncapi/foundry-runtime-events.yaml`.
- `failure-modes.md` FM-15 (timeout); FM-08 (sibling unreachable).
