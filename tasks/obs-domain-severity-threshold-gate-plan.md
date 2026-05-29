# Task plan: obs-domain-severity-threshold-gate

Lane: observability
Crate: oya-observability-domain
Branch: feat/task-obs-domain-severity-threshold-gate-2026-05-28

## Objective

Extend `src/severity.rs` with the missing OTel SeverityNumber inverse
(`Severity::from_otel_int`) and a pure severity-threshold emission gate
(`should_emit`). No I/O, no new crate, no root Cargo.toml edit.

## Subtasks

### sd-1 — `Severity::from_otel_int(n: u8) -> Option<Severity>`

File: `crates/oya-observability-domain/src/severity.rs`

Map the full OTel SeverityNumber range to the six buckets:

| Range | Bucket  |
|-------|---------|
| 1–4   | Trace   |
| 5–8   | Debug   |
| 9–12  | Info    |
| 13–16 | Warn    |
| 17–20 | Error   |
| 21–24 | Fatal   |
| 0, >24| None    |

Acceptance:
- Compiles with exact signature `pub fn from_otel_int(n: u8) -> Option<Self>`
- Canonical ints (1, 5, 9, 13, 17, 21) round-trip via `as_otel_int`
- Out-of-range inputs (0, 25) return `None`
- `cargo check -p oya-observability-domain --all-targets` clean

### sd-2 — `should_emit(record_severity, min_threshold) -> bool`

File: `crates/oya-observability-domain/src/severity.rs`
Re-export from: `crates/oya-observability-domain/src/lib.rs`

Pure function using the existing `Ord` derive: emit iff
`record_severity >= min_threshold`.

Signature:
```rust
pub fn should_emit(record_severity: Severity, min_threshold: Severity) -> bool
```

Re-export in lib.rs alongside existing severity re-exports:
```rust
pub use severity::should_emit;
```

Acceptance:
- `should_emit` is `pub`, visible from `lib.rs`
- `should_emit(Info, Warn)` == false
- `should_emit(Error, Warn)` == true
- `should_emit(Warn, Warn)` == true

### sd-3 — Inline `#[cfg(test)]` cases in `severity.rs`

Add to the existing `#[cfg(test)] mod tests` block in `severity.rs`:

1. **Bucket-boundary test** — `from_otel_int` at the four boundary ints
   per bucket: `4->Trace`, `5->Debug`, `20->Error`, `21->Fatal`,
   `0->None`, `25->None`.

2. **Round-trip test** — `from_otel_int(s.as_otel_int()) == Some(s)`
   for all `Severity::all()` variants.

3. **Threshold-matrix test** — `should_emit` across all 36 (6×6) level
   pairs, asserting emit iff record >= threshold using `Ord`.

Acceptance:
- `cargo nextest run -p oya-observability-domain` passes all new and
  pre-existing tests (bucket-boundary, round-trip, threshold-matrix)

## Verification commands

```
cargo check -p oya-observability-domain --all-targets
cargo nextest run -p oya-observability-domain
```

Run from worktree root:
`/tmp/oya-task-obs-domain-severity-threshold-gate-2026-05-28`

## Boundaries

- Touch ONLY `crates/oya-observability-domain/src/severity.rs` and
  `crates/oya-observability-domain/src/lib.rs`.
- Do NOT edit root `Cargo.toml`.
- Do NOT add any new crate.
- No I/O, no OTel SDK dependency, no trait bounds beyond existing `Ord`.
