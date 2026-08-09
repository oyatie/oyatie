---
doc_class: Runbook
title: AI-form-build rollback (T2 quality regression / safety signal)
microservice: forms
severity: "Sev-3 (quality regression) / Sev-2 (safety signal)"
status: Accepted
owner_team: axis-forms + foundry-providers-team + council-privacy
date: 2026-05-17
related_artifacts:
  - microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md
  - microservices/forms/dashboards/ai-form-build-quality.json
  - microservices/forms/failure-modes.md FM-07
doc_status: published
---

# Runbook: AI-form-build rollback

## Purpose

When AI-form-build (T2 capability) starts emitting low-quality output, or emits a high-risk classified form (Annex III §4), this runbook executes a controlled rollback.

## Trigger

ONE of:

1. **`oya_forms_ai_build_schema_invalid_total / oya_forms_ai_build_response_total > 0.30` ≥ 1h** — schema-valid rate fell below 70%.
2. **`oya_forms_ai_build_safety_signal_total > 0`** in any 24h window — high-risk classification triggered.
3. **`oya_forms_ai_build_acceptance_rate < 0.20`** ≥ 24h — tenant accept rate collapsed.
4. **`oya_forms_ai_build_revert_rate > 0.40`** ≥ 24h — tenants reverting drafts after accept.
5. **External signal**: regulator inquiry on AI-form-build behaviour.

## Severity

- Quality regression (schema rate / acceptance rate): Sev-3.
- Safety signal (high-risk classification, prompt-injection success): Sev-2.
- Confirmed Annex III §4 misuse (employment screening without DPIA): Sev-2 + engage council-legal-compliance.

## Pre-checks

1. `dashboards/ai-form-build-quality.json` panels: schema-valid rate, acceptance rate, safety signals, prompt-injection-detected.
2. Identify tenant(s) affected.
3. Identify LLM provider routing — has the upstream model version changed?
4. Recent ADR-FORMS-0005 changes deployed?

## Recovery Path A — LLM provider model drift

Cause: tenant's BYO-LLM (or pack-default LLM) upgraded silently; emits new format.

| Step | Action |
|---|---|
| 1 | Identify provider model version: `cargo run -p oya-dev-cli -- forms ai-build-provider-status --pack <pack>`. |
| 2 | Pin to previous model version: `cargo run -p oya-dev-cli -- forms ai-build-pin-model --pack <pack> --version <prev>`. |
| 3 | Verify schema-valid rate recovers (24h window). |
| 4 | Coordinate with foundry-providers team for model upgrade test on staging before promotion. |

## Recovery Path B — Prompt-injection success

Cause: schema-valid completion rate looks OK but dsl-loader is rejecting cross-µservice attempts at higher-than-normal rate; suggests adversarial prompts.

| Step | Action |
|---|---|
| 1 | Identify affected tenants: `dashboards/ai-form-build-quality.json` panel "prompt-injection by tenant". |
| 2 | Strengthen PII-redactor + prompt-injection-scrub: deploy newer ruleset. |
| 3 | Tenant comms: AI-form-build temporarily limited; users may see fallback "build it manually" UX. |
| 4 | Engage ops-security for forensic review. |
| 5 | If confirmed adversarial: per ToS + abuse-policy action. |

## Recovery Path C — High-risk classification (Annex III §4) detected

Cause: tenant prompt suggests employment screening / credit / insurance.

| Step | Action |
|---|---|
| 1 | Gate the tenant's AI-form-build: surface mandatory DPIA prompt per ADR-FORMS-0005. |
| 2 | Tenant must complete DPIA + AI-Act conformity assessment before next AI-form-build invocation. |
| 3 | council-legal-compliance review of tenant use-case. |
| 4 | If tenant proceeds: AI-Act Art. 26 deployer obligations apply. |
| 5 | Post-market monitoring (Art. 72): per-tenant safety-signal log retained. |

## Recovery Path D — Quality regression (acceptance rate collapse)

| Step | Action |
|---|---|
| 1 | Compare prompt corpus pre/post regression: any tenant-side pattern change? |
| 2 | Run T2-eval set locally: `cargo run -p oya-dev-cli -- forms ai-build-eval --reference capabilities/eval/t2-auto-reference.jsonl`. |
| 3 | If eval pass-rate drops: rollback foundry-providers route to previous LLM version. |
| 4 | Coordinate with foundry-providers + tenant on next-iteration improvement. |

## Recovery Path E — Full disable (Sev-2)

Cause: cannot trust any AI-form-build output at the moment.

| Step | Action |
|---|---|
| 1 | Disable AI-form-build cluster-wide: `cargo run -p oya-dev-cli -- forms ai-build-disable --duration 24h`. |
| 2 | Tenant UI shows "AI-form-build temporarily unavailable; use manual builder". |
| 3 | All existing drafts remain queryable but no new T2 invocations. |
| 4 | Engineer-on-call investigates; 24h SLA to restore. |
| 5 | Re-enable after ADR-FORMS-0005 §"Risk register" review. |

## Invariant: never silent-accept low-quality output

- LLM output that fails schema validation NEVER auto-saves; tenant sees diagnostic.
- LLM output that emits cross-µservice nodes the tenant has no entitlement to NEVER passes Cedar.
- LLM output that emits data_class mismatch (PII tagged as NORMAL) NEVER passes dsl-loader.

## Verification

After recovery:
- `oya_forms_ai_build_schema_invalid_total / oya_forms_ai_build_response_total ≤ 0.20`.
- `oya_forms_ai_build_safety_signal_total == 0` in last 24h.
- `oya_forms_ai_build_acceptance_rate ≥ 0.50`.
- `oya_forms_ai_build_revert_rate ≤ 0.20`.

## Post-incident updates

- Postmortem within 5 business days.
- ADR-FORMS-0005 review if recurring; supersession if framework needs updating.
- Per-pack AI-Act compliance report (quarterly cadence per Art. 72) updated.
- Provider SLA review.

## References

- ADR-FORMS-0005 AI-form-build bounds.
- `dashboards/ai-form-build-quality.json`.
- `dpia.md` R-03 + R-15.
- `compliance.md` §"4. EU AI Act".
- Regulation (EU) 2024/1689 Arts. 9, 12, 26, 50, 72; Annex III §4.
- OWASP Top 10 for LLM Applications.
