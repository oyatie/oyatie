# Plan: obs-cloud-kernel-signal-headroom-report

**Crate**: `oya-cloud-observability-kernel`  
**Lane**: observability  
**Priority**: high  
**Effort**: S

## Objective

Extend the pure zero-dependency kernel with a non-throwing per-signal headroom report companion to the existing `admit_budget()`.

## Tasks

1. Add `pub struct SignalHeadroom` with fields: `signal: SignalKind`, `max: u64`, `aggregate: u64`, `remaining: u64`, `over_budget: bool`.
2. Add `pub fn budget_headroom(plans: &[EmissionPlan], envelopes: &[CardinalityEnvelope]) -> Result<Vec<SignalHeadroom>, ObservabilityError>` that:
   - Phase 1: Apply EmptyPlanId + NoEnvelopeForSignal structural guards (same as admit_budget)
   - Phase 2: Accumulate per-signal aggregates via fixed-size `[u64; 4]` ordinal array + saturating_add
   - Phase 3: For each seen signal, compute `remaining = max.saturating_sub(aggregate)`, `over_budget = aggregate > max`
   - Return `Vec<SignalHeadroom>` in deterministic SignalKind ordinal order (Trace=0, Metric=1, Log=2, Profile=3)
3. Add hermetic unit tests covering:
   - under-budget case (remaining = max - aggregate, over_budget=false)
   - at-boundary case (remaining=0, over_budget=false)
   - over-budget case (remaining=0, over_budget=true)
   - saturating arithmetic (u64::MAX twice, no panic)
   - multi-signal case (deterministic order)
   - structural guard: EmptyPlanId rejected before rollup
   - structural guard: NoEnvelopeForSignal rejected before rollup

## Implementation Notes

- Pure I/O-free, no async, no new dependencies
- Reuse `signal_index` / `index_signal` pattern from `admit_budget`
- `remaining = max.saturating_sub(aggregate)` ensures no underflow
- `over_budget = aggregate > max` is the single source of truth
- Output order: iterate 0..N, emit only seen signals → Trace < Metric < Log < Profile
