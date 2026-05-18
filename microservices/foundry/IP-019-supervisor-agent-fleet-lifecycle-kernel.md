---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-004-agent-fleet-lifecycle-kernel
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, port-location, layer-correctness, oya-check-data-class]
---

# IP-004: agent-fleet-lifecycle-kernel

## Intent

Scaffold the `kernel` layer crate for the agent-fleet-lifecycle BC: port traits (sealed) + entities (`Agent`, `AgentDeployment`, `FleetState`, `DrainHandle`) + value objects + errors. Zero I/O.

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-supervisor-agent-fleet-lifecycle-kernel/` + workspace registration + catalog row.

## Concrete File Targets

| Path | Action |
|---|---|
| `…-kernel/Cargo.toml` | create |
| `…-kernel/src/lib.rs` | create |
| `…-kernel/src/entities.rs` | create |
| `…-kernel/src/ports.rs` | create |
| `…-kernel/src/errors.rs` | create |
| `Cargo.toml` (workspace) | update — add member |
| `microservices/foundry/catalog/oya-foundry-supervisor-agent-fleet-lifecycle-kernel.yaml` | create |

## Crate Naming

```
NAME: oya-foundry-supervisor-agent-fleet-lifecycle-kernel
JUSTIFICATION:
- microservice = foundry-supervisor (per ADR-0131 §"Foundry split")
- bc-tokens = agent-fleet-lifecycle (per PRD §"Bounded Contexts")
- layer = kernel (ADR-0105 13-value enum)
- exemptions claimed: none
```

## Code Shape

```rust
// src/entities.rs (excerpt)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub agent_id: AgentId,
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub tenant_id: TenantId,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub capability_id: CapabilityId,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub state: AgentState,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentState { Pending, Healthy, Draining, Evicted }
// ... AgentDeployment, FleetState, DrainHandle similarly annotated
```

```rust
// src/ports.rs
#[async_trait]
pub trait FleetStateRepository: Send + Sync + Sealed {
    async fn load(&self, tenant: &TenantId) -> Result<FleetState, RepositoryError>;
    async fn drain(&self, tenant: &TenantId, grace_period: Duration) -> Result<DrainHandle, RepositoryError>;
    async fn evict(&self, agent: &AgentId, reason: EvictionReason) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait AgentDeploymentReconciler: Send + Sync + Sealed {
    async fn reconcile(&self, deployment: &AgentDeployment) -> Result<(), KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-supervisor-agent-fleet-lifecycle-kernel --all-features
cargo build -p oya-foundry-supervisor-agent-fleet-lifecycle-kernel --all-features
cargo clippy -p oya-foundry-supervisor-agent-fleet-lifecycle-kernel --all-features -- -D warnings
cargo nextest run -p oya-foundry-supervisor-agent-fleet-lifecycle-kernel --all-features
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-foundry-supervisor-agent-fleet-lifecycle-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-foundry-supervisor-agent-fleet-lifecycle-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-foundry-supervisor-agent-fleet-lifecycle-kernel
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_agent_construction` | entity invariants |
| `test_deployment_serde` | serde roundtrip |
| `test_fleet_state_arithmetic_pure` | no I/O |
| `test_port_traits_sealed` | external crates cannot impl |
| `test_data_class_annotations_present` | every public field annotated |

## Halt Conditions

- BNF v4.1 violation.
- Port trait introduces business logic.
- Any I/O reachable from kernel.

## Next IP

[`IP-005-autonomy-policy-enforcement.md`](IP-005-autonomy-policy-enforcement.md)

## References

- ADR-0056 BNF v4.1; ADR-0105 13-layer enum; ADR-0131 §"Foundry split".
- PRD §"Bounded Contexts" port-trait table.
- Bominal ADR-0028 (data-class taxonomy).
