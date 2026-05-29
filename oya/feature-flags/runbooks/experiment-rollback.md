---
doc_class: Runbook
microservice: feature-flags
runbook_id: RB-FF-003
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0263
companion_docs:
  - microservices/feature-flags/runbooks/experiment-stat-sig-violation.md
  - microservices/feature-flags/runbooks/killswitch-engaged.md
  - microservices/feature-flags/incident-response.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Runbook: Experiment Rollback

## A. Trigger conditions

- Experiment causing user harm or significant metric degradation.
- Sample Ratio Mismatch (SRM) detected: Chi-squared test fails (p < 0.01 on assignment balance).
- Primary metric showing negative effect with >95% confidence.
- SLO error budget burning faster than expected in experiment group.
- Compliance issue with experiment design (e.g., GDPR consent not obtained for behavioral experiment).
- Statistical significance violation (see `runbooks/experiment-stat-sig-violation.md`).

## B. Pre-checks (≤2 minutes)

1. Identify experiment ID:
   ```bash
   oya experiments list --tenant <tenant_id> --state active
   ```
2. Check current experiment status and metrics:
   ```bash
   oya experiments results <experiment_id> --tenant <tenant_id>
   # Check: statistical_significance, p_value, sample_ratio_mismatch_detected
   ```
3. Verify the rollback target: rollback = stop experiment + restore all users to control variant.
4. If SRM detected: note that statistical results are invalid — do NOT declare a winner before rollback.

## C. Procedure

### Step 1 — Pause experiment traffic (≤30s)

```bash
oya experiments pause <experiment_id> --tenant <tenant_id>
# Halts new assignments; existing assignments remain until step 2
```

### Step 2 — Conclude experiment without winner (rollback path, ≤60s)

```bash
oya experiments conclude <experiment_id> \
  --tenant <tenant_id> \
  --winner-variant control \
  --conclusion-reason "rollback: <reason>"
```

This:
- Sets all users to control (default) variant.
- Emits `ExperimentConcluded` audit event with rollback flag.
- Archives the experiment flag (if `intent: experiment`).

### Step 3 — Verify all users on control variant (≤5 minutes)

```bash
# Check experiment assignment distribution
oya experiments assignment-distribution <experiment_id> --tenant <tenant_id>
# Expected: 100% control variant post-rollback

# Verify flag evaluation reason
oya flags evaluate <experiment_flag_key> \
  --tenant <tenant_id> \
  --principal test-probe
# Expected reason: DEFAULT (experiment concluded; using default variant)
```

### Step 4 — Verify metric recovery (≤30 minutes)

Watch primary metric time-series (dashboard: `dashboards/experiment-results.json`). Confirm metric returns toward pre-experiment baseline within 30 minutes of rollback.

### Step 5 — Metric re-attribution (if bug found, ≤24 hours)

If experiment was rolled back due to attribution bug:
```bash
oya feature-flags experiment reattribute \
  --experiment-id <experiment_id> \
  --start <start_ts> \
  --end <rollback_ts>
```

See `backfill-replay.md §experiment-metric-re-attribution`.

## D. Verification

- `ExperimentConcluded` audit event with `conclusion_reason: "rollback"` present and sealed.
- `oya_experiment_assignments_total{experiment_id="<id>",variant!="control"}` = 0 after rollback.
- Primary metric recovering toward baseline.

## E. Rollback (of this rollback — i.e., re-activating)

If rollback was premature and the experiment should resume:
1. Create a new experiment (don't re-activate the concluded one).
2. Use fresh salt to reset assignment buckets (prevents historical bias).
3. Ensure SRM root cause is identified before re-activation.

## F. Post-incident

- SRM analysis: what caused the ratio mismatch? Bot traffic? SDK bug? Incorrect exclusion criteria?
- Statistical review: if the experiment was running for <minimum sample size, results are underpowered — do not draw conclusions.
- Fair lending review: if experiment was on a financial feature and affected protected classes disproportionately, notify compliance officer.

## G. References

- `runbooks/experiment-stat-sig-violation.md` — if statistical methods were violated.
- `backfill-replay.md §experiment-metric-re-attribution` — metric re-attribution.
- `compliance.md §ml-model-lifecycle` — ML model rollback guidance.
- ADR-0159 — feature-flag substrate binding ADR.
