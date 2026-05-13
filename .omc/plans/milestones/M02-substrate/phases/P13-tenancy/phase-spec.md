---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P13-tenancy
status: Proposed
entry_gate: |
  M02/P03-identity complete; oya-identity-kernel ships with UserStore, PersonStore,
  OrganizationStore, EmployeeStore, SessionStore, AuthChallenger port traits; cargo
  check clean; grit done on all P03 symbols; ICM phase-handoff row emitted.
exit_gate: |
  All P13 impl-plan acceptance gates green; 4 BCs registered in
  docs/standards/bounded-contexts.md (tenancy, tenant-products, tenant-cells,
  tenant-admins); oyatie.set_current_tenant() RLS bootstrap function deployed;
  TenantProductRegistry port wired and tested with isolation tests; all crates
  pass cargo check/build/clippy/nextest/deny; oya gate validate lean-a1/a2/a3/a4
  exit 0; grit done on all P13 symbols; ICM phase-complete row emitted.
depends_on:
  - milestone: M02
    phase: P03-identity
    reason: "Tenancy references identity.users for tenant_admins FK; TenantStore.create() accepts owner_user_id: UserId from identity kernel; RLS bootstrap calls oyatie.set_current_tenant() which feeds into all identity.organizations + identity.employees queries."
owner_team: council-architecture
---

# P13-tenancy: Tenancy Substrate — Multi-Tenant SaaS Contract + Product-Enablement Registry + Cell Placement

## Purpose

Delivers the tenancy substrate: the SaaS contract layer that partitions every subsequent
µservice's data by tenant_id and governs which products each tenant has enabled. The
`tenant_products` table is THE per-tenant µservice catalog — the registry that the B2B
Application shell (P19) reads to build the product-enablement UI ("enable HR", "enable
Payroll", etc.), and that every product µservice queries before processing a request.

The `oyatie.set_current_tenant(tenant_id)` RLS bootstrap function ships here and is
consumed by every downstream µservice that uses Postgres RLS. Cell placement
(TenantCellPlacer port) determines which infrastructure cell a tenant's data lives in
per Bominal ADR-0009 cell architecture.

Per Bominal ADR-0018 (tenancy + RLS posture): DB-native RLS is the end-state; this phase
delivers the full RLS bootstrap, not service-layer filtering. Per [[feedback-flat-product-catalog]]:
tenant_products.microservice is a registered µservice name from ADR-0056 v4.1 microservice
registry — enforcing the flat catalog at the data layer.

Advances Master Plan principles: horizontal scalability mandatory from day one (all tables
declare tenant_id distribution column for Citus); hyperscaler-grade multi-tenancy (cell
architecture from ADR-0009; primary cell uniqueness constraint; region affinity).

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `tenancy` | core | `crates/oya-tenancy-kernel/` | `oya-tenancy-kernel` |
| `tenancy` | core | `crates/oya-tenancy-domain/` | `oya-tenancy-domain` |
| `tenancy` | core | `crates/oya-tenancy-application/` | `oya-tenancy-application` |
| `tenancy` | core | `crates/oya-tenancy-adapter/` | `oya-tenancy-adapter` |
| `tenancy` | core | `crates/oya-tenancy-rest/` | `oya-tenancy-rest` |
| `tenancy` | core | `crates/oya-tenancy-grpc/` | `oya-tenancy-grpc` |
| `tenancy` | core | `crates/oya-tenancy-app/` | `oya-tenancy-app` |
| `tenancy` | core | `contracts/tenancy.openapi.yaml` | — |
| `tenancy` | core | `contracts/tenancy.proto` | — |
| `tenancy` | core | `migrations/tenancy/V001__tenancy_schema.sql` | — |

BC optionality: `tenancy` µservice has a single primary concept (the Tenant SaaS contract
plus its product-enablement / cell-placement sub-tables); BC token omitted per ADR-0056
BC-optionality rule.

Naming justification:

```
NAME: oya-tenancy-kernel
JUSTIFICATION:
- microservice = tenancy: the tenancy substrate µservice; registered in
  [workspace.metadata.oya.microservices]; ADR-0056 v4.1; Bominal ADR-0125
  naming canon (Tenant ≠ Organization — SaaS contract vs legal entity)
- bc-tokens: OMITTED — single primary concept (Tenant SaaS contract +
  product-enablement + cell-placement); BC-optionality rule applies
- layer = kernel: pure types (Tenant, TenantId, TenantStatus, TenantTier,
  MicroserviceId, CellId, Region) + sealed port traits (TenantStore,
  TenantProductRegistry, TenantCellPlacer); ADR-0056 §"Layer semantics"
- exemptions claimed: none
```

### Out-of-scope

- Tenant billing (invoices, subscription plans) — deferred to P18-cloud-tenancy / M03
- Tenant data import / migration runbooks — deferred to M03 (Bominal ADR-0118)
- Per-tenant feature flags beyond product-enablement — deferred to M03
- SSO hub per-tenant SAML/OIDC config — deferred to P19-application

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-tenancy-kernel-scaffold.md`](IP-001-tenancy-kernel-scaffold.md) | Scaffold oya-tenancy-kernel with all port traits + types; full DDL + RLS bootstrap function | pending | `council-architecture` |
| [`IP-002-tenancy-adapter-rest-grpc.md`](IP-002-tenancy-adapter-rest-grpc.md) | Postgres adapter (PgTenantStore, PgTenantProductRegistry, PgTenantCellPlacer) + REST + gRPC | pending | `council-architecture` |
| [`IP-003-tenancy-isolation-tests.md`](IP-003-tenancy-isolation-tests.md) | Mandatory cross-tenant isolation tests on all state-changing endpoints (ADR-0018) | pending | `council-architecture` |
| [`IP-004-tenancy-load-tests.md`](IP-004-tenancy-load-tests.md) | k6 + vegeta load tests; tenant lookup p99 ≤50ms; product registry check p99 ≤50ms | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P13-tenancy   # LEAN-A1
oya gate validate lean-a2 --phase P13-tenancy   # LEAN-A2
oya gate validate lean-a3 --phase P13-tenancy   # LEAN-A3
oya gate validate lean-a4 --phase P13-tenancy   # LEAN-A4
```

### Tenancy-specific gates

```bash
# RLS bootstrap function deployed
psql -c "SELECT oyatie.set_current_tenant(gen_random_uuid())"   # exit 0
# Cross-tenant isolation test passes
cargo nextest run -p oya-tenancy-adapter --test isolation_tenancy  # exit 0
# Product registry used by downstream µservices
oya gate validate product-registry-contract --phase P13-tenancy    # exit 0
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-tenancy-kernel` | `kernel` | Yes — TenantStore, TenantProductRegistry, TenantCellPlacer | N/A | No |
| `oya-tenancy-domain` | `domain` | N/A — calls through ports | N/A | No |
| `oya-tenancy-application` | `application` | N/A | N/A | No |
| `oya-tenancy-adapter` | `adapter` | N/A | Yes — PgTenantStore, PgTenantProductRegistry, PgTenantCellPlacer | No |
| `oya-tenancy-rest` | `rest` | N/A | No direct adapter import | Yes |
| `oya-tenancy-grpc` | `grpc` | N/A | No direct adapter import | Yes |
| `oya-tenancy-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-tenancy-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait TenantStore: Send + Sync + sealed::Sealed {
    async fn create(&self, draft: TenantDraft, owner_user_id: UserId) -> Result<Tenant, TenancyError>;
    async fn get(&self, tenant_id: TenantId) -> Result<Option<Tenant>, TenancyError>;
    async fn suspend(&self, tenant_id: TenantId, reason: SuspensionReason) -> Result<(), TenancyError>;
    async fn activate(&self, tenant_id: TenantId) -> Result<(), TenancyError>;
}

#[async_trait::async_trait]
pub trait TenantProductRegistry: Send + Sync + sealed::Sealed {
    async fn enable(&self, tenant_id: TenantId, microservice: &str, by: UserId) -> Result<(), TenancyError>;
    async fn disable(&self, tenant_id: TenantId, microservice: &str, by: UserId) -> Result<(), TenancyError>;
    async fn enabled(&self, tenant_id: TenantId) -> Result<Vec<MicroserviceId>, TenancyError>;
    async fn is_enabled(&self, tenant_id: TenantId, microservice: &str) -> Result<bool, TenancyError>;
    async fn set_tier_limits(&self, tenant_id: TenantId, microservice: &str, limits: TierLimits) -> Result<(), TenancyError>;
}

#[async_trait::async_trait]
pub trait TenantCellPlacer: Send + Sync + sealed::Sealed {
    async fn assign(&self, tenant_id: TenantId, region: Region) -> Result<CellId, TenancyError>;
    async fn primary_cell_for(&self, tenant_id: TenantId) -> Result<CellId, TenancyError>;
    async fn cells_for(&self, tenant_id: TenantId) -> Result<Vec<TenantCell>, TenancyError>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P13-tenancy` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P13-tenancy` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P13-tenancy` | exit 0 |
| `layer-correctness` | `oya gate validate layer-correctness --phase P13-tenancy` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P13-tenancy` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P13-tenancy` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `tenancy` | `tenancy` | pending |

---

## Grit Claim Symbols

```
crates/oya-tenancy-kernel/src/lib.rs::TenantStore
crates/oya-tenancy-kernel/src/lib.rs::TenantProductRegistry
crates/oya-tenancy-kernel/src/lib.rs::TenantCellPlacer
crates/oya-tenancy-adapter/src/lib.rs::PgTenantStore
crates/oya-tenancy-adapter/src/lib.rs::PgTenantProductRegistry
contracts/tenancy.openapi.yaml::createTenant
contracts/tenancy.proto::TenancyService
migrations/tenancy/V001__tenancy_schema.sql::tenancy.tenants
```

TTL recommendation: `--ttl 3600`. Fallback: ICM `scaffold-locks-oyatie`.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P13-tenancy started; milestone M02-substrate; scope: tenancy µservice; entry gate met: P03-identity complete" \
  -i high \
  -k "M02,P13,phase-start,tenancy"

icm store \
  -t context-oyatie \
  -c "Phase P13-tenancy complete; IPs IP-001..004 merged; RLS bootstrap deployed; TenantProductRegistry live; grit done; next: P14-policy" \
  -i high \
  -k "M02,P13,phase-complete,tenancy"
```

---

## References

- Milestone README: `../../README.md`
- Bominal ADRs inherited: ADR-0018 (tenancy + RLS posture), ADR-0009 (cell architecture), ADR-0125 (domain naming canon)
- oyatie ADRs cited: ADR-0056 v4.1 (BNF)
- M02-substrate-schema-foundation.md §4 (expanded here)
- Memory: `feedback_clean_architecture_requirements.md`, `feedback_flat_product_catalog.md`
