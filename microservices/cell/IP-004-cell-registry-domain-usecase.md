---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-004-cell-registry-domain-usecase
status: pending
owner: axis-cell-substrate
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

# IP-004: oya-cell-cell-registry-domain + oya-cell-cell-registry-usecase

## Intent

Implement the cell-state-machine pure domain (legal transitions enum + validation) + use case orchestrators that read OpenSLO + Postgres via ports + emit events.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-cell-registry-domain/Cargo.toml + src/{lib.rs,state_machine.rs}` | create |
| `microservices/cell/src/crates/oya-cell-cell-registry-usecase/Cargo.toml + src/{lib.rs,get_cell.rs,list_cells.rs,transition_state.rs}` | create |
| Catalog rows under `catalog/` | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-cell-cell-registry-domain
JUSTIFICATION: layer=domain; pure state-machine math; depends on kernel only.

NAME: oya-cell-cell-registry-usecase
JUSTIFICATION: layer=usecase (per ADR-0106; replaces 'application'); orchestrators.
```

## Code Shape

```rust
// domain/src/state_machine.rs
use oya_cell_cell_registry_kernel::CellState;

pub fn is_legal_transition(from: CellState, to: CellState) -> bool {
    use CellState::*;
    matches!((from, to),
        (Requested, Provisioning) |
        (Provisioning, Ready) |
        (Provisioning, Decommissioned) |          // failed-provisioning early-decommission
        (Ready, Draining) |
        (Draining, DecommissioningSoftDelete) |
        (DecommissioningSoftDelete, Decommissioned)
    )
}
```

```rust
// usecase/src/transition_state.rs
pub struct TransitionCellState<R, E> {
    repo: R,
    events: E,
}

impl<R: CellRepository, E: CellEventEmitter> TransitionCellState<R, E> {
    pub async fn execute(&self, cell_id: &CellId, new_state: CellState, by: &str, reason: Option<&str>)
        -> Result<(), UsecaseError>
    {
        let cell = self.repo.get(cell_id).await?;
        if !is_legal_transition(cell.state, new_state) {
            return Err(UsecaseError::IllegalTransition { from: cell.state, to: new_state });
        }
        let signature = sign_transition(cell_id, new_state, by, reason);
        self.repo.transition_state(cell_id, new_state, &signature).await?;
        self.events.emit_lifecycle_transition(&CellLifecycleEvent {
            cell_id: cell_id.clone(),
            prev_state: cell.state,
            new_state,
            transitioned_at: chrono::Utc::now(),
            transitioned_by: by.to_string(),
            reason: reason.map(String::from),
            signature,
        }).await?;
        Ok(())
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-cell-cell-registry-domain -p oya-cell-cell-registry-usecase
cargo nextest run -p oya-cell-cell-registry-domain --test state_machine_props
cargo nextest run -p oya-cell-cell-registry-usecase --test transition_state
```

## Test Plan

- Property tests for state-machine: only legal transitions accepted; illegal returns IllegalTransition.
- Use-case unit tests: 1 per orchestrator (happy + 2 sad paths).
- Coverage: 95% line / 90% branch (domain); 90% line / 80% branch (usecase).

## Halt Conditions

- Illegal-transition acceptance — fix state machine.
- Use case I/O without going through ports — refactor.

## Next IP

[`IP-005-cell-registry-adapter-postgres-rest-sdk-app.md`](IP-005-cell-registry-adapter-postgres-rest-sdk-app.md)

## References

- ADR-0105; ADR-0106.
- Bominal ADR-0009 (cell lifecycle).
