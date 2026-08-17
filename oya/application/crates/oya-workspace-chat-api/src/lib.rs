//! Workspace Chat API boundary for message send.
//!
//! This crate owns REST-boundary normalization, idempotent message-send
//! handling, coarse authorization proof checks, channel membership checks, and
//! thread parent existence around the Workspace Chat kernel. WSS fan-out,
//! moderation, durable storage, bot execution, and Matrix compatibility remain
//! adapter concerns.

pub mod surface_kind;
pub use surface_kind::WorkspaceSurfaceKind;

use std::collections::BTreeMap;

use comms_messenger_domain::{
    AttachmentRef, ChatChannel, ChatChannelCreate, ChatChannelKind, ChatError, ChatMessage,
    ChatMessageCreate, ChatParticipant, ChatParticipantRole, ChatSenderKind,
    workspace_chat_data_class_from_legacy,
};
use oya_data_boundary_kernel::parse_data_class_label;

pub const WORKSPACE_CHAT_MESSAGE_SEND_SURFACE: &str = "workspace.chat.message.send";
pub const WORKSPACE_CHAT_OPENAPI_CONTRACT: &str =
    "contracts/openapi/workspace/workspace-chat-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceChatMessageSendApiStatus {
    Created,
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl WorkspaceChatMessageSendApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceChatApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    ChannelIdInvalid,
    MessageIdInvalid,
    ChannelIdMismatch,
    MessageIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    SenderPermissionDenied,
    IdempotencyKeyReused,
    DataClassInvalid,
    ChannelKindInvalid,
    ParticipantRoleInvalid,
    SenderKindInvalid,
    ChannelNotFound,
    ParentMessageNotFound,
    MessageAlreadyExists,
    ChatInvalidRequest,
}

impl WorkspaceChatApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "WORKSPACE_CHAT_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "WORKSPACE_CHAT_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "WORKSPACE_CHAT_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "WORKSPACE_CHAT_PRINCIPAL_ID_EMPTY",
            Self::ChannelIdInvalid => "WORKSPACE_CHAT_CHANNEL_ID_INVALID",
            Self::MessageIdInvalid => "WORKSPACE_CHAT_MESSAGE_ID_INVALID",
            Self::ChannelIdMismatch => "WORKSPACE_CHAT_CHANNEL_ID_MISMATCH",
            Self::MessageIdMismatch => "WORKSPACE_CHAT_MESSAGE_ID_MISMATCH",
            Self::TenantMismatch => "WORKSPACE_CHAT_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "WORKSPACE_CHAT_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "WORKSPACE_CHAT_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "WORKSPACE_CHAT_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "WORKSPACE_CHAT_AUTHORIZATION_DENIED",
            Self::SenderPermissionDenied => "WORKSPACE_CHAT_SENDER_PERMISSION_DENIED",
            Self::IdempotencyKeyReused => "WORKSPACE_CHAT_IDEMPOTENCY_KEY_REUSED",
            Self::DataClassInvalid => "WORKSPACE_CHAT_DATA_CLASS_INVALID",
            Self::ChannelKindInvalid => "WORKSPACE_CHAT_CHANNEL_KIND_INVALID",
            Self::ParticipantRoleInvalid => "WORKSPACE_CHAT_PARTICIPANT_ROLE_INVALID",
            Self::SenderKindInvalid => "WORKSPACE_CHAT_SENDER_KIND_INVALID",
            Self::ChannelNotFound => "WORKSPACE_CHAT_CHANNEL_NOT_FOUND",
            Self::ParentMessageNotFound => "WORKSPACE_CHAT_PARENT_MESSAGE_NOT_FOUND",
            Self::MessageAlreadyExists => "WORKSPACE_CHAT_MESSAGE_ALREADY_EXISTS",
            Self::ChatInvalidRequest => "WORKSPACE_CHAT_INVALID_REQUEST",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatSendBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: PII_IDENTIFYING
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatAttachmentRequest {
    pub attachment_id: String, // data_class: INTERNAL_ONLY
    pub storage_key: String,   // data_class: INTERNAL_ONLY
    pub mime_type: String,     // data_class: INTERNAL_ONLY
    pub byte_len: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatChannelSeed {
    pub channel_id: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub region: String,       // data_class: INTERNAL_ONLY
    pub cell_id: String,      // data_class: INTERNAL_ONLY
    pub kind: String,         // data_class: INTERNAL_ONLY
    pub name: Option<String>, // data_class: PII_QUASI_IDENTIFIER
    pub data_class: String,   // data_class: INTERNAL_ONLY
    pub participants: Vec<(String, Option<String>, String)>, // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatMessageSendRequest {
    pub message_id: String,                // data_class: INTERNAL_ONLY
    pub channel_id: String,                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub thread_id: Option<String>,         // data_class: INTERNAL_ONLY
    pub parent_message_id: Option<String>, // data_class: INTERNAL_ONLY
    pub sender_ref: String,                // data_class: PII_IDENTIFYING
    pub sender_kind: String,               // data_class: INTERNAL_ONLY
    pub body: Option<String>,              // data_class: PII_IDENTIFYING
    pub attachments: Vec<WorkspaceChatAttachmentRequest>, // data_class: PII_IDENTIFYING
    pub data_class: String,                // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatMessageSendApiRequest {
    pub path_channel_id: String, // data_class: INTERNAL_ONLY
    pub path_message_id: String, // data_class: INTERNAL_ONLY
    pub boundary: WorkspaceChatSendBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: WorkspaceChatApiPrincipal, // data_class: PII_IDENTIFYING
    pub authorization: WorkspaceChatApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: WorkspaceChatMessageSendRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceChatMessageDirectory {
    channels: BTreeMap<WorkspaceChatChannelKey, ChatChannel>, // data_class: INTERNAL_ONLY
    messages: BTreeMap<WorkspaceChatMessageKey, ChatMessage>, // data_class: INTERNAL_ONLY
}

impl WorkspaceChatMessageDirectory {
    pub fn channel_len(&self) -> usize {
        self.channels.len()
    }

    pub fn message_len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.channels.is_empty() && self.messages.is_empty()
    }

    pub fn messages(&self) -> impl Iterator<Item = &ChatMessage> {
        self.messages.values()
    }

    pub fn insert_channel_seed(
        &mut self,
        seed: WorkspaceChatChannelSeed,
    ) -> Result<(), WorkspaceChatApiError> {
        let channel = chat_channel_from_seed(seed)?;
        let key = WorkspaceChatChannelKey {
            tenant_id: channel.tenant_id.value.clone(),
            channel_id: channel.id.value.clone(),
        };
        self.channels.insert(key, channel);
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceChatChannelKey {
    tenant_id: String,  // data_class: INTERNAL_ONLY
    channel_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceChatMessageKey {
    tenant_id: String,  // data_class: INTERNAL_ONLY
    channel_id: String, // data_class: INTERNAL_ONLY
    message_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceChatMessageSendIdempotencyLedger {
    entries: BTreeMap<WorkspaceChatIdempotencyLedgerKey, WorkspaceChatSendLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl WorkspaceChatMessageSendIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceChatIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: PII_IDENTIFYING
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceChatSendLedgerEntry {
    fingerprint: WorkspaceChatRequestFingerprint, // data_class: INTERNAL_ONLY
    result: WorkspaceChatMessageSendSuccessResponse, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceChatRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatMessageSendSuccessResponse {
    pub data: WorkspaceChatMessageRecord, // data_class: INTERNAL_ONLY
    pub metadata: WorkspaceChatMessageMetadata, // data_class: INTERNAL_ONLY
}

impl WorkspaceChatMessageSendSuccessResponse {
    pub fn created(data: WorkspaceChatMessageRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: WorkspaceChatMessageMetadata {
                request_id: request_id.into(),
                surface: WORKSPACE_CHAT_MESSAGE_SEND_SURFACE.to_string(),
                openapi_contract: WORKSPACE_CHAT_OPENAPI_CONTRACT.to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatMessageMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatAttachmentRecord {
    pub attachment_id: String, // data_class: INTERNAL_ONLY
    pub storage_key: String,   // data_class: INTERNAL_ONLY
    pub mime_type: String,     // data_class: INTERNAL_ONLY
    pub byte_len: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatMessageRecord {
    pub message_id: String,                // data_class: INTERNAL_ONLY
    pub channel_id: String,                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub thread_id: Option<String>,         // data_class: INTERNAL_ONLY
    pub parent_message_id: Option<String>, // data_class: INTERNAL_ONLY
    pub sender_ref: String,                // data_class: PII_IDENTIFYING
    pub sender_kind: String,               // data_class: INTERNAL_ONLY
    pub body: Option<String>,              // data_class: PII_IDENTIFYING
    pub attachments: Vec<WorkspaceChatAttachmentRecord>, // data_class: PII_IDENTIFYING
    pub data_class: String,                // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub schema_version: u32,               // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatApiErrorResponse {
    pub error: WorkspaceChatApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatApiErrorBody {
    pub code: String,                              // data_class: INTERNAL_ONLY
    pub message: String,                           // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,         // data_class: INTERNAL_ONLY
    pub request_id: String,                        // data_class: INTERNAL_ONLY
    pub details: Vec<WorkspaceChatApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceChatApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceChatApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    InvalidChannelId {
        channel_id: String,
    },
    InvalidMessageId {
        message_id: String,
    },
    ChannelIdMismatch {
        path_channel_id: String,
        body_channel_id: String,
    },
    MessageIdMismatch {
        path_message_id: String,
        body_message_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        authorization_tenant_id: Option<String>,
        body_tenant_id: Option<String>,
        resource_tenant_id: Option<String>,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        principal_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    SenderPermissionDenied {
        principal_id: String,
        sender_ref: String,
        channel_id: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    InvalidChannelKind {
        kind: String,
    },
    InvalidParticipantRole {
        role: String,
    },
    InvalidSenderKind {
        sender_kind: String,
    },
    ChannelNotFound {
        tenant_id: String,
        channel_id: String,
    },
    ParentMessageNotFound {
        tenant_id: String,
        channel_id: String,
        parent_message_id: String,
    },
    MessageAlreadyExists {
        tenant_id: String,
        channel_id: String,
        message_id: String,
    },
    Chat(ChatError),
}

impl WorkspaceChatApiError {
    pub fn message_status_code(&self) -> u16 {
        match self.status_kind() {
            WorkspaceChatApiStatusKind::BadRequest => 400,
            WorkspaceChatApiStatusKind::Forbidden => 403,
            WorkspaceChatApiStatusKind::NotFound => 404,
            WorkspaceChatApiStatusKind::Conflict => 409,
            WorkspaceChatApiStatusKind::UnprocessableEntity => 422,
        }
    }

    pub fn code(&self) -> WorkspaceChatApiErrorCode {
        match self {
            Self::EmptyRequestId => WorkspaceChatApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => WorkspaceChatApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => WorkspaceChatApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => WorkspaceChatApiErrorCode::PrincipalIdEmpty,
            Self::InvalidChannelId { .. } => WorkspaceChatApiErrorCode::ChannelIdInvalid,
            Self::InvalidMessageId { .. } => WorkspaceChatApiErrorCode::MessageIdInvalid,
            Self::ChannelIdMismatch { .. } => WorkspaceChatApiErrorCode::ChannelIdMismatch,
            Self::MessageIdMismatch { .. } => WorkspaceChatApiErrorCode::MessageIdMismatch,
            Self::TenantMismatch { .. } => WorkspaceChatApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                WorkspaceChatApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                WorkspaceChatApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                WorkspaceChatApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => WorkspaceChatApiErrorCode::AuthorizationDenied,
            Self::SenderPermissionDenied { .. } => {
                WorkspaceChatApiErrorCode::SenderPermissionDenied
            }
            Self::IdempotencyKeyReused { .. } => WorkspaceChatApiErrorCode::IdempotencyKeyReused,
            Self::InvalidDataClassLabel { .. } => WorkspaceChatApiErrorCode::DataClassInvalid,
            Self::InvalidChannelKind { .. } => WorkspaceChatApiErrorCode::ChannelKindInvalid,
            Self::InvalidParticipantRole { .. } => {
                WorkspaceChatApiErrorCode::ParticipantRoleInvalid
            }
            Self::InvalidSenderKind { .. } => WorkspaceChatApiErrorCode::SenderKindInvalid,
            Self::ChannelNotFound { .. } => WorkspaceChatApiErrorCode::ChannelNotFound,
            Self::ParentMessageNotFound { .. } => WorkspaceChatApiErrorCode::ParentMessageNotFound,
            Self::MessageAlreadyExists { .. } => WorkspaceChatApiErrorCode::MessageAlreadyExists,
            Self::Chat(_) => WorkspaceChatApiErrorCode::ChatInvalidRequest,
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> WorkspaceChatApiErrorResponse {
        WorkspaceChatApiErrorResponse {
            error: WorkspaceChatApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> WorkspaceChatApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::SenderPermissionDenied { .. } => WorkspaceChatApiStatusKind::Forbidden,
            Self::ChannelNotFound { .. } | Self::ParentMessageNotFound { .. } => {
                WorkspaceChatApiStatusKind::NotFound
            }
            Self::MessageAlreadyExists { .. } => WorkspaceChatApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => WorkspaceChatApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::InvalidChannelId { .. }
            | Self::InvalidMessageId { .. }
            | Self::ChannelIdMismatch { .. }
            | Self::MessageIdMismatch { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidChannelKind { .. }
            | Self::InvalidParticipantRole { .. }
            | Self::InvalidSenderKind { .. }
            | Self::Chat(_) => WorkspaceChatApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::InvalidChannelId { .. } => "Workspace Chat channel id is required",
            Self::InvalidMessageId { .. } => "Workspace Chat message id is required",
            Self::ChannelIdMismatch { .. } => "Path and body channel ids must match",
            Self::MessageIdMismatch { .. } => "Path and body message ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, authorization, body, and resource"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the Workspace Chat message-send surface"
            }
            Self::SenderPermissionDenied { .. } => {
                "Sender must match the authenticated principal and be a channel participant"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::InvalidChannelKind { .. } => "Channel kind must be known",
            Self::InvalidParticipantRole { .. } => "Participant role must be known",
            Self::InvalidSenderKind { .. } => "Sender kind must be known",
            Self::ChannelNotFound { .. } => "Workspace Chat channel was not found",
            Self::ParentMessageNotFound { .. } => "Workspace Chat parent message was not found",
            Self::MessageAlreadyExists { .. } => "Workspace Chat message already exists",
            Self::Chat(error) => chat_error_message(error),
        }
    }

    fn details(&self) -> Vec<WorkspaceChatApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::InvalidChannelId { .. } => {
                vec![detail("path.channel_id", "must be non-empty")]
            }
            Self::InvalidMessageId { .. } => {
                vec![detail("path.message_id", "must be non-empty")]
            }
            Self::ChannelIdMismatch { .. } => vec![detail(
                "channel_id",
                "path channel_id and body channel_id must match",
            )],
            Self::MessageIdMismatch { .. } => vec![detail(
                "message_id",
                "path message_id and body message_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, authorization tenant, body tenant_id, and resource tenant must match",
            )],
            Self::EmptyAuthorizationDecisionId => vec![detail(
                "authorization.decision_id",
                "must be non-empty authorization evidence",
            )],
            Self::AuthorizationTenantMismatch { .. } => vec![detail(
                "authorization.tenant_id",
                "must match the authenticated principal tenant",
            )],
            Self::AuthorizationPrincipalMismatch { .. } => vec![detail(
                "authorization.principal_id",
                "must match the authenticated principal id",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization.allowed_surfaces",
                "must include workspace.chat.message.send",
            )],
            Self::SenderPermissionDenied { .. } => vec![detail(
                "body.sender_ref",
                "sender_ref must match the authenticated principal and a channel participant",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::InvalidChannelKind { .. } => vec![detail(
                "channel.kind",
                "must be one of direct_message, group, channel",
            )],
            Self::InvalidParticipantRole { .. } => vec![detail(
                "channel.participants.role",
                "must be one of owner, admin, member, bot",
            )],
            Self::InvalidSenderKind { .. } => vec![detail(
                "body.sender_kind",
                "must be one of human, bot, system",
            )],
            Self::ChannelNotFound { .. } => vec![detail(
                "path.channel_id",
                "channel metadata must exist before messages can be sent",
            )],
            Self::ParentMessageNotFound { .. } => vec![detail(
                "body.parent_message_id",
                "parent message must exist in the same tenant channel for threaded replies",
            )],
            Self::MessageAlreadyExists { .. } => vec![detail(
                "path.message_id",
                "message already exists for the requested tenant channel",
            )],
            Self::Chat(error) => vec![detail("workspace_chat", chat_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceChatApiStatusKind {
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_workspace_chat_message_send_request(
    request: &WorkspaceChatMessageSendApiRequest,
) -> Result<(), WorkspaceChatApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_channel_id(&request.path_channel_id)?;
    validate_path_message_id(&request.path_message_id)?;
    validate_path_body_binding(
        &request.path_channel_id,
        &request.body.channel_id,
        &request.path_message_id,
        &request.body.message_id,
    )?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &request.authorization,
        Some(&request.body.tenant_id),
        None,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        WORKSPACE_CHAT_MESSAGE_SEND_SURFACE,
    )?;
    Ok(())
}

pub fn send_workspace_chat_message_from_api(
    directory: &mut WorkspaceChatMessageDirectory,
    idempotency_ledger: &mut WorkspaceChatMessageSendIdempotencyLedger,
    request: WorkspaceChatMessageSendApiRequest,
) -> Result<WorkspaceChatMessageSendSuccessResponse, WorkspaceChatApiError> {
    validate_workspace_chat_message_send_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        WORKSPACE_CHAT_MESSAGE_SEND_SURFACE,
    );
    let fingerprint = send_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(WorkspaceChatApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let principal_id = request.principal.principal_id.clone();
    let channel_id = request.body.channel_id.clone();
    let tenant_id = request.body.tenant_id.clone();
    let channel_key = WorkspaceChatChannelKey {
        tenant_id: tenant_id.clone(),
        channel_id: channel_id.clone(),
    };
    let channel = directory.channels.get(&channel_key).ok_or_else(|| {
        WorkspaceChatApiError::ChannelNotFound {
            tenant_id: tenant_id.clone(),
            channel_id: channel_id.clone(),
        }
    })?;
    require_sender(
        channel,
        &principal_id,
        &request.body.sender_ref,
        &channel_id,
    )?;
    if let Some(parent_message_id) = &request.body.parent_message_id {
        let parent_key = WorkspaceChatMessageKey {
            tenant_id: tenant_id.clone(),
            channel_id: channel_id.clone(),
            message_id: parent_message_id.clone(),
        };
        if !directory.messages.contains_key(&parent_key) {
            return Err(WorkspaceChatApiError::ParentMessageNotFound {
                tenant_id: tenant_id.clone(),
                channel_id: channel_id.clone(),
                parent_message_id: parent_message_id.clone(),
            });
        }
    }

    let message = chat_message_from_request(request.body)?;
    let message_key = WorkspaceChatMessageKey {
        tenant_id,
        channel_id,
        message_id: message.id.value.clone(),
    };
    if directory.messages.contains_key(&message_key) {
        return Err(WorkspaceChatApiError::MessageAlreadyExists {
            tenant_id: message_key.tenant_id,
            channel_id: message_key.channel_id,
            message_id: message_key.message_id,
        });
    }

    let response =
        WorkspaceChatMessageSendSuccessResponse::created(message_record(&message), request_id);
    directory.messages.insert(message_key, message);
    idempotency_ledger.entries.insert(
        key,
        WorkspaceChatSendLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(
    boundary: &WorkspaceChatSendBoundaryContext,
) -> Result<(), WorkspaceChatApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(WorkspaceChatApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(WorkspaceChatApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(WorkspaceChatApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_channel_id(channel_id: &str) -> Result<(), WorkspaceChatApiError> {
    if channel_id.trim().is_empty() {
        return Err(WorkspaceChatApiError::InvalidChannelId {
            channel_id: channel_id.to_string(),
        });
    }
    Ok(())
}

fn validate_path_message_id(message_id: &str) -> Result<(), WorkspaceChatApiError> {
    if message_id.trim().is_empty() {
        return Err(WorkspaceChatApiError::InvalidMessageId {
            message_id: message_id.to_string(),
        });
    }
    Ok(())
}

fn validate_path_body_binding(
    path_channel_id: &str,
    body_channel_id: &str,
    path_message_id: &str,
    body_message_id: &str,
) -> Result<(), WorkspaceChatApiError> {
    if path_channel_id != body_channel_id {
        return Err(WorkspaceChatApiError::ChannelIdMismatch {
            path_channel_id: path_channel_id.to_string(),
            body_channel_id: body_channel_id.to_string(),
        });
    }
    if path_message_id != body_message_id {
        return Err(WorkspaceChatApiError::MessageIdMismatch {
            path_message_id: path_message_id.to_string(),
            body_message_id: body_message_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    header_tenant_id: &str,
    principal: &WorkspaceChatApiPrincipal,
    authorization: &WorkspaceChatApiAuthorization,
    body_tenant_id: Option<&str>,
    resource_tenant_id: Option<&str>,
) -> Result<(), WorkspaceChatApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(WorkspaceChatApiError::EmptyPrincipalId);
    }
    if header_tenant_id != principal.tenant_id
        || header_tenant_id != authorization.tenant_id
        || body_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id)
        || resource_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id)
    {
        return Err(WorkspaceChatApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            authorization_tenant_id: Some(authorization.tenant_id.clone()),
            body_tenant_id: body_tenant_id.map(str::to_string),
            resource_tenant_id: resource_tenant_id.map(str::to_string),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &WorkspaceChatApiPrincipal,
    authorization: &WorkspaceChatApiAuthorization,
    surface: &str,
) -> Result<(), WorkspaceChatApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(WorkspaceChatApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(WorkspaceChatApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(WorkspaceChatApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed| allowed == surface)
    {
        return Err(WorkspaceChatApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn chat_channel_from_seed(
    seed: WorkspaceChatChannelSeed,
) -> Result<ChatChannel, WorkspaceChatApiError> {
    let data_class = parse_privacy_data_class_label(&seed.data_class)?;
    let participants = seed
        .participants
        .into_iter()
        .map(|(actor_ref, display_name, role)| {
            ChatParticipant::new(
                actor_ref,
                display_name,
                participant_role_from_label(&role)?,
                seed.created_at_epoch_seconds,
            )
            .map_err(WorkspaceChatApiError::Chat)
        })
        .collect::<Result<Vec<_>, _>>()?;
    ChatChannel::new(ChatChannelCreate {
        id: seed.channel_id,
        tenant_id: seed.tenant_id,
        region: seed.region,
        cell_id: seed.cell_id,
        kind: channel_kind_from_label(&seed.kind)?,
        name: seed.name,
        data_class: Some(data_class),
        participants,
        created_at_epoch_seconds: seed.created_at_epoch_seconds,
    })
    .map_err(WorkspaceChatApiError::Chat)
}

fn chat_message_from_request(
    request: WorkspaceChatMessageSendRequest,
) -> Result<ChatMessage, WorkspaceChatApiError> {
    let data_class = parse_privacy_data_class_label(&request.data_class)?;
    let attachments = request
        .attachments
        .into_iter()
        .map(attachment_from_request)
        .collect::<Result<Vec<_>, _>>()?;
    ChatMessage::new(ChatMessageCreate {
        id: request.message_id,
        channel_id: request.channel_id,
        tenant_id: request.tenant_id,
        thread_id: request.thread_id,
        parent_message_id: request.parent_message_id,
        sender_ref: request.sender_ref,
        sender_kind: sender_kind_from_label(&request.sender_kind)?,
        body: request.body,
        attachments,
        data_class: Some(data_class),
        created_at_epoch_seconds: request.created_at_epoch_seconds,
    })
    .map_err(WorkspaceChatApiError::Chat)
}

fn attachment_from_request(
    attachment: WorkspaceChatAttachmentRequest,
) -> Result<AttachmentRef, WorkspaceChatApiError> {
    AttachmentRef::new(
        attachment.attachment_id,
        attachment.storage_key,
        attachment.mime_type,
        attachment.byte_len,
    )
    .map_err(WorkspaceChatApiError::Chat)
}

fn parse_privacy_data_class_label(
    data_class: &str,
) -> Result<oya_data_boundary_kernel::PrivacyDataClass, WorkspaceChatApiError> {
    let parsed = parse_data_class_label(data_class).ok_or_else(|| {
        WorkspaceChatApiError::InvalidDataClassLabel {
            data_class: data_class.to_string(),
        }
    })?;
    workspace_chat_data_class_from_legacy(parsed).map_err(|_| {
        WorkspaceChatApiError::InvalidDataClassLabel {
            data_class: data_class.to_string(),
        }
    })
}

fn channel_kind_from_label(kind: &str) -> Result<ChatChannelKind, WorkspaceChatApiError> {
    match kind.trim() {
        "direct_message" => Ok(ChatChannelKind::DirectMessage),
        "group" => Ok(ChatChannelKind::Group),
        "channel" => Ok(ChatChannelKind::Channel),
        _ => Err(WorkspaceChatApiError::InvalidChannelKind {
            kind: kind.to_string(),
        }),
    }
}

fn participant_role_from_label(role: &str) -> Result<ChatParticipantRole, WorkspaceChatApiError> {
    match role.trim() {
        "owner" => Ok(ChatParticipantRole::Owner),
        "admin" => Ok(ChatParticipantRole::Admin),
        "member" => Ok(ChatParticipantRole::Member),
        "bot" => Ok(ChatParticipantRole::Bot),
        _ => Err(WorkspaceChatApiError::InvalidParticipantRole {
            role: role.to_string(),
        }),
    }
}

fn sender_kind_from_label(sender_kind: &str) -> Result<ChatSenderKind, WorkspaceChatApiError> {
    match sender_kind.trim() {
        "human" => Ok(ChatSenderKind::Human),
        "bot" => Ok(ChatSenderKind::Bot),
        "system" => Ok(ChatSenderKind::System),
        _ => Err(WorkspaceChatApiError::InvalidSenderKind {
            sender_kind: sender_kind.to_string(),
        }),
    }
}

fn sender_kind_label(sender_kind: ChatSenderKind) -> &'static str {
    match sender_kind {
        ChatSenderKind::Human => "human",
        ChatSenderKind::Bot => "bot",
        ChatSenderKind::System => "system",
    }
}

fn require_sender(
    channel: &ChatChannel,
    principal_id: &str,
    sender_ref: &str,
    channel_id: &str,
) -> Result<(), WorkspaceChatApiError> {
    if principal_id == sender_ref
        && channel
            .participants
            .value
            .iter()
            .any(|participant| participant.actor_ref.value == sender_ref)
    {
        return Ok(());
    }
    Err(WorkspaceChatApiError::SenderPermissionDenied {
        principal_id: principal_id.to_string(),
        sender_ref: sender_ref.to_string(),
        channel_id: channel_id.to_string(),
    })
}

fn idempotency_key_for(
    boundary: &WorkspaceChatSendBoundaryContext,
    principal: &WorkspaceChatApiPrincipal,
    surface: &str,
) -> WorkspaceChatIdempotencyLedgerKey {
    WorkspaceChatIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn send_fingerprint_for(
    request: &WorkspaceChatMessageSendApiRequest,
) -> WorkspaceChatRequestFingerprint {
    let attachments = request
        .body
        .attachments
        .iter()
        .map(|attachment| {
            format!(
                "{}:{}:{}:{}",
                attachment.attachment_id,
                attachment.storage_key,
                attachment.mime_type,
                attachment.byte_len
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    WorkspaceChatRequestFingerprint {
        canonical: [
            format!("path.channel_id={}", request.path_channel_id),
            format!("path.message_id={}", request.path_message_id),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "authorization.tenant_id={}",
                request.authorization.tenant_id
            ),
            format!(
                "authorization.principal_id={}",
                request.authorization.principal_id
            ),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!(
                "authorization.allowed_surfaces={}",
                request.authorization.allowed_surfaces.join(",")
            ),
            format!("body.message_id={}", request.body.message_id),
            format!("body.channel_id={}", request.body.channel_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.thread_id={:?}", request.body.thread_id),
            format!(
                "body.parent_message_id={:?}",
                request.body.parent_message_id
            ),
            format!("body.sender_ref={}", request.body.sender_ref),
            format!("body.sender_kind={}", request.body.sender_kind),
            format!("body.body={:?}", request.body.body),
            format!("body.attachments={attachments}"),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn message_record(message: &ChatMessage) -> WorkspaceChatMessageRecord {
    WorkspaceChatMessageRecord {
        message_id: message.id.value.clone(),
        channel_id: message.channel_id.value.clone(),
        tenant_id: message.tenant_id.value.clone(),
        thread_id: message.thread_id.value.clone(),
        parent_message_id: message.parent_message_id.value.clone(),
        sender_ref: message.sender_ref.value.clone(),
        sender_kind: sender_kind_label(message.sender_kind.value).to_string(),
        body: message.body.value.clone(),
        attachments: message
            .attachments
            .value
            .iter()
            .map(|attachment| WorkspaceChatAttachmentRecord {
                attachment_id: attachment.attachment_id.value.clone(),
                storage_key: attachment.storage_key.value.clone(),
                mime_type: attachment.mime_type.value.clone(),
                byte_len: attachment.byte_len.value,
            })
            .collect(),
        data_class: message.privacy_data_class().label().to_string(),
        created_at_epoch_seconds: message.created_at_epoch_seconds.value,
        schema_version: message.schema_version.value,
    }
}

fn detail(field: &str, issue: &str) -> WorkspaceChatApiErrorDetail {
    WorkspaceChatApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

fn chat_error_message(error: &ChatError) -> &'static str {
    match error {
        ChatError::InvalidChannelId => "Workspace Chat channel id is invalid",
        ChatError::InvalidTenantId => "Workspace Chat tenant id is invalid",
        ChatError::InvalidRegion => "Workspace Chat region is invalid",
        ChatError::InvalidCellId => "Workspace Chat cell id is invalid",
        ChatError::InvalidChannelName => "Workspace Chat channel name is invalid",
        ChatError::InvalidParticipantRef => "Workspace Chat participant actor ref is invalid",
        ChatError::DuplicateParticipantRef => "Workspace Chat participants must be unique",
        ChatError::EmptyParticipantSet => "Workspace Chat participants are required",
        ChatError::InvalidDirectMessageParticipantCount => {
            "Workspace Chat direct messages require exactly two participants"
        }
        ChatError::InvalidGroupParticipantCount => {
            "Workspace Chat groups require at least three participants"
        }
        ChatError::MissingOwnerParticipant => {
            "Workspace Chat channel requires an owner participant"
        }
        ChatError::InvalidBotPrincipal => {
            "Workspace Chat bot principals must use the bot namespace"
        }
        ChatError::InvalidMessageId => "Workspace Chat message id is invalid",
        ChatError::InvalidThreadId => "Workspace Chat thread id is invalid",
        ChatError::InvalidParentMessageId => "Workspace Chat parent message id is invalid",
        ChatError::InvalidSenderRef => "Workspace Chat sender ref is invalid",
        ChatError::EmptyMessageBody => "Workspace Chat message body is invalid",
        ChatError::EmptyMessageContent => "Workspace Chat message content is required",
        ChatError::InvalidAttachmentId => "Workspace Chat attachment id is invalid",
        ChatError::InvalidAttachmentStorageKey => {
            "Workspace Chat attachment storage key is invalid"
        }
        ChatError::InvalidAttachmentMimeType => "Workspace Chat attachment mime type is invalid",
        ChatError::EmptyAttachment => "Workspace Chat attachments must contain bytes",
        ChatError::InvalidDataClass => "Workspace Chat data class is invalid",
        ChatError::InvalidThreadStateTransition => {
            "Workspace Chat thread state transition is invalid"
        }
        ChatError::CrossPillarSubscriptionDenied => {
            "Workspace Chat cross-pillar subscription is denied"
        }
    }
}

fn chat_error_issue(error: &ChatError) -> &'static str {
    match error {
        ChatError::InvalidChannelId => "channel id must be non-empty",
        ChatError::InvalidTenantId => "tenant id must be non-empty",
        ChatError::InvalidRegion => "region must be non-empty",
        ChatError::InvalidCellId => "cell id must be non-empty",
        ChatError::InvalidChannelName => "channel name must be trimmed and printable",
        ChatError::InvalidParticipantRef => "participant actor ref must be non-empty",
        ChatError::DuplicateParticipantRef => "participant actor refs must be unique",
        ChatError::EmptyParticipantSet => "participants must be non-empty",
        ChatError::InvalidDirectMessageParticipantCount => {
            "direct_message channels require exactly two participants"
        }
        ChatError::InvalidGroupParticipantCount => {
            "group channels require at least three participants"
        }
        ChatError::MissingOwnerParticipant => "channel participants must include an owner",
        ChatError::InvalidBotPrincipal => "bot participants and senders must start with bot:",
        ChatError::InvalidMessageId => "message id must be non-empty",
        ChatError::InvalidThreadId => "thread id must be present for replies and non-empty",
        ChatError::InvalidParentMessageId => "parent message id must be non-empty and not self",
        ChatError::InvalidSenderRef => "sender ref must be non-empty",
        ChatError::EmptyMessageBody => "body must be trimmed and contain no control characters",
        ChatError::EmptyMessageContent => "body or at least one attachment is required",
        ChatError::InvalidAttachmentId => "attachment id must be non-empty",
        ChatError::InvalidAttachmentStorageKey => "attachment storage key must be non-empty",
        ChatError::InvalidAttachmentMimeType => "attachment mime type must be non-empty",
        ChatError::EmptyAttachment => "attachment byte_len must be at least one",
        ChatError::InvalidDataClass => "data class must be a privacy data class",
        ChatError::InvalidThreadStateTransition => {
            "thread state transition must follow the allowed lifecycle"
        }
        ChatError::CrossPillarSubscriptionDenied => {
            "cross-pillar subscription is not permitted for this tenant"
        }
    }
}
