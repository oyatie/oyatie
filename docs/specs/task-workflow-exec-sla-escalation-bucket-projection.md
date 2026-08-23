# Spec: workflow-exec-sla-escalation-bucket-projection

## Summary

Extend the pure SLA value-object surface in `workflow-engine-execution-engine-domain`
with a deterministic escalation-bucket projection: `SlaEscalationLevel` enum and
`project_sla_escalation` pure function.

## Crate

`workflow-engine-execution-engine-domain` — no new workspace member, no new dependency.

## Types

### `SlaEscalationLevel`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SlaEscalationLevel {
    None,
    Notify,
    Page,
    AutoAbort,
}
```

Wire values (via `as_wire() -> &'static str`):

| Variant     | Wire string  |
|-------------|--------------|
| `None`      | `"none"`     |
| `Notify`    | `"notify"`   |
| `Page`      | `"page"`     |
| `AutoAbort` | `"auto-abort"` |

### `project_sla_escalation`

```
fn project_sla_escalation(
    timer: &SlaTimer,
    reference_epoch_seconds: u64,
    breach_grace_seconds: u64,
) -> SlaEscalationLevel
```

Pure function — no wall-clock, no I/O, no randomness.

## Logic

Let:
- `armed_at = timer.armed_at_epoch_seconds`
- `deadline = timer.deadline_epoch_seconds`
- `window = deadline - armed_at`  (saturating subtraction, minimum 0)
- `at_risk_at = armed_at + window * 80 / 100`
- `grace_end = deadline.saturating_add(breach_grace_seconds)`

Decision table (evaluated top-to-bottom):

| Condition                              | Result      |
|----------------------------------------|-------------|
| `reference >= grace_end`               | `AutoAbort` |
| `reference >= deadline`                | `Page`      |
| `reference >= at_risk_at`              | `Notify`    |
| otherwise                              | `None`      |

Arithmetic uses **saturating** operations throughout to guard u64 overflow.

## Acceptance Criteria

1. `project_sla_escalation` returns `None` when reference is well before 80% threshold.
2. Returns `Notify` when reference is exactly at the 80% threshold.
3. Returns `Notify` when reference is between 80% threshold and deadline.
4. Returns `Page` when reference is exactly at deadline (and within grace).
5. Returns `Page` when reference is within the grace window past the deadline.
6. Returns `AutoAbort` when reference equals `deadline + breach_grace_seconds`.
7. Returns `AutoAbort` when reference exceeds `deadline + breach_grace_seconds`.
8. Monotonic: escalation level never decreases as reference increases.
9. Zero-width window (armed_at == deadline) with grace: Page/AutoAbort boundaries respected.
10. Saturating arithmetic: `breach_grace_seconds = u64::MAX` does not overflow.
11. `as_wire()` returns the correct wire string for each variant.
12. `SlaEscalationLevel` implements `Ord` — `None < Notify < Page < AutoAbort`.
13. Cargo nextest passes; no new workspace member; no new dependency.
