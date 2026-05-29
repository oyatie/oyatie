---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-005-shell-routing-rest
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, openapi-conformance, cedar-policy-compiles]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-application-shell-routing-rest

## Intent

REST surface per `contracts/openapi/application.yaml`. axum router with
OIDC middleware + tenant-context middleware + Cedar policy gate middleware.
Implements `GET /routes/resolve`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-shell-routing-rest/Cargo.toml` | create |
| `.../src/{lib.rs,router.rs,handlers.rs,middleware/{auth,tenant_context,cedar}.rs}` | create |
| `microservices/application/catalog/oya-application-shell-routing-rest.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
pub fn build_router(deps: AppDeps) -> Router {
    Router::new()
        .route("/api/v1/routes/resolve", get(handlers::resolve_route))
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
        .layer(middleware::oidc_auth())
        .layer(middleware::tenant_context())
        .layer(middleware::cedar_policy_gate())
        .layer(middleware::audit_route_access())
        .with_state(deps)
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-shell-routing-rest --all-features
cargo run -p oya-dev-cli -- gate validate openapi-conformance --crate oya-application-shell-routing-rest
cargo run -p oya-dev-cli -- gate validate cedar-policy-compiles --crate oya-application-shell-routing-rest
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_resolve_route_handler` | 200 on valid |
| `test_route_oidc_missing_401` | auth required |
| `test_route_cedar_deny_403` | Cedar default-deny |
| `test_route_tenant_mismatch_401` | host vs JWT |
| `test_cedar_eval_p99_under_10ms` | budget enforced |

## Next IP

[`IP-006-tenant-context-kernel.md`](IP-006-tenant-context-kernel.md)
