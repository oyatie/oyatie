---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-guardrails + council-architecture
deciders: axis-foundry-guardrails, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0131, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/foundry-guardrails/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-FGUARD gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (foundry-guardrails µservice)

## Purpose

Quantitative + qualitative parity vs industry-leading agent-safety products. Drives the `oya-governance-hyperscaler-maturity-claims` gate (per ADR-0123 HG-FGUARD) and tells gtm what to say + NOT say in tenant conversations. Re-validated bi-annually.

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| AWS | Bedrock Guardrails | Per-policy denied topics + word filters + sensitive-info filter + content filter (hate/insult/sexual/violence/misconduct/prompt-attack) + contextual grounding | `docs.aws.amazon.com/bedrock/latest/userguide/guardrails.html` |
| Anthropic | Claude Constitutional AI + runtime safety | Constitutional principles at training; runtime prompt_injection + harmful-content classifiers | `anthropic.com/research/constitutional-ai-harmlessness-from-ai-feedback` |
| OpenAI | OpenAI Moderation API | Multi-label categories (hate / harassment / self-harm / sexual / violence + subcategories); free moderation | `platform.openai.com/docs/guides/moderation` |
| Microsoft | Azure AI Content Safety | Text + image moderation; prompt-shield (jailbreak); protected-material detection | `learn.microsoft.com/azure/ai-services/content-safety/` |
| Google | Perspective API + Vertex AI Safety | Toxicity scoring; Vertex content-safety filters; Vertex AI Studio safety controls | `developers.perspectiveapi.com` + `cloud.google.com/vertex-ai/generative-ai/docs/multimodal/configure-safety-attributes` |
| NVIDIA | NeMo Guardrails | Programmable Colang flow-based guardrails; topical / dialog / fact-checking rails | `github.com/NVIDIA/NeMo-Guardrails` |
| Meta | Llama Guard / Prompt Guard | Open-weight classifier models; multi-category prompt + response safety scoring | `ai.meta.com/research/publications/llama-guard-llm-based-input-output-safeguard-for-human-ai-conversations/` |

## Feature Parity Matrix

### Pre-invocation prompt classification

| Capability | oyatie | AWS Bedrock | Anthropic | OpenAI | Azure | Google | NVIDIA | Meta |
|---|---|---|---|---|---|---|---|---|
| PII detection | ✅ | ✅ | partial | ❌ | ✅ | ✅ | ✅ | partial |
| PHI detection (HIPAA-aware) | ✅ pack-us-hc | ✅ Bedrock medical | partial | ❌ | ✅ medical filter | ✅ DLP | partial | partial |
| Prompt injection detection | ✅ | ✅ | ✅ | partial | ✅ Prompt Shield | ✅ | ✅ | ✅ Prompt Guard |
| Jailbreak detection | ✅ ensemble | ✅ | ✅ | partial | ✅ | ✅ | ✅ | ✅ |
| Data-class tagging on prompt | ✅ Bominal ADR-0028 taxonomy | ❌ | ❌ | ❌ | partial | partial | ❌ | ❌ |
| Per-pack regulatory category (KR PIPA Art. 23, EU AI Act, HIPAA, etc.) | ✅ pack-overlay | partial (regional) | ❌ | ❌ | partial | partial | ❌ | ❌ |
| GitOps rule + Cedar overlay authoring | ✅ | partial (Bedrock console + API) | ❌ | ❌ | partial | ❌ | ✅ Colang | ❌ |

### Post-output validation

| Capability | oyatie | AWS Bedrock | Anthropic | OpenAI | Azure | Google | NVIDIA | Meta |
|---|---|---|---|---|---|---|---|---|
| Output safety classification | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Data exfiltration detection | ✅ | ✅ sensitive-info | partial | ❌ | partial | ✅ DLP | partial | partial |
| Secret-leak detection (API keys / tokens in output) | ✅ | partial | partial | ❌ | partial | partial | ❌ | ❌ |
| Hallucinated-tool-args detection | ✅ | ❌ | ❌ | ❌ | partial | ❌ | ✅ (fact-checking rails) | ❌ |
| Contextual grounding (output vs source) | partial M01+1 | ✅ | ✅ | ❌ | partial | partial | ✅ | ❌ |
| AI-slop pattern detection | ✅ (oyatie unique) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Autonomy + Policy

| Capability | oyatie | AWS | Anthropic | OpenAI | Azure | Google | NVIDIA | Meta |
|---|---|---|---|---|---|---|---|---|
| Autonomy-tier gate (ADR-0022) | ✅ Cedar v4 | ❌ | ❌ | ❌ | ❌ | partial (IAM) | partial (Colang topical rails) | ❌ |
| Per-tenant Cedar overlay (entitlement composition) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | partial (per-config) | ❌ |
| Default-deny base policy | ✅ | partial | ❌ | ❌ | ❌ | ❌ | partial | ❌ |
| Multi-detector ensemble (heuristic + classifier + LLM-judge) | ✅ | partial (Bedrock multi-category) | partial | partial | partial | partial | partial | partial |
| Tenant FP escalation budget | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Shadow→enforce rule rollout | ✅ | partial | ❌ | ❌ | partial | partial | ✅ test framework | ❌ |

### Evidence + Audit

| Capability | oyatie | AWS | Anthropic | OpenAI | Azure | Google | NVIDIA | Meta |
|---|---|---|---|---|---|---|---|---|
| Per-decision audit-chain seal (cryptographic non-repudiation) | ✅ Ed25519 + Merkle | partial (CloudTrail) | ❌ | ❌ | partial (Azure Monitor) | partial (Cloud Audit Logs) | ❌ | ❌ |
| Per-decision explanation (Art. 22 / EU AI Act Art. 13) | ✅ structured `block_reason` + `cedar_policy_ids[]` + `classifier_model_versions{}` | partial | partial | ❌ | partial | partial | ✅ verbose | ❌ |
| Auditor JIT scoped tokens | ✅ | partial | ❌ | ❌ | partial | partial | ❌ | ❌ |
| Multispectrum changeset evidence | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Substrate

| Capability | oyatie | AWS | Anthropic | OpenAI | Azure | Google | NVIDIA | Meta |
|---|---|---|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ (in-house classifier-serving) | ❌ (SaaS only) | ❌ (SaaS only) | ❌ (SaaS only) | ❌ (SaaS only) | ❌ (SaaS only) | ✅ (OSS) | ✅ (open weights) |
| Multi-region data-residency (11 packs) | ✅ | ✅ | ✅ | partial | ✅ | ✅ | n/a | n/a |
| HIPAA BAA | conditional | ✅ | partial | partial | ✅ | ✅ | n/a | n/a |
| KR PIPA compliance | conditional | partial | partial | ❌ | partial | partial | n/a | n/a |
| EU GDPR DPA | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | n/a | n/a |
| EU AI Act risk-management system | ✅ DPIA + threat-model | partial | partial | partial | partial | partial | n/a | n/a |
| Cosign-signed classifier artifacts | ✅ | partial | partial | ❌ | partial | partial | partial | partial |

### Operations + SDK

| Capability | oyatie | AWS | Anthropic | OpenAI | Azure | Google | NVIDIA | Meta |
|---|---|---|---|---|---|---|---|---|
| Multi-language SDK (Rust + TS + Python + Go + JVM + .NET) | M01: Rust; M01+1: TS; M02: Py/Go; M03: JVM | ✅ AWS SDK | ✅ | ✅ | ✅ | ✅ | partial (Python) | partial (Python) |
| Streaming decision API | ✅ gRPC | partial | ❌ | ❌ | partial | partial | ❌ | ❌ |
| Cedar / Rego / OPA policy integration | ✅ Cedar v4 | partial (IAM Cedar) | ❌ | ❌ | partial | partial (IAM) | ❌ | ❌ |
| Tenant-isolated SaaS shape | ✅ | ✅ | per-org | per-org | per-tenant | per-project | n/a | n/a |

## Quantitative Performance Parity

Headline latency comparison (vendor-published + oyatie targets; verify at deploy time).

| Metric | oyatie target | AWS Bedrock Guardrails | OpenAI Moderation | Azure Content Safety | Google Perspective |
|---|---|---|---|---|---|
| Pre-invocation classify p99 | ≤ 50ms | ~50-100ms (region-dependent) | ~80ms (single moderation call) | ~80-150ms | ~50-100ms |
| Post-output validate p99 | ≤ 100ms | ~80-150ms | ~80ms | ~80-150ms | ~50-100ms |
| Multi-category coverage | 8+ (toxicity/self-harm/sexual/violence/minors/hate/weapons/illegal) + AI-slop + jailbreak + PII/PHI | ~6 categories | ~6 categories | ~5 categories | ~6 categories |
| Cedar-policy eval p99 | ≤ 10ms | n/a | n/a | n/a | n/a |

Verify-at-deploy: vendor SLAs change; reconfirm against vendor public docs.

## Quantitative Recall + Precision Targets

Per-category recall + precision on oyatie baseline-fixture set (in `tests/jailbreak/baseline_fixtures.rs` + `tests/content-safety/baseline_fixtures.rs`):

| Category | Target recall | Target precision | Notes |
|---|---|---|---|
| Jailbreak (composite) | ≥ 0.95 | ≥ 0.90 | ensemble-bounded; LLM-judge fallback for ambiguous |
| PII detection | ≥ 0.99 | ≥ 0.95 | very high recall required (GDPR / KR PIPA) |
| PHI detection (pack-us-hc) | ≥ 0.999 | ≥ 0.95 | HIPAA-critical |
| Toxicity | ≥ 0.92 | ≥ 0.88 | Perspective-API parity |
| Self-harm | ≥ 0.95 | ≥ 0.92 | safety-critical |
| Sexual | ≥ 0.95 | ≥ 0.90 | |
| Violence | ≥ 0.90 | ≥ 0.85 | |
| Minors | ≥ 0.99 | ≥ 0.95 | safety-critical; pack-eu CSAM duties |
| Secret-leak | ≥ 0.95 | ≥ 0.90 | shared library with oya-governance-evidence-secret-scan |
| AI-slop pattern (catalogue) | ≥ 0.85 | ≥ 0.80 | oyatie unique; new category |
| Hallucinated-tool-args | ≥ 0.85 | ≥ 0.80 | post-output |

Recall + precision validated on baseline-fixture set at every classifier-model rollout via shadow→enforce LEAN lane.

## Parity Gaps (oyatie ahead)

1. **Per-tenant Cedar overlay**: competitors are flat policy; oyatie's overlay-on-default-deny is differentiator.
2. **Audit-chain Ed25519**: competitors emit log lines; oyatie cryptographic non-repudiation.
3. **AI-slop pattern coverage**: unique to oyatie via `docs/quality/ai-slop-defense/` catalogue.
4. **GitOps Cedar overlay authoring**: tenant operators author via PR; CODEOWNERS + branch-protection; competitors mostly UI/API.
5. **Per-tenant FP escalation budget**: humane operational primitive missing from all competitors.
6. **EU AI Act risk-management system**: this DPIA + threat-model + PRD form the system; few competitors yet have explicit AI Act posture.
7. **Multispectrum changeset evidence**: per-IP audit posture differentiator.

## Parity Gaps (oyatie behind)

1. **Image content moderation**: M01 ships text-only; Azure + AWS + Google have image. **Plan**: M02-onward image classifier.
2. **Contextual grounding for RAG**: AWS Bedrock + Anthropic have built-in grounding; oyatie's `hallucinated-tool-args` covers a subset. **Plan**: M01+1 grounding detector.
3. **Vendor-of-record audit pre-cert**: SOC 2 / ISO 27001 / HIPAA BAA already attained by big-cloud competitors; oyatie at audit-target for M02. **Plan**: per `compliance.md` audit cadence.
4. **Open-weight classifier model count**: Meta has Llama Guard 7B+; oyatie ships smaller distilled in-house variants. **Trade-off accepted**: smaller models for lower latency + cost; ensemble + LLM-judge for hard cases.

## HG-FGUARD Gate Components

Per ADR-0123 `/specs/hyperscaler-gates.json` registers HG-FGUARD with these conformance checks:

| Check | Pass criterion |
|---|---|
| Pre-invocation classify p99 ≤ 50ms | load-test |
| Post-output validate p99 ≤ 100ms | load-test |
| Recall targets per category met | baseline-fixture validation |
| Cedar default-deny enforced | LEAN lane |
| Multi-detector ensemble in place | architecture review |
| Audit-chain seal on every decision | integration test |
| Per-tenant FP escalation budget functional | tenant e2e |
| Shadow→enforce rule rollout | rule rollout e2e |
| Multi-region data-residency (pack-pinning) | LEAN lane |
| Cosign-signed classifier artifacts | pod-start verification |

## Verification

- `cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims` — exit 0; HG-FGUARD green.
- Bi-annual competitor-feature refresh: update this matrix.
- Quarterly red-team: validate recall targets remain met.
- Annual external benchmark: third-party comparison published in `evidence/benchmarks/`.

## References

- ADR-0123 Hyperscaler maturity claim gate.
- ADR-0131 Per-microservice flat layout (Foundry split).
- ADR-0133 Industry-best-practice conformance program.
- ADR-0140 Cedar policy substrate.
- `microservices/foundry-guardrails/PRD.md` (§Competitive Benchmark).
- `/specs/hyperscaler-gates.json`.
- AWS Bedrock Guardrails — `docs.aws.amazon.com/bedrock/latest/userguide/guardrails.html`.
- Anthropic Constitutional AI — `anthropic.com/research/constitutional-ai-harmlessness-from-ai-feedback`.
- OpenAI Moderation API — `platform.openai.com/docs/guides/moderation`.
- Microsoft Azure AI Content Safety — `learn.microsoft.com/azure/ai-services/content-safety/`.
- Google Perspective API — `developers.perspectiveapi.com`.
- Google Vertex AI Safety — `cloud.google.com/vertex-ai/generative-ai/docs/multimodal/configure-safety-attributes`.
- NVIDIA NeMo Guardrails — `github.com/NVIDIA/NeMo-Guardrails`.
- Meta Llama Guard — `ai.meta.com/research/publications/llama-guard-llm-based-input-output-safeguard-for-human-ai-conversations/`.
- Meta Prompt Guard — `ai.meta.com/research/publications/prompt-guard/`.
