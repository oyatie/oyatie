---
doc_class: MigrationPlaybook
microservice: intelligence
vendor: OpenAI Enterprise (direct integration)
date: 2026-05-20
doc_status: published
---

# Migration playbook — OpenAI Enterprise (direct) → oyatie intelligence

Audience: a tenant or oyatie internal AI team currently using OpenAI Enterprise direct (gpt-4o, gpt-4o-mini, o1, o1-mini, text-embedding-3-large) and wanting to move to oyatie's `intelligence` µservice. Drivers: multi-provider failover, EU AI Act Annex III gating, sovereign-pack residency, per-tenant LoRA fine-tuning, audit-chain non-repudiation, RAG-pipeline integration.

## Why this migration matters

OpenAI Enterprise is excellent for the 60-70 % of AI use cases that need GPT-4o or text-embedding-3-large. Structural limits:

- Single-provider exposure: each OpenAI outage = your outage.
- No sovereign-pack residency for KR / CN / certain US-GovCloud requirements (Azure OpenAI gives partial coverage but not KR-PIPA-bound).
- No audit-chain non-repudiation; OpenAI logs are stored at OpenAI, not the tenant's cell.
- EU AI Act compliance requires the tenant to build conformity-assessment + Article 14 human-in-the-loop on top of OpenAI's surface.
- Fine-tuning limited to gpt-4o-mini and earlier (gpt-4o not fine-tunable).

oyatie addresses all of these.

## Step 1 — Inventory OpenAI usage (≤ 1 week)

```bash
# From OpenAI admin (org-level access required):
# 1. Export usage report from console.openai.com/usage
# 2. List active API keys + their scopes
# 3. List active fine-tuned models
# 4. List active Assistants + Threads (if Assistants API is in use)
# 5. Export the system prompts in use (where stored externally)
```

Document:

- Active API keys + their associated team/tenant.
- Daily/monthly token usage (input vs output, by model).
- Fine-tuned models (model_id + base model + training data origin).
- Assistants API usage if any (Assistants are OpenAI-managed conversation state; can't be 1:1 migrated).
- Webhook integrations (e.g., usage webhooks).
- Tools / function-calling schemas in use.

Typical mid-enterprise OpenAI install: 5-50 active keys, 1-10M tokens/day, 1-5 fine-tuned models, 20-100 tools.

## Step 2 — Map prompts + functions to oyatie surface (≤ 1-2 weeks)

OpenAI Chat Completions → oyatie `intelligence::request::create`:

```diff
- // OpenAI direct
- const response = await openai.chat.completions.create({
-   model: "gpt-4o",
-   messages: [
-     { role: "system", content: "You are a helpful assistant" },
-     { role: "user", content: userMessage }
-   ],
-   tools: [...]
- });

+ // oyatie
+ const response = await oyaIntelligence.requestCreate({
+   tenant: "acme-corp",
+   task: "chat",
+   prompt: userMessage,
+   tools: [...],  // tools schema preserved 1:1
+   // System prompt comes from the tenant's prompt-fence policy
+   // Model routing comes from the tenant's routing policy
+ });
```

The tools / function-calling schemas (OpenAI JSON-Schema format) work 1:1 with oyatie's tool surface. The system prompt is configured ONCE in the tenant's prompt-fence policy and applied to every request automatically.

OpenAI Embeddings → oyatie `intelligence::request::create` with `task=embedding`:

```diff
- const embedding = await openai.embeddings.create({
-   model: "text-embedding-3-large",
-   input: text
- });

+ const embedding = await oyaIntelligence.requestCreate({
+   tenant: "acme-corp",
+   task: "embedding",
+   prompt: text
+ });
```

OpenAI fine-tuned models → oyatie LoRA adapters:

OpenAI's fine-tuning produces an OpenAI-hosted model ID like `ft:gpt-4o-mini-2024-07-18:acme:abc123`. oyatie's fine-tuning produces a downloadable LoRA adapter that you load alongside Llama 3.3 70B or Mistral Large 2407.

Migration path:

1. Export the fine-tune training data (if you still have it).
2. Re-train as a LoRA against Llama 3.3 70B.
3. Compare LoRA-adapted Llama vs fine-tuned gpt-4o-mini on a held-out evaluation set.
4. If LoRA is competitive (within 2-3 pp on your metric), switch over.

If the fine-tuned gpt-4o-mini is still preferred, you can route specific requests to OpenAI passthrough via oyatie:

```sh
oya intelligence routing-policy configure \
    --tenant acme-corp \
    --task-override 'customer-service:openai-gpt-4o-mini-fine-tune:ft:gpt-4o-mini-2024-07-18:acme:abc123'
```

## Step 3 — Migrate Assistants API (if applicable) (≤ 2-4 weeks)

OpenAI Assistants API is a managed conversation-state + RAG + tool-orchestration service. oyatie doesn't 1:1 replicate it because oyatie's architecture separates state (managed by your application's database) from inference (managed by `intelligence`).

Migration path:

1. **Conversation state**: replace OpenAI's Threads with your own PostgreSQL `conversation_messages` table (per-tenant). Each request to oyatie includes the relevant message history.
2. **Tool orchestration**: oyatie's tool surface is per-request. Move tool definitions from Assistants to your application's per-request payload.
3. **RAG**: replace OpenAI's "File Search" with oyatie's RAG pipeline (Qdrant + BGE-M3 / OpenAI embedding). Same shape, different backend.

This is the heaviest migration step — typically 2-4 weeks of engineering per Assistants-heavy application.

## Step 4 — Dual-call shadow (≤ 2-4 weeks)

For each request, issue to BOTH OpenAI direct AND oyatie:

```javascript
const [openaiResponse, oyaResponse] = await Promise.all([
  openai.chat.completions.create({...}),
  oyaIntelligence.requestCreate({...})
]);

// Return openaiResponse to the user; log oyaResponse for evaluation.
await loggingService.shadow({
  user_id, request_id,
  openai_response_hash: hash(openaiResponse.choices[0].message.content),
  oya_response_hash: hash(oyaResponse.response),
  openai_latency_ms: ...,
  oya_latency_ms: ...,
  openai_tokens: ...,
  oya_tokens: ...
});
```

Reconciliation: nightly compare:

- Latency distributions (oya should be within 200 ms p99 of OpenAI direct).
- Token-count distributions (response length should match within ~ 10 %).
- Sample 50 responses/night for human evaluation of quality parity.

After 4 consecutive weeks of clean parity, cut over.

## Step 5 — Cut over + redirect production traffic (≤ 1 d)

```sh
# Flip routing-policy to oyatie as default
oya governance set-config \
    --tenant acme-corp \
    --key default_ai_provider \
    --value oyatie

# Audit-emit the cutover
oya audit emit \
    --tenant acme-corp \
    --event-class governance.ai_substrate.cut_over \
    --payload '{"from":"openai-direct","to":"oyatie","cutover_at":"2026-05-20T14:00:00Z"}'
```

## Step 6 — OpenAI passthrough or full sunset (≤ 90 d post-cutover)

Two paths post-cutover:

1. **OpenAI as fallback only**: oyatie's routing policy keeps OpenAI in the fallback chain (e.g., for tenant requests that explicitly request OpenAI models). OpenAI Enterprise subscription stays active at a reduced tier.
2. **Full OpenAI sunset**: only self-hosted + Anthropic + Vertex remain. Cancel OpenAI Enterprise subscription per the contract notice period.

Path 1 is more common at first; path 2 once the team is comfortable that self-hosted + Anthropic cover all use cases.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| User-visible quality regression | High | Run dual-call shadow for ≥ 4 weeks; require human eval parity |
| Assistants API migration takes longer than expected | High | Plan 4 weeks minimum; sequence after non-Assistants migration |
| Fine-tuned gpt-4o-mini outperforms LoRA Llama on tenant's metric | Medium | Keep OpenAI passthrough enabled for the fine-tune route |
| EU AI Act high-risk-task classifier mis-classifies tenant's prompts | High | Pre-validate classifier on tenant's prompt sample; tune threshold if needed |
| Token-counting differences between OpenAI tiktoken + oyatie | Low | Token-count metric will differ; budget for ~ 5-10 % variance |
| Webhook integrations break on cutover | Medium | Map OpenAI webhooks to oyatie's audit-chain events before cutover |
| RAG-pipeline performance differs from File Search | Medium | Build evaluation harness; iterate on chunk size + embedding model |
| Sovereign-pack tenant blocked from OpenAI passthrough | Medium (the tenant's design) | Move them to self-hosted + Anthropic; OpenAI not allowed in their pack |
| OpenAI key compromise during dual-call | Low | Standard key rotation; no incremental risk |
| Increased latency from Cedar gate + audit-chain emit | Low | Budget ~ 100 ms p99 overhead; should be within tenant tolerance |
