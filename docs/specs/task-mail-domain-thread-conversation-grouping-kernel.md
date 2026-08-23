# Spec: Thread/Conversation Grouping Kernel

**Crate:** `mail-domain`  
**Module:** `src/thread_grouping.rs`  
**RFC references:** RFC 5322 §3.6.4 (In-Reply-To/References), §3.6.5 (Subject)

## Objective

Provide a pure, deterministic, zero-I/O kernel for assigning inbound messages to
conversation threads and for validating `ThreadStatus` lifecycle transitions.

## Contract

### `group_into_thread(headers: &[(&str, &str)]) -> ThreadAssignment`

Returns which existing thread to join, or how to start a new one.

**Precedence (highest first):**

1. `In-Reply-To`: extract the first `<...>` message-id token; return
   `ThreadAssignment::ExistingThread(id)`.
2. `References`: extract all `<...>` tokens; use the **last** one (closest ancestor);
   return `ThreadAssignment::ExistingThread(id)`.
3. `Subject` fallback: normalize (strip Re:/Fwd:/FW:/RE:/FWD: prefixes
   case-insensitively, collapse whitespace); if non-empty return
   `ThreadAssignment::FreshSubject(normalized)`.
4. Otherwise return `ThreadAssignment::Unthreaded`.

Header name matching is case-insensitive. Multiple headers with the same name are
scanned in order; first match wins per-level.

### `transition_thread_status(current: ThreadStatus, next: ThreadStatus) -> Result<ThreadStatus, ThreadTransitionError>`

**Legal transitions:**

| From     | To       |
|----------|----------|
| Active   | Muted    |
| Active   | Archived |
| Archived | Deleted  |
| Muted    | Deleted  |

Same-state (e.g. `Active→Active`) is idempotent: returns `Ok(current)`.  
All other moves return `Err(ThreadTransitionError::IllegalTransition { from, to })`.

## Types

```rust
pub enum ThreadAssignment {
    ExistingThread(String),   // message-id from In-Reply-To or References
    FreshSubject(String),     // normalized subject for a new thread
    Unthreaded,               // no threading information available
}

pub struct ThreadTransitionError {
    pub from: ThreadStatus,
    pub to: ThreadStatus,
}
```

## Mod layout (flat-clean-arch per ADR-0509)

```
crates/mail-domain/src/
  thread_grouping.rs   ← this slice (pure kernel, no deps beyond std)
  thread_state.rs      ← existing ThreadStatus type (unchanged)
  lib.rs               ← adds `pub mod thread_grouping; pub use thread_grouping::*;`
```

## Testing strategy

All tests hermetic (no I/O, no external state). Covers:

- In-Reply-To precedence over References and Subject.
- References precedence over Subject.
- Subject fallback with various Re:/Fwd: prefix combinations.
- Subject whitespace collapse.
- Subject case-folding.
- Empty/missing headers fall-through.
- Every legal transition.
- Every backward/illegal transition.
- Same-state idempotency for all four statuses.

## Observability / SLO

Pure kernel — no OTel instrumentation at this layer. Callers (use-case layer) emit
spans/metrics. No SLO file required for a pure domain kernel.

## Crate boundary

No new dependencies. Uses only `std` and existing `thread_state::ThreadStatus`.
