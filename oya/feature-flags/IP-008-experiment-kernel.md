# IP-008 — Experiment Kernel Crate

**microservice**: feature-flags
**bc**: experiment
**layer**: kernel
**crate**: oya-feature-flags-experiment-kernel
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0244, ADR-0248, ADR-0252, ADR-0263, ADR-0292, ADR-0307, ADR-0309
**companion_ips**: IP-009, IP-012, IP-020

## Scope

Statistical core for experiment engine: assignment salt store, SRM detection, Bayesian Beta-Binomial posterior, frequentist z-test, mSPRT sequential bounds, Mann-Whitney-U non-parametric test, fairness audit (±2pp TPR/FPR threshold per ADR-0309).

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `ExperimentDefinition` struct | Fields: experiment_id, tenant_id, flag_key, variants (control + ≥1 treatment), allocation_percent, assignment_salt (server-side HMAC), start_hlc, conclude_hlc, sample_size_estimate, stats_method |
| 2 | `AssignmentEngine` | Deterministic: `HMAC-SHA256(assignment_salt + user_id_hash) mod 10000 < allocation_percent * 100`; server-side salt prevents client injection |
| 3 | `SrmDetector` | Chi-squared goodness-of-fit; p<0.01 → `SrmDetected` signal; triggers pause recommendation |
| 4 | `BayesianEngine` | Beta-Binomial conjugate model; computes posterior P(treatment > control); returns 95% credible interval |
| 5 | `FrequentistEngine` | Z-test for proportions; two-tailed; Bonferroni correction for multi-variant; alpha=0.05, power=0.80 |
| 6 | `MsprtEngine` | mSPRT (mixture Sequential Probability Ratio Test); theta=0.5; stops when λ>1/alpha; no Type I error inflation |
| 7 | `FairnessAuditor` | Per-class TPR/FPR delta across protected attributes; threshold ±2pp per ADR-0309; emits `FairnessViolationDetected` |
| 8 | Tests | SRM: injected 60/40 split on 50/50 experiment detects within 1000 observations; Bayesian: synthetic data matches known posterior; mSPRT: 1000-sim Type I error ≤0.05 |

## Statistical Methods Summary

| Method | Use Case | Stop Condition |
|--------|----------|----------------|
| Bayesian Beta-Binomial | Conversion rate | P(treatment>control) ≥ 0.95 |
| Frequentist z-test | Proportions | p ≤ 0.05 + Bonferroni |
| mSPRT | Sequential / early stopping | λ > 1/0.05 = 20 |
| Chi-squared SRM | Assignment balance check | p < 0.01 → SRM alert |
| Mann-Whitney-U | Non-normal continuous metrics | p ≤ 0.05 |

## Definition of Done

- `cargo test -p oya-feature-flags-experiment-kernel` green
- Server-side salt: `AssignmentEngine` rejects `context.user_bucket` if supplied by client
- mSPRT simulation: 1000 runs, H0 true → Type I error ≤0.05
- Fairness auditor: ±2pp threshold fires on synthetic skewed data
