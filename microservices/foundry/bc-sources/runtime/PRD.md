---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-foundry-runtime
microservice: foundry-runtime
status: Accepted
sales_segment: shared-substrate
tier: internal-and-tenant-product
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs: [ADR-0022, ADR-0024, ADR-0025, ADR-0056, ADR-0105, ADR-0106, ADR-0110, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/agent-operating-contract.json, /specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
date: 2026-05-17
owner_team: axis-foundry-runtime
doc_status: published
---

# PRD-foundry-runtime: Agent Runtime + Capability Execution Substrate

## Purpose

The `foundry-runtime` µservice is oyatie's substrate for **hosted agent invocation**: it accepts a capability invocation request, resolves the capability descriptor from the registry mirror, materialises a runtime pod (or attaches to a warm pool worker), maintains the agent's session state across multi-turn interactions, dispatches tool calls through the foundry-providers and foundry-guardrails siblings, and emits structured invocation telemetry to foundry-evidence and observability.

Per ADR-0131 Foundry split, `foundry-runtime` is the **execution plane** of the Foundry. It is consumed by every oyatie µservice that needs to invoke an agent (workflow-engine for agent steps; ops-portal for human-in-the-loop helpers; product workflows for capability dispatch) and exposed to tenants as the hosted-agent compute layer underneath their Workflow Studio surfaces.

This µservice is **shared substrate** (the runtime engine; same shape every tenant) **and an end-user product surface** (tenants address it directly via SDK to invoke their own custom capabilities). Its existence is the precondition for every Foundry-class product per `feedback_quality_performance_scalability_bar.md` and the Workflow Studio scope in `feedback_workflow_studio_scope.md`.

This µservice has no direct Bominal equivalent and originates in oyatie per ADR-0025 (Foundry runtime consolidation).

## Tenant Value

- **Tenant Outcome 1 — Hosted capability execution without per-tenant infra.** Tenants register a capability descriptor; the runtime materialises pods, attaches sessions, and dispatches LLM + tool calls. No tenant-side Kubernetes operation required.
- **Tenant Outcome 2 — Session-coherent multi-turn agents.** Sessions persist across turn boundaries with strict per-tenant Redis-backed state isolation; tenant agents resume on any pod in the pool without state loss.
- **Tenant Outcome 3 — Autonomy-tier-gated execution.** Tenant capabilities declare an ADR-0022 autonomy tier (T0–T4); the runtime refuses to execute a capability outside the tenant's authorised tier ceiling without explicit per-invocation override + audit emission.
- **Internal Outcome 4 — Substrate uniformity.** Every oyatie product invoking agents goes through the same runtime; eliminates per-product divergence in how invocations are scheduled, sessions managed, telemetry shaped, or autonomy enforced.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant operator | to register a capability descriptor and target autonomy tier (T0–T4) | the runtime can execute it within tenant-authorised bounds | capability-registry-cache | Must |
| FR-02 | workflow-engine | to dispatch a capability invocation by `(tenant, capability_id, input_payload)` | my workflow step's agent invocation resolves to a hosted runtime call | capability-executor | Must |
| FR-03 | invocation orchestrator | to read the capability descriptor from the local registry mirror in ≤10ms p99 | per-invocation dispatch latency stays within the 50ms p99 envelope | capability-registry-cache | Must |
| FR-04 | session-state | to load and persist per-session conversation history + tool-call scratchpad in Valkey with p99 ≤10ms | multi-turn agent interactions resume on any runtime pod within the pool | session-state | Must |
| FR-05 | capability-executor | to invoke `foundry-providers` (LLM adapter) and `foundry-guardrails` (safety check) per dispatch | the runtime never reaches an LLM provider directly or bypasses guardrails | capability-executor | Must |
| FR-06 | runtime-pool | to maintain a warm pool of N runtime pods sized per `capacity-model.md` | cold-start cost is amortised; cold-start p99 ≤500ms per ADR-0020 | runtime-pool | Must |
| FR-07 | invocation orchestrator | to emit `InvocationStarted`, `InvocationStepEmitted`, `InvocationCompleted`, `InvocationFailed`, `InvocationCancelled` events | foundry-evidence and observability can stitch the timeline | invocation-orchestrator | Must |
| FR-08 | autonomy gate | to refuse capability execution above the principal's tenant tier ceiling per ADR-0022 | tier escalation can never happen silently | invocation-orchestrator | Must |
| FR-09 | tenant operator | to read invocation history (per session; per tenant; time-bounded) via REST / gRPC | I can debug session behaviour and audit my own agents | capability-executor, session-state | Must |
| FR-10 | runtime pod | to be drained on autonomy-violation, provider compromise, or registry-resync trigger without losing in-flight invocations | rolling drains stay safe; partial completions are durably parked | runtime-pool | Must |
| FR-11 | session-state | to honour DSR cascade requests within 30 days of receipt | tenant DSR obligations propagate into transient session memory | session-state | Must |
| FR-12 | capability-registry-cache | to hot-reload on `CapabilityRegistryUpdated` event from foundry-supervisor | tenants edit a capability descriptor and runtime picks up the new version within ≤30s without restart | capability-registry-cache | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Capability dispatch latency (request → first provider call) | ≤15ms | ≤50ms | ≤120ms | the headline scalability requirement; end-to-end runtime overhead excluding LLM round-trip |
| Session-state hot read (Valkey hit) | ≤3ms | ≤10ms | ≤25ms | per-session conversation history + scratchpad |
| Session-state cold load (Valkey miss → Postgres restore) | ≤30ms | ≤100ms | ≤250ms | bounded to RPO of session-state subsystem |
| Registry-cache lookup | ≤1ms | ≤10ms | ≤30ms | in-memory dictionary backed by Postgres mirror |
| Pool warm-pod cold-start budget | — | ≤500ms | — | per ADR-0020; pre-warmed pool size sized in capacity-model.md |
| Invocation completion event emission lag | ≤20ms | ≤80ms | ≤200ms | event → AsyncAPI bus → foundry-evidence |
| Drain (graceful pod retirement, in-flight invocations parked) | ≤30s | ≤60s | — | bounded by longest in-flight invocation budget |

### Security

- All runtime pods authenticate to foundry-providers, foundry-guardrails, and foundry-evidence over mTLS with per-pod SPIFFE identity (`spiffe://oyatie/foundry-runtime/<pod>`).
- Per-capability execution is scoped by Cedar policy fragments (this µservice ships `tenant-scope`, `ci-scope`, `auditor-scope`, `public-read` per ADR-0140 (retired per ADR-0145)); cross-tenant invocation is default-deny.
- Provider credentials are never resident in runtime pods; the runtime asks `foundry-providers` to invoke the LLM with provider-side credentials that runtime never sees (mitigation for "provider-credential leakage from runtime" threat).
- Session-state is encrypted at rest in Valkey (Valkey TLS + per-pack KMS-bound encryption key) and in Postgres (TDE).
- Capability dispatch is rate-limited per (tenant, capability_id) per autonomy tier; excess returns 429 + per-tenant quota emission.
- Secrets follow the OpenBao SecretReference pattern; raw secrets never enter the repo, chat, checkpoints, or runtime pod environment variables.

### Audit + Compliance

- Every `InvocationStarted`, `InvocationCompleted`, `InvocationFailed`, `InvocationCancelled`, and autonomy-tier-violation event emits an audit-chain record (Ed25519 + Merkle per Bominal ADR-0028).
- Session-state mutations carry an audit trail (`session_mutation_log`) preserved in Postgres for the longer of: 1 year baseline, 6 years for pack-us-healthcare PHI-touching sessions per HIPAA §164.316(b)(2).
- Audit-chain seal latency ≤1s per `(tenant, period)`.

### Availability + SLO

- Availability target: 99.95% monthly for the capability-dispatch path (the executor must remain available even when individual provider backends are degraded; circuit-breakers fail-fast).
- Session-state hot read availability target: 99.9% monthly (Valkey HA cluster with cross-AZ replication).
- RTO: ≤5 min for runtime-pool component (HA failover). RPO: ≤30s for session-state (one Valkey AOF flush cycle).

### Data residency

- Sessions inherit the tenant's `jurisdiction_code` per ADR-0117. Per-pack Valkey and Postgres instances enforce data residency; cross-pack session migration is forbidden by default.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), layers used by this µservice are: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-redis`, `adapter-postgres`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `capability-executor` | `oya-foundry-runtime-capability-executor-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Resolve capability descriptor, dispatch through providers + guardrails, return invocation result | `Capability`, `Invocation`, `InvocationStep`, `InvocationResult`, `AutonomyTier` |
| `session-state` | `oya-foundry-runtime-session-state-{kernel,domain,usecase,api,adapter,adapter-redis,adapter-postgres,sdk,app}` | Per-session conversation history + tool-call scratchpad; durable across pod restarts | `Session`, `SessionTurn`, `ScratchpadEntry`, `SessionLease` |
| `invocation-orchestrator` | `oya-foundry-runtime-invocation-orchestrator-{kernel,domain,usecase,api,adapter,worker,app}` | Lifecycle state machine; event emission; cancellation; timeout handling | `InvocationLifecycle`, `OrchestratorVerdict`, `CancellationToken` |
| `runtime-pool` | `oya-foundry-runtime-runtime-pool-{kernel,usecase,api,adapter,worker,app}` | Warm-pool sizing; pod lifecycle; drain; HPA bridge | `RuntimePod`, `PoolMembership`, `DrainPlan` |
| `capability-registry-cache` | `oya-foundry-runtime-capability-registry-cache-{kernel,usecase,api,adapter,adapter-postgres,worker,app}` | In-memory mirror of capability descriptors with hot-reload | `CapabilityDescriptor`, `RegistryVersion`, `CacheEntry` |

Naming justification — `capability-executor`:

```
NAME: oya-foundry-runtime-capability-executor-<layer>
JUSTIFICATION:
- microservice = foundry-runtime: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder; no shared|vertical bisection.
- bc-tokens = capability-executor: primary BC; dispatches invocations through providers + guardrails;
  declared explicitly because four sibling BCs exist within this µservice.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port traits (CapabilityResolver, ProviderInvoker, GuardrailChecker, EvidenceEmitter,
    AutonomyGate) + entity types (Capability, Invocation, InvocationStep, InvocationResult,
    AutonomyTier). Zero I/O. Every field carries #[data_class(...)] per Bominal ADR-0028.
  - domain: pure capability dispatch math; step-state transitions; autonomy-tier monotonic checks.
  - usecase (per ADR-0106; replaces legacy 'application'): orchestrators reading capability descriptor,
    invoking providers + guardrails via ports, emitting events.
  - api: protocol-neutral typed I/O contracts (request/response types + error variants); consumed
    by rest/sdk; depends on kernel only.
  - adapter: protocol-neutral implementations of kernel ports (e.g., in-process registry-cache reader).
  - rest: HTTP handler/route layer; consumes -api types.
  - sdk: tenant-facing client library (Rust; future TS/Python via bindings) for programmatic
    capability invocation. Closes the industry-standard agent-runtime SDK gap.
  - app: composition root binary; wires rest + adapter clients.
- exemptions claimed: none.
```

Naming justification — `session-state`:

```
NAME: oya-foundry-runtime-session-state-<layer>
JUSTIFICATION:
- microservice = foundry-runtime; bc-tokens = session-state.
- layer = <layer>:
  - kernel: port traits (SessionStore, SessionLeaseManager, SessionMutationLog) + entities
    (Session, SessionTurn, ScratchpadEntry, SessionLease).
  - domain: scratchpad merge logic; conflict-resolution rules for concurrent updates.
  - usecase: load/persist/extend-lease orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral default.
  - adapter-redis: backend-qualified adapter per ADR-0105 Amendment 3 (*-adapter-<backend>);
    implements SessionStore + SessionLeaseManager against Valkey 8.1 (Redis wire-compat) with TLS + AUTH.
  - adapter-postgres: backend-qualified adapter for cold-tier session restore and the
    SessionMutationLog audit table (Postgres 16 LTS).
  - sdk: tenant-facing client library for direct session read (debugging + tenant tooling).
  - app: composition root.
- exemptions claimed: none. -adapter-redis and -adapter-postgres use the canonical
  *-adapter-<backend> pattern.
```

Naming justification — `invocation-orchestrator`:

```
NAME: oya-foundry-runtime-invocation-orchestrator-<layer>
JUSTIFICATION:
- microservice = foundry-runtime; bc-tokens = invocation-orchestrator.
- layer = <layer>:
  - kernel: port traits (LifecycleStore, EventEmitter, TimeoutClock, CancellationSignal) + entities
    (InvocationLifecycle, OrchestratorVerdict, CancellationToken).
  - domain: lifecycle-state-machine validity; deadline arithmetic.
  - usecase: orchestrator that drives a single invocation start→complete.
  - api: typed contracts.
  - adapter: in-memory lifecycle store + AsyncAPI bus emitter (default backend).
  - worker: long-lived background task for timeout monitoring + idempotent re-emission.
  - app: composition root.
- exemptions: none.
```

Naming justification — `runtime-pool`:

```
NAME: oya-foundry-runtime-runtime-pool-<layer>
JUSTIFICATION:
- microservice = foundry-runtime; bc-tokens = runtime-pool.
- layer = <layer>:
  - kernel: port traits (PodFactory, PoolHealthProbe, DrainController) + entities
    (RuntimePod, PoolMembership, DrainPlan).
  - usecase: pool-resize + drain orchestrators (no domain layer; pool math is trivial).
  - api: typed contracts.
  - adapter: Kubernetes client-go wrapper for pod CRUD; HPA bridge.
  - worker: continuous pool health-probe + autoscale-trigger emitter.
  - app: composition root.
- exemptions: domain layer skipped — runtime-pool is mechanism, no business arithmetic warrants
  a dedicated math crate; ADR-0105 §"Amendment 4" permits domain elision when no pure
  arithmetic logic exists.
```

Naming justification — `capability-registry-cache`:

```
NAME: oya-foundry-runtime-capability-registry-cache-<layer>
JUSTIFICATION:
- microservice = foundry-runtime; bc-tokens = capability-registry-cache.
- layer = <layer>:
  - kernel: port traits (RegistryMirror, CacheStore, RegistryVersionClock) + entities
    (CapabilityDescriptor, RegistryVersion, CacheEntry).
  - usecase: read-through cache + invalidate orchestrator.
  - api: typed contracts.
  - adapter: in-memory dictionary default.
  - adapter-postgres: backend-qualified adapter for the durable mirror table
    (Postgres 16 LTS).
  - worker: hot-reload subscriber on CapabilityRegistryUpdated events.
  - app: composition root.
- exemptions: domain elided same as runtime-pool (mechanism, not arithmetic).
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-redis | adapter-postgres | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `capability-executor` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | ✓ |
| `session-state` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | ✓ |
| `invocation-orchestrator` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | — | ✓ |
| `runtime-pool` | ✓ | — | ✓ | ✓ | ✓ | — | — | — | ✓ | — | ✓ |
| `capability-registry-cache` | ✓ | — | ✓ | ✓ | ✓ | — | ✓ | — | ✓ | — | ✓ |

Total crates introduced by this µservice: **35** (8 + 9 + 7 + 6 + 7 = 35; one canonical crate per cell above).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `CapabilityResolver` | `oya-foundry-runtime-capability-executor-kernel` | `oya-foundry-runtime-capability-registry-cache-adapter` (in-proc cache lookup) | `INTERNAL_ONLY` (descriptor) |
| `ProviderInvoker` | same | `oya-foundry-runtime-capability-executor-adapter` (mTLS client to `foundry-providers`) | `BEHAVIORAL_TENANT_PRODUCT` |
| `GuardrailChecker` | same | `oya-foundry-runtime-capability-executor-adapter` (mTLS client to `foundry-guardrails`) | `BEHAVIORAL_TENANT_PRODUCT` |
| `EvidenceEmitter` | same | `oya-foundry-runtime-capability-executor-adapter` (mTLS client to `foundry-evidence`) | `AUDIT` |
| `AutonomyGate` | same | `-usecase` (pure; reads tenant tier ceiling from registry cache) | `AUDIT` |
| `SessionStore` | `oya-foundry-runtime-session-state-kernel` | `-adapter-redis` (hot tier) + `-adapter-postgres` (cold restore) | `BEHAVIORAL_TENANT_PRODUCT`, `SENSITIVE_PIPA_ART23` (per session jurisdiction tag) |
| `SessionLeaseManager` | same | `-adapter-redis` | `INTERNAL_ONLY` |
| `SessionMutationLog` | same | `-adapter-postgres` | `AUDIT` |
| `LifecycleStore` | `oya-foundry-runtime-invocation-orchestrator-kernel` | `-adapter` (Valkey backing for hot lifecycle index) | `AUDIT` |
| `EventEmitter` | same | `-adapter` (AsyncAPI bus client) | `AUDIT` |
| `TimeoutClock` | same | `-adapter` (monotonic clock) | `INTERNAL_ONLY` |
| `CancellationSignal` | same | `-adapter` (Valkey pub/sub) | `INTERNAL_ONLY` |
| `PodFactory` | `oya-foundry-runtime-runtime-pool-kernel` | `-adapter` (Kubernetes client-go) | `INTERNAL_ONLY` |
| `PoolHealthProbe` | same | `-adapter` (HTTP probe + kube watch) | `INTERNAL_ONLY` |
| `DrainController` | same | `-adapter` (graceful pod retire + invocation re-park) | `INTERNAL_ONLY` |
| `RegistryMirror` | `oya-foundry-runtime-capability-registry-cache-kernel` | `-adapter-postgres` (Postgres mirror) + foundry-supervisor pull | `INTERNAL_ONLY` |
| `CacheStore` | same | `-adapter` (in-process dictionary) | `INTERNAL_ONLY` |
| `RegistryVersionClock` | same | `-adapter` (Postgres `registry_version` table read) | `INTERNAL_ONLY` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `foundry-runtime` MUST NOT import any other product µservice crate at any layer. Cross-µservice traffic to siblings (`foundry-providers`, `foundry-guardrails`, `foundry-evidence`, `foundry-supervisor`) is over mTLS-protected REST/gRPC contracts, not direct crate dependency. LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice foundry-runtime`
- `oya gate validate lean-a2 --microservice foundry-runtime`
- `oya gate validate port-location --microservice foundry-runtime`
- `oya gate validate layer-correctness --microservice foundry-runtime`
- `oya gate validate per-microservice-layout --microservice foundry-runtime`
- `oya gate validate statelessness --microservice foundry-runtime` (executor + orchestrator must be stateless-compatible; state lives in -adapter-redis / -adapter-postgres)
- `oya gate validate shardability --microservice foundry-runtime`
- `oya gate validate authority-cohesion` (registers HG-FR per ADR-0123)

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `InvocationStarted` | capability dispatch accepted | foundry-evidence; observability | invocation-lifecycle-state-machine |
| `InvocationStepEmitted` | per provider/guardrail call within an invocation | foundry-evidence; observability | — |
| `InvocationCompleted` | invocation reaches terminal success state | foundry-evidence; observability; workflow-engine (waiting steps) | — |
| `InvocationFailed` | terminal failure (provider error, guardrail block, timeout) | foundry-evidence; observability; workflow-engine | — |
| `InvocationCancelled` | tenant or supervisor cancellation; pod drain | foundry-evidence; observability | — |
| `AutonomyViolationDetected` | capability requested above tenant tier ceiling | foundry-evidence; ops-security; observability OnCall | — |
| `SessionEvicted` | Valkey evicts a session (LRU or TTL); audit emitted | foundry-evidence; tenant ops portal | — |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `CapabilityRegistryUpdated` | foundry-supervisor (on capability descriptor PR-merge) | `capability-registry-cache` | hot-reload affected descriptors within ≤30s |
| `TenantTierCeilingChanged` | tenancy µservice | `capability-executor` (via autonomy-gate) | refresh tenant ceiling in cache |
| `ProviderCredentialRotated` | foundry-providers | `capability-executor` | drain in-flight invocations bound to old credential generation; new generation picks up next dispatch |
| `GuardrailRulesetUpdated` | foundry-guardrails | `capability-executor` | next dispatch uses new ruleset version |
| `TenantDsrCascade` | tenancy µservice | `session-state` | scan session-state for affected subject identifiers; soft-delete within grace window |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Invocation{tenant, capability_id, session_id, started_at, status, outcome}` | `invokes→Capability`; `during→Session` | `invocation-orchestrator` | Ed25519 |
| `Session{tenant, session_id, jurisdiction, opened_at, last_active_at, status}` | `owned_by→Tenant` | `session-state` | Ed25519 |
| `RuntimePod{pod_id, pack, generation, joined_at, drained_at}` | `member_of→RuntimePool` | `runtime-pool` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Tenant` (catalog) | `capability-executor` (autonomy-gate) | `filter(tenant_id).read(autonomy_tier_ceiling, jurisdiction)` |
| `Capability` (registry; mirrored locally) | `capability-executor` | `filter(capability_id, tenant_id).read(descriptor, declared_tier, ruleset_version)` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| AWS | Bedrock Agent runtime | hosted agent invocation; session memory; tool dispatch; provider neutrality | `docs.aws.amazon.com/bedrock/latest/userguide/agents.html` |
| Google Cloud | Vertex AI Agent Builder | agent orchestration; tool calling; eval harness integration | `cloud.google.com/vertex-ai/docs/generative-ai/agents/overview` |
| Microsoft Azure | Azure AI Foundry runtime | agent runtime; capability registry; safety filters | `learn.microsoft.com/azure/ai-foundry/concepts/agents` |
| LangChain | LangServe + LangGraph cloud | hosted graph execution; session persistence; OpenAPI deploy | `python.langchain.com/docs/langserve/` |
| OpenAI | Assistants API + Threads | hosted threads; tool calls; file context | `platform.openai.com/docs/assistants` |
| LlamaIndex | LlamaCloud Agents | hosted agent execution; eval | `docs.cloud.llamaindex.ai/agents` |

Key parity gaps to close (ordered by priority):

1. **Gate-integrated autonomy tier** — none of the listed competitors carry a first-class autonomy ceiling per principal that the runtime refuses to cross at dispatch time. This is oyatie's differentiator (ADR-0022 + this PRD FR-08).
2. **OpenSLO-gated promotion of capability versions** — competitors deploy capability versions atomically; oyatie gates them through ADR-0139 observability runways before they become tenant-default. Cross-µservice integration with `foundry-supervisor` carries this.
3. **Per-pack residency for session-state** — competitors offer regional residency; oyatie's per-pack model with cross-pack-forbidden is stricter (matches the observability µservice).
4. **Audit-chain native** — Ed25519 + Merkle on every invocation event vs competitors' best-effort logs.
5. **Self-hosted substrate** — Bedrock + Vertex + Foundry runtime + LangServe + Assistants are all SaaS; oyatie ships under Kubernetes the tenant or oyatie operates.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Capability dispatch latency | ≤15ms | ≤50ms | ≤120ms | excluding LLM round-trip |
| Session-state hot read | ≤3ms | ≤10ms | ≤25ms | Valkey hit |
| Session-state cold restore | ≤30ms | ≤100ms | ≤250ms | Valkey miss → Postgres |
| Registry-cache lookup | ≤1ms | ≤10ms | ≤30ms | in-memory |
| Pool warm-pod cold-start | — | ≤500ms | — | per ADR-0020 |
| Invocation completion event | ≤20ms | ≤80ms | ≤200ms | event emission lag |
| Drain (full pod) | ≤30s | ≤60s | — | bounded by longest in-flight |

Error budget:
- Monthly error budget for capability-executor: 0.05% (≈22 min/month).
- Monthly error budget for session-state hot path: 0.10% (≈44 min/month).
- Burn-rate alarm on the runtime itself: 14.4× over 1h triggers page.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Rationale: capability-executor + invocation-orchestrator + runtime-pool components are stateless-compatible (state externalised to Valkey + Postgres + foundry-supervisor); session-state owns hot state in Valkey cluster; capability-registry-cache owns a Postgres-backed mirror with in-memory cache.

**Active-active compatibility**: `stateless-compatible` for executor + orchestrator + pool components; session-state is horizontally shardable by `(tenant, session_id_prefix)` to Valkey cluster slots; capability-registry-cache replicates the Postgres mirror.

Per-cell capacity envelope (XS tier, M01 launch):

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max concurrent invocations | 5,000 | 50,000 | runtime-pool queue depth > 200 per pod |
| Max active sessions | 50,000 | 500,000 | Valkey memory > 70% |
| Capabilities mirrored | 10,000 | 100,000 | Postgres mirror table > 1 GB |
| Dispatch throughput | 1,000/s | 10,000/s | executor CPU > 70% |
| Session-state ops/sec | 10,000/s | 100,000/s | Valkey ops/s > 70% of cluster ceiling |

Scale-out policy:
- Kubernetes HPA: executor + orchestrator + pool worker pods scale on CPU `>70%`; min 3 replicas (HA quorum), max 200 replicas per pack.
- Valkey cluster: 6-shard primary + replica per pack; scale shards on memory > 70%.
- Postgres mirror: read-replica fanout up to 8 replicas per pack; primary scales vertically until `db.standard.E4.16`.
- Pre-warmed pool: per `capacity-model.md`; cold-start ≤500ms.

Cross-region story:
- M01 launch: single KR region (OCI ap-seoul-1); per-tenant residency locked per ADR-0117.
- Post-M01: per-pack expansion to EU, US, US-HC, JP, SG, AU, IN, BR, AE, KSA; cross-pack-forbidden invariant matches observability µservice.

Sharding:
- Sessions shard by `(tenant_id, session_id_prefix)` to Valkey cluster slot.
- Invocation lifecycle records shard by `(tenant_id, invocation_id_prefix)`.
- Capability mirror partitions by `tenant_id` for Postgres mirror tables.
- `oya-check-shardability-cli` CI lane verifies partition key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A capability descriptor registered through `foundry-supervisor` is reachable by `foundry-runtime` within ≤30s | end-to-end integration test under `tests/integration/registry-hot-reload.rs` |
| AC-02 | Dispatch latency p99 ≤50ms over a 1-hour synthetic load | load test under `tests/load/dispatch-latency.rs` with k6/locust-style harness |
| AC-03 | Session-state hot read p99 ≤10ms over a 1-hour synthetic load | load test `tests/load/session-state-hot.rs` |
| AC-04 | Capability invocation above tenant tier ceiling is refused + emits `AutonomyViolationDetected` | e2e under `tests/e2e/autonomy-tier-refusal.rs` |
| AC-05 | Pod drain retires a pod with in-flight invocations within ≤60s and zero data loss | e2e under `tests/e2e/runtime-pod-drain.rs` |
| AC-06 | Cross-tenant session read returns 403 + Cedar audit | integration `tests/integration/cross-tenant-refusal.rs` |
| AC-07 | DSR cascade soft-deletes affected session fragments within 30 days | e2e `tests/e2e/dsr-cascade-session.rs` (synthetic 30-day clock) |
| AC-08 | All Helm charts deploy clean against a kind cluster | CI lane `oya-foundry-runtime-iac-smoke` |
| AC-09 | `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-runtime` exit 0 | ADR-0131 lane |
| AC-10 | `cargo run -p oya-dev-cli -- gate validate authority-cohesion` exit 0 | ADR-0123 lane; HG-FR registered |
| AC-11 | Provider-credential leakage probe finds no provider secret in any runtime pod env or memory dump | security e2e `tests/e2e/provider-credential-isolation.rs` |
| AC-12 | OpenSLO manifests at `microservices/foundry-runtime/slos/{availability,latency,correctness,freshness}.openslo.yaml` validate | observability lane `openslo-conformance` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Should the runtime-pool live in the same cluster as observability or its own? Default leaning: dedicated cluster matching AWS Bedrock + GCP Vertex isolation posture. | ops-sre-reliability + axis-foundry-runtime | resolved in IP-001 |
| 2 | Session-state hot tier — Valkey OSS vs Valkey Stack vs KeyDB; trade-off of streams + JSON modules vs OSS LTS posture | axis-foundry-runtime | resolved in IP-002 (Valkey 8.1 (Redis wire-compat) OSS LTS) |
| 3 | Multi-tenant capability registry mirror — Postgres logical replication vs CDC stream from foundry-supervisor | axis-foundry-runtime + axis-foundry | ADR-#### successor-IP |
| 4 | Capability cold-load (cache miss + supervisor pull) cost — acceptable to defer first-call by ≤500ms or must keep ≤50ms always (pre-warming required) | axis-foundry-runtime | resolved in IP-005 (pre-warmed on registration) |
| 5 | Tenant-supplied capability code (custom logic) execution model — sandboxed WASM vs out-of-process container vs disallowed for M01 | council-architecture + axis-foundry-runtime | resolved in IP-008 (disallowed in M01; tenant capabilities are descriptor-only until ADR-0NNN sandbox decision) |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0022 | Autonomy tiers T0-T4 | tier ceiling source of authority |
| ADR-0024 | Foundry eval harness | invocation outputs flow into eval harness |
| ADR-0025 | Foundry runtime consolidation | original design intent for this µservice |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application→usecase rename | layer authority |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0123 | Hyperscaler maturity claim gate | HG-FR registers here |
| ADR-0139 | Agentic SLO-gated promotion | capability versions gated through it |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0132 | No-suite forward policy | no compat seams |
| ADR-0133 | Industry-best-practice conformance | OSS LTS pin posture |
