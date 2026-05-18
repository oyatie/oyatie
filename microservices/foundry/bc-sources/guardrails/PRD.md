---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-foundry-guardrails
microservice: foundry-guardrails
status: Accepted
sales_segment: shared-substrate
tier: internal-and-tenant-product
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs: [ADR-0022, ADR-0056, ADR-0105, ADR-0106, ADR-0110, ADR-0114, ADR-0123, ADR-0130, ADR-0131, ADR-0132, ADR-0133, ADR-0140]
related_specs: [/specs/agent-operating-contract.json, /specs/per-microservice-flat-layout.json, /specs/hyperscaler-gates.json]
date: 2026-05-17
owner_team: axis-foundry-guardrails
doc_status: published
---

# PRD-foundry-guardrails: Agent Safety + Policy Enforcement Substrate

## Purpose

The `foundry-guardrails` µservice is oyatie's substrate for **safety and policy enforcement at the agent-runtime boundary**: per-prompt classification (PII / PHI / jailbreak / prompt-injection / forbidden-topic), per-output validation (data exfiltration / unsafe completion / hallucinated tool args), autonomy-tier gating (per ADR-0022), content safety (toxicity / self-harm / sexual / violence / minors), jailbreak-pattern detection (canonicalisation-evasion / role-play wrapping / multi-turn drift), and AI-slop pattern detection (per `docs/quality/ai-slop-defense/`).

Per ADR-0131 Foundry split, `foundry-guardrails` is the **safety + policy plane** of the Foundry. It is consumed by `foundry-runtime` on every capability invocation (pre-prompt + post-output round-trip) and by every other oyatie µservice that wishes to wrap an LLM call with the same safety substrate. It is the **single, ungated chokepoint** between agent traffic and provider traffic — no foundry-runtime invocation reaches `foundry-providers` without passing through `foundry-guardrails` first, and no provider response reaches a tenant surface without re-passing through it.

This µservice is **shared substrate** (the enforcement engine; uniform every tenant + pack) and **a tenant-visible product surface** (tenants tune Cedar policies + author content-safety rule overlays + receive false-positive escalation budgets). Its existence is the precondition for oyatie's "hyperscaler-grade in every practice" bar per `feedback_quality_performance_scalability_bar.md` against AWS Bedrock Guardrails / Anthropic Constitutional AI / OpenAI Moderation / Azure AI Content Safety / Google Perspective / NVIDIA NeMo Guardrails.

This µservice has no direct Bominal equivalent and originates in oyatie per ADR-0131 Foundry split (`foundry-runtime` + `foundry-guardrails` + `foundry-providers` + `foundry-supervisor` + `foundry-evidence`).

## Tenant Value

- **Tenant Outcome 1 — Default-safe agent traffic.** Every capability invocation is classified pre-prompt and validated post-output; tenants get an enforced safety floor without authoring their own pipeline.
- **Tenant Outcome 2 — Tunable Cedar policy overlays.** Tenants compose per-pack Cedar fragments on top of the default-deny base; tenant-specific entitlements (e.g., medical-coder tenant may discuss PHI under BAA; legal-research tenant may quote case-law verbatim) are first-class.
- **Tenant Outcome 3 — Jailbreak + prompt-injection resistance.** Multi-detector ensemble (heuristic + classifier-model + LLM-as-judge for ambiguous) catches known + emerging jailbreak patterns; every Sev-1 jailbreak success is automatically post-mortem-tracked.
- **Tenant Outcome 4 — Auditable refusal trail.** Every block / redact / allow decision is signed (Ed25519, per Bominal ADR-0028) and queryable by tenant operators + auditors (SOC 2 / ISO 27001 / HIPAA / EU AI Act / KR PIPA / GDPR DPA).
- **Internal Outcome 5 — Substrate uniformity.** Every oyatie agent invocation goes through the same gate; eliminates per-product divergence in how safety is defined, enforced, or audited.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | foundry-runtime invocation orchestrator | to submit a candidate prompt + tenant context + autonomy-tier-claim | I receive an allow / block / redact verdict before dispatching to a provider | prompt-classifier | Must |
| FR-02 | foundry-runtime invocation orchestrator | to submit a candidate provider output + tenant context | I receive an allow / block / redact verdict before returning to the caller | output-validator | Must |
| FR-03 | autonomy-tier-gate | to refuse capability execution above the principal's tier ceiling (per ADR-0022) | tier escalation can never happen silently | autonomy-tier-gate | Must |
| FR-04 | content-safety-rule-engine | to evaluate prompt + output against per-pack content categories (toxicity / self-harm / sexual / violence / minors / hate / weapons / illegal) | tenant + tenant-of-tenant safety expectations are enforced | content-safety-rule-engine | Must |
| FR-05 | jailbreak-detector | to ensemble-evaluate (heuristic + classifier + LLM-as-judge) candidate prompts for known + emerging jailbreak patterns | injection / role-play / instruction-override attacks are caught pre-dispatch | jailbreak-detector | Must |
| FR-06 | ai-slop-detector | to scan candidate outputs for the AI-slop failure modes catalogued in `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md` | low-quality / hallucinated / verbose outputs are tagged + (when severe) blocked | ai-slop-detector | Must |
| FR-07 | tenant operator | to author Cedar policy overlays binding tenant-specific entitlements (e.g., medical-coder tenant authorises PHI prompt discussion under BAA) | tenant business needs override default-deny without weakening cross-tenant safety | (cross-BC) | Must |
| FR-08 | rule-store-writer | to persist rule definitions + classifier-model pointers + Cedar fragment registry in Postgres with audit-chain emission | every rule mutation is replayable + non-repudiable | (Postgres adapter) | Must |
| FR-09 | runtime-pool | to maintain a hot pool of classifier-model + Cedar engine evaluator replicas | pre-invocation classification p99 ≤ 50ms; post-output validation p99 ≤ 100ms | (pool sizing) | Must |
| FR-10 | foundry-guardrails | to refuse-with-explanation on every block decision (with `block_reason` enum + `cedar_policy_id` + `classifier_model_version` + `audit_chain_id`) | tenant developers can debug refusals without leaking sensitive policy internals | (cross-BC) | Must |
| FR-11 | tenant operator | to receive per-tenant false-positive escalation budget (tenant can mark up to N blocks/month as false-positive; triggers rule-author review without weakening enforcement) | overly-aggressive policies are surfaced operationally, not silently swallowed | (cross-BC) | Must |
| FR-12 | rule-author (axis-foundry-guardrails) | to roll out new rules in shadow-mode before enforce-mode (with per-rule shadow-vs-enforce delta dashboard) | rule regressions are caught before tenants are affected | content-safety-rule-engine | Must |
| FR-13 | classifier-model-serving | to serve in-house + adapter-mediated foundry-providers classifier models (BERT-class for fast PII/PHI; Llama-Guard-class for nuanced content-safety; LLM-as-judge for hard cases) | classification quality is competitive without per-provider lock-in | classifier-model-adapter | Must |
| FR-14 | foundry-guardrails | to emit `GuardrailDecisionEmitted` event for every allow / block / redact verdict | foundry-evidence + observability can stitch the decision timeline | (cross-BC) | Must |
| FR-15 | jailbreak-detector | to escalate any Sev-1 jailbreak success (rare: misclassified prompt that produced an unsafe output) as a Sev-1 incident with automatic post-mortem creation | systemic detector gaps are caught + remediated | jailbreak-detector | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Pre-invocation prompt classification | ≤15ms | **≤50ms** | ≤120ms | the headline; sits on every dispatch hot path |
| Post-output validation | ≤30ms | **≤100ms** | ≤250ms | runs after provider responds; before caller sees output |
| Autonomy-tier check (Cedar) | ≤2ms | ≤10ms | ≤25ms | Cedar policy evaluation; in-process |
| Content-safety rule evaluation (rule-store + Postgres cache) | ≤5ms | ≤20ms | ≤50ms | rule cache hit; full fetch on miss |
| Jailbreak-detector heuristic | ≤3ms | ≤10ms | ≤25ms | regex + ngram + canonicalisation passes |
| Jailbreak-detector classifier-model | ≤20ms | ≤80ms | ≤180ms | BERT-class; ONNX inference; in-house |
| Jailbreak-detector LLM-as-judge (ambiguous fallback) | ≤200ms | ≤800ms | ≤1500ms | invoked only when ensemble disagreement |
| AI-slop pattern scan | ≤10ms | ≤40ms | ≤100ms | heuristic + classifier hybrid |
| Guardrail decision audit-chain seal | ≤10ms | ≤30ms | ≤80ms | Ed25519 + Merkle insert |
| Cold-start budget for classifier-serving pod | — | ≤500ms | — | per ADR-0020 |
| Rule hot-reload (Postgres notify → in-pod cache) | ≤2s | ≤5s | ≤10s | propagates rule edits without restart |

### Security

- All in-cluster RPCs use mTLS via SPIFFE identity (`spiffe://oyatie/foundry-guardrails/*`); foundry-runtime is the only external caller in-cluster.
- Cedar v4 policies; default-deny enforced; tenant-overlay fragments composed under deny-overrides semantics.
- Classifier-model artifacts (ONNX / safetensors) signed via Cosign; weight tampering caught at pod-startup integrity check.
- Per-tenant rule overlays scoped via Cedar; cross-tenant rule mutation refused at API + Cedar layer.
- Secret-class material (Cosign keys, OpenBao bindings, LLM-judge provider tokens consumed via foundry-providers) follows OpenBao SecretReference pattern; raw secrets never in repo / pod env / logs.
- Rate-limiting per (tenant, decision-kind) per autonomy tier; excess returns 429 + per-tenant quota emission.
- Prompt + output payloads handled in memory only; never persisted by foundry-guardrails (foundry-evidence is the persistence authority).
- Classifier-model invocations on tenant prompt data emit `data_class=BEHAVIORAL_TENANT_PRODUCT` annotations to OTel collector.

### Audit + Compliance

- Every `GuardrailDecisionEmitted` event emits an audit-chain record (Merkle + Ed25519 per Bominal ADR-0028).
- Per-decision evidence carries: `decision_id` (ULID), `tenant_id` (hashed), `invocation_id`, `decision_kind` (allow/block/redact), `block_reason` (enum), `cedar_policy_ids[]`, `classifier_model_versions{}`, `latency_breakdown{}`, `evidence_hash`.
- Sev-1 jailbreak success events emit additionally: `prompt_hash`, `provider_output_hash`, `redaction_diff`, `incident_id` (auto-allocated).
- Audit-chain seal latency ≤30ms p99 per decision.
- Rule mutations carry GitHub PR ID + Cedar fragment SHA + author SPIFFE identity; rule-mutation audit retention ≥ 1y baseline, 6y for pack-us-healthcare HIPAA §164.316(b)(2).

### Availability + SLO

- Availability target: 99.95% monthly for `pre-invocation classification` decision path (gate decision must be available even when downstream providers are degraded; we fail-closed on classifier outage per `failure-modes.md` FM-03).
- Tenant-facing REST: 99.9% monthly.
- RTO: ≤ 15 min; RPO: ≤ 60 s for rule-store; classifier-model serving is stateless (no RPO).
- Self-SLO emitted to `observability` µservice as `oya-self` tenant; promotion gate consumes via the same model as every other µservice.

### Data residency

- Per-pack Cedar policy bundles + classifier-model artifacts + Postgres rule stores live in pack region; cross-pack replication forbidden by default per `policy/data-residency.md`. Tenant prompt + output payloads transit but are not persisted.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase`), layers used: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-cedar`, `adapter-classifier-model`, `rest`, `worker`, `sdk`, `app`. The Cedar engine adapter, Postgres rule-store adapter, and classifier-model adapter are backend-qualified per ADR-0105 §"Amendment 3" (`*-adapter-<backend>` pattern).

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `prompt-classifier` | `oya-foundry-guardrails-prompt-classifier-{kernel,domain,usecase,api,adapter,adapter-classifier-model,rest,worker,sdk,app}` | Pre-invocation classification: PII / PHI / data-class tag / sensitive-topic / hate-speech / unsafe-medical-advice / known-jailbreak | `Prompt`, `Classification`, `DataClassTag`, `ClassifierModelVersion`, `ClassifierEnsembleResult` |
| `output-validator` | `oya-foundry-guardrails-output-validator-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Post-output validation: data-exfiltration / unsafe-completion / hallucinated-tool-args / secret-leak | `ProviderOutput`, `Validation`, `RedactionDiff`, `BlockReason` |
| `autonomy-tier-gate` | `oya-foundry-guardrails-autonomy-tier-gate-{kernel,domain,usecase,api,adapter-cedar,rest,worker,sdk,app}` | Cedar policy evaluation of effective autonomy ceiling (per ADR-0022); refusal on tier excess | `AutonomyTierClaim`, `EffectiveCeiling`, `TierViolation` |
| `content-safety-rule-engine` | `oya-foundry-guardrails-content-safety-rule-engine-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}` | Per-pack content-category evaluation (toxicity / self-harm / sexual / violence / minors / hate / weapons / illegal) | `ContentCategory`, `RuleDefinition`, `RuleEvaluation`, `CategoryScore` |
| `jailbreak-detector` | `oya-foundry-guardrails-jailbreak-detector-{kernel,domain,usecase,api,adapter,adapter-classifier-model,rest,worker,sdk,app}` | Multi-detector ensemble: heuristic (regex/ngram/canonicalisation), classifier-model (BERT-class), LLM-as-judge (ambiguous fallback via foundry-providers) | `JailbreakSignal`, `EnsembleVerdict`, `DetectorVersion` |
| `ai-slop-detector` | `oya-foundry-guardrails-ai-slop-detector-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Detect AI-slop patterns per `docs/quality/ai-slop-defense/`: stub injection / over-verbose preamble / fabricated citation / shotgun pattern / verbose-without-substance | `AiSlopPattern`, `SlopScore`, `SlopRationale` |

Naming justification — `prompt-classifier`:

```
NAME: oya-foundry-guardrails-prompt-classifier-<layer>
JUSTIFICATION:
- microservice = foundry-guardrails: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. Foundry split per ADR-0131 §"Foundry surface decomposition".
- bc-tokens = prompt-classifier: primary BC for pre-invocation classification.
  ADR-0056 v4.1 BC-optionality rule honoured (multiple sibling BCs exist:
  output-validator, autonomy-tier-gate, content-safety-rule-engine, jailbreak-detector,
  ai-slop-detector), so explicit BC token is required.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (Prompt, Classification, DataClassTag,
    ClassifierModelVersion, ClassifierEnsembleResult). Zero I/O. Carries data_class
    annotations on every field (per Bominal ADR-0028 + oya-check-data-class lane).
  - domain: pure classification arithmetic (ensemble score blending, confidence
    thresholding, data-class tagging logic).
  - usecase: orchestrators reading prompt + tenant context + ensemble outputs;
    composing classification verdict + emitting events via ports.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral classifier-ensemble adapter (heuristic + dictionary
    matchers).
  - adapter-classifier-model: backend-qualified adapter (per ADR-0105 Amendment 3
    `*-adapter-<backend>` pattern); implements ClassifierModelServer trait against
    the in-cluster ONNX-runtime classifier-model-serving deployment.
  - rest: HTTP handler/route layer.
  - worker: long-lived classifier orchestrator for batch-mode (re-classify historical
    prompts on rule changes; emits backfill events).
  - sdk: client library (Rust; M01+1 TS/Python via foundry-providers SDK).
  - app: composition root.
- exemptions claimed: none. -adapter-classifier-model uses the canonical
  *-adapter-<backend> pattern.
```

Naming justification — `output-validator`:

```
NAME: oya-foundry-guardrails-output-validator-<layer>
JUSTIFICATION:
- microservice = foundry-guardrails.
- bc-tokens = output-validator: sibling BC for post-output validation.
- layer = <layer>: one crate per layer per ADR-0105.
- exemptions claimed: none.
```

Naming justification — `autonomy-tier-gate`:

```
NAME: oya-foundry-guardrails-autonomy-tier-gate-<layer>
JUSTIFICATION:
- microservice = foundry-guardrails.
- bc-tokens = autonomy-tier-gate: enforcement of ADR-0022 effective-ceiling.
- adapter-cedar: backend-qualified per ADR-0105 §"Amendment 3" — Cedar v4 is the
  policy backend; no other backend is sanctioned (per ADR-0140 Cedar selection).
```

Naming justification — `content-safety-rule-engine`:

```
NAME: oya-foundry-guardrails-content-safety-rule-engine-<layer>
JUSTIFICATION:
- microservice = foundry-guardrails.
- bc-tokens = content-safety-rule-engine: rule-driven category evaluation.
- adapter-postgres: backend-qualified per ADR-0105 §"Amendment 3" — Postgres
  is the canonical rule-store; rule mutations require relational integrity
  + per-row audit columns. No NoSQL alternative sanctioned.
```

Naming justification — `jailbreak-detector` + `ai-slop-detector`: identical pattern (multiple sibling BCs in this µservice).

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-<backend> | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|
| `prompt-classifier` | ✓ | ✓ | ✓ | ✓ | ✓ | `-adapter-classifier-model` | ✓ | ✓ | ✓ | ✓ |
| `output-validator` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ | ✓ |
| `autonomy-tier-gate` | ✓ | ✓ | ✓ | ✓ | — | `-adapter-cedar` | ✓ | ✓ | ✓ | ✓ |
| `content-safety-rule-engine` | ✓ | ✓ | ✓ | ✓ | — | `-adapter-postgres` | ✓ | ✓ | ✓ | ✓ |
| `jailbreak-detector` | ✓ | ✓ | ✓ | ✓ | ✓ | `-adapter-classifier-model` | ✓ | ✓ | ✓ | ✓ |
| `ai-slop-detector` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ | ✓ |

Total crates introduced by this µservice: **57** (10 in prompt-classifier + 9 in output-validator + 9 in autonomy-tier-gate + 9 in content-safety-rule-engine + 10 in jailbreak-detector + 9 in ai-slop-detector + 1 shared composition `-app` per BC). M01 ships a curated subset; remaining backfilled in M01+1.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `PromptClassifier` | `oya-foundry-guardrails-prompt-classifier-kernel` | `-adapter` + `-adapter-classifier-model` | `BEHAVIORAL_TENANT_PRODUCT` (prompt content); `PII_IDENTIFYING` (when detected); `INTERNAL_ONLY` (classifier scores) |
| `OutputValidator` | `oya-foundry-guardrails-output-validator-kernel` | `-adapter` | `BEHAVIORAL_TENANT_PRODUCT`; `PII_IDENTIFYING`; `SECRET` (when leak detected) |
| `AutonomyTierGate` | `oya-foundry-guardrails-autonomy-tier-gate-kernel` | `-adapter-cedar` | `INTERNAL_ONLY` (tier claim); `AUDIT` (refusal record) |
| `ContentSafetyRuleEvaluator` | `oya-foundry-guardrails-content-safety-rule-engine-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` (prompt + output text); `INTERNAL_ONLY` (rule definitions) |
| `JailbreakDetectorEnsemble` | `oya-foundry-guardrails-jailbreak-detector-kernel` | `-adapter` (heuristic) + `-adapter-classifier-model` (classifier + LLM-judge) | `BEHAVIORAL_TENANT_PRODUCT` + `INTERNAL_ONLY` |
| `AiSlopDetector` | `oya-foundry-guardrails-ai-slop-detector-kernel` | `-adapter` | `BEHAVIORAL_TENANT_PRODUCT` (output text); `INTERNAL_ONLY` (slop scores) |
| `RuleStore` | `oya-foundry-guardrails-content-safety-rule-engine-kernel` | `-adapter-postgres` | `INTERNAL_ONLY` (rule defs); `AUDIT` (mutation history) |
| `CedarEngineHandle` | `oya-foundry-guardrails-autonomy-tier-gate-kernel` | `-adapter-cedar` | `INTERNAL_ONLY` (policy text); `AUDIT` (decision record) |
| `ClassifierModelServer` | `oya-foundry-guardrails-prompt-classifier-kernel` (re-exported by jailbreak-detector kernel) | `-adapter-classifier-model` (ONNX runtime client) | `INTERNAL_ONLY` (model artifact hash); `BEHAVIORAL_TENANT_PRODUCT` (inference input) |
| `GuardrailDecisionEmitter` | `oya-foundry-guardrails-prompt-classifier-kernel` | `-adapter` (AsyncAPI publisher → foundry-evidence) | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time per `feedback_clean_architecture_requirements.md`.

Cross-product rule: `foundry-guardrails` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow events (`GuardrailDecisionEmitted`, `JailbreakDetected`, `AutonomyTierViolation`, `RuleStoreMutated`) or Ontology reads/writes. The LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice foundry-guardrails` — dependency-direction
- `oya gate validate lean-a2 --microservice foundry-guardrails` — cross-product-refusal
- `oya gate validate port-location --microservice foundry-guardrails` — ports in kernel
- `oya gate validate layer-correctness --microservice foundry-guardrails` — layer enum match
- `oya gate validate per-microservice-layout --microservice foundry-guardrails` — ADR-0131
- `oya gate validate statelessness --microservice foundry-guardrails` — classifier replicas stateless
- `oya gate validate shardability --microservice foundry-guardrails` — partition by tenant
- `oya gate validate cedar-fragment-coverage --microservice foundry-guardrails` — Cedar v4 + default-deny + per-tenant
- `oya gate validate data-class --microservice foundry-guardrails`
- `oya gate validate authority-cohesion` — HG-FGUARD registers here

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `GuardrailDecisionEmitted` | Every allow/block/redact verdict | `foundry-evidence`, `observability`, `audit-chain` | per-decision linear |
| `JailbreakDetected` | Detector ensemble verdict ≥ block threshold | `foundry-supervisor`, `audit-chain`, `grafana-oncall` (Sev-1 page) | incident lifecycle |
| `AutonomyTierViolation` | Effective-ceiling refusal at gate | `foundry-supervisor`, `audit-chain`, `tenancy` (tenant notification) | tier-violation review |
| `ContentSafetyRuleFired` | Rule match above threshold | `foundry-evidence`, `audit-chain` | per-decision linear |
| `RuleStoreMutated` | Rule definition created / modified / sunsetted | `audit-chain`, `governance`, `foundry-evidence` | append-only |
| `ClassifierModelDeployed` | New classifier-model rolled out (shadow → enforce) | `audit-chain`, `observability` | shadow-vs-enforce review |
| `FalsePositiveEscalated` | Tenant operator marks a block as false-positive within budget | rule-author queue (axis-foundry-guardrails) | review-and-retune |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `CapabilityInvocationStarted` | `foundry-runtime` | `prompt-classifier` | classify pre-invocation prompt |
| `CapabilityInvocationOutputReady` | `foundry-runtime` | `output-validator` | validate provider output |
| `TenantOnboarded` | `tenancy` | `content-safety-rule-engine` | seed default per-pack rules for new tenant |
| `TenantPackChanged` | `tenancy` | `content-safety-rule-engine` | re-evaluate per-pack rule overlay (e.g., add HIPAA pack rules) |
| `AutonomyPolicyUpdated` | `foundry-supervisor` | `autonomy-tier-gate` | hot-reload Cedar fragments |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `GuardrailDecision{decision_id, invocation_id, tenant, decision_kind, block_reason}` | `decision_for→Invocation` | `prompt-classifier` + `output-validator` | Ed25519 |
| `RuleDefinition{rule_id, category, version, status, author}` | `rule_for→Tenant` | `content-safety-rule-engine` | Ed25519 |
| `ClassifierModelVersion{model_id, version, sha, status, evaluated_at}` | `model_for→Detector` | (operational metadata) | Ed25519 |
| `JailbreakIncident{incident_id, invocation_id, severity, root_cause}` | `incident_for→Invocation` | `jailbreak-detector` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Tenant` (entitlements + pack) | every BC | `get_by_hashed_id(tenant_id)` |
| `Capability` (declared autonomy tier) | `autonomy-tier-gate` | `get_by_id(capability_id)` |
| `Pack` (jurisdiction + active rules) | `content-safety-rule-engine` | `get_by_pack_id(pack_id)` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| AWS | Bedrock Guardrails | Per-policy denied-topic + word filters + sensitive-info filter + content filter (hate/insults/sexual/violence/misconduct/prompt-attack) + contextual grounding | `docs.aws.amazon.com/bedrock/latest/userguide/guardrails.html` |
| Anthropic | Constitutional AI + Claude's built-in safety | Constitutional principles applied at training; runtime input/output classifiers (`prompt_injection`, `harmful` content) | `anthropic.com/research/constitutional-ai-harmlessness-from-ai-feedback` |
| OpenAI | OpenAI Moderation API | Multi-label categories (hate/harassment/self-harm/sexual/violence + subcategories); free-of-charge moderation | `platform.openai.com/docs/guides/moderation` |
| Microsoft | Azure AI Content Safety | Text + image moderation (hate/sexual/violence/self-harm); prompt-shield (jailbreak detection); protected-material detection | `learn.microsoft.com/azure/ai-services/content-safety/` |
| Google | Perspective API + Vertex AI Safety | Toxicity / severe-toxicity / identity-attack / insult / profanity / threat scoring; Vertex content-safety filters | `developers.perspectiveapi.com` + `cloud.google.com/vertex-ai/generative-ai/docs/multimodal/configure-safety-attributes` |
| NVIDIA | NeMo Guardrails | Programmable Colang flow-based guardrails; topical rails / dialog rails / fact-checking rails | `github.com/NVIDIA/NeMo-Guardrails` |
| Meta | Llama Guard / Prompt Guard | Open-weight classifier models; multi-category prompt + response safety scoring | `ai.meta.com/research/publications/llama-guard-llm-based-input-output-safeguard-for-human-ai-conversations/` |

Key parity gaps to close (ordered by priority):

1. **Per-tenant Cedar overlay** — competitors are largely flat policy; oyatie's tenant-overlay-on-default-deny model is differentiator. Target: every BC supports per-tenant Cedar entitlement composition.
2. **Audit-chain-grade evidence** — competitors emit log lines; oyatie emits Ed25519-signed decision records linkable to PromotionEligibilityVerdict. Target: cryptographic non-repudiation by default.
3. **Multi-detector ensemble** — competitors typically expose one classifier per category; oyatie ensembles heuristic + classifier + LLM-judge for hard cases with explicit cost budget. Target: ensemble-by-default for high-risk categories.
4. **In-house classifier-model serving** — competitors are vendor-bound; oyatie ships in-house ONNX-runtime classifier-model-serving so providers are commodity. Target: every category has at least one in-house classifier with competitor-API as adapter, never the only option.
5. **AI-slop pattern coverage** — competitors do not catalogue AI-slop. Oyatie's `docs/quality/ai-slop-defense/` catalogue is unique. Target: maintain 100% catalogue coverage as a CI-tracked metric.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Pre-invocation classification | ≤15ms | ≤50ms | ≤120ms | headline |
| Post-output validation | ≤30ms | ≤100ms | ≤250ms | headline |
| Autonomy-tier check | ≤2ms | ≤10ms | ≤25ms | Cedar in-process |
| Jailbreak ensemble | ≤25ms | ≤90ms | ≤200ms | heuristic + classifier |
| LLM-as-judge fallback | ≤200ms | ≤800ms | ≤1500ms | invoked < 5% requests |

Error budget:
- Monthly error budget for `pre-invocation` SLI: 0.05% (≈22 min/month).
- Burn-rate alarm: 14.4× burn over 1h triggers Sev-1 page.
- Error budget policy: `microservices/foundry-guardrails/slos/error-budget-policy.md`.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `stateless | postgres | object-storage | persistent-volume | mixed` → **`mixed`**. Rationale: classifier-model-serving pods are stateless (ONNX artifact loaded at startup; no per-request state); Postgres holds rule definitions + per-tenant Cedar overlay registry + audit-chain mutation log; classifier-model artifacts in object storage (S3-compatible, per pack) signed with Cosign; Cedar engine policies materialised in-process from Postgres on startup.

**Active-active compatibility**: `stateless-compatible` for classifier serving; Postgres is HA-primary with sync replication to DR-pair within pack; Cedar engine is purely in-process (no shared state).

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Classify QPS per classifier-serving pod | 1000 | 5000 | CPU > 70% OR p99 latency > 40ms |
| Postgres rule store QPS | 5000 | 50000 | pg connection pool > 80% |
| Cedar engine evaluations / pod / s | 5000 | 50000 | CPU > 70% |
| Jailbreak LLM-judge fallback budget / hour / tenant | 100 | 500 | budget-exhaust returns 429 |

Scale-out policy:
- Kubernetes HPA: classifier-serving scales on CPU + p99 latency; min 4 replicas / pack, max 200 replicas / pack.
- Postgres: per-pack HA primary + read replicas (writes through primary; reads from replicas with stale-read tolerance < 1s).
- Pre-warmed pool: 2 standby classifier-serving pods per model; cold-start budget ≤ 500ms per ADR-0020.

Cross-region story:
- M01 launch: single pack-kr region (OCI ap-seoul-1); per-tenant residency locked per ADR-0117 + `policy/data-residency.md`.
- Post-M01: per-pack Postgres + classifier-serving; no cross-pack replication of decisions, rules, or models.

Sharding:
- Rule store partitions by `(pack, tenant, category)`; per-tenant queries fully bounded.
- Classifier-serving pods are stateless; shard by request affinity (round-robin) within pack.
- `oya-check-shardability-cli` CI lane verifies partition key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | `cargo run -p oya-foundry-guardrails-prompt-classifier-app -- classify <prompt-fixture>` returns a verdict matching golden fixture | unit + integration tests under `tests/` |
| AC-02 | Pre-invocation classification p99 ≤ 50ms on reference workload | load test under `tests/load/` |
| AC-03 | Post-output validation p99 ≤ 100ms on reference workload | load test |
| AC-04 | Autonomy-tier excess refusal verified end-to-end via foundry-runtime invocation | e2e drill |
| AC-05 | Cedar policy bundle validates against Cedar v4 schema; default-deny enforced | `cargo run -p oya-foundry-guardrails-autonomy-tier-gate-app -- validate-cedar` exit 0 |
| AC-06 | Jailbreak ensemble catches all golden-fixture jailbreak prompts (5+ open + 5+ proprietary patterns) | `tests/jailbreak/golden_fixtures.rs` |
| AC-07 | AI-slop detector flags 100% of catalogue patterns from `docs/quality/ai-slop-defense/` | `tests/aislop/catalogue.rs` |
| AC-08 | Helm charts deploy clean against kind cluster | CI lane `oya-foundry-guardrails-iac-smoke` |
| AC-09 | `gate validate per-microservice-layout --microservice foundry-guardrails` exit 0 | ADR-0131 lane |
| AC-10 | `gate validate authority-cohesion` exit 0; HG-FGUARD registered green | ADR-0123 lane |
| AC-11 | Sev-1 jailbreak success drill: synthetic prompt that passes classifier; verify post-mortem auto-created | scripted e2e drill |
| AC-12 | False-positive escalation budget honoured per tenant; budget-exceeded returns clear UX | tenant-facing e2e |
| AC-13 | Shadow-mode rule rollout: deploy rule; verify shadow decisions emitted for 7d; verify enforce-promote requires shadow-vs-enforce-delta review | rule-author e2e |
| AC-14 | DSR cascade: classifier-model invocation history (held only in observability, not foundry-guardrails) is erased per tenant request | DSR drill |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | LLM-as-judge fallback: does foundry-providers expose a dedicated low-cost / low-latency endpoint, or does guardrails own its own per-pack low-cost model? | axis-foundry-guardrails + axis-foundry-providers | successor-IP ADR |
| 2 | Shadow-mode duration default (7d? 14d?) for new rules | axis-foundry-guardrails | resolved in `policy/guardrail-enforcement.md` (Slice D) |
| 3 | False-positive escalation budget per tier (trial vs production tenant) | axis-foundry-guardrails + gtm | successor-IP ADR |
| 4 | In-house classifier model versions: pinned LTS vs auto-rollout? Default: pinned + ADR-gated upgrade | axis-foundry-guardrails | resolved in `runbooks/classifier-model-rollback.md` |
| 5 | Does foundry-guardrails maintain its own audit-chain seal or share foundry-evidence's chain? Default: shared chain for cost; per-µservice signature for non-repudiation | council-architecture | successor-IP ADR |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0022 | Autonomy ceiling runtime enforcement | this PRD enforces |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application → usecase | usecase layer naming |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0114 | Canary observability rollback | shadow-vs-enforce rule rollout precedent |
| ADR-0123 | Hyperscaler maturity claim gate | HG-FGUARD registers here |
| ADR-0130 | Agentic SLO-gated promotion | self-SLOs gate guardrails' own promotion |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it; Foundry split |
| ADR-0132 | Product-suite + bundle dissolution | guardrails ships as substrate, not a suite |
| ADR-0133 | Industry-best-practice conformance program | competitor parity matrix mandate |
| ADR-0140 | Cedar policy substrate | Cedar v4 is the autonomy-tier + tenant-overlay engine |

## References

- `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md`
- `docs/standards/observability-slo.md`
- `docs/standards/agentic-dev-team-optimization.md`
- `/specs/agent-operating-contract.json`
- `microservices/foundry-runtime/PRD.md` (the consumer)
- `microservices/foundry-providers/PRD.md` (the downstream gated by guardrails)
- `microservices/foundry-supervisor/PRD.md` (autonomy-policy source)
- `microservices/observability/PRD.md` (the self-SLO substrate)
- AWS Bedrock Guardrails docs (parity)
- Anthropic Constitutional AI (parity)
- OpenAI Moderation API (parity)
- Microsoft Azure AI Content Safety (parity)
- Google Perspective + Vertex AI Safety (parity)
- NVIDIA NeMo Guardrails (parity)
- Meta Llama Guard / Prompt Guard (parity)
