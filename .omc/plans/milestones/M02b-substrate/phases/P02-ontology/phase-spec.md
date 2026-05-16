---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02b-substrate
phase: P02-ontology
status: Proposed
acceptance_lanes: []
entry_gate: 'M01-P05 complete; cargo check --workspace exits 0; Postgres 16 available

  in dev environment; oya-tenancy-kernel exists (tenancy RLS bootstrap function

  oyatie.set_current_tenant must be callable).

  '
exit_gate: "All ontology crates compile; migrations/V001__ontology_init.sql applied;\n\
  RLS policies verified; port traits compile with sealed marker; Cedar policy\nfragment\
  \ lints; Protobuf compiles; k6 smoke test passes p99\u226450ms; grit done\non all\
  \ symbols; ICM phase-handoff row emitted.\n"
depends_on:
- milestone: M01
  phase: P05-scaffold-locks
  reason: workspace scaffold + tenancy kernel prerequisite
owner_team: council-architecture
purpose: This phase delivers the complete Ontology substrate, the information-plane adapter that all oyatie products use for typed-entity storage, link traversal, action execution, function evaluation, and LLM agent-gateway ingress.
---
# P02-ontology: Full Ontology substrate — typed entity/link/action/function/agent-gateway/audit-chain/pillar layers

## Purpose

This phase delivers the complete Ontology substrate, the information-plane adapter that all oyatie products use for typed-entity storage, link traversal, action execution, function evaluation, and LLM agent-gateway ingress. Ontology is the "Palantir Ontology equivalent" — every cross-product data share flows through it (per `feedback_workflow_objectgraph_adapter_layer.md`). Without Ontology no product can store typed entities, no LLM tool-call can query data, and no audit chain has a provenance anchor. The phase ships 7 bounded-context families (entity, link, action, function, agent-gateway, audit-chain, pillar) across 6 layers each (kernel, domain, application, adapter, worker, rest/graphql), advancing Master Plan principles: "Ontology as information adapter" and "audit-chain provenance from day one".

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `ontology` | `entity`, `link`, `action`, `function`, `agent-gateway`, `audit-chain`, `pillar` | `crates/oya-ontology-{entity,link,action,function,agent-gateway,audit-chain,pillar}-{kernel,domain,application,adapter,worker}/`, `crates/oya-ontology-rest/`, `crates/oya-ontology-graphql/`, `crates/oya-ontology-app/` | `oya-ontology-entity-kernel`, `oya-ontology-entity-domain`, `oya-ontology-entity-application`, `oya-ontology-entity-adapter`, `oya-ontology-entity-worker`, `oya-ontology-link-kernel`, … (full matrix below) |

Full crate matrix (42 crates + 2 presentation + 1 app = 45 total):

```
oya-ontology-entity-kernel          oya-ontology-link-kernel
oya-ontology-entity-domain          oya-ontology-link-domain
oya-ontology-entity-application     oya-ontology-link-application
oya-ontology-entity-adapter         oya-ontology-link-adapter
oya-ontology-entity-worker          oya-ontology-link-worker

oya-ontology-action-kernel          oya-ontology-function-kernel
oya-ontology-action-domain          oya-ontology-function-domain
oya-ontology-action-application     oya-ontology-function-application
oya-ontology-action-adapter         oya-ontology-function-adapter
oya-ontology-action-worker          oya-ontology-function-worker

oya-ontology-agent-gateway-kernel   oya-ontology-audit-chain-kernel
oya-ontology-agent-gateway-domain   oya-ontology-audit-chain-domain
oya-ontology-agent-gateway-application  oya-ontology-audit-chain-application
oya-ontology-agent-gateway-adapter  oya-ontology-audit-chain-adapter
oya-ontology-agent-gateway-worker   oya-ontology-audit-chain-worker

oya-ontology-pillar-kernel
oya-ontology-pillar-domain
oya-ontology-pillar-application
oya-ontology-pillar-adapter

oya-ontology-rest                   # unified REST presentation
oya-ontology-graphql                # GraphQL presentation
oya-ontology-app                    # composition root
```

Naming justification (representative):

```
NAME: oya-ontology-entity-kernel
JUSTIFICATION:
- microservice = ontology: the Palantir-Ontology-equivalent information adapter;
  registered in [workspace.metadata.oya.microservices]; ADR-0056 v4.1 flat BNF
- bc-tokens = entity: the Object-Type (typed entity) bounded context within Ontology;
  distinct from link/action/function BCs; ADR-0056 v4.1 BC-optionality — multiple BCs
  exist at kernel layer so BC token is included
- layer = kernel: pure types + port trait declarations; ObjectStore, LinkStore,
  ActionStore, OntologyFunction traits; zero I/O; ADR-0056 §"Layer semantics"
- exemptions claimed: none

NAME: oya-ontology-agent-gateway-kernel
JUSTIFICATION:
- microservice = ontology: same µservice
- bc-tokens = agent-gateway: LLM tool-call ingress BC per Bominal ADR-0107;
  exposes Ontology Functions as MCP/tool-call endpoints for LLM consumers
- layer = kernel: AgentGatewayPort trait declarations
- exemptions claimed: none
```

### Out-of-scope

- Workflow engine integration — owned by Wave-B; Ontology publishes typed events; Workflow subscribes.
- Product-specific Object Type schemas (medical.Encounter, hr.Employee) — registered by product phases in Wave-C/D.
- pgroonga/Tantivy full-text search indexing on Ontology payloads — owned by P09-search.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full DDL + port traits + Cedar + Proto + REST/GraphQL + load test | pending | `council-architecture` |

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
oya gate validate lean-a1 --phase P02-ontology   # LEAN-A1: layer ordering
oya gate validate lean-a2 --phase P02-ontology   # LEAN-A2: cross-vertical refusal
oya gate validate lean-a3 --phase P02-ontology   # LEAN-A3: BC boundary
oya gate validate lean-a4 --phase P02-ontology   # LEAN-A4: naming conformance
```

### Workflow + Ontology integration gates

```bash
oya gate validate ontology-type-registry --phase P02-ontology
oya gate validate workflow-event-registry --phase P02-ontology
```

### Migration gate

```bash
psql $DATABASE_URL -f migrations/V001__ontology_init.sql  # exit 0; no errors
# Verify RLS
psql $DATABASE_URL -c "SET oyatie.tenant_id = 'test-uuid'; SELECT count(*) FROM ontology.objects;"  # 0 rows
```

### Load test gate

```bash
k6 run tests/load/smoke-ontology-entity.js --env BASE_URL=http://localhost:8080
# Pass: p99 ≤50ms (read-only Ontology Functions per quality bar)
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-ontology-entity-kernel` | `kernel` | Yes — `ObjectStore`, `OntologyFunction` | N/A | No |
| `oya-ontology-link-kernel` | `kernel` | Yes — `LinkStore` | N/A | No |
| `oya-ontology-action-kernel` | `kernel` | Yes — `ActionStore` | N/A | No |
| `oya-ontology-agent-gateway-kernel` | `kernel` | Yes — `AgentGatewayPort` | N/A | No |
| `oya-ontology-audit-chain-kernel` | `kernel` | Yes — `AuditChainEmitter` | N/A | No |
| `oya-ontology-pillar-kernel` | `kernel` | Yes — `PillarAssignmentPort` | N/A | No |
| `oya-ontology-entity-domain` | `domain` | N/A — calls through `ObjectStore` | N/A | No |
| `oya-ontology-entity-adapter` | `adapter` | N/A | Yes — Postgres `ObjectStore` impl | No |
| `oya-ontology-rest` | `rest` | N/A | No direct adapter import | Yes |
| `oya-ontology-app` | `app` | N/A | Unrestricted inward (wiring only) | No |

### Port traits declared in kernel (all new ones)

```rust
// oya-ontology-entity-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait ObjectStore: Send + Sync + sealed::Sealed {
    async fn get(&self, tenant_id: TenantId, object_id: ObjectId)
        -> Result<Option<TypedObject>, OntologyError>;
    async fn put(&self, tenant_id: TenantId, action: TypedAction)
        -> Result<TypedObject, OntologyError>;
    async fn query(&self, tenant_id: TenantId, q: ObjectQuery)
        -> Result<Vec<TypedObject>, OntologyError>;
    async fn delete_soft(&self, tenant_id: TenantId, object_id: ObjectId)
        -> Result<(), OntologyError>;
}

pub trait OntologyFunction: Send + Sync + sealed::Sealed {
    fn call(&self, tenant_id: TenantId, input: FunctionInput)
        -> Result<FunctionOutput, OntologyError>;
}

// oya-ontology-link-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait LinkStore: Send + Sync + sealed::Sealed {
    async fn link(&self, tenant_id: TenantId, link: TypedLink) -> Result<LinkId, OntologyError>;
    async fn unlink(&self, tenant_id: TenantId, link_id: LinkId) -> Result<(), OntologyError>;
    async fn traverse(&self, tenant_id: TenantId, from: ObjectId, via: LinkType, depth: u8)
        -> Result<Vec<TypedLink>, OntologyError>;
}

// oya-ontology-action-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait ActionStore: Send + Sync + sealed::Sealed {
    async fn apply(&self, tenant_id: TenantId, action: TypedAction)
        -> Result<ActionResult, OntologyError>;
    async fn reverse(&self, tenant_id: TenantId, action_id: ActionId)
        -> Result<(), OntologyError>;
    async fn get_outcome(&self, tenant_id: TenantId, action_id: ActionId)
        -> Result<ActionOutcome, OntologyError>;
}

// oya-ontology-agent-gateway-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait AgentGatewayPort: Send + Sync + sealed::Sealed {
    async fn invoke_function(&self, tenant_id: TenantId, tool_call: ToolCallRequest)
        -> Result<ToolCallResponse, OntologyError>;
    async fn list_functions(&self, tenant_id: TenantId)
        -> Result<Vec<FunctionDescriptor>, OntologyError>;
}

// oya-ontology-pillar-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait PillarAssignmentPort: Send + Sync + sealed::Sealed {
    async fn assign(&self, tenant_id: TenantId, object_id: ObjectId, pillar: Pillar)
        -> Result<(), OntologyError>;
    async fn get_pillar(&self, tenant_id: TenantId, object_id: ObjectId)
        -> Result<Pillar, OntologyError>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P02-ontology` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P02-ontology` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P02-ontology` | exit 0 |
| `layer-correctness` | `oya gate validate layer-correctness --phase P02-ontology` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P02-ontology` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P02-ontology` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `entity` | `ontology` | pending |
| `link` | `ontology` | pending |
| `action` | `ontology` | pending |
| `function` | `ontology` | pending |
| `agent-gateway` | `ontology` | pending |
| `audit-chain` | `ontology` | pending |
| `pillar` | `ontology` | pending |

---

## Grit Claim Symbols

```
crates/oya-ontology-entity-kernel/src/ports.rs::ObjectStore
crates/oya-ontology-link-kernel/src/ports.rs::LinkStore
crates/oya-ontology-action-kernel/src/ports.rs::ActionStore
crates/oya-ontology-function-kernel/src/ports.rs::OntologyFunction
crates/oya-ontology-agent-gateway-kernel/src/ports.rs::AgentGatewayPort
crates/oya-ontology-pillar-kernel/src/ports.rs::PillarAssignmentPort
crates/oya-ontology-entity-adapter/src/postgres.rs::PgObjectStore
migrations/ontology/V001__ontology_init.sql::ontology_schema
contracts/ontology.proto::ObjectMutated
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P02-ontology started; milestone M02b-substrate; scope: 45 crates across 7 BCs; entry gate met: M01-P05 complete" \
  -i high \
  -k "M02,P02,phase-start,ontology"

icm store \
  -t context-oyatie \
  -c "Phase P02-ontology complete; IPs merged: impl-plan; ontology DDL applied; RLS verified; Cedar linted; k6 p99≤50ms; next phase: P03-identity" \
  -i high \
  -k "M02,P02,phase-complete,ontology"
```

---

## References

- Milestone README: `../../README.md`
- Bominal ADRs inherited: ADR-0106 (Ontology architecture), ADR-0107 (agent gateway), ADR-0132 (pillars)
- oyatie ADRs cited: ADR-0056 (BNF v4.1), ADR-0006 (Ontology kernel)
- Memory files: `feedback_workflow_objectgraph_adapter_layer.md`, `feedback_clean_architecture_requirements.md`
- depends_on: M01-P05
- unblocks: Wave-B product phases (medical, hr, payroll, connect, etc.) — all write to Ontology
