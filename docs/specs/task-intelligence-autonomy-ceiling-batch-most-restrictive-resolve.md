# Spec: intelligence-autonomy-ceiling-batch-most-restrictive-resolve

**Crate**: `intelligence-autonomy-ceiling-kernel`
**Lane**: intelligence
**Priority**: high
**Effort**: M

## Context

`tenant_ceiling.rs` currently exposes a single-surface `resolve` function that
compares a requested `AutonomyTier` against the effective ceiling for a named
surface and returns a `TenantCeilingVerdict`. Many call-sites need to evaluate
multiple surfaces in one logical operation and receive both the per-surface
result and an aggregate "most restrictive" outcome without performing the fold
themselves.

## Proposed API (additive, no breaking changes)

```rust
/// Per-item result plus aggregate most-restrictive clamp.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchCeilingVerdict {
    /// Per-request verdicts in the same order as the input slice.
    pub results: Vec<TenantCeilingVerdict>,
    /// The lowest effective ceiling tier among all `Clamped` items.
    /// `None` when all items are `Permitted` (including when input is empty).
    pub most_restrictive_clamp: Option<AutonomyTier>,
}

/// Batch resolver over an ordered slice of (surface, requested tier) pairs.
///
/// Calls `resolve` for each entry, accumulates per-item verdicts, and
/// computes `most_restrictive_clamp` as the minimum `AutonomyTier` among
/// any `Clamped` results (using `AutonomyTier: Ord`).
///
/// Empty input yields `results: vec![]` and `most_restrictive_clamp: None`.
pub fn resolve_batch(
    requests: &[(String, AutonomyTier)],
    ceiling: &TenantCeiling,
) -> BatchCeilingVerdict;
```

## Acceptance Criteria

| # | Scenario | Expected |
|---|----------|----------|
| A1 | Empty `requests` slice | `results` empty, `most_restrictive_clamp` = None |
| A2 | All requests Permitted | `most_restrictive_clamp` = None |
| A3 | Mix of Permitted + Clamped | `most_restrictive_clamp` = lowest clamped effective tier |
| A4 | Surface override raises ceiling; that item becomes Permitted | Aggregate reflects remaining Clamped items only |
| A5 | Reversing request order | Same `most_restrictive_clamp` (order-independence of aggregate) |
| A6 | All requests Clamped, same tier | `most_restrictive_clamp` = that tier |
| A7 | Multiple clamped tiers | `most_restrictive_clamp` = minimum tier |

## Invariants

- `most_restrictive_clamp` is `None` iff all `results` are `Permitted` or `Denied`
  (with current enum, `Denied` is unreachable; `None` iff all `Permitted`)
- `results.len() == requests.len()` always
- Reuses `resolve` without reimplementing ceiling logic
- No new dependencies; no I/O; deterministic; panic-free in production

## Non-Goals

- No changes to existing `resolve`, `check_tier`, `TenantCeiling` public API
- No async or streaming evaluation
- No cross-tenant aggregation
