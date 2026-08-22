# Spec: messenger-reaction-tally-kernel

**Crate**: `messenger-domain`  
**Module**: `reaction_tally`  
**Kind**: Pure deterministic kernel slice — no I/O, no async, no external dependencies beyond those already in Cargo.toml.

---

## Objective

Extend the messenger domain kernel with a `ReactionTally` aggregate that turns a flat sequence of
`MessageReaction` records (or a `ReactionSet` snapshot) into a deterministic, BTreeMap-ordered
per-emoji summary:

```
emoji → TallyEntry { count: Classified<u64>, actors: Classified<BTreeSet<String>> }
```

Plus a `toggle(actor_ref, emoji)` mutation that adds-or-removes the pair idempotently and
re-derives `count` from the actor roster.

---

## Contracts

No external API contract is introduced by this slice (pure domain kernel). The types become part
of the `messenger_domain` public surface and may be consumed by usecase/adapter layers in
future slices.

---

## Module Layout (flat-clean-arch per ADR-0509)

```
crates/messenger-domain/src/
  lib.rs                    ← add: pub mod reaction_tally; pub use reaction_tally::*;
  reaction.rs               ← existing, unchanged
  reaction_tally.rs         ← NEW: TallyEntry, ReactionTally, all unit tests
  thread_lifecycle.rs       ← existing, unchanged
  delivery_class.rs         ← existing, unchanged
  governance.rs             ← existing, unchanged
```

---

## Public API

```rust
/// A single per-emoji aggregate entry.
pub struct TallyEntry {
    /// Number of distinct actors who reacted with this emoji.
    pub count: Classified<u64>,          // data_class: INTERNAL_ONLY
    /// Sorted set of actor_refs who placed this reaction.
    pub actors: Classified<BTreeSet<String>>, // data_class: PII_IDENTIFYING
}

/// Deterministic per-emoji reaction aggregate for a single message.
///
/// Backed by a `BTreeMap<String, TallyEntry>` for stable emoji-ordering.
pub struct ReactionTally {
    // private: BTreeMap<emoji, TallyEntry>
}

impl ReactionTally {
    /// Construct from a slice of `MessageReaction` (reactions for one message).
    /// Duplicates (same actor+emoji) are silently collapsed (idempotent).
    pub fn from_reactions(reactions: &[MessageReaction]) -> Self;

    /// Iterate over `(emoji, &TallyEntry)` in BTreeMap order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &TallyEntry)>;

    /// Return the entry for `emoji`, or `None` if absent.
    pub fn get(&self, emoji: &str) -> Option<&TallyEntry>;

    /// Toggle `(actor_ref, emoji)`:
    ///   - If the pair is absent, add it.
    ///   - If the pair is present, remove it.
    /// Count is re-derived from the actor roster after the mutation.
    pub fn toggle(&mut self, actor_ref: &str, emoji: &str);
}
```

---

## Data-Class Tagging

| Field | Classified wrapper | data_class |
|---|---|---|
| `TallyEntry.count` | `Classified<u64>` | `INTERNAL_ONLY` |
| `TallyEntry.actors` | `Classified<BTreeSet<String>>` | `PII_IDENTIFYING` |

Actor refs inside the `actors` set are PII_IDENTIFYING per ADR-0083 and consistent with
`reaction_actor_data_class()` in `reaction.rs`.

---

## Testing Strategy

All tests are `#[cfg(test)]` unit tests inside `reaction_tally.rs`.

| Test name | What it checks |
|---|---|
| `empty_input_yields_empty_tally` | `from_reactions(&[])` → empty iterator |
| `single_reaction_correct_count_and_actor` | count=1, actors={actor} |
| `two_actors_same_emoji_count_two` | count=2, both actors present, emoji-ordered |
| `mixed_emoji_stable_btree_order` | BTreeMap order preserved across emoji keys |
| `duplicate_actor_emoji_not_double_counted` | same (actor, emoji) in slice twice → count=1 |
| `toggle_add_then_remove_round_trip` | add → count=1; remove → count=0/absent |
| `toggle_add_is_idempotent_within_slice` | calling toggle add twice leaves count=1 |
| `toggle_remove_absent_is_noop` | removing a pair that was never added is safe |
| `data_class_tags_correct` | count is INTERNAL_ONLY; actors is PII_IDENTIFYING |

---

## Observability / SLO

This is a pure kernel module with no runtime I/O. No SLO entry is required for domain-only slices
(per ADR-0130: SLO authoring is mandatory before a µservice promotes past dev; this module does
not constitute a µservice promotion).

---

## Crate Boundary

- ONLY `messenger-domain` is modified.
- No new workspace member.
- No edit to root `Cargo.toml`.
- No async / I/O / external crate additions.
