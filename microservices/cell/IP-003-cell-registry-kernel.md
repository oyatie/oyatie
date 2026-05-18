---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-003-cell-registry-kernel
status: pending
owner: axis-cell-substrate
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, lean-a1, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: oya-cell-cell-registry-kernel

## Intent

Scaffold the `kernel` layer crate per ADR-0105: port traits (sealed) + entity types + value objects + error types. Zero I/O. Zero business logic.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-cell-registry-kernel/Cargo.toml` | create |
| `.../src/{lib.rs,entities.rs,ports.rs,errors.rs}` | create |
| `microservices/cell/catalog/oya-cell-cell-registry-kernel.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-cell-cell-registry-kernel
JUSTIFICATION:
- microservice = cell.
- bc-tokens = cell-registry (primary BC; siblings exist).
- layer = kernel (port traits + entities; zero I/O).
- exemptions claimed: none.
```

## Code Shape

```rust
// src/entities.rs (excerpt)
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    #[data_class(INTERNAL_ONLY)]
    pub cell_id: CellId,
    #[data_class(INTERNAL_ONLY)]
    pub pack: Pack,
    #[data_class(INTERNAL_ONLY)]
    pub region: String,
    #[data_class(INTERNAL_ONLY)]
    pub state: CellState,
    #[data_class(INTERNAL_ONLY)]
    pub cell_scope: CellScope,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub capacity_envelope: CapacityEnvelope,
    #[data_class(INTERNAL_ONLY)]
    pub version: String,
    #[data_class(AUDIT)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[data_class(AUDIT)]
    pub decommissioned_at: Option<chrono::DateTime<chrono::Utc>>,
    #[data_class(AUDIT)]
    pub signature: Ed25519Signature,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CellState {
    Requested, Provisioning, Ready, Draining,
    DecommissioningSoftDelete, Decommissioned,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CellScope {
    Shared, Dedicated, HipaaDedicated, Sandbox, Internal,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Pack {
    Kr, Eu, Us, UsHealthcare, Jp, Sg, Au, In_, Br, Ae, Ksa,
}
```

```rust
// src/ports.rs
use async_trait::async_trait;
use crate::sealed::Sealed;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait CellRepository: Send + Sync + Sealed {
    async fn get(&self, cell_id: &CellId) -> Result<Cell, RepositoryError>;
    async fn list_by_pack(&self, pack: Pack) -> Result<Vec<Cell>, RepositoryError>;
    async fn insert(&self, cell: &Cell) -> Result<(), RepositoryError>;
    async fn transition_state(
        &self,
        cell_id: &CellId,
        new_state: CellState,
        signature: &Ed25519Signature,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait CellEventEmitter: Send + Sync + Sealed {
    async fn emit_lifecycle_transition(&self, event: &CellLifecycleEvent)
        -> Result<(), KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-cell-cell-registry-kernel
cargo nextest run -p oya-cell-cell-registry-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-cell-cell-registry-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-cell-cell-registry-kernel
```

## Test Plan

Per PHASE-01 kernel class: 1 test per public type + 1 per port trait + 1 sealed-trait smoke. Coverage 90% line / 80% branch.

## Halt Conditions

- BNF v4.1 violation.
- Port trait introduces business logic.
- I/O reachable from kernel.

## Next IP

[`IP-004-cell-registry-domain-usecase.md`](IP-004-cell-registry-domain-usecase.md)

## References

- ADR-0056; ADR-0105; ADR-0106.
- `microservices/cell/PRD.md` §"Bounded Contexts".
- Bominal ADR-0028.
