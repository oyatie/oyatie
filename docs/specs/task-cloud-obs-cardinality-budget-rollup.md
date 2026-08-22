# Spec: cloud-obs-cardinality-budget-rollup

**Vertical:** observability  
**Crate:** `cloud-observability-kernel`  
**ADR references:** ADR-0130 (agentic SLO-gated promotion), ADR-0131 (per-microservice flat layout)  
**Stage:** SPEC

---

## Objective

The existing `admit_plan` function enforces a per-plan cardinality check against a declared
`CardinalityEnvelope`. It cannot detect the cardinality-explosion case where multiple plans
for the same signal each pass their envelope individually, but their combined
`estimated_combinations` exceeds the envelope limit.

This slice adds `admit_budget`, a pure aggregate rollup admission function, plus a new
`AggregateEnvelopeExceeded` error variant and four new inline tests. The change is contained
entirely within the single existing source file (`src/lib.rs`) — no new crates, no I/O,
no external dependencies.

---

## Vertical and crate ownership

```
microservices/observability/
  crates/
    cloud-observability-kernel/   <-- sole crate touched by this task
      src/lib.rs
      Cargo.toml
```

This crate is the pure domain kernel for the observability vertical. It has zero runtime
dependencies and is I/O-free by design. All adapters (OTLP, REST, gRPC) live in sibling
crates and import this kernel.

---

## Module layout (flat clean-arch, per ADR-0509)

The crate is a single `src/lib.rs` file. Per the flat clean-arch doctrine no sub-module
hierarchy is introduced for this slice. All new items (`AggregateEnvelopeExceeded`,
`admit_budget`, tests) are added inline to the existing file.

```
src/lib.rs
  SignalKind                          (existing)
  CardinalityEnvelope                 (existing)
  EmissionPlan                        (existing)
  ObservabilityError                  (existing + new AggregateEnvelopeExceeded variant)
  admit_plan()                        (existing, unchanged)
  admit_budget()                      (NEW — pure aggregate rollup)
  #[cfg(test)] mod tests              (existing + 4 new cases)
```

---

## Public API contracts

### New function

```rust
pub fn admit_budget(
    plans: &[EmissionPlan],
    envelopes: &[CardinalityEnvelope],
) -> Result<(), ObservabilityError>
```

**Semantics:**
- For every plan in `plans`, the existing `EmptyPlanId` and `NoEnvelopeForSignal` guards are
  applied. The function returns the first such error encountered.
- Plans that pass per-plan guards are grouped by `SignalKind`. The `estimated_combinations`
  values within each group are summed using `u64::saturating_add` (no overflow, no panic at
  `u64::MAX`).
- For each `(signal, aggregate)` pair, `admit_budget` looks up the corresponding
  `CardinalityEnvelope`. If `aggregate > max_unique_attribute_combinations`, it returns
  `Err(ObservabilityError::AggregateEnvelopeExceeded { signal, max, aggregate })`.
- Returns `Ok(())` only when all per-plan guards pass and no per-signal aggregate exceeds
  its envelope.

**Invariants:**
- Pure: no side effects, no allocations beyond the temporary grouping map.
- Deterministic: same inputs always produce the same output.
- No async, no I/O, no FFI.

### New error variant

```rust
// Added to ObservabilityError enum:
AggregateEnvelopeExceeded { signal: SignalKind, max: u64, aggregate: u64 }
```

`message()` arm format (stable, low-cardinality, data-class-safe):

```
"aggregate cardinality envelope exceeded: signal={signal_name} max={max} aggregate={aggregate}"
```

Where `{signal_name}` is the output of `signal.name()` — a static `&'static str`.
No plan IDs, no attribute names, no tenant data appear in the message.

---

## Data classification

All fields involved are `INTERNAL_ONLY` operational data (matching existing field annotations
in the crate). The `message()` output is suitable for operational logs and metrics labels
because it contains only static signal names and numeric thresholds.

---

## Testing strategy

All tests are inline `#[cfg(test)]` within `src/lib.rs`, consistent with the existing test
pattern in this crate.

### New test cases for `admit_budget`

| Test | Input | Expected |
|---|---|---|
| `aggregate_over_envelope_rejected` | 2 plans, same signal, individually under limit, sum over | `Err(AggregateEnvelopeExceeded)` |
| `aggregate_at_boundary_passes` | 2 plans, same signal, sum equals limit exactly | `Ok(())` |
| `aggregate_no_envelope_for_signal_rejected` | 1 plan, no envelope for that signal | `Err(NoEnvelopeForSignal)` |
| `aggregate_saturating_add_no_panic` | 2 plans with `u64::MAX`, same signal | No panic; `Err(AggregateEnvelopeExceeded)` |

### Pre-existing tests (must remain green)

`under_envelope_passes`, `over_envelope_rejected`, `at_envelope_boundary_passes`,
`no_envelope_for_signal_rejected`, `empty_plan_id_rejected`, `signal_names_distinct`.

### Verification command

```
cargo check -p cloud-observability-kernel --all-targets
cargo nextest run -p cloud-observability-kernel
```

---

## Boundaries and constraints

| Constraint | Rule |
|---|---|
| Crate scope | Only `crates/cloud-observability-kernel/src/lib.rs` |
| Root `Cargo.toml` | Never edited |
| New crates | None |
| External deps | None added |
| I/O / async | None |
| Other crates | None touched |
| Data safety | `message()` output is low-cardinality operational only |

---

## OpenSLO reference

This crate is a pure domain kernel with no runtime surface; SLO authoring is the
responsibility of the adapter crates that expose it over HTTP/gRPC. No new SLO file is
required for this kernel-only slice.
