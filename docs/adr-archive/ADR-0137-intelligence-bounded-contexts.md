---
id: ADR-0137
status: Superseded
deciders: council-architecture, axis-foundry, axis-foundry-runtime, axis-foundry-supervisor, axis-foundry-eval, axis-foundry-evidence, axis-foundry-guardrails, axis-foundry-providers
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0389]
supersession_note: "Foundry bounded-contexts declared dead context (D-FOUNDRY-CLARIFY); superseded by ADR-0389 cloud-intelligence framework successor. D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-11."
related: [ADR-0022, ADR-0025, ADR-0056, ADR-0105, ADR-0106, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0136, ADR-0138]
related_memory: [feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145), feedback_naming_justification, feedback_quality_performance_scalability_bar]
related_specs:
  - /specs/microservices/foundry.json
purpose: |
  Companion to ADR-0136. Names the six internal bounded contexts of the
  foundry µservice (runtime, supervisor, eval, evidence, guardrails,
  providers), enumerates each BC's contract surface, owner sub-axis,
  and inter-BC dependency rules. Establishes the BC-boundary invariants
  that ADR-0136's "one product, six BCs" decision depends on.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0137: Foundry bounded contexts

## Status

Accepted — 2026-05-18.

## Date

2026-05-18.

## Context

ADR-0136 establishes that foundry is one µservice. To preserve per-BC
ownership clarity, contract-surface modularity, and ubiquitous-language
coherence per DDD (Eric Evans 2003; Vaughn Vernon 2013), the µservice's
internal structure must be a typed BC partition — not a flat
implementation file pile. This ADR enumerates the six BCs and their
inter-BC dependency rules.

The six BCs were already implied by the prior 6-way µservice split. This
ADR makes them first-class internal structure within the consolidated
`microservices/foundry/` µservice.

## Decision

Foundry contains **exactly six bounded contexts**, named below. The BC
list is closed and grows only by ADR amendment.

### BC: `runtime`

- **Purpose**: hosts the capability invocation hot path — capability
  executor, session state, invocation lifecycle orchestrator, runtime
  pod pool, capability registry cache (read-through mirror of the
  supervisor's canonical registry).
- **Owner sub-axis**: axis-foundry-runtime.
- **Crate fan-out**: `oya-intelligence-runtime-{capability-executor,
  session-state, invocation-orchestrator, runtime-pool, capability-
  registry-cache}-{kernel,domain,usecase,api,adapter,adapter-redis,
  adapter-postgres,rest,sdk,worker,app}` per `bc-sources/runtime/PRD.md`
  matrix.
- **Contract surface**: `contracts/openapi/runtime-foundry-runtime.yaml`
  + `contracts/asyncapi/runtime-foundry-runtime-events.yaml` +
  `contracts/proto/runtime-foundry-runtime.proto`.
- **Tenant-facing**: yes (SDK + REST + gRPC).
- **Headline NFR**: dispatch p99 ≤50ms excluding LLM round-trip.

### BC: `supervisor`

- **Purpose**: fleet lifecycle, capability deployment, kill-switch +
  circuit-breaker, autonomy policy enforcement, supervision event bus.
  The control plane of the foundry product.
- **Owner sub-axis**: axis-foundry-supervisor.
- **Crate fan-out**: `oya-intelligence-supervisor-{agent-fleet-lifecycle,
  capability-deployment, kill-switch-circuit-breaker, autonomy-policy-
  enforcement, supervision-event-bus}-{kernel,...,app}` per
  `bc-sources/supervisor/PRD.md`.
- **Contract surface**: `contracts/openapi/supervisor-foundry-supervisor.yaml`
  + `contracts/asyncapi/supervisor-foundry-supervisor-events.yaml` +
  `contracts/proto/supervisor-foundry-supervisor.proto`.
- **Tenant-facing**: yes (ops portal + SDK for capability deploy + kill-
  switch + fleet query).
- **Headline NFR**: command propagation p99 ≤5s.

### BC: `eval`

- **Purpose**: eval harness — eval runner, parity analyzer (compare
  capability output across providers / versions), replay engine, golden-
  output store.
- **Owner sub-axis**: axis-foundry-eval.
- **Crate fan-out**: `oya-intelligence-eval-{eval-runner,parity-analyzer,
  replay-engine,...}-{kernel,...,app}` per `bc-sources/eval/PRD.md`.
- **Contract surface**: `contracts/openapi/eval-eval-runner.yaml` +
  `contracts/asyncapi/eval-eval-events.yaml` +
  `contracts/proto/eval-eval_runner.proto`.
- **Tenant-facing**: yes (dev tooling + SDK).
- **Headline NFR**: eval scheduling p99 ≤500ms.

### BC: `evidence`

- **Purpose**: capability-invocation recorder, evidence pack builder,
  regulator export, audit-chain bridge.
- **Owner sub-axis**: axis-foundry-evidence.
- **Crate fan-out**: `oya-intelligence-evidence-{capability-invocation-
  recorder,evidence-pack-builder,...}-{kernel,...,app}` +
  `oya-intelligence-evidence-sdk` per `bc-sources/evidence/PRD.md`.
- **Contract surface**: `contracts/openapi/evidence-foundry-evidence.yaml`
  + `contracts/asyncapi/evidence-foundry-evidence-events.yaml` +
  `contracts/proto/evidence-foundry-evidence.proto`.
- **Tenant-facing**: yes (regulator export SDK + ops query API).
- **Headline NFR**: pack assembly p99 ≤2s per 100MB.

### BC: `guardrails`

- **Purpose**: prompt classifier, output validator, autonomy-tier gate
  (Cedar adapter), content-safety rule engine, jailbreak detector,
  AI-slop detector.
- **Owner sub-axis**: axis-foundry-guardrails.
- **Crate fan-out**: `oya-intelligence-guardrails-{prompt-classifier,
  output-validator,autonomy-tier-gate,content-safety-rule-engine,
  jailbreak-detector,ai-slop-detector}-{kernel,adapter-cedar,
  adapter-classifier-model,adapter-postgres,rest,app}` per
  `bc-sources/guardrails/PRD.md`.
- **Contract surface**: `contracts/openapi/guardrails-guardrails.yaml` +
  `contracts/asyncapi/guardrails-decision-events.yaml` +
  `contracts/proto/guardrails-guardrails.proto`.
- **Tenant-facing**: no in M01 (invoked inline by runtime); future
  tenant-config surface deferred.
- **Headline NFR**: inline check p99 ≤20ms.

### BC: `providers`

- **Purpose**: LLM provider router + 8 adapters (Anthropic API +
  Subscription, OpenAI API + Subscription, Gemini API + Subscription,
  in-house, OpenBao credential isolation).
- **Owner sub-axis**: axis-foundry-providers.
- **Crate fan-out**: `oya-intelligence-providers-router-{kernel,domain,usecase,
  api,adapter,rest,sdk,worker,app}` + `oya-intelligence-providers-adapter-
  {anthropic-api,anthropic-subscription,openai-api,openai-subscription,
  gemini-api,gemini-subscription,in-house,openbao}` per
  `bc-sources/providers/PRD.md`.
- **Contract surface**: `contracts/openapi/providers-provider-router.yaml`
  + `contracts/asyncapi/providers-provider-events.yaml` +
  `contracts/proto/providers-provider-invoke.proto`.
- **Tenant-facing**: yes (ops portal for provider config + credential
  pin).
- **Headline NFR**: router decision p99 ≤5ms.

### Inter-BC dependency rules

Cross-BC traffic within foundry follows the
`feedback_workflow_objectgraph_adapter_layer.md` rule: BCs talk over
typed events + ontology object references, not direct kernel-port
imports across BCs.

| Producer BC | Consumer BC(s) | Channel | Payload |
|---|---|---|---|
| supervisor | runtime | event `CapabilityRegistryUpdated` | (tenant, capability_id, version) |
| supervisor | runtime, evidence | event `KillSwitchEngaged{tenant,scope}` | (tenant, fleet_scope, engaged_at) |
| supervisor | evidence | event `SupervisionCommandIssued` | (command, scope, issuer, audit_chain_seq) |
| supervisor | eval | event `CapabilityVersionPromotionRequested` | (tenant, capability_id, target_version) |
| providers | runtime | event `ProviderCredentialRotated{generation}` | (provider, generation, rotated_at) |
| providers | evidence | event `ProviderReceiptEmitted` | (invocation_id, provider, model, tokens, cost) |
| guardrails | runtime | event `GuardrailRulesetUpdated` | (tenant, ruleset_version) |
| guardrails | evidence | event `GuardrailDecisionEmitted` | (invocation_id, decision, ruleset_version, hash(prompt)) |
| eval | supervisor | event `EvalRunCompleted{capability, outcome}` | (tenant, capability_id, eval_run_id, outcome) |
| eval | evidence | event `EvalEvidenceEmitted` | (eval_run_id, golden_id, parity_outcome) |
| runtime | guardrails, providers, evidence | per-invocation RPC over mTLS (in-µservice loopback or near-loopback) | (invocation_id, payload, scope) |
| runtime | evidence | event `InvocationCompleted{invocation_id}` | (invocation_id, completed_at, outcome) |
| runtime | guardrails | event `AutonomyViolationDetected` | (invocation_id, tenant, requested_tier, ceiling) |
| runtime | supervisor | event `RuntimePodDrainCompleted` | (pod_id, drained_at) |

**Forbidden inter-BC dependencies** (LEAN-A2-foundry-bc lane enforces;
authored in a follow-up IP):

- Direct kernel-port import across BCs (e.g.,
  `oya-intelligence-runtime-capability-executor-kernel` importing
  `oya-intelligence-providers-router-kernel`) — forbidden. Use the typed
  RPC contract under `contracts/proto/<other-bc>-<service>.proto`.
- Direct adapter import across BCs (e.g., a runtime adapter calling
  guardrails adapter functions directly) — forbidden. Use the
  guardrails REST/gRPC contract.
- Shared-state cross-BC (e.g., runtime + guardrails sharing the same
  Postgres table) — forbidden. Each BC owns its state.
- Ontology read by one BC of another BC's primary-owned object type
  WITHOUT a `feedback_workflow_objectgraph_adapter_layer.md`-compliant
  reader port — forbidden. Each BC declares its object reads through
  an explicit reader port in its kernel.

**Permitted inter-BC**:

- Event subscription via the supervision-event-bus (supervisor BC owns
  the bus; all six BCs may produce + consume events through it).
- Ontology object reads through declared reader ports.
- RPC over the per-BC contract surfaces (openapi / proto) over mTLS.

### BC growth policy

- Net-new BCs require an ADR amendment to this one. The bar: a proposed
  BC must demonstrate (a) independent ubiquitous language, (b)
  independent contract surface, (c) independent state, (d) named owner
  sub-axis, and (e) demonstration that it does not collapse to an
  existing BC's scope.
- BC merge requires an ADR amendment with explicit migration plan
  (similar to ADR-0138).

## Alternatives Considered

### (a) Flat foundry — no internal BC structure

- **Pros**: simplest layout; one PRD, one threat-model, one set of
  contracts.
- **Cons**:
  - Loses DDD ubiquitous-language scoping; runtime's "Session" collides
    with supervisor's "Fleet Session" collides with eval's "Eval Session"
    without BC namespace.
  - Loses per-sub-team ownership clarity.
  - Loses per-BC contract surface modularity (one giant openapi.yaml).
  - Contradicts hyperscaler internal product structure — even AWS
    Bedrock distinguishes Agents / Knowledge Bases / Guardrails in
    documentation, pricing, and APIs.
- **Rejected.** ADR-0136 already enumerates six implicit BCs; making
  them first-class structure is strictly better than burying them.

### (b) Three BCs (collapse runtime+pool, supervisor+guardrails, eval+evidence)

- **Pros**: fewer BCs to learn.
- **Cons**:
  - Collapses orthogonal ubiquitous languages: runtime + pool is one
    language (invocation + pod), supervisor + guardrails is two
    (commands + safety decisions), eval + evidence is two (replays +
    audit packs). Forced collapse loses ubiquitous-language coherence.
  - Collapses orthogonal contract surfaces — clients debugging an eval
    failure no longer have a clean per-BC namespace.
- **Rejected**: false economy.

### (c) Twelve BCs (split runtime's 5 components into BCs)

- **Pros**: maximum modularity.
- **Cons**:
  - Counter to ADR-0136's "match hyperscaler shape" rationale — no
    hyperscaler structures their hosted-agent product into twelve BCs.
  - Many BCs whose ubiquitous language is the same (runtime-pool +
    runtime-orchestrator share Invocation, Pod, Session vocabulary)
    are forced apart.
  - 12-BC governance ceremony (per-BC PRD + threat-model + contract
    surface etc.) is six times the prior 6-way split's documented
    failure mode.
- **Rejected**: over-fragmentation.

### (d) Six BCs (runtime + supervisor + eval + evidence + guardrails + providers)  ← **CHOSEN**

- **Pros**:
  - Matches the six prior-split µservice axes 1:1 — the BC partition is
    already proven by months of independent scaffolding under the prior
    split.
  - Each BC carries its own ubiquitous language, contract surface,
    state, and owner sub-axis.
  - Matches the implicit BC partition in every hyperscaler benchmark
    (AWS Bedrock's internal product divisions, Vertex AI's pillars,
    Azure Foundry's product modules, Anthropic Console's tabs, Palantir
    AIP's product line, LangSmith+LangGraph's pillars).
- **Cons**:
  - Six is more than three; contributors learn six BC namespaces.
- **Accepted**.

## Consequences

### Positive

1. **DDD-compliant BC partition.** Each BC owns its ubiquitous language,
   state, contract surface, and SLO. Per Eric Evans (2003) + Vaughn
   Vernon (2013), this is the architecture good.

2. **Per-BC ownership preserved.** axis-foundry-runtime, -supervisor,
   -eval, -evidence, -guardrails, -providers continue to own their
   BC's contract + crate fan-out + acceptance criteria.

3. **Per-BC contract surface preserved.** Tenants and integrators
   discover per-BC APIs through their own openapi / asyncapi / proto
   files at `contracts/<surface>/<bc>-<service>.<ext>`.

4. **Inter-BC contract rules made explicit + CI-enforced.** The cross-BC
   dependency table above is the single source of truth for what BCs
   may talk to what BCs and over what channel; the LEAN-A2-foundry-bc
   lane (follow-up IP) enforces.

5. **BC growth + merge governance explicit.** Net-new BCs require ADR
   amendment to this one; preserves architectural discipline.

### Negative

1. **Six BCs to learn.** Contributor onboarding learns six BC namespaces;
   mitigated by uniform structure (each BC has the same crate-fan-out
   shape per ADR-0105 + uniform per-BC PRD shape).

2. **BC-boundary CI lane authoring required.** LEAN-A2-foundry-bc lane
   (follow-up IP under `microservices/foundry/IP-NNN-foundry-bc-boundary-lane.md`)
   is BLOCKER-level governance — adds CI runtime cost.

3. **Inter-BC RPC overhead within the µservice.** Cross-BC RPC even
   in-µservice carries serialisation + mTLS overhead vs direct function
   call. Mitigated: in-µservice loopback is near-zero network; cross-BC
   p99 budget is generous (<1ms typical loopback RPC).

### Operational

- **New CI lane** (registered in follow-up ChangeSet):
  `oya-governance-foundry-bc-boundary` — verifies inter-BC dependency
  rules. BLOCKER from M01.
- **New ontology constraint**: each BC's primary-owned object types
  declared in this ADR; ontology-write CI lane refuses cross-BC writes
  to primary-owned objects without reader-port declaration.

## Clean Architecture Impact

| Lane | Impact | Action |
|---|---|---|
| `dependency-direction` (LEAN-A1) | unchanged | layer rules per ADR-0105 still apply to each BC's crate fan-out |
| `cross-product` (LEAN-A2) | unchanged | foundry is one product; cross-product rule still refuses non-foundry imports |
| `foundry-bc-boundary` (NEW) | new BLOCKER | inter-BC dependency rules above |
| `ubiquitous-language-scope` (NEW, follow-up) | new REPORT-ONLY | flag cross-BC vocabulary collisions; promote to BLOCKER post-M01 |

## Verification

- [ ] All six BCs have an entry above with: purpose, owner sub-axis,
      crate fan-out, contract surface, tenant-facing flag, headline NFR.
- [ ] Cross-BC event table covers every event produced + consumed across
      BCs (the table is exhaustive per `microservices/foundry/PRD.md` +
      per-BC PRDs).
- [ ] Forbidden + permitted inter-BC dependency rules enumerated.
- [ ] BC growth policy declared (ADR amendment required).
- [ ] `oya-governance-foundry-bc-boundary` lane authored (deferred to
      follow-up IP; tracked in `microservices/foundry/IP-NNN-foundry-bc-boundary-lane.md`).

## References

- ADR-0136: Foundry as a single µservice — establishes the consolidation
  decision this ADR supports.
- ADR-0138: Foundry six-path deprecation — Strangler migration of the
  prior 6-way split.
- ADR-0022: Autonomy tiers T0–T4.
- ADR-0056: BNF v4.1 — crate-name authority.
- ADR-0105: 13-layer enum.
- ADR-0123: Hyperscaler maturity claim gate — HG-FOUNDRY surface.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- Eric Evans, *Domain-Driven Design* (Addison-Wesley, 2003) — bounded
  contexts + ubiquitous language.
- Vaughn Vernon, *Implementing Domain-Driven Design* (Addison-Wesley,
  2013) — BC integration patterns + context maps.
- AWS Bedrock product documentation — agent / knowledge-base / guardrail
  / model-catalog distinctions inside one product.
- Google Vertex AI Agent Builder — agents / tools / safety / evals /
  deploy pillars inside one product.
- Microsoft Azure AI Foundry — agents / model catalog / safety / eval /
  deploy modules inside one product.
- Anthropic Console — workbench / prompts / tools / evaluations tabs
  inside one product.
- Palantir AIP — Logic / Threads / Evals / Operator / Tools modules
  inside one product.
- `feedback_workflow_objectgraph_adapter_layer.md` — inter-BC dependency
  rule (typed events + ontology reader ports).
- `feedback_naming_justification.md` — naming-justification policy for
  per-BC crate fan-out.
- `microservices/foundry/PRD.md` — per-BC sections + cross-BC FRs.
