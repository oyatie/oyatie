---
doc_class: Benchmark
microservice: intelligence
benchmark_date: 2026-05-20
related_adrs: [ADR-0220, ADR-0255, ADR-0329, ADR-0330, ADR-0331]
doc_status: published
---

# Benchmarks — oyatie intelligence vs OpenAI Enterprise vs Anthropic Console vs Google Vertex AI

Workloads measured: (a) chat completion latency, (b) embedding throughput, (c) RAG retrieval + completion E2E, (d) per-tenant LoRA fine-tune (training + inference), (e) high-risk-task classification accuracy, (f) annual TCO for a 1 000-tenant fleet at 100M tokens/month.

Hardware (oyatie paid tenant_class self-hosted backend): 8× intelligence-api nodes (16 vCPU EPYC 9354P, 64 GiB RAM, 1 TiB NVMe), 8× H100 SXM5 80 GiB on 2 GPU nodes for self-hosted Llama 3.3 70B (tensor-parallel-4 + 2-node pipeline-parallel), Qdrant 1.12 (3 nodes, 32 vCPU, 128 GiB RAM each).

Hosted-provider comparators tested via standard SDK calls from the same region (us-west-2 / eu-west-3) as the oyatie cell to minimise network bias.

## Workload (a) — chat completion latency (200-token completion, 1k-token prompt)

| Provider / model | p50 (ms) | p99 (ms) | Per-1M-input-token cost (USD) | Per-1M-output-token cost (USD) |
|---|---:|---:|---:|---:|
| oyatie (self-hosted Llama 3.3 70B, vLLM tp=4) | 1 480 | 2 980 | 0 (compute amortised) | 0 (compute amortised) |
| oyatie (passthrough → Anthropic Claude 3.5 Sonnet) | 2 412 | 4 920 | 3.00 | 15.00 |
| oyatie (passthrough → OpenAI GPT-4o) | 1 824 | 3 612 | 2.50 | 10.00 |
| oyatie (passthrough → Vertex Gemini 1.5 Pro) | 2 014 | 3 924 | 1.25 | 5.00 |
| oyatie (passthrough → Mistral Large 2411) | 1 612 | 3 184 | 2.00 | 6.00 |
| Direct Anthropic Claude 3.5 Sonnet | 2 312 | 4 824 | 3.00 | 15.00 |
| Direct OpenAI GPT-4o | 1 720 | 3 512 | 2.50 | 10.00 |
| Direct Vertex Gemini 1.5 Pro | 1 920 | 3 824 | 1.25 | 5.00 |

Reading: oyatie's overhead vs direct-provider is ~ 100 ms p50 (Cedar gate + audit-chain emit + watermark). Self-hosted Llama 3.3 70B is competitive on latency at zero per-token cost — break-even vs direct-Anthropic at ~ 60M tokens/month sustained.

## Workload (b) — embedding throughput (text-embedding-3-large class, 8k tokens/doc, 100 docs batched)

| Provider / model | Throughput (docs/sec) | p99 latency (ms) | Per-1M-token cost (USD) |
|---|---:|---:|---:|
| oyatie (self-hosted BGE-M3, GPU H100) | 8 200 | 122 | 0 |
| oyatie (passthrough → OpenAI text-embedding-3-large) | 1 800 | 480 | 0.13 |
| oyatie (passthrough → Vertex text-embedding-005) | 1 200 | 720 | 0.10 |
| Direct OpenAI text-embedding-3-large | 1 700 | 460 | 0.13 |

Reading: self-hosted BGE-M3 on H100 outperforms hosted providers by ~ 5× throughput at zero per-token cost. BGE-M3 produces 1024-dim embeddings vs OpenAI's 3072-dim; for most RAG use cases the latency + cost win exceeds the marginal accuracy loss (BGE-M3 ranks within ~ 2 pp of text-embedding-3-large on MTEB benchmarks at our typical chunk sizes).

## Workload (c) — RAG E2E (5-document retrieval + 200-token completion)

| Stack | Retrieval (ms) | Completion (ms) | Total (ms) p99 |
|---|---:|---:|---:|
| oyatie (Qdrant + self-hosted Llama 3.3 70B + BGE-M3) | 42 | 2 980 | 3 092 |
| oyatie (Qdrant + Anthropic Claude 3.5 Sonnet + BGE-M3) | 42 | 4 920 | 5 032 |
| oyatie (Pinecone + OpenAI GPT-4o + text-embedding-3-large) | 124 | 3 612 | 3 814 |
| Pure Pinecone + OpenAI (Direct, no oyatie) | 124 | 3 512 | 3 712 |

Reading: Qdrant + self-hosted embedding outperforms Pinecone-based stacks on retrieval latency by ~ 80 ms p99. Total E2E latency is dominated by completion latency; self-hosted Llama 3.3 70B + Qdrant is the fastest stack overall.

## Workload (d) — per-tenant LoRA fine-tune (Llama 3.3 70B + LoRA rank 64, 10k examples)

| Stack | Training wall-clock | Training cost (USD) | Inference latency p99 (ms) | Per-tenant adapter size (MB) |
|---|---:|---:|---:|---:|
| oyatie (4× H100 SXM5, vLLM + LoRA) | 6.2 h | 0 (amortised) | 3 084 | 248 |
| Anthropic Claude Custom (closed beta) | (managed; no public benchmarks) | (managed; ~ $200-$500 per training job) | (similar to base) | (managed) |
| OpenAI fine-tuning (gpt-4o-mini fine-tune; not gpt-4o) | (managed; ~ 8-24 h depending on queue) | $8-$25 per million training tokens | (similar to base) | (managed) |
| Vertex Adapter Tuning (Gemini 1.5 Flash) | (managed; ~ 4-12 h) | $20-$80 per million tokens | (similar to base) | (managed) |

Reading: oyatie self-hosted LoRA fine-tuning has higher upfront infrastructure cost but zero per-job cost; at scale (more than ~ 4-6 fine-tune jobs/month per tenant) self-hosted wins. The adapter is portable (the tenant can download + serve elsewhere if needed); hosted-provider fine-tunes are locked to the provider.

## Workload (e) — high-risk-task classification accuracy (EU AI Act Annex III categories)

Test set: 500 hand-crafted prompts spanning the 8 Annex III categories + 500 control prompts.

| Classifier | True positives | False positives | Recall | Precision | F1 |
|---|---:|---:|---:|---:|---:|
| oyatie high-risk classifier v2.4 (Llama 3.3 70B + LoRA) | 472 | 32 | 0.944 | 0.937 | 0.940 |
| GPT-4o zero-shot prompt | 412 | 84 | 0.824 | 0.831 | 0.827 |
| Claude 3.5 Sonnet zero-shot prompt | 438 | 62 | 0.876 | 0.876 | 0.876 |
| OpenAI Moderation API (proxy) | 312 | 18 | 0.624 | 0.945 | 0.752 |

Reading: oyatie's purpose-trained classifier outperforms general-purpose LLM zero-shot prompting. The classifier itself is an Annex III system (conformity-assessment filed with Notified Body NB-DE-12345); its evidence chain is in `intelligence/decisions/ADR-IN-007-high-risk-classifier.md`.

## Workload (f) — annual TCO for 1 000-tenant fleet at 100M tokens/month

Assumptions: 1 000 active tenants, average 100k tokens/tenant/month = 100M total tokens/month, 70/30 input/output split, 30% RAG-augmented, 5% high-risk-classification.

| Stack | Hardware/Compute (USD) | Per-token (USD) | Fine-tune compute (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|---:|
| oyatie (self-hosted Llama 3.3 70B + Qdrant, no passthrough) | 1 240 000 (8× H100 + 2× B200 + Qdrant cluster) | 0 | 0 (self-hosted) | 620 000 (5 SRE × 0.4 FTE) | 1 860 000 |
| oyatie (50/50 self-hosted + passthrough) | 720 000 | ~ 1 200 000 (50% via providers) | 0 | 372 000 | 2 292 000 |
| oyatie (100% passthrough; provider-only) | 192 000 | ~ 2 400 000 | 0 | 248 000 | 2 840 000 |
| Direct Anthropic Claude 3.5 Sonnet (no oyatie) | 0 | ~ 2 700 000 | (limited fine-tune) | 124 000 | 2 824 000 |
| Direct OpenAI GPT-4o (no oyatie) | 0 | ~ 2 100 000 | ~ 180 000 | 124 000 | 2 404 000 |
| Direct Vertex Gemini 1.5 Pro (no oyatie) | 0 | ~ 1 250 000 | ~ 90 000 | 124 000 | 1 464 000 |

Reading: oyatie self-hosted is most economical at scale + provides multi-provider failover + sovereign-pack support + audit-chain integration. Direct Vertex is cheapest at this volume because Gemini 1.5 Pro has the lowest per-token pricing, but it lacks the multi-provider + sovereign + audit-chain capabilities.

Break-even vs hosted providers happens around ~ 50-80M tokens/month sustained for self-hosted Llama 3.3 70B; tenants below this volume should use passthrough.

Caveats:

- These numbers assume H100 GPU cost amortised over 4 years.
- Token-volume estimates are 2026-Q2 list prices; enterprise contracts commonly receive 30-50 % discount.
- Self-hosted ops cost includes the AI-platform-engineering team + GPU lifecycle + Qdrant SRE. Lower-touch deployments need fewer FTEs.

## Reproducibility

The benchmark harness lives at `benchmarks/intelligencebench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks intelligence \
    --workload chat-completion-200-token \
    --providers oyatie-self-hosted,anthropic,openai,vertex \
    --output ./benchmark-results.json
```

Hosted-provider runs require valid API keys for each provider. Results live at `benchmarks/results/intelligence/<date>.csv` and are re-run monthly to detect drift in either direction.
