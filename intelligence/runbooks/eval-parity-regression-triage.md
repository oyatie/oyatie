---
doc_class: Runbook
title: Parity Regression Triage (provider vs in-house variant)
microservice: foundry-eval
severity: "Sev-2 (parity verdict regressed)"
status: Accepted
owner_team: axis-foundry + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-03 parity regression)
  - microservices/intelligence/threat-model.md (R-09 judge bias)
  - ADR-0026 §"In-house cutover gate"
doc_status: published
---

# Runbook: Parity Regression Triage

## Trigger

ONE of:

1. A previously eligible in-house variant (per `InHouseCutoverEligible`) regresses on its next parity verdict (loses to incumbent provider on ≥ 1 cohort).
2. A new provider candidate underperforms the incumbent unexpectedly during A/B verdict.
3. Per-cohort parity verdict differs from prior verdict by ≥ 10 percentage points (sudden drift).

## Severity

- Eligible in-house variant regresses: **Sev-2** (cutover decision compromised).
- A/B candidate underperforms (expected outcome): **Sev-4 informational** (no action; document).
- Sudden drift on previously stable verdict: **Sev-2** (investigate provider drift).

## Pre-checks

1. Confirm latest parity report: `oya-intelligence-eval-parity-analyzer-rest report --capability <cap> --route-a <a> --route-b <b>`.
2. Confirm the eval-set is unchanged (no manifest churn correlating with regression).
3. Confirm the judge identity (HumanJudged cohort: did judge rotate this quarter?).
4. Confirm provider release notes for the incumbent / candidate.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC; declare severity | ≤ 5 min |
| 2 | Pre-checks above | ≤ 10 min |
| 3 | Identify regressing cohort(s): which adversarial / linguistic / domain cohort regressed? `oya-intelligence-eval-parity-analyzer-rest cohort-deltas --capability <cap>` | ≤ 10 min |
| 4 | Categorise: (a) judge-rotation bias (κ < 0.7 against prior judges); (b) provider drift (incumbent changed under us); (c) eval-set contamination (newly added cases overlap training set); (d) tokeniser drift; (e) genuine quality regression | ≤ 30 min |
| 5 | For (a): pause cutover-eligibility recomputation until judge consistency check re-runs; if κ < 0.7 sustained, revert judge to prior + investigate | per category |
| 6 | For (b): compare incumbent's recent provider release-notes to confirm material change; if confirmed, hold cutover until next stable provider version | per category |
| 7 | For (c): run `oya-check-eval-set-contamination` on affected cases; remove contaminated cases + re-run parity | per category |
| 8 | For (d): verify tokeniser version pin held; if drift, revert tokeniser; re-run parity | per category |
| 9 | For (e): if in-house variant genuinely regressed, emit `ReverseCutoverExecuted` + revert routing preference to incumbent; engage modeling team to root-cause | ≤ 1 h |
| 10 | Update parity-state-machine: emit `ParityRegressionInvestigated{cap, root_cause, action}` | ≤ 5 min |
| 11 | Postmortem within 5 business days for Sev-2 | — |

## Cross-Cohort Investigation

If the regression appears across ≥ 3 capabilities simultaneously, this is likely an upstream provider issue OR a foundry-eval infrastructure issue (ClickHouse query plan change; ε-DP-noise misapplied). Escalate to axis-foundry + ops-sre-reliability + cloud-secrets.

## Verification

After completion:
- Parity verdict for affected capability(ies) returns to expected baseline within 2 cycles.
- Either: cutover reversed + audit-chain seal recorded, OR root-cause identified + non-cutover-reversal mitigation applied.
- `ParityRegressionInvestigated` event in audit-chain.

## References

- ADR-0026 §"In-house cutover gate".
- ADR-0024 §"A/B testing of provider routing".
- `microservices/intelligence/failure-modes.md` FM-03.
- `microservices/intelligence/threat-model.md` T-A-02 (judge bias).
