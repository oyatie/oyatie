---
id: ADR-0355
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - ops-sre-reliability
  - ops-compliance
  - axis-intelligence
  - axis-foundry
  - axis-policy-engine
  - axis-tenancy
  - axis-audit-chain
supersedes: []
amends:
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
superseded_by: []
related:
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0050-event-bus-kafka.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0136-intelligence-as-single-microservice.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0249-multi-category-marketplace-doctrine.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/intelligence.json
  - /specs/byok-credential-model.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_bominal_inheritance_precedence
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_workflow_objectgraph_adapter_layer
  - feedback_autonomous_decision_principles
  - feedback_no_silent_regression
doc_class: Architecture-Decision-Record-Amendment
keystone_bundle: 2026-05-20-foundational-doctrine-amendments
amendment_anchor: F-ANTI-1
enforcement_status: advisory-until-intelligence-client-library-lands
enforced_by:
  - oya gate validate intelligence-library-first-default
  - oya gate validate intelligence-network-opt-in-cedar-gated
  - oya gate validate no-unnecessary-intelligence-service-hop
  - oya gate validate library-only-failure-perimeter
  - oya gate validate library-cedar-gate-coherence
  - oya gate validate library-audit-emission-coherence
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Keep Rejected: Duplicate library-first network-opt-in amendment

# ADR-0355: Amendment — Library-First / Network-Opt-In Clarification

## Status

Proposed — 2026-05-20.

This is an **amendment** to ADR-0255 (Intelligence as Two-Layer AI
Substrate, 2026-05-20). It does not supersede ADR-0255; it clarifies
the **delivery shape** and **runtime call topology** of the Intelligence
Substrate so the substrate does not, by accident, re-introduce the
universal-mediator pattern that ADR-0145 retired.

The amendment is filed as a **Tier-1 lockdown** because the defect it
prevents (the Intelligence µservice becoming the platform-wide LLM
gateway) is structurally identical to the pre-ADR-0145 universal-
mediator shape that PR #143's idea-refine review identified as the #1
twelve-month regret with ~70% probability. ADR-0255 in its current
text does not explicitly state library-first delivery, and a careful
reader can interpret §D-1's "every Intelligence call traverses [the 8
substrate BCs] in order" as mandating a network hop to
`microservices/intelligence/`. That interpretation must be foreclosed
in writing before any code is written against ADR-0255.

Enforcement is `advisory-until-intelligence-client-library-lands`. CI
lanes that enforce this amendment promote to BLOCKER once:

1. The `oya-shared-intelligence-client-*` crate family is scaffolded
   per §D-2 below and at least one µservice (Foundry CI agent path) is
   demonstrated to consume the library without making a network call
   to `microservices/intelligence/`.
2. The `oya-check-no-unnecessary-intelligence-service-hop` static
   analysis lane is authored and exercised against the Foundry CI
   agent reference path.
3. The `secret_references` Cedar policy schema includes the
   `network_side_opt_in: bool` per-reference attribute described in
   §D-5.
4. The Intelligence µservice has been re-scoped per §D-4 to **only**
   the cross-cutting coordination concerns (credential pool health,
   global rate-limit budgets, cross-cell observability rollup,
   cost-attribution aggregator). The dispatch path is removed from
   the µservice's runtime surface.
5. ADR-0255 §D-1 frontmatter is annotated with a forward-pointer to
   this amendment so any reader of ADR-0255 lands here before forming
   a runtime-topology mental model.
6. The reference architecture diagram in
   `docs/architecture/intelligence-substrate-runtime-topology.md` is
   re-drawn to show the library-first path as the default and the
   network hop as a labelled opt-in edge.

Until those six items land, validators emit findings without failing
CI. Post-bootstrap, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### F-ANTI-1: the idea-refine finding that triggered this amendment

The 2026-05-20 idea-refine pass over the 14-ADR foundational keystone
bundle surfaced finding **F-ANTI-1**:

> ADR-0145 (2026-05-18) retired the universal-mediator pattern in
> favor of direct service-to-service gRPC/HTTP with three weaker
> invariants (audit, tracing, ontology projection). ADR-0255 (the
> Intelligence two-layer substrate, drafted 2026-05-20) introduces a
> µservice that, by the natural reading of §D-1's call-traversal
> language, sits in the synchronous data path of every LLM call made
> by every µservice in every cell. If that natural reading is the
> implemented topology, ADR-0255 has re-introduced the
> universal-mediator pattern under a different label. The substrate
> would become the platform SLO ceiling for AI-mediated functionality,
> and the cell-level failure perimeter for any AI-dependent µservice
> would expand to include Intelligence's availability. This is exactly
> the anti-pattern ADR-0145 retired.

The finding is not a defect in ADR-0255's *intent*. ADR-0255's intent
is to consolidate provider adapters, guardrails, credential
resolution, audit emission, cost attribution, and Cedar evaluation in
one place so they are not re-implemented per-caller. That consolidation
is sound. The defect is in the unspecified **delivery shape**: ADR-0255
does not say whether that consolidation is delivered as a *library that
callers link* or as a *µservice that callers RPC*. The natural reading
of "every call traverses the 8 BCs" is the µservice-RPC reading.

This amendment forecloses the µservice-RPC reading as the default and
establishes the library-link reading as the canonical delivery shape.

### What ADR-0145 actually said

ADR-0145 (Inter-microservice communication reform, 2026-05-18) made
three structural decisions:

1. **Audit invariant (decentralized).** Every state-changing call
   emits an audit-chain seal at the *calling* service. No central
   mediator owns the seal.
2. **Tracing invariant.** Every call propagates OTel context. No
   central mediator owns the trace.
3. **Ontology projection invariant.** Ontology is a SUBSTRATE for
   cross-µservice read queries, not a GATEWAY. µservices may also
   call each other directly via mTLS gRPC.

ADR-0145 also explicitly states:

> AWS, Google, Microsoft, Stripe, Anthropic do NOT use a universal
> mediator pattern. They use direct service-to-service gRPC/HTTP
> with mTLS + per-service contracts.
>
> Putting orchestration in the data path (ESB 2.0) makes Workflow the
> platform SLO ceiling and failure perimeter.

The same reasoning applies to AI/LLM calls. Putting Intelligence in
the data path of every LLM call makes Intelligence the platform SLO
ceiling and failure perimeter for AI-mediated functionality. If the
Intelligence µservice is down or degraded, every AI-mediated feature
is down or degraded — even though the underlying LLM provider
(Anthropic, OpenAI, Google, etc.) is fully available and the calling
µservice has a Cedar-validated SecretReference resolving to a valid
credential. The Intelligence µservice has no business being a single
point of failure on a path where it has nothing to contribute beyond
what a library could contribute in-process.

### The risk that motivated the amendment

If ADR-0255 is implemented under the µservice-RPC reading without this
amendment, the predictable failure modes are:

1. **SLO ceiling.** Every AI-mediated feature inherits Intelligence's
   availability. If Intelligence is 99.9% available, no AI-mediated
   feature can be more than 99.9% available — even if the provider is
   99.99%. The platform's stated quality bar (hyperscaler-grade,
   `feedback_quality_performance_scalability_bar`) becomes unreachable
   for the AI surface.
2. **Failure perimeter.** A regional outage of `microservices/intelligence/`
   in cell X cascades to every AI-mediated µservice in cell X. The
   blast radius of a single µservice rollback / bad-config / poisoned-
   dependency / queue-saturation expands to the entire AI surface.
3. **Latency tax.** Every LLM call adds a network round-trip
   (calling-µservice → Intelligence → provider → Intelligence →
   calling-µservice). At cell-internal latency budgets (~2-10 ms one
   way over the mesh), the tax is small relative to provider latency
   (~500-5000 ms), but it compounds over multi-turn agentic loops
   (Foundry CI agent runs that issue 20-50 LLM calls per task pay 40
   to 250 ms of pure mediator latency per task).
4. **Capacity coupling.** Intelligence's capacity becomes the platform
   capacity for AI calls. Sizing errors at Intelligence become
   sizing errors everywhere. The hyperscaler shape that ADR-0145
   established (per-µservice capacity, independent scaling) collapses
   for AI workloads.
5. **Observability inversion.** The natural span hierarchy for an
   LLM call is `caller → provider`. Inserting Intelligence makes it
   `caller → Intelligence → provider`, which is an artificial extra
   span that adds no information (the Cedar evaluation and audit
   emission are equally observable in a library span).
6. **Distributed monolith.** Intelligence's evolution becomes coupled
   to every consumer. Schema changes, retry-policy changes,
   guardrail changes require coordinated deploys across N callers.
   ADR-0212's buildability doctrine and `feedback_autonomous_implementation_artifacts`
   (everything must build to green without coordination) become
   infeasible.

These six failure modes are exactly the modes ADR-0145 §Context cited
as the reason to retire the universal-mediator pattern. Re-introducing
them under the Intelligence label is not acceptable.

### What library-first means in this codebase

The pattern is not novel. It is the same pattern used by:

- `oya-shared-policy-engine-client` (per ADR-0150 + ADR-0243). Cedar
  policy evaluation runs in-process in every µservice via the
  shared client library. The Cedar µservice exists for fragment
  authoring, distribution, and fleet observability — not for
  per-call evaluation.
- `oya-shared-audit-chain-client` (per ADR-0145 Invariant 1). Audit
  seal emission runs in-process in every µservice. The audit-chain
  µservice exists for canonical seal storage and Merkle-tree
  consensus — not for per-call mediation.
- `oya-shared-tracing-client` (per ADR-0145 Invariant 2). OTel
  context propagation runs in-process. Tempo exists for storage and
  query, not for per-call mediation.
- `oya-shared-secret-reference` (per ADR-0255 §D-4). SecretReference
  resolution is a library call against OpenBao; OpenBao itself is a
  µservice but per-call resolution is not mediated through a
  separate "secrets gateway".

Intelligence joins this list. The library does the per-call work; the
µservice does the cross-cutting work that genuinely needs central
state.

### What the AWS Bedrock reference actually shows

ADR-0255 §Context cites AWS Bedrock as a hyperscaler reference. The
citation is correct but the runtime topology was not made explicit.
AWS Bedrock is **invoked by AWS workloads via the AWS SDK** — that is,
via an in-process client library that holds adapter logic, retry
policy, request signing, and observability emission. The AWS SDK does
not RPC to a separate "Bedrock gateway µservice" inside the calling
workload's VPC. The Bedrock service itself runs in AWS's control plane
and is the destination of the SDK's signed HTTPS call. The pattern is:

```
caller (in-process AWS SDK) ──HTTPS──→ bedrock.<region>.amazonaws.com
                                       ↑
                                       provider endpoint
```

Not:

```
caller ──RPC──→ in-VPC bedrock-gateway ──HTTPS──→ bedrock.<region>.amazonaws.com
```

The library-first / network-opt-in shape this amendment establishes is
the same shape. The Intelligence µservice plays the role of an
in-cell coordinator for cross-cutting state (credential pool, rate
budgets, cost rollup), not the role of a gateway through which every
LLM call flows.

### Hamilton's static stability principle

James Hamilton's 2007 LISA talk "On Designing and Deploying Internet-
Scale Services" (continued in the AWS Builder's Library 2018
"Avoiding insurmountable queue backlogs" and 2020 "Static stability
using Availability Zones") formalized the principle that **a system
should continue to function statically when its coordinator is
unavailable**. Applied here: when the Intelligence µservice is down,
LLM calls should still flow — because the library in the caller's
process has everything it needs (provider adapter, retry logic,
Cedar policy, SecretReference resolution against OpenBao, audit
emission, OTel propagation) to make the call. The Intelligence
µservice's absence should degrade *cross-cutting* observability and
coordination (cost rollup is delayed; credential pool health is
stale), not break the actual call path.

This is the AZ-static-stability pattern transplanted to the
intra-cell control plane: the data path does not depend on the
control path being up.

## Decision

### D-1. Intelligence Substrate is library-first

The Intelligence Substrate is **delivered as a library by default**.
The canonical entry point is the `oya-shared-intelligence-client-*`
crate family. Every caller (every µservice, every Foundry workflow,
every Ontology Function invocation, every Workflow Studio step that
issues an LLM call) links the library and calls the library's
in-process API.

The library, not a µservice, is the user-visible surface of
Intelligence for the dispatch path.

The eight AI Substrate bounded contexts established in ADR-0255 §D-2
(transport, credential-resolver, policy-engine-client, guardrails,
audit-emit, tool-registry, audience-policy-router, cost-attribution)
are implemented as **in-process Cargo crates** that the library
composes. They are not RPC endpoints by default. ADR-0255's claim
that "every Intelligence call traverses the 8 BCs in order" is
preserved — but the traversal is in-process composition of crates, not
network mediation through a µservice.

The library is the unit of consumption. Each BC's crate is the unit of
authorship.

### D-2. What the library performs in-process

The `oya-shared-intelligence-client-*` library performs **all** of the
following work in the caller's own process:

| Concern | In-process responsibility |
|---|---|
| Provider adapter normalization | Translate the caller's normalized request into the provider-specific request shape (OpenAI chat completions, Anthropic Messages API, Google Generative AI, AWS Bedrock InvokeModel, Azure OpenAI completions, vLLM OpenAI-compatible, etc.). Handle response normalization back. |
| Retry logic | Per-provider retry policy with jittered exponential backoff. Honor `Retry-After`. Distinguish retryable (5xx, 429 with budget) from non-retryable (4xx other than 429, content policy refusals). |
| Circuit breaker | Per-provider, per-cell, per-credential circuit breaker (closed → open → half-open). Open on consecutive failures above threshold; recover via half-open probing. Hedged requests for high-latency P99 budgets per AWS Builder's Library "Avoiding insurmountable queue backlogs" pattern. |
| Cedar gate | In-process call to `oya-shared-policy-engine-client` (per ADR-0150 + ADR-0243) to evaluate the Cedar request for the Intelligence action (LlmDispatch / EmbeddingGenerate / ImageGenerate / AudioTranscribe / ToolCall / etc.). Receive Permit/Forbid/NotApplicable. Halt the call on Forbid. |
| Audit emission | In-process call to `oya-shared-audit-chain-client` (per ADR-0145 Invariant 1) to seal an `IntelligenceDispatch` (and any `IntelligenceGuardrailFired`, `IntelligenceCredentialResolved`, `IntelligenceCostAttributed`, `IntelligenceToolCall`) row. **Per ADR-0296 §D-2 (Wave-3-A amendment): the audit-signing key MUST NOT reside in the caller process's memory. The audit-signing key is held exclusively by the `oyatie.intelligence.credential-sidecar` co-located sidecar process; the library calls the sidecar over a Unix Domain Socket (UDS) to request signing, receives the signed envelope, and emits it. The key never crosses the UDS boundary as plaintext.** The seal is emitted by the caller's process (via the sidecar), not by the Intelligence µservice. |
| OTel propagation | In-process call to `oya-shared-tracing-client` (per ADR-0145 Invariant 2). The span hierarchy is `caller → llm-provider`, with the library's work as child spans of `caller`. No artificial `caller → intelligence-mediator → llm-provider` insertion. |
| SecretReference resolution | **Per ADR-0296 §D-2 (Wave-3-A amendment): provider credentials MUST NOT be cached in the caller process beyond a single in-flight call. Two conformant implementations are permitted: (a) credential-sidecar UDS surface — the `oyatie.intelligence.credential-sidecar` sidecar holds all provider credentials; the library requests a per-call credential handle over UDS; the handle is a short-lived token (≤60s) that the sidecar issues and can revoke; the raw credential never enters the caller process. (b) ≤60s OpenBao token TTL — the library fetches the credential from OpenBao with a TTL ≤60s; the credential is held in-process only for the duration of the single outbound call; no credential is persisted to any cache. RCE in the caller process can therefore expose credentials for at most 60s. In-process credential caching beyond one call lifetime is PROHIBITED.** Intelligence is not in the credential-resolution path. |
| Guardrail execution | Pre-call (prompt-injection detection, PII redaction, prompt-policy-conformance) and post-call (PII leak detection, jailbreak success detection, toxic-content refusal) guardrails run in-process via library-bundled detector modules. Heavy detectors (LLM-as-judge) MAY hit a sibling-µservice for inference, but that sibling is itself another caller of the same library against its own provider credential — not a centralized guardrail gateway. |
| Tool-registry lookup | In-process tool-registry lookup via library-bundled registry snapshot. The registry snapshot is refreshed periodically from the Intelligence µservice's read-only registry endpoint (sub-second freshness is not required; the registry is a slow-moving object). |
| Cost computation | In-process per-call cost computation against the library-bundled rate-card snapshot. Cost row is emitted via the in-process audit-emit path; the cost-attribution aggregator (network-side, per §D-4) consumes the audit-chain stream. |

All ten concerns happen in the caller's process. No synchronous
network hop to `microservices/intelligence/` is required for any of
them on the default path.

### D-3. The library calls the provider directly via HTTPS

After the library has performed its in-process work (provider adapter
selection, Cedar gate green, credential resolved, guardrails passed,
audit row emitted, OTel span opened), the actual outbound call goes
**directly from the caller's process to the LLM provider's endpoint
over HTTPS**.

The call topology is:

```
caller process
  │
  │ (in-process)
  │  oya-shared-intelligence-client-* library
  │   ├─ provider adapter         (in-process)
  │   ├─ retry / circuit breaker  (in-process)
  │   ├─ Cedar gate               (in-process via shared policy lib)
  │   ├─ SecretReference resolve  (in-process via shared secret lib;
  │   │                             OpenBao fetch over network when not cached)
  │   ├─ guardrails pre           (in-process)
  │   ├─ audit-emit               (in-process via shared audit lib)
  │   └─ OTel span open           (in-process)
  │
  │  HTTPS (mTLS-not-required; provider's TLS)
  │
  ▼
provider endpoint
  (api.anthropic.com / api.openai.com / generativelanguage.googleapis.com
   / bedrock-runtime.<region>.amazonaws.com / <azure>.openai.azure.com
   / vllm-self-hosted.<cell>.svc / ...)
```

There is **no network hop to `microservices/intelligence/`** on this
path. The Intelligence µservice is not in the request flow.

This is the **default** path. It is what 100% of LLM calls take unless
the caller has explicitly opted in to the network-side coordination
features described in §D-4 + §D-5.

### D-4. The Intelligence µservice exists for cross-cutting state only

`microservices/intelligence/` continues to exist, but its **runtime
responsibility surface is restricted to cross-cutting state that
genuinely cannot live in the caller's process**. Specifically:

| Cross-cutting concern | Why it cannot live in the library | Network-side responsibility |
|---|---|---|
| Shared credential health pool | Per-credential rate-limit budget consumption is a shared state across all callers using the same SecretReference. If two callers each have local-only budget tracking, they will collectively exceed the provider's published rate limit. | Maintain per-credential token-bucket / leaky-bucket budgets in a shared store (Redis/KeyDB per cell). Expose a `BudgetCheckOut` / `BudgetCheckIn` gRPC for opted-in callers. |
| Global rate-limit budgets | Same logic at the cell-wide aggregate (multi-tenant fair-share). Anti-noisy-neighbor enforcement requires shared visibility. | Per-cell aggregate budget enforcement; cross-tenant fair-share queues; brown-out signal emission (per ADR-0176). |
| Cross-cell observability rollup | Per-call audit rows emit at the caller; cross-cell aggregate views (fleet-wide LLM spend by provider by tenant by audience) require a rollup process. | Subscribe to the audit-chain stream; emit aggregate rows + dashboards. Read-only consumer of audit-chain; not a per-call participant. |
| Cost-attribution aggregator | Per-call cost rows emit at the caller; tenant-scoped FinOps rollups (per ADR-0242 §D-7 deepest-declared-sub-scope) require aggregation. | Subscribe to the audit-chain stream's `IntelligenceCostAttributed` rows; aggregate to FinOps portal. Read-only consumer. |
| Provider adapter registry distribution | Adapter binaries are versioned; new providers are added by deploying new adapter versions to the library. Coordinating which version is canonical requires a central registry. | Maintain the canonical adapter version manifest. Library polls / pulls. Not in the call path. |
| Tool-registry distribution | Tool definitions (Ontology Functions, MCP server references, internal capabilities) are central; library snapshots refresh from here. | Maintain the canonical tool-registry. Library polls / pulls. Not in the call path. |
| Eval runner orchestration | Multispectrum-review eval batches and golden-set evaluations are batch workloads orchestrated centrally. | Run eval batches against library-mediated provider calls. |
| Brand-surface back-end APIs (Layer B per ADR-0255 §D-3) | Consumer Brand Surface BCs (prompt-history, consent-cascade, dsar-cascade, eu-ai-act-tier-ui, tenant-admin-console-controls, brand-ux-surface) are persistent server-side surfaces consumed by tenant consoles + consumer UIs. | These BCs continue to be served by the Intelligence µservice over its HTTP/gRPC interface as today. They are not on the per-call dispatch path. |

None of the eight concerns above is on the **synchronous per-call LLM
dispatch path** for the default caller. They are control-plane and
batch concerns. The runtime topology preserves the static-stability
property: when the Intelligence µservice is unavailable, the per-call
dispatch path continues to function (degraded only in the cross-cutting
sense — cost rollup is stale; brown-out enforcement is local-only).

### D-5. Callers opt in to network-side Intelligence per Cedar policy

Most callers default to the library-only path. A caller that **needs**
network-side coordination opts in per call (or per credential, or per
audience tag) via Cedar policy. The opt-in is explicit, gated, and
audited.

The opt-in surface is two-fold:

**Per-SecretReference opt-in.** The `secret_references` table (ADR-0255
§D-4) gains an attribute:

```sql
ALTER TABLE secret_references
    ADD COLUMN network_side_opt_in BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE secret_references
    ADD COLUMN network_side_opt_in_reasons TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];
    -- enum-validated subset of:
    --   'shared-credential-pool'
    --   'cross-cell-session-state'
    --   'global-rate-limit-budget'
    --   'cost-attribution-realtime'
    --   'eu-ai-act-realtime-tier-enforcement'
```

When `network_side_opt_in = TRUE`, the library performs a
`BudgetCheckOut` against the Intelligence µservice before dispatching
the call (and a `BudgetCheckIn` after the response, including
unused-tokens reconciliation). When `FALSE`, the library uses
local-only budget tracking.

**Per-call Cedar opt-in.** A Cedar fragment in `intelligence-policy/`
governs the opt-in decision:

```cedar
permit (
    principal,
    action == Intelligence::Action::"NetworkSideCoordination",
    resource is SecretReference
) when {
    resource.network_side_opt_in == true &&
    resource.network_side_opt_in_reasons.contains(context.coordination_reason)
};
```

The caller submits `context.coordination_reason` declaring which
cross-cutting concern requires the network side. Cedar evaluates;
permit allows the library to make the network-side RPC; forbid means
the library proceeds local-only.

This per-policy opt-in keeps the network side as an *explicit*
escalation, not an *implicit default*. The hyperscaler shape that
ADR-0145 established is preserved: direct calls are the default;
coordination is opt-in.

### D-6. Default is library-only; network-side is opt-in

The library defaults to **local-only** for every cross-cutting concern
listed in §D-2 unless the per-SecretReference + per-Cedar opt-in
gate explicitly permits otherwise.

The default's properties:

1. The Intelligence µservice has zero per-call network traffic from
   the caller.
2. The Intelligence µservice's availability does not bound the
   caller's availability.
3. The caller's latency budget does not include an Intelligence
   round-trip.
4. The caller's failure perimeter does not include the Intelligence
   µservice.

Opt-in's properties (for the small subset of callers that opt in):

1. The library performs a `BudgetCheckOut` (and `BudgetCheckIn`) RPC
   before/after the provider call.
2. The RPC is on the synchronous path; the caller's latency budget
   includes one extra round-trip per call.
3. The caller's failure perimeter includes the Intelligence µservice
   *for the budget concern*. If the budget RPC fails open (Cedar
   policy decides), the caller proceeds with local-only budget; if
   it fails closed, the caller fails the LLM call.

The opt-in decision is per-SecretReference (i.e., per-credential), so
a tenant's high-volume credential that approaches the provider's
rate limit can opt in for fair-share coordination, while the same
tenant's low-volume internal credentials stay local-only.

### D-7. ADR-0145 alignment statement

ADR-0145 established the doctrine: **Intelligence is a SPECIALIZED
concern with a SPECIALIZED library, NOT a universal mediator.**

This amendment makes that doctrine explicit for Intelligence. The
Intelligence library is parallel in shape to:

- The `oya-shared-policy-engine-client` library (Cedar). The Cedar
  µservice does not mediate every policy evaluation; the library
  does the evaluation in-process.
- The `oya-shared-audit-chain-client` library. The audit-chain
  µservice does not mediate every seal; the library emits seals
  directly.
- The `oya-shared-tracing-client` library. Tempo does not mediate
  every span; the library propagates and emits OTel directly.
- The `oya-shared-secret-reference` library. The cloud-secrets
  µservice does not mediate every fetch beyond the OpenBao
  request itself; the library handles caching and resolution.

Intelligence joins this family. The library is the per-call surface;
the µservice is the coordination + storage surface.

The ADR-0145 three invariants (audit emission, OTel propagation,
Ontology projection) apply unchanged. The library satisfies all three
in-process at the caller. The Intelligence µservice does **not**
emit audit seals on behalf of the caller (the caller emits its own
seals, per Invariant 1).

### D-8. SLO + failure perimeter consequences

The library-first default produces three operational properties:

1. **SLO ceiling is removed.** The platform-wide SLO for AI-mediated
   functionality is bounded by the provider's SLO + the caller's own
   SLO, not by Intelligence's SLO. If the provider is 99.99%
   available and the caller is 99.95% available, the composed SLO
   for the AI-mediated feature is ~99.94% — independent of
   Intelligence's own SLO.
2. **Failure perimeter contracts.** When the Intelligence µservice is
   down, the per-call LLM dispatch path continues to function. The
   degradation is limited to: cost rollup is stale (the audit-chain
   stream still emits; the aggregator catches up when Intelligence
   recovers); brown-out enforcement is local-only (per-caller
   token-bucket budgets are not coordinated across callers); the
   tool-registry snapshot is whatever the library last refreshed.
   None of these block the dispatch.
3. **Latency tax is removed.** The default path adds zero network
   hops over the pre-ADR-0255 baseline. The library composes the
   work in-process at sub-millisecond cost, which is well within the
   noise of provider latency (~500-5000 ms typical for LLM calls).

These three properties are required to honor the hyperscaler-bar
quality target (`feedback_quality_performance_scalability_bar`) for
AI-mediated functionality.

## Alternatives

### Alternative 1 — Keep ADR-0255 as-is (status quo without this amendment)

**Description.** Accept ADR-0255's call-traversal language at face
value. Implement the eight AI Substrate BCs as RPC endpoints on
`microservices/intelligence/`. Every LLM call from every µservice
issues a gRPC call to Intelligence, which then performs the eight
BCs' work and issues the outbound HTTPS call to the provider.

**Pros.**

1. Single binary to test and operate for the dispatch surface.
2. Provider adapter version pinning is centralized (no library
   version skew across callers).
3. Brown-out enforcement is naturally cell-wide because all calls
   pass through Intelligence.
4. Easier to retrofit new cross-cutting concerns (just add another
   BC inside the µservice; callers don't need to upgrade a library).

**Cons.**

1. **Re-introduces the universal-mediator anti-pattern.** ADR-0145
   §Context cites this as the #1 12-month regret with ~70%
   probability. Re-introducing it under the Intelligence label does
   not change the pathology.
2. **SLO ceiling.** Every AI-mediated feature's availability is
   bounded by Intelligence's availability.
3. **Failure perimeter.** A regional Intelligence outage cascades to
   every AI-mediated feature in the cell.
4. **Distributed monolith.** Intelligence evolution becomes
   coordinated with every caller; ADR-0212's buildability and
   `feedback_autonomous_implementation_artifacts` become infeasible.
5. **Latency tax** on every call, compounding over multi-turn
   agentic loops (Foundry CI agent ≈ 20-50 LLM calls per task →
   40-250 ms of pure mediator latency per task).
6. **Capacity coupling.** Intelligence becomes the platform capacity
   for AI, defeating the per-µservice independent-scaling property
   ADR-0145 established.

**Rejected.** This is exactly the anti-pattern ADR-0145 retired. The
defect ADR-0145 closed in PR #143 is the same defect this alternative
would re-open.

### Alternative 2 — Full µservice-only with sidecar mitigation

**Description.** Implement all eight BCs as RPC endpoints on the
Intelligence µservice (as in Alternative 1), but co-locate an
Intelligence sidecar in every caller's pod to absorb the network hop
locally. The sidecar runs the eight BCs in a separate process; the
caller talks to the sidecar over localhost.

**Pros.**

1. Eliminates the cross-pod network hop's latency.
2. The sidecar's failure perimeter is per-pod, not cell-wide.
3. Provider adapter version pinning remains centralized (sidecar
   image is the central artifact).

**Cons.**

1. **Sidecar memory + CPU tax on every pod.** A sidecar replicating
   the eight BCs' working set across N caller pods is wasteful
   relative to a library that shares the address space.
2. **Two-process complexity.** The caller process and the sidecar
   process must remain in lockstep on schema version, credential
   refresh, tool-registry refresh, etc. Library-in-process avoids
   the IPC entirely.
3. **Still operationally a mediator.** The sidecar pattern is
   structurally a service-mesh-style mediator; debugging a Cedar
   evaluation failure means looking at the sidecar's logs, not the
   caller's process. The Linus-style "no silent regression"
   (`feedback_no_silent_regression`) doctrine penalizes the
   diagnostic indirection.
4. **Re-introduces universal-mediator pattern, just per-pod.** The
   sidecar version becomes the platform-wide LLM dispatch
   bottleneck — an upgrade requires rolling the sidecar across all
   AI-mediated pods.

**Rejected.** A sidecar is a worse library. The mitigation cost is
larger than the consolidation benefit, and the diagnostic complexity
violates the `feedback_no_silent_regression` doctrine.

### Alternative 3 — Library-first, network-opt-in (CHOSEN)

**Description.** Per §D-1 through §D-8 above. The library is the
default; the µservice exists for cross-cutting coordination; opt-in
is per-SecretReference + per-Cedar.

**Pros.**

1. **Aligns with ADR-0145.** Direct calls are the default;
   coordination is opt-in. Same shape as Cedar / audit-chain /
   tracing client libraries.
2. **SLO ceiling removed.** AI-mediated feature SLO is bounded by
   provider + caller, not by Intelligence.
3. **Failure perimeter contracts.** Intelligence µservice outage
   does not block dispatch.
4. **No latency tax** on the default path.
5. **Cross-cutting concerns retained.** Credential pool health,
   rate-limit budgets, cost rollup remain centralized where they
   genuinely need to be.
6. **Hyperscaler-shape parity.** AWS SDK + Bedrock pattern;
   Anthropic SDK + API pattern; OpenAI SDK + API pattern. Library
   in caller's process; network call directly to provider.
7. **`feedback_autonomous_implementation_artifacts` preserved.**
   Library version pinning per-caller means caller upgrades are
   independent; the build-everything-to-green property is
   maintained.

**Cons.**

1. **Library version skew across callers.** Mitigated by the
   workspace-wide single-version policy (`feedback_no_silent_regression`
   + Cargo workspace `[workspace.dependencies]` pinning) and by the
   library being designed so version-N+1 reads version-N's audit
   rows and Cedar contexts without break.
2. **Provider adapter authoring discipline required.** New
   provider adapters land as library updates, which must be
   rolled across all callers. Mitigated by ADR-0212 buildability +
   the registry-refresh path being non-call-path.
3. **Brown-out enforcement requires opt-in.** Callers with
   high-volume credentials must opt in to network-side budget
   coordination, which is one extra Cedar policy authoring step.
   Acceptable because the policy is a one-time per-credential
   declaration.

**Accepted.** This is the shape that honors ADR-0145, preserves
hyperscaler-bar SLO, and retains the cross-cutting consolidation
ADR-0255 intended.

## Consequences

### Positive

1. **ADR-0145 universal-mediator retirement is preserved.** The risk
   identified in F-ANTI-1 is closed in writing before any code is
   authored against ADR-0255.
2. **Hyperscaler-bar SLO for AI-mediated functionality is reachable.**
   Provider SLO + caller SLO compose without Intelligence as a third
   multiplicand.
3. **Static stability per Hamilton 2007.** The data path does not
   depend on the control path being up. Intelligence µservice can be
   under maintenance, rolling-restarted, or in a regional outage
   without blocking dispatch.
4. **Latency budget unchanged from baseline.** The default path is
   in-process composition + direct provider HTTPS; no additional
   network round-trip is introduced relative to a hypothetical
   no-Intelligence baseline.
5. **Diagnostic locality.** When an LLM call fails, the failure is
   visible in the caller's OTel span hierarchy as `caller →
   provider`, with library-internal child spans showing Cedar gate,
   guardrail decisions, retry attempts. No artificial
   `caller → Intelligence → provider` indirection.
6. **Per-µservice scaling preserved.** ADR-0145's "no platform SLO
   ceiling" + "µservices scale independently" applies to AI calls.
7. **Cost rollup remains centralized** via the audit-chain stream
   subscription pattern. Tenant-scoped FinOps reports are unchanged.
8. **Cross-cutting concerns remain consolidated** in the
   Intelligence µservice where they genuinely need central state.

### Negative

1. **Library version pinning discipline.** Workspace-wide pinning
   already exists; this amendment makes the discipline binding for
   `oya-shared-intelligence-client-*`. Operators must roll library
   updates across the workspace per ADR-0212 buildability cadence.
2. **Adapter authoring cadence.** New provider support requires a
   library release. Mitigated because adapter additions are
   ~weekly, not ~daily, and the registry-refresh path (non-call-path)
   handles the metadata side.
3. **Opt-in policy authoring.** Tenants with high-volume credentials
   must author Cedar opt-in fragments. The default for new
   credentials is local-only, which preserves the "no surprise
   coupling" principle (`feedback_no_silent_regression`).

### Operational

1. **Authoring sequence.** `oya-shared-intelligence-client-*` crates
   are scaffolded *before* the Intelligence µservice's dispatch
   surface is implemented. The µservice's runtime surface is scoped
   to §D-4 from day one. If the µservice ever needs a per-call
   dispatch endpoint, that endpoint must carry an explicit ADR
   amending this amendment.
2. **CI lane authoring.** The
   `oya-check-no-unnecessary-intelligence-service-hop` lane scans
   for unconditional `IntelligenceClient::dispatch_via_service(...)`
   calls (or equivalent RPC client invocations) that are not gated
   by a per-SecretReference `network_side_opt_in` check. Failures
   block merge.
3. **Documentation updates.** ADR-0255 §D-1 gains a forward-pointer
   to this amendment in its frontmatter and its prose. The
   reference architecture diagram at
   `docs/architecture/intelligence-substrate-runtime-topology.md`
   is re-drawn to show library-first default + opt-in network edge.
4. **Brown-out signal authoring.** The brown-out signal per ADR-0176
   is emitted by the *library* (local-only on the default path) and
   by the *µservice* (cell-wide aggregate, opt-in callers only).
   The two signals do not conflict; the local signal trips first;
   the global signal trips on opted-in callers when the cell-wide
   aggregate is hot.
5. **Cell-µservice load is unchanged.** Per ADR-0148 Cilium Service
   Mesh, mTLS handshakes still happen for the µservice-to-µservice
   surfaces (audit-chain client, secret-reference client, etc.) but
   the LLM provider call is over the provider's own TLS to the
   provider's public endpoint, which does not consume cell-µservice
   SPIFFE issuance budget.
6. **Service-mesh egress.** The provider endpoint (HTTPS to
   `api.anthropic.com`, etc.) is egress through the cell's egress
   gateway with FinOps tagging per ADR-0174 + ADR-0148. The mesh
   NetworkPolicy permits this egress per the per-cell allow-list.
7. **Multi-cell deployment.** Each cell ships the library to its
   callers and an Intelligence µservice for its own cross-cutting
   state. Cross-cell coordination (global FinOps rollup) happens at
   the audit-chain stream layer, not via Intelligence µservice
   federation.
8. **Failure-mode runbook.** A new runbook at
   `docs/operators/intelligence-substrate-failure-modes.md`
   enumerates: provider outage (library retries; circuit breaks;
   caller surfaces a 503 to its own caller); credential expired
   (library detects; caller surfaces 401 reason); cell-local
   Intelligence µservice down (library proceeds local-only;
   cost rollup is stale; alarms on the µservice itself); cell-wide
   network partition to provider (library circuit-breaks per
   provider; failover provider attempted per Cedar fragment).

## Implementation surface

### Library crates (workspace `crates/`)

The library is delivered as a family of crates rather than a single
mega-crate, so callers can depend only on the surfaces they use:

| Crate | Layer (per ADR-0105) | Responsibility |
|---|---|---|
| `oya-shared-intelligence-client-domain` | domain | Caller-facing types: `LlmRequest`, `LlmResponse`, `Modality`, `AudienceTag`, `ProviderFamily`, `IntelligenceError`. Pure types; no I/O. |
| `oya-shared-intelligence-client-kernel` | kernel | Trait `IntelligenceClient` with the async dispatch surface. Generic over provider adapter, Cedar client, audit client, secret-reference client. Pure trait; no concrete adapter. |
| `oya-shared-intelligence-client-app` | app | Default composition of provider adapter + retry + circuit breaker + Cedar gate + guardrails + audit emission + OTel propagation. The library's default `IntelligenceClient` impl. |
| `oya-shared-intelligence-client-providers-anthropic` | adapter | Anthropic Messages API adapter. |
| `oya-shared-intelligence-client-providers-openai` | adapter | OpenAI chat completions + responses + embeddings adapter. |
| `oya-shared-intelligence-client-providers-google` | adapter | Google Generative AI adapter. |
| `oya-shared-intelligence-client-providers-bedrock` | adapter | AWS Bedrock InvokeModel adapter. |
| `oya-shared-intelligence-client-providers-azure-openai` | adapter | Azure OpenAI adapter. |
| `oya-shared-intelligence-client-providers-mistral` | adapter | Mistral adapter. |
| `oya-shared-intelligence-client-providers-cohere` | adapter | Cohere adapter (re-ranking + generation). |
| `oya-shared-intelligence-client-providers-vllm` | adapter | vLLM OpenAI-compatible self-hosted adapter. |
| `oya-shared-intelligence-client-providers-sglang` | adapter | SGLang self-hosted adapter. |
| `oya-shared-intelligence-client-providers-tensorrt-llm` | adapter | TensorRT-LLM self-hosted adapter. |
| `oya-shared-intelligence-client-providers-apple-fm` | adapter | Apple Foundation Models API adapter. |
| `oya-shared-intelligence-client-providers-openrouter` | adapter | OpenRouter aggregator adapter. |
| `oya-shared-intelligence-client-providers-together` | adapter | Together AI adapter. |
| `oya-shared-intelligence-client-providers-groq` | adapter | Groq adapter. |
| `oya-shared-intelligence-client-guardrails` | adapter | Bundled guardrail detectors (heuristic + pluggable LLM-as-judge dispatch back through the library). |
| `oya-shared-intelligence-client-tool-registry` | adapter | Library-side tool-registry snapshot cache + refresh worker. |
| `oya-shared-intelligence-client-cost` | adapter | Library-side rate-card snapshot + per-call cost computation. |
| `oya-shared-intelligence-client-network-opt-in` | adapter | Optional crate for callers that opt in to network-side coordination. Wraps `BudgetCheckOut` / `BudgetCheckIn` against the Intelligence µservice. |

Each provider adapter crate is independently version-pinned in the
workspace. Callers depend on `oya-shared-intelligence-client-app` and
the specific provider adapters they need.

### Opt-in µservice surface (`microservices/intelligence/`)

The µservice retains the Consumer Brand Surface BCs (Layer B in
ADR-0255 §D-3) unchanged. The AI Substrate Layer A surface narrows to
the cross-cutting concerns per §D-4:

| µservice endpoint | Purpose | Caller-facing path |
|---|---|---|
| `POST /v1/budgets/check-out` | Opt-in caller checks out budget for a planned call. | Opt-in only. Caller library calls when SecretReference.network_side_opt_in = TRUE. |
| `POST /v1/budgets/check-in` | Opt-in caller returns unused budget after the call. | Opt-in only. Same gating. |
| `GET /v1/adapter-registry/manifest` | Library polls for canonical adapter version manifest. | Non-call-path; periodic refresh. |
| `GET /v1/tool-registry/snapshot` | Library polls for tool-registry snapshot. | Non-call-path; periodic refresh. |
| `GET /v1/rate-cards/snapshot` | Library polls for rate-card snapshot. | Non-call-path; periodic refresh. |
| `POST /v1/brand-surface/prompt-history/*` | Layer B prompt-history APIs. | Consumer-audience only; tenant console + consumer UI. |
| `POST /v1/brand-surface/consent-cascade/*` | Layer B consent APIs. | Consumer-audience only. |
| `POST /v1/brand-surface/dsar-cascade/*` | Layer B DSAR APIs. | Consumer-audience only. |
| `POST /v1/brand-surface/eu-ai-act-tier-ui/*` | Layer B tier-UI APIs. | Consumer-audience only. |
| `POST /v1/brand-surface/tenant-admin-console-controls/*` | Layer B tenant-admin APIs. | Tenant admin only. |
| `POST /v1/brand-surface/brand-ux-surface/*` | Layer B brand UX APIs. | Consumer-audience only. |
| `GET /v1/admin/observability/cost-rollup` | Cross-cell cost rollup (read-only consumer of audit-chain stream). | FinOps portal + tenant admin. |
| `GET /v1/admin/observability/credential-health` | Per-credential health rollup. | Tenant admin. |

The dispatch endpoint that would have existed under the natural
reading of ADR-0255 §D-1 (`POST /v1/dispatch`) is **deliberately
absent**. No endpoint of that shape ships. If a future requirement
genuinely needs centralized dispatch, that requirement amends this
amendment with a fresh ADR.

### Cedar fragments (`policy/fragments/intelligence/`)

Three Cedar fragments govern the library + µservice boundary:

1. `intelligence-library-dispatch.cedar` — governs the in-process
   dispatch decision (provider routing per data class + jurisdiction
   + tier).
2. `intelligence-network-side-opt-in.cedar` — governs the opt-in
   decision per §D-5. Permit on
   `resource.network_side_opt_in == true` and
   `resource.network_side_opt_in_reasons` containing the declared
   reason.
3. `intelligence-brand-surface-access.cedar` — governs Layer B BC
   access per ADR-0255 §D-3 (unchanged).

## Verification

### CI lanes

Six advisory-until-bootstrap lanes promote to BLOCKER after the
items in §Status are complete:

1. **`oya-check-intelligence-library-first-default`** (static
   analysis). Scans the workspace for any caller that issues an
   `intelligence` µservice RPC on the dispatch path without a
   corresponding per-SecretReference `network_side_opt_in = TRUE`
   declaration. Flags violations.
2. **`oya-check-intelligence-network-opt-in-cedar-gated`** (static
   analysis). Confirms that every network-side RPC site is gated by
   a Cedar evaluation of `intelligence-network-side-opt-in.cedar`
   with a declared `context.coordination_reason`. Flags ungated
   RPCs.
3. **`oya-check-no-unnecessary-intelligence-service-hop`**
   (integration test). Runs the Foundry CI agent reference workflow
   end-to-end against a mock provider and asserts that zero gRPC
   calls hit `microservices/intelligence/` on the dispatch path.
   Asserts that `BudgetCheckOut` is invoked only when the test
   fixture's SecretReference has `network_side_opt_in = TRUE`.
4. **`oya-check-library-only-failure-perimeter`** (chaos test).
   Brings down `microservices/intelligence/` in a test cell;
   asserts that LLM dispatch through the library continues to
   succeed for default callers; asserts that opted-in callers fail
   the `BudgetCheckOut` step and either fall back per Cedar policy
   or fail closed per declared policy.
5. **`oya-check-library-cedar-gate-coherence`** (unit + integration
   test). Confirms that the library's in-process Cedar gate
   evaluates the same fragments the µservice would have evaluated;
   asserts no policy drift between library and µservice.
6. **`oya-check-library-audit-emission-coherence`** (unit +
   integration test). Confirms that the library emits the
   `IntelligenceDispatch`, `IntelligenceGuardrailFired`,
   `IntelligenceCredentialResolved`, `IntelligenceCostAttributed`,
   `IntelligenceToolCall` audit rows from the *caller's* process
   (verified by checking the audit-chain seal's emitting principal
   matches the caller's SPIFFE-ID, not Intelligence's SPIFFE-ID).

### Manual verification gates

1. The reference architecture diagram at
   `docs/architecture/intelligence-substrate-runtime-topology.md`
   shows the library-first path as the default solid edge and the
   network hop as a dashed opt-in edge.
2. The ADR-0255 frontmatter has been annotated with a forward
   pointer to this amendment.
3. The `secret_references` migration has shipped the
   `network_side_opt_in` column and the
   `network_side_opt_in_reasons` array column.
4. The Foundry CI agent's reference workflow is documented in
   Appendix B as the canonical worked example.

## References

- **ADR-0145** — Inter-microservice communication reform (2026-05-18).
  Retires the universal-mediator pattern; establishes three weaker
  invariants (audit, tracing, ontology projection); direct
  service-to-service is the default. This amendment applies that
  doctrine to Intelligence.
- **James Hamilton 2007 LISA** — "On Designing and Deploying Internet-
  Scale Services." Formalizes static stability and the principle
  that the data path must not depend on the control path being up.
- **AWS Builder's Library** — "Avoiding Cascading Failures" (2019)
  and "Avoiding insurmountable queue backlogs" (2020). Document
  the SLO ceiling + failure perimeter pathology that arises when a
  central coordinator sits on the data path; establish circuit
  breaker + hedged requests + static stability as the canonical
  mitigations.
- **AWS Builder's Library** — "Static stability using Availability
  Zones" (2020). Same principle at the AZ scope; this amendment
  transplants it to the intra-cell control-plane scope.
- **Martin Fowler 2014** — "Microservices and the First Law of
  Distributed Object Design." Critique of the ESB 2.0 anti-pattern;
  argues against centralized smart mediators in favor of dumb pipes
  + smart endpoints. ADR-0145 already cites; this amendment
  re-cites for the Intelligence specialization.
- **IEEE 2017** — "Why Enterprise Service Buses Failed." Empirical
  retrospective on ESB deployments; documents the SLO ceiling +
  failure perimeter + distributed monolith failure modes.
- **AWS SDK + Amazon Bedrock product page (2024)** — Bedrock is
  consumed via the AWS SDK in-process; no in-VPC mediator stands
  between caller and Bedrock control plane.
- **Azure AI Foundry product page (2024)** — Foundry endpoints
  consumed via per-deployment direct invocation; Entra ID tenant
  boundary handles audience scoping.
- **Anthropic SDK (2024)** — Per-process Anthropic client library;
  direct HTTPS to `api.anthropic.com`. The reference shape this
  amendment adopts.
- **OpenAI SDK (2024)** — Per-process OpenAI client library; direct
  HTTPS to `api.openai.com`. Same shape.
- **Apple Foundation Models API (WWDC 2024)** — On-device + private-
  cloud-compute substrate consumed via per-app framework, not via
  a system-wide gateway.
- **ADR-0150** — Cedar policy engine. The shared client library
  pattern for Cedar is the precedent this amendment extends to
  Intelligence.
- **ADR-0176** — Brown-out degradation signal. Library emits local;
  µservice emits cell-wide aggregate on opt-in.
- **ADR-0211** — In-house tech stack preference. OpenBao for
  SecretReference resolution; library calls OpenBao directly.
- **ADR-0212** — Buildability doctrine. Library version pinning
  preserves the build-to-green property across callers.
- **ADR-0244** — Tenant as universal scoping primitive. The
  SecretReference's tenant scope determines opt-in eligibility.
- **ADR-0247** — Self-hosting / self-modification doctrine.
  `oyatie.foundry.*` workflows make LLM calls through the library
  on the default path, exactly as customer-tenant workflows do.
- **`feedback_quality_performance_scalability_bar`** — Hyperscaler-
  grade performance + horizontal scalability. The library-first
  default is required to honor this bar for AI-mediated
  functionality.
- **`feedback_no_silent_regression`** — Linus-style protection of
  public contracts. The amendment's `oya-check-no-unnecessary-
  intelligence-service-hop` lane is the CI enforcement; the per-
  SecretReference `network_side_opt_in` column is the contract
  surface.
- **`feedback_autonomous_implementation_artifacts`** — Long-term
  goal that "Implement the masterplan" runs without coordination.
  Library version pinning per-caller preserves this.

## Appendix A — Hyperscaler-pattern attribution

The library-first / network-opt-in shape is not novel. It is the
canonical pattern across every hyperscaler reference cited in the
ADR-0255 + ADR-0145 corpus. The attribution is explicit so that
future readers do not re-derive the pattern by trial and error.

| Reference | Library | Network coordination | Shape |
|---|---|---|---|
| **AWS Bedrock** (2024) | AWS SDK in caller's process; provider adapter + retry + signing + observability. | Bedrock control plane in AWS (not in caller's VPC). | Library-first; provider endpoint is in AWS's control plane, accessed via the caller's SDK over HTTPS. |
| **Azure AI Foundry** (2024) | Azure SDK / Foundry SDK in caller's process. | Foundry control plane in Azure. | Library-first; same shape. |
| **GCP Vertex AI** (2024) | Google Cloud client libraries in caller's process. | Vertex AI control plane in GCP. | Library-first; same shape. |
| **Apple Foundation Models** (WWDC 2024) | Per-app framework in caller's process (on-device + PCC). | PCC orchestration in Apple's control plane. | Library-first; on-device runs entirely local; PCC is a fallback opt-in. |
| **Stripe API client** (2024) | Stripe SDK in caller's process. | Stripe API. | Library-first; no in-VPC Stripe mediator. |
| **Anthropic API** (2024) | Anthropic SDK in caller's process. | `api.anthropic.com`. | Library-first; the reference Intelligence emulates. |
| **OpenAI API** (2024) | OpenAI SDK in caller's process. | `api.openai.com`. | Library-first; same shape. |
| **Cedar v4.2** (per ADR-0150) | `oya-shared-policy-engine-client` in caller's process. | Cedar µservice for fragment authoring + distribution. | Library-first; the precedent for this amendment. |
| **OpenTelemetry** (per ADR-0145 Invariant 2) | OTel SDK in caller's process. | Tempo for storage. | Library-first; Tempo is not on the call path. |
| **SPIFFE/SPIRE** (per ADR-0148) | SPIRE agent on each node + caller library. | SPIRE server for issuance. | Library-first; per-call SPIFFE-ID is from the agent's cached SVID. |
| **`oya-shared-audit-chain-client`** (per ADR-0145 Invariant 1) | Library in caller's process. | Audit-chain µservice for canonical Merkle storage. | Library-first; the audit µservice is the storage tier, not a mediator. |

The convergence is unambiguous. Every reference at the hyperscaler
bar uses the library-first / network-opt-in shape. Intelligence
joins the pattern.

## Appendix B — Worked example: Foundry CI agent calling Anthropic

This appendix walks through a single LLM call from
`oyatie.foundry.ci-agent` (per ADR-0247 self-hosting / self-
modification doctrine) to Anthropic's Messages API. The example is
the canonical reference path the
`oya-check-no-unnecessary-intelligence-service-hop` lane asserts
against.

### Setup

- **Caller principal:** `oyatie.foundry.ci-agent` (per ADR-0242
  tenant doctrine; `oyatie` is the platform-owner tenant).
- **Audience tag:** `oyatie-self-modification` (per ADR-0255 §Context
  audience table).
- **Cell:** `cell-eu-west-1` (Tier 3 data-plane cell).
- **SecretReference:** `secret_reference_slug = "anthropic-prod-2026q2"`,
  `owner_kind = oyatie-platform-subscription`,
  `network_side_opt_in = FALSE` (default).
- **Provider:** Anthropic.
- **Model:** `claude-opus-4-7` (or current canonical per ADR-0255
  §D-2 transport BC's provider-adapter manifest).
- **Cedar fragments active:** `intelligence-library-dispatch.cedar`,
  `intelligence-brand-surface-access.cedar` (latter not invoked for
  this audience tag).

### Step-by-step

1. **Caller constructs the request.** The Foundry CI agent's
   business logic (reviewing a PR, drafting an ADR fragment,
   running a multispectrum-review facet) builds an `LlmRequest`
   from `oya-shared-intelligence-client-domain` with the prompt,
   modality (`Modality::Text`), audience tag
   (`AudienceTag::OyatieSelfModification`), tenant scope
   (`oyatie.foundry`), principal (`oyatie.foundry.ci-agent`), and
   SecretReference slug (`anthropic-prod-2026q2`).
2. **Library opens an OTel span.** `oya-shared-tracing-client`
   opens a child span under the caller's existing span:
   `intelligence.dispatch`. Span attributes: provider, model,
   audience tag, modality.
3. **Library evaluates Cedar gate.** The library calls
   `oya-shared-policy-engine-client` in-process with action
   `Intelligence::Action::"LlmDispatch"`, resource `Provider::"anthropic"`,
   context `{ audience_tag, data_class, jurisdiction, cell,
   modality }`. Cedar returns `Permit` with determining policy
   `intelligence-library-dispatch.cedar::permit-self-modification-anthropic-eu-west-1`.
4. **Library resolves SecretReference.** `oya-shared-secret-reference`
   fetches the credential from `microservices/cloud-secrets/`
   (OpenBao) at `secret://oyatie/anthropic-prod-2026q2`. The fetch
   is a single mTLS gRPC call from the caller's process to
   `cloud-secrets`. No Intelligence µservice hop.
5. **Library checks `network_side_opt_in`.** `FALSE`. The library
   skips the `BudgetCheckOut` RPC. Local-only budget tracking
   applies: the library's in-process token bucket for
   `anthropic-prod-2026q2` is decremented.
6. **Library runs pre-call guardrails.** Bundled detectors check
   the prompt for injection, PII, and policy conformance. All
   pass. The library emits an `IntelligenceGuardrailFired`
   audit row (status: passed; detectors: 4 fired, 4 passed) via
   `oya-shared-audit-chain-client` from the caller's process.
7. **Library composes the Anthropic Messages API request.** The
   `oya-shared-intelligence-client-providers-anthropic` adapter
   normalizes the `LlmRequest` to the Messages API JSON shape.
   Adds `x-api-key` header from the resolved credential. Adds
   `anthropic-version: 2023-06-01` (or current). Adds OTel
   propagation headers per W3C Trace Context.
8. **Library issues HTTPS POST to `api.anthropic.com`.**
   Direct HTTPS from the caller's process. Anthropic's TLS.
   Egress through the cell's egress gateway with FinOps tagging
   per ADR-0174. **No traffic hits `microservices/intelligence/`.**
9. **Anthropic responds.** Streaming SSE response. The library's
   adapter normalizes the response back into `LlmResponse` chunks
   and yields them to the caller's stream consumer.
10. **Library runs post-call guardrails.** Bundled detectors check
    the response for PII leak, jailbreak success, toxic content.
    All pass.
11. **Library emits per-call audit rows.** From the caller's
    process: `IntelligenceDispatch` (provider, model, tokens-in,
    tokens-out, duration), `IntelligenceCredentialResolved`
    (SecretReference, owner_kind), `IntelligenceCostAttributed`
    (cost rows per the library's bundled rate-card snapshot;
    cost center is `oyatie.foundry` per ADR-0242 §D-7
    deepest-declared-sub-scope).
12. **Library closes the OTel span.** Span ends with status
    `OK`. Child spans (Cedar gate, secret resolution, guardrails
    pre, guardrails post, anthropic.messages.create) are nested
    under `intelligence.dispatch`.
13. **Caller receives the response.** Foundry CI agent processes
    the response in its own logic.

### What did NOT happen

- No gRPC call from the caller's process to
  `microservices/intelligence/`.
- No `IntelligenceDispatch` audit seal emitted by Intelligence's
  SPIFFE-ID (the seal was emitted by `oyatie.foundry.ci-agent`'s
  own SPIFFE-ID; the seal's `emitting_principal` field reflects
  this).
- No artificial `intelligence-mediator` span between
  `oyatie.foundry.ci-agent` and `anthropic.messages.create` in the
  OTel trace.
- No SLO contribution from Intelligence's availability to the
  caller's per-call success budget.

### What happened separately (asynchronously)

- The cost-attribution aggregator (per §D-4) subscribes to the
  audit-chain stream and ingests the `IntelligenceCostAttributed`
  row from step 11. The FinOps portal updates `oyatie.foundry`
  spend for the quarter. This ingestion is asynchronous; it does
  not block the per-call path.
- The credential-health rollup (per §D-4) sees the
  `IntelligenceCredentialResolved` row and updates the per-credential
  usage counter for `anthropic-prod-2026q2`. If the counter approaches
  the published Anthropic rate limit, an operator alert fires; an
  operator MAY change `network_side_opt_in` to `TRUE` for that
  credential to engage cell-wide budget coordination.
- The cross-cell observability rollup ingests the
  `IntelligenceDispatch` rows and updates fleet-wide dashboards.

### Hypothetical opt-in variant

If the same call were made by a high-volume tenant with
`network_side_opt_in = TRUE` and
`network_side_opt_in_reasons = ['shared-credential-pool']`:

- Between step 5 (skip / proceed with budget) and step 6 (run
  guardrails), the library would add: `BudgetCheckOut` RPC to
  `microservices/intelligence/` requesting a per-call token budget
  for the SecretReference. Intelligence's shared store
  (Redis/KeyDB per cell) responds with `granted: true, budget: N
  tokens` or `granted: false, retry_after: M ms`.
- After step 9 (Anthropic responds), the library adds:
  `BudgetCheckIn` RPC returning unused tokens back to the shared
  pool.
- Both RPCs are subject to Cedar gating per
  `intelligence-network-side-opt-in.cedar` with
  `context.coordination_reason = 'shared-credential-pool'`.
- If `microservices/intelligence/` is down at step 5b, the
  library's per-credential fallback policy (declared in the
  SecretReference's Cedar fragment) determines whether to
  fail-open (proceed local-only with stale budget) or fail-closed
  (return error to caller). For most credentials, fail-open with
  local-only budget is the appropriate policy: the credential's
  rate limit is large enough that local-only tracking is safe for
  the duration of Intelligence's outage.

### Why this matters

This worked example demonstrates that the canonical AI-mediated
workflow in the platform (the Foundry CI agent reviewing a PR or
drafting an ADR) does **not** require `microservices/intelligence/`
to be up. The platform's self-modification capability — the property
that `oyatie.foundry.*` workflows can autonomously implement the
masterplan per `feedback_autonomous_implementation_artifacts` — is
preserved across Intelligence µservice outages, upgrades, and
schema migrations. That is the static-stability guarantee Hamilton
2007 prescribed and that ADR-0145 codified for the platform's
inter-µservice surface. This amendment extends the same guarantee to
the AI surface.

---

## Change log

- **2026-05-20 (Wave-3-A cross-reference wiring):** Applied §D-2 caller-process scope reduction per ADR-0296:
  - Audit-signing key: MUST NOT reside in caller process memory; held exclusively by `oyatie.intelligence.credential-sidecar` sidecar via UDS; library calls sidecar for signing.
  - Provider credentials: MUST NOT be cached in caller process beyond a single in-flight call; two conformant paths — (a) credential-sidecar UDS with ≤60s handle TTL, or (b) ≤60s OpenBao token TTL; in-process credential caching beyond one call lifetime prohibited.
  - Cross-reference: ADR-0296.

*End of amendment.*
