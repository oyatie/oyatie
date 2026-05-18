---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-ontology
microservice: ontology
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M02b-substrate-ready
bominal_source: [ADR-0106, ADR-0107, ADR-0108, ADR-0109, ADR-0110, ADR-0111, ADR-0112, ADR-0132, ADR-0018, ADR-0028]
related_adrs: [ADR-0006, ADR-0028, ADR-0055, ADR-0056, ADR-0059, ADR-0105, ADR-0106, ADR-0107, ADR-0110, ADR-0114, ADR-0122, ADR-0123, ADR-0139, ADR-0131, ADR-0140]
related_specs: [/specs/microservices/ontology.json, /specs/knowledge-graph-schema.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
owner_team: axis-ontology
doc_status: published
---

# PRD-ontology: Ontology Information Substrate

## Purpose

The `ontology` microservice is oyatie's **Palantir-Foundry-class typed-entity substrate** — the canonical information adapter through which every other µservice reads and writes typed Object Types, Link Types, Action Types, and Functions. Per ADR-0059, Ontology is one half of the inter-µservice integration plane (Workflow handles action/orchestration; Ontology handles information). Per `feedback_workflow_objectgraph_adapter_layer.md` this is **THE load-bearing architectural rule** of the platform: µservices never call each other directly — they exchange typed entities through Ontology and typed events through Workflow.

This µservice is **shared substrate**, not a hero product. It is consumed by every other oyatie µservice + agent + product. Its existence is the precondition for cross-tenant isolation at the data layer (Postgres RLS), audit-grade provenance (Merkle/Ed25519 audit chain per Bominal ADR-0028), jurisdiction overlays (per-pack Cedar policy overlays), and pillar enforcement (org/person property-tier separation per Bominal ADR-0132).

This µservice inherits Bominal ADR-0106 (Ontology architecture) 1:1 with one terminology override: Bominal "Object Graph" → oyatie "Ontology" (per ADR-0055 + ADR-0122 + `feedback_glossary_ontology_not_object_graph.md`).

Competitive parity target: **Palantir Foundry Ontology** + AWS Cedar + Open Policy Agent + Neo4j + Apache TinkerPop + Stardog reasoning + AWS Neptune analytics.

## Tenant Value

- **Tenant Outcome 1 — Typed entity graph across products.** Every entity (Patient, Payslip, Workflow run, Message, Order, etc.) is a typed Object Type queryable via a unified API. Tenants compose products without learning each µservice's private DB schema.
- **Tenant Outcome 2 — Agent-ready data layer.** LLMs query Ontology via the agent gateway (ADR-0107 inheritance — tool-call ingress); agentic workflows read structured data without raw DB credentials.
- **Tenant Outcome 3 — Zero-leakage tenancy.** Every Object Type row carries `tenant_id` enforced by Postgres `FORCE ROW LEVEL SECURITY`; cross-tenant reads are impossible at the DB layer.
- **Tenant Outcome 4 — Audit-grade provenance.** Every Object Type mutation + Action Type invocation Merkle/Ed25519 sealed per Bominal ADR-0028; tamper-evident entity history.
- **Tenant Outcome 5 — Jurisdiction overlays.** Per-tenant `jurisdiction_code` applies Cedar policy overlays restricting which properties are accessible (GDPR personal data, PIPA 개인정보, HIPAA PHI, etc.).
- **Internal Outcome 6 — Substrate uniformity.** Every oyatie µservice's writes go through the same typed gate; eliminates per-team divergence in how "entity" + "edge" + "side-effect" are modelled.
- **Internal Outcome 7 — Workflow Studio's first-class data plane.** The visual editor (`workflow-studio`) reads / writes via Ontology Functions; no proprietary information API.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | µservice writer | register an Object Type schema (name, properties, pillar) via `schema-registry` | my entities are queryable by other µservices without imports | `schema-registry` | Must |
| FR-02 | µservice writer | write Object Type instances with `tenant_id`; receive audit seal | data is isolated and provenance-tracked at write moment | `entity-store` | Must |
| FR-03 | µservice reader | query Object Types via Functions with `tenant_id` filter | cross-product reads without importing the writer crate | `function-engine` | Must |
| FR-04 | µservice writer | register Link Types between Object Types; traverse links | entity relationships navigable as a typed graph | `link-store` | Must |
| FR-05 | µservice writer | register Action Types; invoke them with Cedar policy check | side-effecting operations gated by policy; audit-sealed | `action-engine` | Must |
| FR-06 | LLM / agent | call Ontology agent gateway (per ADR-0107 inheritance) to invoke Functions | agentic workflows read structured data without raw DB | `agent-gateway` | Must |
| FR-07 | µservice writer | register properties with pillar assignment (org / person) | pillar enforcement prevents org-owned data leaking to person-pillar | `schema-registry` + `pillar` | Must |
| FR-08 | µservice writer | register custom property types (vector, geo, timeseries, ciphertext, struct) per ADR-0108-0112 | rich entity models without schema workarounds | `schema-registry` | Should |
| FR-09 | tenant operator | view audit-chain entries for a given entity (provenance trail) | every mutation is inspectable + signed | `audit-chain` | Must |
| FR-10 | tenant operator | author OpenAPI-defined Functions in JSON-IR | tenant-defined queries with same gating as built-ins | `function-engine` | Should |
| FR-11 | governance lane | enumerate every Object Type + Action Type registered platform-wide | catalog generation + cross-product schema drift detection | `schema-registry` | Must |
| FR-12 | DSR cascade runner | locate every Object Type containing a subject identifier; tombstone on erasure | GDPR Art. 17 + KR PIPA Art. 36 honoured | `entity-store` + `audit-chain` | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Function read (simple filter) | ≤ 5 ms | ≤ 50 ms | ≤ 100 ms | Bominal ADR-0107 ≤ 50 ms p99 mandate |
| Function read (3-way join across Object Types) | ≤ 20 ms | ≤ 100 ms | ≤ 200 ms | matches Palantir Foundry Function reference values |
| Action invocation (Cedar gate + write + audit seal) | ≤ 30 ms | ≤ 150 ms | ≤ 300 ms | end-to-end including signature emission |
| Agent gateway tool-call round-trip (LLM → Function → result) | ≤ 20 ms | ≤ 200 ms | ≤ 500 ms | excludes LLM inference time |
| Schema registry lookup (Object Type schema) | ≤ 0.5 ms | ≤ 10 ms | — | Valkey hot cache |
| Audit chain seal (per tenant per period) | — | ≤ 1 s | — | Bominal ADR-0028 budget |
| Object Type write throughput | — | 50k writes/s per cell | — | Postgres + Citus sharded |
| ClickHouse analytics Function | ≤ 100 ms | ≤ 500 ms | ≤ 1 s | aggregations + cross-Object joins on history-mirror |

### Security

- Postgres `FORCE ROW LEVEL SECURITY` on every Object Type table; `tenant_id` policy bound to `current_setting('app.tenant_id')`.
- Cedar v4 policy evaluated on every Action Type invocation; default-deny on every cross-tenant link request.
- Pillar enforcement (Bominal ADR-0132): org-pillar Object Types require `org_id` in JWT; person-pillar requires `user_id`; cross-pillar reads require explicit Cedar grant.
- Agent gateway (Bominal ADR-0107): LLM tool calls validated against Function schema; Cedar `autonomy_tier_ceiling` enforced (`/specs/agent-durable-goal.json#source_driven_contract`).
- `ciphertext` property type (Bominal ADR-0111): per-property client-side envelope encryption; KMS-wrapped DEK per tenant per property.
- All cross-cluster traffic mTLS (Istio); all OpenAPI surfaces require OIDC bearer + X-Tenant-Id header match.
- Audit-chain Ed25519 signing keys live in OpenBao (90d rotation per ADR-0028).
- Cedar fragment coverage CI lane (`oya-foundry-fitness-cedar-coverage`): every Action Type must have a Cedar permit + a default-deny.

### Audit + Compliance

- Every Object Type write, Link Type write, Action Type invocation, and Function evaluation that touches `data_class != PUBLIC` emits an `AuditEvent` Merkle-chained per (tenant, period).
- Seal cadence: 60 s rolling segment OR 10⁴ events, whichever first.
- `oya:ontology_audit_chain_completeness:rate == 1.0` SLO (target 100% emission per Action Type).
- Audit log retention: ≥ 1 y (pack default); ≥ 3 y (KR-FSS); ≥ 6 y (pack-us-healthcare HIPAA §164.316(b)(2)).
- Jurisdiction overlay: per-tenant `jurisdiction_code` applies pack-overlay Cedar policy (e.g., `policy/overlay-pack-kr-pipa.cedar`).
- DUB (Data Use Boundary) enforcement per Bominal ADR-0008 + ADR-0119: property-level access + propagation logged.

### Availability + SLO

- Availability target: **99.99 %** monthly (shared substrate; Ontology outage degrades every product).
- RTO ≤ 10 s (Function read failover via stateless replicas + ClickHouse read-mirror).
- RPO ≤ 1 s (Postgres streaming replication + outbox-to-Kafka per Bominal ADR-0050).
- Error budget: 0.01 % (≈4.4 min/month).
- Burn-rate alarm at 2× over 1h triggers Sev-2 page.

### Data residency

- Object Type instances inherit the tenant's `jurisdiction_code` per ADR-0117; per-pack Postgres + Citus clusters; cross-pack replication forbidden by default (see `policy/data-residency.md`).
- ClickHouse history-mirror clusters are pack-pinned identically.
- Ed25519 audit-chain seals are per-pack; cross-pack seal chains are independent.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), layers used by this µservice are: `kernel`, `domain`, `usecase`, `api` (protocol-neutral typed contracts), `adapter`, `rest`, `worker`, `sdk` (client library), `app` (composition root). Backend-qualified adapters use the `*-adapter-<backend>` pattern (ADR-0105 Amendment 3).

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `object-type-registry` | `oya-ontology-object-type-registry-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Object Type schema authoring, validation, hot-reload, propagation, deprecation handshake | `ObjectTypeSchema`, `PropertyDescriptor`, `PropertyTier`, `PillarKind`, `JurisdictionOverlay`, `SchemaRevision` |
| `link-type-registry` | `oya-ontology-link-type-registry-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Link Type schema (cardinality, traversal-direction, tenant-scope, jurisdiction overlay) | `LinkTypeSchema`, `LinkCardinality`, `TraversalDirection` |
| `action-type-registry` | `oya-ontology-action-type-registry-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Action Type schema (effects, idempotency, Cedar gate, audit emission) | `ActionTypeSchema`, `EffectSpec`, `IdempotencyKey`, `CedarGate` |
| `function-type-registry` | `oya-ontology-function-type-registry-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Function (read-projection) schema + DSL IR + result-shape registry + cache TTL | `FunctionTypeSchema`, `FunctionDSL`, `ResultShape`, `CacheTtl` |
| `entity-store` | `oya-ontology-entity-store-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-clickhouse,worker,sdk,app}` | Object Type instance persistence + RLS + audit emission + Citus sharding + ClickHouse history mirror | `ObjectInstance`, `ObjectId`, `WriteReceipt` |
| `link-store` | `oya-ontology-link-store-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk,app}` | Link Type instance persistence + traversal + cross-Object-Type joins | `LinkInstance`, `LinkId`, `TraversalQuery` |
| `function-engine` | `oya-ontology-function-engine-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Function evaluation (Postgres for OLTP; ClickHouse for OLAP); cache; backpressure | `FunctionCall`, `FunctionResult`, `JoinPlan` |
| `action-engine` | `oya-ontology-action-engine-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Action invocation; Cedar gate; idempotency; outbox-to-Kafka per ADR-0050; audit-chain emission | `ActionInvocation`, `ActionReceipt`, `ActionResult` |
| `cedar-fragment-coverage` | `oya-ontology-cedar-fragment-coverage-{kernel,domain,usecase,api,adapter}` | Cedar policy fragment registry; per-Action permit-set; default-deny CI lane authority | `CedarFragment`, `PolicyDecision`, `AutonomyTierCeiling` |
| `query-engine` | `oya-ontology-query-engine-{kernel,domain,usecase,api,adapter,adapter-clickhouse,worker,sdk,app}` | Cross-Object analytics + read-projections + 3-layer Knowledge Graph (semantic/kinetic/dynamic) per `/specs/knowledge-graph-schema.json` | `KgQuery`, `KgEdge`, `KgNode`, `FreshnessBudget` |
| `agent-gateway` | `oya-ontology-agent-gateway-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | LLM tool-call ingress; OpenAI-tool-spec auto-generation; Cedar autonomy-tier ceiling; rate-limit | `AgentToolCall`, `AgentToolSpec`, `LlmDispatch` |
| `audit-chain` | `oya-ontology-audit-chain-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Per-tenant Merkle tree + Ed25519 seal; per-period segment; tamper detection | `AuditEvent`, `MerkleRoot`, `SealSignature` |
| `pillar` | `oya-ontology-pillar-{kernel,domain,usecase}` | Org-pillar + person-pillar property-tier enforcement; cross-pillar Cedar gate | `PillarContext`, `PillarKind`, `CrossPillarGrant` |

Naming justification — `object-type-registry`:

```
NAME: oya-ontology-object-type-registry-<layer>
JUSTIFICATION:
- microservice = ontology: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder; supersedes the legacy "object-graph" token per ADR-0122.
- bc-tokens = object-type-registry: primary BC for Object Type schema authoring +
  validation + propagation. ADR-0056 v4.1 BC-optionality rule honoured because sibling
  BCs (link-type-registry, action-type-registry, etc.) require explicit BC tokens.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (ObjectTypeSchema, PropertyDescriptor,
    PropertyTier, PillarKind). Zero I/O. Carries data_class annotations per ADR-0028 +
    oya-check-data-class lane.
  - domain: pure schema-evolution logic, pillar/tier inference, jurisdiction overlay merge.
  - usecase (per ADR-0106; replaces legacy 'application'): orchestrators reading
    schema-registry adapter + writing to registry store + emitting SchemaRegistered events.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral Postgres + Valkey registry impl.
  - rest: HTTP handler for tenant + agent + governance reads.
  - worker: schema-propagation worker (hot-reload across Function engine + Action engine).
  - sdk: Rust client + future TS/Python via bindings.
  - app: composition root binary.
- exemptions claimed: none.
```

Naming justification — `entity-store`:

```
NAME: oya-ontology-entity-store-<layer>
JUSTIFICATION:
- microservice = ontology.
- bc-tokens = entity-store: sibling BC for Object Type instance persistence.
- layer = <layer>: 10 crates including two backend-qualified adapters:
  - adapter-postgres: Postgres + Citus implementation (RLS-enforced; tenant-id-sharded).
  - adapter-clickhouse: ClickHouse history-mirror implementation (read-optimized analytics).
  - These follow ADR-0105 Amendment 3 *-adapter-<backend> pattern.
- exemptions: none.
```

Layer mapping per BC (`usecase` per ADR-0106; backend-qualified adapters per ADR-0105 Amendment 3):

| BC | kernel | domain | usecase | api | adapter | adapter-pg | adapter-ch | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `object-type-registry` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | ✓ | ✓ |
| `link-type-registry` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | ✓ |
| `action-type-registry` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | ✓ |
| `function-type-registry` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | ✓ |
| `entity-store` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ |
| `link-store` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ |
| `function-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | ✓ |
| `action-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | ✓ |
| `cedar-fragment-coverage` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — |
| `query-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | ✓ | ✓ | ✓ |
| `agent-gateway` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | ✓ |
| `audit-chain` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | ✓ |
| `pillar` | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — |

Total crates introduced by this µservice: **~92** (counting backend-qualified adapters). The IP series scaffolds these incrementally across IP-001 .. IP-015.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `ObjectTypeStore` | `oya-ontology-entity-store-kernel` | `-adapter-postgres` (RLS + Citus shard `tenant_id`) | varies (per Object Type tier matrix) |
| `LinkTypeStore` | `oya-ontology-link-store-kernel` | `-adapter-postgres` (RLS + cross-Object FK with tenant guard) | varies |
| `ActionTypeStore` | `oya-ontology-action-engine-kernel` | `-adapter` (action registry + idempotency journal) | `AUDIT` |
| `FunctionEvaluator` | `oya-ontology-function-engine-kernel` | `-adapter` (Postgres OLTP) + `-adapter-clickhouse` (OLAP mirror) | varies |
| `SchemaRegistry` | `oya-ontology-object-type-registry-kernel` | `-adapter` (Postgres + Valkey hot cache) | `INTERNAL_ONLY` |
| `CedarPolicyEvaluator` | `oya-ontology-cedar-fragment-coverage-kernel` | `-adapter` (Cedar v4 SDK bindings) | `INTERNAL_ONLY` |
| `AuditChainEmitter` | `oya-ontology-audit-chain-kernel` | `-adapter` (Merkle tree builder + Ed25519 signer; OpenBao-backed key) | `AUDIT` |
| `KgQueryEngine` | `oya-ontology-query-engine-kernel` | `-adapter-clickhouse` (3-layer Knowledge Graph reads) | varies |
| `AgentToolDispatcher` | `oya-ontology-agent-gateway-kernel` | `-adapter` (Cedar autonomy_tier check + Function dispatch + rate limit) | varies |
| `PillarResolver` | `oya-ontology-pillar-kernel` | `-domain` (pure logic; no adapter) | — |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `ontology` MUST NOT import any other product µservice crate at any layer. Workflow may consume Ontology Functions through its `oya-workflow-adapter` (per ADR-0059); Ontology does NOT import Workflow. LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice ontology` — dependency-direction
- `oya gate validate lean-a2 --microservice ontology` — cross-product-refusal
- `oya gate validate port-location --microservice ontology` — ports in kernel
- `oya gate validate layer-correctness --microservice ontology` — layer enum match
- `oya gate validate per-microservice-layout --microservice ontology` — ADR-0131
- `oya gate validate statelessness --microservice ontology`
- `oya gate validate shardability --microservice ontology`
- `oya gate validate ontology-tier-enforcement --microservice ontology` — property-tier
- `oya gate validate cedar-coverage --microservice ontology` — every Action has permit
- `oya gate validate audit-chain-emission --microservice ontology` — completeness 100%

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `ObjectTypeRegistered` | `object-type-registry` write | `workflow`, `observability`, every consumer µservice | `schema-propagation-sm` |
| `ObjectInstanceMutated` | `entity-store` write | `workflow`, downstream Function subscribers | `ontology-change-sm` |
| `ActionTypeInvoked` | `action-engine` execution | `audit-chain`, `workflow` (subscribers) | `action-audit-sm` |
| `LinkTypeRegistered` | `link-type-registry` write | consumers needing edge traversal | `schema-propagation-sm` |
| `AuditChainSealed` | `audit-chain` periodic seal | `audit-chain` µservice (cross-µservice seal), `observability` | `audit-seal-sm` |
| `CrossPillarGrantRequested` | `pillar` request | `workflow`, ops-security approval gate | `cross-pillar-approval-sm` |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `MicroserviceRegistered` | `tenancy` | `object-type-registry` | initialise per-µservice schema scope |
| `TenantOnboarded` | `tenancy` | `entity-store` | provision per-tenant Postgres role + Cedar entitlements |
| `JurisdictionOverlayUpdated` | `governance` | `cedar-fragment-coverage` | hot-reload Cedar overlay fragments |
| `DsrErasureRequested` | `governance` | `entity-store` + `audit-chain` | DSR cascade across Object Types |
| `OpenSLOManifestUpdated` | `observability` | (no-op; observability authors its own SLOs) | — |

### Ontology writes (this µservice is the Ontology authority)

This µservice owns the Ontology — it is the writer of all canonical Object Types. Other µservices write Object Types through the SDK; this µservice persists them.

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Microservice` (catalog) | `object-type-registry` | enumerate µservices needing schema scope |
| `Tenant` | `entity-store` | jurisdiction_code + pillar context resolution |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Palantir | Foundry Ontology | Typed Object Types + Link Types + Action Types + Functions; RLS; audit chain; plugin SDK; agent gateway | `palantir.com/docs/foundry/ontology` |
| Palantir | AIP (AI Platform) | Agent gateway; LLM tool-call dispatch via Ontology Functions | `palantir.com/platforms/aip` |
| AWS Cedar | Cedar v4 policy engine | Permit/forbid policy fragments; default-deny; entity types; context | `cedarpolicy.com` |
| Open Policy Agent | Rego policy + OPA bundle | Decision logging; policy hot-reload | `openpolicyagent.org` |
| Neo4j | Property graph + Cypher | Graph traversal; index/constraint planner | `neo4j.com/docs` |
| Apache TinkerPop | Gremlin graph query DSL | Property-graph TinkerPop bytecode | `tinkerpop.apache.org` |
| AWS Neptune | Multi-language graph (Gremlin / openCypher / SPARQL) | Graph analytics algorithms; vector retrieval | `aws.amazon.com/neptune` |
| Stardog | RDF + virtual graphs + reasoning | Virtual graph adapters; SPARQL + OWL/RDFS reasoning | `docs.stardog.com` |
| Salesforce | Object model + SOQL | Object Type registry; relationship traversal | `developer.salesforce.com/docs/atlas.en-us.object_reference` |
| Open Group | The Open Group Architecture Framework (TOGAF) | Information architecture metamodel | `pubs.opengroup.org/togaf-standard` |

Key parity gaps to close (ordered by priority):

1. **Function p99 ≤ 50 ms** (Palantir Foundry parity) — production-grade query engine over Postgres + ClickHouse mirror.
2. **Agent gateway with auto-generated tool-spec** (Palantir AIP parity) — OpenAI tool-spec generated from Function definition; no manual authoring.
3. **Action transaction receipts** (Palantir parity) — every Action invocation emits a receipt with `action_id, object_ids, link_ids, rule_id, idempotency_key, actor_principal, permission_decision_ref, audit_chain_ref` before canonical state changes.
4. **Cedar coverage 100 %** — every Action Type has a Cedar fragment + a default-deny; LEAN lane enforces.
5. **Virtual-graph adapters** (Stardog parity) — read-only external-source mapping receipts; canonical mutations stay in Ontology.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Function read (simple filter) | 5 ms | 50 ms | 100 ms | per `/specs/microservices/ontology.json#metrics` |
| Function read (3-way join) | 20 ms | 100 ms | 200 ms | OLTP Postgres |
| Action invocation | 30 ms | 150 ms | 300 ms | end-to-end (Cedar + write + seal) |
| Agent gateway round-trip | 20 ms | 200 ms | 500 ms | excludes LLM time |
| Schema registry lookup | 0.5 ms | 10 ms | — | Valkey hot |
| Audit chain seal | — | 1 s | — | per (tenant, period) |
| Object write throughput | — | 50 k writes/s per cell | — | Postgres + Citus |
| 3-layer KG join (semantic+kinetic+dynamic) | 100 ms | 500 ms | 1 s | OLAP ClickHouse |
| Dynamic-layer freshness lag | — | ≤ 2 s | — | OTel + Kafka |

Error budget:
- Function read SLO: 99.99 % monthly (0.01 %; ≈4.4 min/month error budget).
- Action invocation SLO: 99.95 % monthly.
- Audit-chain emission completeness SLO: 100 % (zero tolerance).
- Burn-rate alarm on Function read: 14.4× over 1 h triggers page.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Object Type instances in Postgres + Citus (shard key = `tenant_id`); ClickHouse history-mirror for OLAP Function reads; Valkey for schema-registry cache + hot Function result cache; OpenBao for audit-signing keys; Kafka outbox per ADR-0050 for event emission.

**Active-active compatibility**: `stateless-compatible` for Function read layer, agent gateway, REST surfaces; `single-writer-compatible` for Action invocations with cross-row invariants.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Object Type instances per tenant | 1 M | 10 B | Citus shard fill > 80 % |
| Function read QPS | 10 k | 1 M | Function-engine CPU > 70 % |
| Action invocation QPS | 1 k | 100 k | action-engine queue depth > 1 k |
| Agent gateway concurrent LLM calls | 100 | 10 k | gateway pool > 80 % |
| Audit-chain emission rate | 100 k events/s | 10 M events/s | audit-chain ingester p99 > 1 s |
| 3-layer KG join QPS | 100 | 10 k | ClickHouse mirror CPU > 70 % |

Scale-out policy:
- Kubernetes HPA: Function/agent/REST layers scale on CPU `> 70 %`; min 2 replicas, max 50.
- Citus: auto-rebalance when shard fill > 80 % (manual sign-off for cross-region; auto for in-pack).
- ClickHouse: read-replicas auto-scale on CPU; writes single-master per shard.
- Pre-warmed pool: 2 standby pods per critical Layer-B component.

Cross-region story:
- M02b launch: pack-kr (OCI ap-seoul-1) single region.
- Post-M02b: per-pack Postgres + ClickHouse clusters; cross-pack replication forbidden; ADR successor-IP for cross-region DR pairs.

Sharding:
- Citus distributed Postgres shards by `tenant_id`.
- ClickHouse partitions by `(tenant_id, toYYYYMM(ts))`.
- Audit chain partitions by `(tenant_id, period)` segment.
- `oya-check-shardability-cli` CI lane verifies partition key presence in every kernel.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Object Type write → Function read round-trip; tenant isolation holds (Tenant-A cannot read Tenant-B) | `cargo nextest run -p oya-ontology-entity-store-domain --test rls_isolation` |
| AC-02 | Function read p99 ≤ 50 ms at 10 k QPS | k6 smoke; `http_req_duration{p(99)}<50ms` |
| AC-03 | Action invocation: Cedar policy deny → 403; permit → 200 + audit sealed | `cargo nextest run -p oya-ontology-action-engine-domain --test cedar_gate` |
| AC-04 | Agent gateway: LLM tool-call dispatched to Function; result returned in ≤ 200 ms | `tests/integration/agent_gateway_function_call.rs` |
| AC-05 | Pillar enforcement: org-pillar Object Type unreachable from person-pillar context | `cargo nextest run -p oya-ontology-pillar-domain --test pillar_isolation` |
| AC-06 | Audit chain: Merkle root verifiable; tamper = verification failure | `oya gate validate audit-chain --microservice ontology` |
| AC-07 | LEAN-A2: ontology crates have no µservice-specific imports | `oya gate validate lean-a2 --microservice ontology` exit 0 |
| AC-08 | ADR-0131 per-microservice layout green | `oya gate validate per-microservice-layout --microservice ontology` exit 0 |
| AC-09 | ADR-0123 hyperscaler-maturity HG-ONT registers green | `oya gate validate authority-cohesion` exit 0 |
| AC-10 | Cedar coverage 100 %: every Action Type has a permit + default-deny | `oya gate validate cedar-coverage --microservice ontology` |
| AC-11 | Audit-chain completeness 100 %: zero unsealed Action invocations across 24 h | `oya:ontology_audit_chain_completeness:rate == 1.0` over 24h |
| AC-12 | Dynamic-layer freshness ≤ 2 s p99 | `tests/integration/dynamic_freshness.rs` |
| AC-13 | DSR cascade: erasure request tombstones every matching Object Type within 30 d | `tests/e2e/dsr_cascade.rs` |
| AC-14 | Cross-tenant link refused unless Cedar `CrossTenantLinkGrant` present | `cargo nextest run -p oya-ontology-link-store-domain --test cross_tenant_refused` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | ClickHouse history-mirror: ship in M02b or defer to M03 analytics phase? | council-architecture | resolved in IP-009 (ship in M02b) |
| 2 | Function DSL: embedded Rust DSL vs JSON-serialised IR? | council-architecture | M02b/P02 successor-IP |
| 3 | Plugin SDK distribution format: WASM (Wasmtime per Bominal) or native dylib? | council-architecture | subsequent-to-M02b-completion successor-IP ADR |
| 4 | Sequential agent autonomy ceiling: per-tool-call vs per-session? | council-privacy + axis-ontology | M03 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0006 | Ontology typed-entity layer | foundational |
| ADR-0028 (Bominal) | Audit chain Merkle/Ed25519 | inherited; this µservice is a primary emitter |
| ADR-0055 | Ontology renamed from Object Graph | naming authority |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0059 | Workflow + Ontology = ecosystem adapter layer | THE load-bearing rule |
| ADR-0106 (Bominal) | Ontology architecture | inherited 1:1 (with name override) |
| ADR-0107 (Bominal) | Ontology agent gateway | inherited |
| ADR-0108..0112 (Bominal) | Property types (vector/geo/timeseries/ciphertext/struct) | inherited |
| ADR-0122 | Ontology crate rename | locks naming |
| ADR-0123 | Hyperscaler maturity claim gate | HG-ONT registers here |
| ADR-0139 | Agentic SLO-gated promotion | every Function read SLO author here |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0132 (Bominal) | Data-ownership pillars | inherited |
| ADR-0140 | Cedar policy enforcement | enforces every Action Type |
