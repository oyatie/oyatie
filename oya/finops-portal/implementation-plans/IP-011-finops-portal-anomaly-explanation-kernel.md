---
ip_id: IP-011
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/anomaly-explanation/kernel
related_adrs: [ADR-0083, ADR-0131, ADR-0199]
depends_on: []
target_lines: 150
---

# IP-011 — `anomaly-explanation` kernel slice

## Why this slice

When `TenantCostAnomalySpike` fires (1.5x rolling 7-day average),
the on-call team needs root-cause attribution within seconds. The
anomaly-explanation BC produces a deterministic, explainable
breakdown of a spike: which dimension grew, by how much, and what
correlated events happened.

The kernel hosts the pure types + the explanation algorithm. Per
ADR-0083 Tier-A invariants, the kernel is dependency-free + sync-
pure; statistical computations are simple-arithmetic only (no
external statistics crate beyond `std`).

## Acceptance criteria

1. New crate
   `crates/oya-finops-portal-anomaly-explanation-kernel/`.
2. Public types:
   - `Anomaly` — `tenant_id, fired_at, observed_cost,
     baseline_cost, severity`.
   - `Dimension` — enum: `CostCenter`, `WorkloadClass`,
     `Cell`, `Capability`, `Region`.
   - `Contribution` — `dimension, value, baseline,
     contribution_pct_of_delta`.
   - `Explanation` — `anomaly, top_contributions: Vec<Contribution>,
     correlated_events: Vec<CorrelatedEvent>, confidence: f32`.
   - `CorrelatedEvent` — `event_class, event_time, weight`.
3. Public function `explain`:
   ```rust
   pub fn explain(
       anomaly: &Anomaly,
       slices: &[CostSlice],
       events: &[CorrelatedEvent],
   ) -> Explanation;
   ```
4. The algorithm is deterministic: same input produces same output
   byte-for-byte (so the explanation is auditable).
5. Tier-A 4-INV: no `std::io`, no `tokio`, all wire-types
   `#[non_exhaustive]`, `Anomaly` impls `Ord` by `fired_at`.
6. ≥ 6 unit tests:
   - largest dimension dominates `top_contributions`.
   - ties broken deterministically (alphabetical dimension name).
   - confidence drops with thinner data (fewer slices).
   - correlated event within ±1h window included.
   - correlated event outside ±1h excluded.
   - empty slices returns `Explanation { confidence: 0.0,
     top_contributions: vec![] }` with empty rationale.
7. `cargo test -p oya-finops-portal-anomaly-explanation-kernel`
   green.

## File-level work plan

1. `Cargo.toml` — `serde`, `thiserror`, `time`.
2. `src/lib.rs`.
3. `src/types.rs`.
4. `src/explain.rs` — algorithm.
5. `src/confidence.rs` — confidence scoring.
6. `src/correlate.rs` — event correlation.

## Algorithm: contribution-by-dimension

For each `Dimension`:

1. Compute total observed cost in the anomaly window.
2. Compute total baseline cost in the prior 7-day equivalent window.
3. Compute `delta = observed - baseline` for each dimension value.
4. Sort dimension values by `delta` descending.
5. Pick top-3 contributors per `Dimension`.
6. Compute `contribution_pct_of_delta = delta / total_delta`.

The output is **deterministic** (no random sampling, no time
dependency beyond input) because the algorithm is purely
positional over the input slices.

## Confidence scoring

`confidence` ∈ [0.0, 1.0]:

- 1.0 if ≥ 168 hourly slices baseline + ≥ 60 slices observed.
- Falls off linearly with fewer slices.
- 0.0 if observed window has zero data.

This is a transparent, hand-tunable function — not ML — because
the explanation must be auditable per ADR-0199's audit-chain
requirement on anomaly events.

## Correlation window

A `CorrelatedEvent` is included in `correlated_events` if:

- `event_time` is within `[anomaly.fired_at - 1h, anomaly.fired_at + 1h]`.
- `event_class` is in the allowlist: `Deploy`, `ConfigChange`,
  `CostAllocationPolicyChanged`, `CapacityChange`, `BackupRetentionBump`.
- The event's `weight` (caller-supplied) is ≥ 0.1.

Determinism: events are sorted by `event_time` ascending, ties
broken by `event_class`.

## EU AI Act risk class

This kernel is **NOT** an automated decision-maker; it produces an
**explanation** that humans consume. Per the EU AI Act risk
classification (limited risk), the kernel ships with a
`capabilities/anomaly-explanation.capability.yaml` declaration
that marks it `risk_class: limited` and `human_in_loop: true`.

## Risk + mitigation

- **Risk**: non-determinism through `HashMap` iteration.
  **Mitigation**: use `BTreeMap` throughout; covered by a
  determinism unit test that repeats `explain` 100x and asserts
  byte equality.

## Out-of-scope

- Anomaly detection (Prometheus rules own it).
- Adapters to Mimir / OpenCost (usecase layer).
- UI presentation (api / app layers).

## References

- ADR-0083 — Tier-A kernel invariants.
- ADR-0199 — FinOps + audit-chain.
- `capabilities/anomaly-explanation.capability.yaml`.

## Verification

- `cargo test -p oya-finops-portal-anomaly-explanation-kernel`.
- `oya gate kernel-tier-invariants --crate
  oya-finops-portal-anomaly-explanation-kernel`.
