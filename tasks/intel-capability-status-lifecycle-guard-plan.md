# Task Plan: intel-capability-status-lifecycle-guard

**Crate:** `oya-intelligence-capability-registry-kernel`
**Vertical:** intelligence
**Branch:** `feat/task-intel-capability-status-lifecycle-guard-2026-05-28`

## Subtasks

### ST1 — Transition guard

Add `CapabilityStatus::try_transition_to(&self, next: CapabilityStatus) -> Result<CapabilityStatus, CapabilityStatusTransitionError>` and the `CapabilityStatusTransitionError` error type to `src/status.rs`.

Legal edges:
- `Active -> Deprecated` (soft deprecation)
- `Active -> Disabled` (administrative suspend)
- `Deprecated -> Active` (rescind deprecation)
- `Deprecated -> Disabled` (escalate to suspend)
- `Disabled -> Active` (re-activation)

Illegal edges:
- `Disabled -> Deprecated` (undefined; must go through Active)
- Any same-state transition (`Active -> Active`, etc.)

**Error type** must implement `std::fmt::Display` and `std::error::Error`.
Rustdoc on the method must enumerate the transition matrix.
No new Cargo.toml dependencies.

**Accept:** `cargo check -p oya-intelligence-capability-registry-kernel --all-targets` passes; new error type implements Display; rustdoc enumerates the legal transition matrix.

---

### ST2 — Registry-view helper

Add `registry_view` module (`src/registry_view.rs`) with a pure function:

```rust
pub fn partition_views(
    entries: impl IntoIterator<Item = (CapabilityId, CapabilityStatus)>,
) -> RegistryViews
```

`RegistryViews` holds:
- `discoverable: BTreeMap<CapabilityId, CapabilityStatus>` — entries where `status.is_discoverable()` is true
- `invocable: BTreeMap<CapabilityId, CapabilityStatus>` — entries where `status.is_invocable()` is true

Ordering is deterministic via `BTreeMap` (lexicographic on `CapabilityId` which derives `Ord`).

Re-export from `lib.rs`: `pub mod registry_view; pub use registry_view::{RegistryViews, partition_views};`

**Accept:** `cargo nextest run -p oya-intelligence-capability-registry-kernel` green; tests assert Active is in both views, Deprecated only in invocable, Disabled in neither, ordering is stable.

---

### ST3 — Transition matrix coverage tests

Add a `#[cfg(test)] mod transition_tests` block in `src/status.rs` covering:

- `Active -> Deprecated` → `Ok(Deprecated)`
- `Deprecated -> Active` → `Ok(Active)`
- `Active -> Disabled` → `Ok(Disabled)`
- `Deprecated -> Disabled` → `Ok(Disabled)`
- `Disabled -> Active` → `Ok(Active)`
- `Disabled -> Deprecated` → `Err`
- `Active -> Active` (same-state) → `Err`
- `Deprecated -> Deprecated` (same-state) → `Err`
- `Disabled -> Disabled` (same-state) → `Err`
- Existing `as_str` / `TryFrom` round-trip tests remain green.

**Accept:** All transition tests green; existing status.rs predicate tests remain green; `cargo check --all-targets` + `cargo nextest run` both pass.

## Acceptance summary

| Check | Command |
|---|---|
| Compile (all targets) | `cargo check -p oya-intelligence-capability-registry-kernel --all-targets` |
| Tests | `cargo nextest run -p oya-intelligence-capability-registry-kernel` |
| No new deps | `Cargo.toml` unchanged |
