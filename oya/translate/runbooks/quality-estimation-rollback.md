---
doc_class: Runbook
title: Quality Estimation (QE) — rollback / classifier drift recovery
microservice: translate
severity: "Sev-2 (drift) / Sev-1 (EU AI Act transparency / bound-violation)"
status: Accepted
owner_team: axis-translate + ops-security + council-privacy + axis-foundry-runtime
date: 2026-05-18
related_artifacts:
  - microservices/translate/failure-modes.md (FM-20..FM-23)
  - microservices/translate/decisions/ADR-TRANSLATE-0003-quality-estimation-and-eu-ai-act-bounds.md
  - microservices/translate/capabilities/T1-assist.yaml
  - microservices/translate/dashboards/quality-and-tm-leverage.json
  - microservices/translate/policy/ai-act-overlay.md
doc_status: published
---

# Runbook: Quality Estimation (QE) — rollback / classifier drift recovery

## Trigger

Any of:

- FM-20 (QE score distribution drift; macro-bias > 5 points vs the deployed-version reference eval set).
- FM-21 (QE p99 latency exceeds 200 ms p99 budget for ≥ 15 min sustained).
- FM-22 (QE bound-violation: tenant complains that QE flagged accurate human-quality translations as low-quality at > 10× baseline).
- FM-23 (EU AI Act Art. 13 transparency record missing for ≥ 1 % of `jurisdiction=EU` invocations).
- Council-privacy escalation: regulator-notifiable misuse pattern.

## Severity

| Symptom | Severity | Notify |
|---|---|---|
| Drift (reference-eval bias > 5 pts) | Sev-2 | axis-translate + axis-foundry-runtime |
| QE p99 budget breach | Sev-2 | axis-translate + ops-sre-reliability |
| Bound-violation (tenant impact) | Sev-1 | council-privacy + tenant comms |
| EU AI Act Art. 13 disclosure gap | Sev-1 | council-privacy + DPA-notification clock starts |
| Mass false-flag event | Sev-1 (P0 if regulated content class) | council-privacy + regulator-notifiable |

## Immediate Mitigation (≤ 15 min)

| Step | Action | Time |
|---|---|---|
| 1 | Confirm trigger via `dashboards/quality-and-tm-leverage.json` and `oya_translate_qe_macro_bias` recording rule | ≤ 2 min |
| 2 | Identify QE model version active in pack(s) via `helm get values translate -n translate \| grep qualityEstimationWorker.modelTag` | ≤ 3 min |
| 3 | Roll back QE model: `helm rollback translate-qe <prior-revision> -n translate` (per-pack) | ≤ 5 min |
| 4 | Or: disable T1 QE per-pack via Cedar entitlement revoke (`cargo run -p oya-dev-cli -- translate disable-capability --capability qe --pack <p>`) | ≤ 5 min |
| 5 | Emit `QualityEstimationRollback{from_version, to_version, reason, pack}` to audit-chain | ≤ 2 min |
| 6 | Notify affected tenant operators via status page | ≤ 15 min |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Model drift after retrain | timing matches deploy; reference-eval bias regressed | rerun eval reference set per pack; bisect retrain |
| Language-pair skew | bias clusters on `(source_lang, target_lang)` | retrain with lang-pair-balanced data or disable for that pair |
| Adversarial input pattern | tenant submits crafted segments that score 0 falsely | sample 100 verdicts; ops-security review |
| EU AI Act Art. 13 disclosure gap | `oya_translate_eu_ai_act_disclosure_emitted_total` drops | check disclosure-emitter adapter; verify per-call gate |
| Bound-violation (per ADR-TRANSLATE-0003 §"bounds") | scoring outside [0,100] envelope | bound-enforcer middleware failure; pin previous QE adapter |

## QE Bound Enforcement Verification

Per ADR-TRANSLATE-0003 §"bounds":

- QE score MUST be in `[0.0, 100.0]` inclusive; out-of-band → reject + log.
- Per-call disclosure record MUST be emitted on `jurisdiction = EU` invocations.
- Per-language-pair confidence interval MUST be ≥ 0.7 (per reference eval).
- `oya_translate_qe_bound_violation_total > 0` is HARD; immediate Sev-1.

Verification commands:

```bash
cargo run -p oya-dev-cli -- translate verify-qe-bounds --pack <p> --window 1h
cargo run -p oya-dev-cli -- translate verify-eu-ai-act-disclosure --pack eu --window 1h
```

## Rollback Procedure

1. Identify all QE invocations on the affected version + window.
2. Mark QE scores as "unreliable" in the per-tenant TM (do not delete; preserve audit trail).
3. Re-emit `QualityEstimated{score: null, status: "rolled-back", original_version, replacement_version}` to audit-chain.
4. If tenant has post-edit-distance ground-truth recorded: backfill corrected QE score using stable previous-version adapter.
5. Council-privacy reviews + signs off; ops-security closes incident.

## False-Flag Recovery Procedure

When QE has incorrectly flagged human-quality translations:

1. Identify the set of flagged segments (`SELECT * FROM qe_events WHERE qe_score < threshold AND human_post_edit_distance = 0`).
2. Notify each affected tenant via `incident-response.md` template (per pack).
3. Re-score with rolled-back version; emit corrected `QualityEstimated` events.
4. Council-privacy signs off; ops-security closes.

## Verification (After Recovery)

- `oya_translate_qe_macro_bias` returns within ± 2 pts of reference-eval baseline.
- `oya_translate_qe_score_latency_seconds_p99 < 0.2` for 30 min sustained.
- `oya_translate_qe_bound_violation_total == 0` for 1 h sustained.
- `oya_translate_eu_ai_act_disclosure_emitted_total / oya_translate_qe_invocations_total{jurisdiction="EU"} == 1.0` for 1 h.
- `tests/integration/qe_bound_invariant.rs` re-run.

## Postmortem Triggers

- Within 5 business days for Sev-2.
- Within 3 business days for Sev-1.
- If EU AI Act non-compliance: DPA notification within 72 h (GDPR Art. 33 if personal data involved).
- If KR PIPA Art. 29-2 issue: KISA notification within 24 h if user-rights impact.

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-eu | EU AI Act Art. 13 transparency to deployers; Art. 50 transparency to end-users; misclassification with user-rights impact triggers DPA notification |
| pack-kr | KR PIPA Art. 29-2 automated decision-making rights; misclassification triggers KISA review |
| pack-us-healthcare | HIPAA — QE on PHI translations disabled by default; if engaged, triggers HHS OCR review |
| pack-cn-stub | QE on CN-tenant data: in-house only; vendor rollback path constrained |

## Named Industry Sources

- EU AI Act (Regulation (EU) 2024/1689) Arts. 13 (transparency to deployers) + 50 (transparency to end-users) + Annex III §4 (high-risk AI: employment/credit/legal/medical).
- GDPR Art. 22 (automated decision-making) + Art. 33 (breach notification).
- KR PIPA Art. 29-2 (automated decision-making rights).
- HIPAA 45 CFR §164.502(b) (minimum necessary).
- NIST AI RMF (Risk Management Framework) — measurement of trustworthy AI.
- WMT-24 shared-task QE benchmark (COMET-Kiwi, METRIC-X).
- Unbabel COMET — `unbabel.github.io/COMET/`.

## References

- ADR-TRANSLATE-0003 (QE and EU AI Act bounds).
- `microservices/translate/policy/ai-act-overlay.md`.
- `microservices/translate/capabilities/T1-assist.yaml`.
- `microservices/translate/dashboards/quality-and-tm-leverage.json`.
- `microservices/translate/failure-modes.md`.
