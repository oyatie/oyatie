//! Workspace Meet API boundary for session start.
//!
//! This crate owns REST-boundary normalization, idempotent session-start
//! handling, coarse authorization proof checks, and host-participant validation
//! around the Workspace Meet kernel. WebRTC signaling, SFU admission, recording
//! archive, and transcript streaming remain adapter concerns.

use std::collections::BTreeMap;

use comms_meet_domain::{
    MeetError, MeetSession, MeetSessionCreate, ParticipantConnectionState, ParticipantRef,
    ParticipantRole, RecordingConsentMode, workspace_meet_data_class_from_legacy,
};
use oya_data_boundary_kernel::parse_data_class_label;

pub const WORKSPACE_MEET_SESSION_START_SURFACE: &str = "workspace.meet.session.start";
pub const WORKSPACE_MEET_OPENAPI_CONTRACT: &str =
    "contracts/openapi/workspace/workspace-meet-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceMeetSessionStartApiStatus {
    Created,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl WorkspaceMeetSessionStartApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceMeetApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    SessionIdInvalid,
    SessionIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    HostPermissionDenied,
    IdempotencyKeyReused,
    DataClassInvalid,
    ParticipantRoleInvalid,
    ParticipantStateInvalid,
    RecordingConsentInvalid,
    SessionAlreadyExists,
    MeetInvalidRequest,
}

impl WorkspaceMeetApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "WORKSPACE_MEET_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "WORKSPACE_MEET_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "WORKSPACE_MEET_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "WORKSPACE_MEET_PRINCIPAL_ID_EMPTY",
            Self::SessionIdInvalid => "WORKSPACE_MEET_SESSION_ID_INVALID",
            Self::SessionIdMismatch => "WORKSPACE_MEET_SESSION_ID_MISMATCH",
            Self::TenantMismatch => "WORKSPACE_MEET_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "WORKSPACE_MEET_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "WORKSPACE_MEET_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "WORKSPACE_MEET_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "WORKSPACE_MEET_AUTHORIZATION_DENIED",
            Self::HostPermissionDenied => "WORKSPACE_MEET_HOST_PERMISSION_DENIED",
            Self::IdempotencyKeyReused => "WORKSPACE_MEET_IDEMPOTENCY_KEY_REUSED",
            Self::DataClassInvalid => "WORKSPACE_MEET_DATA_CLASS_INVALID",
            Self::ParticipantRoleInvalid => "WORKSPACE_MEET_PARTICIPANT_ROLE_INVALID",
            Self::ParticipantStateInvalid => "WORKSPACE_MEET_PARTICIPANT_STATE_INVALID",
            Self::RecordingConsentInvalid => "WORKSPACE_MEET_RECORDING_CONSENT_INVALID",
            Self::SessionAlreadyExists => "WORKSPACE_MEET_SESSION_ALREADY_EXISTS",
            Self::MeetInvalidRequest => "WORKSPACE_MEET_INVALID_REQUEST",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetStartBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: PII_IDENTIFYING
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetParticipantRequest {
    pub actor_ref: String,                    // data_class: PII_IDENTIFYING
    pub display_name: Option<String>,         // data_class: PII_QUASI_IDENTIFIER
    pub role: String,                         // data_class: INTERNAL_ONLY
    pub connection_state: String,             // data_class: INTERNAL_ONLY
    pub joined_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub left_at_epoch_seconds: Option<u64>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetSessionStartRequest {
    pub session_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: INTERNAL_ONLY
    pub cell_id: String,               // data_class: INTERNAL_ONLY
    pub sfu_pool_id: String,           // data_class: INTERNAL_ONLY
    pub data_class: String,            // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub participants: Vec<WorkspaceMeetParticipantRequest>, // data_class: PII_IDENTIFYING
    pub recording_consent: String,     // data_class: INTERNAL_ONLY
    pub transcript_session_id: Option<String>, // data_class: INTERNAL_ONLY
    pub summary_id: Option<String>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetSessionStartApiRequest {
    pub path_session_id: String, // data_class: INTERNAL_ONLY
    pub boundary: WorkspaceMeetStartBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: WorkspaceMeetApiPrincipal, // data_class: PII_IDENTIFYING
    pub authorization: WorkspaceMeetApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: WorkspaceMeetSessionStartRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceMeetSessionDirectory {
    sessions: BTreeMap<WorkspaceMeetSessionKey, MeetSession>, // data_class: INTERNAL_ONLY
}

impl WorkspaceMeetSessionDirectory {
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn sessions(&self) -> impl Iterator<Item = &MeetSession> {
        self.sessions.values()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceMeetSessionKey {
    tenant_id: String,  // data_class: INTERNAL_ONLY
    session_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceMeetSessionStartIdempotencyLedger {
    entries: BTreeMap<WorkspaceMeetIdempotencyLedgerKey, WorkspaceMeetStartLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl WorkspaceMeetSessionStartIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceMeetIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: PII_IDENTIFYING
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceMeetStartLedgerEntry {
    fingerprint: WorkspaceMeetRequestFingerprint, // data_class: INTERNAL_ONLY
    result: WorkspaceMeetSessionStartSuccessResponse, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceMeetRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetSessionStartSuccessResponse {
    pub data: WorkspaceMeetSessionRecord, // data_class: INTERNAL_ONLY
    pub metadata: WorkspaceMeetSessionMetadata, // data_class: INTERNAL_ONLY
}

impl WorkspaceMeetSessionStartSuccessResponse {
    pub fn created(data: WorkspaceMeetSessionRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: WorkspaceMeetSessionMetadata {
                request_id: request_id.into(),
                surface: WORKSPACE_MEET_SESSION_START_SURFACE.to_string(),
                openapi_contract: WORKSPACE_MEET_OPENAPI_CONTRACT.to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetSessionMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetParticipantRecord {
    pub actor_ref: String,                    // data_class: PII_IDENTIFYING
    pub display_name: Option<String>,         // data_class: PII_QUASI_IDENTIFIER
    pub role: String,                         // data_class: INTERNAL_ONLY
    pub connection_state: String,             // data_class: INTERNAL_ONLY
    pub joined_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub left_at_epoch_seconds: Option<u64>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetSessionRecord {
    pub session_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: INTERNAL_ONLY
    pub cell_id: String,                     // data_class: INTERNAL_ONLY
    pub sfu_pool_id: String,                 // data_class: INTERNAL_ONLY
    pub data_class: String,                  // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub ended_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub participants: Vec<WorkspaceMeetParticipantRecord>, // data_class: PII_IDENTIFYING
    pub recording_consent: String,           // data_class: INTERNAL_ONLY
    pub transcript_session_id: Option<String>, // data_class: INTERNAL_ONLY
    pub summary_id: Option<String>,          // data_class: INTERNAL_ONLY
    pub schema_version: u32,                 // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetApiErrorResponse {
    pub error: WorkspaceMeetApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetApiErrorBody {
    pub code: String,                              // data_class: INTERNAL_ONLY
    pub message: String,                           // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,         // data_class: INTERNAL_ONLY
    pub request_id: String,                        // data_class: INTERNAL_ONLY
    pub details: Vec<WorkspaceMeetApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMeetApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceMeetApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    InvalidSessionId {
        session_id: String,
    },
    SessionIdMismatch {
        path_session_id: String,
        body_session_id: String,
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
    HostPermissionDenied {
        principal_id: String,
        session_id: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    InvalidParticipantRole {
        role: String,
    },
    InvalidParticipantState {
        connection_state: String,
    },
    InvalidRecordingConsent {
        recording_consent: String,
    },
    SessionAlreadyExists {
        tenant_id: String,
        session_id: String,
    },
    Meet(MeetError),
}

impl WorkspaceMeetApiError {
    pub fn session_status_code(&self) -> u16 {
        match self.status_kind() {
            WorkspaceMeetApiStatusKind::BadRequest => 400,
            WorkspaceMeetApiStatusKind::Forbidden => 403,
            WorkspaceMeetApiStatusKind::Conflict => 409,
            WorkspaceMeetApiStatusKind::UnprocessableEntity => 422,
        }
    }

    pub fn code(&self) -> WorkspaceMeetApiErrorCode {
        match self {
            Self::EmptyRequestId => WorkspaceMeetApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => WorkspaceMeetApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => WorkspaceMeetApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => WorkspaceMeetApiErrorCode::PrincipalIdEmpty,
            Self::InvalidSessionId { .. } => WorkspaceMeetApiErrorCode::SessionIdInvalid,
            Self::SessionIdMismatch { .. } => WorkspaceMeetApiErrorCode::SessionIdMismatch,
            Self::TenantMismatch { .. } => WorkspaceMeetApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                WorkspaceMeetApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                WorkspaceMeetApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                WorkspaceMeetApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => WorkspaceMeetApiErrorCode::AuthorizationDenied,
            Self::HostPermissionDenied { .. } => WorkspaceMeetApiErrorCode::HostPermissionDenied,
            Self::IdempotencyKeyReused { .. } => WorkspaceMeetApiErrorCode::IdempotencyKeyReused,
            Self::InvalidDataClassLabel { .. } => WorkspaceMeetApiErrorCode::DataClassInvalid,
            Self::InvalidParticipantRole { .. } => {
                WorkspaceMeetApiErrorCode::ParticipantRoleInvalid
            }
            Self::InvalidParticipantState { .. } => {
                WorkspaceMeetApiErrorCode::ParticipantStateInvalid
            }
            Self::InvalidRecordingConsent { .. } => {
                WorkspaceMeetApiErrorCode::RecordingConsentInvalid
            }
            Self::SessionAlreadyExists { .. } => WorkspaceMeetApiErrorCode::SessionAlreadyExists,
            Self::Meet(_) => WorkspaceMeetApiErrorCode::MeetInvalidRequest,
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> WorkspaceMeetApiErrorResponse {
        WorkspaceMeetApiErrorResponse {
            error: WorkspaceMeetApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> WorkspaceMeetApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::HostPermissionDenied { .. } => WorkspaceMeetApiStatusKind::Forbidden,
            Self::SessionAlreadyExists { .. } => WorkspaceMeetApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => WorkspaceMeetApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::InvalidSessionId { .. }
            | Self::SessionIdMismatch { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidParticipantRole { .. }
            | Self::InvalidParticipantState { .. }
            | Self::InvalidRecordingConsent { .. }
            | Self::Meet(_) => WorkspaceMeetApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::InvalidSessionId { .. } => "Workspace Meet session id is required",
            Self::SessionIdMismatch { .. } => "Path and body session ids must match",
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
                "Authorization decision does not allow the Workspace Meet session-start surface"
            }
            Self::HostPermissionDenied { .. } => {
                "Starting principal must be present as a host participant"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::InvalidParticipantRole { .. } => "Participant role must be known",
            Self::InvalidParticipantState { .. } => "Participant connection state must be known",
            Self::InvalidRecordingConsent { .. } => "Recording consent mode must be known",
            Self::SessionAlreadyExists { .. } => "Workspace Meet session already exists",
            Self::Meet(error) => meet_error_message(error),
        }
    }

    fn details(&self) -> Vec<WorkspaceMeetApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::InvalidSessionId { .. } => {
                vec![detail("path.session_id", "must be non-empty")]
            }
            Self::SessionIdMismatch { .. } => vec![detail(
                "session_id",
                "path session_id and body session_id must match",
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
                "must include workspace.meet.session.start",
            )],
            Self::HostPermissionDenied { .. } => vec![detail(
                "participants",
                "principal must appear as a host participant before the session can start",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::InvalidParticipantRole { .. } => vec![detail(
                "body.participants.role",
                "must be one of host, co_host, presenter, attendee",
            )],
            Self::InvalidParticipantState { .. } => vec![detail(
                "body.participants.connection_state",
                "must be one of invited, joined, left, removed",
            )],
            Self::InvalidRecordingConsent { .. } => vec![detail(
                "body.recording_consent",
                "must be one of not_requested, participant_opt_in, tenant_policy_default_on",
            )],
            Self::SessionAlreadyExists { .. } => vec![detail(
                "path.session_id",
                "session metadata already exists for the requested tenant",
            )],
            Self::Meet(error) => vec![detail("workspace_meet", meet_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceMeetApiStatusKind {
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_workspace_meet_session_start_request(
    request: &WorkspaceMeetSessionStartApiRequest,
) -> Result<(), WorkspaceMeetApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_session_id(&request.path_session_id)?;
    validate_path_body_binding(&request.path_session_id, &request.body.session_id)?;
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
        WORKSPACE_MEET_SESSION_START_SURFACE,
    )?;
    Ok(())
}

pub fn start_workspace_meet_session_from_api(
    directory: &mut WorkspaceMeetSessionDirectory,
    idempotency_ledger: &mut WorkspaceMeetSessionStartIdempotencyLedger,
    request: WorkspaceMeetSessionStartApiRequest,
) -> Result<WorkspaceMeetSessionStartSuccessResponse, WorkspaceMeetApiError> {
    validate_workspace_meet_session_start_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        WORKSPACE_MEET_SESSION_START_SURFACE,
    );
    let fingerprint = start_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(WorkspaceMeetApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let principal_id = request.principal.principal_id.clone();
    let session_id = request.body.session_id.clone();
    let session = meet_session_from_request(request.body)?;
    require_host(&session, &principal_id, &session_id)?;
    let directory_key = WorkspaceMeetSessionKey {
        tenant_id: session.tenant_id.value.clone(),
        session_id: session.id.value.clone(),
    };
    if directory.sessions.contains_key(&directory_key) {
        return Err(WorkspaceMeetApiError::SessionAlreadyExists {
            tenant_id: directory_key.tenant_id,
            session_id: directory_key.session_id,
        });
    }

    let response =
        WorkspaceMeetSessionStartSuccessResponse::created(session_record(&session), request_id);
    directory.sessions.insert(directory_key, session);
    idempotency_ledger.entries.insert(
        key,
        WorkspaceMeetStartLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(
    boundary: &WorkspaceMeetStartBoundaryContext,
) -> Result<(), WorkspaceMeetApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(WorkspaceMeetApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(WorkspaceMeetApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(WorkspaceMeetApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_session_id(session_id: &str) -> Result<(), WorkspaceMeetApiError> {
    if session_id.trim().is_empty() {
        return Err(WorkspaceMeetApiError::InvalidSessionId {
            session_id: session_id.to_string(),
        });
    }
    Ok(())
}

fn validate_path_body_binding(
    path_session_id: &str,
    body_session_id: &str,
) -> Result<(), WorkspaceMeetApiError> {
    if path_session_id != body_session_id {
        return Err(WorkspaceMeetApiError::SessionIdMismatch {
            path_session_id: path_session_id.to_string(),
            body_session_id: body_session_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    header_tenant_id: &str,
    principal: &WorkspaceMeetApiPrincipal,
    authorization: &WorkspaceMeetApiAuthorization,
    body_tenant_id: Option<&str>,
    resource_tenant_id: Option<&str>,
) -> Result<(), WorkspaceMeetApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(WorkspaceMeetApiError::EmptyPrincipalId);
    }
    if header_tenant_id != principal.tenant_id
        || header_tenant_id != authorization.tenant_id
        || body_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id)
        || resource_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id)
    {
        return Err(WorkspaceMeetApiError::TenantMismatch {
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
    principal: &WorkspaceMeetApiPrincipal,
    authorization: &WorkspaceMeetApiAuthorization,
    surface: &str,
) -> Result<(), WorkspaceMeetApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(WorkspaceMeetApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(WorkspaceMeetApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(WorkspaceMeetApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed| allowed == surface)
    {
        return Err(WorkspaceMeetApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn meet_session_from_request(
    request: WorkspaceMeetSessionStartRequest,
) -> Result<MeetSession, WorkspaceMeetApiError> {
    let data_class = parse_data_class_label(&request.data_class).ok_or_else(|| {
        WorkspaceMeetApiError::InvalidDataClassLabel {
            data_class: request.data_class.clone(),
        }
    })?;
    let data_class = workspace_meet_data_class_from_legacy(data_class).map_err(|_| {
        WorkspaceMeetApiError::InvalidDataClassLabel {
            data_class: request.data_class.clone(),
        }
    })?;
    let participants = request
        .participants
        .into_iter()
        .map(participant_from_request)
        .collect::<Result<Vec<_>, _>>()?;
    MeetSession::new(MeetSessionCreate {
        id: request.session_id,
        tenant_id: request.tenant_id,
        region: request.region,
        cell_id: request.cell_id,
        sfu_pool_id: request.sfu_pool_id,
        data_class: Some(data_class),
        started_at_epoch_seconds: request.started_at_epoch_seconds,
        ended_at_epoch_seconds: None,
        participants,
        recording: None,
        recording_consent: recording_consent_from_label(&request.recording_consent)?,
        transcript_session_id: request.transcript_session_id,
        summary_id: request.summary_id,
    })
    .map_err(WorkspaceMeetApiError::Meet)
}

fn participant_from_request(
    participant: WorkspaceMeetParticipantRequest,
) -> Result<ParticipantRef, WorkspaceMeetApiError> {
    ParticipantRef::new(
        participant.actor_ref,
        participant.display_name,
        participant_role_from_label(&participant.role)?,
        participant_state_from_label(&participant.connection_state)?,
        participant.joined_at_epoch_seconds,
        participant.left_at_epoch_seconds,
    )
    .map_err(WorkspaceMeetApiError::Meet)
}

fn participant_role_from_label(role: &str) -> Result<ParticipantRole, WorkspaceMeetApiError> {
    match role.trim() {
        "host" => Ok(ParticipantRole::Host),
        "co_host" => Ok(ParticipantRole::CoHost),
        "presenter" => Ok(ParticipantRole::Presenter),
        "attendee" => Ok(ParticipantRole::Attendee),
        _ => Err(WorkspaceMeetApiError::InvalidParticipantRole {
            role: role.to_string(),
        }),
    }
}

fn participant_role_label(role: ParticipantRole) -> &'static str {
    match role {
        ParticipantRole::Host => "host",
        ParticipantRole::CoHost => "co_host",
        ParticipantRole::Presenter => "presenter",
        ParticipantRole::Attendee => "attendee",
    }
}

fn participant_state_from_label(
    connection_state: &str,
) -> Result<ParticipantConnectionState, WorkspaceMeetApiError> {
    match connection_state.trim() {
        "invited" => Ok(ParticipantConnectionState::Invited),
        "joined" => Ok(ParticipantConnectionState::Joined),
        "left" => Ok(ParticipantConnectionState::Left),
        "removed" => Ok(ParticipantConnectionState::Removed),
        _ => Err(WorkspaceMeetApiError::InvalidParticipantState {
            connection_state: connection_state.to_string(),
        }),
    }
}

fn participant_state_label(state: ParticipantConnectionState) -> &'static str {
    match state {
        ParticipantConnectionState::Invited => "invited",
        ParticipantConnectionState::Joined => "joined",
        ParticipantConnectionState::Left => "left",
        ParticipantConnectionState::Removed => "removed",
    }
}

fn recording_consent_from_label(
    recording_consent: &str,
) -> Result<RecordingConsentMode, WorkspaceMeetApiError> {
    match recording_consent.trim() {
        "not_requested" => Ok(RecordingConsentMode::NotRequested),
        "participant_opt_in" => Ok(RecordingConsentMode::ParticipantOptIn),
        "tenant_policy_default_on" => Ok(RecordingConsentMode::TenantPolicyDefaultOn),
        _ => Err(WorkspaceMeetApiError::InvalidRecordingConsent {
            recording_consent: recording_consent.to_string(),
        }),
    }
}

fn recording_consent_label(recording_consent: RecordingConsentMode) -> &'static str {
    match recording_consent {
        RecordingConsentMode::NotRequested => "not_requested",
        RecordingConsentMode::ParticipantOptIn => "participant_opt_in",
        RecordingConsentMode::TenantPolicyDefaultOn => "tenant_policy_default_on",
    }
}

fn require_host(
    session: &MeetSession,
    principal_id: &str,
    session_id: &str,
) -> Result<(), WorkspaceMeetApiError> {
    if session.participants.value.iter().any(|participant| {
        participant.actor_ref.value == principal_id
            && participant.role.value == ParticipantRole::Host
    }) {
        return Ok(());
    }
    Err(WorkspaceMeetApiError::HostPermissionDenied {
        principal_id: principal_id.to_string(),
        session_id: session_id.to_string(),
    })
}

fn idempotency_key_for(
    boundary: &WorkspaceMeetStartBoundaryContext,
    principal: &WorkspaceMeetApiPrincipal,
    surface: &str,
) -> WorkspaceMeetIdempotencyLedgerKey {
    WorkspaceMeetIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn start_fingerprint_for(
    request: &WorkspaceMeetSessionStartApiRequest,
) -> WorkspaceMeetRequestFingerprint {
    let participants = request
        .body
        .participants
        .iter()
        .map(|participant| {
            format!(
                "{}:{:?}:{}:{}:{:?}:{:?}",
                participant.actor_ref,
                participant.display_name,
                participant.role,
                participant.connection_state,
                participant.joined_at_epoch_seconds,
                participant.left_at_epoch_seconds
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    WorkspaceMeetRequestFingerprint {
        canonical: [
            format!("path.session_id={}", request.path_session_id),
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
            format!("body.session_id={}", request.body.session_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.region={}", request.body.region),
            format!("body.cell_id={}", request.body.cell_id),
            format!("body.sfu_pool_id={}", request.body.sfu_pool_id),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.started_at_epoch_seconds={}",
                request.body.started_at_epoch_seconds
            ),
            format!("body.participants={participants}"),
            format!("body.recording_consent={}", request.body.recording_consent),
            format!(
                "body.transcript_session_id={:?}",
                request.body.transcript_session_id
            ),
            format!("body.summary_id={:?}", request.body.summary_id),
        ]
        .join("|"),
    }
}

fn session_record(session: &MeetSession) -> WorkspaceMeetSessionRecord {
    WorkspaceMeetSessionRecord {
        session_id: session.id.value.clone(),
        tenant_id: session.tenant_id.value.clone(),
        region: session.region.value.clone(),
        cell_id: session.cell_id.value.clone(),
        sfu_pool_id: session.sfu_pool_id.value.clone(),
        data_class: session.privacy_data_class().label().to_string(),
        started_at_epoch_seconds: session.started_at_epoch_seconds.value,
        ended_at_epoch_seconds: session.ended_at_epoch_seconds.value,
        participants: session
            .participants
            .value
            .iter()
            .map(|participant| WorkspaceMeetParticipantRecord {
                actor_ref: participant.actor_ref.value.clone(),
                display_name: participant.display_name.value.clone(),
                role: participant_role_label(participant.role.value).to_string(),
                connection_state: participant_state_label(participant.connection_state.value)
                    .to_string(),
                joined_at_epoch_seconds: participant.joined_at_epoch_seconds.value,
                left_at_epoch_seconds: participant.left_at_epoch_seconds.value,
            })
            .collect(),
        recording_consent: recording_consent_label(session.recording_consent.value).to_string(),
        transcript_session_id: session.transcript_session_id.value.clone(),
        summary_id: session.summary_id.value.clone(),
        schema_version: session.schema_version.value,
    }
}

fn detail(field: &str, issue: &str) -> WorkspaceMeetApiErrorDetail {
    WorkspaceMeetApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

fn meet_error_message(error: &MeetError) -> &'static str {
    match error {
        MeetError::InvalidSessionId => "Workspace Meet session id is invalid",
        MeetError::InvalidTenantId => "Workspace Meet tenant id is invalid",
        MeetError::InvalidRegion => "Workspace Meet region is invalid",
        MeetError::InvalidCellId => "Workspace Meet cell id is invalid",
        MeetError::InvalidSfuPoolId => "Workspace Meet SFU pool id is invalid",
        MeetError::EmptyParticipantSet => "Workspace Meet participants are required",
        MeetError::MissingHostParticipant => "Workspace Meet session requires a host participant",
        MeetError::InvalidParticipantRef => "Workspace Meet participant actor ref is invalid",
        MeetError::InvalidParticipantDisplayName => {
            "Workspace Meet participant display name is invalid"
        }
        MeetError::InvalidParticipantTimeOrder => {
            "Workspace Meet participant time order is invalid"
        }
        MeetError::InvalidSessionTimeOrder => "Workspace Meet session time order is invalid",
        MeetError::InvalidRecordingId => "Workspace Meet recording id is invalid",
        MeetError::InvalidRecordingStorageKey => "Workspace Meet recording storage key is invalid",
        MeetError::InvalidKmsShredKeyId => "Workspace Meet KMS shred key id is invalid",
        MeetError::InvalidRetentionPolicyId => "Workspace Meet retention policy id is invalid",
        MeetError::EmptyCompletedRecording => "Workspace Meet completed recording is empty",
        MeetError::InvalidRecordingTimeOrder => "Workspace Meet recording time order is invalid",
        MeetError::MissingRecordingConsent => "Workspace Meet recording consent is required",
        MeetError::InvalidTranscriptSessionId => "Workspace Meet transcript session id is invalid",
        MeetError::InvalidSummaryId => "Workspace Meet summary id is invalid",
        MeetError::InvalidTranscriptText => "Workspace Meet transcript text is invalid",
        MeetError::InvalidTranscriptTimeOrder => "Workspace Meet transcript time order is invalid",
        MeetError::InvalidDataClass => "Workspace Meet data class is invalid",
    }
}

fn meet_error_issue(error: &MeetError) -> &'static str {
    match error {
        MeetError::InvalidSessionId => "session_id must be non-empty",
        MeetError::InvalidTenantId => "tenant_id must be non-empty",
        MeetError::InvalidRegion => "region must be non-empty",
        MeetError::InvalidCellId => "cell_id must be non-empty",
        MeetError::InvalidSfuPoolId => "sfu_pool_id must be non-empty",
        MeetError::EmptyParticipantSet => "participants must contain at least one participant",
        MeetError::MissingHostParticipant => "participants must contain a host",
        MeetError::InvalidParticipantRef => "participant actor_ref must be non-empty",
        MeetError::InvalidParticipantDisplayName => {
            "participant display_name must not be empty, padded, or contain control bytes"
        }
        MeetError::InvalidParticipantTimeOrder => "participant left_at must not precede joined_at",
        MeetError::InvalidSessionTimeOrder => "session ended_at must not precede started_at",
        MeetError::InvalidRecordingId => "recording_id must be non-empty",
        MeetError::InvalidRecordingStorageKey => "archive_storage_key must be non-empty",
        MeetError::InvalidKmsShredKeyId => "kms_shred_key_id must be non-empty",
        MeetError::InvalidRetentionPolicyId => "retention_policy_id must be non-empty",
        MeetError::EmptyCompletedRecording => "completed recording byte_len must be non-zero",
        MeetError::InvalidRecordingTimeOrder => "recording ended_at must not precede started_at",
        MeetError::MissingRecordingConsent => {
            "recording metadata requires participant or tenant policy consent"
        }
        MeetError::InvalidTranscriptSessionId => "transcript_session_id must be non-empty",
        MeetError::InvalidSummaryId => "summary_id must be non-empty",
        MeetError::InvalidTranscriptText => "transcript text must be non-empty",
        MeetError::InvalidTranscriptTimeOrder => "transcript ended_at must not precede started_at",
        MeetError::InvalidDataClass => "data_class must be a privacy-program data class",
    }
}
