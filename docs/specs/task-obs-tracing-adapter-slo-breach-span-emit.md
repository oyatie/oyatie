# Spec: obs-tracing-adapter-slo-breach-span-emit

Vertical: observability
Crate (only): observability-tracing-adapter
Status: IN PROGRESS — feat/task-obs-tracing-adapter-slo-breach-span-emit-2026-05-28

---

## Objective

Extend the `observability-tracing-adapter` crate with a concrete
SLO-breach trace-emission path. The path is layered over the domain's existing
alerting vocabulary pattern (field-name constants, value objects, observer
traits, noop defaults) introduced for `CapabilityInvocationTraceObserver`.

The new `SloBreachTraceObserver` trait and its `tracing`-backed implementation
open a stable-named span that carries:

- SLO identity and objective (name, target)
- Error-budget consumption ratio
- Burn-rate inputs (short-window, long-window)
- The resulting alert decision (page / ticket / none)

All new types and constants are defined in the adapter crate. The
`observability-domain` crate is read-only throughout this task.

---

## Vertical

`observability` — tracing adapter layer only. No changes to the domain,
usecase, app, rest, or worker layers.

---

## Mod Layout (flat clean-arch pattern, ADR-0509)

```
crates/observability-tracing-adapter/
  src/
    lib.rs          ← all code; slo_breach module added here
  Cargo.toml        ← add no new workspace member
```

The SLO-breach observer is added directly to `src/lib.rs` following the
existing pattern for `TracingCapabilityInvocationObserver`. No new modules or
files required for the impl; tests in a `#[cfg(test)]` block at the bottom of
the file (or in a `tests/` file if span capture requires it).

---

## New Types (all in adapter, never in domain)

### Field-name constants

```rust
pub mod slo_fields {
    pub const SLO_NAME: &str = "slo.name";
    pub const SLO_OBJECTIVE: &str = "slo.objective";
    pub const SLO_ERROR_BUDGET_CONSUMED: &str = "slo.error_budget.consumed";
    pub const SLO_BURN_RATE_SHORT: &str = "slo.burn_rate.short";
    pub const SLO_BURN_RATE_LONG: &str = "slo.burn_rate.long";
    pub const SLO_ALERT_DECISION: &str = "slo.alert.decision";
}
pub const SLO_BREACH_SPAN_NAME: &str = "slo.breach.evaluate";
```

General fields (`service.name`, `oyatie.tenant.id`, etc.) reuse the existing
`observability_domain::fields::*` constants — no duplication.

### Domain value objects

```rust
pub struct SloObjective {
    pub name: String,   // data_class: INTERNAL_ONLY
    pub target: f64,    // data_class: INTERNAL_ONLY
}

pub struct AlertBurnRate {
    pub short_window: f64,           // data_class: INTERNAL_ONLY
    pub long_window: f64,            // data_class: INTERNAL_ONLY
    pub error_budget_consumed: f64,  // data_class: INTERNAL_ONLY
}

pub enum AlertDecision { Page, Ticket, None }

impl AlertDecision {
    pub fn as_str(&self) -> &'static str { ... }
}
```

### Observer port

```rust
pub trait SloBreachTraceObserver: fmt::Debug + Send + Sync {
    fn observe(&self, ctx: &SloBreachTraceContext);
}
```

### Concrete impl

`TracingSloBreachTraceObserver` — opens `tracing::info_span!(SLO_BREACH_SPAN_NAME, ...)`,
records all fields from `SloBreachTraceContext` using `tracing::field::Empty`
placeholders and `span.record(...)` — mirroring the existing
`TracingCapabilityInvocationSpan::new` style.

### Noop impl

`NoopSloBreachTraceObserver` — satisfies the trait, opens no span, allocates
nothing; the clean-architecture default when telemetry is not installed.

---

## Contracts

This task introduces no HTTP, gRPC, or event-driven contracts. The observer
is a pure in-process tracing adapter.

The span fields emitted are:

| Field | Constant | Type | Cardinality |
|---|---|---|---|
| `slo.name` | `slo_fields::SLO_NAME` | `&str` | low |
| `slo.objective` | `slo_fields::SLO_OBJECTIVE` | `f64` | low |
| `slo.error_budget.consumed` | `slo_fields::SLO_ERROR_BUDGET_CONSUMED` | `f64` | low |
| `slo.burn_rate.short` | `slo_fields::SLO_BURN_RATE_SHORT` | `f64` | low |
| `slo.burn_rate.long` | `slo_fields::SLO_BURN_RATE_LONG` | `f64` | low |
| `slo.alert.decision` | `slo_fields::SLO_ALERT_DECISION` | `&str` (page/ticket/none) | low |

All fields are `INTERNAL_ONLY` data class — safe for operational telemetry.

---

## Testing Strategy

### Approach

Use scoped subscriber (no global install) with `tracing_subscriber`'s
`with_default(subscriber, || { ... })` pattern and a `TestWriter` or
in-memory collector to capture span events. Each test case:

1. Creates a `SloBreachTraceContext` with known inputs
2. Wraps the `TracingSloBreachTraceObserver::observe()` call inside
   `tracing::subscriber::with_default(...)`
3. Asserts the span name equals `SLO_BREACH_SPAN_NAME`
4. Asserts each field value matches expectations for `Page`, `Ticket`, `None`

Noop test: asserts the observer compiles and runs without panicking — no
span capture infrastructure needed.

### Test cases

| Test name | What it asserts |
|---|---|
| `slo_breach_span_name_is_stable` | span name = `"slo.breach.evaluate"` |
| `slo_breach_page_decision_records_all_fields` | all 6 fields present, decision = `"page"` |
| `slo_breach_ticket_decision_records_all_fields` | decision = `"ticket"` |
| `slo_breach_none_decision_records_all_fields` | decision = `"none"` |
| `noop_slo_breach_observer_emits_nothing` | no panic, no side effects |

---

## Boundaries

- ONLY `observability-tracing-adapter` is modified
- `observability-domain` is read-only
- Root `Cargo.toml` is untouched (no new workspace members)
- No new crates created
- No HTTP/gRPC/event contracts
- No global subscriber install in tests
- `cfg(test)` unwrap/expect exemption already declared in crate header

---

## OpenSLO

This task does not introduce or modify an OpenSLO file. The adapter crate is
an in-process observability utility, not a microservice with its own SLO target.
