# Spec: messenger-domain-reaction-kernel

**Vertical**: messenger
**Task slug**: `messenger-domain-reaction-kernel`
**Crate**: `oya-messenger-domain` (`crates/oya-messenger-domain`)
**Module**: `src/reaction.rs` (new file, re-exported from `lib.rs`)
**Stage created**: SPEC
**Date**: 2026-05-28

---

## Objective

Extend the `oya-messenger-domain` crate with a fail-closed message-reaction kernel. The kernel owns
the typed `MessageReaction` record, constructor validation invariants, Classified data-class tagging,
and a per-message dedup invariant (`ReactionSet`). No I/O, no REST, no usecase or adapter dependencies.

---

## Vertical and Crate Boundaries

| Layer | Crate / location |
|-------|-----------------|
| Domain kernel (this task) | `crates/oya-messenger-domain/src/reaction.rs` |
| Usecase (future) | `crates/oya-messenger-message-stream-usecase` (out of scope) |
| REST adapter (future) | messenger REST crate (out of scope) |
| Persistence adapter (future) | `crates/oya-messenger-message-stream-adapter-postgres` (out of scope) |

The reaction kernel is a pure domain module. It imports only `data-boundary-kernel`
(already a declared dependency of `oya-messenger-domain`) and `std`.

---

## Module Layout (flat clean-arch, ADR-0509)

```
crates/oya-messenger-domain/
  src/
    lib.rs            ← adds `pub mod reaction;` + `pub use reaction::*;`
    delivery_class.rs ← unchanged
    governance.rs     ← unchanged
    reaction.rs       ← NEW
```

No new workspace member. Root `Cargo.toml` is untouched.

---

## Domain Types

### Input record

```rust
pub struct MessageReactionCreate {
    pub message_id:  String,   // INTERNAL_ONLY
    pub channel_id:  String,   // INTERNAL_ONLY
    pub tenant_id:   String,   // INTERNAL_ONLY
    pub actor_ref:   String,   // PII_IDENTIFYING
    pub emoji:       String,   // INTERNAL_ONLY (shortcode, e.g. ":thumbsup:")
    pub created_at:  u64,      // INTERNAL_ONLY (epoch seconds)
}
```

### Canonical record

```rust
pub struct MessageReaction {
    pub message_id:  Classified<String>,  // INTERNAL_ONLY
    pub channel_id:  Classified<String>,  // INTERNAL_ONLY
    pub tenant_id:   Classified<String>,  // INTERNAL_ONLY
    pub actor_ref:   Classified<String>,  // PII_IDENTIFYING
    pub emoji:       Classified<String>,  // INTERNAL_ONLY
    pub created_at:  Classified<u64>,     // INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // INTERNAL_ONLY
}
```

### Error enum

```rust
pub enum ReactionError {
    InvalidMessageId,
    InvalidChannelId,
    InvalidTenantId,
    InvalidActorRef,
    InvalidEmoji,
    DuplicateReaction,
}
```

### Dedup container

```rust
pub struct ReactionSet {
    // keyed by message_id; inner set is (actor_ref, emoji)
    reactions: BTreeMap<String, BTreeSet<(String, String)>>,
}

impl ReactionSet {
    pub fn new() -> Self { ... }
    pub fn add(&mut self, reaction: &MessageReaction) -> Result<(), ReactionError> { ... }
}
```

---

## Validation Rules

| Field | Rule | Error on violation |
|-------|------|--------------------|
| `message_id` | non-empty after trim | `InvalidMessageId` |
| `channel_id` | non-empty after trim | `InvalidChannelId` |
| `tenant_id` | non-empty after trim | `InvalidTenantId` |
| `actor_ref` | non-empty after trim; if `bot:` prefix then `len > 4` | `InvalidActorRef` |
| `emoji` | non-empty after trim; no control characters | `InvalidEmoji` |
| dedup | `(actor_ref, emoji)` unique per `message_id` in `ReactionSet` | `DuplicateReaction` |

Bot-principal reuse: an `actor_ref` with prefix `bot:` followed by a non-empty suffix is valid,
consistent with `validate_bot_principal` in `lib.rs`.

---

## Data-Class Tagging

Consistent with existing kernel (ADR-0083, data-boundary-kernel):

| Field | `DataClass` / `PrivacyDataClass` | Helper |
|-------|----------------------------------|--------|
| `actor_ref` | `PiiIdentifying` | `participant_data_class()` (re-use from lib or local equiv) |
| all other fields | `InternalOnly` | `internal()` private helper |

`schema_version` is tagged `INTERNAL_ONLY`, value `1` (matches `CHAT_MESSAGE_SCHEMA_VERSION` convention).

---

## Contracts

### No REST/gRPC contract at this stage

This is a pure domain kernel slice. No OpenAPI or proto3 contract is defined here.
The reaction REST endpoint and proto3 message definition will be authored in a future
adapter/usecase task referencing this kernel.

### Future proto3 stub (informational, not normative at this stage)

```proto
// future: specs/proto/backbone/messenger/reaction.proto
message MessageReaction {
  string message_id  = 1;
  string channel_id  = 2;
  string tenant_id   = 3;
  string actor_ref   = 4;
  string emoji       = 5;
  uint64 created_at  = 6;
  uint32 schema_version = 7;
}

enum ReactionError {
  REACTION_ERROR_UNSPECIFIED    = 0;
  REACTION_ERROR_INVALID_MESSAGE_ID = 1;
  REACTION_ERROR_INVALID_CHANNEL_ID = 2;
  REACTION_ERROR_INVALID_TENANT_ID  = 3;
  REACTION_ERROR_INVALID_ACTOR_REF  = 4;
  REACTION_ERROR_INVALID_EMOJI      = 5;
  REACTION_ERROR_DUPLICATE_REACTION = 6;
}
```

---

## Testing Strategy

All tests live in `src/reaction.rs` under `#[cfg(test)]`. No integration test files.
Pattern matches sibling modules (`lib.rs`, `delivery_class.rs`, `governance.rs`).

| Test | What it asserts |
|------|----------------|
| `happy_path_correct_data_class_tags` | `MessageReaction::new(..)` succeeds; `actor_ref.data_class == PiiIdentifying`; `emoji.data_class == InternalOnly`; `schema_version.value == 1` |
| `empty_ids_are_rejected` | empty `message_id`, `channel_id`, `tenant_id` each produce the right `ReactionError` |
| `empty_actor_ref_is_rejected` | empty `actor_ref` → `InvalidActorRef` |
| `bot_principal_accepted` | `actor_ref = "bot:triage"` passes validation |
| `invalid_emoji_rejected` | empty shortcode and control-char shortcode → `InvalidEmoji` |
| `duplicate_reaction_rejected` | same `(actor_ref, emoji)` on same message via `ReactionSet::add` → `DuplicateReaction` |
| `distinct_pairs_accepted` | different emoji OR different actor on same message → both accepted |

---

## Boundaries

- This task ONLY touches `crates/oya-messenger-domain/src/` and these two docs files.
- Root `Cargo.toml` is untouched; no new workspace member.
- No changes to `delivery_class.rs` or `governance.rs`.
- No changes to any other crate.
- OpenSLO and Helm/IAC files are out of scope for this pure-domain slice.
