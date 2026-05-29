# IP-020 — Experiment Statistics Engine

**microservice**: feature-flags
**bc**: experiment
**layer**: domain
**qualifier**: stats-engine
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0248, ADR-0252, ADR-0263, ADR-0307, ADR-0309
**companion_ips**: IP-008, IP-009, IP-012
**references**: PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE.md §statistics-engine; competitor-parity-matrix.md §statsig-parity

## Scope

Production statistics engine consolidating all statistical methods from `experiment-kernel` (IP-008) into a single callable service with EU AI Act Art. 13 explainability, fairness audit, and sequential test orchestration.

## Deliverables

| # | Component | Acceptance Criterion |
|---|-----------|---------------------|
| 1 | `ExperimentStatsService` | Unified entry point: `compute_results(experiment_id, tenant_id) -> ExperimentResult`; selects method per `stats_method` field |
| 2 | Bayesian decision output | `winner: Option<VariantId>`, `posterior_prob: f64`, `credible_interval_95: (f64, f64)`, `bayes_factor: f64` |
| 3 | Frequentist decision output | `p_value: f64`, `effect_size_cohens_d: f64`, `confidence_interval_95: (f64, f64)`, `bonferroni_adjusted: bool` |
| 4 | mSPRT orchestration | Evaluates after each metric batch; emits `SequentialStopRecommendation` event when λ > 20; prevents result peeking |
| 5 | SRM report | `srm_detected: bool`, `chi_squared_stat: f64`, `p_value: f64`, `expected_vs_actual: Vec<(VariantId, u64, u64)>` |
| 6 | Fairness report (ADR-0309) | Per protected attribute: `tpr_delta: f64`, `fpr_delta: f64`, `violation: bool` (threshold ±2pp); emits `FairnessViolationDetected` |
| 7 | EU AI Act Art. 13 explainability | `feature_importance: Vec<(FeatureName, f64)>` via LIME/SHAP approximation over metric attribution data |
| 8 | Multiple comparisons correction | Benjamini-Hochberg FDR for >2 variants; Bonferroni for primary metric; Holm-Bonferroni for secondary metrics |
| 9 | Stats engine API | gRPC `ComputeExperimentResults(ExperimentStatsRequest) -> ExperimentStatsResponse`; async Tokio; ≤5s p99 for 1M events |
| 10 | Tests | Synthetic 50/50 experiment: Bayesian P(treatment>control) ≥0.95 at N=5000; FDR test: 10 comparisons, 1 true positive → only 1 significant after BH; fairness: inject ±3pp skew → violation detected |

## Hyperscaler Parity

| Capability | LaunchDarkly | Statsig | Split.io | Optimizely | GrowthBook | oyatie |
|------------|-------------|---------|----------|------------|------------|--------|
| Bayesian | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ |
| mSPRT | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ |
| Fairness audit | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| EU AI Act Art.13 | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| SRM detection | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

## Definition of Done

- `cargo test -p oya-feature-flags-experiment-stats` green
- Bayesian: synthetic N=5000, 10% lift → P(treatment>control) ≥0.95
- mSPRT: 1000-run simulation, H0 true → Type I error ≤0.05
- BH FDR: 10 comparisons, 1 true positive → 1 significant result
- Fairness: ±3pp injected skew → `FairnessViolationDetected` emitted
- EU Art. 13: `feature_importance` non-empty for all concluded experiments
