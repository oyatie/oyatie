---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-004-shell-routing-adapter
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

# IP-004: oya-application-shell-routing-adapter

## Intent

Adapter layer: Postgres-backed RouteRegistry + in-memory LRU cache.
Implements kernel ports. RLS-scoped queries by tenant_id.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-shell-routing-adapter/Cargo.toml` | create — sqlx + lru deps |
| `.../src/lib.rs` | create |
| `.../src/postgres_registry.rs` | create — `PostgresRouteRegistry` |
| `.../src/cache.rs` | create — LRU wrapper with TTL |
| `.../migrations/0001-create-route-registration.sql` | create |
| `.../migrations/0002-rls-tenant-id.sql` | create — RLS policy on tenant_id |
| `microservices/application/catalog/oya-application-shell-routing-adapter.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-application-shell-routing-adapter
JUSTIFICATION: microservice=application; bc=shell-routing; layer=adapter (ADR-0105 port implementer)
```

## Code Shape

```rust
pub struct PostgresRouteRegistry { pool: sqlx::PgPool, cache: Arc<RouteCache> }

#[async_trait]
impl RouteRegistry for PostgresRouteRegistry {
    async fn list_for_tenant(&self, tenant_id: &str) -> Result<Vec<Route>, RegistryError> {
        if let Some(v) = self.cache.get(tenant_id) { return Ok(v); }
        let rows = sqlx::query_as!(
            RouteRow,
            r#"SELECT path, tenant_scope, required_roles, pack_residency,
                       admin_scope, csp_module_id, required_mfa
               FROM route_registration WHERE tenant_id = $1"#,
            tenant_id
        ).fetch_all(&self.pool).await?;
        let routes = rows.into_iter().map(Into::into).collect::<Vec<_>>();
        self.cache.put(tenant_id, routes.clone());
        Ok(routes)
    }
    async fn register(&self, reg: &RouteRegistration) -> Result<(), RegistryError> { /* ... */ }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-shell-routing-adapter --all-features
cargo sqlx prepare --check
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-application-shell-routing-adapter
cargo run -p oya-dev-cli -- gate validate rls-pin --crate oya-application-shell-routing-adapter
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_list_for_tenant_hits_db_once` | LRU cache works |
| `test_rls_cross_tenant_denied` | RLS policy refuses cross-tenant SELECT |
| `test_register_idempotent` | INSERT ON CONFLICT |
| `test_cache_invalidate_on_register` | cache flushed |

Coverage: 85 % / 75 %.

## Next IP

[`IP-005-shell-routing-rest.md`](IP-005-shell-routing-rest.md)
