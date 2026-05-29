# Task Plan: messenger-domain-reaction-kernel

**Vertical**: messenger
**Crate**: `oya-messenger-domain` (`crates/oya-messenger-domain`)
**Branch**: `feat/task-messenger-domain-reaction-kernel-2026-05-28`

---

## Objective

Add a fail-closed message-reaction domain kernel as `src/reaction.rs` inside the existing
`oya-messenger-domain` crate. Pure domain logic only — no I/O, no REST/usecase dependencies.

---

## Subtasks

### [messenger-domain-reaction-kernel-1] Reaction types + constructor validation

**Scope**: Create `src/reaction.rs` with:

- `MessageReactionCreate` input struct (plain fields, no `Classified`)
- `MessageReaction` record (all fields wrapped in `Classified`)
- `ReactionError` enum covering all invalid-input variants
- `MessageReaction::new(..)` constructor with non-empty + emoji-shortcode validation
- `REACTION_SCHEMA_VERSION` const (u32 = 1)
- Data-class tagging:
  - `actor_ref` → `PII_IDENTIFYING` (matches `participant_data_class()`)
  - `message_id`, `channel_id`, `tenant_id`, `emoji`, `created_at` → `INTERNAL_ONLY` (matches `internal()`)
- Module-level `#![cfg_attr(test, allow(...))]` consistent with sibling modules

**Acceptance**:
- `cargo check -p oya-messenger-domain --all-targets` is green
- `reaction.rs` compiles cleanly; no new workspace member; root `Cargo.toml` unchanged

---

### [messenger-domain-reaction-kernel-2] Per-message dedup invariant + lib.rs re-export

**Scope**:

- Add `ReactionSet` struct holding a `BTreeSet<(String, String)>` of `(actor_ref, emoji)` pairs
  keyed by `message_id`, mirroring the `DuplicateParticipantRef` BTreeSet pattern in `lib.rs`
- `ReactionSet::add(..)` accepts a `&MessageReaction` and returns `Err(ReactionError::DuplicateReaction)`
  when the same `(actor_ref, emoji)` pair already exists on that message
- Add `ReactionError::DuplicateReaction` variant to the enum
- Wire into `lib.rs`:
  - `pub mod reaction;`
  - `pub use reaction::*;`

**Acceptance**:
- Public types reachable from crate root
- `cargo check -p oya-messenger-domain --all-targets` green
- No symbol collisions with existing `ChatError` / `Chat*` exports

---

### [messenger-domain-reaction-kernel-3] Unit tests in reaction.rs

**Scope**: `#[cfg(test)]` block covering:

1. Happy-path: `MessageReaction::new(..)` with valid inputs; assert correct `Classified` data-class tags
2. Empty-id rejection: empty `message_id`, `channel_id`, `tenant_id` each return the correct `ReactionError` variant
3. Empty-actor rejection: empty `actor_ref` returns `ReactionError::InvalidActorRef`
4. Bot-principal reuse: `actor_ref = "bot:triage"` accepted (namespaced bot principal)
5. Invalid-emoji rejection: control-char shortcode and empty shortcode return `ReactionError::InvalidEmoji`
6. Duplicate-reaction rejection: same `(actor_ref, emoji)` on same message returns `ReactionError::DuplicateReaction`
7. Distinct `(actor_ref, emoji)` pairs on same message are both accepted

**Acceptance**:
- `cargo nextest run -p oya-messenger-domain` passes with new tests
- Every `ReactionError` variant is asserted in at least one test
- Existing crate tests remain green

---

## Acceptance Summary

| Subtask | Gate |
|---------|------|
| 1 | `cargo check -p oya-messenger-domain --all-targets` green |
| 2 | Public types reachable from root; check green; no collisions |
| 3 | `cargo nextest run -p oya-messenger-domain` all tests pass |
