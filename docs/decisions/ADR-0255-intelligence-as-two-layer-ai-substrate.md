---
id: ADR-0255
status: Accepted
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - council-legal
  - ops-compliance
  - ops-sre-reliability
  - axis-intelligence
  - axis-foundry
  - axis-policy-engine
  - axis-tenancy
  - axis-audit-chain
supersedes: []
amends:
  - ADR-0220-consumer-intelligence-substrate.md
superseded_by: []
amended_by: [ADR-0329, ADR-0335]
related:
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0050-event-bus-kafka.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0136-intelligence-as-single-microservice.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-tag-sustainability.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0188-passkey-webauthn.md
  - ADR-0192-milvus-vector-substrate.md
  - ADR-0200-wasmtime-plugin-runtime.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0213-ecosystem-as-a-service-architecture.md
  - ADR-0215-multi-context-platform.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0219-no-code-builder-suite.md
  - ADR-0220-consumer-intelligence-substrate.md
  - ADR-0221-agentic-development-pipeline-hardening.md
  - ADR-0239-amendment-intelligence-internal-scope-clarification-2026-05-18.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/intelligence.json
  - /specs/microservices/embeddings.json
  - /specs/microservices/fine-tuning.json
  - /specs/byok-credential-model.json
  - /specs/multi-modal-transport.json
  - /specs/tool-call-protocol.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_bominal_inheritance_precedence
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_flat_product_catalog
  - feedback_workflow_objectgraph_adapter_layer
  - feedback_workflow_is_shared
  - feedback_glossary_ontology_not_object_graph
  - feedback_autonomous_decision_principles
  - feedback_quality_performance_scalability_bar
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 14-of-14
purpose: >
  Rewrite ADR-0220's framing of Intelligence as a consumer-only
  µservice with model-routing baked in as one of its responsibilities.
  Establish Intelligence as a two-layer µservice: (a) an audience-
  neutral AI Substrate layer that serves every tenant — including
  `oyatie` itself — for any LLM, embedding, multi-modal, tool-calling,
  guardrail, eval, or attribution decision; and (b) a Consumer Brand
  Surface layer scoped to B2B-tenant and B2C-consumer audiences that
  renders the "oyatie intelligence" brand UX. Establish opt-in
  provider-BYOK as the canonical LLM/provider credential model
  (Anthropic / OpenAI / Google / Bedrock / self-hosted vLLM / etc.):
  in B2B-regulated and provider-BYOK-elected paths the substrate owns zero
  provider credentials, while consumer (B2C) surfaces fall back to
  platform-default provider credentials owned by the `oyatie` tenant
  when a tenant has not opted in to provider-BYOK; every provider
  credential — default or tenant-owned — is a SecretReference with an
  explicit owner. encryption-BYOK (tenant KMS / HSM root) is a
  separate concern, owned by ADR-0251 §D-10. Establish multi-modal transport (text, image, audio, video,
  code) as day-one. Establish stateless dispatch composed with Workflow
  durability. Establish caller-side RAG. Promote `microservices/embeddings/`
  and `microservices/fine-tuning/` as separate substrate µservices.
  Absorb Foundry's `providers`, `guardrails`, and `eval` bounded
  contexts into Intelligence. Deploy per-cell.
enforcement_status: advisory-until-intelligence-rewrite-lands
enforced_by:
  - oya gate validate intelligence-two-layer-coherence
  - oya gate validate byok-everywhere-coherence
  - oya gate validate no-credentials-in-substrate
  - oya gate validate multi-modal-transport-coverage
  - oya gate validate caller-side-rag-only
  - oya gate validate audience-tag-on-every-call
  - oya gate validate foundry-bc-absorption-complete
---

# ADR-0255: Intelligence as Two-Layer AI Substrate

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive). This is keystone **#14 of 14** — the final
ADR in the foundational doctrine bundle landing 2026-05-20. The
keystone bundle is multispectrum-reviewed and lands as a single PR;
partial acceptance is rejected because the doctrines are mutually
reinforcing.

Enforcement is `advisory-until-intelligence-rewrite-lands`. CI lanes
that enforce this ADR promote to BLOCKER once:

1. `microservices/intelligence/` is restructured into the two-layer
   bounded-context shape per §D-1 (8 AI Substrate BCs + 6 Consumer
   Brand Surface BCs).
2. `microservices/embeddings/` and `microservices/fine-tuning/` are
   scaffolded as peer substrate µservices per §D-8 and §D-9.
3. Foundry's `providers`, `guardrails`, and `eval` BCs are migrated
   into Intelligence per §D-16 (correlated with ADR-0249 Foundry
   dissolution; this ADR's BC absorption is the Intelligence side of
   that dissolution).
4. The `secret_references` table schema per §D-4 is deployed to
   `microservices/cloud-secrets/` and every Intelligence transport
   call routes credentials through `oya-shared-secret-reference`.
5. Multi-modal transport BCs (text, image, audio, video, code) per
   §D-5 each have at least one provider adapter exercised by
   integration tests.
6. Caller-side RAG (no Intelligence-side retrieval) is verified by
   `oya-check-caller-side-rag-only` static analysis lane.
7. Every Intelligence call carries an `audience` tag in its Cedar
   context per §D-15; verified by `oya-check-audience-tag-on-every-call`.

Until those seven items land, validators emit findings without
failing CI. Post-bootstrap, the lanes promote to BLOCKER.

ADR-0220 is **substantially rewritten** by this ADR — frontmatter on
ADR-0220 carries `superseded_by: [ADR-0255]` for its audience-as-
µservice-scope framing and its model-routing-as-Intelligence-
responsibility framing. ADR-0220's establishment of the substrate
itself (the µservice exists; the brand label is "oyatie intelligence";
in-house Class C per ADR-0211) is **preserved**.

## Date

2026-05-20.

## Context

### What ADR-0220 said (and where it broke)

ADR-0220 (Consumer Intelligence Substrate, 2026-05-18) was authored
in the same session as the ADR-0136 amendment that scoped Foundry to
"internal only." ADR-0220 framed Intelligence as a *consumer-only*
µservice serving B2B tenants and B2C personal users, with explicit
rejection of "Alternative 2: One AI gateway for internal and consumer
users" on the grounds that "gateway unification hides audience
differences."

ADR-0220 made Intelligence responsible for, among other things,
**model routing** (provider, model, capability tier, data class,
region); **cost attribution** (per tenant and per product); and
**EU AI Act classification** tracking (per capability tier).
ADR-0220 also published a shared-substrate table where Intelligence
and Foundry share Milvus, Wasmtime, Cedar, and audit chain "where
isolation is explicit."

Three structural defects in that framing have surfaced in the twelve
days since:

1. **Audience-as-µservice-scope is retired by ADR-0242.** ADR-0242
   (`oyatie`-is-a-tenant doctrine, keystone #1) eliminates the
   internal-vs-consumer µservice distinction. `oyatie` is a tenant;
   `oyatie.foundry.ci-agent` is a principal under that tenant; LLM
   calls made by that principal go through the same substrate as LLM
   calls made by `tenant-customer-xyz.user-7421`. The "Intelligence
   is consumer-only" framing no longer matches the platform shape.
2. **Model-routing as Intelligence's responsibility duplicates
   policy.** Per ADR-0243 (Cedar as universal gate, keystone #2),
   every policy-class decision — including model selection by data
   class, jurisdiction, and tier — is a Cedar evaluation. Embedding
   model routing inside Intelligence as code would re-fork the policy
   engine. The actual responsibility surface is: Intelligence
   *executes* the dispatch decision; Cedar *decides* it.
3. **Foundry-vs-Intelligence shared-substrate table is the wrong
   axis.** Per ADR-0249 (Foundry dissolution, keystone #13), Foundry
   dissolves into the µservice fabric. Foundry's `providers` BC
   (model provider adapters), `guardrails` BC (prompt-injection +
   PII redaction), and `eval` BC (multispectrum review runners +
   golden-set evaluations) are not Foundry-specific — they're
   *Intelligence* responsibilities. Co-locating them in Foundry
   doubled the provider-adapter surface and the guardrail authoring
   workload.

### What ADR-0247 and ADR-0249 imply for Intelligence

ADR-0247 (self-hosting / self-modification doctrine, keystone #6)
establishes that oyatie modifies oyatie — including AI-mediated code
changes — under the same Cedar gates as customer-tenant operations.
The autonomous-masterplan workflows (`oyatie.foundry.adr-drafter`,
`oyatie.foundry.eval-runner`, `oyatie.foundry.merge-queue`, etc. —
naming preserved operationally; principals scoped per ADR-0242)
make LLM calls during planning, drafting, reviewing, and verifying.
Those LLM calls **must** go through the same substrate as customer-
tenant LLM calls. There is no architectural argument for two
substrates.

ADR-0249 (Foundry dissolution, keystone #13) terminates
`microservices/foundry/` as a single µservice. Foundry's bounded
contexts redistribute. This ADR (keystone #14) decides where the
AI-relevant BCs land — and the answer is: in Intelligence.

### What the audience question actually is

The audience-as-µservice-scope framing in ADR-0220 was the wrong
axis. The right axis (per ADR-0242) is:

- **Tenant** is the scoping primitive (every call carries a tenant).
- **Principal** is the actor (every call carries a principal under a
  tenant).
- **Audience** is a *call tag*, not a µservice property. The tag
  determines: which guardrails fire, which tools are available,
  which audit stream is the primary recipient, which cost center
  bills, and which Cedar fragments compose.

The audience tag is one of:

| Audience tag | Caller pattern | Brand surface |
|---|---|---|
| `internal-platform-ops` | `oyatie.*` principals (excl. consumer-facing oyatie products) | No consumer surface |
| `b2b-tenant-product` | `tenant-<id>.*` principals using oyatie products (Workflow Studio, Mail, HR, etc.) | oyatie intelligence brand |
| `b2c-consumer` | `tenant-<id>.consumer.*` personal-user principals | oyatie intelligence brand |
| `developer-platform` | Plugin / Marketplace / external-developer principals | Optional brand surface |
| `oyatie-self-modification` | `oyatie.foundry.*` autonomous workflow principals | No consumer surface; full audit |

The audience tag is set by the caller (validated by Cedar) and
travels through the substrate as a Cedar `context` attribute. The
substrate routes guardrails, tool registry filtering, audit stream
selection, and cost attribution off of it.

### What "two-layer" means concretely

Intelligence becomes one µservice (`microservices/intelligence/`)
with two clearly-separated layers expressed as bounded contexts:

- **Layer A — AI Substrate (audience-neutral, 8 BCs).** Serves
  every tenant. Lives in every Tier 3 data-plane cell. Used by
  every µservice that makes an LLM, multi-modal, or tool call.
  Used by `oyatie.foundry.*` workflows for self-modification.
- **Layer B — Consumer Brand Surface (consumer-scoped, 6 BCs).**
  Renders the "oyatie intelligence" brand. Adds prompt-history,
  consent-cascade, DSAR-cascade, EU-AI-Act-tier-UI,
  tenant-admin-console-controls, and brand-UX-surface BCs atop the
  substrate. Only consumer-audience tenants enable this layer.

Both layers ship from the same µservice path
(`microservices/intelligence/`) because their lifecycles are tightly
coupled (the Brand Surface depends on Substrate APIs that move
together) and their deployment cadence is identical. They are
*architecturally* separated (separate BCs, separate Cedar fragments,
separate audit streams), not *physically* separated into different
µservices.

### Why now (2026-05-20)

Three forcing functions:

- **ADR-0242 + ADR-0243 + ADR-0247 + ADR-0249 land together.** Each
  is a keystone in the 2026-05-20 bundle. Each implies that
  Intelligence's role changes. Authoring those four without
  simultaneously rewriting ADR-0220 produces drift within the same
  bundle.
- **The autonomous-masterplan goal (feedback_autonomous_implementation_artifacts).**
  Achieving "Implement the masterplan runs without user intervention"
  requires that `oyatie.foundry.*` workflows make LLM calls through
  the same substrate as customer-tenant workflows. Without this ADR
  the substrate would be consumer-only and the foundry workflows
  would need a parallel path.
- **Frontier multi-modal model maturity (2024-2026 capability
  curve).** GPT-4o (2024-05), Claude 3.5 Sonnet vision + tool use
  (2024-06), Gemini 1.5 Pro multi-modal (2024-02), GPT-5 (2025),
  Claude Opus 4 (2025), Llama 4 multi-modal open weights (2025),
  Sora video generation (2024-2025), and the broader multi-modal
  toolchain require day-one multi-modal transport. Designing
  Intelligence as text-only and bolting modalities on later is the
  exact anti-pattern that produced Apple Intelligence's late image-
  generation surface scramble (2024).

### What hyperscaler references actually do

Three named references at the 2026 capability bar all converge on
the same shape:

- **AWS Bedrock (2024 GA evolution).** Bedrock is the substrate
  layer; it serves any AWS workload (internal + external) and is
  consumed by AWS internal teams under their AWS-tenant principals
  the same way external customers consume it. AWS Bedrock Guardrails
  (2024-Q2 GA) is a separate-but-co-located guardrails layer; it is
  audience-aware via per-Guardrail-policy attachment. Bedrock
  integrates with Step Functions (Workflows) for durable multi-step
  flows. Knowledge Bases (Bedrock RAG) is opt-in caller-side. Per
  the AWS Bedrock product page 2024 + "Building Generative AI
  Applications with Bedrock and Step Functions" AWS blog 2024.
- **Azure AI Foundry (2024-Q4 rebrand from Azure AI Studio).** One
  substrate for foundation models (Azure OpenAI, Llama, Mistral,
  custom). Audience separation is achieved by per-deployment
  endpoint scoping + Entra ID tenant boundary, not by separate
  Foundry/Studio µservices. Multi-modal day-one (text, image,
  audio, video). encryption-BYOK via Azure Key Vault references. Per Azure
  AI Foundry product page (microsoft.com/azure/ai-foundry) 2024.
- **Apple Intelligence (WWDC 2024).** Apple's on-device + private-
  cloud-compute Intelligence is one substrate consumed by every
  Apple product. Audience separation is at the surface (different
  app UIs render the brand differently), not at the substrate.
  Apple's "Brand Surface" pattern (consumer-only sparkle UI;
  developer-only foundation-models-API surface; internal-only tools
  surface) layers atop one substrate. Per Apple WWDC 2024 keynote
  + "Apple Intelligence Foundation Language Models" technical
  report 2024.

The pattern is unambiguous: **one substrate, layered brand
surfaces, audience-aware policy at the call boundary.**

### The 23-policy-class debt and Intelligence

Per ADR-0243 §Context, the platform's prior shape encoded 23
policy-class decisions as imperative code or static configuration.
At least six of those 23 were Intelligence-relevant:

1. Provider routing (data class → LLM provider).
2. Tool-call permits (LLM → tool capability allowance).
3. Multi-modal modality permits (which modalities for which tenants).
4. Per-call cost-attribution selection.
5. Audit-stream selection (which stream the eval row emits to).
6. EU AI Act tier classification at call time.

This ADR ensures all six are Cedar evaluations (per ADR-0243),
*not* Intelligence-internal code paths. Intelligence executes the
dispatch; Cedar decides.

## Decision

### D-1. Two-layer model — AI Substrate + Consumer Brand Surface

`microservices/intelligence/` is restructured as a single µservice
containing two clearly-separated layers expressed as bounded
contexts:

**Layer A — AI Substrate.** Audience-neutral. Serves every tenant.
Eight BCs (per D-2). Deployed per Tier 3 data-plane cell. Provides
the universal dispatch + credential-resolution + guardrails +
audit-emit + tool-registry + audience-policy-routing + cost-
attribution machinery. Has zero consumer-specific concerns.

**Layer B — Consumer Brand Surface.** Audience-scoped to
`b2b-tenant-product` and `b2c-consumer` calls. Six BCs (per D-3).
Renders the "oyatie intelligence" brand. Provides prompt history,
consent cascades, DSAR cascades, EU AI Act tier UI, tenant-admin-
console controls, and brand UX surface. Sits *atop* the AI
Substrate; calls flow Substrate-first, then Brand-Surface decorates
the consumer-audience subset.

Both layers ship from the same Cargo workspace
(`microservices/intelligence/`) but are distinct Cargo crates per
BC. Each BC has its own contracts (OpenAPI 3.2.0 + AsyncAPI 3.1.0
+ proto), its own Cedar fragments, its own audit-event classes.

Layer interaction:

```text
                              ┌──────────────────────────────────────────┐
                              │           microservices/intelligence/      │
                              │                                            │
   Caller (any µservice) ──→  │  ┌────────────────────────────────────┐  │
   carries:                   │  │  Layer B — Consumer Brand Surface   │  │  ──→ Consumer
   • principal_id             │  │  (only invoked when audience tag ∈   │      tenant
   • tenant_id                │  │  {b2b-tenant-product, b2c-consumer}) │      brand UX
   • audience tag             │  └──────────────┬─────────────────────┘  │
   • request payload          │                  │                         │
                              │                  ▼                         │
                              │  ┌────────────────────────────────────┐  │
                              │  │  Layer A — AI Substrate              │  │  ──→ External
                              │  │  (always invoked, audience-neutral)  │      provider
                              │  └────────────────────────────────────┘  │      OR own-
                              │                                            │      hosted model
                              └──────────────────────────────────────────┘      OR ontology
                                                                                tool call
```

Calls with audience tag `internal-platform-ops` or
`oyatie-self-modification` or `developer-platform` skip Layer B and
go directly to Layer A. Calls with audience tag `b2b-tenant-product`
or `b2c-consumer` enter Layer B first; Layer B records
consent/history/DSAR-eligibility/tier-classification then delegates
to Layer A.

### D-2. AI Substrate BCs (8, audience-neutral)

The AI Substrate layer contains eight bounded contexts. Each is a
separate Cargo crate under `microservices/intelligence/crates/`:

| BC | Crate | Responsibility |
|---|---|---|
| `transport` | `oya-intelligence-transport` | Provider-agnostic LLM + multi-modal call dispatch. Maintains provider adapters for OpenAI, Anthropic, Google, Mistral, Cohere, AWS Bedrock, Azure OpenAI, vLLM-self-hosted, SGLang-self-hosted, TensorRT-LLM-self-hosted, Apple Foundation Models API, OpenRouter, Together, Groq, plus any future adapter. Normalizes request/response across providers. Streaming (SSE) + non-streaming. Multi-modal (text, image, audio, video, code). Per-modality sub-adapter. |
| `credential-resolver` | `oya-intelligence-credential-resolver` | Resolves SecretReferences (per D-4) at call time into ephemeral credentials. Honors owner declarations (oyatie's subscription, oyatie's API key, tenant's subscription, tenant's provider-BYOK API key). Never materializes credentials at rest in Intelligence; always sources from `microservices/cloud-secrets/` (OpenBao) at call. |
| `policy-engine-client` | `oya-intelligence-policy-engine-client` | Wraps `oya-shared-policy-engine-client` (per ADR-0243). Builds the Cedar evaluation request for each Intelligence call (principal, action ∈ {LlmDispatch, EmbeddingGenerate, ImageGenerate, AudioTranscribe, ToolCall, ...}, resource ∈ Provider/Model/Modality/Tool, context including audience tag, data class, jurisdiction, cell, modality). Receives `Permit | Forbid | NotApplicable`. Returns determining-policy list for audit. |
| `guardrails` | `oya-intelligence-guardrails` | Pre-call + post-call content guardrails. Pre-call: prompt-injection detection (LLM-as-judge + heuristic), PII redaction (per data class), prompt-policy-conformance check (jailbreak prompts blocked), toxic-content refusal. Post-call: PII leak detection in response, jailbreak success detection, toxic-content refusal. Per-audience configuration (consumer audience stricter than internal-platform-ops). |
| `audit-emit` | `oya-intelligence-audit-emit` | Emits per-call audit rows to the audit-chain substrate. Per-call rows: `IntelligenceDispatch`, `IntelligenceToolCall`, `IntelligenceGuardrailFired`, `IntelligenceCredentialResolved`, `IntelligenceCostAttributed`. Selects audit stream per audience tag (e.g., `oyatie.foundry` for internal-platform-ops; tenant's primary stream for b2b-tenant-product). |
| `tool-registry` | `oya-intelligence-tool-registry` | Registry of callable tools (Ontology-defined Functions, MCP servers per Anthropic Model Context Protocol 2024, internal capabilities). Per-audience filtering (which tools are available to which audience). Tool metadata: name, schema, autonomy_tier, data classes touched, Cedar fragment governing invocation, MCP-server reference if external. The actual tool-call ingress lives in Ontology's `tool-call-ingress` BC (per D-12); Intelligence's `tool-registry` is a *discovery* surface, not the inbound side. |
| `audience-policy-router` | `oya-intelligence-audience-policy-router` | Routes the audience tag through the call. Does NOT decide audience (audience is a call tag set by the caller and validated by Cedar). Does select which guardrail bundle, which audit stream, which cost center, which tool subset, which provider subset apply per audience. All routing decisions delegate to Cedar fragments. |
| `cost-attribution` | `oya-intelligence-cost-attribution` | Per-call cost computation + attribution to the correct cost center. Inputs: provider rate card (per token, per image, per second of audio, per second of video), modality, tokens-in, tokens-out, image-count, audio-seconds, video-seconds, cache-hit fraction. Outputs: per-call cost row emitted to FinOps portal under the deepest declared sub-scope (per ADR-0242 §D-7). |

These 8 BCs constitute the audience-neutral AI Substrate. Every
Intelligence call traverses them in order. None of them carries
consumer-specific UX logic.

### D-3. Consumer Brand Surface BCs (6, consumer-only)

The Consumer Brand Surface layer contains six bounded contexts.
Each is a separate Cargo crate under `microservices/intelligence/crates/`:

| BC | Crate | Responsibility |
|---|---|---|
| `prompt-history` | `oya-intelligence-prompt-history` | Per-user persistent prompt history. Append-only writes; per-tenant retention policy (Cedar-governed). Searchable. Subject to DSAR cascade. Consumer-audience only — internal-platform-ops + oyatie-self-modification calls do NOT write here (those go to the audit-chain only, per the lighter-weight evidence pattern for self-modification workflows). |
| `consent-cascade` | `oya-intelligence-consent-cascade` | Per-tenant + per-user consent state (training-use opt-in/out, retention preferences, third-party provider opt-in, sub-processor list acceptance). Inputs from tenant-admin-console + end-user consent capture flows. Output: consent attributes that flow into Cedar evaluations as context. Consumer-audience only. |
| `dsar-cascade` | `oya-intelligence-dsar-cascade` | Per Article 17 / KR PIPA Article 36 erasure-request handling specific to AI memory + prompt history. Enumerates Intelligence-owned data classes carrying subject identifier; coordinates with Ontology + audit-chain DSAR substrates. Consumer-audience only. |
| `eu-ai-act-tier-ui` | `oya-intelligence-eu-ai-act-tier-ui` | Per-capability EU AI Act tier classification (Article 6 prohibited vs Article 6(2) high-risk vs Article 50 limited-risk vs minimal-risk) surfaced to tenant admins + end users. Tier authoring (which capability has which tier) lives in Cedar fragments per ADR-0144 + ADR-0243; this BC is the UI/UX surface. Consumer-audience only. |
| `tenant-admin-console-controls` | `oya-intelligence-tenant-admin-console-controls` | Tenant-admin-facing controls: enable/disable AI assist by product, by role, by data class. Per-feature + per-data-class budget controls. Audit-log access. Sub-processor consent management. Consumer-audience only. |
| `brand-ux-surface` | `oya-intelligence-brand-ux-surface` | Consumer-facing UI primitives that render the "oyatie intelligence" brand: sparkle icons, tier-badge UI, streaming-text components, "model thinking" UX, citation rendering, source-attribution rendering, refusal-message UX. Consumer-audience only. |

These 6 BCs are not invoked by `internal-platform-ops`,
`oyatie-self-modification`, or `developer-platform` calls. They are
invoked exclusively when the audience tag is `b2b-tenant-product`
or `b2c-consumer`.

### D-4. Opt-in LLM/provider-BYOK credential model

**Scope of this section.** This section covers LLM and provider API
credentials only — Anthropic, OpenAI, Google, Cohere, AWS Bedrock,
Azure OpenAI, self-hosted vLLM/SGLang/TensorRT-LLM, OpenRouter,
Together, Groq, HuggingFace Inference, Replicate, Apple Foundation
Models, etc. **encryption-BYOK** — tenant-supplied KMS root or
HSM partition for at-rest data encryption — is a separate concern,
owned by the encryption substrate per ADR-0251 §D-10, tracked by the
`byok_enabled` BOOLEAN column on `tenants` per ADR-0244 §D-3. The two
concerns are disjoint: a tenant may use provider-BYOK for its
provider API keys without using encryption-BYOK for its encryption
keys, and vice versa.

**provider-BYOK is opt-in.** Intelligence Substrate owns **zero
provider credentials in B2B-regulated and provider-BYOK-elected tenant
paths**; on B2C consumer surfaces (Messenger, Mail, Community,
Marketplace consumer side, Workflow Studio personal tier) the
substrate owns **platform-default provider credentials** scoped to
the `oyatie` tenant when the calling tenant has not opted in to
provider-BYOK. Every provider credential — default or tenant-owned —
resolves through the SecretReference primitive at call time from
`microservices/cloud-secrets/` (OpenBao, per ADR-0211 in-house +
ADR-0028 cloud microservice architecture); there is no credential
materialization in substrate code or images.

The three valid `provider_credential_mode` values for a tenant are:

- `platform_default` — tenant uses oyatie's default provider
  credentials. Default for B2C personal-use tenants. Cost accrues to
  the `oyatie` tenant under platform float; usage is metered and may
  be capped per tenant tier.
- `byok` — tenant brings its own provider subscription or API key
  (Anthropic org, OpenAI org, Google Cloud project, Bedrock account,
  etc.). Resolves to tenant-scoped SecretReferences only; no
  platform-shared provider credentials reachable from this code path.
- `byok_required_by_pack` — at least one active compliance pack has
  `provider_byok_required: true`. Mode is enforced; the tenant cannot
  select `platform_default`. Typical of HIPAA (provider must have a
  signed BAA with the tenant, not with oyatie), PCI DSS, FedRAMP,
  IL5/6, KR-FSS, and EU AI Act high-risk packs.

The SecretReference primitive is uniform whether the credential is:

- oyatie's organizational subscription with Anthropic / OpenAI /
  Google (paid by oyatie; used for internal-platform-ops + oyatie-
  self-modification + opt-in consumer fallback).
- oyatie's organizational API key with a less-common provider used
  for specific capabilities (e.g., Cohere for re-ranking).
- A tenant's organizational subscription with Anthropic / OpenAI /
  Google paid by the tenant (BYOA — Bring Your Own Account).
- A tenant's organizational API key (provider-BYOK — Bring Your Own provider key).
- A per-end-user OAuth token (for consumer-audience flows where the
  end user authenticates with a provider directly).

Same code path. Same audit row. Different owner declaration in the
SecretReference.

**Postgres DDL for the `secret_references` table:**

This table lives in `microservices/cloud-secrets/`. The contents of
the secret itself live in OpenBao; this table is the metadata +
ownership + policy layer.

```sql
-- microservices/cloud-secrets/migrations/NNNN_create_secret_references.sql

CREATE TABLE IF NOT EXISTS secret_references (
    -- Identity
    secret_reference_id     UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    secret_reference_slug   TEXT            NOT NULL,        -- human-readable identifier within owner scope
                                                              -- e.g., "anthropic-prod-2026q2"
    tenant_id               TEXT            NOT NULL         -- per ADR-0242: owning tenant scope; uses
                                                              -- dotted hierarchical sub-scope notation;
                                                              -- 'oyatie' is the platform-owner tenant
                            REFERENCES tenants(tenant_id),

    -- Owner (who pays + who is liable per ToS)
    owner_kind              owner_kind_t    NOT NULL,        -- enum: see below
    owner_principal_id      TEXT            NOT NULL,        -- principal id of the owner
                                                              -- (e.g., 'oyatie.platform-ops' for oyatie subscriptions;
                                                              --  'tenant-acme.admin' for tenant provider-BYOK;
                                                              --  'tenant-acme.user-7421' for per-user OAuth)

    -- Provider identification
    provider_family         TEXT            NOT NULL,        -- 'anthropic' | 'openai' | 'google' | 'aws-bedrock' |
                                                              -- 'azure-openai' | 'mistral' | 'cohere' | 'vllm-self-hosted' |
                                                              -- 'sglang-self-hosted' | 'tensorrt-llm-self-hosted' |
                                                              -- 'apple-foundation-models' | 'openrouter' | 'together' |
                                                              -- 'groq' | 'huggingface-inference' | 'replicate' | ...
    provider_account_label  TEXT            NULL,            -- optional account label within provider
                                                              -- (e.g., 'anthropic-org-abc-prod')

    -- Storage handle (the actual secret lives in OpenBao)
    openbao_path            TEXT            NOT NULL,        -- e.g., 'kv/tenants/tenant-acme/byok/anthropic-prod-2026q2'
    openbao_kv_version      INT             NOT NULL,        -- versioned secret in OpenBao
                                                              -- (rotation increments version; previous versions retained
                                                              --  per OpenBao versioning + retention policy)

    -- Credential metadata (NOT the credential itself)
    credential_kind         credential_kind_t NOT NULL,      -- enum: see below
    last_rotated_at         TIMESTAMPTZ     NULL,
    next_rotation_due_at    TIMESTAMPTZ     NULL,            -- per rotation policy
    rotation_policy_id      TEXT            NULL             -- references rotation_policies(rotation_policy_id)
                            REFERENCES rotation_policies(rotation_policy_id),

    -- Usage scope (which audiences may use this credential)
    permitted_audiences     audience_tag_t[] NOT NULL,       -- one or more of: internal-platform-ops,
                                                              -- b2b-tenant-product, b2c-consumer,
                                                              -- developer-platform, oyatie-self-modification
    permitted_modalities    modality_t[]    NOT NULL,        -- one or more of: text, image, audio, video,
                                                              -- embedding, code, multi-modal-combined
    permitted_data_classes  TEXT[]          NOT NULL         -- references data_classes per ADR-0099
                            DEFAULT '{}',                    -- empty = no restriction (caller's Cedar overlay
                                                              -- still applies)

    -- ToS clearance (owner's responsibility, encoded for audit)
    tos_clearance_evidence  JSONB           NOT NULL,        -- structured: tos_url, tos_version_hash,
                                                              -- accepted_at, accepted_by_principal,
                                                              -- legal_review_evidence_uri
    tos_last_reviewed_at    TIMESTAMPTZ     NOT NULL DEFAULT now(),
    tos_next_review_due_at  TIMESTAMPTZ     NOT NULL,

    -- Cost attribution
    cost_center             TEXT            NOT NULL,        -- e.g., 'oyatie.platform-ops.intelligence',
                                                              -- 'tenant-acme.finops.intelligence'
    monthly_budget_cents    BIGINT          NULL,            -- optional budget cap; enforced by cost-attribution BC
    hard_cap_cents          BIGINT          NULL,            -- optional hard ceiling; Cedar Forbid past this

    -- Lifecycle
    status                  secret_status_t NOT NULL DEFAULT 'active',
                                                              -- enum: 'active' | 'rotating' | 'compromised' |
                                                              --       'retired' | 'archived'
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    created_by              TEXT            NOT NULL,        -- principal who registered the reference
    updated_at              TIMESTAMPTZ     NOT NULL DEFAULT now(),
    updated_by              TEXT            NOT NULL,
    retired_at              TIMESTAMPTZ     NULL,
    retired_by              TEXT            NULL,
    retire_reason           TEXT            NULL,

    -- Audit-chain reference
    audit_chain_uri         TEXT            NOT NULL,        -- canonical link into audit chain for this reference's
                                                              -- lifecycle events

    -- Uniqueness + sanity
    CONSTRAINT secret_reference_slug_unique_per_tenant
        UNIQUE (tenant_id, secret_reference_slug),
    CONSTRAINT openbao_path_unique
        UNIQUE (openbao_path),
    CONSTRAINT non_empty_audiences
        CHECK (cardinality(permitted_audiences) > 0),
    CONSTRAINT non_empty_modalities
        CHECK (cardinality(permitted_modalities) > 0)
);

-- Enum types
CREATE TYPE owner_kind_t AS ENUM (
    'oyatie-subscription',          -- oyatie pays the provider; oyatie is the contracting party
    'oyatie-byok',                  -- oyatie holds a provider-BYOK arrangement with the provider
    'tenant-subscription',          -- tenant pays the provider directly; tenant is the contracting party
    'tenant-byok',                  -- tenant brings its own key
    'tenant-byoa',                  -- tenant brings its own account (OAuth, e.g., per-user provider auth)
    'end-user-oauth'                -- per-end-user OAuth token (consumer-audience flows)
);

CREATE TYPE credential_kind_t AS ENUM (
    'static-api-key',               -- long-lived API key
    'oauth-refresh-token',          -- OAuth refresh token (substrate gets short-lived access tokens at call time)
    'aws-iam-role-arn',             -- AssumeRole pattern (Bedrock)
    'azure-managed-identity',       -- Azure managed identity reference
    'gcp-service-account-json',     -- GCP SA JSON in OpenBao
    'apple-team-jwt',               -- Apple team JWT (Foundation Models API)
    'cloud-kms-wrapped-key',        -- HSM/KMS-wrapped self-hosted-model encryption key
    'mtls-client-cert'              -- mTLS client cert for self-hosted model endpoints
);

CREATE TYPE audience_tag_t AS ENUM (
    'internal-platform-ops',
    'b2b-tenant-product',
    'b2c-consumer',
    'developer-platform',
    'oyatie-self-modification'
);

CREATE TYPE modality_t AS ENUM (
    'text',
    'image',
    'audio',
    'video',
    'embedding',
    'code',
    'multi-modal-combined'          -- e.g., image+text input → text output
);

CREATE TYPE secret_status_t AS ENUM (
    'active',
    'rotating',
    'compromised',
    'retired',
    'archived'
);

-- Indexes
CREATE INDEX idx_secret_references_tenant
    ON secret_references (tenant_id);

CREATE INDEX idx_secret_references_owner_principal
    ON secret_references (owner_principal_id);

CREATE INDEX idx_secret_references_provider_family
    ON secret_references (provider_family);

CREATE INDEX idx_secret_references_status
    ON secret_references (status);

CREATE INDEX idx_secret_references_next_rotation_due
    ON secret_references (next_rotation_due_at)
    WHERE status = 'active';

CREATE INDEX idx_secret_references_tos_next_review_due
    ON secret_references (tos_next_review_due_at)
    WHERE status = 'active';

-- Per-audience partial index for fast routing-time lookup
CREATE INDEX idx_secret_references_active_per_audience
    ON secret_references USING GIN (permitted_audiences)
    WHERE status = 'active';

-- Per-modality partial index
CREATE INDEX idx_secret_references_active_per_modality
    ON secret_references USING GIN (permitted_modalities)
    WHERE status = 'active';
```

**Companion table — rotation policies:**

```sql
CREATE TABLE IF NOT EXISTS rotation_policies (
    rotation_policy_id      TEXT            PRIMARY KEY,    -- e.g., 'standard-90-day-rotation'
    description             TEXT            NOT NULL,
    rotation_interval_days  INT             NOT NULL,
    grace_period_days       INT             NOT NULL DEFAULT 14,
                                                            -- new key live for grace period before old is retired
    requires_human_approval BOOLEAN         NOT NULL DEFAULT FALSE,
    notification_targets    TEXT[]          NOT NULL,       -- audit-stream subscribers, on-call rotations
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT now()
);
```

**SecretReference resolution flow at call time:**

```rust
// microservices/intelligence/crates/oya-intelligence-credential-resolver/src/resolver.rs

pub async fn resolve(
    secret_reference_id: Uuid,
    audience: AudienceTag,
    modality: Modality,
    cedar_decision: CedarDecision,
) -> Result<EphemeralCredential, ResolveError> {
    // 1. Fetch SecretReference metadata.
    let reference = secret_references_repo::fetch(secret_reference_id).await?;

    // 2. Validate audience + modality + status.
    if !reference.permitted_audiences.contains(&audience) {
        return Err(ResolveError::AudienceNotPermitted);
    }
    if !reference.permitted_modalities.contains(&modality) {
        return Err(ResolveError::ModalityNotPermitted);
    }
    if reference.status != SecretStatus::Active {
        return Err(ResolveError::StatusNotActive(reference.status));
    }

    // 3. Verify Cedar decision permits this resolution.
    //    (Caller has already evaluated the dispatch decision; this verifies
    //     the determining policies included the SecretReference resource.)
    if !cedar_decision.includes_resource(reference.openbao_path.as_str()) {
        return Err(ResolveError::CedarDoesNotPermit);
    }

    // 4. Fetch ephemeral credential from OpenBao.
    let ephemeral = openbao_client::fetch_ephemeral(
        &reference.openbao_path,
        reference.openbao_kv_version,
        reference.credential_kind,
    ).await?;

    // 5. Emit audit row.
    audit_emit::emit(SecretReferenceResolved {
        secret_reference_id,
        owner_principal_id: reference.owner_principal_id.clone(),
        audience,
        modality,
        cedar_evaluation_id: cedar_decision.evaluation_id,
        resolved_at: now(),
    }).await?;

    // 6. Return ephemeral credential. Caller never persists.
    Ok(ephemeral)
}
```

The ephemeral credential is **never persisted by Intelligence**.
The substrate uses it for the current call and discards. OpenBao
remains the only durable holder.

### D-5. Multi-modal transport BCs (day-one)

Multi-modal is day-one, not bolted on later. The `transport` BC
contains per-modality sub-adapters:

| Sub-adapter | Crate sub-module | Inbound modalities | Outbound modalities |
|---|---|---|---|
| `transport::text` | `oya-intelligence-transport::text` | text | text |
| `transport::image-generation` | `oya-intelligence-transport::image_generation` | text (prompt) [+ optional reference image] | image |
| `transport::image-understanding` | `oya-intelligence-transport::image_understanding` | image (+ optional text prompt) | text |
| `transport::audio-transcription` | `oya-intelligence-transport::audio_transcription` | audio | text |
| `transport::audio-synthesis` | `oya-intelligence-transport::audio_synthesis` | text | audio |
| `transport::video-generation` | `oya-intelligence-transport::video_generation` | text (+ optional reference image/video) | video |
| `transport::video-understanding` | `oya-intelligence-transport::video_understanding` | video (+ optional text prompt) | text |
| `transport::code-generation` | `oya-intelligence-transport::code_generation` | text (+ optional code context) | code |
| `transport::code-understanding` | `oya-intelligence-transport::code_understanding` | code (+ optional text prompt) | text |
| `transport::multi-modal-combined` | `oya-intelligence-transport::multi_modal_combined` | any combination of {text, image, audio, video, code} | text [+ optional structured output] |

Multi-modal combined (image+text input → text output, or any
combination) is first-class — not built by chaining single-modal
calls. This matches GPT-4o (2024-05), Claude 3.5 Sonnet (2024-06),
Gemini 1.5 Pro (2024-02), and the broader frontier-model 2024-2026
capability curve.

Each sub-adapter exposes a normalized request/response shape so
that provider adapters under the hood (OpenAI vs Anthropic vs
Google vs Bedrock vs vLLM-self-hosted) can be swapped without
caller code change.

Per-modality streaming:

- Text: SSE token streaming (chunk-by-chunk).
- Audio synthesis: SSE byte chunking (real-time TTS streaming).
- Image generation: not streamed (full image returned); per-step
  progress emitted via WebSocket for UX where supported.
- Video generation: not streamed (full video returned); progress
  callback via Workflow Engine.
- Multi-modal combined: SSE for text-emitting branches; per-modality
  artifacts attached.

### D-6. Stateless dispatch + Workflow durability composition

Intelligence Substrate is **stateless by default** for dispatch.
Every call is request-in → response-out. The substrate holds no
multi-turn state, no session context, no chain-of-thought scratchpad
between calls.

Multi-turn agents are composed by wrapping Intelligence calls in
the Workflow Engine (per the canonical Workflow + Ontology adapter
layer doctrine, `feedback_workflow_objectgraph_adapter_layer`).
A multi-turn agent workflow:

1. Workflow Engine invokes Intelligence (`transport::text` or
   `transport::multi-modal-combined`) with full prompt context
   assembled from durable Workflow state.
2. Intelligence dispatches, returns response.
3. Workflow Engine appends to durable scratchpad, decides next step
   (loop, tool call, terminate).
4. If tool call: Workflow Engine invokes the appropriate
   `microservices/<ms>/` capability OR re-invokes Intelligence
   with the tool result (per D-12 Tool calling).
5. Loop until termination condition.

Durability lives in Workflow. Intelligence stays narrow.

**Opt-in `session-store` for chat-UX use cases.** A single
opt-in BC (NOT one of the 8 substrate BCs; co-located as an
optional Layer-B-adjacent BC) provides ephemeral session storage
for chat-like UX where the caller wants Intelligence to manage
session state without explicit Workflow wrap:

| BC (opt-in) | Crate | Responsibility |
|---|---|---|
| `session-store` | `oya-intelligence-session-store` | Per-user session state for chat-UX. Append-only message log; per-session TTL; per-tenant Cedar policy for retention. Opt-in via per-tenant compliance pack. NOT consumer-only — also usable by `oyatie.foundry.adr-drafter` if a long-running drafting session benefits, but most internal workflows use Workflow Engine directly. |

`session-store` is opt-in because the default is stateless dispatch.

### D-7. Caller-side RAG

RAG (Retrieval-Augmented Generation) is **caller-side**, not
Intelligence-side. The caller:

1. Reads from Ontology Functions (per the Ontology+Workflow adapter
   layer doctrine) — typically a `OntologyFunction::SemanticSearch`
   or `OntologyFunction::HybridSearch` call that hits Milvus (per
   ADR-0192) + Postgres + Citus.
2. Assembles a prompt that includes the retrieved context (with
   citations + provenance).
3. Calls Intelligence (`transport::text` or other modality).
4. Intelligence dispatches; returns response.
5. Caller post-processes (e.g., resolves citations against retrieved
   chunks; renders source attribution).

Intelligence does **not** perform retrieval itself. The reasons:

- **Narrow substrate stays narrow.** Pushing retrieval into
  Intelligence would couple Intelligence to Milvus, Postgres, Citus,
  Ontology schema. The substrate would grow unboundedly.
- **Provenance + citation is the caller's domain.** The caller
  knows what corpus it queried, what filtering it applied, what
  consent state governs which chunks are retrievable. Intelligence
  cannot model that.
- **Per-tenant retrieval policy lives in Ontology.** Per-data-class
  filtering, per-tenant authorization on the retrieval corpus, and
  per-jurisdiction overlay all sit in Ontology Cedar fragments.

**Shared library `oya-shared-rag-*` for common patterns.** A
shared crate family provides common RAG patterns so callers don't
re-implement:

| Crate | Responsibility |
|---|---|
| `oya-shared-rag-retriever` | Common retrieval patterns: top-K semantic, hybrid BM25+semantic, time-decayed, faceted. |
| `oya-shared-rag-chunker` | Document chunking strategies: sentence-aware, fixed-token, hierarchical, semantic. |
| `oya-shared-rag-citation` | Citation linking: chunk → source-document → on-screen citation. |
| `oya-shared-rag-prompt-builder` | Common RAG prompt templates with citation placeholders. |
| `oya-shared-rag-reranker` | Re-ranking helpers (calls Intelligence transport with embedding or reranker provider). |

The caller composes these. Intelligence remains a thin dispatch
layer.

### D-8. Embeddings as separate substrate µservice

A new µservice **`microservices/embeddings/`** handles embedding
generation + vector storage. Promoted as a peer substrate µservice
rather than embedded inside Intelligence because:

- **Different lifecycle.** Embedding generation has different
  latency profile (batch-friendly), different storage tier (Milvus
  primary; cold storage in object store), different rotation cadence
  (re-embed when model changes).
- **Different ownership domain.** Embeddings are a *data* concern
  (data class + retention + DSAR-cascade applies to the embedding
  itself). Intelligence is a *call* concern.
- **Different scaling axis.** Embedding generation scales with
  corpus size; Intelligence dispatch scales with per-second call
  rate. Separate µservices let each scale independently.

`microservices/embeddings/` BCs:

| BC | Responsibility |
|---|---|
| `embedding-generator` | Per-modality embedding generation (text, image, audio, code). Provider-pluggable (OpenAI text-embedding-3-large, Cohere embed-v3, Voyage, Mistral, self-hosted via vLLM-embeddings or SGLang). |
| `embedding-store` | Per-tenant Milvus collections (per ADR-0192). Per-cell deployment. |
| `embedding-policy` | Per-tenant embedding model selection (Cedar-governed); per-data-class retention (Cedar-governed); DSAR cascade for per-subject embedding rows. |
| `embedding-rotation` | Coordinated re-embed when the model changes (e.g., embedding-3-small → embedding-3-large). |
| `embedding-multi-modal` | Multi-modal embeddings (e.g., CLIP, OpenAI ada-vision). Per-modality collection. |

Per-tenant collections are mandatory (no cross-tenant collection
sharing). Multi-modal embeddings supported day-one.

### D-9. Fine-tuning as separate substrate µservice

A new µservice **`microservices/fine-tuning/`** handles per-tenant
fine-tunes. Promoted as a peer substrate µservice because:

- **Training data is highly sensitive.** Per-tenant isolation is
  mandatory (tenant-A's training data can never appear in tenant-
  B's fine-tuned model).
- **Auditable training pipelines.** Per-EU-AI-Act Article 10 (data
  governance for high-risk AI) the training-data provenance, bias
  testing, and validation evidence must be retained.
- **Different lifecycle from dispatch.** Fine-tunes are
  asynchronous, long-running, and infrequent. Intelligence dispatch
  is per-request, immediate.

`microservices/fine-tuning/` BCs:

| BC | Responsibility |
|---|---|
| `training-corpus` | Per-tenant training corpus storage (with provenance + consent state). Per-data-class retention. |
| `bias-validation` | Pre-training + post-training bias testing (per EU AI Act Article 10 §3). |
| `fine-tune-orchestrator` | Coordinates fine-tune jobs across provider-side (OpenAI fine-tune API, Anthropic fine-tune when GA, Google Vertex fine-tune) + self-hosted (vLLM-LoRA, SGLang fine-tune, TensorRT-LLM custom). |
| `fine-tune-registry` | Per-tenant fine-tuned model catalog. Bound to Intelligence's tool-registry for dispatch. |
| `fine-tune-audit` | Per-fine-tune audit trail: training data, validation evidence, deployment events, version lineage. |

**Day-one architecture; per-tenant launch as certification permits.**
The architecture is day-one (the BCs are scaffolded, the table
schemas exist, the contracts are defined). The per-tenant fine-tune
*launch* (when an actual customer fine-tunes a model) is gated on
that customer's compliance pack certifying its training corpus
provenance.

### D-10. Model serving — external + own-hosted

Intelligence's `transport` BC dispatches to two model-serving
tiers:

**External provider APIs** (for frontier capability):

- Anthropic Claude family (Sonnet, Opus, Haiku) via Anthropic API.
- OpenAI GPT family (GPT-4o, GPT-5 if released, o1, o3) via OpenAI
  API or Assistants v2 API.
- Google Gemini family via Google Generative AI API.
- AWS Bedrock for Anthropic / Mistral / Llama 4 / Titan hosted in
  AWS.
- Azure OpenAI for OpenAI models hosted in Azure.
- Apple Foundation Models API (for on-device Apple platform
  deployments where applicable).

**Own-hosted serving stack** (for fine-tunes, small models, and
on-prem regulated deployments):

- **vLLM** for high-throughput continuous batching (latency-optimized
  for long-context dispatch).
- **SGLang** for structured-output + tool-use workloads (per
  SGLang's structured-generation primitives, 2024 release).
- **TensorRT-LLM** for NVIDIA-optimized inference (used in EU
  sovereign-pack cells where NVIDIA hardware is locally available).
- **llama.cpp** for CPU-only fallback in resource-constrained edge
  cells.

The provider adapter normalizes across all serving tiers.
Intelligence callers never see the underlying serving tech; they
see a normalized model identifier (e.g., `anthropic/claude-3-5-sonnet-20240620`
or `oyatie-self-hosted/llama-4-70b-instruct-2025-q1`).

**Model identifier syntax:**

```
<provider-family> / <model-name> [ @ <model-version> ] [ : <variant> ]

Examples:
  anthropic/claude-3-5-sonnet@20240620
  openai/gpt-4o@2024-08-06
  google/gemini-1.5-pro@002
  aws-bedrock/anthropic-claude-3-5-sonnet@20240620
  azure-openai/gpt-4o@2024-08-06:my-azure-deployment
  oyatie-self-hosted/llama-4-70b-instruct@2025-q1
  tenant-acme-fine-tune/customer-support-v2@2026-q1
```

The Cedar fragment governing model selection per call evaluates
against `model_identifier`, `data_class`, `jurisdiction`,
`tenant_id`, `audience_tag`. For example, a Cedar fragment may
permit `anthropic/claude-3-5-sonnet@*` for any data class, but
require `oyatie-self-hosted/*` for `data_class:PHI_PROTECTED` in
US-HIPAA jurisdiction.

### D-11. Per-cell deployment

**Intelligence Substrate** runs in every Tier 3 data-plane cell.
Each cell has a complete Intelligence Substrate deployment:

- `transport` Deployment (3+ replicas, HPA).
- `credential-resolver` Deployment (3+ replicas).
- `policy-engine-client` SDK (in every Intelligence pod; not its
  own Deployment).
- `guardrails` Deployment (3+ replicas; high CPU-bound for LLM-as-
  judge guardrails).
- `audit-emit` Deployment (3+ replicas).
- `tool-registry` Deployment (3+ replicas).
- `audience-policy-router` SDK (in every Intelligence pod).
- `cost-attribution` Deployment (3+ replicas).

**Consumer Brand Surface** runs only in cells that serve consumer
tenants (subset of Tier 3 cells). For cells dedicated to internal-
platform-ops or developer-platform audiences (e.g., the build-
plane cell that runs `oyatie.foundry.ci-agent` workflows), the
Brand Surface layer is absent — only Substrate is deployed.

**Cross-cell behavior** is only failover. Each cell's Intelligence
serves its local tenants. Cross-cell dispatch happens only when a
cell's local Intelligence is unhealthy (per ADR-0241 DR portfolio
T2 capability for Intelligence). Cross-cell dispatch costs an
additional Cedar evaluation (cross-cell-traffic permit per ADR-0243
§D-3).

### D-12. Tool calling

Tool calling (LLM invokes a callable function) is split between
Intelligence and Ontology:

- **Intelligence `transport` BC** receives the LLM's tool-call
  request as part of the response payload, normalizes it, and
  emits an audit row.
- **Ontology `tool-call-ingress` BC** (per the Ontology PRD's
  tool-call surface) is the **inbound** side for tool calls — when
  the LLM requests `OntologyFunction::SemanticSearch(...)`, the
  call lands at Ontology's `tool-call-ingress`, is gated by Cedar,
  is dispatched to the appropriate Ontology Function, and the
  result is returned.
- **Intelligence `transport` BC** receives the tool-call result and
  feeds it back into the LLM dispatch (either in the same
  conversation if the Workflow Engine is orchestrating, or as a
  new request if stateless).

The split exists because **tool calls touch tenant data**; the
tenant-data authorization model lives in Ontology, not in
Intelligence. Intelligence is the dispatcher; Ontology is the
data-access gatekeeper.

**Cedar gate per `autonomy_tier`** governs tool calls per the
existing autonomy-tier doctrine (per ADR-0144 EU AI Act graduated
risk + ADR-0247 self-modification). Each tool registered in
Intelligence's `tool-registry` carries:

- `tool_id`, `tool_name`.
- `autonomy_tier` ∈ {`T0-RO`, `T1-DRY-RUN`, `T2-HUMAN-CONFIRM`,
  `T3-AUDIT-ONLY`} per ADR-0144.
- `data_classes_touched: Vec<DataClass>` per ADR-0099.
- `cedar_fragment_id: FragmentId` governing invocation.
- `mcp_server_reference: Option<McpServerRef>` per Anthropic Model
  Context Protocol 2024 (when the tool is an MCP server).
- `evidence_emission_class: EventClass` for audit-chain.

The Cedar gate evaluates per-call: principal, tool_id, data classes,
audience, tenant. Permits the call only if the autonomy_tier permits
and all other context attributes are satisfied.

### D-13. Streaming

**SSE (Server-Sent Events)** for token streams (text + audio
synthesis chunks). Per-chunk audit emission + per-chunk cost
attribution. Cancellation supported via SSE close.

**WebSocket** for bidirectional streaming (rare; primarily for
real-time voice agents where the audio input + text/audio output
flow concurrently). Less common than SSE.

**No long-poll fallback.** SSE is the canonical streaming mechanism
per Stripe / Anthropic / OpenAI 2024 streaming-API conventions.
Browser support is universal (EventSource API).

Streaming per-chunk audit + cost:

```rust
// Inside `transport::text::dispatch_stream`:

let mut total_tokens_out = 0;

while let Some(chunk) = provider_stream.next().await {
    let chunk = chunk?;
    total_tokens_out += chunk.token_count;

    // Per-chunk audit (sampled if rate is high).
    if sample_rate.should_emit(chunk.chunk_index) {
        audit_emit::emit(IntelligenceStreamChunkEmitted {
            dispatch_id,
            chunk_index: chunk.chunk_index,
            token_count: chunk.token_count,
            emitted_at: now(),
        }).await?;
    }

    // Emit to caller's SSE channel.
    sse_tx.send(chunk.into_sse_event()).await?;
}

// Per-call cost attribution at stream completion.
cost_attribution::emit(IntelligenceCostAttributed {
    dispatch_id,
    tokens_in,
    tokens_out: total_tokens_out,
    cost_cents: provider_rate_card.compute_cost(tokens_in, total_tokens_out),
    cost_center: cedar_decision.cost_center.clone(),
}).await?;
```

### D-14. Conversation state

**Stateless substrate is the default.** Multi-turn coherence
achieved by:

- **Workflow Engine wrap** for autonomous agents (default for
  internal-platform-ops + oyatie-self-modification + developer-
  platform).
- **`session-store` BC opt-in** for chat-UX consumer use cases (b2b-
  tenant-product + b2c-consumer with explicit consent and a Cedar
  fragment activating session storage for the tenant).

`session-store` per-tenant Cedar policy governs:

- Whether transcripts persist beyond the session (default: ephemeral,
  TTL 24h).
- Per-data-class retention (e.g., PHI transcripts kept zero
  retention; non-sensitive transcripts kept per default retention).
- Cross-device visibility (whether user sees the same session on
  multiple devices).
- DSAR cascade scope (transcripts are subject to Article 17).

### D-15. Audience tagging — audience is a call tag, not a µservice boundary

Every Intelligence call carries an `audience` tag in its Cedar
evaluation context. The tag is set by the caller and validated by
Cedar (the caller's principal must be permitted to set that
audience).

The audience tag determines:

- **Which guardrails fire.** Consumer audiences get the strictest
  PII redaction + jailbreak detection. Internal-platform-ops gets a
  lighter guardrail set (no PII redaction needed if the prompt is
  source code; jailbreak detection still applies). Developer-
  platform gets a customizable guardrail set (within Cedar limits).
- **Which tools are available.** The tool-registry BC filters the
  per-call available tools by audience. Consumer audiences may have
  access to user-facing tools (calendar, email composition);
  internal-platform-ops may have access to repository tools (git
  ops, CI ops); developer-platform sees a developer-API tool set.
- **Which audit stream is the primary recipient.** Per ADR-0243
  §D-7: internal-platform-ops + oyatie-self-modification routes to
  `oyatie.*` streams; b2b-tenant-product + b2c-consumer routes to
  the tenant's primary stream; developer-platform routes to a
  per-developer audit channel.
- **Which cost center bills.** Internal-platform-ops + oyatie-self-
  modification: oyatie.platform-ops.intelligence sub-cost-center.
  b2b-tenant-product + b2c-consumer: tenant's intelligence cost
  center. Developer-platform: per-developer's account.
- **Which Cedar fragments compose at evaluation time.** Per ADR-0243
  §D-4 overlay composition: baseline + per-audience-overlay + per-
  pack overlays + tenant overrides.

**Audience is a call tag, not a µservice boundary.** This is the
explicit reversal of ADR-0220's framing. The substrate serves all
audiences uniformly; the audience-shaped routing happens at the
Cedar policy boundary.

### D-16. Foundry BC absorption — providers, guardrails, eval

Per ADR-0249 (Foundry dissolution, keystone #13), Foundry's BCs
redistribute. Three of those BCs are AI-relevant and land in
Intelligence under this ADR. The redistribution is exhaustive: no
Foundry-provider, Foundry-guardrail, or Foundry-eval crate remains
in the platform after absorption. The crate moves are tracked here.

**Full crate redistribution table:**

| Current Foundry crate | Destination | Destination BC | Notes |
|---|---|---|---|
| `crates/oya-intelligence-provider-domain/` | `microservices/intelligence/crates/oya-intelligence-transport/` | Substrate `transport` | Provider abstraction model folded into transport's provider-adapter trait set. |
| `crates/oya-intelligence-provider-kernel/` | `microservices/intelligence/crates/oya-intelligence-transport/` | Substrate `transport` | Provider kernel logic (request shaping, retry, circuit-breaker) folded into transport per-modality adapters. |
| `crates/oya-intelligence-provider-anthropic-adapter/` | `microservices/intelligence/crates/oya-intelligence-transport-anthropic/` | Substrate `transport::providers::anthropic` | Anthropic API adapter; updated to support multi-modal (image input) per Claude 3.5 Sonnet 2024-06. |
| `crates/oya-intelligence-provider-openai-adapter/` | `microservices/intelligence/crates/oya-intelligence-transport-openai/` | Substrate `transport::providers::openai` | OpenAI API adapter; updated to support GPT-4o multi-modal 2024-05 + Assistants v2 + Realtime API. |
| `crates/oya-intelligence-provider-google-adapter/` | `microservices/intelligence/crates/oya-intelligence-transport-google/` | Substrate `transport::providers::google` | Google Generative AI API adapter; Gemini multi-modal day-one. |
| `crates/oya-intelligence-provider-bedrock-adapter/` | `microservices/intelligence/crates/oya-intelligence-transport-aws-bedrock/` | Substrate `transport::providers::aws_bedrock` | AWS Bedrock adapter; supports IAM role assumption per AWS Bedrock 2024 GA. |
| `crates/oya-intelligence-provider-azure-openai-adapter/` | `microservices/intelligence/crates/oya-intelligence-transport-azure-openai/` | Substrate `transport::providers::azure_openai` | Azure OpenAI adapter; supports Azure Managed Identity. |
| `crates/oya-intelligence-provider-vllm-adapter/` | `microservices/intelligence/crates/oya-intelligence-transport-vllm/` | Substrate `transport::providers::vllm_self_hosted` | vLLM self-hosted adapter. |
| `crates/oya-intelligence-provider-sglang-adapter/` | `microservices/intelligence/crates/oya-intelligence-transport-sglang/` | Substrate `transport::providers::sglang_self_hosted` | SGLang self-hosted adapter; supports SGLang structured output. |
| `crates/oya-intelligence-provider-tensorrt-adapter/` | `microservices/intelligence/crates/oya-intelligence-transport-tensorrt/` | Substrate `transport::providers::tensorrt_llm_self_hosted` | TensorRT-LLM adapter for NVIDIA-optimized inference. |
| `crates/oya-intelligence-provider-credential-store-adapter/` | `microservices/intelligence/crates/oya-intelligence-credential-resolver/` | Substrate `credential-resolver` | Credential resolution logic folded into the new provider-BYOK SecretReference flow per §D-4. The Foundry version owned credentials directly; this version owns only references. |
| `crates/oya-intelligence-guardrails-domain/` | `microservices/intelligence/crates/oya-intelligence-guardrails/` | Substrate `guardrails` | Guardrails domain model folded; per-audience configuration added. |
| `crates/oya-intelligence-guardrails-kernel/` | `microservices/intelligence/crates/oya-intelligence-guardrails/` | Substrate `guardrails` | Guardrails kernel; pre-call + post-call pipeline. |
| `crates/oya-intelligence-guardrails-pii-redaction/` | `microservices/intelligence/crates/oya-intelligence-guardrails-pii-redaction/` | Substrate `guardrails::pii_redaction` | PII redaction logic; per-data-class behavior. |
| `crates/oya-intelligence-guardrails-prompt-injection-detector/` | `microservices/intelligence/crates/oya-intelligence-guardrails-prompt-injection-detector/` | Substrate `guardrails::prompt_injection_detector` | Prompt-injection heuristic + LLM-as-judge detector. |
| `crates/oya-intelligence-guardrails-toxic-content-classifier/` | `microservices/intelligence/crates/oya-intelligence-guardrails-toxic-content-classifier/` | Substrate `guardrails::toxic_content_classifier` | Toxic content classifier. |
| `crates/oya-intelligence-eval-domain/` | `microservices/intelligence/crates/oya-intelligence-eval/` | Substrate `eval` (new BC under Intelligence; not in the 8 listed above — added here, see note) | Eval domain model. |
| `crates/oya-intelligence-eval-kernel/` | `microservices/intelligence/crates/oya-intelligence-eval/` | Substrate `eval` | Eval kernel; runs golden-set evaluations + multispectrum review fan-out. |
| `crates/oya-intelligence-eval-runner/` | `microservices/intelligence/crates/oya-intelligence-eval-runner/` | Substrate `eval::runner` | Per-eval-job runner; orchestrates eval rubrics + LLM-as-judge. |
| `crates/oya-intelligence-eval-multispectrum-review-runner/` | `microservices/intelligence/crates/oya-intelligence-eval-multispectrum-review-runner/` | Substrate `eval::multispectrum_review_runner` | Multispectrum review v2.4.0 fan-out runner. |
| `crates/oya-intelligence-eval-golden-set-curator/` | `microservices/intelligence/crates/oya-intelligence-eval-golden-set-curator/` | Substrate `eval::golden_set_curator` | Per-capability golden set curation. |

**Note on `eval` BC:** §D-2 listed 8 substrate BCs. The `eval`
BC absorbed from Foundry brings the substrate to **9 BCs**. The
8→9 list:

1. `transport`
2. `credential-resolver`
3. `policy-engine-client`
4. `guardrails`
5. `audit-emit`
6. `tool-registry`
7. `audience-policy-router`
8. `cost-attribution`
9. `eval` (absorbed from Foundry)

`eval` is part of the AI Substrate because evaluation is an
audience-neutral concern (the same golden-set + multispectrum-review
machinery serves both `oyatie.foundry.eval-runner` evaluating an
ADR and `tenant-acme.product-team` evaluating a custom prompt).

**Crate naming convention.** All Intelligence crates follow the
`oya-intelligence-<bc-name>[-<sub-component>]` pattern per the
project's naming-justification doctrine
(`feedback_naming_justification`). The Foundry crates renamed during
absorption follow the same pattern.

### D-17. ADR-0220 fate — substantially rewritten

ADR-0220 (Consumer Intelligence Substrate, 2026-05-18) is
**substantially rewritten** by this ADR.

ADR-0220's frontmatter is updated:

```yaml
status: Substantially-Rewritten
superseded_by:
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
amendment_history:
  - 2026-05-20: Substantially rewritten by ADR-0255; audience-as-µservice-scope framing retired; model-routing-as-Intelligence-responsibility retired; substrate definition preserved; brand-label "oyatie intelligence" preserved.
```

ADR-0220's content is left in place (not deleted) with an
explanatory banner at the top pointing to ADR-0255. The reasons:

- ADR-0220 is referenced by 14 other ADRs + IP-001 + PRD-intelligence
  + manifest.json. Hard-deletion breaks references.
- The §D-17 amendment pattern (rather than supersession) preserves
  the historical record of *why* the rewrite happened, which is
  part of the keystone-bundle drift-loop closure argument.

**Preserved from ADR-0220:**

- The µservice exists at path `microservices/intelligence/` (not
  `microservices/oyatie-intelligence/`).
- The brand label "oyatie intelligence" remains the consumer-facing
  brand.
- Crate prefix `oya-intelligence-*`.
- Class C in-house mandatory per ADR-0211.
- Phase 1-4 roadmap concept (though the phases evolve per this
  ADR's BC structure).

**Retired from ADR-0220:**

- "Consumer-only" framing of the µservice (now audience-neutral
  substrate + consumer-only brand surface layer).
- "Model routing as one of Intelligence's responsibilities" (now
  Cedar-mediated per ADR-0243).
- The "shared substrate with Foundry" table (now obsolete; Foundry
  dissolves per ADR-0249; AI-relevant BCs absorbed into Intelligence
  per §D-16).
- Alternative 2's rejection rationale ("gateway unification hides
  audience differences"). This rationale is now demonstrably wrong:
  audience is a call tag at the Cedar boundary, not a µservice
  boundary, per ADR-0242 + ADR-0243.

### D-18. provider-BYOK + ToS interaction

**Tenants who use provider-BYOK** with oyatie's Intelligence Substrate
use their own provider credentials. These credentials are subject to the
**tenant's** ToS contract with the provider, not oyatie's.

**Oyatie's organizational subscriptions** (used for internal-
platform-ops + oyatie-self-modification + opt-in consumer
fallback) are subject to **oyatie's** ToS contract with the
provider.

The substrate provides **one code path** for both. The difference
is encoded in the `SecretReference.owner_kind` field:

- `owner_kind = 'oyatie-subscription'` → oyatie's ToS applies.
- `owner_kind = 'tenant-subscription'` → tenant's ToS applies.
- `owner_kind = 'tenant-byok'` → tenant's ToS applies; tenant
  attests to provider-BYOK ToS clearance.

**ToS clearance is the caller's responsibility**, encoded in the
SecretReference `tos_clearance_evidence` field (per §D-4 DDL).
At reference registration time, the tenant admin must attest:

```json
{
  "tos_url": "https://anthropic.com/legal/aup",
  "tos_version_hash": "sha256:abc123...",
  "accepted_at": "2026-05-15T14:32:00Z",
  "accepted_by_principal": "tenant-acme.admin#user-1234",
  "legal_review_evidence_uri": "https://tenant-acme.legal-records.example/anthropic-tos-review-2026q2",
  "scope_attested": ["text-generation", "image-understanding", "tool-use"],
  "sub_processor_disclosure_acknowledged": true,
  "data_residency_acknowledged": ["us-east", "us-west", "eu-west"]
}
```

The audit chain records this attestation. Substrate-level Cedar
fragments verify the attestation is current (per `tos_next_review_due_at`)
before permitting dispatch. Out-of-date attestation triggers a
`Forbid` until renewed.

**Implication for cross-tenant data:** When a tenant uses provider-BYOK,
only that tenant's data may transit the provider-BYOK credential. Cross-tenant
data dispatch always uses oyatie's organizational subscription
(audited per cross-tenant Cedar fragment). The Cedar fragment
governing per-call SecretReference selection enforces this.

## Alternatives considered

### Alt-1. Keep ADR-0220 as-is (consumer-only Intelligence + Foundry-internal providers)

Continue with ADR-0220's framing: Intelligence is consumer-only;
Foundry holds its own provider adapters + guardrails + eval BCs;
internal LLM use cases (`oyatie.foundry.*` workflows) call Foundry,
not Intelligence.

**Pros:**

- Zero migration cost (status quo).
- Sharp visual separation in code review of "is this consumer or
  internal code."
- Per the historical ADR-0220 Alternative 2 reasoning, consumer
  audience differences are visible at the µservice boundary.

**Cons:**

- **Contradicts ADR-0242 keystone #1.** The `oyatie`-is-a-tenant
  doctrine eliminates audience-as-µservice-scope. Foundry's
  internal-only carve-out dissolves per ADR-0249. Continuing to
  scope Intelligence as consumer-only re-introduces the very
  carve-out being retired.
- **Duplicates provider adapters.** Foundry's
  `foundry-provider-anthropic-adapter` and Intelligence's
  consumer-anthropic-adapter would be two crates wrapping the same
  Anthropic API. Drift inevitable.
- **Duplicates guardrails.** Foundry's
  `foundry-guardrails-pii-redaction` and Intelligence's
  consumer-pii-redaction would be two implementations of PII
  redaction. Authoring + review + maintenance doubled.
- **Duplicates eval machinery.** Foundry's golden-set + multispectrum-
  review runner can serve consumer prompt evaluation, but the
  ADR-0220 framing forced separate paths.
- **Contradicts ADR-0243 keystone #2.** Cedar-as-universal-gate
  requires that policy decisions (provider routing, tool-call
  permits, audience routing) be Cedar evaluations. ADR-0220 framed
  model-routing as Intelligence-internal code.
- **Contradicts ADR-0247 keystone #6.** Self-hosting / self-
  modification doctrine requires that `oyatie.foundry.*` autonomous
  workflows operate under the same gates as customer-tenant
  workflows. Two-substrate architecture creates two gate sets.
- **Drift loop evidenced.** ADR-0220 → ADR-0239 amendment in 12
  days demonstrates the framing isn't sticking; agents and
  contributors keep applying audience-as-scope language
  inconsistently.

**Rejected** because the cons accumulate over time and every other
keystone in the bundle implies the audience-as-µservice-scope
framing must dissolve.

### Alt-2. Unified single-layer Intelligence (no surface distinction)

Merge AI Substrate + Consumer Brand Surface into one undifferentiated
layer. Audience is a per-call tag throughout; no Layer-A vs
Layer-B distinction; the brand-UX BCs sit alongside the substrate
BCs as peer BCs.

**Pros:**

- Maximum architectural simplicity (no two-layer distinction).
- Single deployment unit; no per-cell decision whether to deploy
  Brand Surface.
- No risk of Layer-A vs Layer-B coordination bugs.

**Cons:**

- **Brand Surface BCs serve a narrower audience set than substrate
  BCs.** Consumer Brand Surface BCs (prompt-history, consent-cascade,
  DSAR-cascade, EU-AI-Act-tier-UI, tenant-admin-console-controls,
  brand-UX-surface) are only relevant for `b2b-tenant-product` and
  `b2c-consumer` audiences. Deploying them in cells that serve only
  `internal-platform-ops` or `developer-platform` is unnecessary
  compute + storage + Cedar fragment surface area.
- **Cell-deployment optimization lost.** Per ADR-0248 cell
  architecture, internal-platform-ops cells (e.g., the build plane)
  benefit from smaller deployment footprint. Forcing them to
  deploy Brand Surface BCs is wasteful.
- **Per-Layer security boundary lost.** Consumer Brand Surface BCs
  hold consumer prompt history, consent state, DSAR records — all
  consumer-sensitive data. Internal-platform-ops cells should
  NOT have this data resident even in a non-running deployment.
  Two-layer separation gives a clean security boundary.
- **Per-Layer audit-stream routing lost.** Consumer Brand Surface
  emits to consumer-tenant audit streams; substrate emits to per-
  audience streams. Mixing them complicates audit-stream selection.

**Rejected** because the cell-deployment + security-boundary
arguments win. Two-layer separation preserves operational
flexibility.

### Alt-3. Separate µservices for substrate vs brand surface

Promote AI Substrate and Consumer Brand Surface to two peer
µservices: `microservices/intelligence-substrate/` and
`microservices/intelligence-brand-surface/`.

**Pros:**

- Maximum isolation between layers.
- Independent versioning + deployment.
- Independent ownership.

**Cons:**

- **Tightly-coupled lifecycle.** Brand Surface depends on Substrate
  APIs. Every Substrate API change requires coordinated Brand
  Surface release. Two µservices add coordination cost without
  isolating change.
- **Cross-µservice latency.** Brand Surface → Substrate is an
  in-µservice function call when co-located; a network hop when
  split. Per-call latency budget tight.
- **Doubled operational surface.** Two µservices = two manifests,
  two runbooks, two contracts, two failure-mode docs, two threat
  models. Brand Surface alone wouldn't justify the operational
  overhead.
- **Contradicts ADR-0132 (no-grouping forward policy).** Splitting
  Intelligence into substrate + brand-surface µservices is a
  bundle/grouping shape; the policy forbids it for new µservices.
  Note: this is an addition-creation, so ADR-0132 applies.

**Rejected** because the lifecycle coupling + latency budget
arguments outweigh the isolation benefit.

### Alt-4. Outsource AI to OpenAI / Anthropic SaaS only

Don't build any in-house substrate. Use OpenAI's Assistants API or
Anthropic's API directly from each µservice. No `microservices/intelligence/`
at all; remove provider adapters entirely.

**Pros:**

- Lowest engineering cost.
- Provider's roadmap evolves automatically.

**Cons:**

- **Contradicts ADR-0211 (in-house tech stack preference).** Class
  C in-house substrate is mandatory.
- **No provider abstraction.** Each µservice hard-codes provider
  client SDK. Provider switch = N µservice changes.
- **No central guardrails.** PII redaction + prompt-injection
  detection becomes per-µservice (or absent).
- **No central audit.** Per-call audit becomes per-µservice
  (drift inevitable).
- **No central cost attribution.** Per-tenant + per-product cost
  accounting requires central touch point.
- **No multi-modal abstraction.** Each µservice handles modality
  per-provider.
- **No autonomous self-modification.** `oyatie.foundry.*` workflows
  need consistent dispatch + Cedar gates; per-µservice SDKs don't
  provide.
- **No EU AI Act tier classification surface.** Article 50 +
  Article 14 transparency requirements require a unified surface.
- **No provider-BYOK story for tenants.** Tenants bringing their own
  Anthropic key need a central credential-resolution layer.
- **Vendor lock-in.** Provider outage = platform outage with no
  fallback to a different provider.

**Rejected** outright per ADR-0211 + every operational concern.

### Alt-5. Two-layer Intelligence (AI Substrate + Consumer Brand Surface) ← **CHOSEN**

The selected alternative, fully specified in §Decision.

**Pros:**

- **Closes the audience-as-µservice-scope drift loop.** Audience
  is a call tag at the Cedar boundary, not a µservice property.
- **Matches every named industry reference.** AWS Bedrock + Step
  Functions, Azure AI Foundry, Apple Intelligence all operate as
  audience-neutral substrate with layered brand surfaces.
- **Cell-deployment optimization preserved.** Internal-platform-ops
  cells deploy Substrate only; consumer-tenant cells deploy both.
- **Security boundary preserved.** Consumer-sensitive data resident
  only in Brand Surface deployments.
- **provider-BYOK SecretReference code path.** Tenant credentials, oyatie
  credentials, end-user OAuth — same SecretReference primitive.
- **Multi-modal day-one.** Text + image + audio + video + code
  with per-modality sub-adapter. Multi-modal combined first-class.
- **Stateless + Workflow durability.** Substrate stays narrow; agent
  durability composed with Workflow Engine.
- **Caller-side RAG.** Substrate doesn't grow Milvus + Postgres
  dependencies; Ontology owns retrieval policy.
- **Foundry BC absorption.** Provider adapters + guardrails + eval
  consolidate; doubled-doctrine surface eliminated.
- **Embedding + fine-tuning as peer substrates.** Differentiated
  lifecycle + scaling + ownership; promoted out cleanly.
- **Compliance Pack abstraction works.** Per-pack Cedar fragments
  configure per-audience guardrails + tool subsets + model routing.

**Cons:**

- **Bounded one-time migration cost.** Foundry's provider +
  guardrails + eval crates move (per §D-16 table). Per-µservice
  call-site changes from `foundry::dispatch` to
  `intelligence::dispatch`. Bounded; one ChangeSet executes per
  µservice.
- **provider-BYOK SecretReference rollout.** All current
  credential-storage callers migrate to SecretReference resolution.
  ~30 call sites across µservices.
- **Multi-modal adapter authoring workload.** 10 per-modality
  adapters across providers. Mitigation: per-adapter is small
  (~500-1000 LoC); per-provider per-modality is well-tested.
- **Per-cell deployment of additional BCs.** Cell footprint grows.
  Mitigation: per-BC resources tuned; HPA scales replicas to
  per-cell traffic.
- **Eval BC absorbed from Foundry adds 9th BC to substrate.**
  Documented inconsistency with the "8 BCs" framing in §D-2 ↔
  §D-16. Mitigation: explicit note in §D-16; ADR-0249 cross-
  references this.

**Accepted** as the keystone #14 doctrine.

## Consequences

### Positive

1. **Uniformity.** One substrate serves every tenant including
   `oyatie`. Per ADR-0242 doctrine fulfilled at the Intelligence
   layer.
2. **Drift loop closed.** ADR-0220 → ADR-0239 amendment loop
   eliminated by removing the audience-as-µservice-scope framing.
3. **provider-BYOK SecretReference code path.** Same code path for oyatie's
   subscriptions, tenant subscriptions, tenant provider-BYOK, per-user
   OAuth. No special-case logic.
4. **Multi-modal day-one.** Text + image + audio + video + code +
   multi-modal-combined supported on day one. Frontier capability
   tracks tracked.
5. **Stateless substrate composes with Workflow durability.**
   Substrate stays narrow; agent durability composed externally.
6. **Caller-side RAG keeps substrate narrow.** Ontology owns
   retrieval policy; Intelligence dispatches.
7. **Compliance Pack abstraction works at AI layer.** Per-pack
   Cedar fragments configure per-audience guardrails, tool subsets,
   model routing, retention.
8. **Audit-chain coverage uniform.** Every Intelligence call emits
   per ADR-0243; per-audience stream routing.
9. **Cost attribution uniform.** Per-call cost row to FinOps portal
   under the deepest declared sub-scope.
10. **Hyperscaler shape.** Matches AWS Bedrock + Step Functions,
    Azure AI Foundry, Apple Intelligence layered-surface patterns.
11. **Autonomous-masterplan-execution unlocked.**
    `oyatie.foundry.adr-drafter` + `oyatie.foundry.eval-runner` +
    `oyatie.foundry.merge-queue` workflows make LLM calls through
    the same substrate as customer-tenant workflows. Per ADR-0247.
12. **Self-hosted model serving covered.** vLLM + SGLang +
    TensorRT-LLM + llama.cpp adapters for fine-tunes + small models
    + on-prem regulated deployments.

### Negative

1. **Foundry crate migration.** ~20 Foundry crates move per §D-16
   table. Bounded; one ChangeSet executes the migration.
2. **Per-µservice call-site updates.** Every µservice currently
   calling `foundry::*` for LLM dispatch updates to
   `intelligence::*`. ~12 call sites (verified by grep against
   current `crates/oya-application-app/tests/` and µservice
   manifests).
3. **provider-BYOK SecretReference rollout.** All credential-
   storage callers migrate to SecretReference. ~30 sites.
4. **Per-cell Intelligence Substrate deployment footprint.**
   Substrate-only cells gain Intelligence Deployment (9 BCs).
   Mitigation: per-BC resources tuned; substrate-only deployment
   excludes Brand Surface BCs.
5. **Cedar fragment proliferation for per-audience overlays.**
   Per-audience fragments per data class per modality per tool.
   Mitigation: ADR-0243 fragment naming convention + coverage CI.

### Operational

1. **Intelligence Substrate is T1 per ADR-0241.** RTO < 5min;
   zero data loss. Per-cell evaluator pods with HPA + circuit
   breakers + static-stability fallback.
2. **Consumer Brand Surface is T2 per ADR-0241.** RTO < 1h;
   consumer-facing degradation tolerable.
3. **Embeddings substrate is T2.** Per ADR-0241.
4. **Fine-tuning substrate is T3.** Per ADR-0241 (asynchronous;
   degradation tolerable for hours).
5. **Per-cell deployment footprint:**
   - Substrate (9 BCs): ~10-15 pods per cell, ~5-10 CPU cores
     allocated, ~10-20 GB memory.
   - Brand Surface (6 BCs, only in consumer-tenant cells): ~6-10
     pods, ~3-5 CPU cores, ~5-10 GB memory.
6. **Per-cell Cedar evaluator co-located with Intelligence.** Per
   ADR-0243 §D-6 in-cell cache target: 1ms p99 [P5..P95: 0.25ms–0.75ms]
   (modeled; requires DaemonSet co-location + Valkey sidecar + Cilium
   Ambient eBPF; evidence: docs/performance-budgets/cedar-hot-path-1ms-p99.md).
7. **Provider rate-limit + circuit-breaker dashboards.** Per-
   provider per-modality per-cell.
8. **New CI lanes:**
   - `oya-check-intelligence-two-layer-coherence` — verifies the
     8+1 (substrate) + 6 (brand surface) BC structure.
   - `oya-check-byok-everywhere-coherence` — verifies all
     credential resolution goes through SecretReference.
   - `oya-check-no-credentials-in-substrate` — static analysis;
     verifies no credentials persist in Intelligence code.
   - `oya-check-multi-modal-transport-coverage` — verifies
     adapters for text + image + audio + video + code + combined.
   - `oya-check-caller-side-rag-only` — verifies Intelligence
     does no retrieval calls.
   - `oya-check-audience-tag-on-every-call` — verifies every
     Intelligence call carries an audience tag in Cedar context.
   - `oya-check-foundry-bc-absorption-complete` — verifies all
     Foundry-{provider,guardrails,eval} crates have moved per
     §D-16.

### Sustainability

- **Model inference power consumption.** Per-call carbon footprint
  varies dramatically by model + modality. Self-hosted inference
  in cells co-located with low-carbon energy (per ADR-0240
  sovereign-pack locality) is preferred for high-volume bulk
  workloads. Per-call sustainability emission row attached to the
  cost-attribution audit per ADR-0174.
- **Multi-modal carbon footprint disclosure.** Image generation +
  video generation are 10-100× higher per-call than text. The
  cost-attribution BC includes per-modality sustainability tag.
- **Caching reduces re-dispatch.** Per-prompt deterministic caching
  (per-tenant; per-prompt-hash) reduces redundant calls. Cache
  hit rates surface in FinOps portal.

### Compliance

- **EU AI Act Article 14 (transparency obligations).** Every AI-
  mediated decision emits the determining-policies + applied-
  fragments list per ADR-0243 §D-7. Consumer-facing surfaces
  render "AI involved" disclosure per the EU AI Act tier UI BC.
- **EU AI Act Article 50 (transparency for emotion recognition,
  biometric categorization, deepfakes).** Per-capability tier
  classification surfaced. Image + video generation tagged
  appropriately.
- **EU AI Act Article 10 (data governance for high-risk AI).**
  Training-corpus provenance + bias validation evidence in
  `microservices/fine-tuning/`.
- **GDPR Article 22 (automated individual decision-making).**
  Per-decision audit row + rationale; subject-right-to-explanation
  surfaced via Consumer Brand Surface's `brand-ux-surface` BC.
- **HIPAA Security Rule §164.312 (access control).** Per-PHI-call
  Cedar permit + audit emission. PHI-eligible providers restricted
  via Cedar fragment per data class.
- **SOC 2 CC6.1 (logical access).** All AI access Cedar-mediated.
- **KR PIPA Article 22 (consent).** Consent state via Consumer
  Brand Surface `consent-cascade` BC.
- **KR-FSC AI Guidelines (2024-Q4 release).** Per-capability risk
  tier + audit retention applied.
- **NIST AI Risk Management Framework (AI RMF 1.0, 2023).** Govern
  + Map + Measure + Manage functions mapped to Cedar fragments +
  audit chain.
- **ISO/IEC 42001:2023 (AI Management System).** Per-capability
  documented control mapped to Cedar fragments.

## Implementation surface

| Artifact | Status |
|---|---|
| `/specs/microservices/intelligence.json` | UPDATE — two-layer + 9+6 BC structure |
| `/specs/microservices/embeddings.json` | NEW |
| `/specs/microservices/fine-tuning.json` | NEW |
| `/specs/byok-credential-model.json` | NEW — SecretReference canonical schema |
| `/specs/multi-modal-transport.json` | NEW — per-modality contract shape |
| `/specs/tool-call-protocol.json` | NEW — Intelligence ↔ Ontology tool-call protocol |
| `microservices/intelligence/manifest.json` | UPDATE — list 9 substrate BCs + 6 brand surface BCs |
| `microservices/intelligence/crates/oya-intelligence-transport/` | NEW (absorbs Foundry provider crates) |
| `microservices/intelligence/crates/oya-intelligence-transport-anthropic/` | NEW (absorbs `oya-intelligence-provider-anthropic-adapter/`) |
| `microservices/intelligence/crates/oya-intelligence-transport-openai/` | NEW (absorbs `oya-intelligence-provider-openai-adapter/`) |
| `microservices/intelligence/crates/oya-intelligence-transport-google/` | NEW (absorbs `oya-intelligence-provider-google-adapter/`) |
| `microservices/intelligence/crates/oya-intelligence-transport-aws-bedrock/` | NEW (absorbs `oya-intelligence-provider-bedrock-adapter/`) |
| `microservices/intelligence/crates/oya-intelligence-transport-azure-openai/` | NEW (absorbs `oya-intelligence-provider-azure-openai-adapter/`) |
| `microservices/intelligence/crates/oya-intelligence-transport-vllm/` | NEW (absorbs `oya-intelligence-provider-vllm-adapter/`) |
| `microservices/intelligence/crates/oya-intelligence-transport-sglang/` | NEW (absorbs `oya-intelligence-provider-sglang-adapter/`) |
| `microservices/intelligence/crates/oya-intelligence-transport-tensorrt/` | NEW (absorbs `oya-intelligence-provider-tensorrt-adapter/`) |
| `microservices/intelligence/crates/oya-intelligence-credential-resolver/` | NEW (absorbs `oya-intelligence-provider-credential-store-adapter/`; rewritten for provider-BYOK SecretReference resolution) |
| `microservices/intelligence/crates/oya-intelligence-policy-engine-client/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-guardrails/` | NEW (absorbs Foundry guardrails crates) |
| `microservices/intelligence/crates/oya-intelligence-guardrails-pii-redaction/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-guardrails-prompt-injection-detector/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-guardrails-toxic-content-classifier/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-audit-emit/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-tool-registry/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-audience-policy-router/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-cost-attribution/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-eval/` | NEW (absorbs Foundry eval crates) |
| `microservices/intelligence/crates/oya-intelligence-eval-runner/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-eval-multispectrum-review-runner/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-eval-golden-set-curator/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-prompt-history/` | NEW (consumer Brand Surface) |
| `microservices/intelligence/crates/oya-intelligence-consent-cascade/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-dsar-cascade/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-eu-ai-act-tier-ui/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-tenant-admin-console-controls/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-brand-ux-surface/` | NEW |
| `microservices/intelligence/crates/oya-intelligence-session-store/` | NEW (opt-in) |
| `microservices/embeddings/` | NEW µservice (peer substrate) |
| `microservices/fine-tuning/` | NEW µservice (peer substrate) |
| `microservices/cloud-secrets/migrations/NNNN_create_secret_references.sql` | NEW (per §D-4 DDL) |
| `crates/oya-shared-rag-retriever/` | NEW |
| `crates/oya-shared-rag-chunker/` | NEW |
| `crates/oya-shared-rag-citation/` | NEW |
| `crates/oya-shared-rag-prompt-builder/` | NEW |
| `crates/oya-shared-rag-reranker/` | NEW |
| Removal of `crates/oya-intelligence-provider-*/` (10 crates) | SWEEP per §D-16 |
| Removal of `crates/oya-intelligence-guardrails-*/` (5 crates) | SWEEP per §D-16 |
| Removal of `crates/oya-intelligence-eval-*/` (5 crates) | SWEEP per §D-16 |
| Update of call sites currently using `oya-intelligence-provider-*` → `oya-intelligence-transport-*` | SWEEP |
| Update of call sites currently using `oya-intelligence-guardrails-*` → `oya-intelligence-guardrails-*` | SWEEP |
| Update of call sites currently using `oya-intelligence-eval-*` → `oya-intelligence-eval*` | SWEEP |
| ADR-0220 frontmatter updated with `superseded_by: [ADR-0255]` for audience-as-µservice-scope framing | EDIT |
| ADR-0220 explanatory banner pointing to ADR-0255 | EDIT |
| `microservices/intelligence/PRD.md` rewritten | UPDATE |
| `microservices/intelligence/IP-001-consumer-intelligence-substrate.md` rewritten | UPDATE |
| `microservices/intelligence/operational-boundaries.md` rewritten | UPDATE |
| `microservices/intelligence/threat-model.md` rewritten | UPDATE |
| `microservices/intelligence/failure-modes.md` rewritten | UPDATE |
| `microservices/intelligence/cost-budget.md` rewritten | UPDATE |
| `microservices/intelligence/slos/*` rewritten per per-BC SLO | UPDATE |
| `microservices/intelligence/policy/*` rewritten per per-BC Cedar fragments | UPDATE |
| `microservices/intelligence/contracts/openapi/*.yaml` rewritten per BC | UPDATE |
| `microservices/intelligence/contracts/asyncapi/*.yaml` rewritten per BC | UPDATE |
| `microservices/intelligence/contracts/proto/*.proto` rewritten per BC | UPDATE |
| `microservices/intelligence/runbooks/*.md` rewritten per BC | UPDATE |
| `microservices/embeddings/{manifest.json,PRD.md,IP-001-*.md,contracts/,slos/,policy/,runbooks/,threat-model.md,failure-modes.md,cost-budget.md,operational-boundaries.md}` | NEW |
| `microservices/fine-tuning/{manifest.json,PRD.md,IP-001-*.md,contracts/,slos/,policy/,runbooks/,threat-model.md,failure-modes.md,cost-budget.md,operational-boundaries.md}` | NEW |
| `docs/standards/byok-everywhere-credential-model.md` | NEW |
| `docs/standards/multi-modal-transport.md` | NEW |
| `docs/standards/intelligence-two-layer.md` | NEW |
| `docs/standards/caller-side-rag.md` | NEW |
| `docs/runbooks/intelligence-substrate-incident-response.md` | NEW |
| `docs/runbooks/intelligence-brand-surface-incident-response.md` | NEW |
| `docs/runbooks/byok-rotation-ceremony.md` | NEW |
| `docs/runbooks/foundry-bc-absorption.md` | NEW (one-time absorption ChangeSet runbook) |
| `tools/oya-check-intelligence-two-layer-coherence/` | NEW |
| `tools/oya-check-byok-everywhere-coherence/` | NEW |
| `tools/oya-check-no-credentials-in-substrate/` | NEW |
| `tools/oya-check-multi-modal-transport-coverage/` | NEW |
| `tools/oya-check-caller-side-rag-only/` | NEW |
| `tools/oya-check-audience-tag-on-every-call/` | NEW |
| `tools/oya-check-foundry-bc-absorption-complete/` | NEW |

## Verification

- [ ] `microservices/intelligence/` exists with the 9 substrate BCs + 6 brand surface BCs + opt-in `session-store` BC per §D-2, §D-3.
- [ ] `microservices/embeddings/` exists as a peer substrate µservice per §D-8.
- [ ] `microservices/fine-tuning/` exists as a peer substrate µservice per §D-9.
- [ ] `microservices/cloud-secrets/` has the `secret_references` table per §D-4 DDL.
- [ ] All 20 Foundry crates listed in §D-16 have moved to Intelligence; no `oya-foundry-{provider,guardrails,eval}-*` crate remains.
- [ ] `oya gate validate intelligence-two-layer-coherence` exits 0.
- [ ] `oya gate validate byok-everywhere-coherence` exits 0 (every credential resolution goes through SecretReference).
- [ ] `oya gate validate no-credentials-in-substrate` exits 0 (no credentials persist in Intelligence code paths).
- [ ] `oya gate validate multi-modal-transport-coverage` exits 0 (all 10 sub-adapters per §D-5 implemented + tested).
- [ ] `oya gate validate caller-side-rag-only` exits 0 (Intelligence performs no retrieval calls).
- [ ] `oya gate validate audience-tag-on-every-call` exits 0 (every Intelligence call carries audience tag in Cedar context).
- [ ] `oya gate validate foundry-bc-absorption-complete` exits 0.
- [ ] Integration test: `oyatie.foundry.adr-drafter` workflow makes an LLM call. Audit chain records `IntelligenceDispatch` event under `oyatie.foundry` audit stream. Cost attributed to `oyatie.platform-ops.intelligence`. Cedar evaluation uses internal-platform-ops audience tag.
- [ ] Integration test: `tenant-acme.user-7421` consumer-audience chat call. Audit chain records `IntelligenceDispatch` event under `tenant-acme` audit stream. Cost attributed to `tenant-acme.finops.intelligence`. Cedar evaluation uses b2c-consumer audience tag. Consumer Brand Surface `prompt-history` BC writes transcript per tenant's consent policy.
- [ ] Both integration tests above use the same Intelligence Substrate code path (verified by tracing the call through `transport` BC + `credential-resolver` BC + `policy-engine-client` BC + `guardrails` BC + `audit-emit` BC + `cost-attribution` BC). The only differences are: audience tag value, SecretReference owner, audit stream selection, cost center selection, and whether Brand Surface BCs are invoked.
- [ ] Multi-modal integration test: image + text input → text output multi-modal-combined dispatch returns within SLO.
- [ ] Streaming integration test: SSE token stream from text dispatch; per-chunk audit emission sampled; per-call cost attributed at stream completion.
- [ ] provider-BYOK integration test: tenant-acme registers an Anthropic API key as SecretReference with `owner_kind = 'tenant-byok'`; subsequent tenant-acme dispatch uses tenant's key (verified via OpenBao audit log).
- [ ] Stateless dispatch verification: Intelligence holds zero per-call state between consecutive calls (verified by load testing with random call ordering).
- [ ] Caller-side RAG verification: a representative caller (e.g., Workflow Studio) reads from Ontology, builds prompt, calls Intelligence. Intelligence has no Milvus or Ontology dependency in its call path.
- [ ] Per-cell deployment verification: a substrate-only cell (no consumer tenants) has 9 substrate BC Deployments + zero Brand Surface BC Deployments. A consumer-tenant cell has 9 + 6 + opt-in `session-store`.
- [ ] T1 DR drill for Intelligence Substrate completes within 5 minutes per ADR-0241.
- [ ] T2 DR drill for Consumer Brand Surface completes within 1 hour per ADR-0241.
- [ ] EU AI Act tier classification surfaced per call (verified via `IntelligenceDispatch` audit row including tier).
- [ ] ADR-0220 frontmatter updated: `superseded_by: [ADR-0255]` for audience-as-µservice-scope framing; explanatory banner present.

## References

### AI / LLM / multi-modal product references (2024-2026)

- **AWS Bedrock product page (2024).** `aws.amazon.com/bedrock`. Audience-neutral substrate; per-model adapter; Knowledge Bases (RAG opt-in); Guardrails (2024-Q2 GA); Step Functions integration for durable workflows.
- **AWS Bedrock + Step Functions architecture blog (2024).** "Building serverless generative AI applications with Amazon Bedrock and AWS Step Functions" — aws.amazon.com/blogs/compute/.
- **AWS Bedrock Guardrails (2024-Q2 GA).** Pre-call + post-call guardrails; per-policy attachment.
- **AWS Bedrock Knowledge Bases.** Opt-in RAG; caller side opt-in.
- **AWS Bedrock Agents.** Tool-calling agents with Action Groups.
- **AWS Bedrock encryption-BYOK / customer-managed keys.** customer-managed-encryption keys for model inputs/outputs.
- **Azure AI Foundry (2024-Q4 rebrand from Azure AI Studio).** microsoft.com/azure/ai-foundry. Multi-modal day-one; per-deployment endpoint scoping; Entra ID tenant boundary.
- **Apple Intelligence WWDC 2024 keynote.** Apple's audience-neutral on-device + private-cloud-compute substrate; brand surface layered atop.
- **Apple Intelligence Foundation Language Models technical report (2024).** Architecture + training methodology.
- **Apple Foundation Models API (Apple Intelligence developer SDK, 2024).** Provider adapter target.
- **Google Gemini 1.5 Pro multi-modal launch (2024-02).** blog.google/technology/ai/. Multi-modal foundation model.
- **Google Gemini 2.0 (2024-12).** Multi-modal + tool use evolution.
- **OpenAI GPT-4o release blog (2024-05).** Multi-modal text+image+audio combined.
- **OpenAI GPT-5 (if released, 2025).** Frontier model evolution.
- **OpenAI o1 + o3 reasoning models (2024-12 / 2025).** Reasoning-focused model class.
- **OpenAI Sora (2024-2025).** Text-to-video generation.
- **OpenAI Assistants v2 API (2024-04).** Stateful assistants with file_search + code_interpreter + function_calling.
- **OpenAI Realtime API (2024-10).** Bidirectional voice + text streaming.
- **OpenAI Embeddings API (text-embedding-3-large, text-embedding-3-small, 2024-01).** Embedding generation primitive.
- **OpenAI Fine-tuning API (GPT-4o fine-tune, 2024).** Per-tenant fine-tune primitive.
- **Anthropic Claude 3.5 Sonnet (2024-06).** Multi-modal vision + tool use.
- **Anthropic Claude Opus 4 (2025).** Frontier reasoning + multi-modal.
- **Anthropic Claude model spec / Claude AUP.** ToS clearance reference.
- **Anthropic Model Context Protocol (MCP, 2024-11).** Tool-call protocol; provider adapter target.
- **Anthropic prompt caching (2024).** Cache-aware dispatch.
- **Anthropic message batches API (2024-10).** Batch dispatch primitive.

### Self-hosted model serving (2024-2026)

- **vLLM (2024 release evolution).** `vllm.ai`. PagedAttention + continuous batching for high-throughput LLM serving.
- **SGLang (2024).** `sglang.ai`. Structured-generation primitives + LMQL-style programming model.
- **TensorRT-LLM (NVIDIA, 2024).** `github.com/NVIDIA/TensorRT-LLM`. NVIDIA-optimized inference engine.
- **llama.cpp (ggerganov, 2024-2026 evolution).** CPU-only inference fallback.
- **Hugging Face Text Generation Inference (TGI, 2024).** Alternative serving substrate.
- **Triton Inference Server.** General-purpose model serving.

### Embeddings / vector / RAG references

- **Pinecone vector DB.** Commercial vector store reference.
- **Milvus (per ADR-0192).** In-house vector substrate.
- **OpenAI Embeddings API.** Per §above.
- **Cohere embed-v3 (2024).** Embedding provider.
- **Voyage AI embeddings.** Embedding provider.
- **Mistral embed (2024).** Embedding provider.
- **CLIP (Radford et al., 2021).** Multi-modal embedding foundation.
- **OpenAI text-embedding-ada-002 → text-embedding-3.** Embedding model evolution.

### provider-BYOK + encryption-BYOK pattern references

- **Stripe encryption-BYOK pattern.** `stripe.com/docs/security/byok` (customer-managed encryption).
- **AWS Bedrock encryption-BYOK (customer-managed encryption keys).** Per-customer KMS keys for model inputs/outputs.
- **Azure Key Vault customer-managed keys.** encryption-BYOK pattern for Azure resources.
- **GCP Cloud KMS customer-managed encryption keys (CMEK).** encryption-BYOK pattern for GCP resources.
- **HashiCorp Vault Transit secrets engine.** Inspiration for OpenBao SecretReference pattern.

### Compliance / regulatory references

- **EU AI Act 2024/1689.** Articles 6, 10, 14, 50.
- **EU AI Act Article 14 — Transparency obligations.** Per-call disclosure requirements.
- **EU AI Act Article 50 — Transparency for emotion recognition, biometric categorization, deepfakes.** Multi-modal generation tagging.
- **EU AI Act Article 10 — Data governance for high-risk AI.** Training-corpus provenance + bias validation.
- **GDPR Article 17 (right to erasure).** DSAR-cascade scope.
- **GDPR Article 22 (automated individual decision-making).** Per-decision audit + explanation.
- **HIPAA Security Rule §164.312 (access control).** PHI access audit.
- **SOC 2 Type II CC6.1.** Logical access control.
- **KR PIPA Article 22 (consent).** Consent capture surface.
- **KR PIPA Article 36 (erasure equivalent).** DSAR cascade.
- **KR-FSC AI Guidelines (2024-Q4 release).** Per-capability risk tier.
- **NIST AI Risk Management Framework (AI RMF 1.0, 2023).** Govern/Map/Measure/Manage.
- **ISO/IEC 42001:2023 (AI Management System).** Per-capability control.
- **ISO/IEC 23894:2023 (AI risk management).** Risk management framework.
- **OECD AI Principles (2019, updated 2024).** Trustworthy AI principles.
- **FRCP 37(e).** Legal hold supersedes retention sunset.

### Foundational engineering / cloud patterns

- **AWS Builders' Library — "Static stability using Availability Zones" (Becky Weiss + Mike Furr).** Fail-closed default + cache fallback.
- **AWS Builders' Library — "Avoiding insurmountable queue backlogs" (Marc Brooker).** Per-call gate doctrine.
- **AWS Builders' Library — "Implementing health checks."** Per-cell evaluator health.
- **Stripe Engineering — "How Stripe uses Stripe."** Eat-your-own-dogfood reference.
- **AWS re:Invent 2019 keynote (Werner Vogels).** Amazon-on-AWS architectural review.
- **AWS Bedrock + Step Functions architecture blog (2024).** Per §AWS Bedrock above.
- **CNCF OpenTelemetry.** Per-call tracing reference.
- **Server-Sent Events (W3C EventSource).** SSE streaming spec.

### Internal portfolio ADRs

- **ADR-0028 — Cloud microservice architecture.** Intelligence per-cell shape.
- **ADR-0050 — Event bus Kafka.** Audit-chain emission substrate.
- **ADR-0099 — Data class registry.** Data classes drive Cedar fragment evaluation in Intelligence.
- **ADR-0105 — Thirteen-layer canonical enum.** Intelligence BCs live in their per-layer crates.
- **ADR-0128 — Hyperscaler architecture invariants.** Aligned.
- **ADR-0136 — Foundry as single µservice.** Amended (Foundry dissolves per ADR-0249).
- **ADR-0144 — EU AI Act graduated-risk tier model.** Per-capability tier surfaced in Intelligence.
- **ADR-0145 — Inter-microservice communication reform.** Direct gRPC for Intelligence calls.
- **ADR-0150 — Cedar policy engine.** Per ADR-0243 extends to all Intelligence policy decisions.
- **ADR-0174 — FinOps tag + sustainability.** Per-call sustainability emission.
- **ADR-0176 — Brown-out + degradation signal.** Intelligence evaluator health signal.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Cedar for Intelligence app-tier; Kyverno for admission.
- **ADR-0188 — Passkey / WebAuthn.** Tenant admin authentication for Brand Surface BCs.
- **ADR-0192 — Milvus vector substrate.** Embeddings substrate µservice uses Milvus.
- **ADR-0200 — Wasmtime plugin runtime.** Tool sandboxing for Intelligence tool calls.
- **ADR-0211 — In-house Rust-primary tech stack.** Intelligence is Class C in-house.
- **ADR-0212 — Buildability doctrine.** This ADR is itself a buildable artifact.
- **ADR-0213 — Ecosystem-as-a-service architecture.** Intelligence is the AI ecosystem touchpoint.
- **ADR-0215 — Multi-context platform.** Intelligence respects context scoping.
- **ADR-0218 — Tenant granular control surface.** Tenant-admin-console-controls BC realizes this for AI.
- **ADR-0219 — No-code builder suite.** Assist-draft import targets builder µservices.
- **ADR-0220 — Consumer Intelligence Substrate.** Substantially rewritten by this ADR.
- **ADR-0221 — Agentic development pipeline hardening.** §M-04 audience-of-µservice field retired per ADR-0242.
- **ADR-0239 — Foundry internal-scope clarification (2026-05-18).** Amended by ADR-0242.
- **ADR-0240 — Sovereign cloud per regional pack.** Per-pack Cedar overlays applied to Intelligence.
- **ADR-0241 — DR + BC portfolio policy.** Intelligence Substrate T1; Brand Surface T2; Embeddings T2; Fine-tuning T3.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine (keystone #1).** Doctrinal source for audience-as-call-tag.
- **ADR-0243 — Cedar as universal gate (keystone #2).** Doctrinal source for Cedar-mediated dispatch decisions.
- **ADR-0244 — Tenant as universal scoping primitive (keystone #3).** Every Intelligence call carries tenant scope.
- **ADR-0245 — Substrate vs Product layering (keystone #4).** Intelligence is a substrate; Brand Surface is a product layer atop.
- **ADR-0246 — Policy-engine substrate promotion (keystone #5).** Intelligence calls policy-engine.
- **ADR-0247 — Self-hosting / self-modification (keystone #6).** Autonomous workflows make LLM calls through Intelligence.
- **ADR-0248 — Amazon-shape cellular architecture (keystone #7).** Intelligence per-cell.
- **ADR-0249 — Foundry dissolution (keystone #13).** Foundry's AI BCs absorbed by Intelligence per this ADR's §D-16.
- **ADR-0251 — Compliance Pack + Cell Certification Levels.** Per-pack Cedar fragments configure Intelligence behavior.

### Auto-memory feedback

- `feedback_oyatie_is_a_tenant_doctrine` — applies; Intelligence serves `oyatie` tenant same as any tenant.
- `feedback_cedar_as_universal_gate` — applies; all Intelligence policy decisions via Cedar.
- `feedback_bominal_inheritance_precedence` — applies; this ADR overrides Bominal's audience-as-µservice-scope inheritance.
- `feedback_quality_performance_scalability_bar` — reinforced; hyperscaler-grade pattern.
- `feedback_autonomous_implementation_artifacts` — reinforced; autonomous masterplan workflows use Intelligence Substrate.
- `feedback_flat_product_catalog` — preserved; Intelligence remains a single flat µservice.
- `feedback_workflow_objectgraph_adapter_layer` — applied; Workflow durability wraps Intelligence; Ontology owns retrieval.
- `feedback_workflow_is_shared` — applied; Workflow Engine is the durability composer.
- `feedback_glossary_ontology_not_object_graph` — applied; Ontology used (not Object Graph).
- `feedback_autonomous_decision_principles` — applied; long-term right (uniform substrate) > short-term migration cost.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the keystone bundle (ADR-0242
through ADR-0255), every architectural decision in this ADR is
attributed to a named hyperscaler pattern + source + anti-pattern
avoided. Required appendix.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (two-layer model — AI Substrate + Consumer Brand Surface) | "Substrate + Brand Surface Layering" | Apple Intelligence WWDC 2024 keynote; AWS Bedrock + product UI separation; Azure AI Foundry per-deployment endpoint scoping | "Audience-as-µservice-scope" — explicitly retired by ADR-0242 + every named reference |
| D-2 (AI Substrate BCs, audience-neutral) | "Audience-Neutral Substrate" | AWS Bedrock model invocation API; Azure AI Foundry inference endpoint; Apple Foundation Models API | "Consumer-Only Substrate" — substrate refusing internal-platform-ops calls |
| D-3 (Consumer Brand Surface BCs, consumer-only) | "Layered Brand Surface" | Apple Intelligence consumer surfaces; Salesforce Einstein consumer UI atop Einstein substrate | "Brand Concerns in Substrate" — consumer UX leaking into substrate code |
| D-4 (provider-BYOK SecretReference) | "provider-BYOK SecretReference + Owner Declaration" | Provider API-key SecretReference; AWS Bedrock role assumption; tenant-owned provider account references | "Substrate-Owned Credentials" — secrets persisted in substrate code or DB |
| D-5 (multi-modal transport day-one) | "Multi-Modal Day-One Provider Adapter" | GPT-4o multi-modal 2024-05; Claude 3.5 Sonnet 2024-06; Gemini 1.5 Pro 2024-02; Azure AI Foundry multi-modal | "Text-First, Modality-Later" — Apple Intelligence's late image-generation surface scramble 2024 |
| D-6 (stateless dispatch + Workflow durability) | "Stateless Substrate + Durable Composition" | AWS Bedrock stateless API + Step Functions durability; Anthropic message batches + workflow caller composition | "Stateful Substrate" — substrate growing session storage + retry + checkpointing |
| D-7 (caller-side RAG) | "Caller-Side Retrieval" | AWS Bedrock Knowledge Bases (opt-in caller side); Anthropic prompt-caching with caller-built context; Azure AI Foundry retrieval connectors | "Substrate-Side Retrieval Coupling" — substrate growing vector DB + tenant-data dependencies |
| D-8 (embeddings as separate substrate) | "Embeddings Substrate Promotion" | Pinecone-as-substrate; Milvus-as-substrate; AWS Bedrock Knowledge Bases retrieval substrate | "Embeddings Embedded in Inference Substrate" — coupled lifecycle + scaling |
| D-9 (fine-tuning as separate substrate) | "Fine-Tuning Substrate Promotion" | AWS Bedrock model customization; Azure AI Foundry fine-tune; OpenAI fine-tune API | "Fine-Tuning Embedded in Dispatch Substrate" — coupled training + inference paths |
| D-10 (external + own-hosted model serving) | "Hybrid Model Serving" | AWS Bedrock (external + Bedrock-hosted) + custom model import; Azure AI Foundry (Azure OpenAI + Llama + custom) | "Single-Tier Model Serving" — pure-external (vendor lock-in) or pure-self-hosted (no frontier) |
| D-11 (per-cell deployment) | "Per-Cell Substrate Deployment" | AWS region + AZ Bedrock deployment; Apple per-region private cloud compute; Azure AI Foundry per-region endpoint | "Global Singleton Substrate" — cross-region call hops + sovereignty violation |
| D-12 (tool calling split Intelligence/Ontology) | "Tool-Call Ingress + Dispatcher Separation" | Anthropic MCP architecture (server + host separation); AWS Bedrock Agents Action Groups | "Tool-Call Logic in LLM Substrate" — tenant-data authorization leaking into dispatch |
| D-13 (streaming via SSE + WebSocket bi-directional) | "Streaming via SSE Conventions" | Anthropic streaming API; OpenAI streaming API; Stripe SSE conventions | "Long-Polling Streaming" — connection thrash + audit-row loss |
| D-14 (conversation state — stateless default + opt-in session-store) | "Opt-In Session State" | OpenAI Assistants v2 (opt-in stateful); AWS Bedrock Agents conversational state | "Always-Stateful Substrate" — every caller pays session-storage cost |
| D-15 (audience as call tag) | "Audience-As-Call-Tag" | AWS Bedrock Guardrails per-policy attachment; Azure AI Foundry per-deployment scoping; Apple Intelligence per-surface brand | "Audience-As-Service-Boundary" — explicitly retired by ADR-0220 alternative reconsidered |
| D-16 (Foundry BC absorption) | "Substrate Consolidation Under Universal Tenancy" | AWS Bedrock absorbing prior per-team provider adapters; Azure AI Foundry consolidating Azure AI Studio + Azure ML + Azure OpenAI | "Doubled Provider Adapter Surface" — duplicated Anthropic/OpenAI/Google adapters across internal+consumer |
| D-17 (ADR-0220 fate — substantially rewritten) | "ADR Drift-Loop Closure via Keystone Rewrite" | Internal portfolio practice; comparable to AWS Bedrock product page evolution 2023→2024 | "Silent ADR Drift" — keeping a contradictory ADR in force |
| D-18 (provider-BYOK + ToS interaction) | "Owner-Declared ToS Clearance" | AWS Bedrock per-customer ToS; Anthropic AUP per-organization acceptance; OpenAI per-account ToS | "Substrate-Implicit ToS Coverage" — assumed ToS without per-credential attestation |

---

## Appendix B: Worked example — `oyatie.foundry.ci-agent` multispectrum-review LLM call vs `tenant-customer-xyz.user-7421` consumer chat call

To demonstrate the substrate uniformity — that the same Intelligence
code path serves both internal-platform-ops and consumer-audience
calls, with only Cedar context attributes differing — here is a side-
by-side worked example.

### Setup

**Cell:** `data-plane-cell-us-west-2-a` (Tier 3; serves both
oyatie + tenant-customer-xyz; consumer-tenant cell — both Substrate
and Brand Surface BCs deployed).

**Substrate state:**

- Provider adapters loaded: anthropic, openai, google, aws-bedrock,
  vllm-self-hosted (for `llama-4-70b-instruct@2025-q1`).
- Cedar fragments compiled: baseline + us-overlay + eu-overlay +
  oyatie tenant + tenant-customer-xyz tenant + soc2-t2 pack +
  hipaa-pack (tenant-customer-xyz has it) + eu-ai-act-pack.
- SecretReferences active:
  - `secret-ref-oyatie-anthropic-prod-2026q2` (owner_kind:
    oyatie-subscription; permitted_audiences: internal-platform-ops,
    b2b-tenant-product, b2c-consumer, oyatie-self-modification;
    permitted_modalities: text, image, multi-modal-combined; PHI
    not permitted).
  - `secret-ref-customer-xyz-byok-anthropic-2026q2` (owner_kind:
    tenant-byok; permitted_audiences: b2b-tenant-product, b2c-
    consumer; permitted_modalities: text, image, audio,
    multi-modal-combined; PHI permitted via HIPAA pack).

### Call 1 — `oyatie.foundry.ci-agent` multispectrum-review LLM call

**Caller context:**

- Principal: `oyatie.foundry.ci-agent#instance-3421` (per ADR-0242
  dotted hierarchical sub-scope).
- Tenant: `oyatie`.
- Workflow context: `oyatie.foundry.eval-runner` is running a
  multispectrum-review v2.4.0 fan-out for PR #157 (an ADR draft);
  this call is the F1-correctness facet's LLM-as-judge dispatch.

**Caller invocation:**

```rust
let response = intelligence::transport::text::dispatch(
    TextDispatchRequest {
        model_identifier: "anthropic/claude-3-5-sonnet@20240620",
        prompt: build_f1_correctness_prompt(&adr_draft),
        max_tokens: 4096,
        temperature: 0.1,
        // Audience tag — set by caller, validated by Cedar.
        audience: AudienceTag::OyatieSelfModification,
        // No SecretReference specified — Cedar selects.
        secret_reference: None,
        // No tools requested for this call.
        tools: vec![],
        streaming: false,
        evaluation_id: Uuid::new_v4(),
    },
    context::Caller {
        principal: "oyatie.foundry.ci-agent#instance-3421",
        tenant_id: "oyatie",
        workflow_id: Some("oyatie.foundry.eval-runner#pr-157-fan-out"),
        cell_id: "data-plane-cell-us-west-2-a",
        data_classes_touched: vec!["SOURCE_CODE_INTERNAL", "ADR_DRAFT_INTERNAL"],
        jurisdiction: "US-DE",
    },
).await?;
```

**Substrate flow:**

1. **`audience-policy-router` BC** validates audience tag set by
   caller is permitted for principal. Cedar evaluation: principal
   `oyatie.foundry.ci-agent` is permitted to set audience
   `oyatie-self-modification`. ✓
2. **`policy-engine-client` BC** invokes Cedar:
   ```
   EvaluationRequest {
     principal: "oyatie.foundry.ci-agent#instance-3421",
     action: "Intelligence::Action::TextDispatch",
     resource: "Provider::anthropic/claude-3-5-sonnet@20240620",
     context: {
       audience: "oyatie-self-modification",
       data_classes: ["SOURCE_CODE_INTERNAL", "ADR_DRAFT_INTERNAL"],
       jurisdiction: "US-DE",
       cell: "data-plane-cell-us-west-2-a",
     },
     tenant_id: "oyatie",
   }
   ```
   Cedar evaluates against baseline + us-overlay + oyatie-tenant +
   soc2-t2 pack. Determines:
   - Permit: yes (oyatie-baseline-permits-oyatie-self-modification-
     anthropic-dispatch).
   - Selected SecretReference: `secret-ref-oyatie-anthropic-prod-2026q2`.
   - Selected cost center: `oyatie.platform-ops.intelligence`.
   - Selected audit stream: `oyatie.foundry`.
   - Selected guardrail bundle: `internal-platform-ops-guardrails`
     (lighter; PII redaction not applied since prompt is source
     code).
   - Selected tool subset: `oyatie-internal-tools` (none requested
     by caller anyway).
3. **`credential-resolver` BC** resolves
   `secret-ref-oyatie-anthropic-prod-2026q2` → ephemeral Anthropic
   API key from OpenBao. Emits `SecretReferenceResolved` audit row.
4. **`guardrails` BC** applies internal-platform-ops bundle:
   prompt-injection-detection runs (passes); PII redaction skipped
   (data class is SOURCE_CODE_INTERNAL, no PII expected); toxic-
   content-classifier runs (passes).
5. **`transport::text` BC** dispatches to Anthropic API using
   ephemeral key. Receives response.
6. **Post-call `guardrails` BC** scans response: PII leak detection
   (passes); jailbreak success detection (passes).
7. **`cost-attribution` BC** computes:
   - Tokens in: 4128; tokens out: 1832.
   - Provider rate: $3/MTok input + $15/MTok output (Claude 3.5
     Sonnet 2024-06 rates).
   - Cost: $0.04132.
   - Cost center: `oyatie.platform-ops.intelligence`.
   - Emits cost-attribution row.
8. **`audit-emit` BC** emits to audit chain:
   ```json
   {
     "event_class": "IntelligenceDispatch",
     "dispatch_id": "<uuid>",
     "principal": "oyatie.foundry.ci-agent#instance-3421",
     "tenant_id": "oyatie",
     "audience": "oyatie-self-modification",
     "model_identifier": "anthropic/claude-3-5-sonnet@20240620",
     "secret_reference_id": "secret-ref-oyatie-anthropic-prod-2026q2",
     "owner_kind": "oyatie-subscription",
     "modality": "text",
     "tokens_in": 4128,
     "tokens_out": 1832,
     "cost_cents": 4.132,
     "cost_center": "oyatie.platform-ops.intelligence",
     "cedar_evaluation_id": "<uuid>",
     "applied_fragments": ["baseline/intelligence-permits.cedar:v3", "overlay/us-de/oyatie-baseline-overlay.cedar:v1", "tenant/oyatie/foundry-permits.cedar:v2"],
     "guardrails_applied": ["internal-platform-ops-guardrails"],
     "guardrails_fired": [],
     "audit_stream": "oyatie.foundry",
     "emitted_at": "2026-05-20T14:32:11.234Z"
   }
   ```
9. **No Brand Surface BCs invoked** (audience is
   oyatie-self-modification; Brand Surface skipped per D-1).

**Result:** F1-correctness facet receives response; multispectrum-
review continues.

### Call 2 — `tenant-customer-xyz.user-7421` consumer chat call

**Caller context:**

- Principal: `tenant-customer-xyz.user-7421` (an end-user under
  tenant-customer-xyz).
- Tenant: `tenant-customer-xyz`.
- Caller is the Workflow Studio consumer chat UI; user is asking
  the assistant a question about their company's HR policy that
  touches PHI (employee health benefits).

**Caller invocation:**

```rust
// 1. Caller (Workflow Studio) does caller-side RAG via Ontology.
let retrieved_context = ontology::functions::semantic_search(
    SemanticSearchRequest {
        query: user_message.clone(),
        corpus: "tenant-customer-xyz.hr.policy-corpus",
        top_k: 5,
        tenant_id: "tenant-customer-xyz",
        principal: "tenant-customer-xyz.user-7421",
    }
).await?;

// 2. Caller assembles prompt with retrieved context.
let prompt = build_rag_prompt(
    user_message,
    retrieved_context,
    system_prompt_for_consumer_chat(),
);

// 3. Caller dispatches via Intelligence with consumer audience.
let response = intelligence::transport::text::dispatch_stream(
    TextDispatchRequest {
        model_identifier: "anthropic/claude-3-5-sonnet@20240620",
        prompt,
        max_tokens: 2048,
        temperature: 0.3,
        audience: AudienceTag::B2cConsumer,
        secret_reference: None,  // Cedar selects
        tools: vec![],
        streaming: true,
        evaluation_id: Uuid::new_v4(),
    },
    context::Caller {
        principal: "tenant-customer-xyz.user-7421",
        tenant_id: "tenant-customer-xyz",
        workflow_id: None,
        cell_id: "data-plane-cell-us-west-2-a",
        data_classes_touched: vec!["PHI_PROTECTED", "TENANT_BUSINESS_DATA"],
        jurisdiction: "US-CA",  // tenant-customer-xyz is California-based
    },
).await?;
```

**Substrate + Brand Surface flow:**

1. **Brand Surface — `consent-cascade` BC** verifies user 7421 has
   active consent for AI assist for HR data class. ✓
2. **Brand Surface — `prompt-history` BC** records the inbound
   prompt under user 7421's history (subject to tenant retention
   policy).
3. **Brand Surface — `eu-ai-act-tier-ui` BC** classifies the call
   per EU AI Act tier: limited-risk (Article 50); flag for
   transparency disclosure in response.
4. **Substrate — `audience-policy-router` BC** validates audience
   tag (b2c-consumer) is permitted for principal. ✓
5. **Substrate — `policy-engine-client` BC** invokes Cedar:
   ```
   EvaluationRequest {
     principal: "tenant-customer-xyz.user-7421",
     action: "Intelligence::Action::TextDispatch",
     resource: "Provider::anthropic/claude-3-5-sonnet@20240620",
     context: {
       audience: "b2c-consumer",
       data_classes: ["PHI_PROTECTED", "TENANT_BUSINESS_DATA"],
       jurisdiction: "US-CA",
       cell: "data-plane-cell-us-west-2-a",
     },
     tenant_id: "tenant-customer-xyz",
   }
   ```
   Cedar evaluates against baseline + us-overlay + ca-overlay +
   tenant-customer-xyz tenant + hipaa-pack + soc2-t2 pack + eu-ai-
   act-pack. Determines:
   - Permit: yes.
   - Selected SecretReference: `secret-ref-customer-xyz-byok-anthropic-2026q2`
     (tenant's provider-BYOK key; HIPAA-cleared per tenant's
     tos_clearance_evidence).
   - Selected cost center: `tenant-customer-xyz.finops.intelligence`.
   - Selected audit stream: `tenant-customer-xyz` primary stream.
   - Selected guardrail bundle: `b2c-consumer-hipaa-guardrails`
     (stricter; PII + PHI redaction applied; transparency
     disclosure required).
   - Selected tool subset: `tenant-customer-xyz-consumer-tools`.
6. **Substrate — `credential-resolver` BC** resolves tenant's provider-BYOK
   Anthropic key from OpenBao. Emits `SecretReferenceResolved`
   audit row under tenant-customer-xyz stream.
7. **Substrate — `guardrails` BC** applies b2c-consumer-hipaa-
   guardrails: PHI redaction in prompt (preserves de-identified
   medical references; masks specific subject identifiers);
   prompt-injection-detection; toxic-content-classifier.
8. **Substrate — `transport::text` BC** dispatches to Anthropic
   API using tenant's ephemeral key. Streams response back.
9. **Per-chunk `guardrails` BC** scans each chunk: PHI leak
   detection in response (real-time); jailbreak success detection.
10. **Per-call `cost-attribution` BC** computes (per-stream chunks
    aggregated at completion):
    - Tokens in: 1024; tokens out: 1432.
    - Provider rate: same Claude 3.5 Sonnet rates.
    - Cost: $0.02455.
    - Cost center: `tenant-customer-xyz.finops.intelligence`.
11. **Substrate — `audit-emit` BC** emits:
    ```json
    {
      "event_class": "IntelligenceDispatch",
      "dispatch_id": "<uuid>",
      "principal": "tenant-customer-xyz.user-7421",
      "tenant_id": "tenant-customer-xyz",
      "audience": "b2c-consumer",
      "model_identifier": "anthropic/claude-3-5-sonnet@20240620",
      "secret_reference_id": "secret-ref-customer-xyz-byok-anthropic-2026q2",
      "owner_kind": "tenant-byok",
      "modality": "text",
      "tokens_in": 1024,
      "tokens_out": 1432,
      "cost_cents": 2.455,
      "cost_center": "tenant-customer-xyz.finops.intelligence",
      "cedar_evaluation_id": "<uuid>",
      "applied_fragments": ["baseline/intelligence-permits.cedar:v3", "overlay/us-ca/customer-xyz-baseline-overlay.cedar:v1", "tenant/tenant-customer-xyz/consumer-permits.cedar:v4", "pack/hipaa/intelligence-phi-permits.cedar:v2", "pack/eu-ai-act/article-50-transparency.cedar:v1"],
      "guardrails_applied": ["b2c-consumer-hipaa-guardrails"],
      "guardrails_fired": ["phi-redaction:prompt:masked-employee-id-references"],
      "eu_ai_act_tier": "limited-risk",
      "audit_stream": "tenant-customer-xyz",
      "emitted_at": "2026-05-20T14:32:19.512Z"
    }
    ```
12. **Brand Surface — `prompt-history` BC** records the response
    under user 7421's history.
13. **Brand Surface — `brand-ux-surface` BC** renders streaming
    chunks with citation markers (from caller-side RAG context),
    EU-AI-Act tier badge (limited-risk), sparkle icon, "AI
    involved" disclosure per Article 14.

**Result:** User receives streaming chat response with citations,
tier disclosure, brand UX.

### Comparison

| Aspect | Call 1 (`oyatie.foundry.ci-agent`) | Call 2 (`tenant-customer-xyz.user-7421`) |
|---|---|---|
| Substrate BCs invoked | All 9 | All 9 |
| Brand Surface BCs invoked | None | 4 (`consent-cascade`, `prompt-history`, `eu-ai-act-tier-ui`, `brand-ux-surface`) |
| Audience tag | `oyatie-self-modification` | `b2c-consumer` |
| SecretReference owner | `oyatie-subscription` | `tenant-byok` |
| Cost center | `oyatie.platform-ops.intelligence` | `tenant-customer-xyz.finops.intelligence` |
| Audit stream | `oyatie.foundry` | `tenant-customer-xyz` |
| Guardrail bundle | `internal-platform-ops-guardrails` (lighter) | `b2c-consumer-hipaa-guardrails` (stricter) |
| PHI handling | N/A (no PHI in source code) | PHI redaction applied + audited |
| Cedar fragments evaluated | 3 (baseline + overlay + tenant) | 5 (baseline + overlay + tenant + hipaa-pack + eu-ai-act-pack) |
| EU AI Act tier surfaced | No (internal-platform-ops skips Brand Surface tier UI) | Yes (limited-risk badge rendered) |
| Streaming | No (batch dispatch) | Yes (SSE streaming) |
| Caller-side RAG | No (prompt is the ADR + rubric directly) | Yes (Ontology semantic search) |

**Substrate code path identity:** Calls 1 and 2 traverse the same
`transport::text` BC, the same `credential-resolver` BC, the same
`policy-engine-client` BC, the same `guardrails` BC, the same
`audit-emit` BC, the same `cost-attribution` BC. The differences
are entirely encoded in:

- Cedar context attributes (audience, data_classes, jurisdiction).
- Cedar fragment composition (per ADR-0243 overlay composition).
- SecretReference selection (Cedar-driven).
- Audit stream selection (Cedar-driven).
- Cost center selection (Cedar-driven).
- Guardrail bundle selection (Cedar-driven).
- Whether Brand Surface BCs are invoked (audience-driven).

This is the substrate-uniformity property the ADR establishes.
There is no fork in the substrate code path for "internal vs
consumer" — there is only a Cedar context value that the substrate
honors uniformly.

---

## Naming justification

Every name introduced or ratified by this ADR is validated against BNF v4.1
(`oya-<microservice>[-<bc-tokens>]-<layer>`) and the ADR-0105 13-value canonical
layer enum.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|------|-----------------|-------------------|---------------|
| `oya-shared-rag-retriever` | `sdk` (shared library) | `oya` · `shared` · `rag-retriever` | Shared library for caller-side RAG retrieval patterns; `rag-retriever` is a two-word BC token; lives as a `shared` µservice library per ADR-0245 |
| `oya-shared-rag-chunker` | `sdk` (shared library) | `oya` · `shared` · `rag-chunker` | Shared library for document chunking strategies (sentence-aware, fixed-token, hierarchical, semantic) |
| `oya-shared-rag-citation` | `sdk` (shared library) | `oya` · `shared` · `rag-citation` | Shared library for citation linking: chunk → source-document → on-screen citation |
| `oya-shared-rag-prompt-builder` | `sdk` (shared library) | `oya` · `shared` · `rag-prompt-builder` | Shared library of RAG prompt templates with citation placeholders; hyphenated three-word token is BNF-valid |
| `oya-shared-rag-reranker` | `sdk` (shared library) | `oya` · `shared` · `rag-reranker` | Shared library for re-ranking helpers; calls Intelligence transport with embedding or reranker provider |
| `oya-shared-secret-reference` | `sdk` (shared library) | `oya` · `shared` · `secret-reference` | Shared SecretReference primitive; substrate owns zero credentials — tenant provides reference only (per ADR-0255 §D-4 provider-BYOK model) |
| `oya-check-intelligence-two-layer-coherence` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `intelligence-two-layer-coherence` | Fitness-check; verifies the AI Substrate / Consumer Brand Surface boundary per §D-1; `oya-check-*` flat namespace |
| `oya-check-byok-everywhere-coherence` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `byok-everywhere-coherence` | Fitness-check; verifies all provider credentials routed through `oya-shared-secret-reference`; `oya-check-*` flat namespace |
| `oya-check-no-credentials-in-substrate` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `no-credentials-in-substrate` | Fitness-check; static analysis; verifies substrate crates contain no hardcoded credentials or secrets; `oya-check-*` flat namespace |
| `oya-check-multi-modal-transport-coverage` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `multi-modal-transport-coverage` | Fitness-check; verifies multi-modal transport BCs declared day-one per §D-5; `oya-check-*` flat namespace |
| `oya-check-caller-side-rag-only` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `caller-side-rag-only` | Fitness-check; verifies Intelligence substrate does not implement server-side RAG (caller-side only per §D-7); `oya-check-*` flat namespace |
| `oya-check-audience-tag-on-every-call` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `audience-tag-on-every-call` | Fitness-check; verifies every LLM/embedding call carries audience context tag per §D-15; `oya-check-*` flat namespace |
| `oya-check-foundry-bc-absorption-complete` | self-layering (ADR-0105 Amendment 2) | `oya` · `check` · `foundry-bc-absorption-complete` | Fitness-check; verifies all Foundry provider/guardrail/eval BCs absorbed into AI Substrate per §D-16; `oya-check-*` flat namespace |

---

*End of ADR-0255.*
