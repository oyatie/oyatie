---
doc_class: CompetitorParityMatrix
title: Competitor Parity Matrix
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + council-strategy
deciders: axis-foundry, council-strategy, council-architecture
related_adrs: [ADR-0024, ADR-0026, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/intelligence-eval/PRD.md
  - microservices/intelligence-eval/policy/dp-analysis.md
review_cadence: quarterly + on new competitor release
doc_status: published
---

# Competitor Parity Matrix (foundry-eval µservice)

## Purpose

Track per-feature parity vs the leading commercial eval substrates: OpenAI Evals, Anthropic internal evals, LangSmith, Patronus AI, Braintrust, Inspect AI (UK AISI). Updated quarterly with citation snapshots. Used by leadership for go/no-go on capability and feature priority.

## Comparison Dimensions

| Dimension | foundry-eval | OpenAI Evals | Anthropic evals | LangSmith | Patronus AI | Braintrust | Inspect AI |
|---|---|---|---|---|---|---|---|
| Self-hostable | YES (full stack) | NO (GitHub-only + manual run) | NO (internal) | NO (SaaS) | NO (SaaS) | NO (SaaS) | YES (open-source) |
| Eval-set signing (Cosign + Rekor) | YES | NO | unknown internal | NO | NO | NO | NO |
| Adversarial cohort (prompt-injection / data-class / autonomy / tool-exfil) | YES (mandatory 4-sub-cohort) | partial (community-extensible) | YES (internal) | partial | YES (focus area) | partial | YES (focus area) |
| Linguistic cohort (KR + JP + EN minimum) | YES (mandatory) | NO | unknown | NO | NO | NO | NO |
| Replay against past traces (≤ 100ms divergence) | YES (deterministic-seed enforced) | NO | unknown | YES (no divergence-tolerance) | NO | NO | NO |
| Publish-gate integration (refuse capability publish on miss) | YES (runtime-enforced) | NO | unknown | NO (CI-only) | NO | NO | NO |
| A/B routing-preference gate | YES | NO | unknown | partial | NO | partial | NO |
| In-house cutover decision substrate | YES (ADR-0026) | NO | unknown | NO | NO | NO | NO |
| Per-subject DEK + DSR cascade shred | YES | NO | NO (no DSR feature) | NO | NO | NO | NO |
| EU AI Act §15 / §17 evidence emission | YES (by construction) | NO | unknown | NO | partial | NO | NO |
| NIST AI RMF function-mapping | YES (Govern / Map / Measure / Manage) | NO | partial | partial | YES | NO | NO |
| Multi-provider eval (cross-provider parity) | YES | partial (provider-pluggable) | partial | YES | YES | YES | YES |
| LLM-as-judge (with κ ≥ 0.7 rotation) | YES | partial | unknown | YES | YES | YES | partial |
| Per-tenant residency + KEK isolation | YES (pack-pinned) | NO | NO | partial (region pinning) | partial | NO | NO |
| Differential-privacy on cross-tenant aggregates | YES (ε ≤ 1) | NO | NO | NO | NO | NO | NO |
| Cosign supply-chain for eval-set | YES + Rekor inclusion-proof | NO | unknown | NO | NO | NO | NO |
| Open-source repo | NO (internal) | YES | NO | NO | NO | NO | YES |
| Hosted SaaS | NO | NO | NO | YES | YES | YES | NO |

## Parity Gap Closure Priority

Per `PRD.md` §"Competitive Benchmark", ordered by priority:

1. **(closed in M01-P01)** Self-hostable + signed eval-sets + per-subject-keyed replay store. Closed by foundry-eval baseline.
2. **(closed in M01-P01)** Capability-publish gate enforcement (no competitor has this at runtime layer).
3. **(M02-P01)** In-house-model cutover decision substrate (ADR-0026); per-cohort parity-win → cutover-eligibility. Architectural placement in PRD; implementation in M02-P01.
4. **(closed in M01-P01)** Replay-determinism ≤ 100ms divergence tolerance.
5. **(closed in M01-P01)** EU AI Act §15+§17 evidence-grade out-of-box.

## Per-Competitor Strategic Note

### OpenAI Evals (github.com/openai/evals)

- Open-source; GitHub-checked-in YAML eval files; community-extensible.
- No runtime gate; no signing; no DSR; no in-house cutover concept.
- We out-class on enterprise + regulated-vertical operations.

### Anthropic internal evals

- Best-in-class adversarial cohort design (responsible-scaling policy).
- Internal-only; not productised.
- We adopt their adversarial-pattern catalog by reference (cite + cohort-import where licensed).

### LangSmith

- Strong on trace-based eval + LLM-as-judge.
- SaaS; tenant-data flows to LangChain Inc.; not self-hostable in any meaningful sense.
- No publish-gate; no DSR; no AI Act evidence.

### Patronus AI

- Adversarial + safety cohort focus.
- Hosted; per-API-call charged.
- No runtime gate; no signing.

### Braintrust

- Per-experiment scoring + LLM-as-judge.
- CI-integration focused.
- Hosted; no self-hosting.

### Inspect AI (UK AISI)

- Open-source; UK government-backed.
- Adversarial + autonomy focus.
- Strong reproducibility primitives.
- No publish-gate; no DSR; no AI Act evidence-emission by construction.
- We adopt Inspect cohort patterns by reference (open-source).

## Publication Surface (DP-noised)

Per `policy/dp-analysis.md`, the competitor-parity-matrix dashboard (`dashboards/parity-trend.json`) publishes cross-capability competitor-vs-foundry-eval deltas with ε ≤ 1 per published aggregate. Per-tenant per-capability deltas are tenant-scoped only.

## Verification

- Quarterly review (axis-foundry + council-strategy).
- On every competitor release: re-verify matrix rows.
- Annual external benchmark (when contracted with red-team firm or AI eval consortium).

## References

- ADR-0024 (eval harness; design context for parity gap analysis).
- ADR-0026 (in-house model substrate; cutover decision).
- OpenAI Evals: `github.com/openai/evals`.
- Anthropic responsible-scaling policy.
- LangSmith: `docs.langchain.com/langsmith/evaluation`.
- Patronus AI: `patronus.ai/docs`.
- Braintrust: `braintrust.dev/docs`.
- Inspect AI: `github.com/UKGovernmentBEIS/inspect_ai`.
- Apollo Research evaluation framework.
