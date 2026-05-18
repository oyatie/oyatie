---
doc_class: Runbook
title: LLM copilot degraded — fallback handling
microservice: workflow-studio
severity: "Sev-3 (single-tenant degraded) / Sev-2 (cross-tenant LLM-assist down)"
status: Accepted
owner_team: axis-workflow + foundry-providers-team
date: 2026-05-17
related_artifacts:
  - microservices/workflow-studio/PRD.md FR-12 (LLM-assist)
  - microservices/workflow-studio/threat-model.md §"T-D-04" LLM timeout cascade + §"T-I-05" prompt PII leak + §"T-S-05" prompt injection
  - /specs/microservices/workflow-studio.json §metrics LLM-assist latency
  - microservices/foundry/runbooks/* (sibling — LLM provider runbooks)
doc_status: published
---

# Runbook: LLM copilot degraded — fallback

## Purpose

Studio's LLM-assist feature (FR-12) bridges to foundry-providers for prose-to-spec generation. LLM-assist is **non-critical** — Studio works without it. This runbook ensures editor UX stays responsive when LLM-assist degrades, and contains prompt-injection / PII-leak risks.

## Trigger

ONE of:

1. **`oya_workflow_studio_llm_assist_latency_seconds{quantile="0.99"} > 3.0` for ≥ 15 min** (GA budget breached).
2. **`oya_workflow_studio_llm_assist_request_failed_total / oya_workflow_studio_llm_assist_request_total > 0.1`** (error rate > 10% for ≥ 5 min).
3. **`oya_workflow_studio_llm_assist_circuit_breaker_open_total > 0`** (circuit breaker tripped).
4. **`oya_workflow_studio_llm_assist_validation_failed_total / oya_workflow_studio_llm_assist_response_total > 0.5`** — over half of LLM drafts fail schema validation; suggests prompt-engineering regression OR provider degradation.
5. **`oya_workflow_studio_llm_assist_prompt_injection_detected_total > 0`** — attempted prompt injection detected.
6. Tenant reports: "LLM-assist returns gibberish" OR "takes forever" OR "shows another tenant's draft".

## Severity

- Single tenant degraded, others fine: Sev-3.
- Cross-tenant degradation, foundry-providers SLO impacted: Sev-2.
- Prompt injection succeeded AND bypassed schema validation: Sev-1.
- Cross-tenant prompt OR completion leakage: Sev-1.

## Impact

- Tenant authoring proceeds without LLM-assist (manual node-by-node remains available); FR-01..FR-11 unaffected.
- Tenant trust impact if LLM-assist quality drops below threshold.
- AI Act (pack-eu) + per-pack AI governance: LLM-assist invocation logged + auditable; degradation must not bypass logging.

## Pre-checks

1. Identify scope: which tenant(s)? which LLM provider? `dashboards/copilot-quality.json` panel "LLM-assist by tenant + provider".
2. Identify the contributing layer: Studio adapter? foundry-providers SDK? upstream LLM provider? Use trace per `dashboards/copilot-quality.json` panel "latency breakdown".
3. Verify the circuit breaker state: `oya_workflow_studio_llm_assist_circuit_breaker_state{tenant=<h>}` — `open=1` means breaker tripped.
4. Verify PII redactor health: `oya_workflow_studio_llm_assist_pii_redactor_alive == 1`.

## Recovery Path A — Provider latency spike (foundry-providers upstream)

Cause: LLM provider is slow; Studio circuit breaker opens; tenants see "LLM-assist degraded" banner.

| Step | Action | Time |
|---|---|---|
| 1 | Verify upstream: `microservices/foundry/dashboards/provider-health.json`. | ≤ 5 min |
| 2 | If upstream confirmed slow: circuit breaker is doing its job; tenants see graceful banner; editor UX unaffected (per T-D-04). | – |
| 3 | If tenant impacts > 1h: engage foundry-providers on-call; consider failover to per-tenant BYO-LLM if configured. | ≤ 1h |
| 4 | Monitor recovery: breaker auto-closes after 30s of clean requests; verify recovery in dashboard. | – |

## Recovery Path B — Schema validation regression (over-half drafts fail)

Cause: LLM-assist completions are not producing schema-valid spec drafts; tenants frustrated.

| Step | Action |
|---|---|
| 1 | Identify failing pattern: which fields in the spec cause validation rejection most often? `dashboards/copilot-quality.json` panel "validation failures by field". |
| 2 | If provider drift (e.g., model upgrade changed completion shape): adjust prompt template; redeploy. |
| 3 | If new spec field unsupported by prompt template: extend prompt template; redeploy. |
| 4 | If recurring per-model: switch default model for affected tenants (via foundry-providers' tenant-config). |

## Recovery Path C — Prompt injection detected (Sev-1)

Cause: tenant prose contained an injection attempt; PII redactor / classifier flagged it.

| Step | Action | Time |
|---|---|---|
| 1 | Verify the detection: `oya_workflow_studio_llm_assist_prompt_injection_detected_total` increments + audit-chain seal logged. | ≤ 2 min |
| 2 | Confirm Studio refused the prompt: `studio_llm_assist_request_refused_at_redactor{reason="prompt_injection"} > 0`. | – |
| 3 | If Studio refused: defence held; tenant sees "prompt contains forbidden patterns; please rephrase". No further action beyond audit trail. | – |
| 4 | If Studio DIDN'T refuse but completion bypassed schema validation AND was accepted by user: confirmed Sev-1; same forensic + breach-notification chain as `template-marketplace-quarantine.md` Path A. | per pack |
| 5 | Engineering: tighten redactor classifier; add this injection pattern to the test corpus. | per priority |

## Recovery Path D — PII leak suspected (Sev-1)

Cause: tenant reports their LLM-assist prompt was logged with PII not redacted; OR cross-tenant prompt visible in audit.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security + council-privacy. |
| 2 | Verify redactor processed the prompt: `studio_llm_assist_prompt_redacted_total{tenant=<h>} > 0`. |
| 3 | Audit-chain trace: find the actual prompt-as-stored (90d retention per PRD §"Audit + Compliance"). |
| 4 | If PII present in stored prompt: PII redactor failure; remediate per DSR cascade (`oya-dsr-cascade-runner --scope llm-assist`); tenant notification per pack. |
| 5 | If cross-tenant prompt visible: catastrophic isolation breach; immediate tenant notifications per all packs affected. |

## Recovery Path E — Tenant prefers manual; LLM-assist disabled per-tenant

Cause: Tenant requests LLM-assist disabled for their account (e.g., during M&A diligence, regulatory review).

| Step | Action |
|---|---|
| 1 | Tenancy SDK call: `tenancy update --tenant <h> --feature llm-assist --status disabled`. |
| 2 | Studio reads feature flag at editor-open; LLM-assist button hidden for that tenant. |
| 3 | Re-enable via same call when tenant requests. |

## Verification

After recovery:
- `oya_workflow_studio_llm_assist_latency_seconds{quantile="0.99"} <= 3.0` for ≥ 30 min.
- `oya_workflow_studio_llm_assist_request_failed_total / total <= 0.01` for ≥ 30 min.
- `oya_workflow_studio_llm_assist_circuit_breaker_state == 0 (closed)`.
- For Sev-1: tenant notifications complete; forensic analysis filed; audit-chain seal log complete.
- Editor UX unaffected (manual authoring proceeded throughout).

## Post-incident updates

- Postmortem within 5 business days (immediate for Sev-1).
- If injection-pattern new: add to detection corpus + golden tests.
- If redactor missed PII: harden classifier; add a synthetic-PII test set.
- If provider degradation recurring: investigate provider SLA; consider provider-diversity in foundry-providers config.
- AI Act (pack-eu): if degradation impacted user safety (e.g., LLM drafted wrong safety-critical workflow), trigger AI Act Art. 73 incident reporting to relevant DPA within 15 days.

## References

- `microservices/workflow-studio/PRD.md` FR-12 + §"Security" LLM-assist.
- `microservices/workflow-studio/threat-model.md` T-D-04, T-I-05, T-S-05.
- `/specs/microservices/workflow-studio.json` §metrics + §goals.
- OWASP Top 10 LLM Applications (2023) — A01 Prompt Injection + A02 Insecure Output Handling.
- EU AI Act 2024/1689 Arts. 9-15 + 26 + 50 + 73 (high-risk AI systems).
- NIST AI RMF 1.0 — `nist.gov/itl/ai-risk-management-framework`.
- `microservices/foundry/threat-model.md` (sibling, upstream LLM concerns).
