---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-runtime + council-architecture
deciders: axis-foundry-runtime, council-architecture, gtm-customer-success
related_adrs: [ADR-0022, ADR-0024, ADR-0025, ADR-0123, ADR-0130]
related_artifacts:
  - microservices/foundry-runtime/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-FR gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (foundry-runtime µservice)

## Purpose

Quantitative + qualitative parity comparison vs industry-leading agent runtime + capability execution products. Drives the `oya-foundry-fitness-hyperscaler-maturity-claims` gate (per ADR-0123 HG-FR). Tells gtm-customer-success what to say + what NOT to say in tenant conversations.

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| AWS | Bedrock Agent runtime | Hosted agent invocation; native AWS integration; multi-foundation-model | `docs.aws.amazon.com/bedrock/latest/userguide/agents.html` |
| Google Cloud | Vertex AI Agent Builder | Agent orchestration + tool calling + eval | `cloud.google.com/vertex-ai/docs/generative-ai/agents/overview` |
| Microsoft Azure | Azure AI Foundry runtime | Capability registry + safety filters + Azure-native | `learn.microsoft.com/azure/ai-foundry/concepts/agents` |
| LangChain | LangServe + LangGraph cloud | Hosted graph execution; OpenAPI-deploy; OSS lineage | `python.langchain.com/docs/langserve/` |
| OpenAI | Assistants API + Threads | Hosted threads; tool calls; file context | `platform.openai.com/docs/assistants` |
| LlamaIndex | LlamaCloud Agents | Hosted agent execution + eval | `docs.cloud.llamaindex.ai/agents` |
| CrewAI | CrewAI Studio | Multi-agent crew orchestration | `docs.crewai.com` |

## Feature Parity Matrix

### Capability + Session execution

| Capability | oyatie | AWS Bedrock | GCP Vertex | Azure AI Foundry | LangServe | OpenAI Assistants | LlamaCloud | CrewAI |
|---|---|---|---|---|---|---|---|---|
| Hosted capability dispatch (managed infra) | ✅ | ✅ | ✅ | ✅ | partial (self-host helpers) | ✅ | ✅ | ✅ |
| Multi-turn session memory | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ | ✅ |
| Capability descriptor (declarative) | ✅ | ✅ | ✅ | ✅ | code-only | ✅ | partial | ✅ |
| GitOps capability authoring (PR-reviewed) | ✅ | ❌ | ❌ | ❌ | partial | ❌ | ❌ | ❌ |
| Capability schema validation API (anonymous) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Tool-call dispatch (provider-mediated) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Autonomy + safety (the differentiator)

| Capability | oyatie | AWS Bedrock | GCP Vertex | Azure AI Foundry | LangServe | OpenAI Assistants | LlamaCloud | CrewAI |
|---|---|---|---|---|---|---|---|---|
| First-class autonomy tier per principal (refuse-on-dispatch) | ✅ ADR-0022 | ❌ | ❌ | partial (per-resource IAM) | ❌ | ❌ | ❌ | ❌ |
| Guardrails BEFORE provider call + AFTER | ✅ | ✅ Bedrock Guardrails | partial | ✅ Azure Content Filters | external | ✅ moderation API | external | external |
| Cryptographic audit-chain over invocation | ✅ Ed25519 + Merkle | ❌ | ❌ | partial CloudTrail | ❌ | ❌ | ❌ | ❌ |
| Multispectrum changeset evidence per dispatch | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Provider credentials isolated from runtime pod (defence-in-depth) | ✅ architectural | partial (IAM-bound) | partial | partial | tenant-managed | tenant-managed | tenant-managed | tenant-managed |
| Per-tenant + per-capability rate limit | ✅ | ✅ | ✅ | ✅ | tenant-built | ✅ | ✅ | ✅ |

### Substrate + residency

| Capability | oyatie | AWS Bedrock | GCP Vertex | Azure AI Foundry | LangServe | OpenAI Assistants | LlamaCloud | CrewAI |
|---|---|---|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ (Kubernetes + Redis OSS + Postgres OSS) | ❌ SaaS only | ❌ SaaS only | ❌ SaaS only | ✅ | ❌ SaaS only | ❌ SaaS only | partial |
| Multi-region data-residency | ✅ (11 packs) | ✅ (per-region; ~17 regions) | ✅ (per-region; ~30 regions) | ✅ | n/a | ❌ (US default) | partial | n/a |
| HIPAA BAA | conditional | ✅ | ✅ | ✅ | n/a | ❌ (no Assistants BAA today) | n/a | n/a |
| KR PIPA compliance | conditional | partial | partial | partial | n/a | ❌ | n/a | n/a |
| EU GDPR DPA | ✅ | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | n/a |
| EU AI Act Annex III high-risk handling | ✅ first-class | partial (advisory) | partial (advisory) | partial (advisory) | n/a | n/a | n/a | n/a |

### Operations + integrations

| Capability | oyatie | AWS Bedrock | GCP Vertex | Azure AI Foundry | LangServe | OpenAI Assistants | LlamaCloud | CrewAI |
|---|---|---|---|---|---|---|---|---|
| On-call paging integration | Grafana OnCall (OSS) | CloudWatch | Cloud Monitoring | Azure Monitor | external | ❌ | external | external |
| Multi-language SDK | M01: Rust; M01+1: TS; M02: Py/Go; M03: JVM | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cedar / Rego / OPA policy | ✅ Cedar v4 | partial (IAM) | partial (IAM) | partial | ❌ | ❌ | ❌ | ❌ |
| Tenant isolation (multi-tenant) | ✅ Redis prefix + Postgres RLS + Cedar | per-AWS-account | per-GCP-project | per-Azure-subscription | tenant-built | per-OpenAI-org | per-LlamaCloud-org | partial |

## Quantitative Performance Parity

(All numbers reference 1-hour rolling-window load tests on equivalent workloads. Verify-at-deploy.)

| Metric | oyatie target | AWS Bedrock reference | GCP Vertex reference | Notes |
|---|---|---|---|---|
| Capability dispatch p99 (runtime overhead excluding LLM) | ≤ 50ms | undisclosed; whole round-trip target ≤ 2s including LLM | undisclosed | oyatie targets tightest overhead among self-hosted |
| Session-state hot read p99 | ≤ 10ms | undisclosed | undisclosed | parity hard to verify; oyatie targets known-good Redis 7.4 LTS baseline |
| Session-state cold restore p99 | ≤ 100ms | undisclosed | undisclosed | parity hard to verify |
| Invocation completion event lag p99 | ≤ 80ms | undisclosed | undisclosed | first-party measurement |
| Pool warm-pod cold start | ≤ 500ms | Lambda cold start: 100ms–2s | Cloud Run cold start: 1–4s | oyatie matches Lambda fast-start |
| Per-tenant max concurrent invocations | 100 production / 10000 internal | Bedrock: 50 default + quota | Vertex: 200 default + quota | parity |
| Per-pod max concurrent invocations | 50 default | Bedrock managed | Vertex managed | self-hosted advantage |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Multi-language SDK breadth (Py / Go / JVM) | axis-foundry-runtime | M02–M03 |
| 2 | Tenant-supplied custom-code execution (sandboxed WASM / Firecracker) | council-architecture | subsequent-to-M01-completion ADR |
| 3 | Mobile-app on-call (we have web Grafana OnCall) | ops-sre-reliability | M03 |
| 4 | AI-assisted anomaly detection on invocation patterns (Bedrock Watchdog-class) | axis-foundry-runtime | M04 |
| 5 | Multi-foundation-model dispatch transparency (Bedrock-class side-by-side comparison) | foundry-providers | subsequent-to-M01-completion |

## Key oyatie Differentiators (NOT in any competitor)

1. **Gate-integrated autonomy**: per-tenant tier ceiling enforced at runtime dispatch; refusal sealed + paged; no competitor has this first-class.
2. **GitOps capability authoring**: descriptors in git with PR review; competitors use SaaS-side UI or programmatic API.
3. **Cryptographic audit-chain over invocation**: Ed25519 + Merkle on every event vs competitors' best-effort logs.
4. **OpenSLO-gated capability promotion**: capability versions gated through ADR-0130 observability runways (via foundry-supervisor integration).
5. **Multi-pack residency by design**: 11 region-pinned packs with explicit cross-pack-forbidden + SCC exception.
6. **EU AI Act high-risk classification first-class**: descriptor carries Annex III flag; runtime refuses unclassified high-risk in pack-eu.
7. **Provider credentials never in runtime pod**: architectural separation via foundry-providers (defence-in-depth; competitors have IAM-bound but credentials still reside in invocation pod).

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "First-class autonomy tier gating is unique to oyatie among production-deployed agent runtimes" (true as of 2026-05-17; review bi-annually).
- ✅ "Multi-pack residency exceeds OpenAI Assistants (US-default) and matches Bedrock + Vertex regional offering." (Bedrock has ~17 regions; oyatie has 11 active+conditional; comparable.)
- ✅ "Self-hosted; no vendor lock vs Bedrock/Vertex/Azure AI Foundry/OpenAI/LlamaCloud SaaS-only competitors."
- ✅ "EU AI Act Annex III first-class classification + notified-body engagement gate; competitors are advisory only."

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "oyatie is faster than Bedrock on dispatch latency" — no published Bedrock benchmark; unsourced.
- ❌ "oyatie is HIPAA-compliant out of the box" — conditional on BAA + pack-us-healthcare activation.
- ❌ "We beat Vertex on cost" — depends on workload; do not claim universal.
- ❌ "Better than OpenAI Assistants" — bad-faith comparison absent specific dimension.

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-foundry-runtime |
| 3. Re-run quantitative benchmarks (load tests in staging) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/foundry-runtime/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-FR gate.
- ADR-0022 (autonomy tiers); ADR-0024 (eval harness); ADR-0025 (runtime consolidation); ADR-0123 (hyperscaler-maturity-claim-gate); ADR-0130 (SLO gate).
- Competitor docs as cited inline above.
- OWASP Top 10 for LLM Applications 2025 — `genai.owasp.org/llm-top-10/`.
