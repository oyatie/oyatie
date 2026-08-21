# Spec: intelligence-guardrails-fp-budget-headroom-weighted-aggregate

## Crate

`intelligence-guardrails-kernel`

## Purpose

Extend the existing `FpBudget` type with three pure deterministic capabilities:

1. **Severity-weighted FP aggregation** — `SeverityWeight` maps each `RiskLevel` variant to a
   caller-supplied `f64` weight; `FpBudget::weighted_fp` folds a slice of
   `(RiskLevel, u32)` observation counts through those weights.
2. **Remaining headroom accessor** — `FpBudget::remaining_headroom()` returns
   `max(0.0, budget_pct - observed_fp_rate)` so callers can see slack without
   re-implementing the clamp.
3. **Window-merge constructor** — `FpBudget::merge(&self, other: &FpBudget)` sums two
   non-overlapping observation windows into a single `FpBudget`, with constructor-equivalent
   validation (matching `budget_pct`, non-zero merged total).

## New public surface

### `FpBudgetError` variant

```
BudgetPctMismatch   // merge called on budgets with different budget_pct
```

`ZeroTotalEvals` already present — reused when merged `total_evals` sums to zero.
`InvalidBudgetPct` already present — reused by constructor path.

### `SeverityWeight`

```rust
pub struct SeverityWeight {
    pub low: f64,
    pub medium: f64,
    pub high: f64,
}

impl SeverityWeight {
    pub fn weight_for(&self, level: RiskLevel) -> f64;
}
```

### `FpBudget` new methods

```rust
impl FpBudget {
    /// Returns budget_pct - observed_fp_rate, clamped to >= 0.0.
    pub fn remaining_headroom(&self) -> f64;

    /// Severity-weighted FP aggregate over a slice of (RiskLevel, count) pairs.
    /// Returns the sum of weight_for(level) * count for each pair.
    pub fn weighted_fp(&self, sw: &SeverityWeight, findings: &[(RiskLevel, u32)]) -> f64;

    /// Merge two non-overlapping observation windows.
    /// Errors: BudgetPctMismatch if budget_pct values differ,
    ///         ZeroTotalEvals if sum of total_evals is zero.
    pub fn merge(&self, other: &FpBudget) -> Result<FpBudget, FpBudgetError>;
}
```

## Invariants

- `weighted_fp` is self-referentially pure: it does not read `self.observed_fp` or
  `self.total_evals`; it folds the caller-supplied slice only.
- `remaining_headroom` is `f64::max(0.0, self.budget_pct - self.observed_fp_rate())`.
- `merge` uses saturating addition for `observed_fp` and `total_evals` to prevent
  overflow on pathological inputs, then validates the merged total.
- No I/O, no side effects, no new dependencies.

## OpenSLO integration

Feeds the existing `guardrails-shadow-mode-fp-budget` SLO indicator. No behavior change
to `decide_guardrail` or `decide_guardrail_shadow`.

## Acceptance criteria

| Scenario | Expected |
|---|---|
| `remaining_headroom` at exact budget | `0.0` |
| `remaining_headroom` below budget | `> 0.0` |
| `remaining_headroom` above budget | `0.0` (clamped) |
| `weighted_fp` with Low=1.0, Medium=2.0, High=3.0, counts 1/1/1 | `6.0` |
| `merge` happy path | sums `observed_fp` and `total_evals`, shares `budget_pct` |
| `merge` mismatched `budget_pct` | `Err(FpBudgetError::BudgetPctMismatch)` |
| `merge` both zero `total_evals` | `Err(FpBudgetError::ZeroTotalEvals)` |
