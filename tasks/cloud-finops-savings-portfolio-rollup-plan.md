# Plan: cloud-finops-savings-portfolio-rollup

## Objective
Add a pure deterministic portfolio-rollup module to `oya-cloud-finops-kernel` that
aggregates `Recommendation` slices against their baseline `CostReport`(s) into a
`SavingsPortfolio` projection.

## Scope
- Crate: `oya-cloud-finops-kernel` (flat clean-arch mod, no new workspace member)
- New file: `src/savings_portfolio.rs`
- Exports wired through `lib.rs`

## Steps
1. Write failing unit tests (red) in `savings_portfolio.rs`
2. Implement `SavingsPortfolio`, `RollupError`, `roll_up_savings` to make tests green
3. Wire exports in `lib.rs`
4. `cargo check -p oya-cloud-finops-kernel --all-targets`
5. `cargo nextest run -p oya-cloud-finops-kernel`
6. Self-review: clippy clean, no debug code

## Acceptance Criteria
- `pub fn roll_up_savings` + `SavingsPortfolio` struct + `RollupError` enum exported
- std-only, no new deps, no I/O/async
- Unit tests covering:
  a. empty set -> zero portfolio
  b. mixed states only counts Active/Applied
  c. missing-baseline report_id -> `RollupError`
  d. coverage_bps saturates at 10000 when savings >= spend
  e. per-kind counts correct
- clippy clean under workspace lints
