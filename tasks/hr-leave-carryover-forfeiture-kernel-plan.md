# Plan: hr-leave-carryover-forfeiture-kernel

## Objective

Add `evaluate_leave_carryover_forfeiture` — a pure deterministic period-boundary
function that splits a closing balance into `carried_over_units` (clamped to
`[floor, cap]`) and `forfeited_units` (excess above cap), instead of hard-erroring
like the existing `evaluate_leave_balance_accrual` does when the cap is exceeded.

## Crate

`oya-hr-employment-domain` — flat single-crate; changes in `src/lib.rs` only.
No new workspace member. No new file.

## Edge cases

| Scenario | Expected behaviour |
|---|---|
| balance <= cap | forfeited=0, carried_over=balance |
| balance > cap | forfeited=balance-cap, carried_over=cap |
| balance < floor | carried_over=floor (statutory minimum), forfeited=0 |
| cap < floor | `Err(CarryOverCapBelowFloor)` — invalid policy |
| negative / NaN inputs | `Err(InvalidAccrualUnits)` |
| floor < 0 | `Err(InvalidAccrualUnits)` |
| balance == floor == cap | forfeited=0, carried_over=floor |

## Acceptance criteria (tests)

- (a) balance <= cap → zero forfeiture
- (b) balance > cap → forfeited = balance − cap, carried_over = cap
- (c) floor enforcement when balance < floor → carried_over = floor
- (d) cap < floor → `Err(CarryOverCapBelowFloor)`
- (e) negative / NaN inputs → `Err(InvalidAccrualUnits)`
- classification on every FINANCIAL field asserted
- no I/O, no new workspace member

## New additions to `src/lib.rs`

1. `const LEAVE_CARRYOVER_FORFEITURE_SCHEMA_VERSION: u32 = 1;`
2. New error variant `HrDomainError::CarryOverCapBelowFloor`
3. `LeaveCarryoverForfeitureInput` struct (plain, matches accrual-input style)
4. `LeaveCarryoverForfeitureProjection` struct (Classified fields, FINANCIAL-classed units)
5. `pub fn evaluate_leave_carryover_forfeiture(input) -> Result<Projection, HrDomainError>`

## Test file

`tests/leave_carryover_forfeiture.rs` — hermetic unit tests, no I/O.

## Subtasks (ordered)

1. [x] Write plan (this file)
2. [ ] Write spec (`docs/specs/task-hr-leave-carryover-forfeiture-kernel.md`)
3. [ ] Write tests (RED phase) — `tests/leave_carryover_forfeiture.rs`
4. [ ] Confirm tests fail (cargo check `--all-targets` or nextest `--no-run`)
5. [ ] Implement in `src/lib.rs` (GREEN phase)
6. [ ] Verify GREEN: `cargo nextest run -p oya-hr-employment-domain`
7. [ ] Self-review (correctness / arch / security / perf / cloud-native)
8. [ ] Simplify pass
9. [ ] Final nextest run + git add + commit
