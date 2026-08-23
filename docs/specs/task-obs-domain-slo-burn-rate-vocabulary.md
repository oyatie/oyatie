# Spec: obs-domain-slo-burn-rate-vocabulary

## Objective

Add an SLO burn-rate alerting vocabulary to the pure `observability-domain`
kernel crate. The vocabulary consists of:

1. An `SLOObjective` value object capturing a target reliability ratio and
   rolling measurement window.
2. Stable telemetry field-name constants (`slo_fields`) for SLO/error-budget
   labels used in spans, log records, and metrics dimensions.
3. A pure `AlertBurnRate` classifier (`classify_burn_rate`) that maps a trio of
   (error-budget fraction consumed, fast-window burn rate, slow-window burn rate)
   to a low-cardinality `AlertDecision` (Page / Ticket / None) using const
   threshold tables documented against the Google SRE multi-window method.

All additions are pure data and classification logic — no tracing subscriber, no
exporter, no OTel SDK dependency, no heap allocation on the classifier hot path.

## Vertical

Lane: `observability`
Crate (sole target): `observability-domain`
(`crates/observability-domain/`)

## Module layout (flat clean-arch, mod-based)

```
src/
  lib.rs          -- add: pub mod slo; pub use slo::{...}
  severity.rs     -- unchanged
  slo.rs          -- NEW: SLOObjective, InvalidSLOObjective, slo_fields, AlertDecision,
                         PAGE_* / TICKET_* consts, classify_burn_rate
tests/
  severity_threshold_gate.rs  -- unchanged
  slo_burn_rate.rs            -- NEW: integration-level acceptance tests
```

No new files outside this boundary. Root `Cargo.toml` untouched.

## Data contracts

### No external API surface

This crate is a pure domain kernel. It exposes no HTTP/gRPC/proto surface.
Field-name constants are consumed by runtime adapter crates that construct OTel
spans and metrics; they are not part of an OpenAPI schema.

### Field-name constants (slo_fields)

| Constant               | Wire value                           | Data class    |
|------------------------|--------------------------------------|---------------|
| `SLO_NAME`             | `oyatie.slo.name`                    | INTERNAL_ONLY |
| `SLO_OBJECTIVE_RATIO`  | `oyatie.slo.objective_ratio`         | INTERNAL_ONLY |
| `ERROR_BUDGET_REMAINING` | `oyatie.slo.error_budget_remaining` | INTERNAL_ONLY |
| `BURN_RATE`            | `oyatie.slo.burn_rate`               | INTERNAL_ONLY |

These are stable wire names that downstream exporters and dashboards depend on.
They must not change without a deprecation cycle.

### SLOObjective value object

```
SLOObjective {
    target_ratio: f64,   // (0.0, 1.0] — data_class: INTERNAL_ONLY
    window_secs:  u64,   // rolling measurement window — data_class: INTERNAL_ONLY
}
```

Constructor: `SLOObjective::new(target_ratio: f64, window_secs: u64) -> Result<Self, InvalidSLOObjective>`

Invariant: `target_ratio` must be in (0.0, 1.0]. Values <= 0.0 or > 1.0 are
rejected. `window_secs == 0` is allowed (degenerate but not this module's concern).

### AlertDecision enum

```
enum AlertDecision { Page, Ticket, None }
```

Low-cardinality alert routing signal. `Page` triggers immediate on-call
escalation. `Ticket` opens a next-business-day work item. `None` is the
quiescent state.

### classify_burn_rate signature

```rust
pub fn classify_burn_rate(
    error_budget_consumed: f64,   // fraction consumed, 0.0..=1.0
    fast_burn_rate: f64,          // burn rate over fast (short) window
    slow_burn_rate: f64,          // burn rate over slow (long) window
) -> AlertDecision
```

## Threshold tables (Google SRE multi-window method)

The multi-window burn-rate method requires **both** the fast and slow windows to
exceed a burn-rate threshold **and** the error budget to have been consumed above
a minimum fraction before alerting. Using both windows reduces false-positive
pages from transient spikes.

### Page tier (fast ~1 h, slow ~6 h for a 30-day window)

| Parameter                  | Value  | Derivation                                  |
|----------------------------|--------|---------------------------------------------|
| `PAGE_BURN_RATE_THRESHOLD` | 14.4   | Consumes 2% of 30-day budget in ~1 h        |
| `PAGE_BUDGET_CONSUMED_MIN` | 0.02   | At least 2% budget consumed before paging   |

Condition: `fast_burn_rate >= 14.4 AND slow_burn_rate >= 14.4 AND error_budget_consumed >= 0.02`

### Ticket tier (fast ~6 h, slow ~3 days for a 30-day window)

| Parameter                    | Value  | Derivation                                  |
|------------------------------|--------|---------------------------------------------|
| `TICKET_BURN_RATE_THRESHOLD` | 6.0    | Consumes 5% of 30-day budget in ~6 h        |
| `TICKET_BUDGET_CONSUMED_MIN` | 0.05   | At least 5% budget consumed before ticketing|

Condition: `fast_burn_rate >= 6.0 AND slow_burn_rate >= 6.0 AND error_budget_consumed >= 0.05`

Decision priority: Page > Ticket > None (checked in that order).

Reference: Google SRE Workbook, Chapter 5 "Alerting on SLOs",
"Multiwindow, Multi-Burn-Rate Alerts".

## Testing strategy

### Inline unit tests (`#[cfg(test)]` in `src/slo.rs`)

- `SLOObjective::new` accepts valid ratios (0.001, 0.999, 1.0)
- `SLOObjective::new` rejects 0.0 and values > 1.0
- Accessors return stored values without mutation

### Integration tests (`tests/slo_burn_rate.rs`)

Table-driven tests covering every `AlertDecision` tier boundary:

| Test name                                      | Scenario                                            | Expected   |
|------------------------------------------------|-----------------------------------------------------|------------|
| `classify_both_windows_above_page_threshold`   | fast=15, slow=15, consumed=0.03                     | Page       |
| `classify_fast_below_page_slow_above`          | fast=10, slow=15, consumed=0.03                     | None       |
| `classify_fast_above_page_slow_below`          | fast=15, slow=10, consumed=0.03                     | None       |
| `classify_page_budget_not_consumed_enough`     | fast=15, slow=15, consumed=0.01 (< 0.02)            | None       |
| `classify_both_windows_above_ticket_threshold` | fast=7, slow=7, consumed=0.06                       | Ticket     |
| `classify_ticket_budget_not_consumed_enough`   | fast=7, slow=7, consumed=0.04 (< 0.05)              | None       |
| `classify_below_all_thresholds`                | fast=1, slow=1, consumed=0.50                       | None       |
| `classify_page_wins_over_ticket`               | fast=15, slow=15, consumed=0.10 (exceeds both mins) | Page       |
| `classify_exact_page_boundary`                 | fast=14.4, slow=14.4, consumed=0.02                 | Page       |
| `classify_just_below_page_boundary`            | fast=14.39, slow=14.4, consumed=0.02                | depends*   |

*Just-below-boundary test asserts Ticket (if ticket conditions met) or None,
demonstrating that the page threshold is strict >= and the classifier falls
through correctly.

All tests use `observability_domain` public re-exports only; no `src/`
module paths.

## Boundaries

- One crate only: `observability-domain`
- New files: `src/slo.rs`, `tests/slo_burn_rate.rs`
- Modified: `src/lib.rs` (add `pub mod slo` + re-exports)
- Untouched: root `Cargo.toml`, `src/severity.rs`, existing tests
- Zero new external dependencies
- No `std::alloc` calls on the classifier hot path (`classify_burn_rate` uses
  only `f64` comparisons and a match expression)
- All public items annotated with `data_class: INTERNAL_ONLY` doc comments
  consistent with the existing codebase convention

## OpenSLO reference

The field-name constants defined here are intended to be consumed by the
observability vertical's OpenSLO evaluation pipeline
(`microservices/observability/capabilities/slo-evaluate.yaml`). The
`SLO_OBJECTIVE_RATIO` and `ERROR_BUDGET_REMAINING` fields map directly to
OpenSLO `SLO.spec.objectives[].target` and computed budget metrics.
