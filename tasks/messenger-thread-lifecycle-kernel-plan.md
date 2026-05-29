# Task Plan: messenger-thread-lifecycle-kernel

**Vertical**: messenger
**Crate**: `oya-messenger-domain` (`crates/oya-messenger-domain`)
**Branch**: `feat/task-messenger-thread-lifecycle-kernel-2026-05-28`
**Stage created**: SPEC
**Date**: 2026-05-29

---

## Objective

Extend the `oya-messenger-domain` crate with a thread-lifecycle state machine:
typed thread states (`Open`, `Locked`, `Resolved`, `Archived`) with validated
immutable-where-required transitions, and per-participant follow/mute
subscription invariants enforcing pillar-isolation. Pure domain types and
validation only — no I/O, no adapter, no async runtime.

Builds on the existing thread-shape validation (`validate_thread_shape`) and the
`ChatError`/`PresenceState` patterns in `lib.rs` and `governance.rs`. No new
workspace members; root `Cargo.toml` is untouched.

---

## Subtasks

### [messenger-thread-lifecycle-kernel-1] ThreadState enum + ThreadLifecycle struct

**Scope**: Create `src/thread_lifecycle.rs` with:

- `ThreadState` enum: `Open`, `Locked`, `Resolved`, `Archived`
  (`Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd`)
- `ThreadLifecycle` struct holding `Classified<String>` `thread_id` +
  `Classified<String>` `tenant_id` + `Classified<ThreadState>` `state`
- `ThreadLifecycle::new(thread_id, tenant_id, initial_state)` constructor
  returning `Result<_, ChatError>` that:
  - rejects empty / whitespace-only `thread_id` → `ChatError::InvalidThreadId`
  - rejects empty / whitespace-only `tenant_id` → `ChatError::InvalidTenantId`
  - tags `thread_id` / `tenant_id` `INTERNAL_ONLY` via `internal()` helper
  - tags `state` `INTERNAL_ONLY`
- Data-class helpers consistent with sibling modules
- Module-level `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]`
- Declare `pub mod thread_lifecycle;` + `pub use thread_lifecycle::*;` in `lib.rs`

**Acceptance**:
- `cargo check -p oya-messenger-domain --all-targets` passes
- `ThreadState` + `ThreadLifecycle` are `pub` and re-exported from `lib.rs`
- Invalid ids return typed `ChatError` (no panics outside `cfg(test)`)
- No new workspace member; root `Cargo.toml` unchanged

---

### [messenger-thread-lifecycle-kernel-2] Transition method + unit tests

**Scope**: Add `ThreadLifecycle::transition(&self, next: ThreadState) -> Result<Self, ChatError>` enforcing:

| From \ To  | Open | Locked | Resolved | Archived |
|-----------|------|--------|----------|----------|
| Open      | ERR  | OK     | OK       | OK       |
| Locked    | OK   | ERR    | OK       | OK       |
| Resolved  | OK   | ERR    | ERR      | OK       |
| Archived  | ERR  | ERR    | ERR      | ERR (terminal) |

- Illegal same-state or invalid transitions return `ChatError::InvalidThreadStateTransition`
  (new variant added to `ChatError`)
- `Archived` is terminal — all transitions from it are rejected
- Transition returns a new `ThreadLifecycle` with the updated `state`

Unit tests (all in `src/thread_lifecycle.rs` under `#[cfg(test)]`):

| Test | Asserts |
|------|---------|
| `open_to_locked_legal` | `Open → Locked` succeeds |
| `open_to_resolved_legal` | `Open → Resolved` succeeds |
| `open_to_archived_legal` | `Open → Archived` succeeds |
| `locked_to_open_legal` | `Locked → Open` succeeds |
| `locked_to_resolved_legal` | `Locked → Resolved` succeeds |
| `locked_to_archived_legal` | `Locked → Archived` succeeds |
| `resolved_to_open_legal` | `Resolved → Open` succeeds |
| `resolved_to_archived_legal` | `Resolved → Archived` succeeds |
| `archived_terminal_open` | `Archived → Open` → `InvalidThreadStateTransition` |
| `archived_terminal_locked` | `Archived → Locked` → `InvalidThreadStateTransition` |
| `same_state_open_rejected` | `Open → Open` → `InvalidThreadStateTransition` |
| `resolved_to_locked_rejected` | `Resolved → Locked` → `InvalidThreadStateTransition` |

**Acceptance**:
- `cargo nextest run -p oya-messenger-domain` green with new tests
- At least one legal transition per source state and at least two rejected
  illegal transitions (including Archived-terminal) asserted

---

### [messenger-thread-lifecycle-kernel-3] ThreadSubscription + pillar-isolation invariants

**Scope**: Add `ThreadSubscription` struct + `follow()` / `mute()` methods
enforcing the same pillar-isolation pattern as `PresenceState::CrossPillarPresenceDenied`
in `governance.rs`:

- `ThreadSubscriptionCreate` plain input struct with `thread_id`, `tenant_id`,
  `participant_ref` (PII_IDENTIFYING), `participant_pillar` (`OwnershipPillar`),
  `thread_pillar` (`OwnershipPillar`)
- `ThreadSubscription` canonical struct with all fields `Classified`
- `ThreadSubscription::new(..) -> Result<_, ChatError>`:
  - rejects empty `thread_id` / `tenant_id` / `participant_ref`
  - rejects pillar mismatch → `ChatError::CrossPillarSubscriptionDenied` (new variant)
- `ThreadSubscription::follow(&self) -> Result<ThreadSubscriptionMode, ChatError>`
  sets `mode` to `Follow` (no-op if already Follow, returns current state)
- `ThreadSubscription::mute(&self) -> Result<ThreadSubscriptionMode, ChatError>`
  sets `mode` to `Mute`
- `ThreadSubscriptionMode` enum: `Follow`, `Mute`

Unit tests:

| Test | Asserts |
|------|---------|
| `same_pillar_work_follow_succeeds` | Work participant on Work thread follows OK |
| `same_pillar_personal_mute_succeeds` | Personal participant on Personal thread mutes OK |
| `cross_pillar_work_on_personal_denied` | Work participant on Personal thread → `CrossPillarSubscriptionDenied` |
| `cross_pillar_personal_on_work_denied` | Personal participant on Work thread → `CrossPillarSubscriptionDenied` |
| `follow_mute_round_trip` | follow → mute → follow round-trips correctly |
| `invalid_thread_id_rejected` | Empty thread_id → `ChatError::InvalidThreadId` |

**Acceptance**:
- `cargo nextest run -p oya-messenger-domain` green
- Cross-pillar subscription denied with `CrossPillarSubscriptionDenied`
- Same-pillar follow/mute round-trip succeeds
- `cargo check -p oya-messenger-domain --all-targets` passes

---

## Acceptance Summary

| Subtask | Gate |
|---------|------|
| 1 | `cargo check -p oya-messenger-domain --all-targets` green; `ThreadState` + `ThreadLifecycle` pub + re-exported; invalid ids → typed `ChatError`; root `Cargo.toml` unchanged |
| 2 | `cargo nextest run -p oya-messenger-domain` green; legal transitions per source state + two illegal rejections (including Archived-terminal) |
| 3 | `cargo nextest run -p oya-messenger-domain` green; cross-pillar denied + same-pillar follow/mute round-trip proven; check green |
