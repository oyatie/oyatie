# Spec: cloud-capacity-committed-use-amortization

## Crate
`cloud-capacity-kernel`

## Module
`committed_use`

## Summary

Extend the `committed_use` module with three pure deterministic functions for amortization
and committed-coverage math over existing `CommittedUseContract` / `ReservedCapacity` types.
No new struct fields on existing types. No new dependencies. No I/O. No async.

## New Public Functions

### `effective_discounted_rate`

```
pub fn effective_discounted_rate(list_rate_micros: u128, discount_bps: u32) -> Result<u128, CommittedUseError>
```

- Returns `Err(CommittedUseError::ZeroListRate)` when `list_rate_micros == 0`.
- At `discount_bps == 0`: returns `list_rate_micros` unchanged.
- At `discount_bps == 10000`: returns `0`.
- For values in `(0, 10000)`: computes `list_rate_micros - (list_rate_micros * discount_bps as u128 / 10_000)` using saturating arithmetic.
- Callers must pre-validate `discount_bps <= 10000` via `validate_committed_use_contract`;
  values > 10000 are accepted defensively (saturating sub bottoms at 0).

### `amortized_monthly_commit_micros`

```
pub fn amortized_monthly_commit_micros(total_commit_micros: u128, term: ReservationTerm) -> u128
```

- Uses `term.months()` as divisor (always >= 12 with current enum variants).
- Returns `total_commit_micros / term.months() as u128` (floor integer division).
- Defensive guard: if `term.months() == 0` (currently impossible), returns `total_commit_micros`.
- Remainder (`total_commit_micros % months`) is intentionally dropped; callers who need
  exact reconciliation compute the remainder independently.

### `committed_coverage_bps`

```
pub fn committed_coverage_bps(reserved_units: u64, demand_units: u64) -> u32
```

- Returns `0` when `demand_units == 0`.
- Otherwise returns `min(reserved_units * 10_000 / demand_units, 10_000)` cast to `u32`.
- Saturation cap ensures the result never exceeds 10000 bps (100% coverage).
- Uses `u128` intermediate to prevent overflow on large `reserved_units` values.

## New Error Variant

```
CommittedUseError::ZeroListRate
```

Added to the existing `CommittedUseError` enum. Message: `"list_rate_micros must be non-zero"`.

## Exports

All three functions are re-exported from `lib.rs` via the existing `pub use committed_use::{ ... }` block.
`CommittedUseError::ZeroListRate` is available via the already-exported `CommittedUseError` type.

## Acceptance Criteria

| # | Criterion |
|---|-----------|
| a | `effective_discounted_rate` at 0 bps returns list_rate unchanged |
| b | `effective_discounted_rate` at 10000 bps returns 0 |
| c | `validate_committed_use_contract` rejects `discount_bps > 10000` (existing test, preserved) |
| d | `amortized_monthly_commit_micros` with exact division (e.g. 1200 over OneYear -> 100) |
| e | `amortized_monthly_commit_micros` with remainder (e.g. 13 over OneYear -> 1, remainder 1 dropped) |
| f | `committed_coverage_bps` returns 10000 when reserved >= demand |
| g | `committed_coverage_bps` returns 0 when demand == 0 |
| h | All new fns exported via `lib.rs` committed_use re-export |
| i | No new dependencies; std-only; no I/O; no async |
| j | Clippy clean under workspace lints |
| k | All existing tests pass |

## Constraints

- No new workspace members; no edits to root `Cargo.toml`
- No new struct fields on existing types
- `ZeroListRate` is the only new error variant
