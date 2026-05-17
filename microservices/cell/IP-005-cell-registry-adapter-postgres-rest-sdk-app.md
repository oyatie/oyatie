---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-005-cell-registry-adapter-postgres-rest-sdk-app
status: pending
owner: axis-cell-substrate
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

# IP-005: cell-registry — adapter-postgres + rest + sdk + app

## Intent

Land the full cell-registry stack: adapter (sqlx Postgres impls of CellRepository + CellEventEmitter), REST handler crate, SDK client, app composition root.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cell/src/crates/oya-cell-cell-registry-adapter/` | create (protocol-neutral adapter impls) |
| `microservices/cell/src/crates/oya-cell-cell-registry-adapter-postgres/` | create (sqlx + RLS-aware) |
| `microservices/cell/src/crates/oya-cell-cell-registry-api/` | create (typed contracts) |
| `microservices/cell/src/crates/oya-cell-cell-registry-rest/` | create (axum handler stack consuming -api types) |
| `microservices/cell/src/crates/oya-cell-cell-registry-sdk/` | create (client; LRU cache + event subscribe) |
| `microservices/cell/src/crates/oya-cell-cell-registry-app/` | create (binary; wires worker + rest + adapters) |
| Catalog rows for each crate | create |

## Crate Naming

Standard pattern: each crate name follows `oya-cell-cell-registry-<layer>`. `-adapter-postgres` is backend-qualified per ADR-0105 Amendment 3.

## Code Shape

```rust
// adapter-postgres/src/cell_repository.rs
pub struct PgCellRepository {
    pool: sqlx::PgPool,
    pack: Pack,
}

#[async_trait]
impl CellRepository for PgCellRepository {
    async fn get(&self, cell_id: &CellId) -> Result<Cell, RepositoryError> {
        // Set session GUC for RLS scope before query
        sqlx::query!("SET LOCAL app.session_pack = $1", self.pack.to_string())
            .execute(&self.pool).await?;
        let row = sqlx::query!(
            "SELECT cell_id, pack, region, state, cell_scope, capacity_envelope, version,
                    created_at, decommissioned_at, signature
             FROM cells WHERE cell_id = $1", cell_id.as_str()
        ).fetch_one(&self.pool).await?;
        Ok(row.into())
    }
    // ... list_by_pack, insert, transition_state
}
```

```rust
// rest/src/main.rs (handlers)
async fn get_assignment(
    State(state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
    auth: AuthScope,
) -> Result<Json<CellAssignment>, ApiError> {
    auth.verify_scope_or_403(&tenant_id)?;
    let assignment = state.read_assignment_uc.execute(&tenant_id).await?;
    Ok(Json(assignment))
}
```

## Acceptance Gates

```bash
cargo check --workspace
cargo nextest run -p oya-cell-cell-registry-adapter-postgres -p oya-cell-cell-registry-rest
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice cell
```

## Test Plan

- Adapter: 1 test per port-impl method against real Postgres in test container.
- REST: 1 test per route (happy + auth-fail + tenant-mismatch). Coverage 85% line / 75% branch.
- SDK: 1 test per public client method.
- App: composition-root smoke test.

## Halt Conditions

- Postgres connection without RLS-scope setting — fix.
- REST route without Cedar policy guard — fix.

## Next IP

[`IP-006-cell-boundary-gate-lane.md`](IP-006-cell-boundary-gate-lane.md)

## References

- `microservices/cell/contracts/openapi/cell.yaml`.
- `microservices/cell/policy/cell-boundary.md`.
- sqlx — `github.com/launchbadge/sqlx`.
- axum — `github.com/tokio-rs/axum`.
