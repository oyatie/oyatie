# Spec: metering-domain-window-rollup-kernel

## Objective

Extend `metering-domain` with a deterministic, I/O-free window rollup kernel.
Given a populated `Meter` and a closed time-window `[start, end]` (Unix epoch seconds),
produce a stable-ordered aggregate mapping `(tenant_id, capability_id, MeterUnitKind) →
total_quantity_microunits`.

## Crate Boundary

All changes are confined to `crates/metering-domain/src/lib.rs`.
No new workspace members. No new crate dependencies.

## Flat Clean-Architecture Mod Layout

The crate is a single-file library (`src/lib.rs`). The window rollup kernel is added
directly to `lib.rs` following the existing flat pattern (no new sub-modules required
for a single cohesive addition of this size).

Public surface added:

```rust
pub struct RollupKey {
    pub tenant_id: String,
    pub capability_id: String,
    pub unit_kind: MeterUnitKind,
}

pub struct MeterRollup {
    pub totals: BTreeMap<RollupKey, u64>,
}

pub fn rollup_window(
    meter: &Meter,
    window_start_epoch_s: u64,
    window_end_epoch_s: u64,
) -> MeterRollup
```

## Contracts

- **AsyncAPI / proto**: no new event type; rollup is an internal query path only.
- **OpenAPI**: no REST surface in this slice.
- **OTel**: no metrics/traces added (pure-compute kernel; I/O-free).

## Testing Strategy

All tests are `#[cfg(test)]` hermetic unit tests in `lib.rs`. They cover:

1. Window-boundary inclusion/exclusion (exact-boundary events, just-outside events).
2. Per-`(tenant, cap, kind)` sum correctness across multiple events.
3. Distinct unit kinds kept as separate rollup keys.
4. Idempotent replay: replayed events (same idempotency key) do not double-count
   because `Meter::record` deduplicates before rollup ever runs.
5. `u64::MAX` overflow guard via saturating addition.
6. Empty and inverted windows return empty `MeterRollup`.

## Observability / SLO

This is a pure-compute kernel (no I/O, no network calls). No SLO file is required.
OTel instrumentation is the responsibility of the calling service layer, not the domain kernel.

## Security

- No secrets or PII in rollup keys; tenant/capability IDs are already `INTERNAL_ONLY`
  classified values unwrapped from `Classified<T>` via `.value` accessor.
- Saturating accumulation prevents integer overflow panic in adversarial input.
