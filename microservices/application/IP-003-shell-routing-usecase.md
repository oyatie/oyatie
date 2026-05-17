---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-003-shell-routing-usecase
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

# IP-003: oya-application-shell-routing-usecase

## Intent

Orchestrators: `ResolveRoute`, `RegisterRoute`. Reads via kernel ports;
applies domain matcher; writes audit events.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-shell-routing-usecase/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/resolve.rs` | create — `ResolveRouteUseCase` |
| `.../src/register.rs` | create — `RegisterRouteUseCase` |
| `microservices/application/catalog/oya-application-shell-routing-usecase.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Crate Naming

```
NAME: oya-application-shell-routing-usecase
JUSTIFICATION: microservice=application; bc=shell-routing; layer=usecase (ADR-0106 rename; orchestrator)
```

## Code Shape

```rust
pub struct ResolveRouteUseCase<R, S> {
    registry: R, scope_store: S, audit: AuditEmitter,
}

impl<R: RouteRegistry, S: RouteScopeStore> ResolveRouteUseCase<R, S> {
    pub async fn resolve(&self, principal: &Principal, path: &str) -> Result<RouteResolution, UseCaseError> {
        let route = self.registry.list_for_tenant(&principal.tenant_id).await?;
        let matcher = RouteMatcher::build(route);
        let Some(matched) = matcher.match_path(path) else {
            self.audit.emit(RouteAccessDenied::new(principal, path, "not_found"));
            return Err(UseCaseError::NotFound);
        };
        if !intersect_scopes(&matched.required_roles, &principal.roles) {
            self.audit.emit(RouteAccessDenied::new(principal, path, "role_mismatch"));
            return Err(UseCaseError::Forbidden);
        }
        if !require_mfa(matched.required_mfa, principal.mfa_factor) {
            return Err(UseCaseError::MfaRequired);
        }
        Ok(RouteResolution { route: matched.clone() })
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-shell-routing-usecase --all-features
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-application-shell-routing-usecase
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_resolve_happy_path` | matched + role-intersect + mfa ok |
| `test_resolve_role_mismatch_denied` | audit emitted |
| `test_resolve_not_found` | audit emitted |
| `test_resolve_mfa_required` | webauthn-required route refuses TOTP |
| `test_resolve_cross_tenant_denied` | tenant_id mismatch |

Coverage: 90 % line / 80 % branch.

## Next IP

[`IP-004-shell-routing-adapter.md`](IP-004-shell-routing-adapter.md)
