---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-005-execution-engine-usecase-durable-execution
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-deterministic-replay]
---

# IP-005: oya-workflow-engine-execution-engine-{usecase,api,adapter,adapter-postgres,adapter-redis}

## Intent

The engineering heart of this phase: durable-execution authoritative store (Postgres) + ephemeral lease state (Redis) + usecase orchestrators that compose state-machine transitions + retry + SLA-timer + audit-chain emission. This is where PRD AC-02 (deterministic replay) and AC-03 (durable-execution restart) invariants are realised.

## ChangeSet boundary

5 new crates:
- `oya-workflow-engine-execution-engine-usecase` (orchestrator; depends on state-machine-usecase + spec-store-kernel)
- `oya-workflow-engine-execution-engine-api` (typed I/O)
- `oya-workflow-engine-execution-engine-adapter` (protocol-neutral impls)
- `oya-workflow-engine-execution-engine-adapter-postgres` (durable run state authoritative)
- `oya-workflow-engine-execution-engine-adapter-redis` (ephemeral lease + step claim state)

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-execution-engine-usecase/{...}` | create | RunLifecycleOrchestrator, StepLifecycleOrchestrator |
| `src/crates/oya-workflow-engine-execution-engine-api/{...}` | create | RunStartRequest/Response, StepDispatchRequest/Response, error variants |
| `src/crates/oya-workflow-engine-execution-engine-adapter/{...}` | create | protocol-neutral impls |
| `src/crates/oya-workflow-engine-execution-engine-adapter-postgres/{Cargo.toml,src/{lib,run_store,step_store,migrations}.rs,migrations/V1__initial_schema.sql,migrations/V2__add_idempotency.sql,migrations/V3__add_citus_distribution.sql}` | create | Postgres-backed run state + outbox cross-references |
| `src/crates/oya-workflow-engine-execution-engine-adapter-redis/{Cargo.toml,src/{lib,lease_store,ephemeral_state}.rs}` | create | Redis lease coordinator |
| `microservices/workflow-engine/catalog/oya-workflow-engine-execution-engine-{usecase,api,adapter,adapter-postgres,adapter-redis}.yaml` | create | 5 catalog rows |
| `Cargo.toml` (workspace) | update | register 5 crates |

## Code Shape (usecase)

```rust
pub struct RunLifecycleOrchestrator<RS, ES, TE, IV, EB, AC>
where
    RS: WorkflowRunStore,
    ES: EphemeralStateStore,
    TE: TransitionEngine,
    IV: InvariantValidator,
    EB: EventBus,
    AC: AuditChainEmitter,
{
    run_store: Arc<RS>,
    ephemeral: Arc<ES>,
    transition: Arc<TE>,
    invariant: Arc<IV>,
    event_bus: Arc<EB>,
    audit_chain: Arc<AC>,
}

impl<RS, ES, TE, IV, EB, AC> RunLifecycleOrchestrator<RS, ES, TE, IV, EB, AC> {
    pub async fn start_run(&self, request: RunStartRequest) -> Result<WorkflowRun, RunError> {
        // 1. Validate spec version is published
        // 2. Allocate run_id (ULID)
        // 3. Persist initial WorkflowRun row (Postgres)
        // 4. Claim Redis lease for step 0
        // 5. Emit WorkflowStarted event (via outbox)
        // 6. Audit-chain seal
        // Returns run_id; step dispatch is async via worker
    }

    pub async fn dispatch_step(&self, run_id: &RunId) -> Result<StepExecution, RunError> {
        // 1. Verify worker holds Redis lease
        // 2. Read current state from Postgres
        // 3. Evaluate transition via TransitionEngine
        // 4. Validate invariants
        // 5. Execute step body (in Wasmtime sandbox or in-process for trusted)
        // 6. Persist new state (optimistic concurrency check)
        // 7. Emit step event
        // 8. Audit-chain seal
        // 9. Compute next step OR transition to terminal
    }

    // pause / resume / cancel / signal methods ...
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-workflow-engine-execution-engine-usecase --all-features
cargo nextest run -p oya-workflow-engine-execution-engine-adapter-postgres --all-features
cargo nextest run -p oya-workflow-engine-execution-engine-adapter-redis --all-features
cargo run -p oya-dev-cli -- gate validate deterministic-replay --crate oya-workflow-engine-execution-engine-usecase
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_start_run_persists_row` | Postgres row created with correct fields |
| `test_dispatch_step_optimistic_concurrency` | concurrent writes detected; one succeeds, other retries |
| `test_lease_acquisition_single_writer` | only one worker holds lease at a time |
| `test_durable_execution_restart` (e2e) | engine kill mid-run; restart; identical step sequence |
| `test_audit_chain_seal_per_state_transition` | every transition emits seal |
| `test_idempotency_key_dedup` | retry with same idempotency_key returns first response |

## Halt Conditions

- Deterministic replay invariant violated.
- Single-writer guarantee violated.
- Audit-chain seal sequence has gaps.

## Next IP

[`IP-006-event-bus-kernel-domain-adapter.md`](IP-006-event-bus-kernel-domain-adapter.md)

## References

- PRD AC-02, AC-03, AC-04
- ADR-0035 (workflow engine durable execution)
- `policy/spec-integrity.md` (forbidden constructs)
- Temporal durability docs — `docs.temporal.io/dev-guide/durability`
- Postgres optimistic concurrency — `postgresql.org/docs/current/explicit-locking.html`
