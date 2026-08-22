---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-005-tenant-lifecycle-adapter-postgres
status: pending
owner: axis-tenancy
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness, governance-tenant-context-setlocal-present]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: tenancy-tenant-lifecycle-adapter-postgres

## Intent

`-adapter-postgres` crate: implements `TenantRepository` against Postgres + Citus via sqlx; emits `SET LOCAL app.current_tenant_id` on every checkout; runs schema migrations at activation.

## Concrete File Targets

| Path | Action |
|---|---|
| `tenancy-tenant-lifecycle-adapter-postgres/Cargo.toml` | create |
| `tenancy-tenant-lifecycle-adapter-postgres/src/{lib,repository,migration_runner,connection_pool}.rs` | create |
| `tenancy-tenant-lifecycle-adapter-postgres/migrations/*` | create — schema + RLS DDL |
| catalog row | create |

## Code Shape

```rust
// src/repository.rs
#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn create(&self, t: &Tenant) -> Result<(), RepositoryError> {
        let mut conn = self.pool.acquire().await?;
        // Note: tenancy-internal tables also tenant-scoped to tenant:system
        conn.execute(sqlx::query("SET LOCAL app.current_tenant_id = $1").bind("tenant:system")).await?;
        sqlx::query!(
            "INSERT INTO tenancy.tenants (tenant_id, status, jurisdiction_code, plan_tier, cell_id, created_at) VALUES ($1,$2,$3,$4,$5,$6)",
            t.tenant_id.as_str(), t.status, t.jurisdiction_code, t.plan_tier, t.cell_id, t.created_at
        ).execute(&mut *conn).await?;
        Ok(())
    }
    // ... read, list, update_status
}
```

```rust
// src/connection_pool.rs  - CANONICAL CHECKOUT PATTERN per policy/rls-isolation.md Invariant RLS-03
pub async fn checkout_tenant_scoped(pool: &PgPool, tenant_id: &TenantId) -> Result<PoolConnection, ...> {
    let mut conn = pool.acquire().await?;
    conn.execute(sqlx::query("SET LOCAL app.current_tenant_id = $1").bind(tenant_id.as_str())).await?;
    Ok(conn)
}
```

## Acceptance Gates

```bash
cargo nextest run -p tenancy-tenant-lifecycle-adapter-postgres
cargo run -p dev-cli -- gate validate tenant-context-setlocal-present
cargo run -p dev-cli -- gate validate rls-no-superuser-bypass
```

## Test Plan

Per PHASE-01 adapter class: 1 per port-impl method + ≥ 2 against real Postgres test container.
- `test_create_tenant_persists_with_setlocal` — verifies `SET LOCAL` precedes INSERT.
- `test_create_does_not_use_bypassrls` — connection role is `tenancy_app` (not bypass).
- `test_rls_post_migration_force_rls_true` — every tenant-bound table has `relforcerowsecurity=true`.

## Next IP

[`IP-006-isolation-policy-rls-generator.md`](IP-006-isolation-policy-rls-generator.md)
