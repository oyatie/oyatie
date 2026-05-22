---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITOR-PARITY
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence + gtm-strategy
related_adrs: [ADR-0255]
doc_status: published
---

# Competitor Parity Matrix — intelligence µservice

## Purpose

Compare oyatie's intelligence substrate against the canonical AI-API competitors. Identifies parity
floor (table-stakes), parity ceiling (industry-leading), and oyatie-differentiation surface
(library-first + Cedar refusal + audit-tap + provider-credential BYOK per ADR-0255 §D-4 + cell + EU AI Act day one).

## Competitor universe

| Competitor | Class | Coverage |
|---|---|---|
| OpenAI Platform API | direct provider API | text + vision + audio + tool use + structured |
| Anthropic API | direct provider API | text + vision + tool use + cache |
| Google AI Studio | direct provider API | text + vision + audio + video + grounding |
| Google Vertex AI | enterprise direct provider | as above + Gemini-specific + Imagen + Veo |
| AWS Bedrock | model gateway | many providers + enterprise IAM |
| Azure OpenAI | enterprise direct provider | OpenAI models + Microsoft IAM |
| Cohere | direct provider API | text + rerank + embeddings |
| Mistral La Plateforme | direct provider API | text + code (Codestral) + agent |
| Together AI | hosted open-weight | many models |
| Replicate | hosted any model | many |
| OpenRouter | aggregator | unified API across many |
| Groq | low-latency LPU | open-weight models |
| HuggingFace Inference | hosted | many models + Spaces |

## Capability matrix

| Capability | Oyatie intelligence | OpenAI | Anthropic | Google AI Studio | Vertex AI | Bedrock | Azure OpenAI | Cohere | Mistral | Together | Replicate | OpenRouter | Groq | HF Inference |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Text in / text out | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Vision in | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | n/a | n/a | ✓ | ✓ | ✓ | n/a | varies |
| Audio in | ✓ | ✓ | n/a | ✓ | ✓ | ✓ | ✓ | n/a | n/a | varies | varies | varies | n/a | varies |
| Video in | ✓ (flag) | n/a | n/a | ✓ | ✓ | n/a | n/a | n/a | n/a | n/a | varies | n/a | n/a | varies |
| Streaming (SSE) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | webhook+SSE | ✓ | ✓ | ✓ |
| Streaming (WebSocket) | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Tool use / function calling | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Structured output / JSON mode | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | varies | varies | varies | ✓ | varies |
| Prompt caching | passthrough | n/a | ✓ | ✓ | ✓ | ✓ | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Multi-provider routing | ✓ canonical | n/a (single provider) | n/a | n/a | n/a | ✓ (within Bedrock) | n/a | n/a | n/a | ✓ (within Together) | ✓ (within Replicate) | ✓ canonical | n/a | n/a |
| Library-first SDK (in-process) | ✓ canonical | n/a (HTTP only) | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| HTTP/3 + QUIC default | ✓ per ADR-0253 | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial | partial |
| PQ-hybrid TLS (Kyber768 + X25519) | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Audit-tap (per-call sealed audit) | ✓ ADR-0263 | n/a | n/a | partial (Cloud Logging) | partial | partial (CloudTrail) | partial (Azure Monitor) | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| EU AI Act Annex III refusal layer | ✓ day-one | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Per-pack refusal-floor overlay | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| provider-BYOK (tenant brings keys) | ✓ canonical via ADR-0255 §D-4 | n/a | n/a | n/a | n/a | partial (IAM) | partial (Microsoft IAM) | n/a | n/a | n/a | n/a | partial | n/a | n/a |
| Sidecar credential-handle (creds never in process memory) | ✓ ADR-0296 | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Per-tenant cost cap | ✓ | n/a | partial | partial | ✓ (project quota) | partial | ✓ (PTU) | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| FinOps tenant attribution | ✓ canonical via audit-tap | partial | partial | partial | partial | partial | partial | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Citation attribution | ✓ canonical | partial (Search retrieval annotations) | partial (Sources) | ✓ (Grounded gen) | ✓ (Vertex Search) | partial | partial | partial | partial | n/a | n/a | n/a | n/a | n/a |
| Brand-ux-surface (consumer-AI chrome) | ✓ canonical | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Cell-eligible (shuffle-sharded tenant cells) | ✓ ADR-0248 | n/a | n/a | n/a | n/a | partial | partial | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| provider-credential BYOK platform-default disambiguation (ADR-0255 §D-4) | ✓ canonical | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Cedar policy enforcement | ✓ canonical | n/a | n/a | n/a | n/a | n/a (IAM) | n/a (RBAC) | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Open-weight self-hosted dispatch | ✓ (vLLM/SGLang/TRT-LLM) | n/a | n/a | n/a | partial (Model Garden) | n/a | n/a | n/a | n/a | ✓ | ✓ | ✓ | n/a | ✓ |

## Parity gap analysis

### Where oyatie matches table-stakes

Modality coverage, streaming, tool use, structured output, prompt caching passthrough — fully at
parity with the dominant providers.

### Where oyatie leads

| Differentiator | Why it matters | Citation |
|---|---|---|
| Library-first dispatch | In-process latency advantage; cross-process boundary eliminated for in-cluster callers | ADR-0255 amendment |
| Per-call audit-tap (Ed25519 sealed) | Enables EU AI Act Art. 12 record-keeping out of the box | ADR-0263 |
| EU AI Act Annex III refusal layer day-one | First substrate to ship this; competitors require tenant-side implementation | EU AI Act Art. 16 |
| provider-credential BYOK + sidecar credential-handle (ADR-0255 §D-4) | Credentials never enter substrate process memory | ADR-0296 |
| Per-pack refusal-floor overlay | Localised to KR / EU / US / CN / etc. | dpia.md |
| Cell-eligibility | Tenant isolation at infrastructure tier | ADR-0248 |
| Cedar policy on every dispatch | Policy is centrally manageable + auditable | ADR-0243 |
| 16-provider matrix | Eliminates lock-in; competitors are single-provider or single-aggregator | ADR-0255 §"Provider catalog" |
| HTTP/3 + QUIC default + PQ-hybrid TLS | Future-proofed for post-quantum | ADR-0253 |

### Where competitors lead (gaps to close)

| Gap | Competitor leader | Closure plan |
|---|---|---|
| Native fine-tuning lifecycle | OpenAI fine-tuning, Anthropic Fine-Tuning (preview), Google Vertex tuning | Separate µservice `intelligence-fine-tuning` per ADR-0255 §D; not closing in this µservice |
| Native embeddings + vector store | OpenAI Embeddings, Cohere Embed, Voyage AI | Separate µservice `intelligence-embeddings` per ADR-0255 §D |
| Realtime API (low-latency voice) | OpenAI Realtime API, Google Live API | Phased rollout via streaming (SSE + WebSocket); GA in PHASE-02 successor |
| Native browsing tool | OpenAI browsing | Caller-side (substrate stays library-first; caller assembles tools) |
| Native code-execution sandbox | OpenAI code-interpreter, Anthropic computer-use | Out of scope; tenant uses oyatie sandbox µservice |

## Pricing parity

Per-MT pricing in `cost-budget.md` matches the provider's public pricing at pass-through (no
oyatie markup on direct-provider routing; markup only on platform-default routing where oyatie
absorbs cost float).

## References

- ADR-0255 — Intelligence as two-layer AI Substrate.
- `microservices/intelligence/PRD.md`.
- `microservices/intelligence/cost-budget.md`.
- Competitor docs: docs.anthropic.com, platform.openai.com/docs, ai.google.dev/gemini-api/docs,
  cloud.google.com/vertex-ai/docs, docs.aws.amazon.com/bedrock, learn.microsoft.com/en-us/azure/ai-services/openai,
  docs.cohere.com, docs.mistral.ai, docs.together.ai, replicate.com/docs, openrouter.ai/docs,
  console.groq.com/docs, huggingface.co/docs/api-inference.
