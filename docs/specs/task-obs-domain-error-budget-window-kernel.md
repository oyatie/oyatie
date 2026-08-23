# Spec: obs-domain-error-budget-window-kernel

**Crate**: `observability-domain`  
**Module**: `slo::budget`  
**Lane**: observability  
**Priority**: high  
**Effort**: M

## Purpose

Provide a pure, allocation-free error-budget/burn-rate computation kernel
inside `observability-domain` that converts raw good/bad event counts
plus an `SLOObjective` into the inputs that `classify_burn_rate` consumes,
and a one-call helper that drives the full alert-decision pipeline.

## Public API

### `BudgetWindow`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetWindow {
    pub good_events: u64,
    pub bad_events: u64,
}

impl BudgetWindow {
    pub const fn new(good_events: u64, bad_events: u64) -> Self;
    pub fn total(&self) -> u64; // saturating_add
}
```

### `error_budget_remaining_ratio`

```rust
pub fn error_budget_remaining_ratio(
    objective: SLOObjective,
    window: BudgetWindow,
) -> f64
```

Returns a value in [0.0, 1.0]:
- `1.0` = full budget intact
- `0.0` = budget exhausted (clamped; never negative)
- `total == 0` → `1.0` (fail-open)
- `target_ratio == 1.0`, no bad events → `1.0`
- `target_ratio == 1.0`, any bad events → `0.0`

### `burn_rate`

```rust
pub fn burn_rate(
    objective: SLOObjective,
    window: BudgetWindow,
) -> f64
```

Observed bad ratio divided by allowed bad ratio:
- `0.0` when `total == 0` or no bad events
- `f64::INFINITY` when `target_ratio == 1.0` and bad events > 0
- `bad_events` saturated to `total` before ratio computation

### `classify_budget_windows`

```rust
pub fn classify_budget_windows(
    objective: SLOObjective,
    fast_window: BudgetWindow,
    slow_window: BudgetWindow,
) -> AlertDecision
```

One-call adapter:
1. Derives `error_budget_consumed = 1.0 - error_budget_remaining_ratio(objective, fast_window)`
2. Computes `burn_rate(objective, fast_window)` and `burn_rate(objective, slow_window)`
3. Delegates to `classify_burn_rate(consumed, fast_burn, slow_burn)`

## Re-exports

All four items are re-exported from the crate root (`lib.rs`):

```rust
pub use slo::budget::{
    BudgetWindow, burn_rate, classify_budget_windows, error_budget_remaining_ratio,
};
```

## Acceptance Criteria

- [x] Pure / no-I/O; no allocation on hot path
- [x] Deterministic output for identical inputs
- [x] `total == 0`: fail-open (remaining = 1.0, burn = 0.0)
- [x] `bad_events > total`: saturated to `total`
- [x] `target_ratio == 1.0`: handled without panic
- [x] Remaining ratio clamped to [0.0, 1.0] (never negative)
- [x] `burn_rate` returns `f64::INFINITY` at zero-tolerance boundary
- [x] `u64::MAX` inputs: `saturating_add` prevents overflow
- [x] ≥ 8 unit tests (13 implemented)
- [x] PAGE/TICKET thresholds wire through to `AlertDecision` via helper
- [x] Re-exported from `lib.rs`
- [x] No change to existing `classify_burn_rate` signature or thresholds
- [x] No root `Cargo.toml` edit

## Test Coverage

| # | Test name | Covers |
|---|-----------|--------|
| 1 | `clean_budget_no_bad_events_returns_full_remaining_and_zero_burn` | Clean budget |
| 2 | `exhausted_budget_returns_zero_remaining` | Exhausted budget |
| 3 | `partial_budget_returns_proportional_remaining` | Partial consumption |
| 4 | `zero_total_events_returns_full_remaining_and_zero_burn` | Zero traffic |
| 5 | `bad_events_exceeding_total_saturates_at_total` | bad > total saturation |
| 6 | `target_ratio_one_no_bad_events_returns_full_remaining` | 100% target, clean |
| 7 | `target_ratio_one_with_bad_events_returns_zero_remaining_and_infinite_burn` | 100% target, failure |
| 8 | `classify_budget_windows_page_fires_on_high_burn_both_windows` | PAGE threshold wiring |
| 9 | `classify_budget_windows_ticket_fires_on_moderate_burn_both_windows` | TICKET threshold wiring |
| 10 | `classify_budget_windows_none_on_low_burn` | No alert |
| 11 | `burn_rate_at_exactly_page_boundary_returns_page` | Above PAGE boundary |
| 12 | `over_budget_remaining_is_clamped_to_zero_not_negative` | Clamp |
| 13 | `budget_window_total_saturates_on_overflow` | `u64::MAX` safety |

## Implementation Notes

- Module placement: inline `pub mod budget` at the bottom of `slo.rs`
  (before the existing tests block) — cohesive with SLO vocabulary, no
  separate file needed for this scope.
- `data_class: INTERNAL_ONLY` annotated on all public items per project policy.
- ADR-0083 Tier 3 clippy exemptions inherited from the file-level
  `#![cfg_attr(test, allow(...))]` attribute.
