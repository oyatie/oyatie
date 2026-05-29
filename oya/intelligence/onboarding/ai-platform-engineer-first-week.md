---
doc_class: Onboarding
microservice: intelligence
persona: ai-platform-engineer + ml-engineer + ai-governance-officer
related_adrs: [ADR-0220, ADR-0255, ADR-0143, ADR-0095, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# AI Platform Engineer onboarding — first 5 working days on `intelligence`

Audience: a new AI platform engineer, ML engineer, or AI-governance officer joining the `intelligence` rotation. By Day-5 they will have: bootstrapped a demo_trial tenant_class cell with multi-provider routing, exercised an OpenAI + Anthropic + Vertex + Mistral request flow, configured a per-tenant prompt fence, executed a Llama 3.3 70B self-hosted inference, walked an EU AI Act Annex III high-risk refusal, and completed a RAG-ingest dry-run.

## Day 1 — Tour the substrate

1. Read `PRD.md` (∼ 45 min). Note the inheritance from ADR-0220 (two-layer intelligence substrate: AI Substrate + Consumer Brand Surface) + ADR-0255 (intelligence substrate definition).
2. Read `ARCHITECTURE.md` § routing-policy + § prompt-fence + § high-risk-task-classification (∼ 60 min).
3. Read the EU AI Act Annex III refusal contract: `docs/decisions/ADR-0095-eu-ai-act-annex-iii-refusal-contract.md` + ADR-0143 (high-risk task classification).
4. Open the Grafana folder `intelligence`. retired-advanceden boards: `intelligence-request-rate`, `intelligence-routing-latency`, `intelligence-refusal-rate`, `intelligence-token-throughput`, `intelligence-fine-tune-active-adapters`, `intelligence-rag-query-latency`, `intelligence-watermark-emission-rate`.
5. Walk `runbooks/README.md`. The on-call runbooks: `provider-outage-failover.md`, `prompt-fence-breach.md`, `high-risk-refusal-storm.md`, `rag-stale-context.md`, `adapter-load-failure.md`, `vllm-oom.md`, `embedding-drift.md`, `gpu-down.md`.
6. Sit in on the Wednesday AI-substrate handoff. Watch how the outgoing rotation reads the past-week refusal-rate + token-throughput + per-tenant adapter usage.

Acceptance: you can sketch the request path: tenant API → Cedar gate → high-risk-task classifier → prompt-fence engine → routing policy → provider/self-hosted backend → response → audit-chain emit → watermark → response. Plus the RAG path: knowledge-base query → embedding compute → Qdrant query → context-window assembly → completion.

## Day 2 — demo_trial tenant_class cell bootstrap + multi-provider routing

```sh
cargo run -p oya-dev-cli -- intelligence bootstrap \
    --tenant-class demo_trial \
    --cell drill-syd-1 \
    --openai-secret-name openai-enterprise-key \
    --anthropic-secret-name anthropic-console-key \
    --vertex-service-account-secret vertex-sa-json \
    --mistral-secret-name mistral-large-key \
    --postgres-endpoint postgres://drill-pg-syd-1:5432/intelligence \
    --valkey-endpoint valkey://drill-valkey-syd-1:6379 \
    --pulsar-endpoint pulsar://drill-pulsar-syd-1:6650 \
    --audit-chain-endpoint http://drill-audit-syd-1:8080 \
    --kubeconfig ./drill-syd-1.kubeconfig
```

Expected runtime: ≤ 10 min. Verify after bootstrap:

```sh
oya intelligence health --cell drill-syd-1
# Expected:
#   openai:    connected (gpt-4o, gpt-4o-mini, o1, o1-mini)
#   anthropic: connected (claude-3-5-sonnet-20251022, claude-3-5-haiku-20241022)
#   vertex:    connected (gemini-1.5-pro-002, gemini-1.5-flash-002)
#   mistral:   connected (mistral-large-2411, mistral-medium-2410)
#   cedar:     policies-loaded (intelligence_v3 set)
#   audit_chain: emit-pipeline-up
```

Try a request via each provider:

```sh
for provider in openai anthropic vertex mistral; do
    echo "--- $provider ---"
    oya intelligence request create \
        --cell drill-syd-1 \
        --tenant drill-acme \
        --provider $provider \
        --task chat \
        --prompt "Summarize this sentence in 5 words: The quick brown fox jumps over the lazy dog." \
        --max-tokens 50
done
```

Expected: 4 responses, each ≤ 5 s, each emitting `intelligence.request.submitted` + `intelligence.response.emitted` to audit-chain.

Verify the audit emissions:

```sh
oya audit query --tenant drill-acme --event-class intelligence.request.submitted --since 5m
# Expected: 4 events, each with provider + model + prompt_hash + token_count
```

Acceptance: cell up; multi-provider routing functional; audit emissions confirmed.

## Day 3 — Per-tenant prompt fence + content moderation

Configure a per-tenant prompt fence:

```sh
oya intelligence prompt-fence configure \
    --tenant drill-acme \
    --policy-file ./fence-policy.yaml
```

The policy:

```yaml
tenant_id: drill-acme
system_prompt_template: |
  You are ACME Corp's customer-service AI assistant. Your knowledge cut-off is 2026-05-01.
  Always respond in the customer's language. Never discuss our competitors (Competitor-X, Competitor-Y).
  Always cite source documents from the ACME knowledge base when answering product questions.

content_moderation:
  blocked_phrases:
    - "Competitor-X"
    - "Competitor-Y"
    - "[REDACTED-INTERNAL-CODENAME]"
  pii_redaction:
    enabled: true
    classes: [email, phone, ssn, credit_card]

routing_preferences:
  task_chat: anthropic-claude-3-5-sonnet
  task_summarization: openai-gpt-4o-mini
  task_embedding: openai-text-embedding-3-large
  task_code: anthropic-claude-3-5-sonnet
  task_image: openai-gpt-4o-mini
```

Test the fence:

```sh
# Should succeed (no blocked phrase, no PII)
oya intelligence request create \
    --tenant drill-acme \
    --task chat \
    --prompt "How do I reset my ACME router?"
# Expected: response, audit event emitted, fence-pass recorded.

# Should be blocked (blocked phrase)
oya intelligence request create \
    --tenant drill-acme \
    --task chat \
    --prompt "How does the ACME router compare to Competitor-X?"
# Expected: 403, intelligence.prompt_fence.blocked event, response not sent to provider.

# Should be redacted (PII in prompt)
oya intelligence request create \
    --tenant drill-acme \
    --task chat \
    --prompt "My email is alice@example.com. How do I reset my router?"
# Expected: response, prompt redacted (email replaced with [REDACTED-PII-EMAIL]) before being sent to provider.
```

Verify:

```sh
oya audit query --tenant drill-acme --event-class intelligence.prompt_fence.blocked --since 5m
oya audit query --tenant drill-acme --event-class intelligence.pii.redacted --since 5m
```

Acceptance: prompt fence configured; blocked + redacted paths verified.

## Day 4 — Self-hosted Llama 3.3 70B + EU AI Act Annex III refusal walk

Self-hosted Llama 3.3 70B requires a GPU pool. For demo_trial tenant_class drills this is typically a small cluster with 4× A100 80 GiB or 2× H100 80 GiB. Verify:

```sh
oya intelligence vllm health --cell drill-syd-1
# Expected: vllm-llama-3-3-70b: up, tensor-parallel-size=4, kv-cache-utilization=12%
```

Test inference:

```sh
oya intelligence request create \
    --tenant drill-acme \
    --provider self-hosted \
    --model llama-3.3-70b-instruct \
    --task chat \
    --prompt "Explain quantum entanglement in 50 words." \
    --max-tokens 200
# Expected: response in ~ 2-3 s (depending on GPU)
```

Now walk the EU AI Act Annex III refusal. Per ADR-0095 + ADR-0143, certain high-risk task classes (credit-scoring, hiring, immigration, education, critical infrastructure, law enforcement) require explicit deployer-attestation before execution. The refusal kernel fires:

```sh
oya intelligence request create \
    --tenant drill-acme \
    --task chat \
    --prompt "Should we approve this loan application? Applicant: John Doe, income $45 000, credit score 620."
# Expected: 403, intelligence.high_risk_task.refused event
# Reason: "EU AI Act Annex III high-risk task class 'credit-scoring' requires Article 43 conformity-assessment attestation; tenant 'drill-acme' has not filed this attestation."
```

The refusal is mandatory; tenants who need to perform credit-scoring must first file the Article 43 attestation:

```sh
oya governance file-evidence \
    --lane oya-governance-eu-ai-act-attestation \
    --evidence-class article-43-conformity-assessment \
    --tenant drill-acme \
    --task-class credit-scoring \
    --notified-body-id NB-DE-12345 \
    --assessment-report ./conformity-assessment-credit-scoring-2026.pdf
```

After the evidence is filed + verified, the same prompt no longer triggers refusal (but still emits a `high_risk_task.classified` event for monitoring).

Acceptance: self-hosted Llama responds; EU AI Act Annex III refusal kernel fires for credit-scoring; attestation evidence-flow walked.

## Day 5 — RAG ingestion + retrieval-augmented response

Set up a tenant knowledge base:

```sh
oya intelligence knowledge-base create \
    --tenant drill-acme \
    --kb-id acme-product-docs \
    --vector-store qdrant \
    --embedding-model bge-m3 \
    --chunk-size 1000 --chunk-overlap 200
```

Ingest a corpus:

```sh
oya intelligence knowledge-base ingest \
    --tenant drill-acme \
    --kb acme-product-docs \
    --source ./acme-product-docs/ \
    --file-pattern "*.md,*.pdf,*.html"
```

Wait for ingest (~ 2-5 min for ~ 500 docs). Verify:

```sh
oya intelligence knowledge-base stats --tenant drill-acme --kb acme-product-docs
# Expected:
#   chunks_ingested: 4 218
#   vector_dim: 1024 (bge-m3)
#   embedding_compute_total_seconds: 184
#   qdrant_collection_size_mb: 87
```

Issue a RAG-augmented request:

```sh
oya intelligence request create \
    --tenant drill-acme \
    --task chat \
    --prompt "How do I configure WPA3 on the ACME router model RT-7000?" \
    --rag-knowledge-base acme-product-docs \
    --rag-top-k 5
# Expected:
#   response: detailed answer with [Source 1], [Source 2] citations
#   metadata: rag_retrieved_chunks=5, rag_retrieval_ms=42, completion_provider=anthropic
```

Verify the audit event captures the retrieval context:

```sh
oya audit query --tenant drill-acme --event-class intelligence.rag.retrieved --since 5m
# Expected: 1 event with rag_kb_id, retrieved_chunk_hashes, query_embedding_hash
```

Acceptance: knowledge base ingested; RAG-augmented response with citations; retrieval audit-emitted.

## What you've learned

- Multi-provider routing via OpenAI / Anthropic / Vertex / Mistral + audit-chain emission per request.
- Per-tenant prompt fence + content moderation + PII redaction.
- Self-hosted Llama 3.3 70B inference via vLLM.
- EU AI Act Annex III refusal kernel + Article 43 attestation evidence flow.
- RAG ingestion + retrieval-augmented response with citation emission.

Next week: paid tenant_class tour (per-tenant adapter cache, open-source model serving, watermarking, per-tenant LoRA fine-tuning, RAG pipeline, Annex III attestation, sovereign-pack model allowlists), and your first production shadow.
