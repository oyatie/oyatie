# Task plan: obs-domain-slo-burn-rate-vocabulary

Lane: observability
Crate: oya-observability-domain
Branch: feat/task-obs-domain-slo-burn-rate-vocabulary-2026-05-28

## Objective

Extend the pure observability-domain kernel with an SLO burn-rate alerting
vocabulary: a low-cardinality `SLOObjective` value object, stable SLO
telemetry field-name constants, and a pure `AlertBurnRate` classifier that
maps error-budget consumption + burn-rate pairs to Page/Ticket/None alert
decisions. No runtime deps, no new crate, no root Cargo.toml edit.

## Subtasks

### sbr-1 — `slo` module: `SLOObjective` value object + field-name constants

Files:
- `crates/oya-observability-domain/src/slo.rs` (new)
- `crates/oya-observability-domain/src/lib.rs` (add `pub mod slo; pub use slo::...`)

#### SLOObjective

```rust
/// Target SLO expressed as a ratio in the half-open interval (0, 1].
/// Rolling window is the measurement window (e.g. 30 days expressed as seconds).
pub struct SLOObjective { ... }  // data_class: INTERNAL_ONLY

impl SLOObjective {
    pub fn new(target_ratio: f64, window_secs: u64) -> Result<Self, InvalidSLOObjective>
    pub fn target_ratio(&self) -> f64
    pub fn window_secs(&self) -> u64
}

pub struct InvalidSLOObjective { pub reason: &'static str }
```

Constructor rejects `target_ratio` outside (0.0, 1.0] with `InvalidSLOObjective`.

#### Field-name constants (all `data_class: INTERNAL_ONLY`)

```rust
pub mod slo_fields {
    pub const SLO_NAME:                &str = "oyatie.slo.name";
    pub const SLO_OBJECTIVE_RATIO:     &str = "oyatie.slo.objective_ratio";
    pub const ERROR_BUDGET_REMAINING:  &str = "oyatie.slo.error_budget_remaining";
    pub const BURN_RATE:               &str = "oyatie.slo.burn_rate";
}
```

Re-exported from `lib.rs` via `pub use slo::{SLOObjective, InvalidSLOObjective, slo_fields}`.

Acceptance:
- `cargo check -p oya-observability-domain --all-targets` passes
- `SLOObjective::new(0.999, 2_592_000)` succeeds; `.target_ratio()` == 0.999
- `SLOObjective::new(0.0, 86_400)` returns `Err(InvalidSLOObjective)`
- `SLOObjective::new(1.001, 86_400)` returns `Err(InvalidSLOObjective)`
- `SLOObjective::new(1.0, 86_400)` succeeds (boundary: 1.0 is valid)
- Constants and `SLOObjective` are publicly accessible from `oya_observability_domain`
- Covered by a unit test in `slo.rs` `#[cfg(test)]` block

### sbr-2 — `AlertBurnRate` classifier

Files:
- `crates/oya-observability-domain/src/slo.rs` (extend)
- `crates/oya-observability-domain/tests/slo_burn_rate.rs` (new integration tests)

#### Alert decision enum

```rust
/// Low-cardinality alert decision produced by the multi-window burn-rate classifier.
/// data_class: INTERNAL_ONLY
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AlertDecision {
    Page,
    Ticket,
    None,
}
```

#### Classifier

```rust
/// Pure multi-window burn-rate classifier per the Google SRE multi-window method.
///
/// Inputs:
///   error_budget_consumed: fraction of error budget consumed so far (0.0..=1.0)
///   fast_burn_rate:        burn rate over the short (fast) window
///   slow_burn_rate:        burn rate over the long  (slow) window
///
/// Returns AlertDecision::Page when both burn rates exceed the page threshold
/// AND error budget consumed exceeds the page budget threshold.
/// Returns AlertDecision::Ticket when both burn rates exceed the ticket threshold
/// AND error budget consumed exceeds the ticket budget threshold.
/// Returns AlertDecision::None otherwise.
pub fn classify_burn_rate(
    error_budget_consumed: f64,
    fast_burn_rate: f64,
    slow_burn_rate: f64,
) -> AlertDecision
```

No heap allocation on the hot path. No new external dependency.

#### Threshold constants (documented against Google SRE multi-window method)

```rust
// Page tier: fast window >14x, slow window >14x, >2% budget consumed in 1h
pub const PAGE_BURN_RATE_THRESHOLD:   f64 = 14.4;
pub const PAGE_BUDGET_CONSUMED_MIN:   f64 = 0.02;

// Ticket tier: fast window >6x, slow window >6x, >5% budget consumed in 6h
pub const TICKET_BURN_RATE_THRESHOLD: f64 = 6.0;
pub const TICKET_BUDGET_CONSUMED_MIN: f64 = 0.05;
```

Acceptance:
- `cargo nextest run -p oya-observability-domain` passes all tests
- Table-driven tests cover each decision tier boundary for both window pairs:
  - page: fast>=14.4, slow>=14.4, consumed>=0.02 -> Page
  - page miss (slow < threshold): -> None
  - ticket: fast>=6.0, slow>=6.0, consumed>=0.05 -> Ticket
  - ticket miss (budget not consumed enough): -> None
  - below ticket threshold: -> None
- `classify_burn_rate` is a pure `fn` with no allocation on the hot path
- No new entry in `Cargo.toml` dependencies

## Verification commands

```
cargo check -p oya-observability-domain --all-targets
cargo nextest run -p oya-observability-domain
```

Run from worktree root:
`/tmp/oya-task-obs-domain-slo-burn-rate-vocabulary-2026-05-28`

## Boundaries

- Touch ONLY:
  - `crates/oya-observability-domain/src/slo.rs` (new)
  - `crates/oya-observability-domain/src/lib.rs` (add mod + re-exports)
  - `crates/oya-observability-domain/tests/slo_burn_rate.rs` (new)
- Do NOT edit root `Cargo.toml`
- Do NOT add any new crate
- No tracing/exporter/OTel SDK dependency
- Reuse existing `Severity` enum ordering as the severity anchor; `AlertDecision` is
  a separate low-cardinality enum specific to alert routing (Page/Ticket/None)
- All value objects annotated `data_class: INTERNAL_ONLY` per codebase convention
