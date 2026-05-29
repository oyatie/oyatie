# Task plan: engine-exec-domain-sla-deadline-eval

vertical: workflow
crate: oya-workflow-engine-execution-engine-domain
branch: feat/task-engine-exec-domain-sla-deadline-eval-2026-05-28
base: origin/dev

## Objective

Extend the pure execution-engine domain evaluator with deterministic SLA-deadline
reasoning for the `ArmSlaTimer` command path. Add a deadline-classification value
object, wire it into `evaluate_execution_domain`, surface classification + audit_refs
on `ExecutionDomainReceipt`, and add a denial path for malformed/inconsistent SLA
inputs. No DB, clock, network, or randomness — all deadline inputs are caller-supplied
epoch refs already on `ExecutionEngineDomainRequest`.

## Subtasks

### [engine-exec-domain-sla-deadline-eval-1] SlaDeadlineClass value object + classify helper

**Scope:** `crates/oya-workflow-engine-execution-engine-domain/src/lib.rs`

Add:
- `pub enum SlaDeadlineClass { OnTrack, AtRisk, Breached }` with
  `Clone + Copy + Debug + Eq + PartialEq` and `as_wire() -> &'static str`
- `pub fn classify_sla_deadline(timer: &SlaTimer, reference_epoch_seconds: u64) -> SlaDeadlineClass`
  — pure fn, no wall-clock/random/I/O; thresholds:
  - `reference_epoch_seconds >= timer.deadline_epoch_seconds` → `Breached`
  - `reference_epoch_seconds >= at_risk_threshold(timer)` → `AtRisk`
    (at-risk threshold = `armed_at + (deadline - armed_at) * 80 / 100`, integer arithmetic)
  - else → `OnTrack`

**Acceptance:**
- `cargo check -p oya-workflow-engine-execution-engine-domain --all-targets` passes
- `classify_sla_deadline` is a pure fn (no std::time::Instant, no SystemTime::now, no I/O)
- `SlaDeadlineClass` derives `Clone + Copy + Debug + Eq + PartialEq` and exposes `as_wire()`

---

### [engine-exec-domain-sla-deadline-eval-2] Wire classifier into evaluate_execution_domain

**Scope:** `crates/oya-workflow-engine-execution-engine-domain/src/lib.rs`

Changes to `evaluate_execution_domain` for `ExecutionDomainCommandKind::ArmSlaTimer`:

1. Denial path — emit `ExecutionDomainDenial` with kind `InvalidCommandShape` when
   `sla_timer` is `None` (already handled by `has_invalid_command_shape`; confirm it
   covers all malformed cases including inconsistent scope via `has_scope_mismatch`).
2. Receipt path — when `ArmSlaTimer` is accepted:
   - Call `classify_sla_deadline(&timer, request.sla_reference_epoch_seconds)`
   - Set `sla_timer_armed = true` (already set)
   - Append a deterministic `audit_ref`:
     `format!("workflow-execution-domain:sla-class:{}", class.as_wire())`
     into `domain_audit_refs` result before `sorted_unique`

Add `sla_reference_epoch_seconds: u64` to `ExecutionEngineDomainRequest` (caller-supplied,
no wall-clock). Wire through `domain_audit_refs` to inject the classification ref.

**Acceptance:**
- `ArmSlaTimer` with valid `SlaTimer` + any `sla_reference_epoch_seconds` → `Accepted`,
  `sla_timer_armed = true`, `audit_refs` contains the `sla-class:*` ref
- `ArmSlaTimer` with `sla_timer = None` → `Denied` with `InvalidCommandShape`
- All other command paths and existing tests pass unchanged

---

### [engine-exec-domain-sla-deadline-eval-3] Unit tests

**Scope:** `#[cfg(test)] mod tests` in `src/lib.rs`

New test cases:
- `sla_deadline_class_on_track` — reference well before at-risk threshold → `OnTrack`
- `sla_deadline_class_at_risk` — reference past 80% of window → `AtRisk`
- `sla_deadline_class_breached` — reference at or past deadline → `Breached`
- `arm_sla_timer_accepts_with_classification_audit_ref` — full evaluate round-trip;
  asserts `sla_timer_armed = true` and `audit_refs` contains `sla-class:on-track` (or
  matching class)
- `arm_sla_timer_missing_sla_timer_denies` — `sla_timer = None` on `ArmSlaTimer` →
  `Denied`, `kind = InvalidCommandShape`
- `sla_deadline_class_deterministic` — call `evaluate_execution_domain` twice on
  identical input; assert `receipt1.audit_refs == receipt2.audit_refs` (byte-stable)

**Acceptance:**
- `cargo nextest run -p oya-workflow-engine-execution-engine-domain` is green
- At least one test asserts byte-stable `audit_refs` across two evaluations of identical
  input

## Acceptance summary

| Subtask | Gate |
|---------|------|
| 1 | `cargo check -p oya-workflow-engine-execution-engine-domain --all-targets` passes |
| 2 | ArmSlaTimer accepted → `sla_timer_armed=true` + `sla-class:*` audit ref; missing SlaTimer → Denied |
| 3 | `cargo nextest run -p oya-workflow-engine-execution-engine-domain` green; determinism test present |

## Boundaries

- ONLY touches `crates/oya-workflow-engine-execution-engine-domain/src/lib.rs`
- NEVER modifies `Cargo.toml` at root or workspace
- NEVER modifies the kernel crate or any other crate
- No new crates, no new modules (flat single-file domain)
- No I/O, clock, DB, or network in any new code
