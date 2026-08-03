//! Cloud KMS API boundary for encrypt and decrypt authorization receipts.
//!
//! This crate owns tenant/header/path/body normalization before handing typed
//! requests to the Cloud KMS kernel.
//!
//! ## Modules
//!
//! - [`authz`] — fail-closed principal-verification + PDP authorization PORTS
//!   for the crown-jewel KMS encrypt/decrypt control plane
//!   (AUTH-005 / C5-crypto class / ADR-0573). The crypto op is unreachable
//!   without a [`authz::VerifiedKmsPrincipal`] (unforgeable) and a passing PDP
//!   [`authz::KmsCryptoAuthorizer`] decision.

pub mod authz;

use std::collections::BTreeMap;

use authz::{
    KmsCryptoAction, KmsCryptoAuthorizationError, KmsCryptoAuthzProvider, KmsCryptoResource,
    VerifiedKmsPrincipal,
};

use cell_region::{CellId, RegionCode};
use data_boundary_kernel::parse_data_class_label;
use secrets_kms_domain::{
    CloudKmsDirectory, CloudKmsError, KmsDecryptRequest, KmsEncryptRequest, KmsOperation,
    KmsPurpose, KmsRepo, KmsUseReceipt,
};

pub const CLOUD_KMS_ENCRYPT_SURFACE: &str = "cloud.kms.encrypt";
pub const CLOUD_KMS_DECRYPT_SURFACE: &str = "cloud.kms.decrypt";
pub const CLOUD_KMS_PUBLIC_API_VERSION_HEADER: &str = "Oyatie-Version";
pub const CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION: &str = "2026-05-21";
pub const CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS: &[&str] =
    &["2026-05-21", "2026-02-21", "2025-11-21"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudKmsCryptoApiStatus {
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudKmsCryptoApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudKmsApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    PublicApiVersionMissing,
    PublicApiVersionUnsupported,
    RegionHeaderEmpty,
    RegionHeaderInvalid,
    CellHeaderEmpty,
    CellHeaderInvalid,
    BoundaryCellRegionMismatch,
    IdempotencyKeyEmpty,
    PathKeyIdEmpty,
    PathKeyIdMalformed,
    KeyIdMismatch,
    TenantMismatch,
    PrincipalMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    PrincipalUnauthenticated,
    VerifiedPrincipalMismatch,
    VerifiedTenantMismatch,
    CryptoAuthorizationDenied,
    IdempotencyKeyReused,
    DataClassInvalid,
    PurposeInvalid,
    KmsInvalidRequest,
    KmsForbidden,
    KmsNotFound,
    KmsConflict,
}

impl CloudKmsApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_KMS_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_KMS_TENANT_HEADER_EMPTY",
            Self::PublicApiVersionMissing => "CLOUD_KMS_PUBLIC_API_VERSION_MISSING",
            Self::PublicApiVersionUnsupported => "CLOUD_KMS_PUBLIC_API_VERSION_UNSUPPORTED",
            Self::RegionHeaderEmpty => "CLOUD_KMS_REGION_HEADER_EMPTY",
            Self::RegionHeaderInvalid => "CLOUD_KMS_REGION_HEADER_INVALID",
            Self::CellHeaderEmpty => "CLOUD_KMS_CELL_HEADER_EMPTY",
            Self::CellHeaderInvalid => "CLOUD_KMS_CELL_HEADER_INVALID",
            Self::BoundaryCellRegionMismatch => "CLOUD_KMS_BOUNDARY_CELL_REGION_MISMATCH",
            Self::IdempotencyKeyEmpty => "CLOUD_KMS_IDEMPOTENCY_KEY_EMPTY",
            Self::PathKeyIdEmpty => "CLOUD_KMS_PATH_KEY_ID_EMPTY",
            Self::PathKeyIdMalformed => "CLOUD_KMS_PATH_KEY_ID_MALFORMED",
            Self::KeyIdMismatch => "CLOUD_KMS_KEY_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_KMS_TENANT_MISMATCH",
            Self::PrincipalMismatch => "CLOUD_KMS_PRINCIPAL_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "CLOUD_KMS_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "CLOUD_KMS_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "CLOUD_KMS_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "CLOUD_KMS_AUTHORIZATION_DENIED",
            Self::PrincipalUnauthenticated => "CLOUD_KMS_PRINCIPAL_UNAUTHENTICATED",
            Self::VerifiedPrincipalMismatch => "CLOUD_KMS_VERIFIED_PRINCIPAL_MISMATCH",
            Self::VerifiedTenantMismatch => "CLOUD_KMS_VERIFIED_TENANT_MISMATCH",
            Self::CryptoAuthorizationDenied => "CLOUD_KMS_CRYPTO_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_KMS_IDEMPOTENCY_KEY_REUSED",
            Self::DataClassInvalid => "CLOUD_KMS_DATA_CLASS_INVALID",
            Self::PurposeInvalid => "CLOUD_KMS_PURPOSE_INVALID",
            Self::KmsInvalidRequest => "CLOUD_KMS_INVALID_REQUEST",
            Self::KmsForbidden => "CLOUD_KMS_FORBIDDEN",
            Self::KmsNotFound => "CLOUD_KMS_NOT_FOUND",
            Self::KmsConflict => "CLOUD_KMS_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub region: String,          // data_class: PUBLIC
    pub cell_id: String,         // data_class: PUBLIC
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    pub oyatie_version: String,  // data_class: PUBLIC
}

/// The caller-asserted principal identity carried alongside the request.
///
/// ## NOT an authority source
///
/// These fields are caller-supplied and therefore forgeable. They are used ONLY
/// as a cross-check against the [`authz::VerifiedKmsPrincipal`] the
/// [`authz::PrincipalVerifier`] bound from an unforgeable credential. A request
/// whose asserted principal/tenant disagrees with the verified identity is
/// rejected (`VerifiedPrincipalMismatch` / `VerifiedTenantMismatch`, 403). The
/// verified principal — never these fields — is the authority for the audit
/// record and the PDP decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

/// A non-authoritative correlation hint carried alongside the request.
///
/// ## NOT an authorization grant (AUTH-005 / C5 remediation)
///
/// Before ADR-0573 this struct carried `{tenant_id, principal_id,
/// allowed_surfaces}` and [`validate_authorization`] "authorized" a KMS crypto
/// op by checking the caller-supplied `allowed_surfaces` contained the requested
/// surface — a forgeable PDP bypass on the crown-jewel crypto surface. Those
/// authority fields are REMOVED. The only remaining field, [`Self::decision_id`],
/// is a correlation id echoed into telemetry; it confers NO authorization. The
/// authorization decision is now made server-side by the
/// [`authz::KmsCryptoAuthorizer`] PDP port over the VERIFIED principal and the
/// trusted resource binding.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudKmsApiAuthorizationCorrelation {
    /// Caller-supplied correlation id (e.g. an upstream PDP decision id). This is
    /// telemetry/audit correlation ONLY — it is never treated as a grant.
    pub decision_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsEncryptRequest {
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub plaintext_ref: String,           // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub data_class: String,              // data_class: INTERNAL_ONLY
    pub purpose: String,                 // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub aad_fingerprint: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsDecryptRequest {
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub data_class: String,              // data_class: INTERNAL_ONLY
    pub purpose: String,                 // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsEncryptApiRequest {
    pub path_key_id: String,                  // data_class: INTERNAL_ONLY
    pub boundary: CloudKmsApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudKmsApiPrincipal,      // data_class: INTERNAL_ONLY (cross-check only)
    pub correlation: CloudKmsApiAuthorizationCorrelation, // data_class: INTERNAL_ONLY (NOT a grant)
    pub body: CloudKmsEncryptRequest,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsDecryptApiRequest {
    pub path_key_id: String,                  // data_class: INTERNAL_ONLY
    pub boundary: CloudKmsApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudKmsApiPrincipal,      // data_class: INTERNAL_ONLY (cross-check only)
    pub correlation: CloudKmsApiAuthorizationCorrelation, // data_class: INTERNAL_ONLY (NOT a grant)
    pub body: CloudKmsDecryptRequest,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudKmsCryptoIdempotencyLedger {
    entries: BTreeMap<CloudKmsIdempotencyLedgerKey, CloudKmsCryptoIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudKmsCryptoIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudKmsIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudKmsCryptoIdempotencyLedgerEntry {
    fingerprint: CloudKmsRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudKmsCryptoApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudKmsRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudKmsCryptoApiResult = Result<CloudKmsCryptoSuccessResponse, CloudKmsApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsCryptoSuccessResponse {
    pub data: CloudKmsUseReceiptRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudKmsApiResponseMetadata, // data_class: INTERNAL_ONLY
}

impl CloudKmsCryptoSuccessResponse {
    pub fn ok(
        data: CloudKmsUseReceiptRecord,
        request_id: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self {
        Self {
            data,
            metadata: CloudKmsApiResponseMetadata {
                request_id: request_id.into(),
                api_version: api_version.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiResponseMetadata {
    pub request_id: String,  // data_class: INTERNAL_ONLY
    pub api_version: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsUseReceiptRecord {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub key_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub operation: String,              // data_class: PUBLIC
    pub material_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,         // data_class: INTERNAL_ONLY
    pub data_class: String,             // data_class: INTERNAL_ONLY
    pub purpose: String,                // data_class: INTERNAL_ONLY
    pub actor: String,                  // data_class: INTERNAL_ONLY
    pub key_version: u32,               // data_class: INTERNAL_ONLY
    pub hsm_partition_ref: String,      // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiErrorResponse {
    pub error: CloudKmsApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiErrorBody {
    pub code: String,                         // data_class: INTERNAL_ONLY
    pub message: String,                      // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,    // data_class: INTERNAL_ONLY
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub details: Vec<CloudKmsApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudKmsApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    MissingPublicApiVersion,
    UnsupportedPublicApiVersion {
        oyatie_version: String, // data_class: PUBLIC
    },
    EmptyRegionHeader,
    InvalidRegionHeader {
        region: String, // data_class: PUBLIC
    },
    EmptyCellHeader,
    InvalidCellHeader {
        cell_id: String, // data_class: PUBLIC
    },
    BoundaryCellRegionMismatch {
        region: String,  // data_class: PUBLIC
        cell_id: String, // data_class: PUBLIC
    },
    EmptyIdempotencyKey,
    EmptyPathKeyId,
    /// The path key id does not conform to the `kms/{region}/{tenant}/{name}` format.
    /// This is an internal invariant violation — the key id was already validated
    /// as non-empty; a parse failure here means the format contract is broken.
    MalformedPathKeyId {
        path_key_id: String, // data_class: INTERNAL_ONLY
    },
    KeyIdMismatch {
        path_key_id: String, // data_class: INTERNAL_ONLY
        body_key_id: String, // data_class: INTERNAL_ONLY
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
        actor: String,               // data_class: INTERNAL_ONLY
    },
    /// No verifiable credential was presented (fail-closed 401). The KMS crypto
    /// op is unreachable without a verified principal.
    PrincipalUnauthenticated,
    /// The caller-asserted principal id disagrees with the verified principal
    /// (fail-closed 403). The verified identity is authoritative.
    VerifiedPrincipalMismatch {
        verified_principal_id: String, // data_class: INTERNAL_ONLY
        claimed_principal_id: String,  // data_class: INTERNAL_ONLY
    },
    /// The caller-asserted tenant disagrees with the verified principal's tenant
    /// (fail-closed 403). Cross-tenant access is denied on this axis.
    VerifiedTenantMismatch {
        verified_tenant_id: String, // data_class: INTERNAL_ONLY
        claimed_tenant_id: String,  // data_class: INTERNAL_ONLY
    },
    /// The server-side PDP denied or refused the crypto op (fail-closed 403).
    /// A PDP fault (timeout/network/unavailability) maps here as deny, NOT 500.
    CryptoAuthorizationDenied {
        surface: String, // data_class: INTERNAL_ONLY
    },
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
    InvalidDataClassLabel {
        data_class: String, // data_class: INTERNAL_ONLY
    },
    InvalidPurposeLabel {
        purpose: String, // data_class: INTERNAL_ONLY
    },
    Kms(CloudKmsError),
}

impl CloudKmsApiError {
    pub fn crypto_status(&self) -> CloudKmsCryptoApiStatus {
        match self.status_kind() {
            CloudKmsApiStatusKind::BadRequest => CloudKmsCryptoApiStatus::BadRequest,
            CloudKmsApiStatusKind::Unauthorized => CloudKmsCryptoApiStatus::Unauthorized,
            CloudKmsApiStatusKind::Forbidden => CloudKmsCryptoApiStatus::Forbidden,
            CloudKmsApiStatusKind::NotFound => CloudKmsCryptoApiStatus::NotFound,
            CloudKmsApiStatusKind::Conflict => CloudKmsCryptoApiStatus::Conflict,
            CloudKmsApiStatusKind::UnprocessableEntity => {
                CloudKmsCryptoApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn crypto_status_code(&self) -> u16 {
        self.crypto_status().code()
    }

    pub fn code(&self) -> CloudKmsApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudKmsApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudKmsApiErrorCode::TenantHeaderEmpty,
            Self::MissingPublicApiVersion => CloudKmsApiErrorCode::PublicApiVersionMissing,
            Self::UnsupportedPublicApiVersion { .. } => {
                CloudKmsApiErrorCode::PublicApiVersionUnsupported
            }
            Self::EmptyRegionHeader => CloudKmsApiErrorCode::RegionHeaderEmpty,
            Self::InvalidRegionHeader { .. } => CloudKmsApiErrorCode::RegionHeaderInvalid,
            Self::EmptyCellHeader => CloudKmsApiErrorCode::CellHeaderEmpty,
            Self::InvalidCellHeader { .. } => CloudKmsApiErrorCode::CellHeaderInvalid,
            Self::BoundaryCellRegionMismatch { .. } => {
                CloudKmsApiErrorCode::BoundaryCellRegionMismatch
            }
            Self::EmptyIdempotencyKey => CloudKmsApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPathKeyId => CloudKmsApiErrorCode::PathKeyIdEmpty,
            Self::MalformedPathKeyId { .. } => CloudKmsApiErrorCode::PathKeyIdMalformed,
            Self::KeyIdMismatch { .. } => CloudKmsApiErrorCode::KeyIdMismatch,
            Self::TenantMismatch { .. } => CloudKmsApiErrorCode::TenantMismatch,
            Self::PrincipalMismatch { .. } => CloudKmsApiErrorCode::PrincipalMismatch,
            Self::PrincipalUnauthenticated => CloudKmsApiErrorCode::PrincipalUnauthenticated,
            Self::VerifiedPrincipalMismatch { .. } => {
                CloudKmsApiErrorCode::VerifiedPrincipalMismatch
            }
            Self::VerifiedTenantMismatch { .. } => CloudKmsApiErrorCode::VerifiedTenantMismatch,
            Self::CryptoAuthorizationDenied { .. } => {
                CloudKmsApiErrorCode::CryptoAuthorizationDenied
            }
            Self::IdempotencyKeyReused { .. } => CloudKmsApiErrorCode::IdempotencyKeyReused,
            Self::InvalidDataClassLabel { .. } => CloudKmsApiErrorCode::DataClassInvalid,
            Self::InvalidPurposeLabel { .. } => CloudKmsApiErrorCode::PurposeInvalid,
            Self::Kms(error) => match cloud_kms_status_kind(error) {
                CloudKmsApiStatusKind::BadRequest => CloudKmsApiErrorCode::KmsInvalidRequest,
                CloudKmsApiStatusKind::Unauthorized | CloudKmsApiStatusKind::Forbidden => {
                    CloudKmsApiErrorCode::KmsForbidden
                }
                CloudKmsApiStatusKind::NotFound => CloudKmsApiErrorCode::KmsNotFound,
                CloudKmsApiStatusKind::Conflict => CloudKmsApiErrorCode::KmsConflict,
                CloudKmsApiStatusKind::UnprocessableEntity => {
                    CloudKmsApiErrorCode::KmsInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudKmsApiErrorResponse {
        CloudKmsApiErrorResponse {
            error: CloudKmsApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudKmsApiStatusKind {
        match self {
            Self::PrincipalUnauthenticated => CloudKmsApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::PrincipalMismatch { .. }
            | Self::VerifiedPrincipalMismatch { .. }
            | Self::VerifiedTenantMismatch { .. }
            | Self::CryptoAuthorizationDenied { .. } => CloudKmsApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudKmsApiStatusKind::UnprocessableEntity,
            Self::Kms(error) => cloud_kms_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::MissingPublicApiVersion
            | Self::UnsupportedPublicApiVersion { .. }
            | Self::EmptyRegionHeader
            | Self::InvalidRegionHeader { .. }
            | Self::EmptyCellHeader
            | Self::InvalidCellHeader { .. }
            | Self::BoundaryCellRegionMismatch { .. }
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathKeyId
            | Self::MalformedPathKeyId { .. }
            | Self::KeyIdMismatch { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidPurposeLabel { .. } => CloudKmsApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::MissingPublicApiVersion => "Oyatie-Version header is required",
            Self::UnsupportedPublicApiVersion { .. } => {
                "Oyatie-Version header must be a supported YYYY-MM-DD public API version"
            }
            Self::EmptyRegionHeader => "X-Region-Code header is required",
            Self::InvalidRegionHeader { .. } => {
                "X-Region-Code header must be a canonical region code"
            }
            Self::EmptyCellHeader => "X-Cell-Id header is required",
            Self::InvalidCellHeader { .. } => {
                "X-Cell-Id header must be a canonical cell identifier"
            }
            Self::BoundaryCellRegionMismatch { .. } => {
                "X-Cell-Id header must belong to X-Region-Code"
            }
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPathKeyId => "Path key id is required",
            Self::MalformedPathKeyId { .. } => {
                "Path key id must conform to kms/{region}/{tenant}/{name}"
            }
            Self::KeyIdMismatch { .. } => "Path and body key ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::PrincipalMismatch { .. } => {
                "Authenticated principal must match the Cloud KMS actor"
            }
            Self::PrincipalUnauthenticated => {
                "A verified caller credential is required for Cloud KMS crypto operations"
            }
            Self::VerifiedPrincipalMismatch { .. } => {
                "Request principal id must match the verified caller principal"
            }
            Self::VerifiedTenantMismatch { .. } => {
                "Request tenant must match the verified caller tenant"
            }
            Self::CryptoAuthorizationDenied { .. } => {
                "Cloud KMS authorization was denied by the policy decision point"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::InvalidPurposeLabel { .. } => "Request purpose must be a known KMS purpose",
            Self::Kms(error) => cloud_kms_message(error),
        }
    }

    fn details(&self) -> Vec<CloudKmsApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::MissingPublicApiVersion => vec![detail(
                "header.Oyatie-Version",
                "must be a non-empty YYYY-MM-DD public API version",
            )],
            Self::UnsupportedPublicApiVersion { .. } => vec![detail(
                "header.Oyatie-Version",
                "must match a Cloud KMS supported public API version",
            )],
            Self::EmptyRegionHeader => vec![detail("header.X-Region-Code", "must be non-empty")],
            Self::InvalidRegionHeader { .. } => vec![detail(
                "header.X-Region-Code",
                "must be a canonical region code",
            )],
            Self::EmptyCellHeader => vec![detail("header.X-Cell-Id", "must be non-empty")],
            Self::InvalidCellHeader { .. } => vec![detail(
                "header.X-Cell-Id",
                "must be a canonical cell identifier",
            )],
            Self::BoundaryCellRegionMismatch { .. } => vec![detail(
                "header.X-Cell-Id",
                "cell must be scoped to header.X-Region-Code",
            )],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPathKeyId => vec![detail("path.key_id", "must be non-empty")],
            Self::MalformedPathKeyId { .. } => vec![detail(
                "path.key_id",
                "must be kms/{region}/{tenant}/{name}",
            )],
            Self::KeyIdMismatch { .. } => {
                vec![detail("key_id", "path key_id and body key_id must match")]
            }
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
            )],
            Self::PrincipalMismatch { .. } => vec![detail(
                "actor",
                "authenticated subject must match body actor and tenant_id",
            )],
            Self::PrincipalUnauthenticated => vec![detail(
                "header.Authorization",
                "a verified caller credential (bearer/mTLS) is required",
            )],
            Self::VerifiedPrincipalMismatch { .. } => vec![detail(
                "principal.principal_id",
                "must match the verified caller principal",
            )],
            Self::VerifiedTenantMismatch { .. } => vec![detail(
                "principal.tenant_id",
                "must match the verified caller tenant",
            )],
            Self::CryptoAuthorizationDenied { .. } => vec![detail(
                "authorization",
                "policy decision point denied the requested Cloud KMS surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::InvalidPurposeLabel { .. } => vec![detail(
                "body.purpose",
                "must be a canonical Cloud KMS purpose label",
            )],
            Self::Kms(error) => vec![detail("cloud_kms", cloud_kms_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudKmsApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

/// Validate the request shape and consistency.
///
/// This performs only structural/consistency validation (boundary headers,
/// path/body key binding, tenant/actor internal consistency). It does NOT
/// perform authorization — authorization is the server-side PDP decision in
/// [`authorize_cloud_kms_encrypt_from_api`] over the [`VerifiedKmsPrincipal`].
pub fn validate_cloud_kms_encrypt_request(
    request: &CloudKmsEncryptApiRequest,
) -> Result<(), CloudKmsApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_key_id(&request.path_key_id, &request.body.key_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_principal_actor(
        &request.principal,
        &request.body.tenant_id,
        &request.body.actor,
    )
}

/// Validate the request shape and consistency (decrypt). See
/// [`validate_cloud_kms_encrypt_request`]; authorization is the server-side PDP
/// decision, not a request-shape check.
pub fn validate_cloud_kms_decrypt_request(
    request: &CloudKmsDecryptApiRequest,
) -> Result<(), CloudKmsApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_key_id(&request.path_key_id, &request.body.key_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_principal_actor(
        &request.principal,
        &request.body.tenant_id,
        &request.body.actor,
    )
}

/// Authorize and record a Cloud KMS encrypt receipt — fail-closed.
///
/// ## `verified` and `authz` — the fail-closed crypto gate (AUTH-005 / C5)
///
/// The crypto op is UNREACHABLE without a verified principal and a passing PDP
/// decision:
///
/// 1. `verified: &VerifiedKmsPrincipal` is an unforgeable type minted only by a
///    real [`authz::PrincipalVerifier`] (private fields, `pub(crate)`
///    constructor). The edge MUST verify the caller credential and pass the
///    resulting principal here; an unverified caller maps to
///    [`CloudKmsApiError::PrincipalUnauthenticated`] (401) at the edge.
/// 2. The verified identity is cross-checked against the caller-asserted
///    `request.principal` and `request.body` — a mismatch is
///    [`CloudKmsApiError::VerifiedPrincipalMismatch`] /
///    [`CloudKmsApiError::VerifiedTenantMismatch`] (403). The caller-supplied
///    fields NEVER override the verified identity.
/// 3. The server-side PDP ([`authz::KmsCryptoAuthorizer`]) decides over a
///    resource bound to the TARGET key-path tenant + the TARGET key id, with the
///    verified principal passed separately (trusted source, no IDOR / no
///    cross-tenant flattening). Deny or any PDP fault →
///    [`CloudKmsApiError::CryptoAuthorizationDenied`] (403). The caller-supplied
///    `allowed_surfaces` blob no longer exists and confers no authority.
///
/// All of this runs BEFORE the idempotency lookup and any directory mutation, so
/// an unauthorized request never touches state.
pub fn authorize_cloud_kms_encrypt_from_api(
    verified: &VerifiedKmsPrincipal,
    authz: &KmsCryptoAuthzProvider,
    directory: &mut CloudKmsDirectory,
    idempotency_ledger: &mut CloudKmsCryptoIdempotencyLedger,
    request: CloudKmsEncryptApiRequest,
) -> Result<CloudKmsCryptoSuccessResponse, CloudKmsApiError> {
    validate_cloud_kms_encrypt_request(&request)?;
    cross_check_verified_principal(
        verified,
        &request.principal,
        &request.body.tenant_id,
        &request.body.actor,
    )?;
    ensure_crypto_authorized(
        authz,
        verified,
        &CloudKmsCryptoAuthzInputs {
            path_key_id: &request.path_key_id,
            action: KmsCryptoAction::Encrypt,
            data_class: &request.body.data_class,
            purpose: &request.body.purpose,
            request_id: &request.boundary.request_id,
        },
    )?;
    validate_key_placement_boundary(directory, &request.path_key_id, &request.boundary)?;
    let key = idempotency_key_for(verified, CLOUD_KMS_ENCRYPT_SURFACE, &request.boundary);
    let fingerprint = encrypt_fingerprint_for(verified, &request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudKmsApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let api_version = request.boundary.oyatie_version.clone();
    let result = encrypt_input(&request.boundary, request.body)
        .and_then(|input| {
            directory
                .authorize_encrypt(input)
                .map_err(CloudKmsApiError::Kms)
        })
        .map(|receipt| {
            CloudKmsCryptoSuccessResponse::ok(receipt_record(receipt), request_id, api_version)
        });
    idempotency_ledger.entries.insert(
        key,
        CloudKmsCryptoIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

/// Authorize and record a Cloud KMS decrypt receipt — fail-closed.
///
/// Identical fail-closed gate to [`authorize_cloud_kms_encrypt_from_api`] but for
/// the plaintext-revealing `cloud.kms.decrypt` action. Cross-tenant decrypt
/// (verified tenant A targeting tenant B's key) is denied: the PDP resource is
/// bound to the TARGET key-path tenant + the TARGET key id, with the verified
/// principal passed separately, and the kernel additionally enforces
/// `key.tenant == request.tenant` (`ResourceTenantMismatch`). Neither
/// the caller-asserted tenant nor a forged `allowed_surfaces` blob can authorize
/// the op.
pub fn authorize_cloud_kms_decrypt_from_api(
    verified: &VerifiedKmsPrincipal,
    authz: &KmsCryptoAuthzProvider,
    directory: &mut CloudKmsDirectory,
    idempotency_ledger: &mut CloudKmsCryptoIdempotencyLedger,
    request: CloudKmsDecryptApiRequest,
) -> Result<CloudKmsCryptoSuccessResponse, CloudKmsApiError> {
    validate_cloud_kms_decrypt_request(&request)?;
    cross_check_verified_principal(
        verified,
        &request.principal,
        &request.body.tenant_id,
        &request.body.actor,
    )?;
    ensure_crypto_authorized(
        authz,
        verified,
        &CloudKmsCryptoAuthzInputs {
            path_key_id: &request.path_key_id,
            action: KmsCryptoAction::Decrypt,
            data_class: &request.body.data_class,
            purpose: &request.body.purpose,
            request_id: &request.boundary.request_id,
        },
    )?;
    validate_key_placement_boundary(directory, &request.path_key_id, &request.boundary)?;
    let key = idempotency_key_for(verified, CLOUD_KMS_DECRYPT_SURFACE, &request.boundary);
    let fingerprint = decrypt_fingerprint_for(verified, &request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudKmsApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let api_version = request.boundary.oyatie_version.clone();
    let result = decrypt_input(&request.boundary, request.body)
        .and_then(|input| {
            directory
                .authorize_decrypt(input)
                .map_err(CloudKmsApiError::Kms)
        })
        .map(|receipt| {
            CloudKmsCryptoSuccessResponse::ok(receipt_record(receipt), request_id, api_version)
        });
    idempotency_ledger.entries.insert(
        key,
        CloudKmsCryptoIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(boundary: &CloudKmsApiBoundaryContext) -> Result<(), CloudKmsApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyTenantHeader);
    }
    validate_public_api_version(&boundary.oyatie_version)?;
    let _placement = boundary_placement(boundary)?;
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudKmsBoundaryPlacement {
    region: RegionCode, // data_class: PUBLIC
    cell_id: CellId,    // data_class: PUBLIC
}

fn boundary_placement(
    boundary: &CloudKmsApiBoundaryContext,
) -> Result<CloudKmsBoundaryPlacement, CloudKmsApiError> {
    if boundary.region.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyRegionHeader);
    }
    let region = RegionCode::new(boundary.region.clone()).map_err(|_| {
        CloudKmsApiError::InvalidRegionHeader {
            region: boundary.region.clone(),
        }
    })?;
    if boundary.cell_id.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyCellHeader);
    }
    let cell_id =
        CellId::new(boundary.cell_id.clone()).map_err(|_| CloudKmsApiError::InvalidCellHeader {
            cell_id: boundary.cell_id.clone(),
        })?;
    let expected_cell_prefix = format!("cell-{}-", region.value);
    if !cell_id.value.starts_with(&expected_cell_prefix) {
        return Err(CloudKmsApiError::BoundaryCellRegionMismatch {
            region: region.value,
            cell_id: cell_id.value,
        });
    }
    Ok(CloudKmsBoundaryPlacement { region, cell_id })
}

fn validate_key_placement_boundary(
    directory: &CloudKmsDirectory,
    path_key_id: &str,
    boundary: &CloudKmsApiBoundaryContext,
) -> Result<(), CloudKmsApiError> {
    let placement = boundary_placement(boundary)?;
    let Some(key) = directory
        .keys()
        .find(|key| key.key_id.value.value == path_key_id)
    else {
        return Ok(());
    };
    if key.region.value != placement.region {
        return Err(CloudKmsApiError::Kms(CloudKmsError::ResourceRegionMismatch));
    }
    if key.cell_id.value != placement.cell_id {
        return Err(CloudKmsApiError::Kms(CloudKmsError::CellPlacementMismatch));
    }
    Ok(())
}

fn validate_public_api_version(oyatie_version: &str) -> Result<(), CloudKmsApiError> {
    if oyatie_version.trim().is_empty() {
        return Err(CloudKmsApiError::MissingPublicApiVersion);
    }
    if !CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS.contains(&oyatie_version) {
        return Err(CloudKmsApiError::UnsupportedPublicApiVersion {
            oyatie_version: oyatie_version.to_string(),
        });
    }
    Ok(())
}

fn validate_path_key_id(path_key_id: &str, body_key_id: &str) -> Result<(), CloudKmsApiError> {
    if path_key_id.trim().is_empty() {
        return Err(CloudKmsApiError::EmptyPathKeyId);
    }
    key_tenant_from_path(path_key_id)?;
    if path_key_id != body_key_id {
        return Err(CloudKmsApiError::KeyIdMismatch {
            path_key_id: path_key_id.to_string(),
            body_key_id: body_key_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudKmsApiBoundaryContext,
    principal: &CloudKmsApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudKmsApiError> {
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudKmsApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_principal_actor(
    principal: &CloudKmsApiPrincipal,
    body_tenant_id: &str,
    actor: &str,
) -> Result<(), CloudKmsApiError> {
    if principal.principal_id != actor {
        return Err(CloudKmsApiError::PrincipalMismatch {
            principal_tenant_id: principal.tenant_id.clone(),
            principal_id: principal.principal_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
            actor: actor.to_string(),
        });
    }
    Ok(())
}

/// Cross-check the VERIFIED principal against the caller-asserted identity.
///
/// The verified principal (minted from an unforgeable credential) is
/// authoritative. The caller-asserted `request.principal` and the `body` actor /
/// tenant are forgeable inputs; if any of them disagrees with the verified
/// identity the request is rejected (403) — the caller cannot substitute a
/// different identity than the one the credential proves. This binds the receipt
/// actor (which the kernel sets from `body.actor`) to the verified principal.
fn cross_check_verified_principal(
    verified: &VerifiedKmsPrincipal,
    principal: &CloudKmsApiPrincipal,
    body_tenant_id: &str,
    body_actor: &str,
) -> Result<(), CloudKmsApiError> {
    if verified.tenant_id() != principal.tenant_id || verified.tenant_id() != body_tenant_id {
        return Err(CloudKmsApiError::VerifiedTenantMismatch {
            verified_tenant_id: verified.tenant_id().to_string(),
            claimed_tenant_id: principal.tenant_id.clone(),
        });
    }
    if verified.principal_id() != principal.principal_id || verified.principal_id() != body_actor {
        return Err(CloudKmsApiError::VerifiedPrincipalMismatch {
            verified_principal_id: verified.principal_id().to_string(),
            claimed_principal_id: principal.principal_id.clone(),
        });
    }
    Ok(())
}

/// Trusted inputs for the server-side PDP decision. Caller identity comes from
/// the verified principal; resource identity comes from the trusted target key
/// path binding (format `kms/{region}/{tenant}/{name}`).
struct CloudKmsCryptoAuthzInputs<'a> {
    path_key_id: &'a str,
    action: KmsCryptoAction,
    data_class: &'a str,
    purpose: &'a str,
    request_id: &'a str,
}

/// Extract the tenant segment from a key path of the form `kms/{region}/{tenant}/{name}`.
///
/// The path is the TRUSTED authority for the target key's tenant — it comes from
/// the request URL path binding, not from the caller-supplied body.  Using the
/// caller's verified tenant instead (see pre-fix history) creates a cross-tenant
/// key-id IDOR: the PDP sees "may `ten_alpha` access `ten_alpha`'s key?" even when
/// the actual key belongs to `ten_beta`.
fn key_tenant_from_path(path_key_id: &str) -> Result<&str, CloudKmsApiError> {
    // Expected format: kms/{region}/{tenant}/{name}  (exactly 4 slash-delimited segments)
    let malformed = || CloudKmsApiError::MalformedPathKeyId {
        path_key_id: path_key_id.to_string(),
    };
    let mut parts = path_key_id.split('/');
    let (Some("kms"), Some(region), Some(tenant), Some(name), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(malformed());
    };
    if region.trim().is_empty() || tenant.trim().is_empty() || name.trim().is_empty() {
        return Err(malformed());
    }
    Ok(tenant)
}

/// Run the server-side PDP decision (fail-closed). The PDP resource is bound to
/// the TARGET KEY's tenant (parsed from the trusted path binding `kms/{region}/{tenant}/{name}`)
/// + the TARGET key id. It is NOT bound to the verified caller's tenant — doing so
/// would create a cross-tenant IDOR: a `ten_alpha` principal targeting `ten_beta/key`
/// would produce `{resource.tenant: ten_alpha}` which a same-tenant policy ALLOWS.
/// By binding the target key's tenant, the PDP correctly sees "may `ten_alpha`
/// access `ten_beta`'s key?" and a same-tenant policy DENIES at the authz layer.
///
/// A deny OR any PDP fault maps to [`CloudKmsApiError::CryptoAuthorizationDenied`]
/// (403, NOT 500).
fn ensure_crypto_authorized(
    authz: &KmsCryptoAuthzProvider,
    verified: &VerifiedKmsPrincipal,
    inputs: &CloudKmsCryptoAuthzInputs<'_>,
) -> Result<(), CloudKmsApiError> {
    let target_tenant = key_tenant_from_path(inputs.path_key_id)?;
    let resource = KmsCryptoResource {
        tenant_id: target_tenant.to_string(),
        key_id: inputs.path_key_id.to_string(),
        action: inputs.action,
        data_class: inputs.data_class.to_string(),
        purpose: inputs.purpose.to_string(),
        request_id: inputs.request_id.to_string(),
    };
    match authz.ensure_authorized(verified, &resource) {
        Ok(()) => Ok(()),
        Err(KmsCryptoAuthorizationError::Denied | KmsCryptoAuthorizationError::Refused) => {
            Err(CloudKmsApiError::CryptoAuthorizationDenied {
                surface: inputs.action.surface().to_string(),
            })
        }
    }
}

fn encrypt_input(
    boundary: &CloudKmsApiBoundaryContext,
    body: CloudKmsEncryptRequest,
) -> Result<KmsEncryptRequest, CloudKmsApiError> {
    Ok(KmsEncryptRequest {
        event_id: body.event_id,
        key_id: body.key_id,
        tenant_id: body.tenant_id,
        region: boundary.region.clone(),
        cell_id: boundary.cell_id.clone(),
        plaintext_ref: body.plaintext_ref,
        ciphertext_ref: body.ciphertext_ref,
        data_class: parse_api_data_class(body.data_class)?,
        purpose: parse_api_purpose(body.purpose)?,
        actor: body.actor,
        aad_fingerprint: body.aad_fingerprint,
        requested_at_epoch_seconds: body.requested_at_epoch_seconds,
    })
}

fn decrypt_input(
    boundary: &CloudKmsApiBoundaryContext,
    body: CloudKmsDecryptRequest,
) -> Result<KmsDecryptRequest, CloudKmsApiError> {
    Ok(KmsDecryptRequest {
        event_id: body.event_id,
        key_id: body.key_id,
        tenant_id: body.tenant_id,
        region: boundary.region.clone(),
        cell_id: boundary.cell_id.clone(),
        ciphertext_ref: body.ciphertext_ref,
        data_class: parse_api_data_class(body.data_class)?,
        purpose: parse_api_purpose(body.purpose)?,
        actor: body.actor,
        requested_at_epoch_seconds: body.requested_at_epoch_seconds,
    })
}

fn parse_api_data_class(
    label: String,
) -> Result<data_boundary_kernel::DataClass, CloudKmsApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudKmsApiError::InvalidDataClassLabel { data_class: label })
}

fn parse_api_purpose(label: String) -> Result<KmsPurpose, CloudKmsApiError> {
    match label.as_str() {
        "cloud_object_storage" => Ok(KmsPurpose::CloudObjectStorage),
        "cloud_block_storage" => Ok(KmsPurpose::CloudBlockStorage),
        "cloud_file_storage" => Ok(KmsPurpose::CloudFileStorage),
        "cloud_archive_storage" => Ok(KmsPurpose::CloudArchiveStorage),
        "workspace_drive_object" => Ok(KmsPurpose::WorkspaceDriveObject),
        "workspace_recording" => Ok(KmsPurpose::WorkspaceRecording),
        "secret_provider" => Ok(KmsPurpose::SecretProvider),
        "cross_region_replication" => Ok(KmsPurpose::CrossRegionReplication),
        "database_backup" => Ok(KmsPurpose::DatabaseBackup),
        _ => Err(CloudKmsApiError::InvalidPurposeLabel { purpose: label }),
    }
}

fn idempotency_key_for(
    verified: &VerifiedKmsPrincipal,
    surface: &str,
    boundary: &CloudKmsApiBoundaryContext,
) -> CloudKmsIdempotencyLedgerKey {
    CloudKmsIdempotencyLedgerKey {
        tenant_id: verified.tenant_id().to_string(),
        principal_id: verified.principal_id().to_string(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn encrypt_fingerprint_for(
    verified: &VerifiedKmsPrincipal,
    request: &CloudKmsEncryptApiRequest,
) -> CloudKmsRequestFingerprint {
    CloudKmsRequestFingerprint {
        canonical: [
            format!("path.key_id={}", request.path_key_id),
            format!("header.Oyatie-Version={}", request.boundary.oyatie_version),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("header.region={}", request.boundary.region),
            format!("header.cell_id={}", request.boundary.cell_id),
            format!("verified.tenant_id={}", verified.tenant_id()),
            format!("verified.principal_id={}", verified.principal_id()),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "correlation.decision_id={}",
                request.correlation.decision_id
            ),
            format!("body.event_id={}", request.body.event_id),
            format!("body.key_id={}", request.body.key_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.plaintext_ref={}", request.body.plaintext_ref),
            format!("body.ciphertext_ref={}", request.body.ciphertext_ref),
            format!("body.data_class={}", request.body.data_class),
            format!("body.purpose={}", request.body.purpose),
            format!("body.actor={}", request.body.actor),
            format!("body.aad_fingerprint={}", request.body.aad_fingerprint),
            format!(
                "body.requested_at_epoch_seconds={}",
                request.body.requested_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn decrypt_fingerprint_for(
    verified: &VerifiedKmsPrincipal,
    request: &CloudKmsDecryptApiRequest,
) -> CloudKmsRequestFingerprint {
    CloudKmsRequestFingerprint {
        canonical: [
            format!("path.key_id={}", request.path_key_id),
            format!("header.Oyatie-Version={}", request.boundary.oyatie_version),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("header.region={}", request.boundary.region),
            format!("header.cell_id={}", request.boundary.cell_id),
            format!("verified.tenant_id={}", verified.tenant_id()),
            format!("verified.principal_id={}", verified.principal_id()),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "correlation.decision_id={}",
                request.correlation.decision_id
            ),
            format!("body.event_id={}", request.body.event_id),
            format!("body.key_id={}", request.body.key_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.ciphertext_ref={}", request.body.ciphertext_ref),
            format!("body.data_class={}", request.body.data_class),
            format!("body.purpose={}", request.body.purpose),
            format!("body.actor={}", request.body.actor),
            format!(
                "body.requested_at_epoch_seconds={}",
                request.body.requested_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn receipt_record(receipt: KmsUseReceipt) -> CloudKmsUseReceiptRecord {
    CloudKmsUseReceiptRecord {
        event_id: receipt.event_id.value.value,
        key_id: receipt.key_id.value.value,
        tenant_id: receipt.tenant_id.value,
        operation: kms_operation_label(receipt.operation.value).to_string(),
        material_ref: receipt.material_ref.value.map(|value| value.value),
        ciphertext_ref: receipt.ciphertext_ref.value.value,
        data_class: receipt.data_class.value.label().to_string(),
        purpose: kms_purpose_label(receipt.purpose.value).to_string(),
        actor: receipt.actor.value.value,
        key_version: receipt.key_version.value,
        hsm_partition_ref: receipt.hsm_partition_ref.value.value,
        occurred_at_epoch_seconds: receipt.occurred_at_epoch_seconds.value,
        schema_version: receipt.schema_version.value,
    }
}

fn kms_operation_label(operation: KmsOperation) -> &'static str {
    match operation {
        KmsOperation::Encrypt => "encrypt",
        KmsOperation::Decrypt => "decrypt",
    }
}

fn kms_purpose_label(purpose: KmsPurpose) -> &'static str {
    match purpose {
        KmsPurpose::CloudObjectStorage => "cloud_object_storage",
        KmsPurpose::CloudBlockStorage => "cloud_block_storage",
        KmsPurpose::CloudFileStorage => "cloud_file_storage",
        KmsPurpose::CloudArchiveStorage => "cloud_archive_storage",
        KmsPurpose::WorkspaceDriveObject => "workspace_drive_object",
        KmsPurpose::WorkspaceRecording => "workspace_recording",
        KmsPurpose::SecretProvider => "secret_provider",
        KmsPurpose::CrossRegionReplication => "cross_region_replication",
        KmsPurpose::DatabaseBackup => "database_backup",
    }
}

fn cloud_kms_status_kind(error: &CloudKmsError) -> CloudKmsApiStatusKind {
    match error {
        CloudKmsError::DuplicateKey | CloudKmsError::DuplicateUseEvent => {
            CloudKmsApiStatusKind::Conflict
        }
        CloudKmsError::UnknownKey => CloudKmsApiStatusKind::NotFound,
        CloudKmsError::ResourceTenantMismatch
        | CloudKmsError::ResourceRegionMismatch
        | CloudKmsError::CellPlacementMismatch
        | CloudKmsError::InvalidDataClass
        | CloudKmsError::InvalidKeyState
        | CloudKmsError::InvalidKeyUsage => CloudKmsApiStatusKind::Forbidden,
        CloudKmsError::InvalidTenantId
        | CloudKmsError::InvalidResourceId
        | CloudKmsError::ResourceKindMismatch
        | CloudKmsError::InvalidKeyId
        | CloudKmsError::KeyIdOriginMismatch
        | CloudKmsError::KeyIdTenantMismatch
        | CloudKmsError::KeyIdRegionMismatch
        | CloudKmsError::InvalidCellId
        | CloudKmsError::CellRegionMismatch
        | CloudKmsError::InvalidHsmPartitionRef
        | CloudKmsError::HsmPartitionMismatch
        | CloudKmsError::HsmValidationDenied
        | CloudKmsError::ResidencyRegionMismatch
        | CloudKmsError::InvalidRotationPeriod
        | CloudKmsError::InvalidEventId
        | CloudKmsError::InvalidMaterialRef
        | CloudKmsError::InvalidCiphertextRef
        | CloudKmsError::InvalidActorRef
        | CloudKmsError::InvalidAadFingerprint
        | CloudKmsError::InvalidTimeOrder
        | CloudKmsError::DestructionSlaExceeded
        | CloudKmsError::InvalidDestructionProofRef
        | CloudKmsError::ProviderMismatch
        | CloudKmsError::InvalidEvidenceRef
        | CloudKmsError::InvalidEvidenceSchemaVersion => CloudKmsApiStatusKind::BadRequest,
    }
}

fn cloud_kms_message(error: &CloudKmsError) -> &'static str {
    match cloud_kms_status_kind(error) {
        CloudKmsApiStatusKind::Conflict => "Cloud KMS resource already exists",
        CloudKmsApiStatusKind::Unauthorized | CloudKmsApiStatusKind::Forbidden => {
            "Cloud KMS policy denied the request"
        }
        CloudKmsApiStatusKind::NotFound => "Cloud KMS key was not found",
        CloudKmsApiStatusKind::BadRequest => "Cloud KMS rejected the request shape",
        CloudKmsApiStatusKind::UnprocessableEntity => "Cloud KMS rejected request idempotency",
    }
}

fn cloud_kms_issue(error: &CloudKmsError) -> &'static str {
    match error {
        CloudKmsError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudKmsError::InvalidResourceId => "resource_id must be a canonical cloud resource id",
        CloudKmsError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudKmsError::ResourceRegionMismatch => "resource region must match key region",
        CloudKmsError::ResourceKindMismatch => "resource kind must be kms-key",
        CloudKmsError::InvalidKeyId => "key_id must be origin/region/tenant/name",
        CloudKmsError::KeyIdOriginMismatch => "key_id origin must match key origin",
        CloudKmsError::KeyIdTenantMismatch => "key_id tenant must match request tenant",
        CloudKmsError::KeyIdRegionMismatch => "key_id region must match request region",
        CloudKmsError::InvalidCellId => "cell_id must be a canonical cell identifier",
        CloudKmsError::CellRegionMismatch => "cell_id must belong to key region",
        CloudKmsError::CellPlacementMismatch => "request cell must match key cell",
        CloudKmsError::InvalidHsmPartitionRef => "hsm_partition_ref must be canonical",
        CloudKmsError::HsmPartitionMismatch => "hsm_partition_ref must bind region and cell",
        CloudKmsError::HsmValidationDenied => "KMS keys require an accepted HSM validation profile",
        CloudKmsError::ResidencyRegionMismatch => "residency class must allow key home region",
        CloudKmsError::InvalidDataClass => "request data_class must match key policy class",
        CloudKmsError::InvalidKeyState => "key state must allow cryptographic use",
        CloudKmsError::InvalidKeyUsage => "key usage must be encrypt/decrypt",
        CloudKmsError::InvalidRotationPeriod => "rotation period must be between 30 and 730 days",
        CloudKmsError::InvalidEventId => "event_id must be a kmsuse_ identifier",
        CloudKmsError::InvalidMaterialRef => "plaintext_ref must be a matref/ reference",
        CloudKmsError::InvalidCiphertextRef => "ciphertext_ref must be a ct/ reference",
        CloudKmsError::InvalidActorRef => "actor must be a usr_ or sp_ principal",
        CloudKmsError::InvalidAadFingerprint => {
            "aad_fingerprint must be a 64-character hexadecimal digest"
        }
        CloudKmsError::InvalidTimeOrder => "request timestamps must be monotonic",
        CloudKmsError::DestructionSlaExceeded => {
            "key destruction proof must complete within 24 hours"
        }
        CloudKmsError::InvalidDestructionProofRef => {
            "destruction proof must be a kproof_ reference"
        }
        CloudKmsError::ProviderMismatch => "provider receipt must match the requested provider",
        CloudKmsError::InvalidEvidenceRef => {
            "evidence_ref must be immutable metadata evidence, not raw key material"
        }
        CloudKmsError::InvalidEvidenceSchemaVersion => {
            "evidence receipt schema_version must match Cloud KMS schema"
        }
        CloudKmsError::DuplicateKey => "key identifier is already present",
        CloudKmsError::UnknownKey => "key must exist before cryptographic use",
        CloudKmsError::DuplicateUseEvent => "KMS use event identifier is already present",
    }
}

fn detail(field: &str, issue: &str) -> CloudKmsApiErrorDetail {
    CloudKmsApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

/// Resolve an optional raw `Oyatie-Version` header value to the accepted API version.
///
/// | Header value | Result |
/// |---|---|
/// | `None` (absent) | `Ok(CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION)` |
/// | empty or whitespace-only | `Err(MissingPublicApiVersion)` → 400 |
/// | member of `CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS` | `Ok(version)` as `&'static str` |
/// | any other value | `Err(UnsupportedPublicApiVersion { oyatie_version })` → 400 |
///
/// The `Ok` arm returns a pointer directly into `CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS`
/// so no heap allocation occurs on the success path.
pub fn negotiate_cloud_kms_api_version(
    header: Option<&str>,
) -> Result<&'static str, CloudKmsApiError> {
    let Some(raw) = header else {
        return Ok(CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION);
    };
    if raw.trim().is_empty() {
        return Err(CloudKmsApiError::MissingPublicApiVersion);
    }
    CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS
        .iter()
        .copied()
        .find(|&v| v == raw)
        .ok_or_else(|| CloudKmsApiError::UnsupportedPublicApiVersion {
            oyatie_version: raw.to_string(),
        })
}

#[cfg(test)]
mod version_negotiation_tests {
    use super::{
        CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION, CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS,
        CloudKmsApiError, negotiate_cloud_kms_api_version,
    };

    #[test]
    fn absent_header_returns_default_version() {
        let result = negotiate_cloud_kms_api_version(None);
        assert_eq!(result, Ok(CLOUD_KMS_DEFAULT_PUBLIC_API_VERSION));
    }

    #[test]
    fn each_supported_version_is_echoed() {
        for &version in CLOUD_KMS_SUPPORTED_PUBLIC_API_VERSIONS {
            let result = negotiate_cloud_kms_api_version(Some(version));
            assert_eq!(result, Ok(version), "version {version} should be accepted");
        }
    }

    #[test]
    fn unknown_version_string_returns_typed_error_with_status_400() {
        let result = negotiate_cloud_kms_api_version(Some("1999-01-01"));
        let err = result.expect_err("unknown version must be rejected");
        assert_eq!(
            err,
            CloudKmsApiError::UnsupportedPublicApiVersion {
                oyatie_version: "1999-01-01".to_string(),
            }
        );
        assert_eq!(err.crypto_status_code(), 400);
    }

    #[test]
    fn empty_or_whitespace_header_returns_missing_error_with_status_400() {
        for raw in ["", "  ", "\t"] {
            let result = negotiate_cloud_kms_api_version(Some(raw));
            let err = result.expect_err("empty/whitespace version must be rejected");
            assert_eq!(
                err,
                CloudKmsApiError::MissingPublicApiVersion,
                "input {raw:?} should yield MissingPublicApiVersion"
            );
            assert_eq!(err.crypto_status_code(), 400);
        }
    }
}
