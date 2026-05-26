---
doc_class: Policy
title: Differential Privacy Analysis (foundry-eval cross-tenant aggregates)
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + council-privacy
deciders: axis-foundry, council-privacy, ops-security
related_adrs: [ADR-0024, ADR-0131]
related_artifacts:
  - microservices/intelligence/threat-model.md
  - microservices/intelligence/dpia.md
  - microservices/intelligence/policy/tenant-isolation.md
review_cadence: quarterly + on every new published aggregate
doc_status: published
---

# Differential Privacy Analysis (foundry-eval µservice)

## Purpose

Any cross-tenant aggregate published by foundry-eval (competitor-parity-matrix, public eval-pass-rate trends, cross-tenant capability quality benchmarks) passes through differential-privacy aggregation with ε ≤ 1 per published aggregate. This policy specifies the mechanism, the budget, and the verification.

## Threat being mitigated

Threat-model T-I-01 (cross-tenant leak via aggregation) and T-L-01 (linkability across cohorts). Cross-tenant aggregates without DP-noise can leak per-tenant signal via differencing attacks.

## Mechanism

- **DP primitive**: Gaussian mechanism (preferred) or Laplace (when bounded-sensitivity arithmetic is needed).
- **Sensitivity**: per-aggregate computed at publication time; recorded in `policy/dp-budget-ledger.md`.
- **Noise scale**: σ = sensitivity / ε; σ floor enforced even when ε > 1 would notionally apply (ε ≤ 1 hard cap).
- **Privacy budget**: per-(aggregate, time window) ε-budget tracked in `policy/dp-budget-ledger.md`; once budget exhausted, aggregate no longer republishes for the window.
- **Composition**: serial composition over time windows; advanced composition (Dwork-Rothblum-Vadhan) for multiple aggregates.

## Per-Aggregate Budget

| Aggregate | ε per publish | Cadence | Max budget per year |
|---|---|---|---|
| Competitor-parity-matrix (provider A vs B per capability) | 1.0 | quarterly | 4.0 |
| Public eval-pass-rate trend (per capability across tenants) | 0.5 | monthly | 6.0 |
| Cross-tenant top-10-quality-improvers leaderboard | 1.0 | monthly | 12.0 (with composition discount) |
| Cross-tenant adversarial-cohort pass rate | 0.5 | quarterly | 2.0 |
| Cross-tenant replay-determinism-divergence median | 0.5 | monthly | 6.0 |

## Verification

- **Pre-publish DP-check**: `oya-check-dp-noise-on-cross-tenant-aggregates` LEAN lane refuses publication of any cross-tenant aggregate without a DP-budget entry + verified noise scale.
- **Post-publish audit-chain**: every DP-aggregated publish emits `EvalCrossTenantAggregatePublished{aggregate_id, epsilon, sensitivity, sigma, time_window}` to audit-chain.
- **Annual privacy audit**: external privacy auditor reviews DP-budget ledger + sensitivity computations.

## Per-Pack Considerations

- **pack-eu**: DP-aggregation does not relieve GDPR Art. 5(1)(f) confidentiality obligations; treated as defence-in-depth.
- **pack-us-healthcare**: DP-aggregation does not relieve HIPAA expert-determination de-identification requirements; HHS expert-determination is the primary mechanism; DP is supplemental.
- **pack-kr**: DP-aggregation does not relieve KR PIPA Art. 23 sensitive-data protections.

## References

- Dwork, McSherry, Nissim, Smith (2006) "Calibrating Noise to Sensitivity in Private Data Analysis".
- Dwork, Rothblum, Vadhan (2010) "Boosting and Differential Privacy" (advanced composition).
- threat-model.md T-I-01, T-L-01.
- dpia.md R-02.
- policy/tenant-isolation.md TI-07.
