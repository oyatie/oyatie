//! Platform Identity app boundary.
//!
//! This crate owns authenticated REST-boundary normalization, purpose parsing,
//! request fingerprint idempotency, and short-lived STS credential projection for
//! `identity.token.issue` before handing typed issuance to the platform identity
//! kernel.

pub mod observability;

use std::collections::BTreeMap;

use data_boundary_kernel::{Purpose, parse_purpose_pascal_label};
use iam_identity_domain::{
    CredentialRequest, CredentialRequestKind, IdentityError, Principal, issue_credential,
};

pub const IDENTITY_TOKEN_ISSUE_SURFACE: &str = "identity.token.issue";
pub const IDENTITY_TOKEN_ISSUE_OPENAPI_CONTRACT: &str =
    "contracts/openapi/platform/platform-identity-token-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityTokenIssueApiStatus {
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

impl IdentityTokenIssueApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityTokenIssueApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PrincipalKindEmpty,
    PrincipalKindInvalid,
    SubjectIdEmpty,
    SubjectKindInvalid,
    TenantMismatch,
    PrincipalMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    CredentialKindEmpty,
    CredentialKindInvalid,
    PurposeInvalid,
    PreviousTokenFingerprintEmpty,
    PreviousTokenNotYetActive,
    PreviousTokenExpired,
    RotationBindingMismatch,
    RotationPurposeScopeMismatch,
    IdempotencyKeyReused,
    IdentityInvalidTenant,
    IdentityInvalidUser,
    IdentityInvalidServicePrincipal,
    IdentityInvalidCapability,
    IdentityTokenTtlTooLong,
    IdentityTokenTtlZero,
    IdentityMissingScope,
    IdentityLongLivedForbidden,
}

impl IdentityTokenIssueApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "IDENTITY_TOKEN_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "IDENTITY_TOKEN_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "IDENTITY_TOKEN_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "IDENTITY_TOKEN_PRINCIPAL_ID_EMPTY",
            Self::PrincipalKindEmpty => "IDENTITY_TOKEN_PRINCIPAL_KIND_EMPTY",
            Self::PrincipalKindInvalid => "IDENTITY_TOKEN_PRINCIPAL_KIND_INVALID",
            Self::SubjectIdEmpty => "IDENTITY_TOKEN_SUBJECT_ID_EMPTY",
            Self::SubjectKindInvalid => "IDENTITY_TOKEN_SUBJECT_KIND_INVALID",
            Self::TenantMismatch => "IDENTITY_TOKEN_TENANT_MISMATCH",
            Self::PrincipalMismatch => "IDENTITY_TOKEN_PRINCIPAL_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "IDENTITY_TOKEN_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "IDENTITY_TOKEN_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "IDENTITY_TOKEN_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "IDENTITY_TOKEN_AUTHORIZATION_DENIED",
            Self::CredentialKindEmpty => "IDENTITY_TOKEN_CREDENTIAL_KIND_EMPTY",
            Self::CredentialKindInvalid => "IDENTITY_TOKEN_CREDENTIAL_KIND_INVALID",
            Self::PurposeInvalid => "IDENTITY_TOKEN_PURPOSE_INVALID",
            Self::PreviousTokenFingerprintEmpty => "IDENTITY_TOKEN_PREVIOUS_FINGERPRINT_EMPTY",
            Self::PreviousTokenNotYetActive => "IDENTITY_TOKEN_PREVIOUS_TOKEN_NOT_YET_ACTIVE",
            Self::PreviousTokenExpired => "IDENTITY_TOKEN_PREVIOUS_TOKEN_EXPIRED",
            Self::RotationBindingMismatch => "IDENTITY_TOKEN_ROTATION_BINDING_MISMATCH",
            Self::RotationPurposeScopeMismatch => "IDENTITY_TOKEN_ROTATION_PURPOSE_SCOPE_MISMATCH",
            Self::IdempotencyKeyReused => "IDENTITY_TOKEN_IDEMPOTENCY_KEY_REUSED",
            Self::IdentityInvalidTenant => "IDENTITY_TOKEN_IDENTITY_INVALID_TENANT",
            Self::IdentityInvalidUser => "IDENTITY_TOKEN_IDENTITY_INVALID_USER",
            Self::IdentityInvalidServicePrincipal => {
                "IDENTITY_TOKEN_IDENTITY_INVALID_SERVICE_PRINCIPAL"
            }
            Self::IdentityInvalidCapability => "IDENTITY_TOKEN_IDENTITY_INVALID_CAPABILITY",
            Self::IdentityTokenTtlTooLong => "IDENTITY_TOKEN_TTL_TOO_LONG",
            Self::IdentityTokenTtlZero => "IDENTITY_TOKEN_TTL_ZERO",
            Self::IdentityMissingScope => "IDENTITY_TOKEN_SCOPE_MISSING",
            Self::IdentityLongLivedForbidden => "IDENTITY_TOKEN_LONG_LIVED_FORBIDDEN",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityApiPrincipal {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub principal_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_kind: String,               // data_class: INTERNAL_ONLY
    pub owning_capability_id: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityScopeRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurposeScope {
    pub purpose: String,               // data_class: INTERNAL_ONLY
    pub scopes: Vec<IdentityScopeRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueRequest {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub subject_id: String,                   // data_class: INTERNAL_ONLY
    pub subject_kind: String,                 // data_class: INTERNAL_ONLY
    pub owning_capability_id: Option<String>, // data_class: INTERNAL_ONLY
    pub credential_kind: String,              // data_class: INTERNAL_ONLY
    pub purpose: String,                      // data_class: INTERNAL_ONLY
    pub ttl_seconds: u64,                     // data_class: INTERNAL_ONLY
    pub scopes: Vec<IdentityScopeRef>,        // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueApiRequest {
    pub boundary: IdentityApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: IdentityApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: IdentityApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: IdentityTokenIssueRequest,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenRotationRequest {
    pub previous: IdentityTokenRecord, // data_class: INTERNAL_ONLY
    pub replacement: IdentityTokenIssueApiRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IdentityTokenIssueIdempotencyLedger {
    entries:
        BTreeMap<IdentityTokenIssueIdempotencyLedgerKey, IdentityTokenIssueIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl IdentityTokenIssueIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct IdentityTokenIssueIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IdentityTokenIssueIdempotencyLedgerEntry {
    fingerprint: IdentityTokenIssueRequestFingerprint, // data_class: INTERNAL_ONLY
    result: IdentityTokenIssueSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IdentityTokenIssueRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueSuccessResponse {
    pub data: IdentityTokenRecord,            // data_class: INTERNAL_ONLY
    pub metadata: IdentityTokenIssueMetadata, // data_class: INTERNAL_ONLY
}

impl IdentityTokenIssueSuccessResponse {
    pub fn ok(data: IdentityTokenRecord, request: &IdentityTokenIssueApiRequest) -> Self {
        Self {
            data,
            metadata: IdentityTokenIssueMetadata {
                request_id: request.boundary.request_id.clone(),
                tenant_id: request.boundary.tenant_id.clone(),
                principal_id: request.principal.principal_id.clone(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueMetadata {
    pub request_id: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenRecord {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub subject_id: String,                   // data_class: INTERNAL_ONLY
    pub subject_kind: String,                 // data_class: INTERNAL_ONLY
    pub owning_capability_id: Option<String>, // data_class: INTERNAL_ONLY
    pub credential_kind: String,              // data_class: INTERNAL_ONLY
    pub purpose: String,                      // data_class: INTERNAL_ONLY
    pub scopes: Vec<IdentityScopeRef>,        // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub token_fingerprint: String,            // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueApiErrorResponse {
    pub error: IdentityTokenIssueApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<IdentityTokenIssueApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityTokenIssueApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityTokenIssueApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPrincipalKind,
    InvalidPrincipalKind {
        principal_kind: String,
    },
    EmptySubjectId,
    InvalidSubjectKind {
        subject_kind: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        body_tenant_id: String,
    },
    PrincipalMismatch {
        principal_id: String,
        subject_id: String,
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
    EmptyCredentialKind,
    InvalidCredentialKind {
        credential_kind: String,
    },
    InvalidPurpose {
        purpose: String,
    },
    EmptyPreviousTokenFingerprint,
    PreviousTokenNotYetActive {
        previous_issued_at_epoch_seconds: u64,
        rotate_at_epoch_seconds: u64,
    },
    PreviousTokenExpired {
        previous_expires_at_epoch_seconds: u64,
        rotate_at_epoch_seconds: u64,
    },
    RotationBindingMismatch {
        previous_subject_id: String,
        replacement_subject_id: String,
    },
    RotationPurposeScopeMismatch,
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Identity(IdentityError),
}

impl IdentityTokenIssueApiError {
    pub fn identity_token_issue_status(&self) -> IdentityTokenIssueApiStatus {
        match self.status_kind() {
            IdentityTokenIssueApiStatusKind::BadRequest => IdentityTokenIssueApiStatus::BadRequest,
            IdentityTokenIssueApiStatusKind::Unauthorized => {
                IdentityTokenIssueApiStatus::Unauthorized
            }
            IdentityTokenIssueApiStatusKind::Forbidden => IdentityTokenIssueApiStatus::Forbidden,
            IdentityTokenIssueApiStatusKind::UnprocessableEntity => {
                IdentityTokenIssueApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn identity_token_issue_status_code(&self) -> u16 {
        self.identity_token_issue_status().code()
    }

    pub fn code(&self) -> IdentityTokenIssueApiErrorCode {
        match self {
            Self::EmptyRequestId => IdentityTokenIssueApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => IdentityTokenIssueApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => IdentityTokenIssueApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => IdentityTokenIssueApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPrincipalKind => IdentityTokenIssueApiErrorCode::PrincipalKindEmpty,
            Self::InvalidPrincipalKind { .. } => {
                IdentityTokenIssueApiErrorCode::PrincipalKindInvalid
            }
            Self::EmptySubjectId => IdentityTokenIssueApiErrorCode::SubjectIdEmpty,
            Self::InvalidSubjectKind { .. } => IdentityTokenIssueApiErrorCode::SubjectKindInvalid,
            Self::TenantMismatch { .. } => IdentityTokenIssueApiErrorCode::TenantMismatch,
            Self::PrincipalMismatch { .. } => IdentityTokenIssueApiErrorCode::PrincipalMismatch,
            Self::EmptyAuthorizationDecisionId => {
                IdentityTokenIssueApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                IdentityTokenIssueApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                IdentityTokenIssueApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => IdentityTokenIssueApiErrorCode::AuthorizationDenied,
            Self::EmptyCredentialKind => IdentityTokenIssueApiErrorCode::CredentialKindEmpty,
            Self::InvalidCredentialKind { .. } => {
                IdentityTokenIssueApiErrorCode::CredentialKindInvalid
            }
            Self::InvalidPurpose { .. } => IdentityTokenIssueApiErrorCode::PurposeInvalid,
            Self::EmptyPreviousTokenFingerprint => {
                IdentityTokenIssueApiErrorCode::PreviousTokenFingerprintEmpty
            }
            Self::PreviousTokenNotYetActive { .. } => {
                IdentityTokenIssueApiErrorCode::PreviousTokenNotYetActive
            }
            Self::PreviousTokenExpired { .. } => {
                IdentityTokenIssueApiErrorCode::PreviousTokenExpired
            }
            Self::RotationBindingMismatch { .. } => {
                IdentityTokenIssueApiErrorCode::RotationBindingMismatch
            }
            Self::RotationPurposeScopeMismatch => {
                IdentityTokenIssueApiErrorCode::RotationPurposeScopeMismatch
            }
            Self::IdempotencyKeyReused { .. } => {
                IdentityTokenIssueApiErrorCode::IdempotencyKeyReused
            }
            Self::Identity(error) => identity_error_code(error),
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> IdentityTokenIssueApiErrorResponse {
        IdentityTokenIssueApiErrorResponse {
            error: IdentityTokenIssueApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> IdentityTokenIssueApiStatusKind {
        match self {
            Self::EmptyPrincipalId => IdentityTokenIssueApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::PrincipalMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => IdentityTokenIssueApiStatusKind::Forbidden,
            Self::PreviousTokenNotYetActive { .. } => IdentityTokenIssueApiStatusKind::Forbidden,
            Self::PreviousTokenExpired { .. } => IdentityTokenIssueApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => {
                IdentityTokenIssueApiStatusKind::UnprocessableEntity
            }
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPrincipalKind
            | Self::InvalidPrincipalKind { .. }
            | Self::EmptySubjectId
            | Self::InvalidSubjectKind { .. }
            | Self::EmptyCredentialKind
            | Self::InvalidCredentialKind { .. }
            | Self::InvalidPurpose { .. }
            | Self::EmptyPreviousTokenFingerprint
            | Self::RotationBindingMismatch { .. }
            | Self::RotationPurposeScopeMismatch
            | Self::Identity(_) => IdentityTokenIssueApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPrincipalKind => "Authenticated principal kind is required",
            Self::InvalidPrincipalKind { .. } => {
                "Authenticated principal kind must be human or service"
            }
            Self::EmptySubjectId => "Token subject id is required",
            Self::InvalidSubjectKind { .. } => "Token subject kind must be human or service",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::PrincipalMismatch { .. } => {
                "Authenticated principal must match the requested token subject"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested identity token issue surface"
            }
            Self::EmptyCredentialKind => "Request credential_kind is required",
            Self::InvalidCredentialKind { .. } => "Request credential_kind must be sts",
            Self::InvalidPurpose { .. } => "Request purpose must be a supported PascalCase purpose",
            Self::EmptyPreviousTokenFingerprint => {
                "Previous STS token fingerprint is required for rotation"
            }
            Self::PreviousTokenNotYetActive { .. } => {
                "Previous STS token must already be active at rotation time"
            }
            Self::PreviousTokenExpired { .. } => {
                "Previous STS token must still be active at rotation time"
            }
            Self::RotationBindingMismatch { .. } => {
                "Replacement STS token must keep the same tenant, subject, and credential binding"
            }
            Self::RotationPurposeScopeMismatch => {
                "Replacement STS token must keep the same purpose and scopes"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::Identity(error) => identity_error_message(error),
        }
    }

    fn details(&self) -> Vec<IdentityTokenIssueApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPrincipalKind => {
                vec![detail("principal.principal_kind", "must be non-empty")]
            }
            Self::InvalidPrincipalKind { .. } => vec![detail(
                "principal.principal_kind",
                "must be either human or service",
            )],
            Self::EmptySubjectId => vec![detail("body.subject_id", "must be non-empty")],
            Self::InvalidSubjectKind { .. } => vec![detail(
                "body.subject_kind",
                "must be either human or service",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
            )],
            Self::PrincipalMismatch { .. } => vec![detail(
                "body.subject_id",
                "subject id, kind, and owning capability must match authenticated principal",
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
                "must include the requested identity token issue surface",
            )],
            Self::EmptyCredentialKind => vec![detail("body.credential_kind", "must be non-empty")],
            Self::InvalidCredentialKind { .. } => {
                vec![detail("body.credential_kind", "must be sts")]
            }
            Self::InvalidPurpose { .. } => vec![detail(
                "body.purpose",
                "must match a supported PascalCase purpose label",
            )],
            Self::EmptyPreviousTokenFingerprint => vec![detail(
                "previous.token_fingerprint",
                "must be a non-empty STS fingerprint",
            )],
            Self::PreviousTokenNotYetActive { .. } => vec![detail(
                "previous.issued_at_epoch_seconds",
                "must be less than or equal to replacement.body.issued_at_epoch_seconds",
            )],
            Self::PreviousTokenExpired { .. } => vec![detail(
                "previous.expires_at_epoch_seconds",
                "must be greater than replacement.body.issued_at_epoch_seconds",
            )],
            Self::RotationBindingMismatch { .. } => vec![detail(
                "replacement.body.subject_id",
                "must match previous tenant, subject, subject kind, credential kind, and owner",
            )],
            Self::RotationPurposeScopeMismatch => vec![detail(
                "replacement.body.scopes",
                "must match previous purpose and scope set exactly",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::Identity(error) => vec![detail("identity_kernel", identity_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdentityTokenIssueApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

pub fn validate_identity_token_issue_request(
    request: &IdentityTokenIssueApiRequest,
) -> Result<(), IdentityTokenIssueApiError> {
    validate_boundary(&request.boundary)?;
    validate_tenant_binding(&request.boundary, &request.principal, &request.body)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        IDENTITY_TOKEN_ISSUE_SURFACE,
    )?;
    parse_subject_kind(&request.body.subject_kind)?;
    parse_credential_kind(&request.body.credential_kind)?;
    parse_api_purpose(&request.body.purpose)?;
    Ok(())
}

pub fn issue_identity_token_from_app(
    idempotency_ledger: &mut IdentityTokenIssueIdempotencyLedger,
    request: IdentityTokenIssueApiRequest,
) -> Result<IdentityTokenIssueSuccessResponse, IdentityTokenIssueApiError> {
    validate_identity_token_issue_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        IDENTITY_TOKEN_ISSUE_SURFACE,
    );
    let fingerprint = identity_token_issue_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(IdentityTokenIssueApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let result = credential_request(&request.body)
        .and_then(|credential_request| {
            issue_credential(credential_request).map_err(IdentityTokenIssueApiError::Identity)
        })
        .map(|credential| {
            IdentityTokenIssueSuccessResponse::ok(
                token_record(request.body.clone(), credential),
                &request,
            )
        });

    let success = result?;
    idempotency_ledger.entries.insert(
        key,
        IdentityTokenIssueIdempotencyLedgerEntry {
            fingerprint,
            result: success.clone(),
        },
    );
    Ok(success)
}

pub fn rotate_identity_token_from_app(
    idempotency_ledger: &mut IdentityTokenIssueIdempotencyLedger,
    request: IdentityTokenRotationRequest,
) -> Result<IdentityTokenIssueSuccessResponse, IdentityTokenIssueApiError> {
    validate_identity_token_rotation_request(&request)?;
    issue_identity_token_from_app(idempotency_ledger, request.replacement)
}

pub fn validate_identity_token_rotation_request(
    request: &IdentityTokenRotationRequest,
) -> Result<(), IdentityTokenIssueApiError> {
    validate_identity_token_issue_request(&request.replacement)?;
    if request.previous.token_fingerprint.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyPreviousTokenFingerprint);
    }
    let rotate_at_epoch_seconds = request.replacement.body.issued_at_epoch_seconds;
    if request.previous.issued_at_epoch_seconds > rotate_at_epoch_seconds {
        return Err(IdentityTokenIssueApiError::PreviousTokenNotYetActive {
            previous_issued_at_epoch_seconds: request.previous.issued_at_epoch_seconds,
            rotate_at_epoch_seconds,
        });
    }
    if request.previous.expires_at_epoch_seconds <= rotate_at_epoch_seconds {
        return Err(IdentityTokenIssueApiError::PreviousTokenExpired {
            previous_expires_at_epoch_seconds: request.previous.expires_at_epoch_seconds,
            rotate_at_epoch_seconds,
        });
    }
    if request.previous.tenant_id != request.replacement.body.tenant_id
        || request.previous.subject_id != request.replacement.body.subject_id
        || request.previous.subject_kind != request.replacement.body.subject_kind
        || request.previous.owning_capability_id != request.replacement.body.owning_capability_id
        || request.previous.credential_kind != request.replacement.body.credential_kind
    {
        return Err(IdentityTokenIssueApiError::RotationBindingMismatch {
            previous_subject_id: request.previous.subject_id.clone(),
            replacement_subject_id: request.replacement.body.subject_id.clone(),
        });
    }
    if PurposeScope::from(&request.previous) != PurposeScope::from(&request.replacement.body) {
        return Err(IdentityTokenIssueApiError::RotationPurposeScopeMismatch);
    }
    Ok(())
}

impl From<&IdentityTokenRecord> for PurposeScope {
    fn from(record: &IdentityTokenRecord) -> Self {
        Self {
            purpose: record.purpose.clone(),
            scopes: record.scopes.clone(),
        }
    }
}

impl From<&IdentityTokenIssueRequest> for PurposeScope {
    fn from(request: &IdentityTokenIssueRequest) -> Self {
        Self {
            purpose: request.purpose.clone(),
            scopes: request.scopes.clone(),
        }
    }
}

fn validate_boundary(
    boundary: &IdentityApiBoundaryContext,
) -> Result<(), IdentityTokenIssueApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &IdentityApiBoundaryContext,
    principal: &IdentityApiPrincipal,
    body: &IdentityTokenIssueRequest,
) -> Result<(), IdentityTokenIssueApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyPrincipalId);
    }
    if principal.principal_kind.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyPrincipalKind);
    }
    parse_principal_kind(&principal.principal_kind)?;
    if body.subject_id.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptySubjectId);
    }
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body.tenant_id {
        return Err(IdentityTokenIssueApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body.tenant_id.clone(),
        });
    }
    if principal.principal_id != body.subject_id
        || principal.principal_kind != body.subject_kind
        || principal.owning_capability_id != body.owning_capability_id
    {
        return Err(IdentityTokenIssueApiError::PrincipalMismatch {
            principal_id: principal.principal_id.clone(),
            subject_id: body.subject_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &IdentityApiPrincipal,
    authorization: &IdentityApiAuthorization,
    surface: &str,
) -> Result<(), IdentityTokenIssueApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(IdentityTokenIssueApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(IdentityTokenIssueApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(IdentityTokenIssueApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn credential_request(
    body: &IdentityTokenIssueRequest,
) -> Result<CredentialRequest, IdentityTokenIssueApiError> {
    let purpose = parse_api_purpose(&body.purpose)?;
    Ok(CredentialRequest {
        principal: principal_from_body(body)?,
        kind: parse_credential_kind(&body.credential_kind)?,
        purpose,
        scopes: body
            .scopes
            .iter()
            .map(|scope| scope.value.clone())
            .collect(),
        ttl_seconds: body.ttl_seconds,
        issued_at_epoch_seconds: body.issued_at_epoch_seconds,
    })
}

fn principal_from_body(
    body: &IdentityTokenIssueRequest,
) -> Result<Principal, IdentityTokenIssueApiError> {
    match parse_subject_kind(&body.subject_kind)? {
        IdentitySubjectKind::Human => {
            Principal::human(body.tenant_id.clone(), body.subject_id.clone())
                .map_err(IdentityTokenIssueApiError::Identity)
        }
        IdentitySubjectKind::Service => Principal::service(
            body.tenant_id.clone(),
            body.subject_id.clone(),
            body.owning_capability_id.clone().unwrap_or_default(),
        )
        .map_err(IdentityTokenIssueApiError::Identity),
    }
}

fn parse_api_purpose(label: &str) -> Result<Purpose, IdentityTokenIssueApiError> {
    parse_purpose_pascal_label(label).ok_or(IdentityTokenIssueApiError::InvalidPurpose {
        purpose: label.to_string(),
    })
}

fn parse_principal_kind(label: &str) -> Result<IdentitySubjectKind, IdentityTokenIssueApiError> {
    if label.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyPrincipalKind);
    }
    match label {
        "human" => Ok(IdentitySubjectKind::Human),
        "service" => Ok(IdentitySubjectKind::Service),
        _ => Err(IdentityTokenIssueApiError::InvalidPrincipalKind {
            principal_kind: label.to_string(),
        }),
    }
}

fn parse_subject_kind(label: &str) -> Result<IdentitySubjectKind, IdentityTokenIssueApiError> {
    match label {
        "human" => Ok(IdentitySubjectKind::Human),
        "service" => Ok(IdentitySubjectKind::Service),
        _ => Err(IdentityTokenIssueApiError::InvalidSubjectKind {
            subject_kind: label.to_string(),
        }),
    }
}

fn parse_credential_kind(label: &str) -> Result<CredentialRequestKind, IdentityTokenIssueApiError> {
    if label.trim().is_empty() {
        return Err(IdentityTokenIssueApiError::EmptyCredentialKind);
    }
    match label {
        "sts" => Ok(CredentialRequestKind::Sts),
        _ => Err(IdentityTokenIssueApiError::InvalidCredentialKind {
            credential_kind: label.to_string(),
        }),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdentitySubjectKind {
    Human,
    Service,
}

fn idempotency_key_for(
    boundary: &IdentityApiBoundaryContext,
    principal: &IdentityApiPrincipal,
    surface: &str,
) -> IdentityTokenIssueIdempotencyLedgerKey {
    IdentityTokenIssueIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn identity_token_issue_fingerprint_for(
    request: &IdentityTokenIssueApiRequest,
) -> IdentityTokenIssueRequestFingerprint {
    IdentityTokenIssueRequestFingerprint {
        canonical: [
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "principal.principal_kind={}",
                request.principal.principal_kind
            ),
            format!(
                "principal.owning_capability_id={:?}",
                request.principal.owning_capability_id
            ),
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
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.subject_id={}", request.body.subject_id),
            format!("body.subject_kind={}", request.body.subject_kind),
            format!(
                "body.owning_capability_id={:?}",
                request.body.owning_capability_id
            ),
            format!("body.credential_kind={}", request.body.credential_kind),
            format!("body.purpose={}", request.body.purpose),
            format!("body.ttl_seconds={}", request.body.ttl_seconds),
            format!(
                "body.scopes={}",
                request
                    .body
                    .scopes
                    .iter()
                    .map(|scope| scope.value.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            format!(
                "body.issued_at_epoch_seconds={}",
                request.body.issued_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn token_record(
    body: IdentityTokenIssueRequest,
    credential: iam_identity_domain::StsCredential,
) -> IdentityTokenRecord {
    IdentityTokenRecord {
        tenant_id: body.tenant_id,
        subject_id: body.subject_id,
        subject_kind: body.subject_kind,
        owning_capability_id: body.owning_capability_id,
        credential_kind: body.credential_kind,
        purpose: credential.purpose.value.pascal_label().to_string(),
        scopes: credential
            .scopes
            .value
            .into_iter()
            .map(|value| IdentityScopeRef { value })
            .collect(),
        issued_at_epoch_seconds: credential.issued_at_epoch_seconds.value,
        expires_at_epoch_seconds: credential.expires_at_epoch_seconds.value,
        token_fingerprint: credential.token_fingerprint.value,
        schema_version: 1,
    }
}

fn identity_error_code(error: &IdentityError) -> IdentityTokenIssueApiErrorCode {
    match error {
        IdentityError::InvalidTenantId => IdentityTokenIssueApiErrorCode::IdentityInvalidTenant,
        IdentityError::InvalidUserId => IdentityTokenIssueApiErrorCode::IdentityInvalidUser,
        IdentityError::InvalidRegionPack
        | IdentityError::InvalidIdentityProviderId
        | IdentityError::EmptyExternalSubject => {
            IdentityTokenIssueApiErrorCode::IdentityInvalidUser
        }
        IdentityError::InvalidServicePrincipalId => {
            IdentityTokenIssueApiErrorCode::IdentityInvalidServicePrincipal
        }
        IdentityError::InvalidCapabilityId => {
            IdentityTokenIssueApiErrorCode::IdentityInvalidCapability
        }
        IdentityError::EmptyPrimaryIdentifier => {
            IdentityTokenIssueApiErrorCode::IdentityInvalidUser
        }
        IdentityError::TokenTtlTooLong => IdentityTokenIssueApiErrorCode::IdentityTokenTtlTooLong,
        IdentityError::TokenTtlZero => IdentityTokenIssueApiErrorCode::IdentityTokenTtlZero,
        IdentityError::MissingCredentialScope => {
            IdentityTokenIssueApiErrorCode::IdentityMissingScope
        }
        IdentityError::LongLivedCredentialForbidden => {
            IdentityTokenIssueApiErrorCode::IdentityLongLivedForbidden
        }
    }
}

fn identity_error_message(error: &IdentityError) -> &'static str {
    match error {
        IdentityError::InvalidTenantId => "Identity kernel rejected the tenant id",
        IdentityError::InvalidUserId => "Identity kernel rejected the user id",
        IdentityError::InvalidRegionPack => "Identity kernel rejected the region pack",
        IdentityError::InvalidIdentityProviderId => {
            "Identity kernel rejected the identity provider id"
        }
        IdentityError::EmptyExternalSubject => "Identity kernel rejected the external subject",
        IdentityError::InvalidServicePrincipalId => {
            "Identity kernel rejected the service principal id"
        }
        IdentityError::InvalidCapabilityId => "Identity kernel rejected the owning capability id",
        IdentityError::EmptyPrimaryIdentifier => "Identity kernel rejected the primary identifier",
        IdentityError::TokenTtlTooLong => "STS token TTL must be at most one hour",
        IdentityError::TokenTtlZero => "STS token TTL must be positive",
        IdentityError::MissingCredentialScope => {
            "STS token request must include at least one scope"
        }
        IdentityError::LongLivedCredentialForbidden => {
            "Long-lived API keys are forbidden on identity.token.issue"
        }
    }
}

fn identity_error_issue(error: &IdentityError) -> &'static str {
    match error {
        IdentityError::InvalidTenantId => "tenant id must use the ten_ prefix",
        IdentityError::InvalidUserId => "human subject id must use the usr_ prefix",
        IdentityError::InvalidRegionPack => "region pack must use the pack- prefix",
        IdentityError::InvalidIdentityProviderId => "identity provider id must use the idp_ prefix",
        IdentityError::EmptyExternalSubject => "external subject must be non-empty",
        IdentityError::InvalidServicePrincipalId => "service subject id must use the sp_ prefix",
        IdentityError::InvalidCapabilityId => "owning capability id must use the cap. prefix",
        IdentityError::EmptyPrimaryIdentifier => "primary identifier must be non-empty",
        IdentityError::TokenTtlTooLong => "ttl_seconds must be <= 3600",
        IdentityError::TokenTtlZero => "ttl_seconds must be > 0",
        IdentityError::MissingCredentialScope => "scopes must contain non-empty scope values",
        IdentityError::LongLivedCredentialForbidden => {
            "credential_kind must not request long-lived material"
        }
    }
}

fn detail(field: impl Into<String>, issue: impl Into<String>) -> IdentityTokenIssueApiErrorDetail {
    IdentityTokenIssueApiErrorDetail {
        field: field.into(),
        issue: issue.into(),
    }
}
