# Plan: mail-domain-thread-conversation-grouping-kernel

## Objective

Add a pure deterministic thread/conversation-grouping kernel to `oya-mail-domain`.
No I/O, no DNS, no network. Hermetic unit tests only.

## Acceptance Criteria

1. New `thread_grouping` mod re-exported from `lib.rs`.
2. `group_into_thread` resolves thread assignment using precedence:
   - `In-Reply-To` header (first valid message-id)
   - `References` header (last valid message-id in the list)
   - Normalized `Subject` fallback (strip Re:/Fwd: prefixes, collapse whitespace, case-fold)
3. `transition_thread_status` validates `ThreadStatus` transitions:
   - Legal forward transitions: `Active→Muted`, `Active→Archived`, `Archived→Deleted`, `Muted→Deleted`
   - Same-state is idempotent (no-op, returns current state)
   - All other moves return `ThreadTransitionError::IllegalTransition`
4. All paths covered by deterministic unit tests.
5. No change to workspace `Cargo.toml`.

## Edge Cases

- `In-Reply-To` with multiple IDs: use first (RFC 5322 §3.6.4 allows a list).
- `References` with multiple IDs: use last (closest ancestor).
- Subject stripping: `Re:`, `Fwd:`, `FW:`, `RE:`, `FWD:` case-insensitive, repeated.
- Whitespace collapse after prefix strip (trim + single-space).
- Empty/absent headers: fall through to next precedence level.
- All headers absent/empty: return `ThreadAssignment::FreshSubject(normalized)` with the
  normalized subject, or `ThreadAssignment::Unthreaded` if subject also empty.
- Backward transitions (`Archived→Active`, `Deleted→*`): `IllegalTransition`.
- Same-state is always OK (idempotent).

## Subtasks

- [x] Write plan file
- [x] Write spec file
- [x] Implement `thread_grouping.rs` with tests (red first, then green)
- [x] Re-export from `lib.rs`
- [x] Verify build + tests green
- [x] Self-review + simplify
