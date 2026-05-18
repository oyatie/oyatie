---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-control-plane + council-architecture
deciders: axis-foundry-control-plane, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0139, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/foundry-supervisor/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-FND-SUP)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (foundry-supervisor µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading agentic-control-plane products. Drives the `oya-foundry-fitness-hyperscaler-maturity-claims` gate (ADR-0123 HG-FND-SUP) and informs gtm-customer-success.

## Competitor Set

| Competitor | Product | Primary differentiator | Source |
|---|---|---|---|
| AWS Bedrock Agents | control plane + Guardrails | mature canary + Guardrails kill-switch | `docs.aws.amazon.com/bedrock/latest/userguide/agents.html` |
| Anthropic Claude | admin API + Workspaces | autonomy + safety profile + escalation | `docs.anthropic.com/en/docs/admin-api` |
| OpenAI Assistants API | admin surface | assistant lifecycle + tool admission + run cancel | `platform.openai.com/docs/api-reference/assistants` |
| Google Vertex AI Agent | Agent Builder admin | versioning + canary + cancel + audit | `cloud.google.com/vertex-ai/docs/generative-ai/agents/` |
| Databricks Mosaic AI Gateway | control plane | admit + rate-limit + kill-switch | `docs.databricks.com/en/generative-ai/` |
| Hugging Face Inference Endpoints | minimal control plane | basic versioning + auth | `huggingface.co/docs/inference-endpoints` |
| Microsoft Azure AI Studio | agent admin | versioning + content filters | `learn.microsoft.com/en-us/azure/ai-studio/` |

## Feature Parity Matrix

### Capability lifecycle

| Capability | oyatie | AWS Bedrock | Anthropic | OpenAI | Vertex AI | Databricks | HF | Azure |
|---|---|---|---|---|---|---|---|---|
| GitOps capability authoring (YAML in tenant repo, PR-reviewed) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Canary rollout (1 % → 10 % → 50 % → 100 %) | ✅ | ✅ | partial | ❌ | ✅ | partial | ❌ | partial |
| SLO-gated phase advance (per ADR-0139) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Automated rollback on production burn-rate breach | ✅ | partial (manual rollback only) | manual | manual | partial | manual | manual | manual |
| Per-component release pointer (per µservice) | ✅ | per-agent | per-assistant | per-assistant | per-agent | per-endpoint | per-endpoint | per-agent |
| Schema validation API (anonymous) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Kill-switch + safety

| Capability | oyatie | AWS Bedrock | Anthropic | OpenAI | Vertex AI | Databricks | HF | Azure |
|---|---|---|---|---|---|---|---|---|
| Multi-scope kill-switch (fleet / tenant / capability / agent) | ✅ | Guardrails (capability-scope) | partial | partial (run-cancel) | partial | partial | ❌ | partial |
| Sub-second engage p99 | ✅ ≤ 1 s | ~2 s | unknown | run-cancel ~5 s | unknown | unknown | n/a | unknown |
| 2-person rule for fleet-wide | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Cryptographic engagement audit (Ed25519) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Tenant DPO-initiated own-scope engage | ✅ | per-account admin | per-workspace | per-org | per-project | per-workspace | n/a | per-resource |

### Autonomy policy

| Capability | oyatie | AWS Bedrock | Anthropic | OpenAI | Vertex AI | Databricks | HF | Azure |
|---|---|---|---|---|---|---|---|---|
| Default-deny Cedar policy | ✅ Cedar v4 | IAM (default-allow with explicit deny) | system-prompt-based | system-prompt-based | IAM (default-allow) | UC + IAM | basic auth | RBAC |
| Per-invocation precondition check | ✅ ≤ 15 ms p99 | partial (Guardrails inline) | partial (system prompt) | partial | partial | partial | n/a | partial |
| Per-tenant entitlement store | ✅ OpenBao | per-account quotas | per-workspace | per-org | per-project | per-workspace | n/a | per-resource |
| Tier escalation requires DPO + ops-security signatures | ✅ | manual workflow | manual | manual | manual | manual | n/a | manual |

### Audit + compliance

| Capability | oyatie | AWS Bedrock | Anthropic | OpenAI | Vertex AI | Databricks | HF | Azure |
|---|---|---|---|---|---|---|---|---|
| Cryptographic Merkle audit-chain | ✅ Ed25519 | CloudTrail (unsigned) | unsigned | unsigned | Cloud Audit Logs (unsigned) | UC audit (unsigned) | basic | Activity Log (unsigned) |
| Per-tenant audit isolation | ✅ Postgres RLS + Cedar | per-account | per-org | per-org | per-project | per-workspace | per-account | per-tenant |
| EU AI Act Art. 12 record-keeping | ✅ | partial | partial | partial | partial | partial | partial | partial |
| EU AI Act Art. 27 FRIA support | ✅ (per-tenant overlay) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| HIPAA BAA | conditional (pack-us-healthcare) | ✅ | partial | partial | ✅ | ✅ | ❌ | ✅ |
| KR PIPA compliance | conditional | partial | ❌ | ❌ | partial | partial | ❌ | partial |
| GDPR DPA | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Substrate

| Capability | oyatie | AWS Bedrock | Anthropic | OpenAI | Vertex AI | Databricks | HF | Azure |
|---|---|---|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ (Postgres + Valkey + kube-rs) | ❌ (SaaS only) | ❌ (SaaS only) | ❌ (SaaS only) | ❌ (SaaS only) | ❌ (SaaS only) | partial | ❌ |
| Multi-region data residency | ✅ 11 packs | ✅ | ✅ | partial | ✅ | ✅ | partial | ✅ |
| Kubernetes Operator pattern | ✅ kube-rs | ❌ (proprietary) | ❌ (proprietary) | ❌ (proprietary) | ❌ (proprietary) | partial (Spark) | partial | partial |
| Cedar policy integration | ✅ | partial (IAM) | ❌ | ❌ | partial (IAM) | partial (UC) | ❌ | partial (RBAC) |

### Operations + integrations

| Capability | oyatie | AWS Bedrock | Anthropic | OpenAI | Vertex AI | Databricks | HF | Azure |
|---|---|---|---|---|---|---|---|---|
| On-call paging | Grafana OnCall (OSS) | CloudWatch + 3rd-party | unknown | unknown | Cloud Monitoring + 3rd-party | Databricks Alerts | n/a | Action Groups |
| Multi-language SDK | M01: Rust; M01+1: TS; M02: Py/Go; M03: JVM | extensive | extensive | extensive | extensive | extensive | extensive | extensive |
| Tenant isolation | ✅ Postgres RLS + Valkey ACL + Cedar | per-account | per-org | per-org | per-project | per-workspace | per-account | per-tenant |
| Cost budget per capability + per-tenant | ✅ | ✅ | per-org | per-org | per-project | per-workspace | per-endpoint | per-resource |

## Quantitative Performance Parity

(All numbers reference 30-day rolling-window evaluations on equivalent workloads.)

| Metric | oyatie target | AWS Bedrock | Anthropic | OpenAI | Vertex AI | Databricks |
|---|---|---|---|---|---|---|
| Kill-switch engage p99 | ≤ 1 s | ~2 s (Guardrails) | unknown | run-cancel ~5 s | unknown | unknown |
| Capability admit→100% rollout p99 | ≤ 5 min | ~10 min | ~30 min | manual | ~10 min | manual |
| Autonomy-precondition eval p99 | ≤ 15 ms (Cedar) | ~100 ms (Guardrails inline) | system-prompt overhead ~500 ms | similar | ~50 ms | ~100 ms |
| Per-component release pointer scope | per-µservice | per-agent | per-assistant | per-assistant | per-agent | per-endpoint |
| Cross-tenant query refusal | server-side (RLS + Cedar) | per-account IAM | per-org auth | per-org auth | per-project IAM | per-workspace UC |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Multi-language SDK breadth (Py / Go / JVM) | axis-foundry-control-plane | M02–M03 |
| 2 | Tenant-facing GUI for capability authoring (we have YAML PR; Bedrock has UI) | application + workflow-studio | M02–M03 (workflow-studio surface) |
| 3 | AI-assisted capability optimization (Bedrock Studio peer) | axis-foundry-control-plane | M04 |
| 4 | Mobile-app on-call (Grafana OnCall has web; AWS has mobile) | ops-sre-reliability | M03 |

## Key oyatie Differentiators (NOT in any competitor)

1. **GitOps capability authoring with PR review + signed commits** — capability definitions live in tenant-owned git repos; admit-loop validates against schema + autonomy-tier + cost budget + Cedar.
2. **SLO-gated rollout** — phase advance gated by `observability` `EligibilityChanged` verdict; competitors deploy on click.
3. **Cryptographic audit-chain over supervision events** — Ed25519 + Merkle per ADR-0028; competitors emit unsigned admin events.
4. **2-person rule on fleet-wide kill-switch** — competitors permit single-admin engagement.
5. **EU AI Act Art. 27 FRIA + Art. 12 record-keeping native** — per-tenant overlay required at first high-risk Annex III capability; competitors leave compliance to tenant.
6. **Multi-pack residency by design** — 11 region-pinned packs with explicit cross-pack-forbidden + SCC exception path.
7. **Sub-second multi-scope kill-switch** — engage p99 ≤ 1 s across fleet / tenant / capability / agent; AWS Bedrock Guardrails ~2 s.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "GitOps capability authoring is unique to oyatie" (true as of 2026-05-17; bi-annual review).
- ✅ "Sub-second kill-switch latency exceeds AWS Bedrock Guardrails ~2 s p99" (cite the AWS public latency claim).
- ✅ "Cryptographic Merkle audit-chain native to control plane" (per ADR-0028; competitors emit unsigned admin events).
- ✅ "EU AI Act Art. 27 FRIA + Art. 12 record-keeping native" (cite ADR-0131 + this matrix).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "oyatie is faster than AWS Bedrock on every metric" (per-workload variance; do not claim universal).
- ❌ "oyatie is HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation).
- ❌ "We beat all competitors on cost" (depends on workload shape).
- ❌ "Our autonomy policy is provably safe" (Cedar is best-effort default-deny; do not overclaim).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-foundry-control-plane |
| 3. Re-run quantitative benchmarks (load tests in staging) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/foundry-supervisor/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-FND-SUP gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0139 (SLO-gated promotion).
- ADR-0131 (foundry split).
- ADR-0133 (industry-best-practice conformance).
- Competitor docs as cited inline above.
