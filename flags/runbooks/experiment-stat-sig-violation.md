---
doc_class: Runbook
microservice: feature-flags
runbook_id: RB-FF-007
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0308
  - ADR-0309
companion_docs:
  - flags/runbooks/experiment-rollback.md
  - microservices/feature-flags/compliance.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Runbook: Experiment Statistical Significance Violation

## A. Trigger conditions

- Sample Ratio Mismatch (SRM) detected: Chi-squared goodness-of-fit p < 0.01 on variant assignment distribution.
- Peeking alarm: experiment conclusions drawn before pre-specified minimum sample size reached.
- Multiple comparisons without correction (running >5 metrics without Bonferroni/Benjamini-Hochberg).
- Fairness violation: per-class TPR/FPR outside ±2pp baseline (EU AI Act high-risk).
- Bayesian posterior computed on <100 conversions per variant (underpowered).
- Novelty effect not accounted for in week-over-week experiment.

## B. Pre-checks (≤5 minutes)

1. Check SRM:
   ```bash
   oya experiments srm-check <experiment_id> --tenant <tenant_id>
   # chi_squared_p_value < 0.01 → SRM detected
   ```
2. Check sample size vs. pre-specified minimum:
   ```bash
   oya experiments power-check <experiment_id> --tenant <tenant_id>
   # achieved_n vs required_n
   ```
3. Check fairness metrics:
   ```bash
   oya experiments fairness-audit <experiment_id> --tenant <tenant_id>
   # per_class_tpr_fpr_delta per protected class
   ```
4. Check number of metrics being tested simultaneously.

## C. Procedure

### Case A — SRM detected

SRM invalidates all statistical conclusions from the experiment. Causes: bot traffic, SDK bug, exclusion criteria bug, holdback mismatch.

1. Pause experiment immediately: `oya experiments pause <experiment_id> --tenant <tenant_id>`.
2. Do NOT declare a winner.
3. Diagnose SRM root cause:
   - Bot traffic: check `oya_feature_flag_eval_total` for bot-score>50 traffic in experiment.
   - SDK bug: check SDK version distribution across assigned users.
   - Exclusion criteria: verify exclusion logic in targeting rule.
4. Fix root cause; reset salt: `oya experiments reset-salt <experiment_id>`.
5. Re-activate with fresh assignment (new salt prevents historical bias).

### Case B — Peeking (underpowered conclusion)

1. Do NOT conclude the experiment.
2. If using sequential testing (mSPRT): peeking is permitted by design — no violation.
3. If using frequentist z-test: wait until minimum sample size is reached.
   ```bash
   oya experiments time-to-sufficient-sample <experiment_id>
   # Returns: estimated days to minimum sample
   ```
4. Enable sequential testing for future experiments: `--statistical-method sequential_msprt`.

### Case C — Fairness violation

Per ADR-0309 and EU AI Act Art. 14:

1. Pause experiment.
2. Run fairness audit: `oya experiments fairness-audit <experiment_id> --tenant <tenant_id>`.
3. Identify biased feature in experiment model.
4. Notify compliance officer: fairness violation in experiment `<experiment_id>`.
5. If experiment is high-risk AI (EU AI Act Article 6 Annex III): mandatory regulator notification.
6. Re-design experiment excluding biased features; re-run fairness validation before re-activation.

### Case D — Multiple comparisons without correction

1. Apply Benjamini-Hochberg FDR correction to all p-values: `oya experiments apply-fdr-correction <experiment_id>`.
2. Re-evaluate significance with corrected p-values.
3. If previously-declared winner no longer significant: retract conclusion; re-run experiment or accept null.

## D. Verification

- SRM check: `oya experiments srm-check` returns `no_srm_detected`.
- Power check: `achieved_n >= required_n`.
- Fairness audit: per-class TPR/FPR delta < ±2pp.
- All metrics have Bonferroni/BH correction applied if n_metrics > 5.

## E. Rollback

Per `runbooks/experiment-rollback.md` — roll back to control variant.

## F. Post-incident

- Experiment design review: was SRM preventable with better exclusion criteria?
- Statistical education: add experiment design checklist (sample size calc, SRM check plan, fairness slice plan) to experiment creation workflow.
- Fairness: if fairness violation caused harm, trigger GDPR Art. 22 adverse-action notification + appeal surface.

## G. References

- `runbooks/experiment-rollback.md` — rollback procedure.
- `compliance.md §detection-fairness-audit` — fairness audit requirements.
- `compliance.md §ml-model-lifecycle` — ML lifecycle including fairness re-audit.
- ADR-0308 — ML model lifecycle.
- ADR-0309 — detection fairness audit.
