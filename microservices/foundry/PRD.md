---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-foundry
microservice: foundry
status: Accepted
sales_segment: shared-substrate
tier: internal-and-tenant-product
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs: [ADR-0022, ADR-0024, ADR-0025, ADR-0056, ADR-0105, ADR-0106, ADR-0110, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0136, ADR-0137, ADR-0138]
related_specs: [/specs/microservices/foundry.json, /specs/agent-operating-contract.json, /specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
date: 2026-05-18
owner_team: axis-foundry
doc_status: published
bc_archive: microservices/foundry/bc-sources/
---

# PRD-foundry — Hosted Agent Platform (Substrate + Tenant Product)

## Purpose

The `foundry` µservice is oyatie's hosted-agent platform — the substrate that
hosts capability registration, agent runtime execution, provider routing,
guardrail enforcement, evaluation, supervision, and evidence emission as a
single product surface with six internal bounded contexts (BCs). It is the
parallel of AWS Bedrock, Google Vertex AI Agent Builder, Microsoft Azure AI
Foundry, Anthropic Console, and Palantir AIP — each of which ships as **one
product, many internal components** (per ADR-0136).

The six internal BCs are:

| BC | Role | Primary surface |
|---|---|---|
| `runtime` | Hosts agent invocation, session state, capability execution, runtime pool | tenant-facing SDK + REST + gRPC |
| `supervisor` | Fleet lifecycle, capability deployment, kill-switch, autonomy enforcement, supervision event bus | internal control plane + ops portal |
| `eval` | Eval harness, parity analysis, replay determinism, golden-output store | dev + tenant SDK |
| `evidence` | Capability-invocation recording, evidence-pack assembly, regulator export, audit-chain bridge | internal + regulator surface |
| `guardrails` | Prompt classification, output validation, autonomy-tier gate, content safety, jailbreak detection, AI-slop detection | inline in runtime hot path |
| `providers` | LLM-provider router + adapters (Anthropic API+Subscription, OpenAI API+Subscription, Gemini API+Subscription, in-house, OpenBao credential isolation) | runtime-internal + ops portal |

Per ADR-0137 (foundry bounded contexts), each BC owns its own contract surface,
its own kernel/domain/usecase crates, and its own deployment topology — yet
they ship as a single product because they are operationally inseparable: an
invocation cannot complete without provider routing AND guardrail enforcement
AND evidence emission AND supervisor approval AND runtime execution. Splitting
them externally adds network hops without capability isolation; consolidating
them internally allows BC owners to evolve independently without forcing
cross-µservice deployment dances.

## Tenant Value

Per the six BCs:

- **Hosted capability execution without per-tenant infra** (runtime BC) —
  Tenants register a capability descriptor; the foundry materialises pods,
  attaches sessions, dispatches LLM + tool calls. No tenant-side Kubernetes
  required.
- **Session-coherent multi-turn agents** (runtime BC) — Sessions persist
  across turn boundaries with per-tenant Redis-backed isolation.
- **Autonomy-tier-gated execution** (runtime + guardrails + supervisor BCs) —
  Tenant capabilities declare an ADR-0022 autonomy tier (T0–T4); execution
  outside authorised ceiling refuses with audit emission.
- **Substrate uniformity** — Every oyatie product invoking agents goes through
  the same foundry.
- **Capability deployment + kill-switch** (supervisor BC) — Tenants deploy and
  emergency-disable capability fleets without per-pod intervention.
- **Reproducible offline evaluation** (eval BC) — Tenants run capability evals
  against golden outputs with provider-parity comparison.
- **Audit-chain-grade evidence** (evidence BC) — Every invocation emits
  Ed25519+Merkle-sealed evidence; regulator exports assemble compliance packs
  on demand.
- **Inline safety enforcement** (guardrails BC) — Prompt + output + autonomy-
  tier gating happens inline in the runtime hot path, not after the fact.
- **Provider-neutral routing** (providers BC) — Tenants do not bind a
  capability to a specific provider; the router selects per policy and
  cost/latency constraints. Subscription-mode adapters (Anthropic/OpenAI/
  Gemini) coexist with API-mode adapters under uniform contract.

This µservice is **shared substrate** (the platform; same shape every tenant)
**and an end-user product surface** (tenants address it directly via SDK and
console). Its existence is the precondition for every Foundry-class product
per `feedback_quality_performance_scalability_bar.md` and the Workflow Studio
scope in `feedback_workflow_studio_scope.md`.

This µservice has no direct Bominal equivalent and originates in oyatie per
ADR-0025 (foundry runtime consolidation) + ADR-0136 (foundry as single
µservice).

## Functional Requirements

Functional requirements are owned per BC. The canonical per-BC FR matrix
lives at `bc-sources/<bc>/PRD.md §"Functional Requirements"`. Cross-BC
invariants:

| ID | Invariant | BCs touched |
|---|---|---|
| FR-X1 | Every invocation traverses the chain: supervisor (admit) → runtime (dispatch) → guardrails (pre-check) → providers (LLM call) → guardrails (post-check) → evidence (seal) → runtime (return) | all 6 |
| FR-X2 | Capability deployment is initiated by supervisor, mirrored to runtime registry-cache within ≤30s | supervisor, runtime |
| FR-X3 | Provider credential rotation drains in-flight invocations bound to old credential generation without cross-tenant leakage | providers, runtime |
| FR-X4 | Autonomy-tier violation refuses dispatch + emits `AutonomyViolationDetected` consumable by ops-security | runtime, guardrails, supervisor, evidence |
| FR-X5 | Eval harness consumes capability descriptors from supervisor mirror; replays through runtime in a sandboxed pool | eval, supervisor, runtime |
| FR-X6 | Evidence packs assemble across BCs: invocation evidence (runtime), supervision events (supervisor), guardrail decisions (guardrails), eval outcomes (eval), provider receipts (providers) | all 6 |
| FR-X7 | Kill-switch engagement by supervisor halts runtime fleets within ≤10s + emits audit-chain record | supervisor, runtime, evidence |

Per-BC FRs (consult the bc-sources archive for the full matrix):

- `bc-sources/runtime/PRD.md` — FR-01..FR-12 (capability dispatch, session
  state, pool, registry cache, autonomy gate, DSR cascade, hot-reload).
- `bc-sources/supervisor/PRD.md` — fleet lifecycle, capability deployment,
  kill-switch, autonomy policy enforcement, supervision event bus.
- `bc-sources/eval/PRD.md` — eval runner, parity analysis, replay
  determinism, golden-output store.
- `bc-sources/evidence/PRD.md` — capability-invocation recording, evidence-
  pack builder, regulator export, audit-chain bridge.
- `bc-sources/guardrails/PRD.md` — prompt classifier, output validator,
  autonomy-tier gate, content-safety rule engine, jailbreak detector,
  AI-slop detector.
- `bc-sources/providers/PRD.md` — provider router, Anthropic/OpenAI/Gemini
  API+Subscription adapters, in-house adapter, OpenBao credential adapter.

## Non-Functional Requirements

### Performance

The headline performance envelope is dispatch latency p99 ≤50ms excluding LLM
round-trip (runtime BC owns; foundry inherits). Each BC carries its own
sub-envelope:

| BC | Headline NFR | p99 budget |
|---|---|---|
| runtime | capability dispatch latency | ≤50ms |
| supervisor | fleet command propagation | ≤5s |
| eval | eval-run scheduling latency | ≤500ms |
| evidence | evidence-pack assembly | ≤2s/100MB |
| guardrails | inline guardrail check | ≤20ms |
| providers | router decision latency | ≤5ms |

The per-BC PRD in `bc-sources/<bc>/PRD.md §"Non-Functional Requirements"`
carries the full Performance / Security / Audit / Availability / Data-
residency matrix for that BC.

### Security

- All BCs authenticate inter-BC over mTLS with per-pod SPIFFE identity
  (`spiffe://oyatie/foundry/<bc>/<pod>`).
- All cross-tenant access is default-deny via Cedar policy.
- Provider credentials are isolated to the providers BC and OpenBao-bound;
  no other BC ever sees them.
- Session state, evidence packs, and supervision events are encrypted at rest
  + in transit.
- Audit-chain (Ed25519+Merkle per Bominal ADR-0028) seals every cross-BC
  state transition.

### Audit + Compliance

- Every invocation start/complete/fail/cancel; every supervision command;
  every guardrail decision; every eval-run completion; every provider
  receipt; every evidence-pack assembly — all emit audit-chain records.
- Retention: 1 year baseline; 6 years for pack-us-healthcare PHI-touching
  paths per HIPAA §164.316(b)(2); pack-eu retention per GDPR Art.30.
- Per-BC `compliance.md` (see `bc-sources/<bc>/compliance.md`) enumerates
  the per-BC regulatory mapping.

### Availability + SLO

- Availability target: 99.95% monthly for the canonical hot path (runtime
  dispatch); 99.9% for guardrails inline; 99.9% for providers router; 99.9%
  for supervisor; 99.5% for eval; 99.9% for evidence.
- RTO: ≤5 min per BC. RPO: ≤30s per BC (state lives in adapter-redis /
  adapter-postgres / adapter-s3 with sync replication where appropriate).

### Data residency

- All BCs inherit the tenant's `jurisdiction_code` per ADR-0117. Per-pack
  Redis/Postgres/S3/ClickHouse instances enforce residency; cross-pack
  state migration is forbidden by default.

## Bounded Contexts

Per ADR-0137, foundry has **six internal BCs**. Each BC's crate fan-out
follows ADR-0131 (per-µservice flat layout) and ADR-0105 (13-layer enum):

### BC: runtime

Crate families:

- `oya-foundry-runtime-capability-executor-{kernel,domain,usecase,api,adapter,rest,sdk,app}`
- `oya-foundry-runtime-session-state-{kernel,domain,usecase,api,adapter,adapter-redis,adapter-postgres,sdk,app}`
- `oya-foundry-runtime-invocation-orchestrator-{kernel,domain,usecase,api,adapter,worker,app}`
- `oya-foundry-runtime-runtime-pool-{kernel,usecase,api,adapter,worker,app}`
- `oya-foundry-runtime-capability-registry-cache-{kernel,usecase,api,adapter,adapter-postgres,worker,app}`

### BC: supervisor

Crate families:

- `oya-foundry-supervisor-agent-fleet-lifecycle-{kernel,domain,usecase,api,adapter,adapter-k8s-operator,adapter-postgres,rest,sdk,worker,app}`
- `oya-foundry-supervisor-autonomy-policy-enforcement-{kernel,domain,usecase,api,adapter,rest,sdk,app}`
- `oya-foundry-supervisor-capability-deployment-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,worker,app}`
- `oya-foundry-supervisor-supervision-event-bus-{kernel,usecase,api,adapter,sdk,worker,app}`
- `oya-foundry-supervisor-kill-switch-circuit-breaker-{kernel,domain,usecase,api,adapter,adapter-k8s-operator,rest,sdk,worker,app}`

### BC: eval

Crate families:

- `oya-foundry-eval-eval-runner-{kernel,domain,usecase,api,adapter,adapter-gpu,adapter-s3,rest,sdk,worker,app}`
- `oya-foundry-eval-parity-analyzer-{adapter-clickhouse,...}` (full fan-out per `bc-sources/eval/PRD.md`)

### BC: evidence

Crate families:

- `oya-foundry-evidence-evidence-pack-builder-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,adapter-audit-chain-bridge,rest,worker,app}`
- `oya-foundry-evidence-capability-invocation-recorder-{kernel,...}`
- `oya-foundry-evidence-sdk`

### BC: guardrails

Crate families:

- `oya-foundry-guardrails-prompt-classifier-{kernel,rest,app}`
- `oya-foundry-guardrails-output-validator-kernel`
- `oya-foundry-guardrails-autonomy-tier-gate-{kernel,adapter-cedar}`
- `oya-foundry-guardrails-content-safety-rule-engine-{kernel,adapter-postgres}`
- `oya-foundry-guardrails-jailbreak-detector-{kernel,adapter-classifier-model}`
- `oya-foundry-guardrails-ai-slop-detector-kernel`

### BC: providers

Crate families:

- `oya-foundry-providers-router-{kernel,domain,usecase,api,adapter,rest,sdk,worker,app}`
- `oya-foundry-providers-adapter-{anthropic-api,anthropic-subscription,openai-api,openai-subscription,gemini-api,gemini-subscription,in-house,openbao}`

**Crate names preserved across migration.** Per the consolidation contract,
crate names use the `oya-foundry-<bc>-<feature>-<layer>` BNF v4.1 form;
moving from `microservices/foundry-<bc>/` to `microservices/foundry/` does
NOT rename crates. The Cargo catalog at `microservices/foundry/catalog/`
preserves all 135 crate-catalog records.

## Integration via Workflow + Ontology

Per `feedback_workflow_objectgraph_adapter_layer.md`: all inter-BC and
cross-µservice traffic flows through Workflow events + Ontology reads/writes.
No direct cross-product crate dependency.

### Cross-BC workflow events (internal foundry)

| Event | Producer BC | Consumer BC | Purpose |
|---|---|---|---|
| `CapabilityRegistryUpdated` | supervisor | runtime | hot-reload registry cache |
| `KillSwitchEngaged{tenant,scope}` | supervisor | runtime | drain affected fleets |
| `ProviderCredentialRotated{generation}` | providers | runtime | rotate hot adapters |
| `GuardrailRulesetUpdated` | guardrails | runtime | rotate ruleset |
| `EvalRunCompleted{capability,outcome}` | eval | supervisor | promotion decision input |
| `AutonomyViolationDetected` | runtime, guardrails | supervisor, evidence | record + page ops-security |
| `InvocationCompleted{invocation_id}` | runtime | evidence, eval | seal + aggregate |

### Cross-µservice workflow events (external)

| Event | Direction | Counterparty µservice |
|---|---|---|
| `TenantTierCeilingChanged` | in | tenancy |
| `TenantDsrCascade` | in | tenancy |
| `CapabilityInvocationCompleted` | out | observability, workflow-engine |
| `EvidencePackSealed` | out | audit-chain |
| `RegulatorExportRequested` | in | audit-chain |

### Ontology surfaces

Per BC; consult `bc-sources/<bc>/PRD.md §"Ontology writes/reads"` for full
matrix. Cross-BC objects: `Invocation`, `Session`, `RuntimePod` (runtime BC
owns) link to `Capability`, `CapabilityDescriptor`, `CapabilityVersion`
(supervisor BC owns) and `EvalSet`, `EvalRun`, `GoldenOutput` (eval BC owns).

## Competitive Benchmark

| Competitor | Product | Single-product? | Internal BCs |
|---|---|---|---|
| AWS | Bedrock | yes | agent runtime, model catalog, guardrails, knowledge bases, agent supervision |
| Google Cloud | Vertex AI Agent Builder | yes | agent runtime, model garden, safety filters, eval, deploy |
| Microsoft Azure | Azure AI Foundry | yes | agent runtime, model catalog, safety filters, eval, deploy, supervision |
| Anthropic | Console / Claude API | yes | API surface, system prompts, tools, evaluations, usage |
| Palantir | AIP | yes | AIP Logic, AIP Threads, AIP Evals, AIP Operator, AIP Tools |
| LangChain | LangSmith + LangGraph | yes | LangGraph (runtime), LangSmith (eval+trace), LangServe (deploy) |

All competitors ship **one product** with internal BCs that match foundry's
6-BC split (runtime / supervisor / eval / evidence / guardrails / providers).
This is the topology ADR-0136 codifies.

Per `bc-sources/<bc>/competitor-parity-matrix.md` for per-BC parity matrices.

## Performance Targets

See per-BC `bc-sources/<bc>/PRD.md §"Performance Targets"`. Aggregate envelope:

| Path | p50 | p99 | p999 |
|---|---|---|---|
| End-to-end invocation (runtime+guardrails+providers; excluding LLM) | ≤25ms | ≤80ms | ≤200ms |
| Inline guardrail check | ≤8ms | ≤20ms | ≤50ms |
| Provider router decision | ≤2ms | ≤5ms | ≤15ms |
| Supervision command fan-out | ≤1s | ≤5s | ≤10s |
| Evidence pack assembly (100MB) | ≤800ms | ≤2s | ≤5s |
| Eval-run scheduling | ≤150ms | ≤500ms | ≤1.2s |

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed` across BCs:

| BC | Strategy | Hot store | Cold store |
|---|---|---|---|
| runtime | stateless-compatible executor + Redis-shardable session-state | Redis | Postgres |
| supervisor | stateless-compatible commands + Postgres fleet-state | — | Postgres |
| eval | stateless-compatible runner + ClickHouse parity store | — | ClickHouse + S3 (golden) |
| evidence | stateless-compatible builder + Postgres index + S3 blob | — | Postgres + S3 |
| guardrails | stateless inline checkers + Postgres rule store + ONNX classifier | — | Postgres |
| providers | stateless router + Redis rate-limit + OpenBao credential | Redis | OpenBao + Postgres |

**Active-active compatibility**: stateless-compatible (executor, supervisor,
eval-runner, evidence-builder, guardrail checkers, providers router); state
sharded by `(tenant, ${entity}_id_prefix)` in adapters.

Per-cell capacity envelope (XS tier, M01 launch): consult per-BC
`capacity-model.md` (canonical merged version at
`microservices/foundry/capacity-model.md`).

## Acceptance Criteria

Foundry's acceptance is the union of all 6 BCs' AC matrices + cross-BC
invariants FR-X1..FR-X7. Per-BC ACs at `bc-sources/<bc>/PRD.md §"Acceptance
Criteria"`. Cross-BC acceptance:

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-X1 | End-to-end invocation traverses all 6 BCs and emits evidence pack | e2e test `tests/e2e/cross-bc-invocation.rs` |
| AC-X2 | Kill-switch from supervisor halts a runtime fleet within ≤10s | e2e test `tests/e2e/kill-switch-fleet-halt.rs` |
| AC-X3 | Capability registered by supervisor reaches runtime registry-cache within ≤30s | integration `tests/integration/cross-bc-registry-sync.rs` |
| AC-X4 | Provider credential rotation drains in-flight invocations with zero data loss | e2e `tests/e2e/credential-rotation-drain.rs` |
| AC-X5 | Autonomy violation refuses dispatch + page ops-security + emit audit record | e2e `tests/e2e/autonomy-violation-end-to-end.rs` |
| AC-X6 | Eval run replays a recorded invocation deterministically through runtime sandbox pool | e2e `tests/e2e/eval-replay-determinism.rs` |
| AC-X7 | Evidence-pack assembly aggregates contributions from all 6 BCs for one (tenant, period) | e2e `tests/e2e/evidence-pack-cross-bc.rs` |
| AC-X8 | All Helm sub-charts deploy clean against a kind cluster | CI lane `oya-foundry-iac-smoke` |
| AC-X9 | `oya gate validate per-microservice-layout --microservice foundry` exit 0 | ADR-0131 lane |
| AC-X10 | `oya gate validate authority-cohesion` exit 0 | ADR-0123 lane |
| AC-X11 | All 4 OpenSLO manifests at `slos/` validate | observability lane `openslo-conformance` |
| AC-X12 | All 6 BCs' contract surfaces validate (OpenAPI lint + AsyncAPI lint + protobuf-lint) | CI lane `oya-foundry-contracts` |

## Open Questions

Carried per BC; consult `bc-sources/<bc>/PRD.md §"Open Questions"` for the
list. Cross-BC open questions:

| # | Question | Owner | Target |
|---|---|---|---|
| X1 | Should the 6 BCs share a single Kubernetes namespace or split across 6? Default: 1 namespace per BC for blast-radius isolation. | ops-sre-reliability + axis-foundry | ADR-#### subsequent-to-M01-completion |
| X2 | Cross-BC tracing: OpenTelemetry trace IDs propagated through all 6 BCs; sampling policy. | axis-foundry + observability µservice | ADR-0114 successor-IP |
| X3 | Versioning: per-BC contract versioning vs unified foundry version. Default: per-BC SemVer; foundry product version is a digest of the 6. | council-architecture | ADR-#### |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0022 | Autonomy tiers T0-T4 | tier ceiling source of authority |
| ADR-0024 | Foundry eval harness | eval BC origin |
| ADR-0025 | Foundry runtime consolidation | runtime BC origin |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application→usecase rename | layer authority |
| ADR-0110 | ChangeSet state machine | per-IP ChangeSets |
| ADR-0123 | Hyperscaler maturity claim gate | HG-FOUNDRY registers here |
| ADR-0139 | Agentic SLO-gated promotion | capability + per-BC version gating |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | No-suite forward policy | foundry is one µservice, not a suite |
| ADR-0133 | Industry-best-practice conformance | OSS LTS pin posture |
| ADR-0136 | Foundry as single µservice | **this consolidation** |
| ADR-0137 | Foundry bounded contexts | 6-BC contract |
| ADR-0138 | Foundry six-path deprecation | Strangler migration off old paths |

## Per-BC Archive

All six per-BC PRDs are preserved verbatim under
`microservices/foundry/bc-sources/<bc>/PRD.md`. The per-BC PRDs remain the
authoritative source for BC-internal contract surfaces, port traits, layer
maps, data-class annotations, and per-BC acceptance criteria. This top-level
PRD is the canonical foundry-product surface; the per-BC PRDs are its
chapters.
