//! TDD RED tests — message-reaction domain kernel.
//!
//! These tests define the *full* expected behaviour of `MessageReaction`,
//! `MessageReactionCreate`, `ReactionSet`, and `ReactionError` as specified in
//! the slice acceptance criteria.  They are written BEFORE (or independently of)
//! the implementation so that any gap in the implementation causes a test
//! failure here.
//!
//! ADR-0083 Tier 3: integration tests are permitted to call `.unwrap()` /
//! `.expect()` to assert invariants.
//! ADR-0208: Professional InternalAuditable delivery-class semantics — reaction
//! visibility is strictly InternalAuditable; this is verified through
//! data-class tagging rather than delivery routing (delivery is an adapter
//! concern).

use comms_messenger_domain::{
    MessageReaction, MessageReactionCreate, ReactionError, ReactionSet, reaction_actor_data_class,
};
use data_boundary_kernel::{DataClass, DataClassification};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn valid_input() -> MessageReactionCreate {
    MessageReactionCreate {
        message_id: "msg-abc".into(),
        channel_id: "chan-xyz".into(),
        tenant_id: "tenant-t1".into(),
        actor_ref: "user:carol@example.com".into(),
        emoji: ":wave:".into(),
        created_at: 1_700_000_042,
    }
}

// ---------------------------------------------------------------------------
// Subtask-1: typed records + constructor validation
// ---------------------------------------------------------------------------

/// Happy path: a valid input produces a `MessageReaction` without error.
#[test]
fn new_returns_ok_for_fully_valid_input() {
    assert!(MessageReaction::new(valid_input()).is_ok());
}

/// `actor_ref` must be tagged `PII_IDENTIFYING`.
#[test]
fn actor_ref_field_is_tagged_pii_identifying() {
    let r = MessageReaction::new(valid_input()).unwrap();
    assert_eq!(
        r.actor_ref.data_class,
        DataClassification::Privacy(reaction_actor_data_class()),
    );
}

/// Every non-PII field must be tagged `INTERNAL_ONLY`.
#[test]
fn non_pii_fields_are_tagged_internal_only() {
    let r = MessageReaction::new(valid_input()).unwrap();
    let internal: DataClassification = DataClass::InternalOnly.into();
    assert_eq!(r.message_id.data_class, internal, "message_id");
    assert_eq!(r.channel_id.data_class, internal, "channel_id");
    assert_eq!(r.tenant_id.data_class, internal, "tenant_id");
    assert_eq!(r.emoji.data_class, internal, "emoji");
    assert_eq!(r.created_at.data_class, internal, "created_at");
    assert_eq!(r.schema_version.data_class, internal, "schema_version");
}

/// `schema_version` must be pinned at exactly 1.
#[test]
fn schema_version_is_pinned_at_1() {
    let r = MessageReaction::new(valid_input()).unwrap();
    assert_eq!(r.schema_version.value, 1u32);
}

/// Empty `message_id` (blank string) must return `InvalidMessageId`.
#[test]
fn empty_message_id_returns_invalid_message_id() {
    let mut i = valid_input();
    i.message_id = String::new();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidMessageId));
}

/// Whitespace-only `message_id` must return `InvalidMessageId`.
#[test]
fn whitespace_message_id_returns_invalid_message_id() {
    let mut i = valid_input();
    i.message_id = "   ".into();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidMessageId));
}

/// Empty `channel_id` must return `InvalidChannelId`.
#[test]
fn empty_channel_id_returns_invalid_channel_id() {
    let mut i = valid_input();
    i.channel_id = String::new();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidChannelId));
}

/// Empty `tenant_id` must return `InvalidTenantId`.
#[test]
fn empty_tenant_id_returns_invalid_tenant_id() {
    let mut i = valid_input();
    i.tenant_id = String::new();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidTenantId));
}

/// Empty `actor_ref` must return `InvalidActorRef`.
#[test]
fn empty_actor_ref_returns_invalid_actor_ref() {
    let mut i = valid_input();
    i.actor_ref = String::new();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidActorRef));
}

/// Whitespace-only `actor_ref` must return `InvalidActorRef`.
#[test]
fn whitespace_actor_ref_returns_invalid_actor_ref() {
    let mut i = valid_input();
    i.actor_ref = "   ".into();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidActorRef));
}

/// A bot principal with a valid suffix must be accepted.
#[test]
fn bot_principal_with_valid_suffix_is_accepted() {
    let mut i = valid_input();
    i.actor_ref = "bot:summariser".into();
    assert!(MessageReaction::new(i).is_ok());
}

/// `"bot:"` with no suffix must return `InvalidActorRef`.
#[test]
fn bot_principal_with_empty_suffix_returns_invalid_actor_ref() {
    let mut i = valid_input();
    i.actor_ref = "bot:".into();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidActorRef));
}

/// `"bot:  "` — bot prefix with whitespace-only suffix — must return
/// `InvalidActorRef`.  A whitespace suffix is semantically empty; accepting it
/// would allow a non-namespaced bot principal to bypass the namespace check.
#[test]
fn bot_principal_with_whitespace_only_suffix_returns_invalid_actor_ref() {
    let mut i = valid_input();
    i.actor_ref = "bot:  ".into();
    assert_eq!(
        MessageReaction::new(i),
        Err(ReactionError::InvalidActorRef),
        "bot principal with whitespace-only suffix must be rejected (namespace check)"
    );
}

/// Empty emoji shortcode must return `InvalidEmoji`.
#[test]
fn empty_emoji_returns_invalid_emoji() {
    let mut i = valid_input();
    i.emoji = String::new();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidEmoji));
}

/// Whitespace-only emoji shortcode must return `InvalidEmoji`.
#[test]
fn whitespace_emoji_returns_invalid_emoji() {
    let mut i = valid_input();
    i.emoji = "   ".into();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidEmoji));
}

/// Emoji containing a NUL control character must return `InvalidEmoji`.
#[test]
fn emoji_with_nul_control_char_returns_invalid_emoji() {
    let mut i = valid_input();
    i.emoji = ":thumb\x00up:".into();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidEmoji));
}

/// Emoji containing a tab (`\t`) control character must return `InvalidEmoji`.
#[test]
fn emoji_with_tab_control_char_returns_invalid_emoji() {
    let mut i = valid_input();
    i.emoji = ":tab\there:".into();
    assert_eq!(MessageReaction::new(i), Err(ReactionError::InvalidEmoji));
}

/// `created_at = 0` (Unix epoch) is a legitimate timestamp; must be accepted.
#[test]
fn created_at_zero_epoch_is_accepted() {
    let mut i = valid_input();
    i.created_at = 0;
    assert!(MessageReaction::new(i).is_ok());
}

// ---------------------------------------------------------------------------
// Subtask-2: ReactionSet per-message dedup invariant
// ---------------------------------------------------------------------------

/// A freshly created `ReactionSet` must accept the first addition without
/// error.
#[test]
fn reaction_set_accepts_first_reaction() {
    let r = MessageReaction::new(valid_input()).unwrap();
    let mut set = ReactionSet::new();
    assert!(set.add(&r).is_ok());
}

/// Adding the same `(actor_ref, emoji)` pair to the same message twice must
/// return `DuplicateReaction`.
#[test]
fn reaction_set_rejects_duplicate_actor_emoji_pair() {
    let r = MessageReaction::new(valid_input()).unwrap();
    let mut set = ReactionSet::new();
    set.add(&r).unwrap();
    assert_eq!(set.add(&r), Err(ReactionError::DuplicateReaction));
}

/// The same actor may add a *different* emoji to the same message.
#[test]
fn reaction_set_accepts_different_emoji_same_actor_same_message() {
    let r1 = MessageReaction::new(valid_input()).unwrap();
    let mut i2 = valid_input();
    i2.emoji = ":fire:".into();
    let r2 = MessageReaction::new(i2).unwrap();

    let mut set = ReactionSet::new();
    assert!(set.add(&r1).is_ok());
    assert!(set.add(&r2).is_ok());
}

/// Different actors may add the same emoji to the same message.
#[test]
fn reaction_set_accepts_same_emoji_different_actors_same_message() {
    let r1 = MessageReaction::new(valid_input()).unwrap();
    let mut i2 = valid_input();
    i2.actor_ref = "user:dave@example.com".into();
    let r2 = MessageReaction::new(i2).unwrap();

    let mut set = ReactionSet::new();
    assert!(set.add(&r1).is_ok());
    assert!(set.add(&r2).is_ok());
}

/// The same `(actor_ref, emoji)` pair on *different* messages must be accepted
/// (dedup is per-message, not global).
#[test]
fn reaction_set_accepts_same_pair_on_different_messages() {
    let r1 = MessageReaction::new(valid_input()).unwrap();
    let mut i2 = valid_input();
    i2.message_id = "msg-other".into();
    let r2 = MessageReaction::new(i2).unwrap();

    let mut set = ReactionSet::new();
    assert!(set.add(&r1).is_ok());
    assert!(set.add(&r2).is_ok());
}

/// After a third distinct emoji, a duplicate of the first must still be
/// rejected (state accumulates correctly across multiple distinct adds).
#[test]
fn reaction_set_rejects_duplicate_after_multiple_distinct_adds() {
    let r1 = MessageReaction::new(valid_input()).unwrap();
    let mut i2 = valid_input();
    i2.emoji = ":heart:".into();
    let r2 = MessageReaction::new(i2).unwrap();
    let mut i3 = valid_input();
    i3.emoji = ":clap:".into();
    let r3 = MessageReaction::new(i3).unwrap();

    let mut set = ReactionSet::new();
    set.add(&r1).unwrap();
    set.add(&r2).unwrap();
    set.add(&r3).unwrap();
    // Now attempt a duplicate of r1
    assert_eq!(set.add(&r1), Err(ReactionError::DuplicateReaction));
}

// ---------------------------------------------------------------------------
// Subtask-3: public re-export surface (lib.rs wiring)
// ---------------------------------------------------------------------------

/// All key types must be reachable directly from the crate root.  This test
/// exercises only the import path; the compiler enforces visibility.
#[test]
fn public_types_are_reachable_from_crate_root() {
    // If the types are not pub-used from lib.rs this file will fail to compile.
    let _: MessageReaction;
    let _: MessageReactionCreate;
    let _: ReactionError;
    let _: ReactionSet;
}
