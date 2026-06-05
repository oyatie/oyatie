---
doc_class: Tutorial
microservice: intelligence
persona: ai-platform-engineer + ml-engineer + tenant-product-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Build a RAG-augmented chat for a tenant's product knowledge base

You will: create a tenant knowledge base, ingest 200+ markdown product docs, configure a routing policy + prompt fence, execute RAG-augmented chat completions with citations, verify the audit-chain emission, and emit the EU AI Act Annex III non-high-risk attestation. Total time ≤ 75 minutes.

## Pre-requisites

- A tenant cell with `paid` tenant_class eligibility per ADR-0329, ADR-0330, and ADR-0331.
- Buck2/Prow-backed Intelligence control-plane access.
- A tenant principal in the `ai_admin` Cedar role.
- A corpus of markdown product docs (~ 200 docs, ~ 50-200 KiB each).

## Step 1 — Configure the routing policy + prompt fence (≤ 10 min)

```sh
oya intelligence routing-policy configure \
    --tenant acme-corp \
    --policy-file ./routing.yaml

oya intelligence prompt-fence configure \
    --tenant acme-corp \
    --policy-file ./fence.yaml
```

The routing policy (`./routing.yaml`):

```yaml
version: 1
tenant_id: acme-corp
default_provider: anthropic
task_routing:
  chat: anthropic-claude-3-5-sonnet-20251022
  summarization: openai-gpt-4o-mini
  code: anthropic-claude-3-5-sonnet-20251022
  embedding: openai-text-embedding-3-large
  image_generation: openai-gpt-4o-mini  # gpt-4o-mini handles image generation in 2025-Q4
fallback_chain:
  - primary: anthropic-claude-3-5-sonnet-20251022
    fallback: anthropic-claude-3-5-haiku-20241022
  - primary: openai-gpt-4o
    fallback: vertex-gemini-1.5-pro-002
per_request_overrides_allowed: true
```

The prompt fence (`./fence.yaml`):

```yaml
version: 1
tenant_id: acme-corp
system_prompt_template: |
  You are ACME Corporation's customer-service AI assistant.
  Always answer in the customer's language. Always cite the source documents using the format [Source N].
  If you don't know the answer, say so — never invent product features.
  Knowledge cutoff: 2026-05-01.

content_moderation:
  blocked_phrases: ["Competitor-X", "Competitor-Y", "[INTERNAL-CODENAME-PROJECT-OMEGA]"]
  pii_redaction:
    enabled: true
    classes: [email, phone, ssn, credit_card, ip_address]

response_constraints:
  max_tokens: 1500
  temperature: 0.7
  citations_required: true

watermark:
  enabled: true
  detection_key_published_to_tenant: true
```

## Step 2 — Create the knowledge base (≤ 5 min)

```sh
oya intelligence knowledge-base create \
    --tenant acme-corp \
    --kb-id acme-product-docs \
    --vector-store qdrant \
    --embedding-model bge-m3 \
    --chunk-size 1000 \
    --chunk-overlap 200 \
    --metadata-schema '{"product_line": "string", "doc_type": "enum(faq,guide,reference)", "version": "string"}'
```

Output:

```
Knowledge base created.
  kb_id: acme-product-docs
  qdrant_collection: kb_acme_product_docs
  vector_dim: 1024 (bge-m3)
  embedding_provider: self-hosted
```

## Step 3 — Ingest the corpus (≤ 15 min for 200 docs)

```sh
oya intelligence knowledge-base ingest \
    --tenant acme-corp \
    --kb acme-product-docs \
    --source ./acme-product-docs/ \
    --file-pattern "*.md" \
    --metadata-from-frontmatter
```

The ingest worker:

1. Reads each markdown file.
2. Parses YAML frontmatter for metadata (`product_line`, `doc_type`, `version`).
3. Chunks the content (1000 chars + 200 overlap).
4. Computes BGE-M3 embeddings (batched, GPU-accelerated).
5. Uploads to Qdrant with metadata.

Wait for completion:

```sh
oya intelligence knowledge-base stats --tenant acme-corp --kb acme-product-docs
```

Expected output:

```yaml
kb_id: acme-product-docs
chunks_ingested: 4218
embedding_compute_total_seconds: 248
qdrant_collection_size_mb: 87
last_ingest_at: 2026-05-20T14:42:08Z
ingest_errors: 0
```

## Step 4 — Issue a RAG-augmented chat completion (≤ 10 min)

```sh
oya intelligence request create \
    --tenant acme-corp \
    --task chat \
    --prompt "How do I configure WPA3 + WPA2 mixed mode on the ACME router RT-7000, and what's the security implication?" \
    --rag-knowledge-base acme-product-docs \
    --rag-top-k 5 \
    --rag-min-similarity 0.65 \
    --output ./response.json
```

Expected response:

```json
{
  "request_id": "req_01HZX9K3...",
  "provider_used": "anthropic-claude-3-5-sonnet-20251022",
  "rag_retrieved_chunks": [
    {
      "chunk_id": "ch_acme_product_docs_4218_42",
      "source_doc": "rt-7000-security-guide.md",
      "similarity": 0.892,
      "snippet": "WPA3 + WPA2 mixed mode allows older clients..."
    },
    {
      "chunk_id": "ch_acme_product_docs_4218_28",
      "source_doc": "rt-7000-wifi-config.md",
      "similarity": 0.876,
      "snippet": "Access the router admin panel at 192.168.1.1..."
    },
    ...
  ],
  "rag_retrieval_ms": 42,
  "response": "To configure WPA3 + WPA2 mixed mode on the ACME RT-7000:\n\n1. Access the admin panel at 192.168.1.1 [Source 2].\n2. Navigate to Wi-Fi → Security.\n3. Set Encryption Mode to 'WPA3-Personal + WPA2-Personal Transition Mode' [Source 1].\n4. The router will accept connections from both WPA3-capable clients (using SAE) and legacy WPA2 clients (using PSK).\n\nSecurity implication: WPA2 PSK is vulnerable to offline dictionary attacks if the passphrase is weak. Mixed mode means an attacker can target the WPA2 PSK independently of the WPA3 SAE. For environments where all clients support WPA3, use 'WPA3-Personal Only' for forward secrecy [Source 1].",
  "response_token_count": 184,
  "watermark_id": "wm_01HZX9K3...",
  "completion_latency_ms": 2304,
  "high_risk_class": null,
  "cedar_decision": "allow"
}
```

The response uses [Source 1], [Source 2] citations grounded in the retrieved chunks. Verify the chunks exist:

```sh
oya intelligence knowledge-base chunk-show \
    --tenant acme-corp \
    --kb acme-product-docs \
    --chunk-id ch_acme_product_docs_4218_42
```

## Step 5 — Watermark detection (≤ 5 min)

The response carries a steganographic watermark. Verify:

```sh
oya intelligence watermark detect \
    --tenant acme-corp \
    --input "$(jq -r .response ./response.json)"
# Output:
#   watermarked: true
#   watermark_id: wm_01HZX9K3...
#   tenant_id: acme-corp
#   request_id: req_01HZX9K3...
#   confidence: 0.9982
```

The watermark survives minor edits (truncation up to ~ 20%, paraphrasing of ~ 10% of words). Larger edits degrade confidence; below 0.5 confidence the watermark is considered non-detectable.

## Step 6 — Audit-chain verification (≤ 5 min)

```sh
oya audit query --tenant acme-corp --event-class "intelligence.*" --since 5m
```

Expected events:

- `intelligence.request.submitted` (1)
- `intelligence.prompt_fence.applied` (1)
- `intelligence.rag.retrieved` (1, with chunk hashes)
- `intelligence.high_risk_task.classified` (1, with class=not-high-risk)
- `intelligence.cedar.evaluated` (1)
- `intelligence.response.emitted` (1)
- `intelligence.watermark.applied` (1)

All events Ed25519-signed. The chain verifies end-to-end:

```sh
oya audit verify-chain --tenant acme-corp --since 5m
# Output: chain verified: 7 events, batches: 1, signature_gaps: 0
```

## Step 7 — EU AI Act Annex III non-high-risk attestation (≤ 10 min)

Even for non-high-risk tasks, tenants must declare the system's purpose for transparency under EU AI Act Article 50. Author the attestation:

```sh
oya governance file-evidence \
    --lane oya-governance-eu-ai-act-transparency \
    --evidence-class article-50-transparency-declaration \
    --tenant acme-corp \
    --system-purpose "Customer-service chat for ACME router products" \
    --intended-deployment-class "consumer-product-support" \
    --not-classified-as "annex-iii-high-risk" \
    --classifier-version "intelligence-high-risk-classifier-v2.4" \
    --declaration-author "alice@acme-corp.example"
```

This emits:

- `governance.eu_ai_act.transparency_declaration` to audit-chain.
- A row in the `oya-governance-eu-ai-act-transparency` lane.

For EU-deployed AI systems, this declaration is mandatory under Article 50 (transparency obligations). It must be re-filed annually or when the system purpose materially changes.

## Step 8 — Set up an ongoing evaluation harness (≤ 15 min)

For production use, create an evaluation set + nightly evaluation:

```sh
oya intelligence eval-suite create \
    --tenant acme-corp \
    --suite-id customer-service-quality \
    --eval-set ./eval-set.jsonl \
    --metrics correctness,citation-accuracy,refusal-appropriateness \
    --schedule "nightly:02:00:UTC"
```

The `./eval-set.jsonl` contains ~ 50-200 hand-crafted question-answer pairs with expected citation source-docs. Each night the harness runs:

1. For each eval question, issue a RAG-augmented chat completion.
2. Compare response correctness via LLM-as-judge (Claude 3.5 Sonnet on a held-out grading rubric).
3. Verify citations point to the expected source docs.
4. Track score over time; alert if score degrades > 5 pp.

The eval results emit to audit-chain (`intelligence.eval.executed`) and surface in the per-tenant ML-ops dashboard.

## What you've learned

- Routing policy + prompt fence configuration for a tenant.
- Knowledge-base creation + corpus ingest with embedding.
- RAG-augmented chat with citations.
- Watermark detection round-trip.
- Audit-chain verification of the full request lifecycle.
- EU AI Act Article 50 transparency declaration.
- Continuous evaluation harness setup.

Next tutorial: `tutorials/fine-tune-tenant-lora.md` — train a per-tenant LoRA adapter on the tenant's customer-service transcripts and deploy it alongside the RAG pipeline.
