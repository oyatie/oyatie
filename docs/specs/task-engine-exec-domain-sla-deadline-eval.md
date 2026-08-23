# Spec: engine-exec-domain-sla-deadline-eval

vertical: workflow
crate: workflow-engine-execution-engine-domain
task: engine-exec-domain-sla-deadline-eval
status: spec

## Objective

Extend the pure execution-engine domain evaluator
(`crates/workflow-engine-execution-engine-domain/src/lib.rs`) with deterministic
SLA-deadline reasoning for the existing `ArmSlaTimer` command path. The change:

1. Adds `SlaDeadlineClass` (OnTrack / AtRisk / Breached) — a pure value object
2. Adds `classify_sla_deadline(timer, reference_epoch_seconds)` — a pure fn consuming
   only the `SlaTimer` fields and a caller-supplied reference epoch already present on
   the request
3. Extends `ExecutionEngineDomainRequest` with `sla_reference_epoch_seconds: u64`
4. Wires the classifier into `evaluate_execution_domain` for `ArmSlaTimer`, populating
   a deterministic `sla-class:*` audit ref on `ExecutionDomainReceipt`
5. Confirms the denial path for malformed/inconsistent SLA inputs (missing `sla_timer`
   already emits `InvalidCommandShape`)

All code is pure domain — no DB, clock, network, filesystem, or randomness.

## Vertical and crate

```
vertical:  workflow
lane:      engine-exec-domain-sla-deadline-eval
crate:     workflow-engine-execution-engine-domain
path:      crates/workflow-engine-execution-engine-domain/src/lib.rs
```

## Domain model additions

### SlaDeadlineClass

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlaDeadlineClass {
    OnTrack,
    AtRisk,
    Breached,
}

impl SlaDeadlineClass {
    pub fn as_wire(self) -> &'static str { ... }
}
```

Wire strings: `"on-track"`, `"at-risk"`, `"breached"`.

Trait set matches sibling domain enums (`ExecutionDomainOrigin`,
`ExecutionDomainCommandKind`, `ExecutionDomainDenialKind`).

### classify_sla_deadline

```rust
pub fn classify_sla_deadline(timer: &SlaTimer, reference_epoch_seconds: u64) -> SlaDeadlineClass
```

Thresholds (pure integer arithmetic, no floating point):

```
window      = timer.deadline_epoch_seconds - timer.armed_at_epoch_seconds
at_risk_at  = timer.armed_at_epoch_seconds + window * 80 / 100

Breached  if reference_epoch_seconds >= timer.deadline_epoch_seconds
AtRisk    if reference_epoch_seconds >= at_risk_at
OnTrack   otherwise
```

No `std::time::SystemTime::now()`, no `std::time::Instant::now()`, no I/O.

### ExecutionEngineDomainRequest extension

Add one field:

```rust
pub sla_reference_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
```

Callers supply this alongside the `SlaTimer`. For non-SLA commands the value is
ignored by the domain; callers may pass `0`.

### Audit ref contract

When `ArmSlaTimer` is accepted, `domain_audit_refs` appends:

```
"workflow-execution-domain:sla-class:<wire-string>"
```

e.g. `"workflow-execution-domain:sla-class:on-track"`.

This ref is deterministic: same `SlaTimer` + same `sla_reference_epoch_seconds` →
same `audit_refs` vector (after `sorted_unique`).

### Denial contract for malformed SLA

`ArmSlaTimer` with `sla_timer = None` → `Denied`,
`kind = ExecutionDomainDenialKind::InvalidCommandShape`.

This is already enforced by `has_invalid_command_shape`; no new denial kind needed.

## Mod layout (flat clean-arch, single file)

The crate is a single `src/lib.rs` (flat clean-arch per ADR-0509). All additions land
in that file, grouped with their conceptual siblings:

```
// -- value objects --
pub enum SlaDeadlineClass { ... }
impl SlaDeadlineClass { pub fn as_wire ... }
pub fn classify_sla_deadline(...) -> SlaDeadlineClass { ... }

// -- existing structs/enums unchanged --
// ExecutionEngineDomainRequest  (+ sla_reference_epoch_seconds field)
// ExecutionDomainReceipt        (sla_timer_armed already present)
// ...

// -- evaluate_execution_domain --
// domain_audit_refs extended to call classify_sla_deadline for ArmSlaTimer
```

No new modules, no new files, no new crates.

## Contracts

### Internal domain contract (no external API surface)

This crate is a pure domain library; it has no HTTP, gRPC, or queue surface.
The contract is the public Rust API:

```rust
// New public items
pub enum SlaDeadlineClass { OnTrack, AtRisk, Breached }
impl SlaDeadlineClass { pub fn as_wire(self) -> &'static str }
pub fn classify_sla_deadline(timer: &SlaTimer, reference_epoch_seconds: u64) -> SlaDeadlineClass

// Modified request struct
pub struct ExecutionEngineDomainRequest {
    // ... existing fields ...
    pub sla_reference_epoch_seconds: u64,  // NEW
}
```

No OpenAPI or proto surface — the domain crate has none. The usecase/adapter layer
above this crate will carry the wire contract.

## Testing strategy

All tests in `#[cfg(test)] mod tests` inside `src/lib.rs`.

| Test | Assertion |
|------|-----------|
| `sla_deadline_class_on_track` | reference << at_risk threshold → `OnTrack` |
| `sla_deadline_class_at_risk` | reference past 80% of window → `AtRisk` |
| `sla_deadline_class_breached` | reference >= deadline → `Breached` |
| `arm_sla_timer_accepts_with_classification_audit_ref` | full round-trip; `sla_timer_armed=true`; `audit_refs` contains `sla-class:*` |
| `arm_sla_timer_missing_sla_timer_denies` | `sla_timer=None` → `Denied`, `kind=InvalidCommandShape` |
| `sla_deadline_class_deterministic` | two identical evaluations → identical `audit_refs` |

Determinism gate: `receipt1.audit_refs == receipt2.audit_refs` asserted as a
byte-stable equality check.

## Boundaries

- Single file: `crates/workflow-engine-execution-engine-domain/src/lib.rs`
- No root `Cargo.toml` changes
- No kernel crate changes
- No other crate changes
- No new workspace members
- Pure domain: no DB, clock, network, filesystem, or randomness
- Denial uses existing `ExecutionDomainDenialKind::InvalidCommandShape` (no new kind)

## Dependencies

No new Cargo dependencies. The crate already depends on
`workflow-engine-execution-engine-kernel` which provides `SlaTimer`.
