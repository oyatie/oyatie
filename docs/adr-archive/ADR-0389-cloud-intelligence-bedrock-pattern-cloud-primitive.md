---
id: ADR-0389
title: "cloud-intelligence: Bedrock-on-Talos pattern as a cloud primitive"
status: Superseded
date: 2026-05-28
authority: founder
owner: council-architecture
planning_impact: true
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0364, ADR-0384, ADR-0387]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0389 — cloud-intelligence: Bedrock-on-Talos pattern as a cloud primitive

## Status

Accepted — 2026-05-28.

## Context

Oyatie runs its own Talos-based substrate (ADR-0378). Cloud-intelligence v1 ships a pure OAuth-pool proxy (Anthropic + OpenAI + Gemini passthrough) via the 8-stage pipeline (ADR-0390). The open question is: how do we position the Bedrock Converse / InvokeModel surface as a **cloud primitive** — i.e., an abstraction that any tenant or internal workload can call without knowing which underlying provider fulfils it — while keeping the substrate fully self-hostable and hyperscaler-equivalent?

The naive answer ("just proxy all providers") conflates two different concerns: (a) the routing + pooling + cost-control layer (cloud-intelligence v1, ADR-0384 + ADR-0390) and (b) the capability-port abstraction that insulates callers from provider lock-in. This ADR formalises the Bedrock-on-Talos pattern: treat `oya-invoke` as the capability port, run AWS Bedrock-compatible endpoints via the cloud-intelligence gateway on Talos, and defer cross-provider transparent failover to v2.

## Goals

1. Define the phased delivery model for the cloud-intelligence surface (v1 → v1.5 → v2 → v3+).
2. Establish `oya-invoke` as the canonical capability port for internal LLM workloads.
3. Specify the Bedrock Converse compat surface as the v2 target and justify it as the hyperscaler-equivalent API.
4. Define the Talos deployment pattern for the Bedrock-compat surface alongside the v1 passthrough surface.

## Non-Goals

- Implementing the Bedrock-compat surface in v1 (deferred to v2).
- Bedrock Guardrails compatibility (provider-side filters enforced upstream).
- InvokeModel (legacy non-Converse API) — deprecated by AWS for new models.
- Sidecar deployment mode (v3+).
- Cross-provider transparent failover in v2 (v2 adds Converse surface; failover in v3+).

## Proposal

### Surface positioning

```
                  ┌─────────────────────────────────────────┐
                  │         cloud-intelligence gateway        │
                  │  (ADR-0384 kernel + ADR-0390 pipeline)   │
                  │                                           │
  oya-invoke ────>│  Bedrock-compat surface (v2+)            │
  (capability     │  ┌──────────────────────────────────────┐│
   port)          │  │ /bedrock/v1/models/{model}:invoke     ││
                  │  │ /bedrock/v1/models/{model}:invokeStream││
                  │  │ /bedrock/v1/converse                   ││
                  │  └──────────────────────────────────────┘│
                  │                                           │
                  │  v1 passthrough surface (current)         │
                  │  ┌──────────────────────────────────────┐│
                  │  │ /anthropic/v1/messages                ││
                  │  │ /openai/v1/chat/completions           ││
                  │  │ /gemini/v1/models/{model}:generateContent││
                  │  └──────────────────────────────────────┘│
                  └─────────────────────────────────────────┘
```

### Phased delivery

| Phase | Scope | Blocked on |
|---|---|---|
| **v1** | Pure provider passthrough only. No Bedrock-compat surface. OAuth-pool kernel (ADR-0384) + 8-stage pipeline (ADR-0390). Talos deploy via ArgoCD. | Nothing (in flight) |
| **v1.5** | SSE streaming on the passthrough surface. Codex OAuth adapter. | v1 ship |
| **v2** | Bedrock Converse / InvokeModel compat surface on Talos. `oya-invoke` capability port. | v1.5 ship + ADR-0390 D-lanes complete |
| **v3+** | Cedar-everywhere routing math. Sidecar deployment mode. Capability-port abstraction replaces direct provider calls for all internal workloads. | v2 stable + ≥5 tenants |

### Why Bedrock Converse is the right v2 target

1. **Hyperscaler-equivalent**: AWS Bedrock Converse is the internal interface used by hyperscaler LLM workloads. Implementing it self-hosted means oyatie tenants get the same abstraction hyperscalers use internally.
2. **Provider agnosticism**: Bedrock Converse maps to any backend model (Anthropic Claude, Amazon Titan, Meta Llama, Mistral). The gateway maps Converse requests to available OAuth-pool seats, achieving transparent failover without caller changes.
3. **Fully self-hostable**: The Converse API spec is publicly documented. The HTTP surface is implemented on Talos using the same Rust/axum stack as v1.
4. **Clean interface**: `oya-invoke` as a capability port means internal services never import provider SDKs directly — they call the gateway. This is the hyperscaler-internal-equivalent of a managed inference service.

### Talos deployment pattern (v2 extension)

The cloud-intelligence gateway runs as a Talos workload via ArgoCD ApplicationSet. The v2 extension adds a Bedrock-compat Deployment + Service alongside the existing v1 Deployment, behind a single Istio VirtualService. The Bedrock-compat surface is gated by a per-tenant Cedar policy until v2 is stable.

The model-ID → provider-seat mapping lives in the cloud-intelligence admin API (ADR-0390 Lane A deliverable), making it dynamic and auditable. Internal services call `oya-invoke` via plain HTTP client pointed at the gateway — no Rust trait coupling.

## Alternatives Considered

| Alternative | Reason rejected |
|---|---|
| Mix passthrough + Bedrock + `oya-invoke` in v1 | Already pushed back; data-gated expansion. Shipping all surfaces simultaneously without tenants to validate against produces unvalidated complexity. |
| InvokeModel (legacy non-Converse API) as v2 target | Deprecated by AWS for new models. Converse is the forward-looking API. |
| WebSocket transport | REST + SSE only in v1/v2. WebSocket adds complexity for marginal UX gain in v1/v2 timelines. |
| Cedar trait coupling for `oya-invoke` | HTTP client is simpler, no trait coupling, gateway is the right abstraction boundary. |

## Cross-Cutting Concerns

- **Hyperscaler lens** (per project memory): Bedrock Converse has active upstream, clean license (Apache 2.0 compatible), is fully self-hostable (public API spec), and is a hyperscaler-internal-equivalent. Passes all four filters.
- **ADR-0131 flat layout**: v2 Bedrock-compat crate lands under `microservices/cloud-intelligence/crates/oya-cloud-intelligence-bedrock-compat/`.
- **ADR-0132 flat packaging**: single-concern crate; no bundle grouping.
- **Dogfood tenancy**: `oyatie-dogfood` tenant traverses the Bedrock-compat surface as a regular tenant; no internal bypass.
- **Data residency**: no tenant PII stored in the routing layer; prompt content flows through only if the invocation-log flag is enabled (per ADR-0390 P4 spec).

## Migration Plan

- v1 endpoints (`/anthropic/v1/messages`, `/openai/v1/chat/completions`, `/gemini/v1/...`) remain unchanged through v2 and v3. Existing callers need no changes.
- Internal workloads that currently call provider SDKs directly should migrate to `oya-invoke` (gateway HTTP client) in v2. Migration is gated on v2 stability + ≥1 tenant validated.
- The v2 Bedrock-compat surface is an additive extension; no breaking changes to existing tenants.

## Open Issues

- [ ] **Bedrock Converse → Anthropic Claude mapping losslessness**: validate with a round-trip test covering all 3 message roles (user, assistant, system).
- [ ] **SSE streaming on Converse**: confirm the streaming format matches Anthropic's Server-Sent Events format or document the delta.
- [ ] **Cedar performance with 2 policy sets loaded**: benchmark Cedar evaluation latency with both v1 passthrough policy + v2 Bedrock-compat policy active.
- [ ] **ApplicationSet dry-run**: confirm a single ApplicationSet can manage both v1 passthrough + v2 Bedrock-compat Deployments.
- [ ] **`oya-invoke` call convention**: finalize as plain HTTP client (lean accepted here); document in the admin API ADR (ADR-0390 Lane A).

## Historical residual from ADR-136 (E3 fold 2026-08-06)

**Title:** ADR-0136-intelligence-as-single-microservice

**Preserved decision gist:** **Foundry is one µservice with six internal bounded contexts.** - The single µservice directory is `microservices/foundry/`. - The six bounded contexts inside it are: `runtime`, `supervisor`, `eval`, `evidence`, `guardrails`, `providers`. - Each BC retains its crate fan-out under BNF v4.1 form `oya-foundry-<bc>-<feature>-<layer>` (ADR-0056). Crate names DO NOT change across this consolidation — only the parent directory changes from `microservices/foundry-<bc>/` to `microservices/foundry/`. - The per-BC content is preserved verbatim at `microservices/foundry/bc-sources/<bc>/` (PRD, PHASE-01, t

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-143 (E3 fold 2026-08-06)

**Title:** ADR-0143-intelligence-per-bc-release-pointer

**Preserved decision gist:** Foundry's six bounded contexts each get an independent release pointer of the form: ``` release/foundry-runtime/<environment> release/foundry-supervisor/<environment> release/foundry-eval/<environment> release/foundry-evidence/<environment> release/foundry-guardrails/<environment> release/foundry-providers/<environment> ``` The foundry **Helm chart** remains a single umbrella chart per ADR-0136. Each BC has its own container image (and its own image tag); the umbrella chart's `values.yaml` references the six BC tags independently. A release of (e.g.) guardrails bumps only the `guardrails.image

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-137 (E3 fold 2026-08-06)

**Title:** ADR-0137-intelligence-bounded-contexts

**Preserved decision gist:** Foundry contains **exactly six bounded contexts**, named below. The BC list is closed and grows only by ADR amendment. ### BC: `runtime` - **Purpose**: hosts the capability invocation hot path — capability executor, session state, invocation lifecycle orchestrator, runtime pod pool, capability registry cache (read-through mirror of the supervisor's canonical registry). - **Owner sub-axis**: axis-foundry-runtime. - **Crate fan-out**: `oya-intelligence-runtime-{capability-executor, session-state, invocation-orchestrator, runtime-pool, capability- registry-cache}-{kernel,domain,usecase,api,adapte

_Source file archived after fold; full body in git history / docs/adr-archive/._
