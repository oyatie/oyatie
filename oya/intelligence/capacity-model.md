---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence + ops-sre-reliability
related_adrs: [ADR-0255, ADR-0252, ADR-0253]
review_cadence: quarterly + on every provider-adapter add
doc_status: published
---

# Capacity Model — intelligence µservice

## Purpose

Define the per-provider QPS / RPM / TPM budgets, per-tenant quotas, cold-start latency model,
streaming-throughput assumptions, and headroom planning rules used by the dispatch substrate.

## Per-provider QPS / RPM / TPM budget

The substrate enforces a client-side token-bucket budget per (provider, region, account-tier).
Budgets are sourced from each provider's published rate limits + the contracted negotiated tier.

| Provider | Tier | QPS (req/s) | RPM (req/min) | TPM (tok/min) | Burst | Notes |
|---|---|---|---|---|---|---|
| Anthropic API | Enterprise | 1000 | 60000 | 80,000,000 | 1500 | Per-org; auto-scales with usage history |
| Anthropic API | Tier-3 | 100 | 6000 | 8,000,000 | 200 | Auto-promotes to Enterprise on spend trigger |
| OpenAI API | Enterprise | 1000 | 60000 | 60,000,000 | 1500 | Per-account; o-series + GPT-5 share TPM |
| OpenAI API | Tier-5 | 500 | 30000 | 30,000,000 | 750 | Lower tier |
| Google AI Studio | Enterprise | 500 | 30000 | 50,000,000 | 750 | Gemini 2.5 Pro/Flash share TPM |
| Vertex AI | Enterprise | 1000 | 60000 | 60,000,000 | 1500 | Per-project; per-region budget |
| AWS Bedrock | Provisioned-throughput | per-MU | per-MU | per-MU model-unit | per-MU | MU sized to traffic |
| Azure OpenAI | PTU | per-PTU | per-PTU | per-PTU | per-PTU | Provisioned-throughput unit |
| Cohere | Production | 1000 | 60000 | 40,000,000 | 1500 | |
| Mistral La Plateforme | Enterprise | 1000 | 60000 | 80,000,000 | 1500 | EU-native |
| vLLM (self-hosted) | per-deployment | per-deployment | per-deployment | per-GPU | per-GPU | Tenant-owned cluster |
| SGLang (self-hosted) | per-deployment | per-deployment | per-deployment | per-GPU | per-GPU | Tenant-owned |
| TensorRT-LLM | per-deployment | per-deployment | per-deployment | per-GPU | per-GPU | Tenant-owned |
| Apple Foundation Models | on-device | per-device | per-device | per-device | per-device | iOS only |
| OpenRouter | Aggregator | 100 | 6000 | varies | 200 | per upstream |
| Together AI | Enterprise | 500 | 30000 | 50,000,000 | 750 | |
| Groq | Enterprise | 500 | 30000 | 40,000,000 | 750 | Very-low-latency LPU |
| HuggingFace Inference | Enterprise | 500 | 30000 | varies | 750 | per-model |
| Replicate | Pro | 100 | 6000 | varies | 200 | per-model |

## Per-tenant quota

| Tenant scope | QPS | RPM | TPM | Daily token cap |
|---|---|---|---|---|
| trial | 5 | 300 | 100,000 | 1,000,000 |
| sandbox | 20 | 1,200 | 500,000 | 5,000,000 |
| production (default) | 200 | 12,000 | 8,000,000 | 100,000,000 |
| production (negotiated) | per contract | per contract | per contract | per contract |
| internal-foundry | 1000 | 60,000 | 60,000,000 | 1,000,000,000 |

Per-tenant TPM > provider TPM is permitted only when the tenant operates provider-credential BYOK with their own provider account and provider tier sufficient to back the quota (ADR-0255 §D-4); the credential-resolver propagates this to the
adapter pool.

## Cold-start latency model

| Provider | Cold-start (no-cache) p50 | Warm (cached connection) p50 |
|---|---|---|
| Anthropic API | 350 ms first-byte | 250 ms |
| OpenAI API | 300 ms first-byte | 200 ms |
| Google AI Studio | 350 ms first-byte | 220 ms |
| Vertex AI | 320 ms first-byte | 220 ms |
| AWS Bedrock | 400 ms first-byte | 280 ms |
| Azure OpenAI | 380 ms first-byte | 240 ms |
| vLLM self-hosted | 200 ms first-byte | 150 ms |
| SGLang | 180 ms first-byte | 140 ms |
| TensorRT-LLM | 160 ms first-byte | 130 ms |
| Groq | 90 ms first-byte | 80 ms |
| Apple Foundation Models | 100 ms first-byte | 80 ms |

Cold-start budget assumption: 5 % cold-call ratio at steady-state; SLO uses the warm p50 +
cold p99 hybrid.

## Streaming-throughput model

Streaming token rate is provider-dependent and modality-dependent.

| Provider × model | Tok/s p50 | Tok/s p99 (warm) |
|---|---|---|
| Anthropic Claude Opus 4.7 | 60 | 80 |
| Anthropic Claude Sonnet 4 | 80 | 110 |
| Anthropic Claude Haiku 4 | 150 | 220 |
| OpenAI GPT-5 | 70 | 100 |
| OpenAI GPT-4.5 | 90 | 130 |
| OpenAI o-series | 50 | 70 (reasoning models slower at first-token) |
| Google Gemini 2.5 Pro | 70 | 100 |
| Google Gemini 2.5 Flash | 130 | 200 |
| Mistral Large | 90 | 130 |
| vLLM Llama-3.3-70B | 30 (1×H100) → 90 (8×H100) | 130 |
| Groq Llama-3.3-70B | 200 | 350 |
| Apple Foundation Models | 25 (on-device) | 35 |

## Modality budget

| Modality | Per-call cost class | Daily cap (per tenant production) |
|---|---|---|
| Text in / text out | base | 100M tokens |
| Image in / text out | 3× base | 50,000 images |
| Audio in / text out | 5× base | 100,000 minutes |
| Video in / text out | 30× base | 10,000 minutes (behind feature flag) |
| Text in / image out | 10× base | 50,000 images |
| Text in / audio out | 10× base | 50,000 audio outputs |
| Text in / video out | 100× base | 1,000 video outputs (behind feature flag) |

## Headroom planning

- Per-provider budget at 80 % steady-state utilisation triggers warning (`oya_intelligence_provider_budget_saturation_total`).
- 90 % utilisation triggers page; runbook `provider-rate-limit-saturation.md`.
- 100 % utilisation triggers automatic fallback per `provider-routing.cedar` to secondary
  provider.

## Cell-eligibility

Per ADR-0248 (Amazon-shape cellular architecture), the intelligence µservice is cell-eligible for
shuffle-sharding across Tier-0..Tier-4. Per-cell capacity assumptions match the per-tenant quota
multiplied by the cell's tenant population (typically 25–100 production tenants per Tier-3 cell).

## References

- ADR-0255, ADR-0252, ADR-0253, ADR-0248.
- `microservices/intelligence/cost-budget.md`.
- `microservices/intelligence/failure-modes.md`.
- `microservices/intelligence/multi-region.md`.
- Provider docs: docs.anthropic.com/en/api/rate-limits, platform.openai.com/docs/guides/rate-limits,
  ai.google.dev/gemini-api/docs/rate-limits, docs.aws.amazon.com/bedrock/latest/userguide/quotas.html,
  learn.microsoft.com/en-us/azure/ai-services/openai/quotas-limits.
