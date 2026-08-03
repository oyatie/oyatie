# Plan: intelligence-guardrails-fp-budget-headroom-weighted-aggregate

## Objective

Extend `FpBudget` in `oya-intelligence-guardrails-kernel` with:

1. `SeverityWeight` — maps `RiskLevel` (Low/Medium/High) to `f64` weights
2. `FpBudget::weighted_fp()` — severity-weighted false-positive aggregate
3. `FpBudget::remaining_headroom()` — `budget_pct - observed_fp_rate` clamped at `>=0.0`
4. `FpBudget::merge(&self, other: &FpBudget) -> Result<FpBudget, FpBudgetError>` — sums observations across two windows with same `budget_pct`

## Error variants needed

`FpBudgetError` gains a new variant:
- `BudgetPctMismatch` — merge called on two budgets with different `budget_pct`

`ZeroTotalEvals` is already present and serves double duty (merge of two zero-total budgets).

## Acceptance criteria

- `remaining_headroom()` at budget: `0.0`
- `remaining_headroom()` below budget: `> 0.0`
- `remaining_headroom()` over budget: `0.0` (clamped)
- `weighted_fp()` across Low/Medium/High with non-trivial weights produces correct sum
- `merge()` happy path: sums `observed_fp` and `total_evals`, uses common `budget_pct`
- `merge()` error: `BudgetPctMismatch` when `budget_pct` values differ
- `merge()` error: `ZeroTotalEvals` when merged `total_evals` sums to zero

## Constraints

- No new deps
- No new workspace member
- No root Cargo.toml edits
- No I/O — pure deterministic functions only
- Existing tests must remain green

## Steps

1. Add `BudgetPctMismatch` variant to `FpBudgetError`
2. Add `SeverityWeight` struct with per-level weights and a `weight_for(RiskLevel)` method
3. Add `weighted_fp(&self, sw: &SeverityWeight, findings: &[(RiskLevel, u32)])` to `FpBudget`
4. Add `remaining_headroom(&self)` to `FpBudget`
5. Add `merge(&self, other: &FpBudget) -> Result<FpBudget, FpBudgetError>` to `FpBudget`
6. Write `#[cfg(test)]` unit tests covering all acceptance criteria
7. `cargo check -p intelligence-guardrails-kernel --all-targets`
8. `cargo nextest run -p intelligence-guardrails-kernel`
