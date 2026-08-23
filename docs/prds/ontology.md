---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-ontology
microservice: ontology
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M02b-substrate-ready
bominal_source:
  - ADR-0106  # Ontology architecture (typed Object/Link/Action/Function; RLS; audit; jurisdiction; plugin SDK; multi-renderer)
  - ADR-0107  # Ontology agent gateway (LLM tool-call ingress)
  - ADR-0108  # property types: vector
  - ADR-0109  # property types: geo
  - ADR-0110  # property types: timeseries
  - ADR-0111  # property types: ciphertext
  - ADR-0112  # property types: struct
  - ADR-0132  # data ownership pillars (org-pillar / person-pillar)
  - ADR-0018  # tenancy RLS posture
  - ADR-0028  # audit chain Merkle/Ed25519
doc_status: published
---

# PRD-ontology: Ontology shared substrate

---

## Purpose

Ontology is the information layer of the oyatie ecosystem — the Palantir
Ontology equivalent (per `feedback_glossary_ontology_not_object_graph.md`).
It provides typed Object Types, Link Types, Action Types, and Functions with
multi-tenant RLS, Merkle/Ed25519 audit chain, jurisdiction overlays, org/person
pillar enforcement, and a plugin SDK for custom renderers.

All cross-µservice data sharing flows through Ontology. No µservice reads
another µservice's Postgres tables directly. Products write Object Types and
read them via Ontology Functions — Ontology is the canonical data adapter
(per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`).

Inherits from Bominal ADR-0106 (Ontology architecture) 1:1 with one naming
override: Bominal calls this "Object Graph"; oyatie renames it "Ontology"
(per `feedback_glossary_ontology_not_object_graph.md` — oyatie override #2).
All architectural content is adopted unchanged.

---

## Tenant Value

Ontology is internal substrate; value manifests through every product built on it.

- **Typed entity graph**: every entity in the system (Employee, Payslip,
  Workflow run, Message, etc.) is a typed Object Type queryable via a
  unified API — no per-µservice ad-hoc APIs for cross-product reads.
- **Agent-ready data layer**: LLMs query Ontology via the agent gateway
  (ADR-0107 tool-call ingress) to power agentic workflows without raw DB access.
- **Zero-leakage tenancy**: every Object Type row carries `tenant_id` enforced
  by Postgres RLS; cross-tenant reads are impossible at the DB level.
- **Audit-grade provenance**: every Object Type mutation sealed in Merkle/Ed25519
  audit chain; tamper-evident history for every entity.
- **Jurisdiction overlays**: Cedar policy overlays applied per
  `tenant.jurisdiction_code`; GDPR, PIPA, HIPAA property-tier enforcement.

---

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | Product µservice (writer) | register an Object Type schema (name, properties, pillar) | my entities are queryable by other µservices via Ontology | `schema-registry` | Must |
| FR-02 | Product µservice (writer) | write Object Type instances with `tenant_id`; receive audit seal | data is isolated and provenance-tracked from write moment | `entity` | Must |
| FR-03 | Product µservice (reader) | query Object Types via Functions with `tenant_id` filter | cross-product reads without importing the writer µservice | `function` | Must |
| FR-04 | Product µservice | register Link Types between Object Types; traverse links | entity relationships navigable as a graph | `link` | Must |
| FR-05 | Product µservice | register Action Types; invoke them with Cedar policy check | side-effecting operations gated by policy; audit-sealed | `action` | Must |
| FR-06 | LLM / agent | call Ontology agent gateway (ADR-0107) to query Functions | agentic workflows read structured data without raw DB | `agent-gateway` | Must |
| FR-07 | Product µservice | register properties with pillar assignment (org / person) | pillar enforcement prevents org-owned data leaking to person-pillar | `schema-registry` | Must |
| FR-08 | Product µservice | register custom property types (vector, geo, timeseries, ciphertext, struct) per ADR-0108-0112 | rich entity models without schema workarounds | `schema-registry` | Should |

---

## Non-Functional Requirements

### Performance
- P99 Function read (Ontology query): ≤50 ms (per Bominal ADR-0107 §"Threat model" target).
- P99 Action Type invocation: ≤200 ms (includes Cedar policy check + write + audit seal).
- P99 agent gateway tool-call (LLM → Function → result): ≤200 ms.
- Schema registry read (Object Type lookup): ≤10 ms (cached in Valkey).

### Security
- Postgres RLS `tenant_id` enforced on every Object Type table; `FORCE ROW LEVEL SECURITY`.
- Cedar policy enforced on every Action Type invocation (ADR-0007 + ADR-0140 (retired per ADR-0145)).
- Pillar enforcement (ADR-0132): org-pillar Object Types require `org_id` in JWT;
  person-pillar require `user_id`; cross-pillar reads require explicit Cedar grant.
- Agent gateway (ADR-0107): LLM tool calls validated against Function schema;
  Cedar policy gates which Functions each agent role can call.
- ciphertext property type (ADR-0111): client-side encrypted at property level;
  KMS-wrapped key per tenant per property.

### Audit + Compliance
- Every Object Type write + Action Type invocation Merkle/Ed25519 sealed per
  ADR-0028; seal latency ≤1 s.
- Jurisdiction overlay: per-tenant `jurisdiction_code` applies Cedar policy
  overlay restricting which properties are accessible (GDPR personal data
  properties, PIPA 개인정보, HIPAA PHI).
- DUB (data-use binding) enforcement: property-level access logged and enforced
  per ADR-0119 data tier assignment matrix.

### Availability + SLO
- 99.99% monthly (shared substrate; Ontology outage degrades all products).
- RTO ≤10 s; RPO ≤1 s.

---

## Bounded Contexts

| BC name | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `entity` | `ontology-entity-{kernel,domain,application,infrastructure}` | Object Type CRUD; tenant-isolated rows; audit seal | `ObjectInstance` |
| `link` | `ontology-link-{kernel,domain,application,infrastructure}` | Link Type registry + traversal; graph edges | `LinkInstance` |
| `action` | `ontology-action-{kernel,domain,application,infrastructure}` | Action Type registry; invocation; Cedar gate; audit | `ActionInvocation` |
| `function` | `ontology-function-{kernel,domain,application,infrastructure}` | Function registry; query evaluation; caching | `FunctionResult` |
| `schema-registry` | `ontology-schema-registry-{domain,application,infrastructure,rest}` | Object/Link/Action/Function type registration; pillar + property schema | `ObjectTypeSchema` |
| `agent-gateway` | `ontology-agent-gateway-{domain,application,rest}` | LLM tool-call ingress; Function dispatch; Cedar gate | `AgentToolCall` |
| `audit-chain` | `ontology-audit-chain-{domain,application,infrastructure}` | Merkle tree per (tenant, period); Ed25519 sealing | `AuditSegment` |
| `pillar` | `ontology-pillar-{kernel,domain}` | Org-pillar / person-pillar property-tier + DUB enforcement | `PillarContext` |

### Clean Architecture Layer Map

Dependency direction: strictly inward-only. Per `feedback_clean_architecture_requirements.md`.

```
{schema-registry-rest, agent-gateway-rest}
        ↑ depends on
   {entity-adapter, link-adapter, action-adapter, function-adapter,
    audit-chain-adapter}           (implements kernel ports)
        ↑ depends on
   {entity-application, action-application, function-application,
    agent-gateway-application}
        ↑ depends on
   {entity-domain, link-domain, action-domain, function-domain,
    audit-chain-domain, pillar-domain}
        ↑ depends on
   {entity-kernel, link-kernel, action-kernel, function-kernel,
    pillar-kernel}                 ← ontology-sdk
        ↑
   ontology-app  (composition root)
```

Port traits in kernel — ZERO business logic, ZERO I/O:

```rust
// ontology-entity-kernel/src/ports.rs

#[doc(hidden)]
mod sealed { pub trait Sealed {} }

/// Object Type persistence port — implemented in ontology-entity-adapter
#[async_trait::async_trait]
pub trait ObjectTypeStore: Send + Sync + sealed::Sealed {
    async fn write(&self, tenant: &TenantId, obj: &ObjectInstance)
        -> Result<ObjectId, StoreError>;
    async fn read_by_id(&self, tenant: &TenantId, id: &ObjectId)
        -> Result<ObjectInstance, StoreError>;
    async fn query(&self, tenant: &TenantId, filter: &ObjectFilter)
        -> Result<Vec<ObjectInstance>, StoreError>;
}

// ontology-link-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait LinkTypeStore: Send + Sync + sealed::Sealed {
    async fn write(&self, tenant: &TenantId, link: &LinkInstance)
        -> Result<LinkId, StoreError>;
    async fn traverse(&self, tenant: &TenantId, from: &ObjectId, link_type: &LinkTypeId)
        -> Result<Vec<ObjectId>, StoreError>;
}

// ontology-action-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait ActionTypeStore: Send + Sync + sealed::Sealed {
    async fn invoke(&self, tenant: &TenantId, action: &ActionInvocation)
        -> Result<ActionResult, ActionError>;
}

// ontology-function-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait FunctionEvaluator: Send + Sync + sealed::Sealed {
    async fn evaluate(&self, tenant: &TenantId, func: &FunctionCall)
        -> Result<FunctionResult, EvalError>;
}

// ontology-audit-chain-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait AuditChainStore: Send + Sync + sealed::Sealed {
    async fn append(&self, tenant: &TenantId, entry: &AuditEntry)
        -> Result<MerkleNodeId, StoreError>;
    async fn seal_segment(&self, tenant: &TenantId, period: &AuditPeriod)
        -> Result<MerkleRoot, SealError>;
}
```

Implementations in `ontology-entity-adapter` (Postgres + Citus + RLS),
`ontology-function-adapter` (query engine; ClickHouse replica),
`ontology-audit-chain-adapter` (Merkle tree + Ed25519).
Domain calls through ports; domain never imports adapters.

```
NAME: ontology-entity-kernel
JUSTIFICATION:
- microservice = ontology: Ontology shared substrate µservice; flat catalog; ADR-0056 v4.1; "Ontology" not "Object Graph" per oyatie override
- bc-tokens = entity: ontology has multiple BCs (entity/link/action/function/schema-registry/agent-gateway/audit-chain/pillar); entity BC owns ObjectInstance; ADR-0056 v4.1 BC-optionality
- layer = kernel: shared TenantId-scoped ObjectId type + ObjectInstance port-trait; consumed cross-layer; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none
```

---

## Integration via Workflow + Ontology

Ontology IS the information adapter. Workflow is its peer (the action adapter).
Ontology's agent-gateway is the entry point for agentic Workflow nodes.

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `SchemaRegistrationRequested` | Any µservice at startup | `schema-registry` | Register/update Object Type schema |

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `OntologyEntityMutated` | Object Type write | `workflow` (ontology-event trigger) | `ontology-change-sm` |
| `ActionTypeInvoked` | Action invocation | `audit-chain` | `action-audit-sm` |

---

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| Palantir | Foundry Ontology | Typed Object Types + Link Types + Action Types + Functions; RLS; audit chain; plugin SDK; agent gateway | https://www.palantir.com/docs/foundry/ontology |
| Palantir | AIP (AI Platform) | Agent gateway; LLM tool-call dispatch via Ontology Functions | https://www.palantir.com/platforms/aip |
| Salesforce | Salesforce Object Model | Object Type registry; relationship traversal; SOQL Function queries | https://developer.salesforce.com/docs/atlas.en-us.object_reference |
| Notion | Notion Database + API | Flexible schema; linked databases; typed property system | https://developers.notion.com |
| Airtable | Airtable Universe | Typed fields; linked records; formula evaluation; API surface | https://airtable.com/developers/web/api |

Key parity gaps (Palantir Foundry Ontology is the primary reference):
1. **Typed Function evaluation** (Palantir parity): Functions must support row-level filtering, aggregation, and joins across linked Object Types — not just simple property reads.
2. **Agent gateway schema** (Palantir AIP parity): LLM tool schema auto-generated from Function definitions; no manual OpenAI tool-spec authoring.
3. **Plugin SDK** (ADR-0037 inheritance): custom Object Type renderers + Action Type UI components deployable without oyatie core changes.

---

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Function read (simple filter) | 5 ms | 50 ms | 100 ms | ADR-0107 ≤50ms p99 target |
| Function read (join, 3 types) | 20 ms | 100 ms | 200 ms | |
| Action invocation (Cedar + write + seal) | 30 ms | 150 ms | 300 ms | |
| Agent gateway tool-call round-trip | 20 ms | 200 ms | 500 ms | Excludes LLM inference time |
| Schema registry lookup | 0.5 ms | 10 ms | — | Valkey-cached |
| Audit chain seal | — | 1 s | — | Per (tenant_id, period); ADR-0028 |
| Object write throughput per cell | — | 50k writes/s | — | Postgres + Citus |

Error budget: 0.01% monthly. SLO burn-rate alarm: 2×.

---

## Horizontal Scalability

**State strategy**: `postgres` — Object Type instances in Postgres + Citus;
`tenant_id` as Citus shard key; ClickHouse replica for aggregate/analytics
Function queries; Valkey for schema registry cache + hot Function result cache.

**Active-active compatibility**: `stateless-compatible` for Function read layer;
`single-writer-compatible` for Action invocations with cross-row invariants.

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Object Type instances per tenant | 1,000,000 | 10,000,000,000 | Citus shard fill > 80% |
| Function read QPS | 10,000 | 1,000,000 | CPU > 70% |
| Action invocation QPS | 1,000 | 100,000 | Queue depth > 1k |
| Agent gateway concurrent LLM calls | 100 | 10,000 | Gateway pool > 80% |

Scale-out: Function read layer stateless HPA; Action layer single-writer per
shard; agent-gateway workers HPA on concurrent call count; ClickHouse replica
auto-scales reads. Cross-region replication required for high-consequence
Object Types (Medical, Payments, Audit chain).

---

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Object Type write → Function read round-trip; tenant isolation holds | `cargo nextest run -p ontology-entity-domain --test rls_isolation` |
| AC-02 | Function read p99 ≤50 ms at 10k RPS | k6 smoke; `http_req_duration{p(99)}<50` |
| AC-03 | Action invocation: Cedar policy block returns 403; permitted returns 200 + audit sealed | `cargo nextest run -p ontology-action-domain --test cedar_gate` |
| AC-04 | Agent gateway: LLM tool-call dispatched to Function; result returned in ≤200 ms | integration test `test_agent_gateway_function_call` |
| AC-05 | Pillar enforcement: org-pillar Object Type unreachable via person-pillar context | `cargo nextest run -p ontology-pillar-domain --test pillar_isolation` |
| AC-06 | Audit chain: Merkle root verifiable; tamper = verification failure | `presubmit` (retired CLI `gate validate audit-chain --ms ontology`) |
| AC-07 | LEAN-A2: ontology crates have no µservice-specific imports | `presubmit` (retired CLI `gate validate lean-a2 --ms ontology`) exits 0 |

---

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | ClickHouse replica: M02 or deferred to M03 analytics phase? | council-architecture | ADR-#### |
| 2 | Function DSL: Rust embedded DSL or JSON-serialized IR? | council-architecture | M02/P02 |
| 3 | Plugin SDK distribution format: WASM (Wasmtime per Bominal) or native dylib? | council-architecture | ADR-#### |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0106 | Ontology architecture | inherited — primary authority (oyatie renames Object Graph → Ontology) |
| Bominal ADR-0107 | Ontology agent gateway | inherited |
| Bominal ADR-0108-0112 | Property types (vector/geo/timeseries/ciphertext/struct) | inherited |
| Bominal ADR-0132 | Data ownership pillars | inherited |
| Bominal ADR-0018 | Tenancy RLS posture | inherited |
| Bominal ADR-0028 | Audit chain Merkle/Ed25519 | inherited |
| oyatie override | Object Graph → Ontology | `feedback_glossary_ontology_not_object_graph.md` |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0103 | Workflow hexagonal | peer adapter (action plane) |
