---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-006-tenant-context-kernel
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, port-location, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-application-tenant-context-kernel

## Intent

Kernel for tenant-context BC: port traits (TenantResolver,
TenantBindingStore) + entities (TenantContext, TenantBinding, Principal).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-tenant-context-kernel/Cargo.toml` | create |
| `.../src/{lib,entities,ports,errors}.rs` | create |
| `microservices/application/catalog/oya-application-tenant-context-kernel.yaml` | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantContext {
    #[data_class(SENSITIVE_PIPA_ART23)] pub tenant_id: String,
    #[data_class(INTERNAL_ONLY)] pub pack: String,
    #[data_class(INTERNAL_ONLY)] pub jurisdiction: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Principal {
    #[data_class(PII_IDENTIFYING)] pub user_id: String,
    #[data_class(SENSITIVE_PIPA_ART23)] pub tenant_id: String,
    #[data_class(INTERNAL_ONLY)] pub roles: Vec<String>,
    #[data_class(PII_AUTHN_CREDENTIAL)] pub mfa_factor: MfaFactor,
    #[data_class(INTERNAL_ONLY)] pub pack: String,
}

#[async_trait]
pub trait TenantResolver: Send + Sync + Sealed {
    async fn from_host(&self, host: &str) -> Result<TenantContext, KernelError>;
    async fn verify_jwt_claim(&self, jwt_tenant: &str, ctx: &TenantContext) -> Result<(), KernelError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-tenant-context-kernel --all-features
buck2 build //:quality-lane-registry-authority-check # lane=port-location --crate oya-application-tenant-context-kernel
buck2 build //:quality-lane-registry-authority-check # lane=data-class --crate oya-application-tenant-context-kernel
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_tenant_context_serde` | roundtrip |
| `test_principal_data_class_complete` | every field annotated |
| `test_port_sealed` | external impl forbidden |

## Next IP

[`IP-007-tenant-context-usecase-rest.md`](IP-007-tenant-context-usecase-rest.md)
