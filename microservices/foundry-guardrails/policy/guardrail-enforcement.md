---
doc_class: PolicySpec
title: Guardrail Enforcement Specification
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry-guardrails
deciders: council-architecture, ops-security, axis-foundry-guardrails, council-privacy
related_adrs: [ADR-0022, ADR-0028, ADR-0117, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/foundry-guardrails/threat-model.md
  - microservices/foundry-guardrails/dpia.md
  - microservices/foundry-guardrails/policy/tenant-isolation.md
  - microservices/foundry-guardrails/policy/data-residency.md
  - microservices/foundry-guardrails/policy/*.cedar
review_cadence: quarterly + on every classifier-model rollout + on every Cedar bundle change
doc_status: published
---

# Guardrail Enforcement Specification (foundry-guardrails µservice)

## Purpose

Authoritative reference for HOW foundry-guardrails decides allow / block / redact + WHY default-deny applies + HOW Cedar overlays compose + HOW shadow→enforce rollout protects rule regressions. This document is reviewed by SOC 2 / ISO 27001 / HIPAA / EU AI Act / KR PIPA examiners asking "how does foundry-guardrails enforce the safety floor?"

## Default-Deny Posture

Every action — classify / validate / autonomy / read / mutation — is refused unless an explicit Cedar permit matches. The base Cedar fragment at `policy/cedar-base.cedar` declares:

```cedar
forbid (
  principal,
  action,
  resource
);
```

Per-action permits live in the four Cedar fragments (`tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`) PLUS per-tenant overlays loaded at runtime from Postgres rule-store.

Cedar's deny-overrides semantics: any matching `forbid` overrides any matching `permit`. Defence-in-depth.

CI lane `oya-foundry-fitness-cedar-default-deny-enforced` validates the base fragment is present in every deployed bundle.

## Detector Ensemble

For categories where multiple detectors exist, the ensemble runs:

1. **Heuristic** (regex + ngram + canonicalisation) — always; cheap; first-pass.
2. **Classifier model** (in-cluster ONNX) — always for category coverage where a model exists.
3. **LLM-judge fallback** (via foundry-providers) — invoked ONLY when ensemble disagreement among (1) + (2); 5% rate cap globally; per-tenant budget.

Disagreement rules:
- If heuristic + classifier AGREE → fast-path verdict.
- If only one is definite (other ambiguous / no-signal) → use the definite one with `confidence_floor` adjustment.
- If both are definite AND disagree → invoke LLM-judge; verdict = LLM-judge output.

Budget exhaustion behaviour:
- LLM-judge invocation when budget exhausted → **fail-closed (block + block_reason=llm_judge_budget_exceeded)**. Per ADR-0022 spirit (when in doubt, deny).
- Tenant operator can request budget increase via FP escalation workflow.

## Block Reason Taxonomy

Every block emits a structured `block_reason` from the closed enum below; tenants receive enough information to debug without leaking policy internals:

| block_reason | When raised | Cedar `block_kind` |
|---|---|---|
| `pii` | PII detected in prompt or output | content_safety |
| `phi` | PHI detected; tenant lacks BAA entitlement | content_safety |
| `jailbreak_injection` | Jailbreak ensemble verdict ≥ threshold | jailbreak |
| `content_safety_toxicity` | Toxicity ≥ pack threshold | content_safety |
| `content_safety_self_harm` | Self-harm category ≥ threshold | content_safety |
| `content_safety_sexual` | Sexual category ≥ threshold | content_safety |
| `content_safety_violence` | Violence category ≥ threshold | content_safety |
| `content_safety_minors` | Minors-protection category match | content_safety |
| `content_safety_hate` | Hate category ≥ threshold | content_safety |
| `content_safety_weapons` | Weapons category ≥ threshold | content_safety |
| `content_safety_illegal` | Illegal-activity category ≥ threshold | content_safety |
| `autonomy_tier_exceeded` | Request above effective ceiling per ADR-0022 | autonomy |
| `policy_deny` | Tenant Cedar overlay denies | policy |
| `llm_judge_budget_exceeded` | LLM-judge needed; budget gone | budget |
| `ai_slop` | AI-slop pattern density ≥ threshold | quality |

## Pack-Threshold Matrix

Per-pack default thresholds (per `policy/data-residency.md` per-pack overlay). Lower threshold = stricter; higher = more permissive.

| Category | pack-kr | pack-eu | pack-us | pack-us-hc | pack-jp | pack-sg | pack-au | pack-in | pack-br | pack-ae | pack-ksa |
|---|---|---|---|---|---|---|---|---|---|---|---|
| toxicity | 0.70 | 0.65 | 0.70 | 0.65 | 0.75 | 0.70 | 0.70 | 0.70 | 0.70 | 0.65 | 0.60 |
| self_harm | 0.50 | 0.50 | 0.50 | 0.40 | 0.50 | 0.50 | 0.50 | 0.50 | 0.50 | 0.50 | 0.50 |
| sexual | 0.60 | 0.60 | 0.65 | 0.55 | 0.60 | 0.60 | 0.60 | 0.55 | 0.60 | 0.50 | 0.50 |
| violence | 0.65 | 0.65 | 0.65 | 0.60 | 0.65 | 0.65 | 0.65 | 0.65 | 0.65 | 0.60 | 0.60 |
| minors | 0.40 | 0.40 | 0.40 | 0.40 | 0.40 | 0.40 | 0.40 | 0.40 | 0.40 | 0.40 | 0.40 |
| hate | 0.55 | 0.50 | 0.55 | 0.55 | 0.60 | 0.55 | 0.55 | 0.55 | 0.55 | 0.50 | 0.50 |
| weapons | 0.60 | 0.55 | 0.65 | 0.65 | 0.60 | 0.55 | 0.55 | 0.55 | 0.55 | 0.55 | 0.55 |
| illegal | 0.60 | 0.60 | 0.60 | 0.60 | 0.60 | 0.55 | 0.55 | 0.55 | 0.60 | 0.55 | 0.55 |
| jailbreak | 0.60 | 0.60 | 0.60 | 0.55 | 0.60 | 0.60 | 0.60 | 0.60 | 0.60 | 0.60 | 0.60 |
| ai_slop | 0.75 | 0.75 | 0.75 | 0.70 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 | 0.75 |

Thresholds are starting defaults; tenants may compose Cedar overlays tightening (never loosening below pack default) per their DPA.

## Shadow→Enforce Rule Rollout

Per ADR-0114 precedent + IP-014.

| Stage | Duration | Action | Promotion gate |
|---|---|---|---|
| Author | per PR | Rule authored via git PR; CODEOWNERS approves | merge → status `shadow` |
| Shadow | ≥ 7 days (default; pack-us-healthcare ≥ 14d for PHI rules) | Rule emits shadow decisions; live unaffected; shadow-vs-enforce delta dashboard tracks | rule-author + ops-security approval; LEAN lane `shadow-enforce-promotion-readiness` |
| Enforce | indefinite | Rule live; affects invocations | sunset path: rule → `sunsetted` (12mo deprecation notice) |

Shadow phase mandatory for all safety-bearing categories (toxicity / self-harm / sexual / violence / minors). Optional for AI-slop, ai_slop, and non-safety categories at rule-author discretion.

## Per-Tenant False-Positive Escalation Budget

Per autonomy tier (per `tenant_scope` enum in `tenant-isolation.md`):

| Tenant scope | FP budget per month |
|---|---|
| trial | 50 |
| sandbox | 50 |
| production | 500 |
| internal | 5000 |

Budget mechanism per IP-014:
1. Tenant operator marks a block as FP via REST (`POST /v1/decisions/{id}/mark-false-positive`); requires `reason` field (≥ 10 chars).
2. Budget counter increments; remaining returned.
3. FP escalation emits `FalsePositiveEscalated` event consumed by rule-author dashboard queue.
4. Budget exhausted: subsequent FP-marks return 429; rule-author engagement required.
5. Monthly rollover: budget resets at calendar-month boundary in pack timezone.

## LLM-judge Budget (per tenant per hour)

| Tenant scope | Soft budget / hour | Hard budget / hour |
|---|---|---|
| trial | 50 | 100 |
| sandbox | 50 | 100 |
| production | 500 | 2000 |
| internal | 2000 | 10000 |

Soft: returns warning header `X-LlmJudge-Budget-Remaining`. Hard: subsequent invocations return block + `llm_judge_budget_exceeded` reason.

## Audit Trail

Every decision + rule mutation + classifier rollout + Cedar bundle change is audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| GuardrailDecisionEmitted | per-BC usecase | `decision_id, invocation_id, tenant_id_hashed, verdict, block_reason, cedar_policy_ids[], classifier_model_versions{}, evidence_hash, signature` | ≥ 1y; 6y for HIPAA |
| JailbreakDetected | jailbreak-detector usecase | `incident_id, invocation_id, prompt_hash, severity, detector_versions` | indefinite (incident history) |
| AutonomyTierViolation | autonomy-tier-gate usecase | `violation_id, invocation_id, requested_tier, effective_ceiling, ceiling_source` | ≥ 1y |
| ContentSafetyRuleFired | content-safety usecase | `decision_id, rule_id, category, score, threshold` | ≥ 1y |
| RuleStoreMutated | rule-store-writer SA | `rule_id, version, action, pack, author_spiffe, commit_sha, pr_id, prior_version` | indefinite (git history) |
| ClassifierModelDeployed | model-rollout pipeline | `model_id, version, sha, cosign_signature_sha, status, prior_version, shadow_vs_enforce_delta` | indefinite |
| FalsePositiveEscalated | rest-endpoint | `escalation_id, decision_id, reason, budget_remaining` | ≥ 1y |

All audit-chain seals are Ed25519 + Merkle per Bominal ADR-0028.

## Verification

- `oya gate validate cedar-default-deny-enforced` — exit 0.
- `oya gate validate cedar-fragment-coverage --microservice foundry-guardrails` — exit 0.
- `oya gate validate shadow-enforce-promotion-readiness --rule <id>` — exit 0.
- Quarterly chaos drill: induce shadow → enforce rollout; FP escalation; classifier rollback.
- Annual pen-test against Cedar bundle.

## References

- ADR-0022 (autonomy ceiling); ADR-0140 (Cedar substrate); ADR-0114 (shadow→enforce precedent).
- `microservices/foundry-guardrails/threat-model.md` T-T-01 + T-T-04 + T-T-06.
- `microservices/foundry-guardrails/dpia.md` R-01 + R-02 + R-06 + R-07.
- `microservices/foundry-guardrails/policy/*.cedar`.
- `microservices/foundry-guardrails/incident-response.md` (FP / jailbreak escalation).
- `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md`.
- Cedar v4 docs — `docs.cedarpolicy.com`.
