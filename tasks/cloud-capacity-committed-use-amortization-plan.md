# Plan: cloud-capacity-committed-use-amortization

## Objective

Extend `oya-cloud-capacity-kernel`'s `committed_use` module with three pure deterministic
functions for amortization and coverage math over existing types. No new struct fields on
existing types, no new dependencies, no I/O, no async.

## Tasks

1. Add `effective_discounted_rate` to `committed_use.rs`
   - Signature: `pub fn effective_discounted_rate(list_rate_micros: u128, discount_bps: u32) -> u128`
   - Saturating arithmetic; 0 bps returns list_rate unchanged; 10000 bps returns 0
   - Callers are expected to pre-validate bps ≤ 10000 via `validate_committed_use_contract`

2. Add `amortized_monthly_commit_micros` to `committed_use.rs`
   - Signature: `pub fn amortized_monthly_commit_micros(total_commit_micros: u128, term: ReservationTerm) -> u128`
   - Divides by `term.months()` using integer division; remainder distributed to first month
     (spec says "exact integer division + remainder handling" — interpret as floor div; callers
     who need remainder can compute `total % months`)
   - Guard against zero months (impossible with current enum but defensive)

3. Add `committed_coverage_bps` to `committed_use.rs`
   - Signature: `pub fn committed_coverage_bps(reserved_units: u64, demand_units: u64) -> u32`
   - Returns 0 when demand_units == 0
   - Returns min(reserved*10000/demand, 10000) capped at 10000

4. Add new `CommittedUseError` variant `ZeroListRate`
   - Returned by `effective_discounted_rate` when `list_rate_micros == 0`

5. Export new fns from `lib.rs` via `committed_use` re-export

6. Write inline `cfg(test)` unit tests:
   - Discount at 0 bps boundary
   - Discount at 10000 bps boundary (returns 0)
   - `validate_committed_use_contract` rejects discount_bps > 10000
   - Amortization exact division (12 months, no remainder)
   - Amortization with remainder (e.g. 13 micros over 12 months)
   - Coverage saturates at 10000 when reserved >= demand
   - Zero demand returns 0

## Constraints

- No new workspace members; no edits to root Cargo.toml
- std-only; zero new dependencies
- Clippy clean under workspace lints
- Existing tests must continue to pass
