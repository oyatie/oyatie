---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P17-capability-registry
status: Proposed
acceptance_lanes: []
entry_gate: |
  M02/P14-policy complete; oya-policy-engine-kernel ships with PolicyEvaluator port;
  Cedar engine live; cargo check clean; grit done on all P14 symbols; ICM phase-handoff
  emitted.
exit_gate: |
  All P17 impl-plan acceptance gates green; per-tenant MCP endpoint registered and
  discoverable; LLM tool-call discovery API live; Cedar authorization gate on every
  capability invocation; 2 BCs registered (capability-registry, capability-bindings);
  all crates pass cargo check/build/clippy/nextest/deny; oya gate validate
  lean-a1/a2/a3/a4 exit 0; grit done on all P17 symbols; ICM phase-complete row emitted.
depends_on:
  - milestone: M02
    phase: P14-policy
    reason: "Every capability invocation is authorized through PolicyEvaluator before execution; per-tenant capability access policies stored as Cedar rule packs in policy.tenant_rule_packs."
owner_team: council-foundry
---

# P17-capability-registry: Capability Registry — MCP Gateway + Per-Tenant Endpoint + LLM Tool-Call Discovery

## Purpose

Delivers the capability registry: the MCP-compatible gateway that exposes per-tenant
capability endpoints for LLM tool-call discovery. Per Bominal ADR-0021 (OCI A1 launch
profile) and the Foundry internal-engine catalog, every tenant-enabled product's
capabilities are registered here and discoverable via the MCP protocol by agent runtimes.

The registry enables the "Implement the masterplan" autonomous-execution model
([[feedback-autonomous-implementation-artifacts]]): Foundry agents discover available
tools through the MCP gateway; Cedar policy gates ensure only authorized principals invoke
each capability; per-tenant endpoint isolation prevents cross-tenant capability leakage.

Two BCs: `capability-registry` (the MCP endpoint catalog + tool discovery) and
`capability-bindings` (per-tenant × per-product capability activation bindings, referencing
`tenancy.tenant_products` as the authority).

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `capability-registry` | `registry` | `crates/oya-capability-registry-kernel/` | `oya-capability-registry-kernel` |
| `capability-registry` | `registry` | `crates/oya-capability-registry-domain/` | `oya-capability-registry-domain` |
| `capability-registry` | `registry` | `crates/oya-capability-registry-application/` | `oya-capability-registry-application` |
| `capability-registry` | `registry` | `crates/oya-capability-registry-adapter/` | `oya-capability-registry-adapter` |
| `capability-registry` | `registry` | `crates/oya-capability-registry-rest/` | `oya-capability-registry-rest` |
| `capability-registry` | `registry` | `crates/oya-capability-registry-app/` | `oya-capability-registry-app` |
| `capability-registry` | `bindings` | `crates/oya-capability-bindings-kernel/` | `oya-capability-bindings-kernel` |
| `capability-registry` | `bindings` | `crates/oya-capability-bindings-adapter/` | `oya-capability-bindings-adapter` |
| `capability-registry` | all | `contracts/capability_registry.openapi.yaml` | — |
| `capability-registry` | all | `migrations/capability_registry/V001__capability_registry_schema.sql` | — |

Naming justification:

```
NAME: oya-capability-registry-kernel
JUSTIFICATION:
- microservice = capability-registry: MCP gateway µservice; Bominal ADR-0021;
  ADR-0056 v4.1 (2-token microservice name: capability-registry)
- bc-tokens = registry: the endpoint catalog BC; separate from bindings BC
- layer = kernel: CapabilityStore + ToolDiscoveryPort sealed ports; Capability +
  ToolManifest + McpEndpoint types; ZERO I/O
- exemptions claimed: none
```

### Out-of-scope

- WASM plugin execution sandbox (Bominal ADR-0161) — deferred to M03
- Capability versioning + deprecation lifecycle — deferred to M03
- Marketplace / capability publishing workflow — deferred to M03

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-capability-registry-kernel-scaffold.md`](IP-001-capability-registry-kernel-scaffold.md) | Scaffold all 8 capability-registry crates; MCP endpoint model; CapabilityStore + ToolDiscoveryPort; DDL | pending | `council-foundry` |
| [`IP-002-capability-registry-mcp-gateway.md`](IP-002-capability-registry-mcp-gateway.md) | MCP protocol handler; tool-call discovery endpoint; Cedar policy gate on invocation | pending | `council-foundry` |
| [`IP-003-capability-registry-load-tests.md`](IP-003-capability-registry-load-tests.md) | k6 load tests; tool discovery p99 ≤50ms; invocation auth p99 ≤10ms | pending | `council-foundry` |

---

## Acceptance Gates

### Cargo / CI gates

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
oya gate validate lean-a1 --phase P17-capability-registry
oya gate validate lean-a2 --phase P17-capability-registry
oya gate validate lean-a3 --phase P17-capability-registry
oya gate validate lean-a4 --phase P17-capability-registry
```

### Capability-specific gates

```bash
# MCP tool discovery returns only tenant-enabled capabilities
cargo nextest run -p oya-capability-registry-adapter --test tool_discovery_tenant_scoped  # exit 0
# Cedar policy gate blocks unauthorized invocations
cargo nextest run -p oya-capability-registry-adapter --test cedar_invocation_gate  # exit 0
# Cross-tenant isolation
cargo nextest run -p oya-capability-registry-adapter --test isolation_capability_registry  # exit 0
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-capability-registry-kernel` | `kernel` | Yes — CapabilityStore, ToolDiscoveryPort | N/A |
| `oya-capability-registry-domain` | `domain` | N/A | N/A |
| `oya-capability-registry-application` | `application` | N/A | N/A |
| `oya-capability-registry-adapter` | `adapter` | N/A | Yes — PgCapabilityStore, McpToolDiscoveryAdapter |
| `oya-capability-registry-rest` | `rest` | N/A | No direct adapter import |
| `oya-capability-registry-app` | `app` | N/A | Unrestricted inward |
| `oya-capability-bindings-kernel` | `kernel` | Yes — CapabilityBindingStore | N/A |
| `oya-capability-bindings-adapter` | `adapter` | N/A | Yes — PgCapabilityBindingStore |

### Port traits declared in kernel

```rust
// oya-capability-registry-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait CapabilityStore: Send + Sync + sealed::Sealed {
    async fn register(&self, tenant_id: TenantId, capability: CapabilityDraft) -> Result<CapabilityId, CapabilityError>;
    async fn get(&self, tenant_id: TenantId, capability_id: CapabilityId) -> Result<Option<Capability>, CapabilityError>;
    async fn list_enabled(&self, tenant_id: TenantId) -> Result<Vec<Capability>, CapabilityError>;
    async fn deregister(&self, tenant_id: TenantId, capability_id: CapabilityId) -> Result<(), CapabilityError>;
}

#[async_trait::async_trait]
pub trait ToolDiscoveryPort: Send + Sync + sealed::Sealed {
    /// Returns MCP-compatible tool manifest for all capabilities enabled for tenant
    async fn discover(&self, tenant_id: TenantId, principal_id: PrincipalId) -> Result<Vec<ToolManifest>, CapabilityError>;
    /// Returns the MCP endpoint URL for a specific capability
    async fn endpoint_for(&self, tenant_id: TenantId, capability_id: CapabilityId) -> Result<McpEndpoint, CapabilityError>;
}
```

### CI lanes that must green

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P17-capability-registry` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P17-capability-registry` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P17-capability-registry` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P17-capability-registry` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `capability-registry` | `capability-registry` | pending |
| `capability-bindings` | `capability-registry` | pending |

---

## Grit Claim Symbols

```
crates/oya-capability-registry-kernel/src/lib.rs::CapabilityStore
crates/oya-capability-registry-kernel/src/lib.rs::ToolDiscoveryPort
crates/oya-capability-bindings-kernel/src/lib.rs::CapabilityBindingStore
contracts/capability_registry.openapi.yaml::discoverTools
migrations/capability_registry/V001__capability_registry_schema.sql::capability.endpoints
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P17-capability-registry started; MCP gateway; per-tenant tool discovery; Cedar invocation gate; depends P14-policy" \
  -i high \
  -k "M02,P17,phase-start,capability-registry"

icm store \
  -t context-oyatie \
  -c "Phase P17-capability-registry complete; MCP endpoint live; tool discovery per-tenant; Cedar policy gate; next: P18-cloud-tenancy" \
  -i high \
  -k "M02,P17,phase-complete,capability-registry"
```

---

## References

- Bominal ADRs inherited: ADR-0021 (capability-registry MCP gateway), ADR-0007 (Cedar)
- oyatie ADRs cited: ADR-0056 v4.1
- M02-substrate-schema-foundation §6-N (capability-registry outlined)
