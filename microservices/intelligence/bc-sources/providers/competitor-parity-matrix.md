---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + council-architecture
deciders: axis-foundry, council-architecture, gtm-customer-success
related_adrs: [ADR-0025, ADR-0026, ADR-0123, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/intelligence-providers/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-FPRV gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (foundry-providers µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading provider-abstraction and LLM-routing products. Drives the `oya-governance-hyperscaler-maturity-claims` gate (HG-FPRV per ADR-0123) and tells gtm what to say + what NOT to say in tenant conversations.

## Competitor Set

| Competitor | Product / surface | Differentiator | Source |
|---|---|---|---|
| LiteLLM | Open-source provider abstraction (~100 vendors) | Wide coverage; permissive defaults | `github.com/BerriAI/litellm` |
| LangChain | Provider abstractions in agent framework | Coupled to agent runtime | `python.langchain.com` |
| Vellum AI | Closed-source enterprise router | Built-in evals; closed | `vellum.ai` |
| Portkey | Hosted observability + routing | Hosted SaaS | `portkey.ai` |
| OpenRouter | Hosted router + billing aggregator | Hosted aggregator | `openrouter.ai` |
| Cloudflare AI Gateway | Hosted proxy + analytics | Edge-network; CF-only | `developers.cloudflare.com/ai-gateway` |
| AWS Bedrock | Provider aggregation (Anthropic / AI21 / Mistral / Meta / Cohere / Amazon Titan) | AWS-only; vendor lock | `aws.amazon.com/bedrock` |
| Azure OpenAI | Microsoft-hosted OpenAI | Azure-only; vendor lock | `azure.microsoft.com/products/ai-services/openai-service` |

## Feature Parity Matrix

### Adapter coverage

| Capability | oyatie | LiteLLM | LangChain | Vellum | Portkey | OpenRouter | CF-AIG | Bedrock | Az-OAI |
|---|---|---|---|---|---|---|---|---|---|
| Anthropic API | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Claude Pro/Max subscription | ✅ | partial | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| OpenAI API | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| ChatGPT Plus subscription | ✅ | partial | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Gemini API | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Gemini Advanced subscription | ✅ | partial | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| In-house (vLLM/TGI) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Open-source local (planned ADR-0026) | M01+ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ |

### Router intelligence

| Capability | oyatie | LiteLLM | LangChain | Vellum | Portkey | OpenRouter | CF-AIG | Bedrock | Az-OAI |
|---|---|---|---|---|---|---|---|---|---|
| Capability-aware routing (cost × latency × residency × health) | ✅ | partial | ❌ | ✅ | ✅ | partial | partial | ❌ | ❌ |
| **Per-pack residency-aware routing** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | partial | ❌ | partial |
| Real-time health-driven demote/recover | ✅ | partial | ❌ | ✅ | ✅ | ✅ | partial | ❌ | ❌ |
| In-process decision (sub-5ms p99) | ✅ | ❌ (hosted) | ✅ | ❌ (hosted) | ❌ (hosted) | ❌ (hosted) | ❌ (hosted) | ❌ (hosted) | ❌ (hosted) |
| Cedar policy gate | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | partial (IAM) | partial (RBAC) |

### Security + audit

| Capability | oyatie | LiteLLM | LangChain | Vellum | Portkey | OpenRouter | CF-AIG | Bedrock | Az-OAI |
|---|---|---|---|---|---|---|---|---|---|
| OpenBao-bridged credentials (zero-leak invariants) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | partial (IAM) | partial (KV) |
| BLAKE3 + Ed25519 envelope per call | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Audit-chain emission on every call | ✅ | ❌ | ❌ | partial | partial | partial | partial | ✅ | ✅ |
| EU AI Act Art. 50 per-call disclosure | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | partial | partial |
| Adapter-substitution attack hardening (Sigstore + digest pin) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| 2-person rule for adapter publish | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Multi-region + residency

| Capability | oyatie | LiteLLM | LangChain | Vellum | Portkey | OpenRouter | CF-AIG | Bedrock | Az-OAI |
|---|---|---|---|---|---|---|---|---|---|
| Multi-region (≥ 5 regions) | ✅ (11 packs) | ❌ | ❌ | partial | partial | partial | ✅ | ✅ | ✅ |
| HIPAA BAA-capable | ✅ (per BAA + ZDR) | ❌ | ❌ | partial | partial | ❌ | ✅ | ✅ | ✅ |
| KR PIPA-compliant | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | partial | ❌ | partial |
| EU GDPR DPA | ✅ | ❌ | ❌ | ✅ | ✅ | partial | ✅ | ✅ | ✅ |
| Cross-pack data-flow refusal at router | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Operations

| Capability | oyatie | LiteLLM | LangChain | Vellum | Portkey | OpenRouter | CF-AIG | Bedrock | Az-OAI |
|---|---|---|---|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ (Rust + OSS) | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Per-tenant cost ceiling | ✅ | partial | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Streaming response support | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tool-use proposal isolation (no auto-execute) | ✅ | varies | ❌ (executes) | varies | varies | varies | ❌ | varies | varies |
| Adapter version pin per-tenant | ✅ | partial | ❌ | ❌ | partial | ❌ | ❌ | ❌ | ❌ |
| In-house model rollout w/ burn-rate auto-rollback | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ (Bedrock-managed) | ❌ |

## Quantitative Performance Parity

| Metric | oyatie target | LiteLLM | Portkey | OpenRouter | CF-AIG | Notes |
|---|---|---|---|---|---|---|
| Router decision p99 (in-process) | ≤ 5 ms | n/a (in-process when self-hosted) | hosted; +50-200 ms | hosted; +20-100 ms | edge; +10-50 ms | oyatie unique in-process |
| Credential resolution p99 | ≤ 10 ms | n/a | n/a | n/a | n/a | OpenBao bridge |
| Adapter overhead p99 | ≤ 8 ms | comparable | comparable | comparable | comparable | parity |
| Cost-per-call overhead (oyatie substrate cost / call) | ≤ $0.0000001 | n/a | hosted markup | hosted markup | hosted markup | oyatie advantage via self-host |

## Key Parity Gaps to Close

| # | Gap | Owner | Target |
|---|---|---|---|
| 1 | More vendor adapters (Mistral / Cohere / AI21 / Llama-hosted; planned via in-house) | axis-foundry | M02-onward |
| 2 | In-IDE workflow-studio integration parity with Vellum | axis-foundry + gtm | M03 |
| 3 | Hosted-deployment option (currently self-host only) | council-architecture | M04-onward if tenant demand |
| 4 | Anthropic prompt-caching integration | axis-foundry | M01+1 |

## Key oyatie Differentiators

1. **In-process router decision (≤ 5 ms p99)** — every hosted competitor adds 20–200 ms of RTT.
2. **OpenBao-bridged credentials with zero-leak invariants** — no competitor mandates a secrets-broker isolation contract.
3. **Per-pack residency-aware routing as a first-class decision constraint** — no competitor enforces residency at the router decision layer.
4. **BLAKE3 + Ed25519 envelope on every call + audit-chain emission** — no competitor cryptographically seals per-call evidence.
5. **EU AI Act Art. 50 per-call disclosure** — no competitor emits the disclosure as a first-class event.
6. **Subscription transports (Claude Pro/Max, ChatGPT Plus, Gemini Advanced)** — LiteLLM has partial; most competitors don't support subscription channels.
7. **In-house-model blue/green w/ burn-rate auto-rollback** — only Bedrock has a comparable managed primitive, and Bedrock-only.
8. **Self-hosted Rust substrate** — most competitors are hosted SaaS or Python; oyatie's substrate matches hyperscaler practice (Cloudflare uses Rust at the edge).

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "Per-pack residency-aware routing is unique to oyatie among production-deployed solutions" (true as of 2026-05-17; review bi-annually).
- ✅ "In-process router decision sub-5 ms p99 is faster than any hosted competitor".
- ✅ "OpenBao secrets-broker isolation contract is unique to oyatie".
- ✅ "EU AI Act Art. 50 per-call disclosure is unique to oyatie".

Sales claims FORBIDDEN (per ADR-0123):
- ❌ "oyatie is cheaper than Bedrock at scale" (depends on workload; do not claim universal).
- ❌ "oyatie supports more vendors than LiteLLM" (LiteLLM has ~100; oyatie has 4 native vendor families at M01 + in-house).
- ❌ "oyatie is HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes (new features / pricing / claims) | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-foundry |
| 3. Re-run quantitative benchmarks (load tests in staging) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/intelligence-providers/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-FPRV.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0133 (industry-best-practice conformance program).
- Competitor sources as cited inline.
