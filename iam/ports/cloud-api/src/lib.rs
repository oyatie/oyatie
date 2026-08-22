//! Cloud IAM API boundary for IAM role creation and STS token issuance.
//!
//! This crate owns tenant/header/path/body normalization before handing typed
//! requests to the Cloud IAM kernel.

use std::collections::BTreeMap;

use iam_cloud_domain::{
    AssumeRoleRequest, CloudIamError, IamDirectory, IamRole, IamRoleCreate, IdentityProvider,
    IdentityProviderCreate, IdentityProviderKind, IdentityProviderUpdate, StsSession,
};
pub use iam_cloud_domain::{
    CloudIamBoundaryCellId, CloudIamBoundaryRegionId, CloudIamBoundaryTenantId,
    CloudIamPlacementBoundary as CloudIamApiPlacementBoundary,
};
use data_boundary_kernel::parse_data_class_label;

pub const CLOUD_IAM_IDENTITY_PROVIDER_CREATE_SURFACE: &str = "cloud.iam.identity_provider.create";
pub const CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE: &str = "cloud.iam.identity_provider.delete";
pub const CLOUD_IAM_IDENTITY_PROVIDER_LIST_SURFACE: &str = "cloud.iam.identity_provider.list";
pub const CLOUD_IAM_IDENTITY_PROVIDER_UPDATE_SURFACE: &str = "cloud.iam.identity_provider.update";
pub const CLOUD_IAM_ROLE_CREATE_SURFACE: &str = "cloud.iam.role.create";
pub const CLOUD_IAM_STS_TOKEN_SURFACE: &str = "cloud.iam.sts.token";
pub const CLOUD_IAM_PUBLIC_API_VERSION_HEADER: &str = "Oyatie-Version";
pub const CLOUD_IAM_DEFAULT_PUBLIC_API_VERSION: &str = "2026-05-21";
pub const CLOUD_IAM_SUPPORTED_PUBLIC_API_VERSIONS: &[&str] =
    &["2026-05-21", "2026-02-21", "2025-11-21"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudIamIdentityProviderDeleteApiStatus {
    Ok,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CloudIamIdentityProviderDeleteApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudIamIdentityProviderListApiStatus {
    Ok,
    BadRequest,
    Forbidden,
}

impl CloudIamIdentityProviderListApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudIamIdentityProviderUpdateApiStatus {
    Ok,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CloudIamIdentityProviderUpdateApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudIamIdentityProviderCreateApiStatus {
    Created,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CloudIamIdentityProviderCreateApiStatus {
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
pub enum CloudIamRoleCreateApiStatus {
    Created,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CloudIamRoleCreateApiStatus {
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
pub enum CloudIamStsTokenApiStatus {
    Ok,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CloudIamStsTokenApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudIamApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    BoundaryCellEmpty,
    BoundaryRegionEmpty,
    PlacementTenantMismatch,
    PublicApiVersionMissing,
    PublicApiVersionUnsupported,
    IdempotencyKeyEmpty,
    PathProviderIdEmpty,
    PathRoleIdEmpty,
    ProviderIdMismatch,
    RoleIdMismatch,
    TenantMismatch,
    PrincipalMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    DataClassInvalid,
    IamInvalidRequest,
    IamForbidden,
    IamConflict,
}

impl CloudIamApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_IAM_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_IAM_TENANT_HEADER_EMPTY",
            Self::BoundaryCellEmpty => "CLOUD_IAM_BOUNDARY_CELL_EMPTY",
            Self::BoundaryRegionEmpty => "CLOUD_IAM_BOUNDARY_REGION_EMPTY",
            Self::PlacementTenantMismatch => "CLOUD_IAM_PLACEMENT_TENANT_MISMATCH",
            Self::PublicApiVersionMissing => "CLOUD_IAM_PUBLIC_API_VERSION_MISSING",
            Self::PublicApiVersionUnsupported => "CLOUD_IAM_PUBLIC_API_VERSION_UNSUPPORTED",
            Self::IdempotencyKeyEmpty => "CLOUD_IAM_IDEMPOTENCY_KEY_EMPTY",
            Self::PathProviderIdEmpty => "CLOUD_IAM_PATH_PROVIDER_ID_EMPTY",
            Self::PathRoleIdEmpty => "CLOUD_IAM_PATH_ROLE_ID_EMPTY",
            Self::ProviderIdMismatch => "CLOUD_IAM_PROVIDER_ID_MISMATCH",
            Self::RoleIdMismatch => "CLOUD_IAM_ROLE_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_IAM_TENANT_MISMATCH",
            Self::PrincipalMismatch => "CLOUD_IAM_PRINCIPAL_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "CLOUD_IAM_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "CLOUD_IAM_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "CLOUD_IAM_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "CLOUD_IAM_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_IAM_IDEMPOTENCY_KEY_REUSED",
            Self::DataClassInvalid => "CLOUD_IAM_DATA_CLASS_INVALID",
            Self::IamInvalidRequest => "CLOUD_IAM_INVALID_REQUEST",
            Self::IamForbidden => "CLOUD_IAM_FORBIDDEN",
            Self::IamConflict => "CLOUD_IAM_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamApiBoundaryContext {
    pub request_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                 // data_class: INTERNAL_ONLY
    pub oyatie_version: String,                  // data_class: PUBLIC
    pub placement: CloudIamApiPlacementBoundary, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamApiReadBoundaryContext {
    pub request_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub oyatie_version: String,                  // data_class: PUBLIC
    pub placement: CloudIamApiPlacementBoundary, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamPrincipalRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamScopeRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudIamIdentityProviderKind {
    Saml,
    Oidc,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderCreateRequest {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub identity_provider_id: String,       // data_class: INTERNAL_ONLY
    pub region_pack: String,                // data_class: INTERNAL_ONLY
    pub kind: CloudIamIdentityProviderKind, // data_class: PUBLIC
    pub issuer_uri: String,                 // data_class: INTERNAL_ONLY
    pub audience: String,                   // data_class: INTERNAL_ONLY
    pub verification_material_ref: String,  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderCreateApiRequest {
    pub path_identity_provider_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudIamApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudIamApiPrincipal,   // data_class: INTERNAL_ONLY
    pub authorization: CloudIamApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudIamIdentityProviderCreateRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderUpdateRequest {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub identity_provider_id: String,       // data_class: INTERNAL_ONLY
    pub region_pack: String,                // data_class: INTERNAL_ONLY
    pub kind: CloudIamIdentityProviderKind, // data_class: PUBLIC
    pub issuer_uri: String,                 // data_class: INTERNAL_ONLY
    pub audience: String,                   // data_class: INTERNAL_ONLY
    pub verification_material_ref: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderUpdateApiRequest {
    pub path_identity_provider_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudIamApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudIamApiPrincipal,   // data_class: INTERNAL_ONLY
    pub authorization: CloudIamApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudIamIdentityProviderUpdateRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderDeleteApiRequest {
    pub path_identity_provider_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudIamApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudIamApiPrincipal,   // data_class: INTERNAL_ONLY
    pub authorization: CloudIamApiAuthorization, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderListApiRequest {
    pub boundary: CloudIamApiReadBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudIamApiPrincipal,          // data_class: INTERNAL_ONLY
    pub authorization: CloudIamApiAuthorization,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamRoleCreateRequest {
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub role_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                          // data_class: PUBLIC
    pub name: String,                            // data_class: PUBLIC
    pub cedar_policy_id: String,                 // data_class: INTERNAL_ONLY
    pub cedar_policy_version: String,            // data_class: INTERNAL_ONLY
    pub assumable_by: Vec<CloudIamPrincipalRef>, // data_class: INTERNAL_ONLY
    pub max_session_duration_sec: u32,           // data_class: INTERNAL_ONLY
    pub data_class: String,                      // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamRoleCreateApiRequest {
    pub path_role_id: String,                    // data_class: INTERNAL_ONLY
    pub boundary: CloudIamApiBoundaryContext,    // data_class: INTERNAL_ONLY
    pub principal: CloudIamApiPrincipal,         // data_class: INTERNAL_ONLY
    pub authorization: CloudIamApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudIamRoleCreateRequest,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamStsTokenRequest {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub session_id: String,            // data_class: INTERNAL_ONLY
    pub role_id: String,               // data_class: INTERNAL_ONLY
    pub assumed_by: String,            // data_class: INTERNAL_ONLY
    pub external_id: Option<String>,   // data_class: INTERNAL_ONLY
    pub requested_duration_sec: u32,   // data_class: INTERNAL_ONLY
    pub scopes: Vec<CloudIamScopeRef>, // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamStsTokenApiRequest {
    pub boundary: CloudIamApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudIamApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: CloudIamApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudIamStsTokenRequest,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudIamIdentityProviderCreateIdempotencyLedger {
    entries: BTreeMap<
        CloudIamIdempotencyLedgerKey,
        CloudIamIdentityProviderCreateIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl CloudIamIdentityProviderCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudIamIdentityProviderUpdateIdempotencyLedger {
    entries: BTreeMap<
        CloudIamIdempotencyLedgerKey,
        CloudIamIdentityProviderUpdateIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl CloudIamIdentityProviderUpdateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudIamIdentityProviderDeleteIdempotencyLedger {
    entries: BTreeMap<
        CloudIamIdempotencyLedgerKey,
        CloudIamIdentityProviderDeleteIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl CloudIamIdentityProviderDeleteIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudIamRoleCreateIdempotencyLedger {
    entries: BTreeMap<CloudIamIdempotencyLedgerKey, CloudIamRoleCreateIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudIamRoleCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudIamStsTokenIdempotencyLedger {
    entries: BTreeMap<CloudIamIdempotencyLedgerKey, CloudIamStsTokenIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudIamStsTokenIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudIamIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudIamIdentityProviderCreateIdempotencyLedgerEntry {
    fingerprint: CloudIamRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudIamIdentityProviderCreateApiResult, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudIamIdentityProviderUpdateIdempotencyLedgerEntry {
    fingerprint: CloudIamRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudIamIdentityProviderUpdateApiResult, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudIamIdentityProviderDeleteIdempotencyLedgerEntry {
    fingerprint: CloudIamRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudIamIdentityProviderDeleteApiResult, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudIamRoleCreateIdempotencyLedgerEntry {
    fingerprint: CloudIamRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudIamRoleCreateApiResult,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudIamStsTokenIdempotencyLedgerEntry {
    fingerprint: CloudIamRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudIamStsTokenApiResult,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudIamRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudIamRoleCreateApiResult = Result<CloudIamRoleCreateSuccessResponse, CloudIamApiError>;
type CloudIamStsTokenApiResult = Result<CloudIamStsTokenSuccessResponse, CloudIamApiError>;
type CloudIamIdentityProviderCreateApiResult =
    Result<CloudIamIdentityProviderCreateSuccessResponse, CloudIamApiError>;
type CloudIamIdentityProviderUpdateApiResult =
    Result<CloudIamIdentityProviderUpdateSuccessResponse, CloudIamApiError>;
type CloudIamIdentityProviderDeleteApiResult =
    Result<CloudIamIdentityProviderDeleteSuccessResponse, CloudIamApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderCreateSuccessResponse {
    pub data: CloudIamIdentityProviderRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudIamApiResponseMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderUpdateSuccessResponse {
    pub data: CloudIamIdentityProviderRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudIamApiResponseMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderDeleteSuccessResponse {
    pub data: CloudIamIdentityProviderRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudIamApiResponseMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderListSuccessResponse {
    pub data: Vec<CloudIamIdentityProviderRecord>, // data_class: INTERNAL_ONLY
    pub metadata: CloudIamApiResponseMetadata,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamRoleCreateSuccessResponse {
    pub data: CloudIamRoleRecord,              // data_class: INTERNAL_ONLY
    pub metadata: CloudIamApiResponseMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamStsTokenSuccessResponse {
    pub data: CloudIamStsSessionRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudIamApiResponseMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamApiResponseMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamIdentityProviderRecord {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub identity_provider_id: String,       // data_class: INTERNAL_ONLY
    pub region_pack: String,                // data_class: INTERNAL_ONLY
    pub kind: CloudIamIdentityProviderKind, // data_class: PUBLIC
    pub issuer_uri: String,                 // data_class: INTERNAL_ONLY
    pub audience: String,                   // data_class: INTERNAL_ONLY
    pub verification_material_ref: String,  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub schema_version: u32,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamRoleRecord {
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub role_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                          // data_class: PUBLIC
    pub name: String,                            // data_class: PUBLIC
    pub cedar_policy_id: String,                 // data_class: INTERNAL_ONLY
    pub cedar_policy_version: String,            // data_class: INTERNAL_ONLY
    pub assumable_by: Vec<CloudIamPrincipalRef>, // data_class: INTERNAL_ONLY
    pub max_session_duration_sec: u32,           // data_class: INTERNAL_ONLY
    pub data_class: String,                      // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub schema_version: u32,                     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamStsSessionRecord {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub session_id: String,            // data_class: INTERNAL_ONLY
    pub role_id: String,               // data_class: INTERNAL_ONLY
    pub assumed_by: String,            // data_class: INTERNAL_ONLY
    pub external_id: Option<String>,   // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub scopes: Vec<CloudIamScopeRef>, // data_class: INTERNAL_ONLY
    pub token_fingerprint: String,     // data_class: INTERNAL_ONLY
    pub data_class: String,            // data_class: PUBLIC
    pub schema_version: u32,           // data_class: PUBLIC
}

impl CloudIamRoleCreateSuccessResponse {
    pub fn created(data: CloudIamRoleRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudIamApiResponseMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

impl CloudIamIdentityProviderCreateSuccessResponse {
    pub fn created(data: CloudIamIdentityProviderRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudIamApiResponseMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

impl CloudIamIdentityProviderUpdateSuccessResponse {
    pub fn updated(data: CloudIamIdentityProviderRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudIamApiResponseMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

impl CloudIamIdentityProviderDeleteSuccessResponse {
    pub fn deleted(data: CloudIamIdentityProviderRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudIamApiResponseMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

impl CloudIamIdentityProviderListSuccessResponse {
    pub fn ok(data: Vec<CloudIamIdentityProviderRecord>, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudIamApiResponseMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

impl CloudIamStsTokenSuccessResponse {
    pub fn ok(data: CloudIamStsSessionRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudIamApiResponseMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamApiErrorResponse {
    pub error: CloudIamApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamApiErrorBody {
    pub code: String,                         // data_class: INTERNAL_ONLY
    pub message: String,                      // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,    // data_class: INTERNAL_ONLY
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub details: Vec<CloudIamApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIamApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIamApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyBoundaryCell,
    EmptyBoundaryRegion,
    PlacementTenantMismatch {
        header_tenant_id: String,    // data_class: INTERNAL_ONLY
        placement_tenant_id: String, // data_class: INTERNAL_ONLY
    },
    MissingPublicApiVersion,
    UnsupportedPublicApiVersion {
        oyatie_version: String, // data_class: PUBLIC
    },
    EmptyIdempotencyKey,
    EmptyPathProviderId,
    EmptyPathRoleId,
    ProviderIdMismatch {
        path_identity_provider_id: String, // data_class: INTERNAL_ONLY
        body_identity_provider_id: String, // data_class: INTERNAL_ONLY
    },
    RoleIdMismatch {
        path_role_id: String, // data_class: INTERNAL_ONLY
        body_role_id: String, // data_class: INTERNAL_ONLY
    },
    TenantMismatch {
        header_tenant_id: String,    // data_class: INTERNAL_ONLY
        principal_tenant_id: String, // data_class: INTERNAL_ONLY
        body_tenant_id: String,      // data_class: INTERNAL_ONLY
    },
    PrincipalMismatch {
        principal_tenant_id: String, // data_class: INTERNAL_ONLY
        principal_id: String,        // data_class: INTERNAL_ONLY
        body_tenant_id: String,      // data_class: INTERNAL_ONLY
        assumed_by: String,          // data_class: INTERNAL_ONLY
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String, // data_class: INTERNAL_ONLY
        principal_tenant_id: String,     // data_class: INTERNAL_ONLY
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String, // data_class: INTERNAL_ONLY
        principal_id: String,               // data_class: INTERNAL_ONLY
    },
    AuthorizationDenied {
        surface: String, // data_class: INTERNAL_ONLY
    },
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
    InvalidDataClassLabel {
        data_class: String, // data_class: PUBLIC
    },
    Iam(CloudIamError),
}

impl CloudIamApiError {
    pub fn identity_provider_create_status(&self) -> CloudIamIdentityProviderCreateApiStatus {
        match self.status_kind() {
            CloudIamApiStatusKind::BadRequest => {
                CloudIamIdentityProviderCreateApiStatus::BadRequest
            }
            CloudIamApiStatusKind::Forbidden => CloudIamIdentityProviderCreateApiStatus::Forbidden,
            CloudIamApiStatusKind::Conflict => CloudIamIdentityProviderCreateApiStatus::Conflict,
            CloudIamApiStatusKind::UnprocessableEntity => {
                CloudIamIdentityProviderCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn identity_provider_update_status(&self) -> CloudIamIdentityProviderUpdateApiStatus {
        match self.status_kind() {
            CloudIamApiStatusKind::BadRequest => {
                CloudIamIdentityProviderUpdateApiStatus::BadRequest
            }
            CloudIamApiStatusKind::Forbidden => CloudIamIdentityProviderUpdateApiStatus::Forbidden,
            CloudIamApiStatusKind::Conflict => CloudIamIdentityProviderUpdateApiStatus::Conflict,
            CloudIamApiStatusKind::UnprocessableEntity => {
                CloudIamIdentityProviderUpdateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn identity_provider_delete_status(&self) -> CloudIamIdentityProviderDeleteApiStatus {
        match self.status_kind() {
            CloudIamApiStatusKind::BadRequest => {
                CloudIamIdentityProviderDeleteApiStatus::BadRequest
            }
            CloudIamApiStatusKind::Forbidden => CloudIamIdentityProviderDeleteApiStatus::Forbidden,
            CloudIamApiStatusKind::Conflict => CloudIamIdentityProviderDeleteApiStatus::Conflict,
            CloudIamApiStatusKind::UnprocessableEntity => {
                CloudIamIdentityProviderDeleteApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn role_create_status(&self) -> CloudIamRoleCreateApiStatus {
        match self.status_kind() {
            CloudIamApiStatusKind::BadRequest => CloudIamRoleCreateApiStatus::BadRequest,
            CloudIamApiStatusKind::Forbidden => CloudIamRoleCreateApiStatus::Forbidden,
            CloudIamApiStatusKind::Conflict => CloudIamRoleCreateApiStatus::Conflict,
            CloudIamApiStatusKind::UnprocessableEntity => {
                CloudIamRoleCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn sts_token_status(&self) -> CloudIamStsTokenApiStatus {
        match self.status_kind() {
            CloudIamApiStatusKind::BadRequest => CloudIamStsTokenApiStatus::BadRequest,
            CloudIamApiStatusKind::Forbidden => CloudIamStsTokenApiStatus::Forbidden,
            CloudIamApiStatusKind::Conflict => CloudIamStsTokenApiStatus::Conflict,
            CloudIamApiStatusKind::UnprocessableEntity => {
                CloudIamStsTokenApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn role_create_status_code(&self) -> u16 {
        self.role_create_status().code()
    }

    pub fn identity_provider_create_status_code(&self) -> u16 {
        self.identity_provider_create_status().code()
    }

    pub fn identity_provider_update_status_code(&self) -> u16 {
        self.identity_provider_update_status().code()
    }

    pub fn identity_provider_delete_status_code(&self) -> u16 {
        self.identity_provider_delete_status().code()
    }

    pub fn sts_token_status_code(&self) -> u16 {
        self.sts_token_status().code()
    }

    pub fn code(&self) -> CloudIamApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudIamApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudIamApiErrorCode::TenantHeaderEmpty,
            Self::EmptyBoundaryCell => CloudIamApiErrorCode::BoundaryCellEmpty,
            Self::EmptyBoundaryRegion => CloudIamApiErrorCode::BoundaryRegionEmpty,
            Self::PlacementTenantMismatch { .. } => CloudIamApiErrorCode::PlacementTenantMismatch,
            Self::MissingPublicApiVersion => CloudIamApiErrorCode::PublicApiVersionMissing,
            Self::UnsupportedPublicApiVersion { .. } => {
                CloudIamApiErrorCode::PublicApiVersionUnsupported
            }
            Self::EmptyIdempotencyKey => CloudIamApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPathProviderId => CloudIamApiErrorCode::PathProviderIdEmpty,
            Self::EmptyPathRoleId => CloudIamApiErrorCode::PathRoleIdEmpty,
            Self::ProviderIdMismatch { .. } => CloudIamApiErrorCode::ProviderIdMismatch,
            Self::RoleIdMismatch { .. } => CloudIamApiErrorCode::RoleIdMismatch,
            Self::TenantMismatch { .. } => CloudIamApiErrorCode::TenantMismatch,
            Self::PrincipalMismatch { .. } => CloudIamApiErrorCode::PrincipalMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudIamApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudIamApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudIamApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudIamApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudIamApiErrorCode::IdempotencyKeyReused,
            Self::InvalidDataClassLabel { .. } => CloudIamApiErrorCode::DataClassInvalid,
            Self::Iam(error) => match cloud_iam_status_kind(error) {
                CloudIamApiStatusKind::Conflict => CloudIamApiErrorCode::IamConflict,
                CloudIamApiStatusKind::Forbidden => CloudIamApiErrorCode::IamForbidden,
                CloudIamApiStatusKind::BadRequest | CloudIamApiStatusKind::UnprocessableEntity => {
                    CloudIamApiErrorCode::IamInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudIamApiErrorResponse {
        CloudIamApiErrorResponse {
            error: CloudIamApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudIamApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::PrincipalMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudIamApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudIamApiStatusKind::UnprocessableEntity,
            Self::Iam(error) => cloud_iam_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyBoundaryCell
            | Self::EmptyBoundaryRegion
            | Self::PlacementTenantMismatch { .. }
            | Self::MissingPublicApiVersion
            | Self::UnsupportedPublicApiVersion { .. }
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathProviderId
            | Self::EmptyPathRoleId
            | Self::ProviderIdMismatch { .. }
            | Self::RoleIdMismatch { .. }
            | Self::InvalidDataClassLabel { .. } => CloudIamApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyBoundaryCell => "Cloud IAM cell boundary is required",
            Self::EmptyBoundaryRegion => "Cloud IAM region boundary is required",
            Self::PlacementTenantMismatch { .. } => {
                "Cloud IAM typed placement tenant must match request tenant"
            }
            Self::MissingPublicApiVersion => "Oyatie-Version header is required",
            Self::UnsupportedPublicApiVersion { .. } => {
                "Oyatie-Version header must be a supported YYYY-MM-DD public API version"
            }
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPathProviderId => "Path identity provider id is required",
            Self::EmptyPathRoleId => "Path role id is required",
            Self::ProviderIdMismatch { .. } => "Path and body identity provider ids must match",
            Self::RoleIdMismatch { .. } => "Path and body role ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::PrincipalMismatch { .. } => {
                "Authenticated principal must match the STS assumed_by subject"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud IAM surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidDataClassLabel { .. } => "Request data_class must be a known data class",
            Self::Iam(error) => cloud_iam_message(error),
        }
    }

    fn details(&self) -> Vec<CloudIamApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyBoundaryCell => vec![detail("boundary.cell_id", "must be non-empty")],
            Self::EmptyBoundaryRegion => vec![detail("boundary.region_id", "must be non-empty")],
            Self::PlacementTenantMismatch { .. } => vec![detail(
                "boundary.tenant_id",
                "must match the request tenant header before IAM logic executes",
            )],
            Self::MissingPublicApiVersion => vec![detail(
                "header.Oyatie-Version",
                "must be a non-empty YYYY-MM-DD public API version",
            )],
            Self::UnsupportedPublicApiVersion { .. } => vec![detail(
                "header.Oyatie-Version",
                "must match a Cloud IAM supported public API version",
            )],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPathProviderId => {
                vec![detail("path.identity_provider_id", "must be non-empty")]
            }
            Self::EmptyPathRoleId => vec![detail("path.role_id", "must be non-empty")],
            Self::ProviderIdMismatch { .. } => vec![detail(
                "identity_provider_id",
                "path identity_provider_id and body identity_provider_id must match",
            )],
            Self::RoleIdMismatch { .. } => vec![detail(
                "role_id",
                "path role_id and body role_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
            )],
            Self::PrincipalMismatch { .. } => vec![detail(
                "principal",
                "authenticated subject must match body assumed_by and tenant_id",
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
                "must include the requested Cloud IAM surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::Iam(error) => vec![detail("cloud_iam", cloud_iam_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudIamApiStatusKind {
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_iam_identity_provider_create_request(
    request: &CloudIamIdentityProviderCreateApiRequest,
) -> Result<(), CloudIamApiError> {
    validate_boundary(&request.boundary)?;
    if request.path_identity_provider_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyPathProviderId);
    }
    if request.path_identity_provider_id != request.body.identity_provider_id {
        return Err(CloudIamApiError::ProviderIdMismatch {
            path_identity_provider_id: request.path_identity_provider_id.clone(),
            body_identity_provider_id: request.body.identity_provider_id.clone(),
        });
    }
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_IAM_IDENTITY_PROVIDER_CREATE_SURFACE,
    )
}

pub fn validate_cloud_iam_identity_provider_update_request(
    request: &CloudIamIdentityProviderUpdateApiRequest,
) -> Result<(), CloudIamApiError> {
    validate_boundary(&request.boundary)?;
    if request.path_identity_provider_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyPathProviderId);
    }
    if request.path_identity_provider_id != request.body.identity_provider_id {
        return Err(CloudIamApiError::ProviderIdMismatch {
            path_identity_provider_id: request.path_identity_provider_id.clone(),
            body_identity_provider_id: request.body.identity_provider_id.clone(),
        });
    }
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_IAM_IDENTITY_PROVIDER_UPDATE_SURFACE,
    )
}

pub fn validate_cloud_iam_identity_provider_delete_request(
    request: &CloudIamIdentityProviderDeleteApiRequest,
) -> Result<(), CloudIamApiError> {
    validate_boundary(&request.boundary)?;
    if request.path_identity_provider_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyPathProviderId);
    }
    validate_tenant_binding(&request.boundary, &request.principal, &request.tenant_id)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE,
    )
}

pub fn validate_cloud_iam_identity_provider_list_request(
    request: &CloudIamIdentityProviderListApiRequest,
) -> Result<(), CloudIamApiError> {
    validate_read_boundary(&request.boundary)?;
    validate_read_tenant_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_IAM_IDENTITY_PROVIDER_LIST_SURFACE,
    )
}

pub fn validate_cloud_iam_role_create_request(
    request: &CloudIamRoleCreateApiRequest,
) -> Result<(), CloudIamApiError> {
    validate_boundary(&request.boundary)?;
    if request.path_role_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyPathRoleId);
    }
    if request.path_role_id != request.body.role_id {
        return Err(CloudIamApiError::RoleIdMismatch {
            path_role_id: request.path_role_id.clone(),
            body_role_id: request.body.role_id.clone(),
        });
    }
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_IAM_ROLE_CREATE_SURFACE,
    )
}

pub fn validate_cloud_iam_sts_token_request(
    request: &CloudIamStsTokenApiRequest,
) -> Result<(), CloudIamApiError> {
    validate_boundary(&request.boundary)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    if request.principal.principal_id != request.body.assumed_by {
        return Err(CloudIamApiError::PrincipalMismatch {
            principal_tenant_id: request.principal.tenant_id.clone(),
            principal_id: request.principal.principal_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
            assumed_by: request.body.assumed_by.clone(),
        });
    }
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_IAM_STS_TOKEN_SURFACE,
    )
}

pub fn create_cloud_iam_identity_provider_from_api(
    directory: &mut IamDirectory,
    idempotency_ledger: &mut CloudIamIdentityProviderCreateIdempotencyLedger,
    request: CloudIamIdentityProviderCreateApiRequest,
) -> Result<CloudIamIdentityProviderCreateSuccessResponse, CloudIamApiError> {
    validate_cloud_iam_identity_provider_create_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_IAM_IDENTITY_PROVIDER_CREATE_SURFACE,
    );
    let fingerprint = identity_provider_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = directory
        .register_identity_provider(identity_provider_create_input(request.body))
        .map_err(CloudIamApiError::Iam)
        .map(|provider| {
            CloudIamIdentityProviderCreateSuccessResponse::created(
                identity_provider_record(provider),
                request_id,
            )
        });
    idempotency_ledger.entries.insert(
        key,
        CloudIamIdentityProviderCreateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

pub fn update_cloud_iam_identity_provider_from_api(
    directory: &mut IamDirectory,
    idempotency_ledger: &mut CloudIamIdentityProviderUpdateIdempotencyLedger,
    request: CloudIamIdentityProviderUpdateApiRequest,
) -> Result<CloudIamIdentityProviderUpdateSuccessResponse, CloudIamApiError> {
    validate_cloud_iam_identity_provider_update_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_IAM_IDENTITY_PROVIDER_UPDATE_SURFACE,
    );
    let fingerprint = identity_provider_update_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = directory
        .update_identity_provider(identity_provider_update_input(request.body))
        .map_err(CloudIamApiError::Iam)
        .map(|provider| {
            CloudIamIdentityProviderUpdateSuccessResponse::updated(
                identity_provider_record(provider),
                request_id,
            )
        });
    idempotency_ledger.entries.insert(
        key,
        CloudIamIdentityProviderUpdateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

pub fn delete_cloud_iam_identity_provider_from_api(
    directory: &mut IamDirectory,
    idempotency_ledger: &mut CloudIamIdentityProviderDeleteIdempotencyLedger,
    request: CloudIamIdentityProviderDeleteApiRequest,
) -> Result<CloudIamIdentityProviderDeleteSuccessResponse, CloudIamApiError> {
    validate_cloud_iam_identity_provider_delete_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE,
    );
    let fingerprint = identity_provider_delete_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = directory
        .delete_identity_provider(&request.tenant_id, &request.path_identity_provider_id)
        .map_err(CloudIamApiError::Iam)
        .map(|provider| {
            CloudIamIdentityProviderDeleteSuccessResponse::deleted(
                identity_provider_record(provider),
                request_id,
            )
        });
    idempotency_ledger.entries.insert(
        key,
        CloudIamIdentityProviderDeleteIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

pub fn list_cloud_iam_identity_providers_from_api(
    directory: &IamDirectory,
    request: CloudIamIdentityProviderListApiRequest,
) -> Result<CloudIamIdentityProviderListSuccessResponse, CloudIamApiError> {
    validate_cloud_iam_identity_provider_list_request(&request)?;
    let request_id = request.boundary.request_id.clone();
    let providers = directory
        .list_identity_providers(&request.boundary.tenant_id)
        .map_err(CloudIamApiError::Iam)?
        .into_iter()
        .map(identity_provider_record)
        .collect();
    Ok(CloudIamIdentityProviderListSuccessResponse::ok(
        providers, request_id,
    ))
}

pub fn create_cloud_iam_role_from_api(
    directory: &mut IamDirectory,
    idempotency_ledger: &mut CloudIamRoleCreateIdempotencyLedger,
    request: CloudIamRoleCreateApiRequest,
) -> Result<CloudIamRoleCreateSuccessResponse, CloudIamApiError> {
    validate_cloud_iam_role_create_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_IAM_ROLE_CREATE_SURFACE,
    );
    let fingerprint = role_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = role_create_input(request.body)
        .and_then(|input| directory.create_role(input).map_err(CloudIamApiError::Iam))
        .map(|role| CloudIamRoleCreateSuccessResponse::created(role_record(role), request_id));
    idempotency_ledger.entries.insert(
        key,
        CloudIamRoleCreateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

pub fn issue_cloud_iam_sts_token_from_api(
    directory: &mut IamDirectory,
    idempotency_ledger: &mut CloudIamStsTokenIdempotencyLedger,
    request: CloudIamStsTokenApiRequest,
) -> Result<CloudIamStsTokenSuccessResponse, CloudIamApiError> {
    validate_cloud_iam_sts_token_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_IAM_STS_TOKEN_SURFACE,
    );
    let fingerprint = sts_token_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudIamApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = directory
        .assume_role(assume_role_request(request.body))
        .map_err(CloudIamApiError::Iam)
        .map(|session| CloudIamStsTokenSuccessResponse::ok(session_record(session), request_id));
    idempotency_ledger.entries.insert(
        key,
        CloudIamStsTokenIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(boundary: &CloudIamApiBoundaryContext) -> Result<(), CloudIamApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyTenantHeader);
    }
    validate_placement_boundary(&boundary.tenant_id, &boundary.placement)?;
    validate_public_api_version(&boundary.oyatie_version)?;
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudIamApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_read_boundary(
    boundary: &CloudIamApiReadBoundaryContext,
) -> Result<(), CloudIamApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyTenantHeader);
    }
    validate_placement_boundary(&boundary.tenant_id, &boundary.placement)?;
    validate_public_api_version(&boundary.oyatie_version)?;
    Ok(())
}

fn validate_placement_boundary(
    tenant_id: &str,
    placement: &CloudIamApiPlacementBoundary,
) -> Result<(), CloudIamApiError> {
    if placement.tenant_id.value.trim().is_empty() {
        return Err(CloudIamApiError::EmptyTenantHeader);
    }
    if placement.tenant_id.value != tenant_id {
        return Err(CloudIamApiError::PlacementTenantMismatch {
            header_tenant_id: tenant_id.to_string(),
            placement_tenant_id: placement.tenant_id.value.clone(),
        });
    }
    if placement.cell_id.value.trim().is_empty() {
        return Err(CloudIamApiError::EmptyBoundaryCell);
    }
    if placement.region_id.value.trim().is_empty() {
        return Err(CloudIamApiError::EmptyBoundaryRegion);
    }
    Ok(())
}

fn validate_public_api_version(oyatie_version: &str) -> Result<(), CloudIamApiError> {
    let normalized_version = oyatie_version.trim();
    if normalized_version.is_empty() {
        return Err(CloudIamApiError::MissingPublicApiVersion);
    }
    if !CLOUD_IAM_SUPPORTED_PUBLIC_API_VERSIONS.contains(&oyatie_version) {
        return Err(CloudIamApiError::UnsupportedPublicApiVersion {
            oyatie_version: oyatie_version.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudIamApiBoundaryContext,
    principal: &CloudIamApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudIamApiError> {
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudIamApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_read_tenant_binding(
    boundary: &CloudIamApiReadBoundaryContext,
    principal: &CloudIamApiPrincipal,
) -> Result<(), CloudIamApiError> {
    if boundary.tenant_id != principal.tenant_id {
        return Err(CloudIamApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: boundary.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudIamApiPrincipal,
    authorization: &CloudIamApiAuthorization,
    surface: &str,
) -> Result<(), CloudIamApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudIamApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudIamApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudIamApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudIamApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn identity_provider_create_input(
    body: CloudIamIdentityProviderCreateRequest,
) -> IdentityProviderCreate {
    IdentityProviderCreate {
        id: body.identity_provider_id,
        tenant_id: body.tenant_id,
        region_pack: body.region_pack,
        kind: identity_provider_kind_from_api(body.kind),
        issuer_uri: body.issuer_uri,
        audience: body.audience,
        verification_material_ref: body.verification_material_ref,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    }
}

fn identity_provider_update_input(
    body: CloudIamIdentityProviderUpdateRequest,
) -> IdentityProviderUpdate {
    IdentityProviderUpdate {
        id: body.identity_provider_id,
        tenant_id: body.tenant_id,
        region_pack: body.region_pack,
        kind: identity_provider_kind_from_api(body.kind),
        issuer_uri: body.issuer_uri,
        audience: body.audience,
        verification_material_ref: body.verification_material_ref,
    }
}

fn identity_provider_kind_from_api(kind: CloudIamIdentityProviderKind) -> IdentityProviderKind {
    match kind {
        CloudIamIdentityProviderKind::Saml => IdentityProviderKind::Saml,
        CloudIamIdentityProviderKind::Oidc => IdentityProviderKind::Oidc,
    }
}

fn identity_provider_kind_to_api(kind: IdentityProviderKind) -> CloudIamIdentityProviderKind {
    match kind {
        IdentityProviderKind::Saml => CloudIamIdentityProviderKind::Saml,
        IdentityProviderKind::Oidc => CloudIamIdentityProviderKind::Oidc,
    }
}

fn role_create_input(body: CloudIamRoleCreateRequest) -> Result<IamRoleCreate, CloudIamApiError> {
    let data_class = parse_data_class_label(&body.data_class).ok_or_else(|| {
        CloudIamApiError::InvalidDataClassLabel {
            data_class: body.data_class.clone(),
        }
    })?;
    Ok(IamRoleCreate {
        id: body.role_id,
        tenant_id: body.tenant_id,
        region: body.region,
        name: body.name,
        cedar_policy_id: body.cedar_policy_id,
        cedar_policy_version: body.cedar_policy_version,
        assumable_by: body
            .assumable_by
            .into_iter()
            .map(|principal| principal.value)
            .collect(),
        max_session_duration_sec: body.max_session_duration_sec,
        data_class,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn assume_role_request(body: CloudIamStsTokenRequest) -> AssumeRoleRequest {
    AssumeRoleRequest {
        session_id: body.session_id,
        tenant_id: body.tenant_id,
        role_id: body.role_id,
        assumed_by: body.assumed_by,
        external_id: body.external_id,
        requested_duration_sec: body.requested_duration_sec,
        scopes: body.scopes.into_iter().map(|scope| scope.value).collect(),
        issued_at_epoch_seconds: body.issued_at_epoch_seconds,
    }
}

fn idempotency_key_for(
    boundary: &CloudIamApiBoundaryContext,
    principal: &CloudIamApiPrincipal,
    surface: &str,
) -> CloudIamIdempotencyLedgerKey {
    CloudIamIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn identity_provider_create_fingerprint_for(
    request: &CloudIamIdentityProviderCreateApiRequest,
) -> CloudIamRequestFingerprint {
    CloudIamRequestFingerprint {
        canonical: [
            format!(
                "path.identity_provider_id={}",
                request.path_identity_provider_id
            ),
            format!("header.Oyatie-Version={}", request.boundary.oyatie_version),
            format!(
                "boundary.tenant_id={}",
                request.boundary.placement.tenant_id.value
            ),
            format!(
                "boundary.cell_id={}",
                request.boundary.placement.cell_id.value
            ),
            format!(
                "boundary.region_id={}",
                request.boundary.placement.region_id.value
            ),
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
            format!("body.tenant_id={}", request.body.tenant_id),
            format!(
                "body.identity_provider_id={}",
                request.body.identity_provider_id
            ),
            format!("body.region_pack={}", request.body.region_pack),
            format!("body.kind={:?}", request.body.kind),
            format!("body.issuer_uri={}", request.body.issuer_uri),
            format!("body.audience={}", request.body.audience),
            format!(
                "body.verification_material_ref={}",
                request.body.verification_material_ref
            ),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn identity_provider_update_fingerprint_for(
    request: &CloudIamIdentityProviderUpdateApiRequest,
) -> CloudIamRequestFingerprint {
    CloudIamRequestFingerprint {
        canonical: [
            format!(
                "path.identity_provider_id={}",
                request.path_identity_provider_id
            ),
            format!("header.Oyatie-Version={}", request.boundary.oyatie_version),
            format!(
                "boundary.tenant_id={}",
                request.boundary.placement.tenant_id.value
            ),
            format!(
                "boundary.cell_id={}",
                request.boundary.placement.cell_id.value
            ),
            format!(
                "boundary.region_id={}",
                request.boundary.placement.region_id.value
            ),
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
            format!("body.tenant_id={}", request.body.tenant_id),
            format!(
                "body.identity_provider_id={}",
                request.body.identity_provider_id
            ),
            format!("body.region_pack={}", request.body.region_pack),
            format!("body.kind={:?}", request.body.kind),
            format!("body.issuer_uri={}", request.body.issuer_uri),
            format!("body.audience={}", request.body.audience),
            format!(
                "body.verification_material_ref={}",
                request.body.verification_material_ref
            ),
        ]
        .join("|"),
    }
}

fn identity_provider_delete_fingerprint_for(
    request: &CloudIamIdentityProviderDeleteApiRequest,
) -> CloudIamRequestFingerprint {
    CloudIamRequestFingerprint {
        canonical: [
            format!(
                "path.identity_provider_id={}",
                request.path_identity_provider_id
            ),
            format!("header.Oyatie-Version={}", request.boundary.oyatie_version),
            format!(
                "boundary.tenant_id={}",
                request.boundary.placement.tenant_id.value
            ),
            format!(
                "boundary.cell_id={}",
                request.boundary.placement.cell_id.value
            ),
            format!(
                "boundary.region_id={}",
                request.boundary.placement.region_id.value
            ),
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
            format!("body.tenant_id={}", request.tenant_id),
        ]
        .join("|"),
    }
}

fn role_create_fingerprint_for(
    request: &CloudIamRoleCreateApiRequest,
) -> CloudIamRequestFingerprint {
    CloudIamRequestFingerprint {
        canonical: [
            format!("path.role_id={}", request.path_role_id),
            format!("header.Oyatie-Version={}", request.boundary.oyatie_version),
            format!(
                "boundary.tenant_id={}",
                request.boundary.placement.tenant_id.value
            ),
            format!(
                "boundary.cell_id={}",
                request.boundary.placement.cell_id.value
            ),
            format!(
                "boundary.region_id={}",
                request.boundary.placement.region_id.value
            ),
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
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.role_id={}", request.body.role_id),
            format!("body.region={}", request.body.region),
            format!("body.name={}", request.body.name),
            format!("body.cedar_policy_id={}", request.body.cedar_policy_id),
            format!(
                "body.cedar_policy_version={}",
                request.body.cedar_policy_version
            ),
            format!(
                "body.assumable_by={}",
                request
                    .body
                    .assumable_by
                    .iter()
                    .map(|principal| principal.value.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            format!(
                "body.max_session_duration_sec={}",
                request.body.max_session_duration_sec
            ),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn sts_token_fingerprint_for(request: &CloudIamStsTokenApiRequest) -> CloudIamRequestFingerprint {
    CloudIamRequestFingerprint {
        canonical: [
            format!("header.Oyatie-Version={}", request.boundary.oyatie_version),
            format!(
                "boundary.tenant_id={}",
                request.boundary.placement.tenant_id.value
            ),
            format!(
                "boundary.cell_id={}",
                request.boundary.placement.cell_id.value
            ),
            format!(
                "boundary.region_id={}",
                request.boundary.placement.region_id.value
            ),
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
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.session_id={}", request.body.session_id),
            format!("body.role_id={}", request.body.role_id),
            format!("body.assumed_by={}", request.body.assumed_by),
            format!(
                "body.external_id={}",
                request.body.external_id.as_deref().unwrap_or("")
            ),
            format!(
                "body.requested_duration_sec={}",
                request.body.requested_duration_sec
            ),
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

fn identity_provider_record(provider: IdentityProvider) -> CloudIamIdentityProviderRecord {
    CloudIamIdentityProviderRecord {
        tenant_id: provider.tenant_id.value,
        identity_provider_id: provider.id.value.value,
        region_pack: provider.region_pack.value,
        kind: identity_provider_kind_to_api(provider.kind.value),
        issuer_uri: provider.issuer_uri.value,
        audience: provider.audience.value,
        verification_material_ref: provider.verification_material_ref.value,
        created_at_epoch_seconds: provider.created_at_epoch_seconds.value,
        schema_version: provider.schema_version.value,
    }
}

fn role_record(role: IamRole) -> CloudIamRoleRecord {
    CloudIamRoleRecord {
        tenant_id: role.tenant_id.value,
        role_id: role.id.value.value,
        region: role.region.value.value,
        name: role.name.value.value,
        cedar_policy_id: role.cedar_policy_id.value.value,
        cedar_policy_version: role.cedar_policy_version.value,
        assumable_by: role
            .assumable_by
            .value
            .into_iter()
            .map(|principal| CloudIamPrincipalRef {
                value: principal.value,
            })
            .collect(),
        max_session_duration_sec: role.max_session_duration_sec.value,
        data_class: role.data_class.value.label().to_string(),
        created_at_epoch_seconds: role.created_at_epoch_seconds.value,
        schema_version: role.schema_version.value,
    }
}

fn session_record(session: StsSession) -> CloudIamStsSessionRecord {
    CloudIamStsSessionRecord {
        tenant_id: session.tenant_id.value,
        session_id: session.id.value.value,
        role_id: session.assumed_role.value.value,
        assumed_by: session.assumed_by.value.value,
        external_id: session.external_id.value,
        issued_at_epoch_seconds: session.issued_at_epoch_seconds.value,
        expires_at_epoch_seconds: session.expires_at_epoch_seconds.value,
        scopes: session
            .scopes
            .value
            .into_iter()
            .map(|scope| CloudIamScopeRef { value: scope.value })
            .collect(),
        token_fingerprint: session.token_fingerprint.value,
        data_class: session.data_class.value.label().to_string(),
        schema_version: session.schema_version.value,
    }
}

fn cloud_iam_status_kind(error: &CloudIamError) -> CloudIamApiStatusKind {
    match error {
        CloudIamError::DuplicateProvider
        | CloudIamError::DuplicatePrincipal
        | CloudIamError::DuplicateRole
        | CloudIamError::DuplicateSession
        | CloudIamError::DuplicateIdentityProviderRegistrySnapshot
        | CloudIamError::DuplicateIdentityProviderRegistryRecord
        | CloudIamError::ProviderInUse => CloudIamApiStatusKind::Conflict,
        CloudIamError::PrincipalCannotAssumeRole
        | CloudIamError::MfaNotVerified
        | CloudIamError::ExternalIdRequired
        | CloudIamError::ProviderTenantMismatch
        | CloudIamError::TrustPolicyDenied
        | CloudIamError::TenantMismatch => CloudIamApiStatusKind::Forbidden,
        CloudIamError::InvalidTenantId
        | CloudIamError::InvalidPrincipalId
        | CloudIamError::InvalidRoleId
        | CloudIamError::InvalidProviderId
        | CloudIamError::InvalidCedarPolicyId
        | CloudIamError::InvalidRoleName
        | CloudIamError::InvalidScope
        | CloudIamError::InvalidSubjectUri
        | CloudIamError::InvalidRegionalPack
        | CloudIamError::InvalidIssuerUri
        | CloudIamError::InvalidAudience
        | CloudIamError::InvalidVerificationMaterialRef
        | CloudIamError::InvalidSessionId
        | CloudIamError::InvalidExternalId
        | CloudIamError::InvalidIdentityProviderRegistrySnapshotId
        | CloudIamError::InvalidIdentityProviderRegistrySnapshotSchemaVersion
        | CloudIamError::InvalidDataClass
        | CloudIamError::InvalidSemver
        | CloudIamError::InvalidSessionDuration
        | CloudIamError::EmptyIdentityProviderRegistrySnapshot
        | CloudIamError::IdentityProviderRegistryRawMaterialForbidden
        | CloudIamError::MissingAssumablePrincipal
        | CloudIamError::DuplicateAssumablePrincipal
        | CloudIamError::DuplicateScope
        | CloudIamError::PrincipalKindMismatch
        | CloudIamError::ProviderRequired
        | CloudIamError::InvalidProviderEvidenceRef
        | CloudIamError::ProviderMismatch
        | CloudIamError::MissingExternalSubject
        | CloudIamError::UnexpectedExternalSubject
        | CloudIamError::UnknownProvider
        | CloudIamError::UnknownPrincipal
        | CloudIamError::UnknownRole
        | CloudIamError::PlatformIdentityRejected(_) => CloudIamApiStatusKind::BadRequest,
    }
}

fn cloud_iam_message(error: &CloudIamError) -> &'static str {
    match cloud_iam_status_kind(error) {
        CloudIamApiStatusKind::Conflict => "Cloud IAM resource already exists",
        CloudIamApiStatusKind::Forbidden => "Cloud IAM policy denied the request",
        CloudIamApiStatusKind::BadRequest => "Cloud IAM rejected the request shape",
        CloudIamApiStatusKind::UnprocessableEntity => "Cloud IAM rejected request idempotency",
    }
}

fn cloud_iam_issue(error: &CloudIamError) -> &'static str {
    match error {
        CloudIamError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudIamError::InvalidPrincipalId => "principal id must match principal kind",
        CloudIamError::InvalidRoleId => "role_id must be a role_ identifier",
        CloudIamError::InvalidProviderId => "identity_provider_id must be an idp_ identifier",
        CloudIamError::InvalidCedarPolicyId => "cedar_policy_id must be a pol_ identifier",
        CloudIamError::InvalidRoleName => "role name must be canonical lowercase",
        CloudIamError::InvalidScope => "STS scopes must be non-empty cloud.* scopes",
        CloudIamError::InvalidSubjectUri => "federated subject must be saml:// or oidc://",
        CloudIamError::InvalidRegionalPack => "regional pack must use pack- prefix",
        CloudIamError::InvalidIssuerUri => "issuer_uri must be https",
        CloudIamError::InvalidAudience => "audience must be non-empty",
        CloudIamError::InvalidVerificationMaterialRef => {
            "verification material must match provider kind"
        }
        CloudIamError::InvalidSessionId => "session_id must be a sts_ identifier",
        CloudIamError::InvalidExternalId => "external_id must be non-empty when present",
        CloudIamError::InvalidIdentityProviderRegistrySnapshotId => {
            "identity provider registry snapshot id must be non-empty metadata"
        }
        CloudIamError::InvalidIdentityProviderRegistrySnapshotSchemaVersion => {
            "identity provider registry snapshot schema version is unsupported"
        }
        CloudIamError::InvalidDataClass => "role data_class must be a public privacy class",
        CloudIamError::InvalidSemver => "cedar_policy_version must be semver",
        CloudIamError::InvalidSessionDuration => {
            "session duration must be >0 and <= role/platform limit"
        }
        CloudIamError::MissingAssumablePrincipal => "role must trust at least one principal",
        CloudIamError::EmptyIdentityProviderRegistrySnapshot => {
            "identity provider registry snapshot must contain at least one record"
        }
        CloudIamError::IdentityProviderRegistryRawMaterialForbidden => {
            "identity provider registry snapshots must not contain raw provider, credential, assertion, or STS material"
        }
        CloudIamError::DuplicateAssumablePrincipal => "role trust policy has duplicate principals",
        CloudIamError::DuplicateScope => "STS scope list has duplicate scopes",
        CloudIamError::PrincipalKindMismatch => "principal kind does not match identifier",
        CloudIamError::PrincipalCannotAssumeRole => "role principals cannot assume roles",
        CloudIamError::MfaNotVerified => "principal must have verified MFA",
        CloudIamError::ExternalIdRequired => "external principal requires an external_id",
        CloudIamError::ProviderRequired => {
            "federated/external principal requires an identity provider"
        }
        CloudIamError::ProviderTenantMismatch => {
            "identity provider tenant must match principal tenant"
        }
        CloudIamError::ProviderInUse => {
            "identity provider cannot be deleted while principals reference it"
        }
        CloudIamError::InvalidProviderEvidenceRef => {
            "provider evidence ref must use an allowlisted opaque IdP evidence-ref scheme"
        }
        CloudIamError::ProviderMismatch => {
            "identity provider registry evidence provider does not match expected provider"
        }
        CloudIamError::MissingExternalSubject => {
            "federated/external principal requires external_subject"
        }
        CloudIamError::UnexpectedExternalSubject => {
            "local principal must not carry external federation fields"
        }
        CloudIamError::DuplicateProvider
        | CloudIamError::DuplicatePrincipal
        | CloudIamError::DuplicateRole
        | CloudIamError::DuplicateSession
        | CloudIamError::DuplicateIdentityProviderRegistrySnapshot
        | CloudIamError::DuplicateIdentityProviderRegistryRecord => {
            "resource identifier is already present"
        }
        CloudIamError::UnknownProvider => {
            "identity provider must exist before principal registration"
        }
        CloudIamError::UnknownPrincipal => "principal must exist and match tenant",
        CloudIamError::UnknownRole => "role must exist before STS issuance",
        CloudIamError::TrustPolicyDenied => "principal is not trusted by the role",
        CloudIamError::TenantMismatch => "tenant-bound IAM resources must match",
        CloudIamError::PlatformIdentityRejected(_) => {
            "platform identity rejected credential issuance"
        }
    }
}

fn detail(field: &str, issue: &str) -> CloudIamApiErrorDetail {
    CloudIamApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
