# Plan: metering-domain-window-rollup-kernel

## Objective

Add a pure, I/O-free window rollup kernel to `oya-metering-domain` that groups recorded
`MeterEvent` values within a `[window_start_epoch_s, window_end_epoch_s]` interval by
`(tenant_id, capability_id, MeterUnitKind)` and deterministically sums
`quantity_microunits` with saturating/overflow-checked accumulation.

## Edge Cases & Acceptance Criteria

1. **Window boundary inclusion**: only events where
   `window_start_epoch_s <= recorded_at_epoch_seconds <= window_end_epoch_s` are included;
   events outside the window are excluded.
2. **Per-key sum correctness**: multiple events for the same
   `(tenant_id, capability_id, unit_kind)` produce a correct cumulative total.
3. **Distinct unit kinds are kept separate**: `(tenant_id, cap_id, Request)` and
   `(tenant_id, cap_id, ByteOut)` remain distinct keys in the rollup output.
4. **Idempotent-replay semantics**: because the `Meter` deduplicates by idempotency key,
   a replayed event is NOT double-counted — `rollup_window` simply iterates whatever
   events the `Meter` currently holds (dedup already happened at record time).
5. **Overflow handling**: accumulation uses `u64::saturating_add` so that u64 overflow
   never panics; value is capped at `u64::MAX`.
6. **Stable-ordered output**: result is a `BTreeMap` keyed on
   `(tenant_id: String, capability_id: String, unit_kind: MeterUnitKind)` — deterministic
   iteration order guaranteed by `BTreeMap` + derived `Ord`.
7. **Empty window**: returns an empty `BTreeMap` without error.
8. **Inverted window** (`end < start`): returns an empty `BTreeMap` (no events qualify).

## Contract Implications

- Pure function: `fn rollup_window(meter: &Meter, window_start_epoch_s: u64, window_end_epoch_s: u64) -> MeterRollup`
- No new dependencies; no I/O.
- `MeterRollup` is a newtype wrapping `BTreeMap<RollupKey, u64>` where
  `RollupKey = (String, String, MeterUnitKind)`.
- All types derive `Debug`, `Clone`, `Eq`, `PartialEq`.

## Subtasks (ordered)

- [x] Write spec (`docs/specs/task-metering-domain-window-rollup-kernel.md`)
- [x] Write red tests in `crates/oya-metering-domain/src/lib.rs`
- [x] Implement `MeterRollup`, `RollupKey`, `rollup_window` to make tests green
- [x] `cargo check -p oya-metering-domain --all-targets` passes
- [x] `cargo nextest run -p oya-metering-domain` green
- [x] Self-review (correctness/security/perf/cloud-native)
- [x] Simplify pass; re-verify green
