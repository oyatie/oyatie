//! Workspace chat kernel.
//!
//! Typed kernel records for the W-Workspace-Stable Chat surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns channel,
//! participant, message, attachment, thread, and bot identity invariants while
//! WSS, REST, moderation, and storage remain adapter concerns.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod delivery_class;
pub mod governance;
pub mod reaction;
pub mod reaction_tally;
pub mod thread_lifecycle;
pub use delivery_class::{MessageDeliveryClass, MessengerChannelKind};
pub use governance::*;
pub use reaction::*;
pub use reaction_tally::*;
pub use thread_lifecycle::*;

use std::collections::BTreeSet;

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const CHAT_CHANNEL_SCHEMA_VERSION: u32 = 1;
const CHAT_MESSAGE_SCHEMA_VERSION: u32 = 1;
const MIN_ATTACHMENT_BYTES: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChatError {
    InvalidChannelId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidChannelName,
    InvalidParticipantRef,
    DuplicateParticipantRef,
    EmptyParticipantSet,
    InvalidDirectMessageParticipantCount,
    InvalidGroupParticipantCount,
    MissingOwnerParticipant,
    InvalidBotPrincipal,
    InvalidMessageId,
    InvalidThreadId,
    InvalidParentMessageId,
    InvalidSenderRef,
    EmptyMessageBody,
    EmptyMessageContent,
    InvalidAttachmentId,
    InvalidAttachmentStorageKey,
    InvalidAttachmentMimeType,
    EmptyAttachment,
    InvalidDataClass,
    InvalidThreadStateTransition,
    CrossPillarSubscriptionDenied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChatChannelKind {
    DirectMessage,
    Group,
    Channel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChatParticipantRole {
    Owner,
    Admin,
    Member,
    Bot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChatSenderKind {
    Human,
    Bot,
    System,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatChannelCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub kind: ChatChannelKind,                // data_class: INTERNAL_ONLY
    pub name: Option<String>,                 // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub participants: Vec<ChatParticipant>,   // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatChannel {
    pub id: Classified<String>,                   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,            // data_class: INTERNAL_ONLY
    pub region: Classified<String>,               // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,              // data_class: INTERNAL_ONLY
    pub kind: Classified<ChatChannelKind>,        // data_class: INTERNAL_ONLY
    pub name: Classified<Option<String>>,         // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub participants: Classified<Vec<ChatParticipant>>, // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatParticipant {
    pub actor_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub display_name: Classified<Option<String>>, // data_class: PII_QUASI_IDENTIFIER
    pub role: Classified<ChatParticipantRole>, // data_class: INTERNAL_ONLY
    pub joined_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentRef {
    pub attachment_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub storage_key: Classified<String>,   // data_class: INTERNAL_ONLY
    pub mime_type: Classified<String>,     // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatMessageCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub channel_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub thread_id: Option<String>,            // data_class: INTERNAL_ONLY
    pub parent_message_id: Option<String>,    // data_class: INTERNAL_ONLY
    pub sender_ref: String,                   // data_class: PII_IDENTIFYING
    pub sender_kind: ChatSenderKind,          // data_class: INTERNAL_ONLY
    pub body: Option<String>,                 // data_class: PII_IDENTIFYING
    pub attachments: Vec<AttachmentRef>,      // data_class: PII_IDENTIFYING
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatMessage {
    pub id: Classified<String>,                // data_class: INTERNAL_ONLY
    pub channel_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub thread_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub parent_message_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub sender_ref: Classified<String>,        // data_class: PII_IDENTIFYING
    pub sender_kind: Classified<ChatSenderKind>, // data_class: INTERNAL_ONLY
    pub body: Classified<Option<String>>,      // data_class: PII_IDENTIFYING
    pub attachments: Classified<Vec<AttachmentRef>>, // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,       // data_class: INTERNAL_ONLY
}

pub trait ChatMessageReader {
    fn messages_for_channel(
        &self,
        tenant_id: &str,
        channel_id: &str,
    ) -> Result<Vec<ChatMessage>, ChatError>;
}

impl ChatChannel {
    pub fn new(input: ChatChannelCreate) -> Result<Self, ChatError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_chat_data_class());
        validate_non_empty(&input.id, ChatError::InvalidChannelId)?;
        validate_non_empty(&input.tenant_id, ChatError::InvalidTenantId)?;
        validate_non_empty(&input.region, ChatError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, ChatError::InvalidCellId)?;
        validate_optional_name(input.name.as_deref())?;
        validate_participants(input.kind, &input.participants)?;

        Ok(Self {
            id: internal(input.id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            kind: internal(input.kind),
            name: Classified::new(input.name, channel_name_data_class()),
            data_class: internal(data_class),
            participants: Classified::new(input.participants, participant_data_class()),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(CHAT_CHANNEL_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl ChatParticipant {
    pub fn new(
        actor_ref: String,
        display_name: Option<String>,
        role: ChatParticipantRole,
        joined_at_epoch_seconds: u64,
    ) -> Result<Self, ChatError> {
        validate_non_empty(&actor_ref, ChatError::InvalidParticipantRef)?;
        validate_optional_name(display_name.as_deref())?;
        if role == ChatParticipantRole::Bot {
            validate_bot_principal(&actor_ref)?;
        }
        Ok(Self {
            actor_ref: Classified::new(actor_ref, participant_data_class()),
            display_name: Classified::new(display_name, channel_name_data_class()),
            role: internal(role),
            joined_at_epoch_seconds: internal(joined_at_epoch_seconds),
        })
    }
}

impl AttachmentRef {
    pub fn new(
        attachment_id: String,
        storage_key: String,
        mime_type: String,
        byte_len: u64,
    ) -> Result<Self, ChatError> {
        validate_non_empty(&attachment_id, ChatError::InvalidAttachmentId)?;
        validate_non_empty(&storage_key, ChatError::InvalidAttachmentStorageKey)?;
        validate_non_empty(&mime_type, ChatError::InvalidAttachmentMimeType)?;
        if byte_len < MIN_ATTACHMENT_BYTES {
            return Err(ChatError::EmptyAttachment);
        }
        Ok(Self {
            attachment_id: internal(attachment_id),
            storage_key: internal(storage_key),
            mime_type: internal(mime_type),
            byte_len: internal(byte_len),
        })
    }
}

impl ChatMessage {
    pub fn new(input: ChatMessageCreate) -> Result<Self, ChatError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_chat_data_class());
        validate_non_empty(&input.id, ChatError::InvalidMessageId)?;
        validate_non_empty(&input.channel_id, ChatError::InvalidChannelId)?;
        validate_non_empty(&input.tenant_id, ChatError::InvalidTenantId)?;
        validate_optional_non_empty(input.thread_id.as_deref(), ChatError::InvalidThreadId)?;
        validate_optional_non_empty(
            input.parent_message_id.as_deref(),
            ChatError::InvalidParentMessageId,
        )?;
        validate_non_empty(&input.sender_ref, ChatError::InvalidSenderRef)?;
        validate_message_content(input.body.as_deref(), &input.attachments)?;
        validate_thread_shape(&input.id, &input.thread_id, &input.parent_message_id)?;
        if input.sender_kind == ChatSenderKind::Bot {
            validate_bot_principal(&input.sender_ref)?;
        }

        Ok(Self {
            id: internal(input.id),
            channel_id: internal(input.channel_id),
            tenant_id: internal(input.tenant_id),
            thread_id: internal(input.thread_id),
            parent_message_id: internal(input.parent_message_id),
            sender_ref: Classified::new(input.sender_ref, participant_data_class()),
            sender_kind: internal(input.sender_kind),
            body: Classified::new(input.body, message_body_data_class()),
            attachments: Classified::new(input.attachments, message_body_data_class()),
            data_class: internal(data_class),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(CHAT_MESSAGE_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

pub fn default_workspace_chat_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn participant_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn channel_name_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn message_body_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_chat_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, ChatError> {
    PrivacyDataClass::new(data_class).map_err(|_| ChatError::InvalidDataClass)
}

fn validate_participants(
    kind: ChatChannelKind,
    participants: &[ChatParticipant],
) -> Result<(), ChatError> {
    if participants.is_empty() {
        return Err(ChatError::EmptyParticipantSet);
    }
    match kind {
        ChatChannelKind::DirectMessage if participants.len() != 2 => {
            return Err(ChatError::InvalidDirectMessageParticipantCount);
        }
        ChatChannelKind::Group if participants.len() < 3 => {
            return Err(ChatError::InvalidGroupParticipantCount);
        }
        _ => {}
    }
    let mut actor_refs = BTreeSet::new();
    let mut has_owner = false;
    for participant in participants {
        validate_non_empty(
            &participant.actor_ref.value,
            ChatError::InvalidParticipantRef,
        )?;
        validate_optional_name(participant.display_name.value.as_deref())?;
        if participant.role.value == ChatParticipantRole::Bot {
            validate_bot_principal(&participant.actor_ref.value)?;
        }
        if participant.role.value == ChatParticipantRole::Owner {
            has_owner = true;
        }
        if !actor_refs.insert(participant.actor_ref.value.clone()) {
            return Err(ChatError::DuplicateParticipantRef);
        }
    }
    if !has_owner {
        return Err(ChatError::MissingOwnerParticipant);
    }
    Ok(())
}

fn validate_message_content(
    body: Option<&str>,
    attachments: &[AttachmentRef],
) -> Result<(), ChatError> {
    if body.is_none() && attachments.is_empty() {
        return Err(ChatError::EmptyMessageContent);
    }
    if let Some(body) = body
        && (body.trim().is_empty() || body.chars().any(char::is_control))
    {
        return Err(ChatError::EmptyMessageBody);
    }
    Ok(())
}

fn validate_thread_shape(
    message_id: &str,
    thread_id: &Option<String>,
    parent_message_id: &Option<String>,
) -> Result<(), ChatError> {
    if parent_message_id.as_deref() == Some(message_id) {
        return Err(ChatError::InvalidParentMessageId);
    }
    if parent_message_id.is_some() && thread_id.is_none() {
        return Err(ChatError::InvalidThreadId);
    }
    Ok(())
}

fn validate_bot_principal(actor_ref: &str) -> Result<(), ChatError> {
    if actor_ref.starts_with("bot:") && actor_ref.len() > "bot:".len() {
        Ok(())
    } else {
        Err(ChatError::InvalidBotPrincipal)
    }
}

fn validate_optional_name(value: Option<&str>) -> Result<(), ChatError> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.trim() != value || value.is_empty() || value.chars().any(char::is_control) {
        Err(ChatError::InvalidChannelName)
    } else {
        Ok(())
    }
}

fn validate_optional_non_empty(value: Option<&str>, error: ChatError) -> Result<(), ChatError> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_non_empty(value, error)
}

fn validate_non_empty(value: &str, error: ChatError) -> Result<(), ChatError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn owner(actor_ref: &str) -> ChatParticipant {
        ChatParticipant::new(
            actor_ref.into(),
            Some("Owner".into()),
            ChatParticipantRole::Owner,
            1_700_000_000,
        )
        .unwrap()
    }

    fn member(actor_ref: &str) -> ChatParticipant {
        ChatParticipant::new(
            actor_ref.into(),
            Some("Member".into()),
            ChatParticipantRole::Member,
            1_700_000_001,
        )
        .unwrap()
    }

    fn bot() -> ChatParticipant {
        ChatParticipant::new(
            "bot:triage".into(),
            Some("Triage Bot".into()),
            ChatParticipantRole::Bot,
            1_700_000_002,
        )
        .unwrap()
    }

    fn channel_input() -> ChatChannelCreate {
        ChatChannelCreate {
            id: "channel-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            kind: ChatChannelKind::Channel,
            name: Some("incident-room".into()),
            data_class: None,
            participants: vec![
                owner("user:owner@example.com"),
                member("user:member@example.com"),
                bot(),
            ],
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn message_input() -> ChatMessageCreate {
        ChatMessageCreate {
            id: "message-1".into(),
            channel_id: "channel-1".into(),
            tenant_id: "tenant-1".into(),
            thread_id: None,
            parent_message_id: None,
            sender_ref: "user:owner@example.com".into(),
            sender_kind: ChatSenderKind::Human,
            body: Some("ship status?".into()),
            attachments: Vec::new(),
            data_class: None,
            created_at_epoch_seconds: 1_700_000_010,
        }
    }

    #[test]
    fn channel_defaults_to_identifying_and_validates_participants() {
        let channel = ChatChannel::new(channel_input()).unwrap();

        assert_eq!(
            channel.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            channel.name.data_class,
            DataClassification::Privacy(channel_name_data_class())
        );
        assert_eq!(
            channel.participants.data_class,
            DataClassification::Privacy(participant_data_class())
        );
        assert_eq!(channel.schema_version.value, 1);
    }

    #[test]
    fn direct_message_and_group_participant_counts_are_fail_closed() {
        let mut invalid_dm = channel_input();
        invalid_dm.kind = ChatChannelKind::DirectMessage;
        invalid_dm.participants = vec![owner("user:one@example.com")];
        assert_eq!(
            ChatChannel::new(invalid_dm),
            Err(ChatError::InvalidDirectMessageParticipantCount)
        );

        let mut invalid_group = channel_input();
        invalid_group.kind = ChatChannelKind::Group;
        invalid_group.participants = vec![
            owner("user:one@example.com"),
            member("user:two@example.com"),
        ];
        assert_eq!(
            ChatChannel::new(invalid_group),
            Err(ChatError::InvalidGroupParticipantCount)
        );
    }

    #[test]
    fn bot_principals_are_explicitly_namespaced() {
        assert_eq!(
            ChatParticipant::new(
                "user:fake-bot@example.com".into(),
                None,
                ChatParticipantRole::Bot,
                1_700_000_000,
            ),
            Err(ChatError::InvalidBotPrincipal)
        );

        let mut bot_message = message_input();
        bot_message.sender_ref = "bot:triage".into();
        bot_message.sender_kind = ChatSenderKind::Bot;
        assert!(ChatMessage::new(bot_message).is_ok());
    }

    #[test]
    fn messages_require_content_and_valid_thread_shape() {
        let message = ChatMessage::new(message_input()).unwrap();
        assert_eq!(
            message.body.data_class,
            DataClassification::Privacy(message_body_data_class())
        );

        let mut empty = message_input();
        empty.body = None;
        assert_eq!(ChatMessage::new(empty), Err(ChatError::EmptyMessageContent));

        let mut invalid_thread = message_input();
        invalid_thread.parent_message_id = Some("message-0".into());
        assert_eq!(
            ChatMessage::new(invalid_thread),
            Err(ChatError::InvalidThreadId)
        );
    }

    #[test]
    fn attachments_and_legacy_data_classes_are_validated() {
        assert_eq!(
            AttachmentRef::new(
                "attachment-1".into(),
                "tenant-1/chat/channel-1/file.bin".into(),
                "application/octet-stream".into(),
                0,
            ),
            Err(ChatError::EmptyAttachment)
        );
        assert_eq!(
            workspace_chat_data_class_from_legacy(DataClass::Audit),
            Err(ChatError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.messenger STAGING surface markers (SPEC §4).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessengerSurfaceStaging {
    pub thread_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub message_count: Classified<u64>, // data_class: INTERNAL_ONLY
}

impl MessengerSurfaceStaging {
    pub fn new(thread_id: String, tenant_id: String, message_count: u64) -> Self {
        Self {
            thread_id: Classified::new(thread_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            message_count: Classified::new(message_count, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> MessengerSurfaceStaging {
        MessengerSurfaceStaging::new("t-1".into(), "tenant-1".into(), 0)
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.thread_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
