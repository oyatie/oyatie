# Spec: messenger-thread-lifecycle-kernel

**Vertical**: messenger
**Task slug**: `messenger-thread-lifecycle-kernel`
**Crate**: `messenger-domain` (`crates/messenger-domain`)
**Modules**: `src/thread_lifecycle.rs` (new file, re-exported from `lib.rs`)
**Stage created**: SPEC
**Date**: 2026-05-29

---

## Objective

Extend the `messenger-domain` crate with a thread-lifecycle state machine.
The kernel owns:

1. A typed `ThreadState` enum (`Open`, `Locked`, `Resolved`, `Archived`) with a
   `ThreadLifecycle` record enforcing constructor invariants (non-empty ids,
   `Classified` data-class tagging).
2. A `transition()` method enforcing only legal state progressions and
   rejecting all others with a typed `ChatError` variant.
3. Per-participant `ThreadSubscription` follow/mute invariants that mirror the
   `PresenceState` cross-pillar isolation pattern in `governance.rs`, so a
   participant from a disallowed `OwnershipPillar` cannot subscribe.

Pure domain types and validation only — no I/O, no REST/gRPC surface, no
usecase or adapter dependencies.

---

## Vertical and Crate Boundaries

| Layer | Location |
|-------|----------|
| Domain kernel (this task) | `crates/messenger-domain/src/thread_lifecycle.rs` |
| Usecase (future) | `crates/messenger-message-stream-usecase` (out of scope) |
| REST adapter (future) | `crates/messenger-message-stream-rest` (out of scope) |
| Persistence (future) | `crates/messenger-message-stream-adapter-postgres` (out of scope) |

The thread-lifecycle kernel is a pure domain module. It imports only
`data-boundary-kernel` (already a declared `[dependencies]` entry in
`crates/messenger-domain/Cargo.toml`) and `std`. No new workspace member.
Root `Cargo.toml` is untouched.

---

## Module Layout (flat clean-arch, ADR-0509)

```
crates/messenger-domain/
  src/
    lib.rs               ← adds `pub mod thread_lifecycle;` + `pub use thread_lifecycle::*;`
    delivery_class.rs    ← unchanged
    governance.rs        ← unchanged
    reaction.rs          ← unchanged
    thread_lifecycle.rs  ← NEW
```

---

## Domain Types

### ThreadState

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ThreadState {
    Open,
    Locked,
    Resolved,
    Archived,
}
```

`Archived` is a terminal state — no outbound transitions are permitted.

### Input record

```rust
pub struct ThreadLifecycleCreate {
    pub thread_id:     String,       // INTERNAL_ONLY
    pub tenant_id:     String,       // INTERNAL_ONLY
    pub initial_state: ThreadState,  // INTERNAL_ONLY
}
```

### Canonical record

```rust
pub struct ThreadLifecycle {
    pub thread_id: Classified<String>,      // INTERNAL_ONLY
    pub tenant_id: Classified<String>,      // INTERNAL_ONLY
    pub state:     Classified<ThreadState>, // INTERNAL_ONLY
}
```

### Subscription input record

```rust
pub struct ThreadSubscriptionCreate {
    pub thread_id:         String,          // INTERNAL_ONLY
    pub tenant_id:         String,          // INTERNAL_ONLY
    pub participant_ref:   String,          // PII_IDENTIFYING
    pub participant_pillar: OwnershipPillar, // INTERNAL_ONLY
    pub thread_pillar:     OwnershipPillar, // INTERNAL_ONLY
}
```

### Canonical subscription record

```rust
pub struct ThreadSubscription {
    pub thread_id:         Classified<String>,          // INTERNAL_ONLY
    pub tenant_id:         Classified<String>,          // INTERNAL_ONLY
    pub participant_ref:   Classified<String>,          // PII_IDENTIFYING
    pub participant_pillar: Classified<OwnershipPillar>, // INTERNAL_ONLY
    pub thread_pillar:     Classified<OwnershipPillar>, // INTERNAL_ONLY
    pub mode:              Classified<ThreadSubscriptionMode>, // INTERNAL_ONLY
}
```

### Subscription mode

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreadSubscriptionMode {
    Follow,
    Mute,
}
```

### Error additions to ChatError

Two new variants are added to the existing `ChatError` enum in `lib.rs`:

```rust
pub enum ChatError {
    // ... existing variants ...
    InvalidThreadStateTransition,   // illegal or no-op state transition
    CrossPillarSubscriptionDenied,  // participant_pillar != thread_pillar
}
```

---

## State-Transition Table

| From \ To  | Open | Locked | Resolved | Archived |
|-----------|:----:|:------:|:--------:|:--------:|
| **Open**     | ERR  | OK     | OK       | OK       |
| **Locked**   | OK   | ERR    | OK       | OK       |
| **Resolved** | OK   | ERR    | ERR      | OK       |
| **Archived** | ERR  | ERR    | ERR      | ERR      |

- `ERR` → `ChatError::InvalidThreadStateTransition`
- `Archived` is terminal: all outbound transitions are rejected.
- Self-transitions (same state) are rejected regardless of state.

---

## Pillar-Isolation Invariant

A `ThreadSubscription` is created only when `participant_pillar == thread_pillar`.
If the pillars differ, `ThreadSubscription::new` returns
`Err(ChatError::CrossPillarSubscriptionDenied)`. This mirrors the
`PresenceState::CrossPillarPresenceDenied` guard in `governance.rs` and is
consistent with the dual-context isolation policy in
`microservices/messenger/policy/dual-context-isolation.md`.

---

## Validation Rules

| Field | Rule | Error |
|-------|------|-------|
| `thread_id` | non-empty after trim | `ChatError::InvalidThreadId` |
| `tenant_id` | non-empty after trim | `ChatError::InvalidTenantId` |
| `participant_ref` (subscription) | non-empty after trim | `ChatError::InvalidParticipantRef` |
| `participant_pillar` vs `thread_pillar` | must be equal | `ChatError::CrossPillarSubscriptionDenied` |
| `transition(next)` | `next` must be reachable from current state per table | `ChatError::InvalidThreadStateTransition` |

---

## Data-Class Tagging

Consistent with existing kernel (`data-boundary-kernel`, ADR-0083):

| Field | `DataClass` / `PrivacyDataClass` |
|-------|----------------------------------|
| `thread_id`, `tenant_id`, `state`, `mode`, pillar fields | `DataClass::InternalOnly` |
| `participant_ref` | `PrivacyDataClass::pii_identifying()` |

---

## Contracts

### No REST/gRPC contract at this stage

This is a pure domain kernel slice. No OpenAPI 3.2.0 or proto3 contract is
defined here. Future adapter tasks referencing this kernel will author the
REST/gRPC surface.

### Future proto3 stub (informational, not normative)

```proto
// future: microservices/messenger/contracts/proto/messenger.proto
// (append to ThreadTree service)

enum ThreadState {
  THREAD_STATE_UNSPECIFIED = 0;
  THREAD_STATE_OPEN        = 1;
  THREAD_STATE_LOCKED      = 2;
  THREAD_STATE_RESOLVED    = 3;
  THREAD_STATE_ARCHIVED    = 4;
}

message ThreadLifecycle {
  string      thread_id  = 1;
  string      tenant_id  = 2;
  ThreadState state      = 3;
}

message TransitionThreadRequest {
  string      thread_id  = 1;
  string      tenant_id  = 2;
  ThreadState next_state = 3;
}

enum ThreadSubscriptionMode {
  THREAD_SUBSCRIPTION_MODE_UNSPECIFIED = 0;
  THREAD_SUBSCRIPTION_MODE_FOLLOW      = 1;
  THREAD_SUBSCRIPTION_MODE_MUTE        = 2;
}

message ThreadSubscription {
  string                  thread_id         = 1;
  string                  tenant_id         = 2;
  string                  participant_ref   = 3;
  ThreadSubscriptionMode  mode              = 4;
}
```

---

## Testing Strategy

All tests live in `src/thread_lifecycle.rs` under `#[cfg(test)]`. No new
integration test files. Pattern matches sibling modules (`reaction.rs`,
`governance.rs`).

### Subtask 1 — Constructor invariants

| Test | Asserts |
|------|---------|
| `constructor_valid_open` | `ThreadLifecycle::new(.., Open)` succeeds; `thread_id.data_class == InternalOnly`; `state.data_class == InternalOnly` |
| `empty_thread_id_rejected` | empty `thread_id` → `ChatError::InvalidThreadId` |
| `whitespace_thread_id_rejected` | whitespace-only `thread_id` → `ChatError::InvalidThreadId` |
| `empty_tenant_id_rejected` | empty `tenant_id` → `ChatError::InvalidTenantId` |

### Subtask 2 — Transition table

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

### Subtask 3 — Subscription pillar invariants

| Test | Asserts |
|------|---------|
| `same_pillar_work_follow_succeeds` | Work/Work subscription follows OK |
| `same_pillar_personal_mute_succeeds` | Personal/Personal subscription mutes OK |
| `cross_pillar_work_on_personal_denied` | Work participant / Personal thread → `CrossPillarSubscriptionDenied` |
| `cross_pillar_personal_on_work_denied` | Personal participant / Work thread → `CrossPillarSubscriptionDenied` |
| `follow_mute_round_trip` | follow → mute → follow returns correct modes each time |
| `invalid_thread_id_rejected` | Empty `thread_id` → `ChatError::InvalidThreadId` |
| `invalid_participant_ref_rejected` | Empty `participant_ref` → `ChatError::InvalidParticipantRef` |

---

## Boundaries

- This task touches ONLY:
  - `crates/messenger-domain/src/thread_lifecycle.rs` (new)
  - `crates/messenger-domain/src/lib.rs` (add mod + re-export + 2 ChatError variants)
  - `docs/specs/task-messenger-thread-lifecycle-kernel.md` (this file)
  - `tasks/messenger-thread-lifecycle-kernel-plan.md`
- Root `Cargo.toml` is untouched; no new workspace member.
- No changes to `delivery_class.rs`, `governance.rs`, or `reaction.rs`.
- No changes to any other crate.
- OpenSLO and Helm/IaC files are out of scope for this pure-domain slice.
