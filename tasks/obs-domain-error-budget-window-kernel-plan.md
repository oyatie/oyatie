# Plan: obs-domain-error-budget-window-kernel

## Objective

Add a pure error-budget/burn-rate computation kernel (`slo::budget`) to
`oya-observability-domain` that produces the inputs `classify_burn_rate`
already consumes, plus a one-call helper.

## Scope

- Crate: `crates/oya-observability-domain`
- Module: `src/slo.rs` — new inline `pub mod budget`
- Re-exports: `src/lib.rs`
- No new workspace members, no root `Cargo.toml` edits

## Task List

### 1. Explore existing slo.rs API [completed]

Read `slo.rs` and `lib.rs` to understand `SLOObjective`, `AlertDecision`,
`classify_burn_rate`, and existing thresholds.

### 2. Design BudgetWindow and pure fns [completed]

- `BudgetWindow { good_events: u64, bad_events: u64 }` — pure value object
- `error_budget_remaining_ratio(obj, w) -> f64` in [0,1]
- `burn_rate(obj, w) -> f64`
- `classify_budget_windows(obj, fast, slow) -> AlertDecision` — one-call helper

Edge-case handling:
- `total == 0`: `remaining = 1.0`, `burn_rate = 0.0` (fail-open)
- `bad_events > total`: saturate via `effective_bad = bad.min(total)`
- `target_ratio == 1.0`: `burn_rate = INFINITY` on any bad event
- Over-consumed budget: clamp `remaining` to 0.0 (never negative)
- `u64::MAX` overflow in `total()`: use `saturating_add`

### 3. Implement slo::budget module [completed]

Added `pub mod budget` at the bottom of `slo.rs`, before the existing
`#[cfg(test)]` block.

### 4. Update lib.rs re-exports [completed]

Added `pub use slo::budget::{BudgetWindow, burn_rate, classify_budget_windows, error_budget_remaining_ratio};`

### 5. Write ≥8 unit tests [completed]

13 unit tests covering:
1. Clean budget (no bad events) → full remaining + zero burn
2. Exhausted budget → zero remaining
3. Partial budget → proportional remaining
4. Zero total events → full remaining + zero burn
5. bad_events > total saturation
6. target_ratio == 1.0, no bad events → full remaining
7. target_ratio == 1.0, bad events → zero remaining + INFINITY burn
8. PAGE alert fires on high burn (both windows)
9. TICKET alert fires on moderate burn (both windows)
10. No alert on low burn
11. Burn rate above PAGE boundary fires PAGE
12. Over-budget remaining clamped to 0.0
13. BudgetWindow total() saturating add

### 6. Verification [completed]

- `cargo check -p oya-observability-domain --all-targets`: PASS
- `cargo nextest run -p oya-observability-domain`: 99/99 PASS

## Decisions

- Placed `budget` as an inline `pub mod` inside `slo.rs` rather than a
  separate file; the module is cohesive with SLO vocabulary and the spec
  allowed "extension of slo.rs".
- `total == 0` → fail-open (remaining = 1.0, burn = 0.0): no traffic is
  not evidence of failure; this is consistent with the SRE principle that
  absence of data should not trigger pages.
- `burn_rate` returns `f64::INFINITY` for `target_ratio == 1.0` with bad
  events, which is mathematically correct (division by zero allowed bad ratio)
  and safely handled by `classify_burn_rate` comparisons (INFINITY >= any
  finite threshold).
