# Spec: Savings Portfolio Rollup

**Slug**: cloud-finops-savings-portfolio-rollup  
**Crate**: cloud-finops-kernel  
**Lane**: cloud  
**Priority**: high  
**Effort**: M

## Summary

Add a `savings_portfolio` module to `cloud-finops-kernel` that aggregates
a set of `Recommendation`s against their baseline `CostReport`(s) into a
deterministic `SavingsPortfolio` projection.

## Public Surface

```rust
pub fn roll_up_savings(
    recommendations: &[Recommendation],
    reports: &[CostReport],
) -> Result<SavingsPortfolio, RollupError>

pub struct SavingsPortfolio {
    pub estimated_savings_micros: u128,
    pub counts_by_kind: std::collections::HashMap<RecommendationKind, u32>,
    pub coverage_bps: u16,   // basis points, saturating, capped at 10_000
}

pub enum RollupError {
    MissingBaselineReport { baseline_report_id: String },
}
```

## Behaviour

- Only `Active` and `Applied` recommendations contribute to the rollup.
- `Draft` and `Dismissed` recommendations are silently skipped.
- `estimated_savings_micros`: sum of `estimated_savings_micros` over contributing
  recommendations (saturating u128 addition).
- `counts_by_kind`: count of contributing recommendations per `RecommendationKind`.
- `coverage_bps`: computed as
  `(estimated_savings_micros * 10_000) / total_baseline_spend_micros`, saturating
  at `10_000` (100%). Zero if baseline spend is zero.
  `total_baseline_spend_micros` is the sum of `total_spend_micros` from all
  reports referenced by contributing recommendations (deduplicated by report_id).
- A contributing recommendation that references a `baseline_report_id` not present
  in the supplied `reports` slice returns `Err(RollupError::MissingBaselineReport)`.
- An empty input set (or all Draft/Dismissed) returns `Ok` with a zero portfolio.
- Recommendations with `baseline_report_id = None` that are Active/Applied still
  contribute to savings and counts; they do not contribute to baseline spend.

## Invariants

- No I/O, no async, no new external dependencies (`std` only).
- Pure deterministic function; same inputs always produce identical output.
- clippy clean under workspace lints.

## Acceptance Tests

| # | Scenario | Expected |
|---|----------|----------|
| a | Empty recommendation slice | `Ok(SavingsPortfolio { estimated_savings_micros: 0, counts_by_kind: {}, coverage_bps: 0 })` |
| b | Mix of Draft/Dismissed/Active/Applied | Only Active+Applied counted |
| c | Active rec with `baseline_report_id` absent from reports | `Err(RollupError::MissingBaselineReport { .. })` |
| d | savings_micros >= baseline spend | `coverage_bps` capped at `10_000` |
| e | Multiple kinds | `counts_by_kind` tracks each kind separately |
