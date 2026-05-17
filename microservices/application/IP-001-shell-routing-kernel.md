---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-001-shell-routing-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-application
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, data-class]
---

# IP-001: oya-application-shell-routing-kernel

## Intent

Kernel layer per ADR-0105: port traits (sealed) + entities (Route,
RouteRegistration, RouteScope) + errors. Zero I/O; zero business logic.
Foundation for every other shell-routing crate.

## ChangeSet boundary

One new Rust crate at
`microservices/application/src/crates/oya-application-shell-routing-kernel/`.
Workspace member added; catalog row created. No downstream consumers in
this IP.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/application/src/crates/oya-application-shell-routing-kernel/Cargo.toml` | create | minimal deps (`async-trait`, `serde`) |
| `.../src/lib.rs` | create | module declarations + `pub use` |
| `.../src/entities.rs` | create | Route, RouteRegistration, RouteScope, TenantScope, PackResidency with `data_class` annotations |
| `.../src/ports.rs` | create | sealed port traits |
| `.../src/errors.rs` | create | error variants |
| `Cargo.toml` (workspace) | update | add member |
| `microservices/application/catalog/oya-application-shell-routing-kernel.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-application-shell-routing-kernel
JUSTIFICATION:
- microservice = application
- bc-tokens = shell-routing
- layer = kernel (ADR-0105; inner/pure)
- exemptions claimed: none
```

## Code Shape

```rust
// src/lib.rs
pub mod entities;
pub mod errors;
pub mod ports;

pub use entities::{Route, RouteRegistration, RouteScope, TenantScope, PackResidency, MfaFactor};
pub use errors::{KernelError, RegistryError};
pub use ports::{RouteRegistry, RouteResolver, RouteScopeStore};

#[doc(hidden)]
mod sealed {
    pub trait Sealed {}
}
```

```rust
// src/entities.rs (excerpt)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    #[data_class(INTERNAL_ONLY)] pub path: String,
    #[data_class(INTERNAL_ONLY)] pub tenant_scope: TenantScope,
    #[data_class(INTERNAL_ONLY)] pub required_roles: Vec<String>,
    #[data_class(INTERNAL_ONLY)] pub pack_residency: PackResidency,
    #[data_class(INTERNAL_ONLY)] pub admin_scope: bool,
    #[data_class(INTERNAL_ONLY)] pub csp_module_id: String,
    #[data_class(INTERNAL_ONLY)] pub required_mfa: MfaFactor,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TenantScope { GlobalPublic, TenantScoped, CrossTenantOperator }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackResidency { Any, InheritFromTenant, Specific(String) }
```

```rust
// src/ports.rs
#[async_trait]
pub trait RouteRegistry: Send + Sync + Sealed {
    async fn register(&self, reg: &RouteRegistration) -> Result<(), RegistryError>;
    async fn list_for_tenant(&self, tenant_id: &str) -> Result<Vec<Route>, RegistryError>;
}

#[async_trait]
pub trait RouteResolver: Send + Sync + Sealed {
    async fn resolve(&self, path: &str, tenant_id: &str) -> Result<Option<Route>, RegistryError>;
}

#[async_trait]
pub trait RouteScopeStore: Send + Sync + Sealed {
    async fn scopes_for(&self, route: &Route, principal_roles: &[String]) -> Result<RouteScope, RegistryError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-application-shell-routing-kernel --all-features
cargo build -p oya-application-shell-routing-kernel --all-features
cargo clippy -p oya-application-shell-routing-kernel --all-features -- -D warnings
cargo nextest run -p oya-application-shell-routing-kernel --all-features
cargo deny check
cargo doc -p oya-application-shell-routing-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-application-shell-routing-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-application-shell-routing-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-application-shell-routing-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-application-shell-routing-kernel
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_route_construction` | entity invariants |
| `test_route_serde` | serde roundtrip |
| `test_tenant_scope_variants_exhaustive` | enum coverage |
| `test_port_traits_sealed` | external impl forbidden |
| `test_data_class_annotations_present` | every field annotated |

Coverage: 95 % line / 80 % branch.

## Halt Conditions

- BNF v4.1 violation
- Any port introduces business logic
- Any I/O reachable from kernel

## Next IP

[`IP-002-shell-routing-domain.md`](IP-002-shell-routing-domain.md)

## References

- ADR-0056, ADR-0105, ADR-0131.
- `microservices/application/PRD.md` §"Bounded Contexts".
- Bominal ADR-0028 data-class taxonomy.
