# Plan: workflow-exec-sla-escalation-bucket-projection

## Objective

Extend the pure SLA value-object surface in `oya-workflow-engine-execution-engine-domain`
with a deterministic escalation-bucket projection function.

## Scope

Crate: `oya-workflow-engine-execution-engine-domain` (flat clean-arch mod in `src/lib.rs`)
No new crate, no new dependency, no workspace change.

## Acceptance

- `SlaEscalationLevel` enum with variants `None/Notify/Page/AutoAbort` + `as_wire()`
- `project_sla_escalation(timer, reference_epoch_seconds, breach_grace_seconds) -> SlaEscalationLevel`
  - `None` when `OnTrack` (reference < armed_at + window*80/100)
  - `Notify` at AtRisk threshold (reference >= armed_at + window*80/100 and < deadline)
  - `Page` at/after deadline and within grace (reference >= deadline and < deadline + breach_grace_seconds)
  - `AutoAbort` once reference >= deadline + breach_grace_seconds
- Monotonic: escalation level is non-decreasing as reference increases
- Saturating arithmetic to guard overflow
- >= 8 hermetic unit tests: boundary epochs, zero-width window, overflow guard

## Steps

1. Add `SlaEscalationLevel` enum + `as_wire()` to `src/lib.rs`
2. Add `project_sla_escalation` function to `src/lib.rs`
3. Add >= 8 cfg(test) unit tests in `src/lib.rs`
4. `cargo check -p oya-workflow-engine-execution-engine-domain --all-targets`
5. `cargo nextest run -p oya-workflow-engine-execution-engine-domain`
