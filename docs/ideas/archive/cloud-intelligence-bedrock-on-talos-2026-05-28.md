---
title: "cloud-intelligence: Bedrock-on-Talos pattern as a cloud primitive"
status: superseded
superseded_by: ADR-0389
date: 2026-05-28
companion_docs:
  - cloud-intelligence-v1-pipeline-2026-05-28.md
  - n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md
---

# cloud-intelligence: Bedrock-on-Talos pattern as a cloud primitive

**Status**: ideation artifact (2026-05-28).
**Companion docs**:
- `cloud-intelligence-v1-pipeline-2026-05-28.md` — v1 request pipeline (8 stages)
- `n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md` — proof + visibility surfaces

## Problem Statement

Oyatie runs its own Talos-based substrate (ADR-0378). Cloud-intelligence v1 ships a pure OAuth-pool proxy (Anthropic + OpenAI + Gemini passthrough). The open question is: how do we position the Bedrock Converse / InvokeModel surface as a **cloud primitive** — i.e., an abstraction that any tenant or internal workload can call without knowing which underlying provider fulfils it — while keeping the substrate fully self-hostable and hyperscaler-equivalent?

The naive answer ("just proxy all providers") conflates two different concerns: (a) the routing + pooling + cost-control layer (cloud-intelligence v1) and (b) the capability-port abstraction that insulates callers from provider lock-in. This idea-pager argues for the Bedrock-on-Talos pattern: treat `invoke` as the capability port, run AWS Bedrock-compatible endpoints via the cloud-intelligence gateway on Talos, and defer cross-provider transparent failover to v2.

## Recommended Direction

### Positioning

```
                  ┌─────────────────────────────────────────┐
                  │         cloud-intelligence gateway        │
                  │  (ADR-0384 kernel + v1 pipeline ADR-NNNN) │
                  │                                           │
  invoke ────>│  Bedrock-compat surface (v2+)            │
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

**Phased delivery**:

- **v1** (current lane): pure provider passthrough only. No Bedrock-compat surface. OAuth-pool kernel (ADR-0384) + 8-stage pipeline (see companion doc). Talos deploy via ArgoCD.
- **v1.5**: SSE streaming on the passthrough surface. Codex OAuth adapter.
- **v2**: Bedrock Converse / InvokeModel compat surface on Talos. `invoke` capability port. Cross-provider transparent failover via Application Inference Profiles (AWS pattern, self-hosted equivalent).
- **v3+**: Cedar-everywhere routing math. Sidecar deployment mode. Capability-port abstraction fully replaces direct provider calls for all internal workloads.

### Why Bedrock Converse is the right v2 target

1. **Hyperscaler-equivalent**: AWS Bedrock Converse is the internal interface used by hyperscaler LLM workloads. Implementing it self-hosted means oyatie tenants get the same abstraction hyperscalers use internally.
2. **Provider agnosticism**: Bedrock Converse maps to any backend model (Anthropic Claude, Amazon Titan, Meta Llama, Mistral). Our gateway maps Converse requests to whatever OAuth-pool seat is available, achieving transparent failover without caller changes.
3. **Fully self-hostable**: The Converse API spec is publicly documented. We implement the HTTP surface on Talos using the same Rust/axum stack as v1.
4. **Clean interface**: `invoke` as a capability port means internal services NEVER import provider SDKs directly — they call the gateway. This is the hyperscaler-internal-equivalent of a managed inference service.

### Talos deployment pattern

The cloud-intelligence gateway runs as a Talos workload via ArgoCD ApplicationSet:

```yaml
# microservices/cloud-intelligence/k8s/applicationset.yaml (v2 extension)
# Adds:
#   - bedrock-compat Deployment + Service
#   - shared Secret ref (OpenBao ESO) for provider credentials
#   - HPA on concurrent-requests metric
```

The gateway exposes both surfaces behind a single Istio VirtualService. The Bedrock-compat surface is gated by a feature flag (per-tenant Cedar policy) until v2 is stable.

### Key Assumptions

- [ ] **Bedrock Converse → Anthropic Claude mapping is lossless for the 3 message roles** (user, assistant, system). Validate: write a round-trip test with a real Converse payload mapped to Claude messages API.
- [ ] **SSE streaming on Converse uses Server-Sent Events identical to Anthropic's streaming format.** Validate: review Bedrock streaming spec vs Anthropic's.
- [ ] **Per-tenant Cedar policy can gate the Bedrock-compat surface without performance regression.** Validate: benchmark Cedar evaluation with 2 policy sets loaded.
- [ ] **ArgoCD ApplicationSet can manage both v1 passthrough + v2 Bedrock-compat Deployments from a single ApplicationSet definition.** Validate: dry-run the ApplicationSet expansion.

## Not Doing (and Why)

- **Bedrock Guardrails compat in v2** — provider-side filters are enforced upstream by Anthropic/AWS; duplicating them adds cost with no tenant benefit.
- **InvokeModel (legacy non-Converse API) in v2** — Converse is the forward-looking API; InvokeModel is deprecated by AWS for new models.
- **Sidecar mode in v2** — adds k8s complexity; defer to v3 when we have ≥5 tenants with sidecar requirements.

## Open Questions

- **`invoke` call convention**: should it be a Rust trait (`InvokeGate`) or a plain HTTP client pointed at the gateway? Lean HTTP client (simpler, no trait coupling, gateway is the right abstraction boundary).
- **Converse → provider mapping registry**: where does the model-ID → provider-seat mapping live? Options: (a) Cedar policy, (b) manifest.json per gateway instance, (c) admin API. Lean admin API (dynamic, auditable).
- **v1 → v2 migration**: do existing v1 callers need to change? No — v1 endpoints remain. v2 adds the Bedrock-compat surface alongside.
