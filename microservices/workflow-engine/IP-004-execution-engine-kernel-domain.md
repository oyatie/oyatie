---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-004-execution-engine-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness]
---

# IP-004: oya-workflow-engine-execution-engine-{kernel,domain}

## Intent

Scaffold execution-engine kernel + domain layers. Kernel: port traits (`WorkflowRunStore`, `StepDispatcher`, `RetryPolicyEvaluator`, `SlaTimerStore`, `EphemeralStateStore`) + entities (`WorkflowRun`, `StepExecution`, `RetryAttempt`, `SlaTimer`). Domain: pure retry-backoff math + SLA-timer arithmetic + deterministic step-state arithmetic.

## ChangeSet boundary

Two new crates. Workspace + catalog updates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-execution-engine-kernel/{Cargo.toml,src/{lib,entities,ports,errors}.rs}` | create | port traits + entities; data_class annotated |
| `src/crates/oya-workflow-engine-execution-engine-domain/{Cargo.toml,src/{lib,retry_backoff,sla_timer,step_state}.rs}` | create | pure math; determinism property tests |
| `microservices/workflow-engine/catalog/oya-workflow-engine-execution-engine-{kernel,domain}.yaml` | create | 2 catalog rows |
| `Cargo.toml` (workspace) | update | register 2 crates |

## Code Shape (kernel)

```rust
#[async_trait]
pub trait WorkflowRunStore: Send + Sync + Sealed {
    async fn create(&self, run: &WorkflowRun) -> Result<(), StoreError>;
    async fn load(&self, run_id: &RunId) -> Result<WorkflowRun, StoreError>;
    async fn update_state(&self, run_id: &RunId, new_state: &str, expected_version: u64)
        -> Result<(), StoreError>;  // optimistic concurrency
    async fn save_step(&self, step: &StepExecution) -> Result<(), StoreError>;
}

#[async_trait]
pub trait StepDispatcher: Send + Sync + Sealed {
    async fn dispatch(&self, run_id: &RunId, step_index: u32) -> Result<(), DispatchError>;
}

#[async_trait]
pub trait RetryPolicyEvaluator: Send + Sync + Sealed {
    async fn next_attempt(&self, attempt: &RetryAttempt) -> Result<Option<Duration>, KernelError>;
}

#[async_trait]
pub trait SlaTimerStore: Send + Sync + Sealed {
    async fn arm(&self, timer: &SlaTimer) -> Result<(), StoreError>;
    async fn cancel(&self, timer_id: &TimerId) -> Result<(), StoreError>;
    async fn fire_expired(&self) -> Result<Vec<SlaTimer>, StoreError>;
}

#[async_trait]
pub trait EphemeralStateStore: Send + Sync + Sealed {
    async fn claim_step_lease(&self, run_id: &RunId, step_index: u32, worker_id: &WorkerId, ttl: Duration)
        -> Result<bool, StoreError>;
    async fn release_lease(&self, run_id: &RunId, step_index: u32, worker_id: &WorkerId)
        -> Result<(), StoreError>;
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_retry_backoff_exponential` | property: backoff(n) ≥ backoff(n-1); bounded |
| `test_retry_backoff_deterministic` | property: same attempt → same delay |
| `test_sla_timer_arithmetic` | timer expiry computation deterministic |
| `test_step_state_transition_pure` | no side effects in domain logic |

## Next IP

[`IP-005-execution-engine-usecase-durable-execution.md`](IP-005-execution-engine-usecase-durable-execution.md)

## References

- PRD §"Bounded Contexts" execution-engine row
- `policy/spec-integrity.md` (forbidden constructs in step bodies)
