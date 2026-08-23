# Spec: obs-domain-severity-threshold-gate

## Objective

The `observability-domain` crate provides the stable telemetry
severity vocabulary used across all Oyatie services. Today `Severity`
exposes `as_otel_int` (one-way, lossy) but has no inverse; a downstream
adapter that receives a raw OTel `SeverityNumber` integer cannot map back
to the canonical `Severity` bucket without duplicating the mapping.
Similarly, there is no shared, pure predicate that downstream emitters
can consult before forwarding a log record to an exporter.

This task adds two tightly scoped additions to `src/severity.rs`:

1. `Severity::from_otel_int(n: u8) -> Option<Severity>` — the tolerant
   inverse of `as_otel_int`, covering the full OTel SeverityNumber
   range-of-ranges (1–24) and returning `None` for out-of-range inputs.

2. `fn should_emit(record_severity: Severity, min_threshold: Severity) -> bool`
   — a pure gate leveraging the existing `Ord` derive: emit iff
   `record_severity >= min_threshold`.

Both are pure, no-alloc, no-I/O, and carry no new dependencies.

## Vertical and crate

- Lane: `observability`
- Crate: `observability-domain`
  (`crates/observability-domain/`)
- ADR alignment: ADR-0131 per-microservice flat layout; ADR-0509
  single-crate-per-service with mod-based subsystems

## Contracts

### No HTTP/gRPC surface

This crate is a pure domain vocabulary library with no network surface.
All contracts are Rust type signatures.

### `Severity::from_otel_int` — OTel SeverityNumber inverse

```rust
/// Map an OTel SeverityNumber integer to its Severity bucket.
///
/// The OTel spec defines six named levels, each spanning a range of
/// four integers (the "severity number ranges"). This function maps any
/// integer in a range to the bucket regardless of position within the
/// range. Returns `None` for 0 and any value > 24.
///
/// | Range | Bucket |
/// |-------|--------|
/// | 1–4   | Trace  |
/// | 5–8   | Debug  |
/// | 9–12  | Info   |
/// | 13–16 | Warn   |
/// | 17–20 | Error  |
/// | 21–24 | Fatal  |
pub fn from_otel_int(n: u8) -> Option<Self>
```

Round-trip invariant:
```
from_otel_int(s.as_otel_int()) == Some(s)  for all s in Severity::all()
```

### `should_emit` — severity-threshold gate

```rust
/// Return true when `record_severity` meets or exceeds `min_threshold`.
///
/// Emitters call this before forwarding a log record to any exporter,
/// using the configured minimum severity as the threshold. The
/// implementation delegates to `Ord` which is already derived on
/// `Severity` (Trace < Debug < Info < Warn < Error < Fatal).
pub fn should_emit(record_severity: Severity, min_threshold: Severity) -> bool
```

Re-exported from `lib.rs`:
```rust
pub use severity::should_emit;
```

## Module layout (flat clean-arch)

```
crates/observability-domain/
  src/
    lib.rs          -- re-exports Severity, UnknownSeverityLabel,
                       should_emit; hosts data-class telemetry vocab
    severity.rs     -- Severity enum + from_otel_int + should_emit
                       + inline #[cfg(test)]
  Cargo.toml        -- no changes; no new deps
```

No new modules, no new crates.

## Testing strategy

All tests are inline `#[cfg(test)]` in `severity.rs`, following the
existing pattern in that file (see `wire_labels_round_trip`,
`otel_int_mapping_matches_spec`, etc.).

Three new test cases:

### from_otel_int bucket boundaries

Asserts the four discriminating boundary values (last of one bucket,
first of the next) plus out-of-range sentinels:

```
4  -> Some(Trace)
5  -> Some(Debug)
20 -> Some(Error)
21 -> Some(Fatal)
0  -> None
25 -> None
```

### from_otel_int round-trip

```rust
for s in Severity::all() {
    assert_eq!(Severity::from_otel_int(s.as_otel_int()), Some(s));
}
```

### should_emit threshold matrix

All 36 ordered pairs of (record_level, threshold_level). For each pair,
`should_emit(r, t)` must equal `r >= t` (by `Ord`). This is expressed
as a deterministic loop over `Severity::all()` cross product.

## Boundaries

- No root `Cargo.toml` edit.
- No new crate.
- No I/O, no network, no allocator beyond the existing crate baseline.
- No OTel SDK dependency; the integer mapping is a pure const table.
- Adjacent code (data-class telemetry, trace-context types) is
  untouched.
