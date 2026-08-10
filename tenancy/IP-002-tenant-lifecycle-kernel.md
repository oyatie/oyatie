---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-002-tenant-lifecycle-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tenancy
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: oya-tenancy-tenant-lifecycle-kernel

## Intent

Scaffold the `kernel` layer crate per ADR-0105: port traits (sealed) + entity types + value objects + error types for the tenant-lifecycle BC. Zero I/O; zero business logic. Foundation that every other tenant-lifecycle layer crate depends on.

## ChangeSet boundary

One new Rust crate at `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-kernel/`. Workspace member added to root `Cargo.toml`. Catalog row at `tenancy/catalog/oya-tenancy-tenant-lifecycle-kernel.yaml`. No downstream consumers in this IP.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-kernel/Cargo.toml` | create |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-kernel/src/lib.rs` | create |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-kernel/src/entities.rs` | create — `Tenant`, `TenantId`, `TenantStatus`, `JurisdictionCode`, `PlanTier`, `TenantContext` |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-kernel/src/ports.rs` | create — `TenantRepository`, `TenantContextResolver` traits |
| `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-kernel/src/errors.rs` | create |
| `Cargo.toml` (workspace) | update — register member |
| `tenancy/catalog/oya-tenancy-tenant-lifecycle-kernel.yaml` | create |

## Crate Naming

```
NAME: oya-tenancy-tenant-lifecycle-kernel
JUSTIFICATION:
- microservice = tenancy
- bc-tokens = tenant-lifecycle (primary BC per PRD)
- layer = kernel (ADR-0105 13-value enum; pure; zero I/O)
- exemptions claimed: none
```

## Code Shape

```rust
// src/entities.rs (excerpt)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tenant {
    #[data_class(SENSITIVE_PIPA_ART23)]
    pub tenant_id: TenantId,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub status: TenantStatus,
    #[data_class(INTERNAL_ONLY)]
    pub jurisdiction_code: JurisdictionCode,
    #[data_class(INTERNAL_ONLY)]
    pub plan_tier: PlanTier,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub cell_id: CellId,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[data_class(AUDIT)]
    pub activated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TenantId(String);  // pattern: tenant:<16-hex>; newtype for type-safety

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TenantStatus { Created, Activated, Suspended, DeletionRequested, Deleted }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum JurisdictionCode { KR, EU, US, USHC, JP, SG, AU, IN, BR, AE, KSA }

pub struct TenantContext {
    pub tenant_id: TenantId,
    pub pack: Pack,
    pub jurisdiction_code: JurisdictionCode,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
```

```rust
// src/ports.rs
#[async_trait]
pub trait TenantRepository: Send + Sync + Sealed {
    async fn create(&self, t: &Tenant) -> Result<(), RepositoryError>;
    async fn read(&self, id: &TenantId) -> Result<Tenant, RepositoryError>;
    async fn list(&self, f: &Filter) -> Result<Vec<Tenant>, RepositoryError>;
    async fn update_status(&self, id: &TenantId, status: TenantStatus, actor: &Actor) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TenantContextResolver: Send + Sync + Sealed {
    async fn resolve_from_jwt(&self, jwt: &str) -> Result<TenantContext, KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-tenancy-tenant-lifecycle-kernel --all-features
cargo build -p oya-tenancy-tenant-lifecycle-kernel --all-features
cargo clippy -p oya-tenancy-tenant-lifecycle-kernel --all-features -- -D warnings
cargo nextest run -p oya-tenancy-tenant-lifecycle-kernel --all-features
cargo deny check
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-tenancy-tenant-lifecycle-kernel
cargo run -p oya-dev-cli -- gate validate port-location --crate oya-tenancy-tenant-lifecycle-kernel
cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-tenancy-tenant-lifecycle-kernel
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-tenancy-tenant-lifecycle-kernel
```

## Test Plan

Per PHASE-01 kernel class: 1 test per public type + 1 per port trait + 1 sealed-trait smoke. Coverage 90% line / 80% branch.

## Halt Conditions

- BNF v4.1 naming violation.
- Any port trait introduces business logic.
- Any I/O reachable from kernel.

## Next IP

[`IP-003-tenant-lifecycle-domain.md`](IP-003-tenant-lifecycle-domain.md)

## References

- ADR-0056 BNF v4.1; ADR-0105 13-layer enum; Bominal ADR-0018 + ADR-0028.
- PRD §"Bounded Contexts" port-trait table.
