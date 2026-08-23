//! Platform Identity user API boundary.
//!
//! This crate owns authenticated REST-boundary normalization, tenant/path/body
//! binding, request fingerprint idempotency, per-tenant primary-identifier
//! uniqueness, and per-region IdP binding validation for `identity.user.upsert`
//! before handing typed user construction to the platform identity kernel.

use std::collections::BTreeMap;

use iam_identity_domain::{IdentityError, IdpBinding, User};

pub const IDENTITY_USER_UPSERT_SURFACE: &str = "identity.user.upsert";
pub const IDENTITY_USER_UPSERT_OPENAPI_CONTRACT: &str =
    "contracts/openapi/platform/platform-identity-user-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityUserUpsertApiStatus {
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl IdentityUserUpsertApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityUserUpsertApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathTenantIdEmpty,
    PathUserIdEmpty,
    TenantPathBodyMismatch,
    UserPathBodyMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    RegionPackInvalid,
    IdentityProviderInvalid,
    ExternalSubjectEmpty,
    PrimaryIdentifierConflict,
    IdempotencyKeyReused,
    IdentityInvalidTenant,
    IdentityInvalidUser,
    IdentityPrimaryIdentifierEmpty,
    IdentityTokenTtlTooLong,
    IdentityTokenTtlZero,
    IdentityMissingScope,
    IdentityLongLivedForbidden,
    IdentityInvalidServicePrincipal,
    IdentityInvalidCapability,
}

impl IdentityUserUpsertApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "IDENTITY_USER_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "IDENTITY_USER_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "IDENTITY_USER_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "IDENTITY_USER_PRINCIPAL_ID_EMPTY",
            Self::PathTenantIdEmpty => "IDENTITY_USER_PATH_TENANT_ID_EMPTY",
            Self::PathUserIdEmpty => "IDENTITY_USER_PATH_USER_ID_EMPTY",
            Self::TenantPathBodyMismatch => "IDENTITY_USER_TENANT_PATH_BODY_MISMATCH",
            Self::UserPathBodyMismatch => "IDENTITY_USER_USER_PATH_BODY_MISMATCH",
            Self::TenantMismatch => "IDENTITY_USER_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "IDENTITY_USER_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "IDENTITY_USER_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "IDENTITY_USER_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "IDENTITY_USER_AUTHORIZATION_DENIED",
            Self::RegionPackInvalid => "IDENTITY_USER_REGION_PACK_INVALID",
            Self::IdentityProviderInvalid => "IDENTITY_USER_IDENTITY_PROVIDER_INVALID",
            Self::ExternalSubjectEmpty => "IDENTITY_USER_EXTERNAL_SUBJECT_EMPTY",
            Self::PrimaryIdentifierConflict => "IDENTITY_USER_PRIMARY_IDENTIFIER_CONFLICT",
            Self::IdempotencyKeyReused => "IDENTITY_USER_IDEMPOTENCY_KEY_REUSED",
            Self::IdentityInvalidTenant => "IDENTITY_USER_IDENTITY_INVALID_TENANT",
            Self::IdentityInvalidUser => "IDENTITY_USER_IDENTITY_INVALID_USER",
            Self::IdentityPrimaryIdentifierEmpty => "IDENTITY_USER_PRIMARY_IDENTIFIER_EMPTY",
            Self::IdentityTokenTtlTooLong => "IDENTITY_USER_IDENTITY_TOKEN_TTL_TOO_LONG",
            Self::IdentityTokenTtlZero => "IDENTITY_USER_IDENTITY_TOKEN_TTL_ZERO",
            Self::IdentityMissingScope => "IDENTITY_USER_IDENTITY_SCOPE_MISSING",
            Self::IdentityLongLivedForbidden => "IDENTITY_USER_IDENTITY_LONG_LIVED_FORBIDDEN",
            Self::IdentityInvalidServicePrincipal => {
                "IDENTITY_USER_IDENTITY_INVALID_SERVICE_PRINCIPAL"
            }
            Self::IdentityInvalidCapability => "IDENTITY_USER_IDENTITY_INVALID_CAPABILITY",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserRoleRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserUpsertRequest {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub user_id: String,                 // data_class: PII_IDENTIFYING
    pub primary_identifier: String,      // data_class: PII_IDENTIFYING
    pub display_name: String,            // data_class: PII_QUASI_IDENTIFIER
    pub roles: Vec<IdentityUserRoleRef>, // data_class: INTERNAL_ONLY
    pub region_pack: String,             // data_class: INTERNAL_ONLY
    pub identity_provider_id: String,    // data_class: INTERNAL_ONLY
    pub external_subject: String,        // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserUpsertApiRequest {
    pub path_tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub path_user_id: String,                     // data_class: PII_IDENTIFYING
    pub boundary: IdentityUserApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: IdentityUserApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: IdentityUserApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: IdentityUserUpsertRequest,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IdentityUserDirectory {
    users: BTreeMap<IdentityUserDirectoryKey, IdentityUserDirectoryEntry>, // data_class: INTERNAL_ONLY
    primary_identifiers: BTreeMap<IdentityUserPrimaryIdentifierKey, String>, // data_class: INTERNAL_ONLY
}

impl IdentityUserDirectory {
    pub fn len(&self) -> usize {
        self.users.len()
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    pub fn get(&self, tenant_id: &str, user_id: &str) -> Option<&IdentityUserDirectoryEntry> {
        self.users.get(&IdentityUserDirectoryKey {
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct IdentityUserDirectoryKey {
    tenant_id: String, // data_class: INTERNAL_ONLY
    user_id: String,   // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct IdentityUserPrimaryIdentifierKey {
    tenant_id: String,          // data_class: INTERNAL_ONLY
    primary_identifier: String, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserDirectoryEntry {
    pub user: User,                   // data_class: INTERNAL_ONLY
    pub region_pack: String,          // data_class: INTERNAL_ONLY
    pub identity_provider_id: String, // data_class: INTERNAL_ONLY
    pub external_subject: String,     // data_class: PII_IDENTIFYING
    pub schema_version: u32,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IdentityUserUpsertIdempotencyLedger {
    entries:
        BTreeMap<IdentityUserUpsertIdempotencyLedgerKey, IdentityUserUpsertIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl IdentityUserUpsertIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct IdentityUserUpsertIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IdentityUserUpsertIdempotencyLedgerEntry {
    fingerprint: IdentityUserUpsertRequestFingerprint, // data_class: INTERNAL_ONLY
    result: IdentityUserUpsertSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IdentityUserUpsertRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserUpsertSuccessResponse {
    pub data: IdentityUserRecord,             // data_class: INTERNAL_ONLY
    pub metadata: IdentityUserUpsertMetadata, // data_class: INTERNAL_ONLY
}

impl IdentityUserUpsertSuccessResponse {
    pub fn ok(
        data: IdentityUserRecord,
        request: &IdentityUserUpsertApiRequest,
        result: impl Into<String>,
    ) -> Self {
        Self {
            data,
            metadata: IdentityUserUpsertMetadata {
                request_id: request.boundary.request_id.clone(),
                tenant_id: request.boundary.tenant_id.clone(),
                principal_id: request.principal.principal_id.clone(),
                result: result.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserUpsertMetadata {
    pub request_id: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
    pub result: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserRecord {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub user_id: String,                 // data_class: PII_IDENTIFYING
    pub primary_identifier: String,      // data_class: PII_IDENTIFYING
    pub display_name: String,            // data_class: PII_QUASI_IDENTIFIER
    pub roles: Vec<IdentityUserRoleRef>, // data_class: INTERNAL_ONLY
    pub region_pack: String,             // data_class: INTERNAL_ONLY
    pub identity_provider_id: String,    // data_class: INTERNAL_ONLY
    pub external_subject: String,        // data_class: PII_IDENTIFYING
    pub schema_version: u32,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserUpsertApiErrorResponse {
    pub error: IdentityUserUpsertApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserUpsertApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<IdentityUserUpsertApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUserUpsertApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityUserUpsertApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathTenantId,
    EmptyPathUserId,
    TenantPathBodyMismatch {
        path_tenant_id: String,
        body_tenant_id: String,
    },
    UserPathBodyMismatch {
        path_user_id: String,
        body_user_id: String,
    },
    TenantMismatch {
        boundary_tenant_id: String,
        path_tenant_id: String,
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
    InvalidRegionPack {
        region_pack: String,
    },
    InvalidIdentityProvider {
        identity_provider_id: String,
    },
    EmptyExternalSubject,
    PrimaryIdentifierConflict {
        tenant_id: String,
        primary_identifier: String,
        existing_user_id: String,
        requested_user_id: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Identity(IdentityError),
}

impl IdentityUserUpsertApiError {
    pub fn identity_user_upsert_status(&self) -> IdentityUserUpsertApiStatus {
        match self.status_kind() {
            IdentityUserUpsertApiStatusKind::BadRequest => IdentityUserUpsertApiStatus::BadRequest,
            IdentityUserUpsertApiStatusKind::Unauthorized => {
                IdentityUserUpsertApiStatus::Unauthorized
            }
            IdentityUserUpsertApiStatusKind::Forbidden => IdentityUserUpsertApiStatus::Forbidden,
            IdentityUserUpsertApiStatusKind::Conflict => IdentityUserUpsertApiStatus::Conflict,
            IdentityUserUpsertApiStatusKind::UnprocessableEntity => {
                IdentityUserUpsertApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn identity_user_upsert_status_code(&self) -> u16 {
        self.identity_user_upsert_status().code()
    }

    pub fn code(&self) -> IdentityUserUpsertApiErrorCode {
        match self {
            Self::EmptyRequestId => IdentityUserUpsertApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => IdentityUserUpsertApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => IdentityUserUpsertApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => IdentityUserUpsertApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathTenantId => IdentityUserUpsertApiErrorCode::PathTenantIdEmpty,
            Self::EmptyPathUserId => IdentityUserUpsertApiErrorCode::PathUserIdEmpty,
            Self::TenantPathBodyMismatch { .. } => {
                IdentityUserUpsertApiErrorCode::TenantPathBodyMismatch
            }
            Self::UserPathBodyMismatch { .. } => {
                IdentityUserUpsertApiErrorCode::UserPathBodyMismatch
            }
            Self::TenantMismatch { .. } => IdentityUserUpsertApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                IdentityUserUpsertApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                IdentityUserUpsertApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                IdentityUserUpsertApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => IdentityUserUpsertApiErrorCode::AuthorizationDenied,
            Self::InvalidRegionPack { .. } => IdentityUserUpsertApiErrorCode::RegionPackInvalid,
            Self::InvalidIdentityProvider { .. } => {
                IdentityUserUpsertApiErrorCode::IdentityProviderInvalid
            }
            Self::EmptyExternalSubject => IdentityUserUpsertApiErrorCode::ExternalSubjectEmpty,
            Self::PrimaryIdentifierConflict { .. } => {
                IdentityUserUpsertApiErrorCode::PrimaryIdentifierConflict
            }
            Self::IdempotencyKeyReused { .. } => {
                IdentityUserUpsertApiErrorCode::IdempotencyKeyReused
            }
            Self::Identity(error) => identity_error_code(error),
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> IdentityUserUpsertApiErrorResponse {
        IdentityUserUpsertApiErrorResponse {
            error: IdentityUserUpsertApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> IdentityUserUpsertApiStatusKind {
        match self {
            Self::EmptyPrincipalId => IdentityUserUpsertApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => IdentityUserUpsertApiStatusKind::Forbidden,
            Self::PrimaryIdentifierConflict { .. } => IdentityUserUpsertApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => {
                IdentityUserUpsertApiStatusKind::UnprocessableEntity
            }
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathTenantId
            | Self::EmptyPathUserId
            | Self::TenantPathBodyMismatch { .. }
            | Self::UserPathBodyMismatch { .. }
            | Self::InvalidRegionPack { .. }
            | Self::InvalidIdentityProvider { .. }
            | Self::EmptyExternalSubject
            | Self::Identity(_) => IdentityUserUpsertApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathTenantId => "Path tenant id is required",
            Self::EmptyPathUserId => "Path user id is required",
            Self::TenantPathBodyMismatch { .. } => {
                "Path tenant id must match request body tenant_id"
            }
            Self::UserPathBodyMismatch { .. } => "Path user id must match request body user_id",
            Self::TenantMismatch { .. } => "Authenticated tenant header must match the path tenant",
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested identity user upsert surface"
            }
            Self::InvalidRegionPack { .. } => "Region pack must be an pack-* identifier",
            Self::InvalidIdentityProvider { .. } => {
                "Identity provider id must be an idp_* identifier"
            }
            Self::EmptyExternalSubject => "External IdP subject is required",
            Self::PrimaryIdentifierConflict { .. } => {
                "Primary identifier is already bound to another user in this tenant"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::Identity(error) => identity_error_message(error),
        }
    }

    fn details(&self) -> Vec<IdentityUserUpsertApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathTenantId => vec![detail("path.tenant_id", "must be non-empty")],
            Self::EmptyPathUserId => vec![detail("path.user_id", "must be non-empty")],
            Self::TenantPathBodyMismatch { .. } => vec![detail(
                "body.tenant_id",
                "must match the tenant_id path parameter",
            )],
            Self::UserPathBodyMismatch { .. } => vec![detail(
                "body.user_id",
                "must match the user_id path parameter",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "header.X-Tenant-Id",
                "must match the tenant_id path parameter and bearer tenant",
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
                "must include the requested identity.user.upsert surface",
            )],
            Self::InvalidRegionPack { .. } => {
                vec![detail("body.region_pack", "must start with pack-")]
            }
            Self::InvalidIdentityProvider { .. } => {
                vec![detail("body.identity_provider_id", "must start with idp_")]
            }
            Self::EmptyExternalSubject => {
                vec![detail("body.external_subject", "must be non-empty")]
            }
            Self::PrimaryIdentifierConflict { .. } => vec![detail(
                "body.primary_identifier",
                "must be unique per tenant",
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
enum IdentityUserUpsertApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_identity_user_upsert_request(
    request: &IdentityUserUpsertApiRequest,
) -> Result<(), IdentityUserUpsertApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_body_binding(request)?;
    validate_authenticated_tenant_binding(request)?;
    validate_principal(&request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        IDENTITY_USER_UPSERT_SURFACE,
    )?;
    validate_idp_binding(&request.body)?;
    validate_tenant_id_shape(&request.body.tenant_id)?;
    Ok(())
}

pub fn upsert_identity_user_from_api(
    directory: &mut IdentityUserDirectory,
    idempotency_ledger: &mut IdentityUserUpsertIdempotencyLedger,
    request: IdentityUserUpsertApiRequest,
) -> Result<IdentityUserUpsertSuccessResponse, IdentityUserUpsertApiError> {
    validate_identity_user_upsert_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        IDENTITY_USER_UPSERT_SURFACE,
    );
    let fingerprint = identity_user_upsert_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(IdentityUserUpsertApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let user_key = directory_key_for(&request.body.tenant_id, &request.body.user_id);
    let primary_key =
        primary_identifier_key_for(&request.body.tenant_id, &request.body.primary_identifier);
    if let Some(existing_user_id) = directory.primary_identifiers.get(&primary_key)
        && existing_user_id != &request.body.user_id
    {
        return Err(IdentityUserUpsertApiError::PrimaryIdentifierConflict {
            tenant_id: request.body.tenant_id,
            primary_identifier: request.body.primary_identifier,
            existing_user_id: existing_user_id.clone(),
            requested_user_id: request.body.user_id,
        });
    }

    let result = if directory.users.contains_key(&user_key) {
        "updated"
    } else {
        "created"
    };
    let user = user_from_request(&request.body)?;
    let entry = IdentityUserDirectoryEntry {
        user,
        region_pack: request.body.region_pack.clone(),
        identity_provider_id: request.body.identity_provider_id.clone(),
        external_subject: request.body.external_subject.clone(),
        schema_version: 1,
    };
    let response = IdentityUserUpsertSuccessResponse::ok(user_record(&entry), &request, result);
    directory
        .primary_identifiers
        .insert(primary_key, request.body.user_id.clone());
    directory.users.insert(user_key, entry);
    idempotency_ledger.entries.insert(
        key,
        IdentityUserUpsertIdempotencyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(
    boundary: &IdentityUserApiBoundaryContext,
) -> Result<(), IdentityUserUpsertApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(IdentityUserUpsertApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(IdentityUserUpsertApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(IdentityUserUpsertApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_body_binding(
    request: &IdentityUserUpsertApiRequest,
) -> Result<(), IdentityUserUpsertApiError> {
    if request.path_tenant_id.trim().is_empty() {
        return Err(IdentityUserUpsertApiError::EmptyPathTenantId);
    }
    if request.path_user_id.trim().is_empty() {
        return Err(IdentityUserUpsertApiError::EmptyPathUserId);
    }
    if request.path_tenant_id != request.body.tenant_id {
        return Err(IdentityUserUpsertApiError::TenantPathBodyMismatch {
            path_tenant_id: request.path_tenant_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
        });
    }
    if request.path_user_id != request.body.user_id {
        return Err(IdentityUserUpsertApiError::UserPathBodyMismatch {
            path_user_id: request.path_user_id.clone(),
            body_user_id: request.body.user_id.clone(),
        });
    }
    Ok(())
}

fn validate_authenticated_tenant_binding(
    request: &IdentityUserUpsertApiRequest,
) -> Result<(), IdentityUserUpsertApiError> {
    if request.boundary.tenant_id != request.path_tenant_id {
        return Err(IdentityUserUpsertApiError::TenantMismatch {
            boundary_tenant_id: request.boundary.tenant_id.clone(),
            path_tenant_id: request.path_tenant_id.clone(),
        });
    }
    if request.principal.tenant_id != request.boundary.tenant_id {
        return Err(IdentityUserUpsertApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: request.principal.tenant_id.clone(),
            principal_tenant_id: request.boundary.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_principal(
    principal: &IdentityUserApiPrincipal,
) -> Result<(), IdentityUserUpsertApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(IdentityUserUpsertApiError::EmptyPrincipalId);
    }
    Ok(())
}

fn validate_authorization(
    principal: &IdentityUserApiPrincipal,
    authorization: &IdentityUserApiAuthorization,
    surface: &str,
) -> Result<(), IdentityUserUpsertApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(IdentityUserUpsertApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(IdentityUserUpsertApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(IdentityUserUpsertApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(IdentityUserUpsertApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn validate_idp_binding(
    body: &IdentityUserUpsertRequest,
) -> Result<(), IdentityUserUpsertApiError> {
    if !body.region_pack.starts_with("pack-") || body.region_pack.len() <= "pack-".len() {
        return Err(IdentityUserUpsertApiError::InvalidRegionPack {
            region_pack: body.region_pack.clone(),
        });
    }
    if !body.identity_provider_id.starts_with("idp_") || body.identity_provider_id.len() <= 4 {
        return Err(IdentityUserUpsertApiError::InvalidIdentityProvider {
            identity_provider_id: body.identity_provider_id.clone(),
        });
    }
    if body.external_subject.trim().is_empty() {
        return Err(IdentityUserUpsertApiError::EmptyExternalSubject);
    }
    Ok(())
}

fn validate_tenant_id_shape(tenant_id: &str) -> Result<(), IdentityUserUpsertApiError> {
    if tenant_id.starts_with("ten_") && tenant_id.len() > 4 {
        Ok(())
    } else {
        Err(IdentityUserUpsertApiError::Identity(
            IdentityError::InvalidTenantId,
        ))
    }
}

fn user_from_request(body: &IdentityUserUpsertRequest) -> Result<User, IdentityUserUpsertApiError> {
    let idp_binding = IdpBinding::new(
        body.region_pack.clone(),
        body.identity_provider_id.clone(),
        body.external_subject.clone(),
        0,
    )
    .map_err(IdentityUserUpsertApiError::Identity)?;

    User::new(
        body.tenant_id.clone(),
        body.user_id.clone(),
        body.primary_identifier.clone(),
        body.display_name.clone(),
        body.roles.iter().map(|role| role.value.clone()).collect(),
        idp_binding,
    )
    .map_err(IdentityUserUpsertApiError::Identity)
}

fn directory_key_for(tenant_id: &str, user_id: &str) -> IdentityUserDirectoryKey {
    IdentityUserDirectoryKey {
        tenant_id: tenant_id.to_string(),
        user_id: user_id.to_string(),
    }
}

fn primary_identifier_key_for(
    tenant_id: &str,
    primary_identifier: &str,
) -> IdentityUserPrimaryIdentifierKey {
    IdentityUserPrimaryIdentifierKey {
        tenant_id: tenant_id.to_string(),
        primary_identifier: primary_identifier.to_string(),
    }
}

fn idempotency_key_for(
    boundary: &IdentityUserApiBoundaryContext,
    principal: &IdentityUserApiPrincipal,
    surface: &str,
) -> IdentityUserUpsertIdempotencyLedgerKey {
    IdentityUserUpsertIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn identity_user_upsert_fingerprint_for(
    request: &IdentityUserUpsertApiRequest,
) -> IdentityUserUpsertRequestFingerprint {
    IdentityUserUpsertRequestFingerprint {
        canonical: [
            format!("path.tenant_id={}", request.path_tenant_id),
            format!("path.user_id={}", request.path_user_id),
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
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.user_id={}", request.body.user_id),
            format!(
                "body.primary_identifier={}",
                request.body.primary_identifier
            ),
            format!("body.display_name={}", request.body.display_name),
            format!(
                "body.roles={}",
                request
                    .body
                    .roles
                    .iter()
                    .map(|role| role.value.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            format!("body.region_pack={}", request.body.region_pack),
            format!(
                "body.identity_provider_id={}",
                request.body.identity_provider_id
            ),
            format!("body.external_subject={}", request.body.external_subject),
        ]
        .join("\n"),
    }
}

fn user_record(entry: &IdentityUserDirectoryEntry) -> IdentityUserRecord {
    IdentityUserRecord {
        tenant_id: entry.user.tenant_id.value.clone(),
        user_id: entry.user.id.value.as_str().to_string(),
        primary_identifier: entry.user.primary_identifier.value.clone(),
        display_name: entry.user.display_name.value.clone(),
        roles: entry
            .user
            .roles
            .value
            .iter()
            .map(|role| IdentityUserRoleRef {
                value: role.clone(),
            })
            .collect(),
        region_pack: entry.region_pack.clone(),
        identity_provider_id: entry.identity_provider_id.clone(),
        external_subject: entry.external_subject.clone(),
        schema_version: entry.schema_version,
    }
}

fn identity_error_code(error: &IdentityError) -> IdentityUserUpsertApiErrorCode {
    match error {
        IdentityError::InvalidTenantId => IdentityUserUpsertApiErrorCode::IdentityInvalidTenant,
        IdentityError::InvalidUserId => IdentityUserUpsertApiErrorCode::IdentityInvalidUser,
        IdentityError::InvalidRegionPack => IdentityUserUpsertApiErrorCode::RegionPackInvalid,
        IdentityError::InvalidIdentityProviderId => {
            IdentityUserUpsertApiErrorCode::IdentityProviderInvalid
        }
        IdentityError::InvalidServicePrincipalId => {
            IdentityUserUpsertApiErrorCode::IdentityInvalidServicePrincipal
        }
        IdentityError::InvalidCapabilityId => {
            IdentityUserUpsertApiErrorCode::IdentityInvalidCapability
        }
        IdentityError::EmptyPrimaryIdentifier => {
            IdentityUserUpsertApiErrorCode::IdentityPrimaryIdentifierEmpty
        }
        IdentityError::EmptyExternalSubject => IdentityUserUpsertApiErrorCode::ExternalSubjectEmpty,
        IdentityError::TokenTtlTooLong => IdentityUserUpsertApiErrorCode::IdentityTokenTtlTooLong,
        IdentityError::TokenTtlZero => IdentityUserUpsertApiErrorCode::IdentityTokenTtlZero,
        IdentityError::MissingCredentialScope => {
            IdentityUserUpsertApiErrorCode::IdentityMissingScope
        }
        IdentityError::LongLivedCredentialForbidden => {
            IdentityUserUpsertApiErrorCode::IdentityLongLivedForbidden
        }
    }
}

fn identity_error_message(error: &IdentityError) -> &'static str {
    match error {
        IdentityError::InvalidTenantId => "Tenant id must be a ten_ identifier",
        IdentityError::InvalidUserId => "User id must be a usr_ identifier",
        IdentityError::InvalidRegionPack => "Region pack must be an pack-* identifier",
        IdentityError::InvalidIdentityProviderId => {
            "Identity provider id must be an idp_* identifier"
        }
        IdentityError::InvalidServicePrincipalId => {
            "Service principal id must be an sp_ identifier"
        }
        IdentityError::InvalidCapabilityId => "Capability id must be a cap.* identifier",
        IdentityError::EmptyPrimaryIdentifier => "Primary identifier is required",
        IdentityError::EmptyExternalSubject => "External IdP subject is required",
        IdentityError::TokenTtlTooLong => "Token ttl exceeds maximum",
        IdentityError::TokenTtlZero => "Token ttl must be positive",
        IdentityError::MissingCredentialScope => "At least one credential scope is required",
        IdentityError::LongLivedCredentialForbidden => "Long-lived credentials are forbidden",
    }
}

fn identity_error_issue(error: &IdentityError) -> &'static str {
    match error {
        IdentityError::InvalidTenantId => "tenant_id must start with ten_",
        IdentityError::InvalidUserId => "user_id must start with usr_",
        IdentityError::InvalidRegionPack => "region_pack must start with pack-",
        IdentityError::InvalidIdentityProviderId => "identity_provider_id must start with idp_",
        IdentityError::InvalidServicePrincipalId => "service_principal_id must start with sp_",
        IdentityError::InvalidCapabilityId => "owning_capability_id must start with cap.",
        IdentityError::EmptyPrimaryIdentifier => "primary_identifier must be non-empty",
        IdentityError::EmptyExternalSubject => "external_subject must be non-empty",
        IdentityError::TokenTtlTooLong => "ttl_seconds must be at most one hour",
        IdentityError::TokenTtlZero => "ttl_seconds must be greater than zero",
        IdentityError::MissingCredentialScope => "scopes must be non-empty",
        IdentityError::LongLivedCredentialForbidden => "credential_kind must be sts",
    }
}

fn detail(field: &str, issue: &str) -> IdentityUserUpsertApiErrorDetail {
    IdentityUserUpsertApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
