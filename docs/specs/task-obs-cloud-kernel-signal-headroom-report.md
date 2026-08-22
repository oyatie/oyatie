# Spec: obs-cloud-kernel-signal-headroom-report

**Crate**: `cloud-observability-kernel`  
**Lane**: observability  
**Priority**: high  
**Effort**: S

## Summary

Extend the pure zero-dependency kernel with a non-throwing per-signal headroom report
companion to the existing `admit_budget()`. Returns a `Vec<SignalHeadroom>` — one entry
per seen SignalKind — in deterministic ordinal order.

## New Public API

### `SignalHeadroom`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHeadroom {
    pub signal: SignalKind,
    pub max: u64,
    pub aggregate: u64,
    pub remaining: u64,
    pub over_budget: bool,
}
```

### `budget_headroom`

```rust
pub fn budget_headroom(
    plans: &[EmissionPlan],
    envelopes: &[CardinalityEnvelope],
) -> Result<Vec<SignalHeadroom>, ObservabilityError>
```

## Behaviour

1. **Phase 1 — structural guards** (same as `admit_budget`):
   - Any plan with `plan_id.is_empty()` → `Err(ObservabilityError::EmptyPlanId)`
   - Any plan whose `signal` has no matching `CardinalityEnvelope` → `Err(ObservabilityError::NoEnvelopeForSignal { signal })`
   - Returns the first error encountered.

2. **Phase 2 — accumulate**:
   - Group `estimated_combinations` by `SignalKind` via fixed-size `[u64; 4]` ordinal array.
   - Use `saturating_add` to prevent overflow/panic.
   - Track which indices were seen via `[bool; 4]`.

3. **Phase 3 — build report**:
   - For each seen signal (in ordinal order Trace=0, Metric=1, Log=2, Profile=3):
     - `remaining = max.saturating_sub(aggregate)`
     - `over_budget = aggregate > max`
   - Collect into `Vec<SignalHeadroom>` — length equals number of distinct seen signals.

## Invariants

- `remaining = 0` when `aggregate >= max` (saturating_sub).
- `over_budget = true` iff `aggregate > max`.
- At boundary (`aggregate == max`): `remaining = 0`, `over_budget = false`.
- Output order is strictly deterministic: Trace < Metric < Log < Profile.
- No I/O, no async, no new dependencies.
- Saturating arithmetic prevents any panic on `u64::MAX` inputs.

## Acceptance Criteria

| Scenario | Expected |
|---|---|
| Under budget: aggregate < max | remaining = max - aggregate, over_budget = false |
| At boundary: aggregate == max | remaining = 0, over_budget = false |
| Over budget: aggregate > max | remaining = 0, over_budget = true |
| Saturating: two plans each with u64::MAX | no panic, over_budget = true |
| Multi-signal: Trace + Metric plans | two entries in Trace, Metric order |
| EmptyPlanId | Err(ObservabilityError::EmptyPlanId) |
| NoEnvelopeForSignal | Err(ObservabilityError::NoEnvelopeForSignal) |
