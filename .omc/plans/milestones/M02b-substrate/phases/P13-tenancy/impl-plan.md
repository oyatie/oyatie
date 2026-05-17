---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P13-tenancy
impl_plan_id: IP-001-tenancy-kernel-scaffold
status: pending
owner: council-architecture
blocked_by:
- impl_plan: P03-identity/IP-001
  reason: TenantStore.create() accepts UserId from oya-identity-kernel; tenant_admins
    table FKs to identity.users
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
purpose: "Scaffolds all 7 tenancy crates, authors the complete Postgres DDL (expanding M02b-substrate-schema-foundation §4 with full indexes + RLS + `oyatie.set_current_tenant()` function), implements all three kernel port traits with sealed markers."
execution_variant: merge-into-existing-crates
execution_variant_decided_at: 2026-05-17
execution_variant_decided_by: user-directive-option-2
execution_variant_note: "User chose merge-variant 2026-05-17 — FROM-SCRATCH scaffold below preserved as reference. First delta landed: TierStatus enum in oya-tenancy-kernel::tier_status (2026-05-17). Tracking: F-M02B-PLAN-LIVE-CRATE-RECONCILIATION."
---
# IP-001-tenancy-kernel-scaffold: Scaffold Tenancy Kernel, Domain, Application, Adapter, REST, gRPC, App — Full DDL + RLS Bootstrap

## Intent

Scaffolds all 7 tenancy crates, authors the complete Postgres DDL (expanding
M02b-substrate-schema-foundation §4 with full indexes + RLS + `oyatie.set_current_tenant()`
function), implements all three kernel port traits with sealed markers, and wires the
composition-root app binary. The RLS bootstrap function is the load-bearing primitive that
all downstream µservices depend on — it must be deployed and tested before any other
M02 µservice can claim RLS compliance.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add 7 tenancy workspace members |
| `crates/oya-tenancy-kernel/Cargo.toml` | create | Package manifest; zero framework deps |
| `crates/oya-tenancy-kernel/src/lib.rs` | create | pub mod types; pub mod ports; pub mod errors |
| `crates/oya-tenancy-kernel/src/types.rs` | create | Tenant, TenantId, TenantDraft, TenantStatus, TenantTier, MicroserviceId, CellId, Region, TenantCell, TierLimits, SuspensionReason |
| `crates/oya-tenancy-kernel/src/ports.rs` | create | TenantStore, TenantProductRegistry, TenantCellPlacer — all sealed |
| `crates/oya-tenancy-kernel/src/errors.rs` | create | TenancyError enum (thiserror) |
| `crates/oya-tenancy-domain/Cargo.toml` | create | Depends on oya-tenancy-kernel only |
| `crates/oya-tenancy-domain/src/lib.rs` | create | TenantDomainLogic: validate_microservice_name(), validate_region(), enforce_single_primary_cell() |
| `crates/oya-tenancy-application/Cargo.toml` | create | Depends on domain + kernel |
| `crates/oya-tenancy-application/src/lib.rs` | create | CreateTenantUseCase, EnableProductUseCase, DisableProductUseCase, AssignCellUseCase, SuspendTenantUseCase |
| `crates/oya-tenancy-adapter/Cargo.toml` | create | Depends on application + domain + kernel + sqlx + tokio |
| `crates/oya-tenancy-adapter/src/lib.rs` | create | module declarations |
| `crates/oya-tenancy-adapter/src/pg_tenant_store.rs` | create | PgTenantStore: impl TenantStore + sealed::Sealed; full CRUD on tenancy.tenants |
| `crates/oya-tenancy-adapter/src/pg_product_registry.rs` | create | PgTenantProductRegistry: impl TenantProductRegistry; validates microservice name against workspace registry |
| `crates/oya-tenancy-adapter/src/pg_cell_placer.rs` | create | PgTenantCellPlacer: impl TenantCellPlacer; enforces single primary cell uniqueness index |
| `crates/oya-tenancy-rest/Cargo.toml` | create | Depends on application + kernel; axum |
| `crates/oya-tenancy-rest/src/lib.rs` | create | REST handlers: POST /tenants, GET /tenants/{id}, POST /tenants/{id}/products, DELETE /tenants/{id}/products/{ms} |
| `crates/oya-tenancy-grpc/Cargo.toml` | create | Depends on application + kernel; tonic |
| `crates/oya-tenancy-grpc/src/lib.rs` | create | gRPC TenancyService handlers |
| `crates/oya-tenancy-app/Cargo.toml` | create | Composition root |
| `crates/oya-tenancy-app/src/main.rs` | create | DI assembly; PgPool → adapters → use-cases → REST + gRPC |
| `contracts/tenancy.openapi.yaml` | create | Full OpenAPI 3.1: createTenant, getTenant, enableProduct, disableProduct, listEnabledProducts, assignCell |
| `contracts/tenancy.proto` | create | Protobuf: TenancyService rpc CreateTenant / GetTenant / EnableProduct / IsProductEnabled / AssignCell |
| `migrations/tenancy/V001__tenancy_schema.sql` | create | Full DDL (see Code Shape) |
| `docs/standards/bounded-contexts.md` | update | Register `tenancy` BC |

---

## Crate Naming

```
NAME: oya-tenancy-kernel
JUSTIFICATION:
- microservice = tenancy: registered; Bominal ADR-0125 Tenant ≠ Organization canon
- bc-tokens: OMITTED — single concept (Tenant SaaS contract); BC-optionality rule
- layer = kernel: pure types + sealed ports; no I/O; ADR-0056
- exemptions claimed: none

NAME: oya-tenancy-adapter
JUSTIFICATION:
- microservice = tenancy, bc-tokens OMITTED: same rationale
- layer = adapter: Postgres impls of TenantStore + TenantProductRegistry + TenantCellPlacer
- exemptions claimed: none
```

---

## Code Shape

### `crates/oya-tenancy-kernel/src/types.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TenantId = Uuid;
pub type UserId = Uuid;  // re-imported from oya-identity-kernel in app layer
pub type MicroserviceId = String;
pub type CellId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus { Active, Suspended, Terminated }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantTier { Starter, Pro, Enterprise }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Region { Kr, Us, Eu, Jp }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub tenant_id: TenantId,
    pub display_name: String,
    pub status: TenantStatus,
    pub tier: TenantTier,
    pub region: Region,
    pub primary_jurisdiction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDraft {
    pub display_name: String,
    pub tier: TenantTier,
    pub region: Region,
    pub primary_jurisdiction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantCell {
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub region: Region,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierLimits {
    pub max_users: Option<u32>,
    pub max_storage_gb: Option<u32>,
    pub custom: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspensionReason {
    PaymentOverdue,
    TosViolation,
    AdminRequest,
    Other(String),
}
```

### `migrations/tenancy/V001__tenancy_schema.sql`

```sql
CREATE SCHEMA IF NOT EXISTS tenancy;

-- The RLS bootstrap function used by ALL downstream µservices
CREATE SCHEMA IF NOT EXISTS oyatie;
CREATE OR REPLACE FUNCTION oyatie.set_current_tenant(p_tenant_id uuid)
    RETURNS void
    LANGUAGE plpgsql
    SECURITY DEFINER
AS $$
BEGIN
    PERFORM set_config('oyatie.tenant_id', p_tenant_id::text, true);  -- LOCAL to txn
END $$;

CREATE TABLE tenancy.tenants (
    tenant_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name text NOT NULL,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspended','terminated')),
    tier text NOT NULL DEFAULT 'starter' CHECK (tier IN ('starter','pro','enterprise')),
    region text NOT NULL CHECK (region IN ('KR','US','EU','JP')),
    primary_jurisdiction text NOT NULL DEFAULT 'KR',
    created_at timestamptz NOT NULL DEFAULT now(),
    activated_at timestamptz NULL,
    suspended_at timestamptz NULL
);
COMMENT ON TABLE tenancy.tenants IS 'distribution_column:tenant_id';

CREATE TABLE tenancy.tenant_products (
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants(tenant_id) ON DELETE CASCADE,
    microservice text NOT NULL,   -- validated against [workspace.metadata.oya.microservices]
    enabled bool NOT NULL DEFAULT false,
    enabled_at timestamptz NULL,
    disabled_at timestamptz NULL,
    tier_limits jsonb NOT NULL DEFAULT '{}'::jsonb,
    config jsonb NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (tenant_id, microservice)
);
ALTER TABLE tenancy.tenant_products FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenancy.tenant_products
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE TABLE tenancy.tenant_cells (
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants(tenant_id) ON DELETE CASCADE,
    cell_id text NOT NULL,
    region text NOT NULL CHECK (region IN ('KR','US','EU','JP')),
    is_primary bool NOT NULL DEFAULT false,
    PRIMARY KEY (tenant_id, cell_id)
);
ALTER TABLE tenancy.tenant_cells FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenancy.tenant_cells
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
-- Enforce exactly one primary cell per tenant
CREATE UNIQUE INDEX idx_tenant_primary_cell ON tenancy.tenant_cells (tenant_id)
    WHERE is_primary = true;

CREATE TABLE tenancy.tenant_admins (
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants(tenant_id) ON DELETE CASCADE,
    user_id uuid NOT NULL,  -- FK to identity.users enforced at application layer (cross-schema)
    role text NOT NULL CHECK (role IN ('owner','admin','billing_admin','security_admin')),
    invited_by uuid NOT NULL,
    invited_at timestamptz NOT NULL DEFAULT now(),
    accepted_at timestamptz NULL,
    PRIMARY KEY (tenant_id, user_id)
);
ALTER TABLE tenancy.tenant_admins FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenancy.tenant_admins
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_tenant_products_enabled ON tenancy.tenant_products (tenant_id)
    WHERE enabled = true;
CREATE INDEX idx_tenants_status ON tenancy.tenants (status, region)
    WHERE status = 'active';
```

---

## Acceptance Gates

```bash
# 1. Compile all tenancy crates
cargo check -p oya-tenancy-kernel --all-features             # exit 0
cargo check -p oya-tenancy-domain --all-features             # exit 0
cargo check -p oya-tenancy-application --all-features        # exit 0
cargo check -p oya-tenancy-adapter --all-features            # exit 0
cargo check --workspace --all-features                       # exit 0

# 2. Build
cargo build --workspace --all-features                       # exit 0

# 3. Lint
cargo clippy --workspace --all-features -- -D warnings       # exit 0

# 4. Tests (including mandatory isolation tests per ADR-0018)
cargo nextest run --workspace --all-features                 # exit 0; 0 failures
cargo nextest run -p oya-tenancy-adapter --test isolation_tenancy  # exit 0

# 5. Supply chain
cargo deny check                                             # exit 0

# 6. LEAN checks
oya gate validate lean-a1 --phase P13-tenancy
oya gate validate lean-a2 --phase P13-tenancy
oya gate validate lean-a3 --phase P13-tenancy
oya gate validate lean-a4 --phase P13-tenancy
oya gate validate shardability --phase P13-tenancy           # tenant_id declared on all tables
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_tenant_draft_validation` | display_name non-empty; region is valid enum; jurisdiction non-empty |
| `test_tenant_product_registry_mock` | enable → is_enabled returns true; disable → is_enabled returns false |
| `test_cell_placer_single_primary_constraint` | Assigning second primary cell to same tenant returns error |
| `test_tier_limits_json_round_trip` | TierLimits serialize/deserialize without loss |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_pg_create_tenant` | PgTenantStore.create() inserts row; get() returns same data |
| `integration_pg_enable_product` | PgTenantProductRegistry.enable() then is_enabled() returns true |
| `integration_pg_isolation_tenancy` | Tenant A cannot read tenant B's tenant_products (RLS check) |
| `integration_rls_bootstrap_function` | oyatie.set_current_tenant() correctly sets session var; subsequent SELECT on tenant_products filtered |
| `integration_primary_cell_uniqueness` | Assigning two primary cells to same tenant fails at DB constraint |

### E2E / acceptance tests

| Scenario | Command | Expected output |
|---|---|---|
| Create tenant + enable products | `cargo nextest run --test e2e_tenancy` | PASS |
| RLS isolation end-to-end | `cargo nextest run --test isolation_tenancy` | PASS |

---

## Clean Architecture Compliance

### Dependency direction

| Crate | Layer | Imports | Forbidden |
|---|---|---|---|
| `oya-tenancy-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-tenancy-domain` | `domain` | `kernel` | `application`, `adapter`, presentation, `app` |
| `oya-tenancy-application` | `application` | `domain`, `kernel` | `adapter`, presentation, `app` |
| `oya-tenancy-adapter` | `adapter` | `application`, `domain`, `kernel` | presentation, `app` |
| `oya-tenancy-rest` | `rest` | `application`, `kernel` | `adapter` directly |
| `oya-tenancy-app` | `app` | every layer | (none) |

### Cross-product check

Tenancy µservice imports ONLY from its own layers + `oya-identity-kernel` (for UserId type
at application/adapter layer — kernel-only dep, which is explicitly permitted per clean-arch
rule: sdk/kernel deps between µservices allowed; no cross-µservice non-kernel imports).

---

## Load Test

```javascript
// tests/load/smoke-tenancy.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 100,
  duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<50'],   // p99 ≤50ms for product registry check (read-only Ontology Function target)
    http_req_failed: ['rate<0.001'],
  },
};

export default function () {
  const res = http.get(
    `${__ENV.BASE_URL}/api/v1/tenants/${__ENV.TENANT_ID}/products`,
    { headers: { 'X-Tenant-Id': __ENV.TENANT_ID } }
  );
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(0.05);
}
```

| Scenario | Tool | Target | Pass criterion |
|---|---|---|---|
| List enabled products | k6 | p99 ≤50ms at 2k RPS | `http_req_duration{p(99)}<50` |
| Create tenant | k6 | p99 ≤200ms at 200 RPS | `http_req_duration{p(99)}<200` |
| Enable product | k6 | p99 ≤200ms at 500 RPS | `http_req_duration{p(99)}<200` |

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-architecture \
  --intent "IP-001-tenancy-kernel-scaffold: full tenancy substrate + RLS bootstrap" \
  --ttl 3600 \
  crates/oya-tenancy-kernel/src/lib.rs::TenantStore \
  crates/oya-tenancy-kernel/src/lib.rs::TenantProductRegistry \
  crates/oya-tenancy-kernel/src/lib.rs::TenantCellPlacer \
  migrations/tenancy/V001__tenancy_schema.sql::oyatie.set_current_tenant
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-tenancy-kernel-scaffold merged; crates: oya-tenancy-kernel/domain/application/adapter/rest/grpc/app; oyatie.set_current_tenant() deployed; RLS on all 4 tables; isolation tests pass; next IP: IP-002-tenancy-adapter-rest-grpc" \
  -i high \
  -k "M02,P13,IP-001,tenancy"
```

---

## Halt Conditions

1. `cargo check` fails after 3 attempts.
2. `oyatie.set_current_tenant()` does not correctly set `oyatie.tenant_id` session variable — escalate; this is the root RLS primitive for all µservices.
3. Cross-tenant isolation test fails after 3 fix attempts — escalate; do NOT weaken the RLS policy.
4. LEAN-A2 violation: tenancy importing a non-kernel product crate — escalate.

---

## Next IP Pointer

`IP-002-tenancy-adapter-rest-grpc.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0056 (BNF v4.1), ADR-0018 (tenancy + RLS), ADR-0009 (cell architecture), ADR-0125 (naming canon)
- M02b-substrate-schema-foundation.md §4
