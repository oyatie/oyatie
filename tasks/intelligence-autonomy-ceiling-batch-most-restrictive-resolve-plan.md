# Plan: intelligence-autonomy-ceiling-batch-most-restrictive-resolve

## Objective
Extend `oya-intelligence-autonomy-ceiling-kernel` with a pure batch resolver over an ordered
slice of `(surface, requested AutonomyTier)` pairs.

## Scope
Single crate: `microservices/intelligence/crates/oya-intelligence-autonomy-ceiling-kernel`

## Steps

1. Add `BatchCeilingVerdict` value type to `tenant_ceiling.rs`:
   - `items: Vec<TenantCeilingVerdict>` — per-request verdict in input order
   - `most_restrictive_clamp: Option<AutonomyTier>` — lowest effective ceiling among
     Clamped items; `None` when all Permitted (or empty input)

2. Add `resolve_batch(requests: &[(String, AutonomyTier)], ceiling: &TenantCeiling) -> BatchCeilingVerdict`:
   - Iterate input in order, call `resolve()` for each item
   - Track the minimum effective ceiling from Clamped items
   - Empty input → `items=[]`, `most_restrictive_clamp=None`

3. Update `lib.rs` pub re-exports to include `BatchCeilingVerdict` and `resolve_batch`.

4. Add `#[cfg(test)]` unit tests covering:
   - Empty batch → all-permitted / None aggregate
   - All-permitted batch → None aggregate
   - Mixed permitted+clamped → correct most-restrictive selection
   - Surface-override interaction
   - Ordering-independence of the aggregate (same items, different order → same aggregate)

5. Verify: `cargo check -p oya-intelligence-autonomy-ceiling-kernel --all-targets`
   and `cargo nextest run -p oya-intelligence-autonomy-ceiling-kernel`

## Constraints
- No new dependencies
- No I/O
- No new workspace member
- No root Cargo.toml edit
- T4-disabled-by-default semantics preserved
