//! Message reaction domain kernel.
//!
//! Typed `MessageReaction` record, constructor validation, Classified data-class
//! tagging, and per-message dedup invariant (`ReactionSet`) for the messenger
//! vertical.  Pure domain logic — no I/O, no REST/usecase dependencies.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const REACTION_SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReactionError {
    InvalidMessageId,
    InvalidChannelId,
    InvalidTenantId,
    InvalidActorRef,
    InvalidEmoji,
    DuplicateReaction,
}

// ---------------------------------------------------------------------------
// Input record (plain fields, no Classified)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageReactionCreate {
    pub message_id: String, // data_class: INTERNAL_ONLY
    pub channel_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
    pub actor_ref: String,  // data_class: PII_IDENTIFYING
    pub emoji: String,      // data_class: INTERNAL_ONLY
    pub created_at: u64,    // data_class: INTERNAL_ONLY (epoch seconds)
}

// ---------------------------------------------------------------------------
// Canonical record (Classified fields)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageReaction {
    pub message_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub channel_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub actor_ref: Classified<String>,   // data_class: PII_IDENTIFYING
    pub emoji: Classified<String>,       // data_class: INTERNAL_ONLY
    pub created_at: Classified<u64>,     // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

impl MessageReaction {
    pub fn new(input: MessageReactionCreate) -> Result<Self, ReactionError> {
        validate_non_empty(&input.message_id, ReactionError::InvalidMessageId)?;
        validate_non_empty(&input.channel_id, ReactionError::InvalidChannelId)?;
        validate_non_empty(&input.tenant_id, ReactionError::InvalidTenantId)?;
        validate_actor_ref(&input.actor_ref)?;
        validate_emoji(&input.emoji)?;

        Ok(Self {
            message_id: internal(input.message_id),
            channel_id: internal(input.channel_id),
            tenant_id: internal(input.tenant_id),
            actor_ref: Classified::new(input.actor_ref, reaction_actor_data_class()),
            emoji: internal(input.emoji),
            created_at: internal(input.created_at),
            schema_version: internal(REACTION_SCHEMA_VERSION),
        })
    }
}

// ---------------------------------------------------------------------------
// Per-message dedup invariant
// ---------------------------------------------------------------------------

/// Tracks which `(actor_ref, emoji)` pairs have been applied to each message.
///
/// Mirrors the `BTreeSet` / `DuplicateParticipantRef` dedup pattern in
/// `lib.rs`.  A given actor may only place one reaction of a given emoji on
/// a given message; a second attempt returns
/// `Err(ReactionError::DuplicateReaction)`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReactionSet {
    // keyed by message_id; inner set is (actor_ref, emoji)
    reactions: BTreeMap<String, BTreeSet<(String, String)>>,
}

impl ReactionSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, reaction: &MessageReaction) -> Result<(), ReactionError> {
        let key = (
            reaction.actor_ref.value.clone(),
            reaction.emoji.value.clone(),
        );
        let set = self
            .reactions
            .entry(reaction.message_id.value.clone())
            .or_default();
        if !set.insert(key) {
            return Err(ReactionError::DuplicateReaction);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Data-class helpers
// ---------------------------------------------------------------------------

pub fn reaction_actor_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

// ---------------------------------------------------------------------------
// Private validation helpers
// ---------------------------------------------------------------------------

fn validate_non_empty(value: &str, error: ReactionError) -> Result<(), ReactionError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_actor_ref(actor_ref: &str) -> Result<(), ReactionError> {
    if actor_ref.trim().is_empty() {
        return Err(ReactionError::InvalidActorRef);
    }
    // Bot principals must be namespaced: "bot:<non-empty-suffix>"
    // (consistent with validate_bot_principal in lib.rs)
    // The suffix must be non-empty after trimming — "bot:  " is semantically
    // equivalent to "bot:" and must be rejected.
    if let Some(suffix) = actor_ref.strip_prefix("bot:")
        && suffix.trim().is_empty()
    {
        return Err(ReactionError::InvalidActorRef);
    }
    Ok(())
}

fn validate_emoji(emoji: &str) -> Result<(), ReactionError> {
    if emoji.trim().is_empty() || emoji.chars().any(char::is_control) {
        return Err(ReactionError::InvalidEmoji);
    }
    Ok(())
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_boundary_kernel::{DataClass, DataClassification};

    fn valid_input() -> MessageReactionCreate {
        MessageReactionCreate {
            message_id: "msg-1".into(),
            channel_id: "chan-1".into(),
            tenant_id: "tenant-1".into(),
            actor_ref: "user:alice@example.com".into(),
            emoji: ":thumbsup:".into(),
            created_at: 1_700_000_000,
        }
    }

    // --- happy path ---

    #[test]
    fn happy_path_correct_data_class_tags() {
        let r = MessageReaction::new(valid_input()).unwrap();

        // actor_ref is PII_IDENTIFYING
        assert_eq!(
            r.actor_ref.data_class,
            DataClassification::Privacy(reaction_actor_data_class())
        );
        // all other fields are INTERNAL_ONLY
        assert_eq!(r.message_id.data_class, DataClass::InternalOnly.into());
        assert_eq!(r.channel_id.data_class, DataClass::InternalOnly.into());
        assert_eq!(r.tenant_id.data_class, DataClass::InternalOnly.into());
        assert_eq!(r.emoji.data_class, DataClass::InternalOnly.into());
        assert_eq!(r.created_at.data_class, DataClass::InternalOnly.into());
        assert_eq!(r.schema_version.data_class, DataClass::InternalOnly.into());
        assert_eq!(r.schema_version.value, 1);
    }

    // --- empty-id rejection ---

    #[test]
    fn empty_message_id_is_rejected() {
        let mut input = valid_input();
        input.message_id = "  ".into();
        assert_eq!(
            MessageReaction::new(input),
            Err(ReactionError::InvalidMessageId)
        );
    }

    #[test]
    fn empty_channel_id_is_rejected() {
        let mut input = valid_input();
        input.channel_id = String::new();
        assert_eq!(
            MessageReaction::new(input),
            Err(ReactionError::InvalidChannelId)
        );
    }

    #[test]
    fn empty_tenant_id_is_rejected() {
        let mut input = valid_input();
        input.tenant_id = "\t".into();
        assert_eq!(
            MessageReaction::new(input),
            Err(ReactionError::InvalidTenantId)
        );
    }

    // --- actor_ref validation ---

    #[test]
    fn empty_actor_ref_is_rejected() {
        let mut input = valid_input();
        input.actor_ref = String::new();
        assert_eq!(
            MessageReaction::new(input),
            Err(ReactionError::InvalidActorRef)
        );
    }

    #[test]
    fn bot_principal_with_valid_suffix_is_accepted() {
        let mut input = valid_input();
        input.actor_ref = "bot:triage".into();
        assert!(MessageReaction::new(input).is_ok());
    }

    #[test]
    fn bot_principal_without_suffix_is_rejected() {
        let mut input = valid_input();
        input.actor_ref = "bot:".into();
        assert_eq!(
            MessageReaction::new(input),
            Err(ReactionError::InvalidActorRef)
        );
    }

    // --- emoji validation ---

    #[test]
    fn empty_emoji_shortcode_is_rejected() {
        let mut input = valid_input();
        input.emoji = String::new();
        assert_eq!(
            MessageReaction::new(input),
            Err(ReactionError::InvalidEmoji)
        );
    }

    #[test]
    fn whitespace_only_emoji_is_rejected() {
        let mut input = valid_input();
        input.emoji = "   ".into();
        assert_eq!(
            MessageReaction::new(input),
            Err(ReactionError::InvalidEmoji)
        );
    }

    #[test]
    fn control_char_emoji_is_rejected() {
        let mut input = valid_input();
        input.emoji = ":thumb\x00up:".into();
        assert_eq!(
            MessageReaction::new(input),
            Err(ReactionError::InvalidEmoji)
        );
    }

    // --- dedup invariant ---

    #[test]
    fn duplicate_reaction_is_rejected() {
        let r = MessageReaction::new(valid_input()).unwrap();
        let mut set = ReactionSet::new();
        assert!(set.add(&r).is_ok());
        assert_eq!(set.add(&r), Err(ReactionError::DuplicateReaction));
    }

    #[test]
    fn distinct_emoji_same_actor_same_message_accepted() {
        let r1 = MessageReaction::new(valid_input()).unwrap();
        let mut input2 = valid_input();
        input2.emoji = ":heart:".into();
        let r2 = MessageReaction::new(input2).unwrap();

        let mut set = ReactionSet::new();
        assert!(set.add(&r1).is_ok());
        assert!(set.add(&r2).is_ok());
    }

    #[test]
    fn distinct_actor_same_emoji_same_message_accepted() {
        let r1 = MessageReaction::new(valid_input()).unwrap();
        let mut input2 = valid_input();
        input2.actor_ref = "user:bob@example.com".into();
        let r2 = MessageReaction::new(input2).unwrap();

        let mut set = ReactionSet::new();
        assert!(set.add(&r1).is_ok());
        assert!(set.add(&r2).is_ok());
    }

    #[test]
    fn same_pair_on_different_messages_accepted() {
        let r1 = MessageReaction::new(valid_input()).unwrap();
        let mut input2 = valid_input();
        input2.message_id = "msg-2".into();
        let r2 = MessageReaction::new(input2).unwrap();

        let mut set = ReactionSet::new();
        assert!(set.add(&r1).is_ok());
        assert!(set.add(&r2).is_ok());
    }
}
