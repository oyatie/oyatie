//! Reaction-tally aggregate kernel.
//!
//! Deterministic, BTreeMap-ordered per-emoji aggregate over a set of
//! `MessageReaction` records for a single message.  Pure domain logic —
//! no I/O, no async, no REST/usecase dependencies.
//!
//! # Data-class tagging
//! - `TallyEntry.count`:  `Classified<u64>`              — `INTERNAL_ONLY`
//! - `TallyEntry.actors`: `Classified<BTreeSet<String>>` — `PII_IDENTIFYING`
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::reaction::MessageReaction;

// ---------------------------------------------------------------------------
// TallyEntry
// ---------------------------------------------------------------------------

/// Aggregated reaction data for a single emoji on a single message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TallyEntry {
    /// Number of distinct actors who reacted with this emoji.
    pub count: Classified<u64>, // data_class: INTERNAL_ONLY
    /// Sorted roster of actor_refs who placed this reaction.
    pub actors: Classified<BTreeSet<String>>, // data_class: PII_IDENTIFYING
}

impl TallyEntry {
    fn from_actor_set(actors: BTreeSet<String>) -> Self {
        let count = actors.len() as u64;
        Self {
            count: internal(count),
            actors: Classified::new(actors, tally_actor_data_class()),
        }
    }
}

// ---------------------------------------------------------------------------
// ReactionTally
// ---------------------------------------------------------------------------

/// Deterministic per-emoji reaction aggregate for a single message.
///
/// Backed by a `BTreeMap<String, TallyEntry>` for stable emoji ordering.
/// Entries with an empty actor set are removed from the map.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReactionTally {
    entries: BTreeMap<String, TallyEntry>,
}

impl ReactionTally {
    /// Construct from a slice of `MessageReaction` (all for one message).
    ///
    /// Duplicate `(actor_ref, emoji)` pairs in the input are silently
    /// collapsed — consistent with `ReactionSet` semantics.
    pub fn from_reactions(reactions: &[MessageReaction]) -> Self {
        let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for r in reactions {
            map.entry(r.emoji.value.clone())
                .or_default()
                .insert(r.actor_ref.value.clone());
        }
        let entries = map
            .into_iter()
            .map(|(emoji, actors)| (emoji, TallyEntry::from_actor_set(actors)))
            .collect();
        Self { entries }
    }

    /// Iterate over `(emoji, &TallyEntry)` in BTreeMap (lexicographic) order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &TallyEntry)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Return the entry for `emoji`, or `None` if absent.
    pub fn get(&self, emoji: &str) -> Option<&TallyEntry> {
        self.entries.get(emoji)
    }

    /// Toggle `(actor_ref, emoji)`:
    ///
    /// - If the pair is absent → add it.
    /// - If the pair is present → remove it.
    ///
    /// `count` is re-derived from the actor roster after each mutation.
    /// Removing a pair that was never added is a no-op (idempotent).
    /// When the last actor for an emoji is removed, the emoji entry is
    /// dropped entirely so the tally stays clean.
    pub fn toggle(&mut self, actor_ref: &str, emoji: &str) {
        let actor_set = self
            .entries
            .entry(emoji.to_owned())
            .or_insert_with(|| TallyEntry::from_actor_set(BTreeSet::new()));

        if actor_set.actors.value.contains(actor_ref) {
            actor_set.actors.value.remove(actor_ref);
        } else {
            actor_set.actors.value.insert(actor_ref.to_owned());
        }

        let new_count = actor_set.actors.value.len() as u64;
        actor_set.count = internal(new_count);

        // Drop the emoji entry when the actor roster becomes empty.
        if new_count == 0 {
            self.entries.remove(emoji);
        }
    }
}

// ---------------------------------------------------------------------------
// Data-class helpers
// ---------------------------------------------------------------------------

/// Actor refs inside a tally are PII_IDENTIFYING, consistent with
/// `reaction_actor_data_class()` in `reaction.rs`.
pub fn tally_actor_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClass, DataClassification};

    use crate::reaction::MessageReactionCreate;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn reaction(actor: &str, emoji: &str) -> MessageReaction {
        MessageReaction::new(MessageReactionCreate {
            message_id: "msg-1".into(),
            channel_id: "chan-1".into(),
            tenant_id: "tenant-1".into(),
            actor_ref: actor.into(),
            emoji: emoji.into(),
            created_at: 1_700_000_000,
        })
        .unwrap()
    }

    // -----------------------------------------------------------------------
    // Acceptance tests
    // -----------------------------------------------------------------------

    #[test]
    fn empty_input_yields_empty_tally() {
        let tally = ReactionTally::from_reactions(&[]);
        assert_eq!(tally.entries().count(), 0);
    }

    #[test]
    fn single_reaction_correct_count_and_actor() {
        let r = reaction("user:alice@example.com", ":thumbsup:");
        let tally = ReactionTally::from_reactions(&[r]);

        let entry = tally.get(":thumbsup:").unwrap();
        assert_eq!(entry.count.value, 1);
        assert!(entry.actors.value.contains("user:alice@example.com"));
    }

    #[test]
    fn two_actors_same_emoji_count_two() {
        let r1 = reaction("user:alice@example.com", ":thumbsup:");
        let r2 = reaction("user:bob@example.com", ":thumbsup:");
        let tally = ReactionTally::from_reactions(&[r1, r2]);

        let entry = tally.get(":thumbsup:").unwrap();
        assert_eq!(entry.count.value, 2);
        assert!(entry.actors.value.contains("user:alice@example.com"));
        assert!(entry.actors.value.contains("user:bob@example.com"));
    }

    #[test]
    fn mixed_emoji_stable_btree_order() {
        // BTreeMap orders lexicographically: ":heart:" < ":thumbsup:" < ":wave:"
        let reactions = vec![
            reaction("user:alice@example.com", ":wave:"),
            reaction("user:bob@example.com", ":heart:"),
            reaction("user:carol@example.com", ":thumbsup:"),
        ];
        let tally = ReactionTally::from_reactions(&reactions);
        let keys: Vec<&str> = tally.entries().map(|(k, _)| k).collect();
        assert_eq!(keys, vec![":heart:", ":thumbsup:", ":wave:"]);
    }

    #[test]
    fn duplicate_actor_emoji_not_double_counted() {
        // Same (actor, emoji) pair appears twice in the input slice.
        let r = reaction("user:alice@example.com", ":thumbsup:");
        let tally = ReactionTally::from_reactions(&[r.clone(), r]);

        let entry = tally.get(":thumbsup:").unwrap();
        assert_eq!(entry.count.value, 1);
        assert_eq!(entry.actors.value.len(), 1);
    }

    #[test]
    fn toggle_add_then_remove_round_trip() {
        let mut tally = ReactionTally::from_reactions(&[]);
        assert!(tally.get(":thumbsup:").is_none());

        // Add
        tally.toggle("user:alice@example.com", ":thumbsup:");
        let entry = tally.get(":thumbsup:").unwrap();
        assert_eq!(entry.count.value, 1);

        // Remove — must return to empty
        tally.toggle("user:alice@example.com", ":thumbsup:");
        assert!(tally.get(":thumbsup:").is_none());
    }

    #[test]
    fn toggle_add_second_actor_then_remove_first_leaves_one() {
        let r = reaction("user:alice@example.com", ":thumbsup:");
        let mut tally = ReactionTally::from_reactions(&[r]);

        tally.toggle("user:bob@example.com", ":thumbsup:");
        assert_eq!(tally.get(":thumbsup:").unwrap().count.value, 2);

        tally.toggle("user:alice@example.com", ":thumbsup:");
        let entry = tally.get(":thumbsup:").unwrap();
        assert_eq!(entry.count.value, 1);
        assert!(!entry.actors.value.contains("user:alice@example.com"));
        assert!(entry.actors.value.contains("user:bob@example.com"));
    }

    #[test]
    fn toggle_remove_absent_is_noop() {
        let mut tally = ReactionTally::from_reactions(&[]);
        // Removing a pair that was never added must not panic and must be a no-op.
        tally.toggle("user:alice@example.com", ":thumbsup:");
        // We just added it — remove it to get back to empty.
        tally.toggle("user:alice@example.com", ":thumbsup:");
        assert!(tally.get(":thumbsup:").is_none());

        // Toggle remove on a completely absent emoji is also safe.
        // Collect the entry count before (owned, not borrowed) to avoid borrow conflict.
        let count_before = tally.entries().count();
        tally.toggle("user:ghost@example.com", ":ghost:");
        // After one toggle on absent, it's now present with count=1.
        tally.toggle("user:ghost@example.com", ":ghost:");
        // After two toggles (add+remove), it's absent again — same count as before.
        let count_after = tally.entries().count();
        assert_eq!(count_before, count_after);
    }

    #[test]
    fn data_class_tags_correct() {
        let r = reaction("user:alice@example.com", ":thumbsup:");
        let tally = ReactionTally::from_reactions(&[r]);
        let entry = tally.get(":thumbsup:").unwrap();

        // count → INTERNAL_ONLY
        assert_eq!(entry.count.data_class, DataClass::InternalOnly.into());
        // actors → PII_IDENTIFYING
        assert_eq!(
            entry.actors.data_class,
            DataClassification::Privacy(tally_actor_data_class())
        );
    }

    #[test]
    fn toggle_preserves_data_class_after_mutation() {
        let mut tally = ReactionTally::from_reactions(&[]);
        tally.toggle("user:alice@example.com", ":thumbsup:");
        let entry = tally.get(":thumbsup:").unwrap();
        assert_eq!(entry.count.data_class, DataClass::InternalOnly.into());
        assert_eq!(
            entry.actors.data_class,
            DataClassification::Privacy(tally_actor_data_class())
        );
    }
}
