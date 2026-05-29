# Task Plan: cloud-obs-cardinality-budget-rollup

**Lane:** observability  
**Crate:** `oya-cloud-observability-kernel`  
**Branch:** `feat/task-cloud-obs-cardinality-budget-rollup-2026-05-28`  
**Base:** `origin/dev`

---

## Objective

Extend the pure cardinality-envelope admission kernel with a multi-plan `EmissionBudget`
rollup function (`admit_budget`). A single `admit_plan` call cannot detect the case where
each plan passes its envelope individually but the per-signal aggregate of all plans exceeds
the declared envelope. `admit_budget` closes that gap with a saturating aggregate then
rejects on overage, emitting a new `AggregateEnvelopeExceeded` variant.

---

## Subtasks

### [co-1] Add `admit_budget` pure function

**Description.**  
Add `pub fn admit_budget(plans: &[EmissionPlan], envelopes: &[CardinalityEnvelope]) -> Result<(), ObservabilityError>`
to `src/lib.rs`.

Algorithm:
1. For each plan in `plans`, run the existing `EmptyPlanId` and `NoEnvelopeForSignal` guards
   (reuse the existing code paths — do not duplicate the lookup logic ad-hoc).
2. Group plans by `SignalKind`; accumulate `estimated_combinations` per signal using
   `saturating_add` (overflow-safe for u64::MAX inputs).
3. After the grouping pass, for each `(signal, aggregate)` pair, look up its envelope and
   reject with `AggregateEnvelopeExceeded { signal, max, aggregate }` if the aggregate
   exceeds `max_unique_attribute_combinations`.

**Acceptance criteria:**
- `admit_budget` exists with the exact pure signature above.
- Uses `saturating_add` for the per-signal accumulation.
- Reuses existing `EmptyPlanId` and `NoEnvelopeForSignal` guards (called via the existing
  helper or inline, not duplicated with different semantics).
- `cargo check -p oya-cloud-observability-kernel --all-targets` is clean.

---

### [co-2] Add `ObservabilityError::AggregateEnvelopeExceeded` variant

**Description.**  
Add the new error variant to `ObservabilityError`:

```rust
AggregateEnvelopeExceeded { signal: SignalKind, max: u64, aggregate: u64 }
```

Extend `ObservabilityError::message()` with a match arm that formats a stable,
low-cardinality, data-class-safe operational message — no payload, no attribute values,
no plan IDs. Format: `"aggregate cardinality envelope exceeded: signal={} max={} aggregate={}"`.

**Acceptance criteria:**
- New variant and `message()` arm compile without errors.
- The message string is stable and low-cardinality: contains only the signal name (a static
  string) plus the two integers `max` and `aggregate`. No dynamic user/tenant/attribute data.

---

### [co-3] Add inline `#[cfg(test)]` cases

**Description.**  
Inside the existing `#[cfg(test)] mod tests` block in `src/lib.rs`, add four new test functions:

| Test name | Scenario | Expected result |
|---|---|---|
| `aggregate_over_envelope_rejected` | Two plans, same signal, each under-envelope individually, sum exceeds envelope | `Err(AggregateEnvelopeExceeded { .. })` |
| `aggregate_at_boundary_passes` | Two plans, same signal, sum equals envelope exactly | `Ok(())` |
| `aggregate_no_envelope_for_signal_rejected` | One plan for a signal with no declared envelope | `Err(NoEnvelopeForSignal { .. })` |
| `aggregate_saturating_add_no_panic` | Two plans with `u64::MAX` estimated_combinations, same signal | Does not panic; returns `Err(AggregateEnvelopeExceeded { .. })` (saturating sum stays at `u64::MAX`, exceeds any realistic envelope) |

**Acceptance criteria:**
- `cargo nextest run -p oya-cloud-observability-kernel` passes all new and pre-existing tests.
- All four named scenarios are present and green.

---

## Ordering

`co-2` must land before `co-1` (the new error variant must exist before `admit_budget`
references it). `co-3` follows `co-1`.

Commit order: `co-2` -> `co-1` -> `co-3` (or all together in one commit if implemented
atomically).

---

## Boundaries

- **Only** `crates/oya-cloud-observability-kernel/src/lib.rs` is modified.
- **No** new crates, **no** root `Cargo.toml` edits.
- **No** I/O, **no** async, **no** external dependencies added.
- **No** changes to any other crate.
