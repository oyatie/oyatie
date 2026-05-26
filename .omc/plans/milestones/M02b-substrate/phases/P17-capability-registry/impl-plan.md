---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P17-capability-registry
impl_plan_id: IP-001-capability-registry-kernel-scaffold
status: pending
owner: council-foundry
execution_variant: merge-into-existing-crates
decided_at: "2026-05-17"
decided_by: user-directive-option-2
execution_variant_note: >
  User directive 2026-05-17 (option 2): do NOT scaffold the 8 new crates listed in
  the impl-plan. Instead, backport the smallest net-new type into the existing live
  oya-intelligence-capability-registry-kernel crate. Delta-1 adds CapabilityStatus
  (Active/Deprecated/Disabled) via a new status.rs module + re-export.
blocked_by:
- impl_plan: P14-policy/IP-001
  reason: McpToolDiscoveryAdapter calls PolicyEvaluator to gate capability invocations
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
purpose: Scaffolds all 8 capability-registry crates across 2 BCs, implements the MCP-compatible tool discovery endpoint, wires the Cedar policy gate on every capability invocation, and authors the full DDL.
---
# IP-001-capability-registry-kernel-scaffold: Scaffold Capability Registry + Bindings Kernel/Domain/Application/Adapter/REST/App — MCP Gateway + DDL

## Intent

Scaffolds all 8 capability-registry crates across 2 BCs, implements the MCP-compatible
tool discovery endpoint, wires the Cedar policy gate on every capability invocation, and
authors the full DDL. After this IP merges, Foundry agent runtimes can call
`ToolDiscoveryPort::discover()` to enumerate tenant-enabled capabilities, and each
invocation is Cedar-authorized before execution.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add 8 capability-registry workspace members |
| `crates/oya-capability-registry-kernel/Cargo.toml` | create | Zero framework deps |
| `crates/oya-capability-registry-kernel/src/lib.rs` | create | pub mod types; pub mod ports; pub mod errors |
| `crates/oya-capability-registry-kernel/src/types.rs` | create | Capability, CapabilityId, CapabilityDraft, CapabilityStatus, ToolManifest, McpEndpoint, McpToolSchema, PrincipalId |
| `crates/oya-capability-registry-kernel/src/ports.rs` | create | CapabilityStore + ToolDiscoveryPort — sealed |
| `crates/oya-capability-registry-kernel/src/errors.rs` | create | CapabilityError enum |
| `crates/oya-capability-registry-domain/Cargo.toml` | create | Depends on kernel only |
| `crates/oya-capability-registry-domain/src/lib.rs` | create | CapabilityNameValidator; mcp_tool_schema_from_capability(); filter_by_tenant_products() |
| `crates/oya-capability-registry-application/Cargo.toml` | create | Depends on domain + kernel |
| `crates/oya-capability-registry-application/src/lib.rs` | create | RegisterCapabilityUseCase, DiscoverToolsUseCase, GetCapabilityEndpointUseCase |
| `crates/oya-capability-registry-adapter/Cargo.toml` | create | Depends on application + domain + kernel + oya-policy-engine-kernel + oya-tenancy-kernel + sqlx |
| `crates/oya-capability-registry-adapter/src/lib.rs` | create | module declarations |
| `crates/oya-capability-registry-adapter/src/pg_capability_store.rs` | create | PgCapabilityStore: impl CapabilityStore + sealed::Sealed; RLS |
| `crates/oya-capability-registry-adapter/src/mcp_discovery.rs` | create | McpToolDiscoveryAdapter: impl ToolDiscoveryPort; filters by TenantProductRegistry; Cedar gate on each tool |
| `crates/oya-capability-registry-rest/Cargo.toml` | create | axum; depends on application + kernel |
| `crates/oya-capability-registry-rest/src/lib.rs` | create | GET /capabilities/discover, GET /capabilities/{id}/endpoint, POST /capabilities |
| `crates/oya-capability-registry-app/Cargo.toml` | create | Composition root |
| `crates/oya-capability-registry-app/src/main.rs` | create | DI assembly |
| `crates/oya-capability-bindings-kernel/Cargo.toml` | create | Zero framework deps |
| `crates/oya-capability-bindings-kernel/src/lib.rs` | create | CapabilityBindingStore port; CapabilityBinding type |
| `crates/oya-capability-bindings-adapter/Cargo.toml` | create | Depends on bindings-kernel + sqlx |
| `crates/oya-capability-bindings-adapter/src/lib.rs` | create | PgCapabilityBindingStore |
| `contracts/capability_registry.openapi.yaml` | create | OpenAPI 3.1: discoverTools, getCapabilityEndpoint, registerCapability |
| `migrations/capability_registry/V001__capability_registry_schema.sql` | create | Full DDL |
| `docs/standards/bounded-contexts.md` | update | Register capability-registry + capability-bindings BCs |

---

## Code Shape

### `migrations/capability_registry/V001__capability_registry_schema.sql`

```sql
CREATE SCHEMA IF NOT EXISTS capability;

-- Capability endpoint catalog
CREATE TABLE capability.endpoints (
    capability_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    microservice text NOT NULL,       -- which product this capability belongs to
    capability_name text NOT NULL,
    description text NOT NULL,
    mcp_schema jsonb NOT NULL,        -- MCP tool schema (inputSchema + outputSchema)
    endpoint_url text NOT NULL,       -- per-tenant MCP endpoint URL
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','deprecated','disabled')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE capability.endpoints FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON capability.endpoints
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_capability_name ON capability.endpoints
    (tenant_id, microservice, capability_name) WHERE status = 'active';
CREATE INDEX idx_capability_microservice ON capability.endpoints
    (tenant_id, microservice) WHERE status = 'active';
COMMENT ON TABLE capability.endpoints IS 'distribution_column:tenant_id';

-- Per-tenant × per-product capability binding activation
CREATE TABLE capability.bindings (
    binding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    capability_id uuid NOT NULL REFERENCES capability.endpoints(capability_id),
    enabled bool NOT NULL DEFAULT true,
    enabled_by uuid NOT NULL,
    enabled_at timestamptz NULL,
    disabled_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE capability.bindings FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON capability.bindings
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_binding_capability ON capability.bindings
    (tenant_id, capability_id) WHERE enabled = true;
COMMENT ON TABLE capability.bindings IS 'distribution_column:tenant_id';
```

---

## Acceptance Gates

```bash
cargo check --workspace --all-features                                               # exit 0
cargo build --workspace --all-features                                               # exit 0
cargo clippy --workspace --all-features -- -D warnings                               # exit 0
cargo nextest run --workspace --all-features                                         # exit 0
cargo nextest run -p oya-capability-registry-adapter --test tool_discovery_tenant_scoped  # exit 0
cargo nextest run -p oya-capability-registry-adapter --test cedar_invocation_gate    # exit 0
cargo nextest run -p oya-capability-registry-adapter --test isolation_capability_registry  # exit 0
cargo deny check                                                                     # exit 0
oya gate validate lean-a1 --phase P17-capability-registry
oya gate validate lean-a2 --phase P17-capability-registry
oya gate validate shardability --phase P17-capability-registry
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_capability_name_validator` | Empty name rejected; name > 128 chars rejected |
| `test_mcp_tool_schema_generation` | mcp_tool_schema_from_capability() returns valid MCP JSON schema |
| `test_filter_by_tenant_products` | Only capabilities for enabled products returned |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_tool_discovery_tenant_scoped` | Tenant A tools not returned for Tenant B discover() |
| `integration_cedar_invocation_gate` | Principal without capability:invoke → PolicyEffect::Deny |
| `integration_isolation_capability_registry` | Tenant A cannot read Tenant B capability.endpoints |
| `integration_capability_binding_enable_disable` | enable() → binding row; disable() → enabled=false |

---

## Load Test

| Scenario | Target | Pass criterion |
|---|---|---|
| Tool discovery (20 capabilities) | p99 ≤50ms at 2k RPS | `http_req_duration{p(99)}<50` |
| Cedar invocation gate | p99 ≤10ms at 10k RPS | `http_req_duration{p(99)}<10` |

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-foundry \
  --intent "IP-001-capability-registry-kernel-scaffold: MCP gateway + Cedar gate" \
  --ttl 3600 \
  crates/oya-capability-registry-kernel/src/lib.rs::CapabilityStore \
  crates/oya-capability-registry-kernel/src/lib.rs::ToolDiscoveryPort \
  migrations/capability_registry/V001__capability_registry_schema.sql::capability.endpoints
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-capability-registry-kernel-scaffold merged; MCP tool discovery live; Cedar gate on invocations; per-tenant endpoint isolation; next: IP-002-capability-registry-mcp-gateway" \
  -i high \
  -k "M02,P17,IP-001,capability-registry"
```

---

## Halt Conditions

1. Cedar gate bypassable via direct adapter call — escalate; invocation auth is mandatory.
2. Tool discovery leaks capabilities from another tenant — escalate; cross-tenant isolation failure.
3. LEAN-A2 violation: capability-registry importing a product crate — escalate.

---

## Next IP Pointer

`IP-002-capability-registry-mcp-gateway.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0021 (capability-registry), ADR-0007 (Cedar), ADR-0056 (BNF v4.1)
