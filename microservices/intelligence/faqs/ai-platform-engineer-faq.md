---
doc_class: FAQ
microservice: intelligence
persona: ai-platform-engineer + ml-engineer + ai-governance-officer
date: 2026-05-20
doc_status: published
---

# AI Platform Engineer FAQ — intelligence

## Why multi-provider routing instead of one provider?

Per ADR-0220 + ADR-0255 §A-4. Three drivers:

1. **No single provider is best at everything**: Claude 3.5 Sonnet leads on reasoning + coding; GPT-4o leads on multilingual + image; Gemini 1.5 Pro leads on long-context (1M tokens); Mistral leads on cost-per-token for European deployments; Llama 3.3 70B self-hosted wins on data-residency + zero per-token cost.
2. **Provider outages**: each of OpenAI, Anthropic, Vertex had multi-hour outages in 2024-2025. Multi-provider routing fails over.
3. **Sovereignty**: KR + CN + EU + US-GovCloud tenants need different provider allowlists. Per-pack routing is mandatory for paid tenant_class sovereign-pack obligations.

The routing decision can be per-task-class (`task_chat: claude-3-5-sonnet`, `task_code: claude-3-5-sonnet`, `task_image: gpt-4o`, `task_embedding: text-embedding-3-large`) configured per tenant. Per-tenant override available.

## What's the provider-credential BYOK model (ADR-0255 §D-4)? When does a tenant want their own provider credentials?

Per ADR-0255 §D-4. BYOK = Bring Your Own Key. Default: oyatie uses negotiated wholesale provider keys (oyatie pays the provider; tenant pays oyatie + a margin). BYOK: tenant supplies their OpenAI / Anthropic / Vertex API key; oyatie routes the call but the tenant's account pays.

Tenants choose provider-credential BYOK (ADR-0255 §D-4) when:

- They have an existing enterprise contract with the provider (e.g., OpenAI Enterprise at $X discount) they want to use.
- They have provider-specific quotas (e.g., GPU reservations on Vertex AI) they want to deploy through oyatie.
- They need provider-account-level compliance attestation (e.g., a specific provider account whitelisted for HIPAA).

Provider-credential BYOK requires per-tenant provider credential storage; we use HSM-backed SecretReferences and sidecar-scoped access. The tenant's provider credential is never logged or exposed outside the provider call (ADR-0255 §D-4).

## What does the EU AI Act Annex III refusal kernel actually do?

Per ADR-0095 + ADR-0143. Annex III of the EU AI Act enumerates high-risk AI systems: credit scoring, employment, education, immigration, law enforcement, critical infrastructure, and several others. Deploying these requires:

- Article 43 conformity assessment (pre-deployment third-party audit) OR self-assessment + EU registration.
- Article 14 human-in-the-loop oversight for irreversible decisions.
- Article 72 continuous monitoring.
- Article 73 serious-incident reporting.

The `intelligence` µservice classifies every request via the high-risk-task classifier kernel. If classified as Annex III high-risk AND the tenant hasn't filed the Article 43 conformity-assessment evidence to `oya-governance-eu-ai-act-attestation` lane, the request is REFUSED at the kernel level (synchronous Cedar gate). The refusal emits `intelligence.high_risk_task.refused` with the task class.

Tenants who have filed conformity-assessment can still execute the high-risk task, but every request emits `intelligence.high_risk_task.classified` with mandatory human-in-the-loop confirmation for irreversible decisions.

## Why Llama 3.3 70B self-hosted and not just route everything to providers?

Per ADR-0255 §A-5. Three drivers:

1. **Data residency**: tenants whose data cannot leave the region (KR PIPA, EU GDPR with consent-restricted models, US-GovCloud) need self-hosted inference. Providers offer regional residency but not air-gap.
2. **Zero per-token cost**: at high volume, self-hosted breaks even vs hosted providers at ~ 50M tokens/month/tenant. Tenants above this threshold save significantly.
3. **Latency**: self-hosted Llama 3.3 70B on H100 SXM5 has p99 ≤ 3 s for 200-token completions. Hosted providers add network + provider-side queuing.

The trade-off: self-hosted requires GPU pool ops. We maintain a fleet of H100 + B200 GPUs for paid tenant_class workloads; tenants share the pool with per-tenant rate limits.

## What's a per-tenant LoRA fine-tune and when does it make sense?

LoRA (Low-Rank Adaptation) is a fine-tuning technique that adds small (rank 8-256) low-rank matrices to a frozen base model. The adapter is ~ 50-500 MiB; training a Llama 3.3 70B LoRA on a 10k-example dataset takes ~ 4-12 hours on 4× H100.

Per-tenant LoRA makes sense when:

- The tenant has > 5 000 examples of domain-specific data (e.g., legal contracts, medical notes, internal customer-service transcripts).
- The tenant needs the model to learn domain-specific terminology + response style that pure prompt engineering can't achieve.
- The tenant's domain has a clear right-answer that prompt-tuning alone can't reach.

We do NOT recommend per-tenant fine-tune for:

- Tenants with < 1 000 examples (overfit risk).
- Tenants who just want a system prompt (use the prompt fence).
- Tenants who want fresh knowledge (use RAG instead; fine-tuning bakes knowledge into weights which becomes stale).

The fine-tune training happens in a separate `intelligence-training` job class; `intelligence` µservice consumes the resulting adapter for inference.

## What's the RAG pipeline and how does it differ from fine-tuning?

RAG (Retrieval-Augmented Generation): at inference time, query a vector store for relevant context, prepend that context to the prompt, let the model answer with citations. Knowledge stays in the vector store; model weights are unchanged.

vs Fine-tuning: knowledge is baked into the model weights via supervised training. At inference, no retrieval.

Use cases:

- **RAG**: knowledge that changes (product docs, policies, recent events). Cited responses. Fast iteration on KB updates.
- **Fine-tuning**: response style, domain terminology, structured-output formatting. Stable across time.

Often both are used together: fine-tune for style + RAG for fresh knowledge.

oyatie's RAG pipeline:

- Chunk ingest (chunk_size default 1000 chars; overlap 200).
- Embedding via BGE-M3 (open-source, 1024-dim) or text-embedding-3-large (OpenAI, 3072-dim).
- Vector store: Qdrant per-tenant collection (paid tenant_class) or Weaviate (alt).
- Retrieval: cosine similarity, top-K (default 5).
- Context-window assembly: rank-by-relevance + diversity-promotion (MMR) + budget-bounded.

## How does ProvenAI watermarking work?

Per ADR-0220 §A-7. Every text response from `intelligence` carries a steganographic watermark that survives most copy-paste + minor edits (truncation, paraphrasing). The watermark encodes: tenant_id, request_id, timestamp.

Method (text): sentence-level token-distribution biasing per Google SynthID-Text. Statistical fingerprint extractable with the public detection key.
Method (image): SynthID-Image (DeepMind 2024 release; we license + use).
Method (audio): SynthID-Audio (DeepMind 2024 release; we license + use).

Detection:

```sh
oya intelligence watermark detect --input ./suspect.txt
# Output: { "watermarked": true, "tenant_id": "acme-corp", "request_id": "req_..." }
```

Cedar enforces: `intelligence::watermark::detect` is unauthorized for tenant-of-tenant principals (only oyatie governance can detect watermarks across tenants; tenant can detect their own).

Watermarking is tenant_class-bound: paid tenants require it; demo_trial exceptions must be explicitly policy-gated when a dev cell disables it for performance.

## What's the per-request audit-chain emission shape?

Per ADR-0220 + ADR-0028:

```json
{
  "event_class": "intelligence.request.submitted",
  "tenant_id": "acme-corp",
  "principal_id": "u-customer-service-bot-42",
  "request_id": "req_01HZX9...",
  "provider": "anthropic",
  "model": "claude-3-5-sonnet-20251022",
  "task_class": "chat",
  "prompt_hash": "blake3:abc123...",
  "prompt_token_count": 1218,
  "rag_kb_id": "acme-product-docs",
  "rag_retrieved_chunk_hashes": ["blake3:def456...", "blake3:111aaa..."],
  "fence_decision": "pass",
  "high_risk_class": null,
  "cedar_decision": "allow",
  "cedar_policy_id": "intelligence_v3",
  "timestamp": "2026-05-20T14:32:17.892Z"
}
```

The response event has shape `intelligence.response.emitted`:

```json
{
  "event_class": "intelligence.response.emitted",
  "tenant_id": "acme-corp",
  "request_id": "req_01HZX9...",
  "response_hash": "blake3:fedcba...",
  "response_token_count": 384,
  "watermark_id": "wm_01HZX9...",
  "completion_latency_ms": 2412,
  "provider_billed_tokens": 1602,
  "timestamp": "2026-05-20T14:32:20.304Z"
}
```

Hashed prompts + responses are stored in audit-chain by default; raw text is stored only if the tenant opts into `audit_chain.intelligence_full_text_retention = true` (significant storage cost; required for some regulated industries).

## Can we ground LLM responses in a tenant's own ontology?

Yes. The `ontology` µservice exposes entity/relationship queries; `intelligence` integrates via the RAG pipeline by treating ontology entities as structured chunks. A tenant query like "show me deals with my top-5 customers" goes through:

1. Cedar gate (`intelligence::ontology::query`).
2. Ontology query: get top-5 customers by deal-value.
3. RAG retrieval: get product docs + deal notes for each.
4. LLM completion: synthesize.

This bridges structured + unstructured knowledge per ADR-0220's "AI Substrate" + "Consumer Brand Surface" thesis.

## What's the difference between this µservice and the substrate AI services (model-hosting, training)?

Per ADR-0220 two-layer thesis:

- **AI Substrate** (lower layer): model hosting (`intelligence-substrate-vllm`), training pipelines (`intelligence-training`), embedding services, vector stores. NOT tenant-facing directly.
- **Consumer Brand Surface** (upper layer = `intelligence` µservice): tenant-facing API. Routes to substrate + hosted providers; applies Cedar + audit-chain + watermarking + EU AI Act gates.

A tenant talks to `intelligence`. `intelligence` talks to the substrate + hosted providers under the hood. The substrate layer is owned by an AI-platform-engineering team; `intelligence` is owned by an AI-product team.

## Why do we use Qdrant + Weaviate instead of pgvector?

Per ADR-XXX-intelligence-vector-store. pgvector is excellent for ≤ 10M vectors at low QPS. Above that:

- Qdrant: native HNSW + per-collection sharding + per-collection multi-tenancy + native filter-pushdown. Scales to ≥ 100M vectors per collection.
- Weaviate: similar shape; better module ecosystem (text2vec, generative).

We support both for paid tenant_class deployments to let tenants pick based on their existing skill set. Default is Qdrant.

The `analytics` ClickHouse already supports vector data (`Array(Float32)` + `cosineDistance` UDF), but it's not optimized for nearest-neighbor at the scale of RAG.

## What's the high-risk-task classifier? Is it itself an AI system?

Yes. Per ADR-0143, the high-risk-task classifier is itself a Llama 3.3 70B fine-tuned model (with a small LoRA) trained on EU AI Act Annex III scenario examples. Inputs: the user prompt + the tenant context. Outputs: one of [`not-high-risk`, `credit-scoring`, `hiring`, `immigration`, `education`, `critical-infrastructure`, `law-enforcement`, `other-high-risk`].

The classifier is itself an Annex III system (per Article 6) and we have filed conformity-assessment with Notified Body NB-DE-12345. Its outputs go into the audit chain; its training data, evaluation, and bias-audit are documented in `intelligence/decisions/ADR-IN-007-high-risk-classifier.md`.
