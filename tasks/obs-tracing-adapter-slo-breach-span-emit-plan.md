# Plan: obs-tracing-adapter-slo-breach-span-emit

Vertical: observability
Crate: oya-observability-tracing-adapter (ONLY crate this task may touch)
Branch: feat/task-obs-tracing-adapter-slo-breach-span-emit-2026-05-28

## Objective

Extend the tracing-subscriber adapter with a concrete SLO-breach trace-emission
path. Add a `SloBreachTraceObserver` (and a `NoopSloBreachTraceObserver`) that
opens a stable-named `tracing` span carrying SLO, error-budget, burn-rate, and
alert-decision fields. Add unit tests that capture spans without a global
subscriber install and assert all required fields for page/ticket/none cases.

Domain crate (`oya-observability-domain`) is read-only — no edits allowed.
All new types, field-name constants, and the span-name constant live in the
adapter crate, reusing existing `oya_observability_domain::fields::*` constants
for the general fields already defined there.

---

## Subtasks

### [obs-tracing-adapter-slo-breach-span-emit-1] SloBreachTraceObserver

**What**: Add to `oya-observability-tracing-adapter/src/lib.rs`:

- Field-name constants for the SLO-breach span:
  - `SLO_SPAN_NAME` — stable span name `"slo.breach.evaluate"`
  - `fields::SLO_NAME` — `"slo.name"`
  - `fields::SLO_OBJECTIVE` — `"slo.objective"`
  - `fields::SLO_ERROR_BUDGET_CONSUMED` — `"slo.error_budget.consumed"`
  - `fields::SLO_BURN_RATE_SHORT` — `"slo.burn_rate.short"`
  - `fields::SLO_BURN_RATE_LONG` — `"slo.burn_rate.long"`
  - `fields::SLO_ALERT_DECISION` — `"slo.alert.decision"`
- Domain types (all in the adapter, no domain edits):
  - `SloObjective` — `{ name: String, target: f64 }` — low-cardinality input
  - `AlertBurnRate` — `{ short_window: f64, long_window: f64, error_budget_consumed: f64 }`
  - `AlertDecision` — `enum { Page, Ticket, None }` with `as_str()`
- `SloBreachTraceContext` — combines the above into a single input value object
- Trait `SloBreachTraceObserver`: `observe(&self, ctx: &SloBreachTraceContext)`
- `TracingSloBreachTraceObserver` — opens an `info_span!` with `SLO_SPAN_NAME`
  recording all fields from `SloBreachTraceContext`; mirrors the
  `TracingCapabilityInvocationSpan::new` style (Empty initial fields, then
  `span.record(...)`)
- `NoopSloBreachTraceObserver` — no-op default; opens nothing

**Acceptance**:
- `cargo check -p oya-observability-tracing-adapter --all-targets` passes
- No field-name string literals duplicated; all names defined as constants
- No edits to `oya-observability-domain`

---

### [obs-tracing-adapter-slo-breach-span-emit-2] Unit tests

**What**: Add a `#[cfg(test)]` test module to `src/lib.rs` (or a separate
`tests/` integration test file) using a scoped `tracing_subscriber` install
(not the global) to capture emitted spans. Assert:

1. Span name equals `SLO_SPAN_NAME`
2. Recorded fields: `slo.name`, `slo.objective`, `slo.error_budget.consumed`,
   `slo.burn_rate.short`, `slo.burn_rate.long`
3. Alert-decision field for `Page`, `Ticket`, `None` cases
4. `NoopSloBreachTraceObserver::observe` emits no span events

Test infrastructure: use `tracing_subscriber::fmt::TestWriter` with
`with_test_writer()` plus a local `tracing::subscriber::with_default(...)` scope
so no global install is needed and tests are independent.

**Acceptance**:
- `cargo nextest run -p oya-observability-tracing-adapter` passes
- Tests run under `cfg(test)` unwrap/expect exemption (already declared in
  the crate's `#![cfg_attr(test, allow(...))]` header)
- No new workspace member; root `Cargo.toml` untouched

---

## Acceptance Summary

| Check | Command |
|---|---|
| Build clean | `cargo check -p oya-observability-tracing-adapter --all-targets` |
| Tests green | `cargo nextest run -p oya-observability-tracing-adapter` |
| No domain edits | `git diff -- crates/oya-observability-domain` → empty |
| No root Cargo.toml edits | `git diff -- Cargo.toml` → empty |

---

## Constraints

- ONLY crate: `oya-observability-tracing-adapter`
- NEVER edit root `Cargo.toml`
- NEVER edit `oya-observability-domain`
- All new field constants defined once, no string-literal duplication
- No global subscriber install in tests
- Match existing codebase patterns: `cfg_attr(test, allow(...))`, `tracing::field::Empty`,
  `span.record(...)`, `#[derive(Clone, Debug)]`
