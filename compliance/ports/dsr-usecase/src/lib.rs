//! Platform DSR cascade app boundary.
//!
//! This crate owns REST-boundary normalization, authorization proof checks,
//! request fingerprint idempotency, cross-axis store/proof projection, and
//! DSR completion record assembly around the Platform DSR kernel. Queue fan-out,
//! adapter execution, audit-chain append, and Trust Portal publication remain
//! adapter/application concerns.

use std::collections::BTreeMap;

use compliance_dsr::{
    DsrAckReason, DsrAckStatus, DsrAction, DsrAxis, DsrCascadeAck, DsrCascadeAckCreate,
    DsrCompletionRecord, DsrCompletionRecordCreate, DsrDispatch, DsrDispatchCreate, DsrProofMethod,
    DsrRequest, DsrRequestCreate, DsrSlaStatus, DsrSlaTier, DsrStoreKind, DsrStoreRef,
    DsrStoreRefCreate, ErasureProof, ErasureProofCreate, PlatformDsrError,
    platform_dsr_data_class_from_legacy,
};
use oya_data_boundary_kernel::parse_data_class_label;

/// Fail-closed authorization seam for the DSR erasure cascade (AUTH-005 /
/// Wave-2b; ADR-0589). The cascade is UNREACHABLE without a verified principal
/// and a passing server-side PDP decision — the retired caller-supplied
/// `allowed_surfaces` blob no longer confers any authority.
pub mod authz;

pub use authz::{
    DsrCascadeAuthorizationError, DsrCascadeAuthorizer, DsrCascadeAuthzProvider,
    DsrCascadeResource, VerifiedDsrPrincipal,
};

pub const PLATFORM_DSR_CASCADE_EXECUTE_SURFACE: &str = "dsr.cascade.execute";
pub const PLATFORM_DSR_OPENAPI_CONTRACT: &str = "contracts/openapi/platform/platform-dsr-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformDsrCascadeExecuteApiStatus {
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl PlatformDsrCascadeExecuteApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformDsrApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    DsrIdInvalid,
    DsrIdMismatch,
    TenantMismatch,
    PrincipalUnauthenticated,
    VerifiedTenantMismatch,
    VerifiedPrincipalMismatch,
    CascadeAuthorizationDenied,
    IdempotencyKeyReused,
    CascadeAlreadyCompleted,
    ActionInvalid,
    SlaTierInvalid,
    AxisInvalid,
    StoreKindInvalid,
    ProofMethodInvalid,
    AckStatusInvalid,
    AckReasonInvalid,
    DataClassInvalid,
    ProofFieldMissing,
    KernelInvalidRequest,
}

impl PlatformDsrApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "PLATFORM_DSR_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "PLATFORM_DSR_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "PLATFORM_DSR_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "PLATFORM_DSR_PRINCIPAL_ID_EMPTY",
            Self::DsrIdInvalid => "PLATFORM_DSR_ID_INVALID",
            Self::DsrIdMismatch => "PLATFORM_DSR_ID_MISMATCH",
            Self::TenantMismatch => "PLATFORM_DSR_TENANT_MISMATCH",
            Self::PrincipalUnauthenticated => "PLATFORM_DSR_PRINCIPAL_UNAUTHENTICATED",
            Self::VerifiedTenantMismatch => "PLATFORM_DSR_VERIFIED_TENANT_MISMATCH",
            Self::VerifiedPrincipalMismatch => "PLATFORM_DSR_VERIFIED_PRINCIPAL_MISMATCH",
            Self::CascadeAuthorizationDenied => "PLATFORM_DSR_CASCADE_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "PLATFORM_DSR_IDEMPOTENCY_KEY_REUSED",
            Self::CascadeAlreadyCompleted => "PLATFORM_DSR_CASCADE_ALREADY_COMPLETED",
            Self::ActionInvalid => "PLATFORM_DSR_ACTION_INVALID",
            Self::SlaTierInvalid => "PLATFORM_DSR_SLA_TIER_INVALID",
            Self::AxisInvalid => "PLATFORM_DSR_AXIS_INVALID",
            Self::StoreKindInvalid => "PLATFORM_DSR_STORE_KIND_INVALID",
            Self::ProofMethodInvalid => "PLATFORM_DSR_PROOF_METHOD_INVALID",
            Self::AckStatusInvalid => "PLATFORM_DSR_ACK_STATUS_INVALID",
            Self::AckReasonInvalid => "PLATFORM_DSR_ACK_REASON_INVALID",
            Self::DataClassInvalid => "PLATFORM_DSR_DATA_CLASS_INVALID",
            Self::ProofFieldMissing => "PLATFORM_DSR_PROOF_FIELD_MISSING",
            Self::KernelInvalidRequest => "PLATFORM_DSR_KERNEL_INVALID_REQUEST",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrCascadeBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

/// The caller-asserted principal identity carried alongside the request.
///
/// ## NOT an authority source
///
/// These fields are caller-supplied and therefore forgeable. They are used ONLY
/// as a cross-check against the [`authz::VerifiedDsrPrincipal`] the
/// [`authz::DsrCascadePrincipalVerifier`] bound from an unforgeable credential.
/// A request whose asserted principal/tenant disagrees with the verified
/// identity is rejected (`VerifiedPrincipalMismatch` / `VerifiedTenantMismatch`,
/// 403). The verified principal — never these fields — is the authority for the
/// audit record, the idempotency key, and the PDP decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY (cross-check only)
    pub principal_id: String, // data_class: PII_IDENTIFYING (cross-check only)
}

/// A non-authoritative correlation hint carried alongside the request.
///
/// ## NOT an authorization grant (AUTH-005 / Wave-2b remediation, ADR-0589)
///
/// Before ADR-0589 this struct carried `{tenant_id, principal_id, decision_id,
/// allowed_surfaces}` and [`validate_authorization`] "authorized" the GDPR
/// erasure cascade by checking the caller-supplied `allowed_surfaces` contained
/// `dsr.cascade.execute` — a forgeable PDP bypass on a destructive,
/// compliance-critical surface. The authority fields (`tenant_id`,
/// `principal_id`, `allowed_surfaces`) are REMOVED. The only remaining field,
/// [`Self::decision_id`], is a correlation id echoed into telemetry; it confers
/// NO authorization. The authorization decision is now made server-side by the
/// [`authz::DsrCascadeAuthorizer`] PDP port over the VERIFIED principal and the
/// trusted resource binding.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlatformDsrApiAuthorization {
    /// Caller-supplied correlation id (e.g. an upstream PDP decision id). This is
    /// telemetry/audit correlation ONLY — it is never treated as a grant.
    pub decision_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrCascadeTargetRequest {
    pub dispatch_id: String,              // data_class: INTERNAL_ONLY
    pub dispatch_idempotency_key: String, // data_class: INTERNAL_ONLY
    pub ack_id: String,                   // data_class: INTERNAL_ONLY
    pub ack_status: String,               // data_class: INTERNAL_ONLY
    pub ack_reason: Option<String>,       // data_class: INTERNAL_ONLY
    pub axis: String,                     // data_class: INTERNAL_ONLY
    pub store_kind: String,               // data_class: INTERNAL_ONLY
    pub store_id: String,                 // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: INTERNAL_ONLY
    pub cell_id: String,                  // data_class: INTERNAL_ONLY
    pub record_ref: String,               // data_class: INTERNAL_ONLY
    pub data_class: String,               // data_class: INTERNAL_ONLY
    pub proof_id: Option<String>,         // data_class: INTERNAL_ONLY
    pub proof_method: Option<String>,     // data_class: INTERNAL_ONLY
    pub evidence_hash: Option<String>,    // data_class: INTERNAL_ONLY
    pub witness_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub signer_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub signature_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub rekor_log_index: Option<u64>,     // data_class: INTERNAL_ONLY
    pub processed_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrCascadeExecuteRequest {
    pub dsr_id: String,                                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub region: String,                                // data_class: INTERNAL_ONLY
    pub subject_ref: String,                           // data_class: PII_IDENTIFYING
    pub action: String,                                // data_class: INTERNAL_ONLY
    pub sla_tier: String,                              // data_class: INTERNAL_ONLY
    pub data_classes: Vec<String>,                     // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: u64,                   // data_class: INTERNAL_ONLY
    pub completion_id: String,                         // data_class: INTERNAL_ONLY
    pub aggregate_proof_hash: String,                  // data_class: INTERNAL_ONLY
    pub signer_ref: String,                            // data_class: INTERNAL_ONLY
    pub signature_ref: String,                         // data_class: INTERNAL_ONLY
    pub rekor_log_index: u64,                          // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64,               // data_class: INTERNAL_ONLY
    pub targets: Vec<PlatformDsrCascadeTargetRequest>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrCascadeExecuteApiRequest {
    pub path_dsr_id: String,                         // data_class: INTERNAL_ONLY
    pub boundary: PlatformDsrCascadeBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: PlatformDsrApiPrincipal,          // data_class: PII_IDENTIFYING
    pub authorization: PlatformDsrApiAuthorization,  // data_class: INTERNAL_ONLY
    pub body: PlatformDsrCascadeExecuteRequest,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlatformDsrCascadeDirectory {
    completions: BTreeMap<PlatformDsrCompletionKey, PlatformDsrCompletionRecord>, // data_class: INTERNAL_ONLY
}

impl PlatformDsrCascadeDirectory {
    pub fn len(&self) -> usize {
        self.completions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.completions.is_empty()
    }

    pub fn completions(&self) -> impl Iterator<Item = &PlatformDsrCompletionRecord> {
        self.completions.values()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct PlatformDsrCompletionKey {
    tenant_id: String, // data_class: INTERNAL_ONLY
    dsr_id: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PlatformDsrCascadeExecuteIdempotencyLedger {
    entries: BTreeMap<PlatformDsrIdempotencyLedgerKey, PlatformDsrCascadeLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl PlatformDsrCascadeExecuteIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct PlatformDsrIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: PII_IDENTIFYING
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlatformDsrCascadeLedgerEntry {
    fingerprint: PlatformDsrRequestFingerprint, // data_class: INTERNAL_ONLY
    result: PlatformDsrCascadeExecuteSuccessResponse, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlatformDsrRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrCascadeExecuteSuccessResponse {
    pub data: PlatformDsrCompletionRecord, // data_class: INTERNAL_ONLY
    pub metadata: PlatformDsrCascadeMetadata, // data_class: INTERNAL_ONLY
}

impl PlatformDsrCascadeExecuteSuccessResponse {
    pub fn accepted(data: PlatformDsrCompletionRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: PlatformDsrCascadeMetadata {
                request_id: request_id.into(),
                surface: PLATFORM_DSR_CASCADE_EXECUTE_SURFACE.to_string(),
                openapi_contract: PLATFORM_DSR_OPENAPI_CONTRACT.to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrCascadeMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrCompletionRecord {
    pub dsr_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub subject_ref: String,             // data_class: PII_IDENTIFYING
    pub action: String,                  // data_class: INTERNAL_ONLY
    pub completion_id: String,           // data_class: INTERNAL_ONLY
    pub completion_status: String,       // data_class: INTERNAL_ONLY
    pub sla_status: String,              // data_class: INTERNAL_ONLY
    pub dispatch_ids: Vec<String>,       // data_class: INTERNAL_ONLY
    pub ack_ids: Vec<String>,            // data_class: INTERNAL_ONLY
    pub proof_ids: Vec<String>,          // data_class: INTERNAL_ONLY
    pub aggregate_proof_hash: String,    // data_class: INTERNAL_ONLY
    pub signer_ref: String,              // data_class: INTERNAL_ONLY
    pub signature_ref: String,           // data_class: INTERNAL_ONLY
    pub rekor_log_index: u64,            // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,             // data_class: PUBLIC
    pub store_count: u64,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrApiErrorResponse {
    pub error: PlatformDsrApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrApiErrorBody {
    pub code: String,                            // data_class: INTERNAL_ONLY
    pub message: String,                         // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,       // data_class: INTERNAL_ONLY
    pub request_id: String,                      // data_class: INTERNAL_ONLY
    pub details: Vec<PlatformDsrApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformDsrApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlatformDsrApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    InvalidDsrId {
        dsr_id: String,
    },
    DsrIdMismatch {
        path_dsr_id: String,
        body_dsr_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        body_tenant_id: Option<String>,
    },
    /// No verifiable principal credential was presented (401). The cascade is
    /// unreachable without a [`authz::VerifiedDsrPrincipal`].
    PrincipalUnauthenticated,
    /// The verified principal's tenant disagrees with the caller-asserted /
    /// body tenant (403). The verified identity — never caller input — is the
    /// authority.
    VerifiedTenantMismatch {
        verified_tenant_id: String,
        claimed_tenant_id: String,
    },
    /// The verified principal id disagrees with the caller-asserted principal
    /// (403). The verified identity — never caller input — is the authority.
    VerifiedPrincipalMismatch {
        verified_principal_id: String,
        claimed_principal_id: String,
    },
    /// The server-side PDP denied, or refused to decide (any PDP fault is
    /// fail-closed), the `dsr.cascade.execute` decision for the verified
    /// principal on the target tenant/dsr (403).
    CascadeAuthorizationDenied {
        surface: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    CascadeAlreadyCompleted {
        tenant_id: String,
        dsr_id: String,
    },
    InvalidActionLabel {
        action: String,
    },
    InvalidSlaTierLabel {
        sla_tier: String,
    },
    InvalidAxisLabel {
        axis: String,
    },
    InvalidStoreKindLabel {
        store_kind: String,
    },
    InvalidProofMethodLabel {
        proof_method: String,
    },
    InvalidAckStatusLabel {
        ack_status: String,
    },
    InvalidAckReasonLabel {
        ack_reason: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    MissingCompletedProofField {
        field: String,
    },
    Kernel(PlatformDsrError),
}

impl PlatformDsrApiError {
    pub fn status_code(&self) -> u16 {
        match self.status_kind() {
            PlatformDsrApiStatusKind::Unauthorized => 401,
            PlatformDsrApiStatusKind::BadRequest => 400,
            PlatformDsrApiStatusKind::Forbidden => 403,
            PlatformDsrApiStatusKind::Conflict => 409,
            PlatformDsrApiStatusKind::UnprocessableEntity => 422,
        }
    }

    pub fn code(&self) -> PlatformDsrApiErrorCode {
        match self {
            Self::EmptyRequestId => PlatformDsrApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => PlatformDsrApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => PlatformDsrApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => PlatformDsrApiErrorCode::PrincipalIdEmpty,
            Self::InvalidDsrId { .. } => PlatformDsrApiErrorCode::DsrIdInvalid,
            Self::DsrIdMismatch { .. } => PlatformDsrApiErrorCode::DsrIdMismatch,
            Self::TenantMismatch { .. } => PlatformDsrApiErrorCode::TenantMismatch,
            Self::PrincipalUnauthenticated => PlatformDsrApiErrorCode::PrincipalUnauthenticated,
            Self::VerifiedTenantMismatch { .. } => PlatformDsrApiErrorCode::VerifiedTenantMismatch,
            Self::VerifiedPrincipalMismatch { .. } => {
                PlatformDsrApiErrorCode::VerifiedPrincipalMismatch
            }
            Self::CascadeAuthorizationDenied { .. } => {
                PlatformDsrApiErrorCode::CascadeAuthorizationDenied
            }
            Self::IdempotencyKeyReused { .. } => PlatformDsrApiErrorCode::IdempotencyKeyReused,
            Self::CascadeAlreadyCompleted { .. } => {
                PlatformDsrApiErrorCode::CascadeAlreadyCompleted
            }
            Self::InvalidActionLabel { .. } => PlatformDsrApiErrorCode::ActionInvalid,
            Self::InvalidSlaTierLabel { .. } => PlatformDsrApiErrorCode::SlaTierInvalid,
            Self::InvalidAxisLabel { .. } => PlatformDsrApiErrorCode::AxisInvalid,
            Self::InvalidStoreKindLabel { .. } => PlatformDsrApiErrorCode::StoreKindInvalid,
            Self::InvalidProofMethodLabel { .. } => PlatformDsrApiErrorCode::ProofMethodInvalid,
            Self::InvalidAckStatusLabel { .. } => PlatformDsrApiErrorCode::AckStatusInvalid,
            Self::InvalidAckReasonLabel { .. } => PlatformDsrApiErrorCode::AckReasonInvalid,
            Self::InvalidDataClassLabel { .. } => PlatformDsrApiErrorCode::DataClassInvalid,
            Self::MissingCompletedProofField { .. } => PlatformDsrApiErrorCode::ProofFieldMissing,
            Self::Kernel(_) => PlatformDsrApiErrorCode::KernelInvalidRequest,
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> PlatformDsrApiErrorResponse {
        PlatformDsrApiErrorResponse {
            error: PlatformDsrApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> PlatformDsrApiStatusKind {
        match self {
            Self::PrincipalUnauthenticated => PlatformDsrApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::VerifiedTenantMismatch { .. }
            | Self::VerifiedPrincipalMismatch { .. }
            | Self::CascadeAuthorizationDenied { .. } => PlatformDsrApiStatusKind::Forbidden,
            Self::CascadeAlreadyCompleted { .. } => PlatformDsrApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => PlatformDsrApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::InvalidDsrId { .. }
            | Self::DsrIdMismatch { .. }
            | Self::InvalidActionLabel { .. }
            | Self::InvalidSlaTierLabel { .. }
            | Self::InvalidAxisLabel { .. }
            | Self::InvalidStoreKindLabel { .. }
            | Self::InvalidProofMethodLabel { .. }
            | Self::InvalidAckStatusLabel { .. }
            | Self::InvalidAckReasonLabel { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::MissingCompletedProofField { .. }
            | Self::Kernel(_) => PlatformDsrApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::InvalidDsrId { .. } => "DSR id is required",
            Self::DsrIdMismatch { .. } => "Path and body DSR ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match the caller-asserted principal and body tenant"
            }
            Self::PrincipalUnauthenticated => {
                "A verified principal credential is required for the DSR erasure cascade"
            }
            Self::VerifiedTenantMismatch { .. } => {
                "Verified principal tenant must match the request tenant"
            }
            Self::VerifiedPrincipalMismatch { .. } => {
                "Verified principal must match the caller-asserted principal"
            }
            Self::CascadeAuthorizationDenied { .. } => {
                "The policy decision point denied dsr.cascade.execute for this principal and tenant"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::CascadeAlreadyCompleted { .. } => "DSR cascade already completed",
            Self::InvalidActionLabel { .. } => "DSR action label is unknown",
            Self::InvalidSlaTierLabel { .. } => "DSR SLA tier label is unknown",
            Self::InvalidAxisLabel { .. } => "DSR axis label is unknown",
            Self::InvalidStoreKindLabel { .. } => "DSR store kind label is unknown",
            Self::InvalidProofMethodLabel { .. } => "DSR proof method label is unknown",
            Self::InvalidAckStatusLabel { .. } => "DSR acknowledgement status label is unknown",
            Self::InvalidAckReasonLabel { .. } => "DSR acknowledgement reason label is unknown",
            Self::InvalidDataClassLabel { .. } => {
                "DSR data_classes must be known privacy data classes"
            }
            Self::MissingCompletedProofField { .. } => "Completed DSR target requires proof fields",
            Self::Kernel(error) => platform_dsr_error_message(error),
        }
    }

    fn details(&self) -> Vec<PlatformDsrApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::InvalidDsrId { .. } => vec![detail("path.dsr_id", "must be non-empty")],
            Self::DsrIdMismatch { .. } => {
                vec![detail("dsr_id", "path dsr_id and body dsr_id must match")]
            }
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, caller-asserted principal tenant, and body tenant_id must match",
            )],
            Self::PrincipalUnauthenticated => vec![detail(
                "header.Authorization",
                "a verified principal credential (bearer/mTLS) is required",
            )],
            Self::VerifiedTenantMismatch { .. } => vec![detail(
                "principal.tenant_id",
                "must match the verified principal tenant",
            )],
            Self::VerifiedPrincipalMismatch { .. } => vec![detail(
                "principal.principal_id",
                "must match the verified principal id",
            )],
            Self::CascadeAuthorizationDenied { .. } => vec![detail(
                "authorization",
                "the PDP must allow dsr.cascade.execute for the verified principal on the target tenant",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::CascadeAlreadyCompleted { .. } => vec![detail(
                "body.dsr_id",
                "DSR cascade completion already exists for tenant",
            )],
            Self::InvalidActionLabel { .. } => vec![detail(
                "body.action",
                "must be one of erase, correct, export, restrict, object_to_processing, automated_decision_opt_out",
            )],
            Self::InvalidSlaTierLabel { .. } => vec![detail(
                "body.sla_tier",
                "must be one of preview, stable, ga",
            )],
            Self::InvalidAxisLabel { .. } => vec![detail(
                "body.targets.axis",
                "must be one of saas, workspace, vertical, foundry, cloud, search, ads, analytics",
            )],
            Self::InvalidStoreKindLabel { .. } => vec![detail(
                "body.targets.store_kind",
                "must be a known DSR store kind",
            )],
            Self::InvalidProofMethodLabel { .. } => vec![detail(
                "body.targets.proof_method",
                "must be compatible with the DSR action",
            )],
            Self::InvalidAckStatusLabel { .. } => vec![detail(
                "body.targets.ack_status",
                "must be a known DSR acknowledgement status",
            )],
            Self::InvalidAckReasonLabel { .. } => vec![detail(
                "body.targets.ack_reason",
                "must be a known DSR acknowledgement reason",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_classes",
                "must be canonical privacy data-class labels",
            )],
            Self::MissingCompletedProofField { field } => {
                vec![detail(field, "is required when ack_status is completed")]
            }
            Self::Kernel(error) => vec![detail("platform_dsr", platform_dsr_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlatformDsrApiStatusKind {
    Unauthorized,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_platform_dsr_cascade_execute_request(
    request: &PlatformDsrCascadeExecuteApiRequest,
) -> Result<(), PlatformDsrApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_dsr_id(&request.path_dsr_id)?;
    if request.path_dsr_id != request.body.dsr_id {
        return Err(PlatformDsrApiError::DsrIdMismatch {
            path_dsr_id: request.path_dsr_id.clone(),
            body_dsr_id: request.body.dsr_id.clone(),
        });
    }
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        Some(&request.body.tenant_id),
    )?;
    Ok(())
}

/// Authorize and execute a DSR erasure cascade — fail-closed (AUTH-005 /
/// Wave-2b; ADR-0589).
///
/// ## `verified` and `authz` — the fail-closed cascade gate
///
/// The irreversible erasure cascade is UNREACHABLE without a verified principal
/// and a passing server-side PDP decision:
///
/// 1. `verified: &VerifiedDsrPrincipal` is an unforgeable type minted only by a
///    real [`authz::DsrCascadePrincipalVerifier`] (private fields, `pub(crate)`
///    constructor). The edge MUST verify the caller credential and pass the
///    resulting principal here; an unverified caller maps to
///    [`PlatformDsrApiError::PrincipalUnauthenticated`] (401) at the edge.
/// 2. The verified identity is cross-checked against the caller-asserted
///    `request.principal` and `request.body.tenant_id` — a mismatch is
///    [`PlatformDsrApiError::VerifiedPrincipalMismatch`] /
///    [`PlatformDsrApiError::VerifiedTenantMismatch`] (403). The caller-supplied
///    fields NEVER override the verified identity.
/// 3. The server-side PDP ([`authz::DsrCascadeAuthorizer`]) decides over a
///    resource bound to the VERIFIED tenant + the TARGET dsr id (trusted
///    sources, no cross-tenant flattening of a forged `allowed_surfaces` blob).
///    Deny or any PDP fault → [`PlatformDsrApiError::CascadeAuthorizationDenied`]
///    (403). The caller-supplied `allowed_surfaces` blob no longer exists and
///    confers no authority.
///
/// All of this runs BEFORE the idempotency lookup and any directory mutation, so
/// an unauthorized request never touches state. The idempotency key and the
/// completion record are derived from the VERIFIED principal, not the caller
/// blob.
pub fn execute_dsr_cascade_from_api(
    verified: &VerifiedDsrPrincipal,
    authz: &DsrCascadeAuthzProvider,
    directory: &mut PlatformDsrCascadeDirectory,
    idempotency_ledger: &mut PlatformDsrCascadeExecuteIdempotencyLedger,
    request: PlatformDsrCascadeExecuteApiRequest,
) -> Result<PlatformDsrCascadeExecuteSuccessResponse, PlatformDsrApiError> {
    validate_platform_dsr_cascade_execute_request(&request)?;
    cross_check_verified_principal(verified, &request.principal, &request.body.tenant_id)?;
    ensure_cascade_authorized(
        authz,
        verified,
        &request.body.dsr_id,
        &request.boundary.request_id,
    )?;
    let key = idempotency_key_for(
        &request.boundary,
        verified,
        PLATFORM_DSR_CASCADE_EXECUTE_SURFACE,
    );
    let fingerprint = cascade_fingerprint_for(verified, &request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(PlatformDsrApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let tenant_id = request.body.tenant_id.clone();
    let dsr_id = request.body.dsr_id.clone();
    let completion_key = PlatformDsrCompletionKey {
        tenant_id: tenant_id.clone(),
        dsr_id: dsr_id.clone(),
    };
    if directory.completions.contains_key(&completion_key) {
        return Err(PlatformDsrApiError::CascadeAlreadyCompleted { tenant_id, dsr_id });
    }

    let cascade = build_dsr_cascade(request.body)?;
    let record = completion_record(&cascade);
    let response = PlatformDsrCascadeExecuteSuccessResponse::accepted(record.clone(), request_id);
    directory.completions.insert(completion_key, record);
    idempotency_ledger.entries.insert(
        key,
        PlatformDsrCascadeLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

struct PlatformDsrBuiltCascade {
    request: DsrRequest,
    completion: DsrCompletionRecord,
    dispatch_count: usize,
}

fn build_dsr_cascade(
    request: PlatformDsrCascadeExecuteRequest,
) -> Result<PlatformDsrBuiltCascade, PlatformDsrApiError> {
    let action = dsr_action_from_label(&request.action)?;
    let dsr_request = DsrRequest::new(DsrRequestCreate {
        dsr_id: request.dsr_id.clone(),
        tenant_id: request.tenant_id.clone(),
        region: request.region,
        subject_ref: request.subject_ref,
        action,
        sla_tier: dsr_sla_tier_from_label(&request.sla_tier)?,
        data_classes: request
            .data_classes
            .iter()
            .map(|data_class| parse_privacy_data_class_label(data_class))
            .collect::<Result<Vec<_>, _>>()?,
        received_at_epoch_seconds: request.received_at_epoch_seconds,
        deadline_epoch_seconds: request.deadline_epoch_seconds,
    })
    .map_err(PlatformDsrApiError::Kernel)?;

    let mut dispatches = Vec::with_capacity(request.targets.len());
    let mut proofs = Vec::with_capacity(request.targets.len());
    let mut acks = Vec::with_capacity(request.targets.len());
    for target in request.targets {
        let store = store_ref_from_target(&target, &dsr_request.tenant_id.value)?;
        let dispatch = DsrDispatch::new(
            DsrDispatchCreate {
                dispatch_id: target.dispatch_id.clone(),
                idempotency_key: target.dispatch_idempotency_key,
                store: store.clone(),
                dispatched_at_epoch_seconds: target.processed_at_epoch_seconds,
            },
            &dsr_request,
        )
        .map_err(PlatformDsrApiError::Kernel)?;
        let ack_status = ack_status_from_label(&target.ack_status)?;
        let proof = if ack_status == DsrAckStatus::Completed {
            Some(
                ErasureProof::new(
                    ErasureProofCreate {
                        proof_id: require_target_field(&target.proof_id, "body.targets.proof_id")?,
                        dispatch_id: dispatch.dispatch_id.value.clone(),
                        dsr_id: dsr_request.dsr_id.value.clone(),
                        action,
                        store: store.clone(),
                        method: proof_method_from_label(&require_target_field(
                            &target.proof_method,
                            "body.targets.proof_method",
                        )?)?,
                        evidence_hash: require_target_field(
                            &target.evidence_hash,
                            "body.targets.evidence_hash",
                        )?,
                        witness_ref: require_target_field(
                            &target.witness_ref,
                            "body.targets.witness_ref",
                        )?,
                        signer_ref: require_target_field(
                            &target.signer_ref,
                            "body.targets.signer_ref",
                        )?,
                        signature_ref: require_target_field(
                            &target.signature_ref,
                            "body.targets.signature_ref",
                        )?,
                        rekor_log_index: target.rekor_log_index.ok_or_else(|| {
                            PlatformDsrApiError::MissingCompletedProofField {
                                field: "body.targets.rekor_log_index".to_string(),
                            }
                        })?,
                        proved_at_epoch_seconds: target.processed_at_epoch_seconds,
                    },
                    &dispatch,
                )
                .map_err(PlatformDsrApiError::Kernel)?,
            )
        } else {
            None
        };
        let ack = DsrCascadeAck::new(
            DsrCascadeAckCreate {
                ack_id: target.ack_id,
                dispatch_id: dispatch.dispatch_id.value.clone(),
                dsr_id: dsr_request.dsr_id.value.clone(),
                status: ack_status,
                reason: target
                    .ack_reason
                    .as_deref()
                    .map(ack_reason_from_label)
                    .transpose()?,
                proof_id: proof.as_ref().map(|proof| proof.proof_id.value.clone()),
                evidence_hash: proof
                    .as_ref()
                    .map(|proof| proof.evidence_hash.value.clone()),
                acknowledged_at_epoch_seconds: target.processed_at_epoch_seconds,
            },
            &dispatch,
            proof.as_ref(),
        )
        .map_err(PlatformDsrApiError::Kernel)?;
        dispatches.push(dispatch);
        if let Some(proof) = proof {
            proofs.push(proof);
        }
        acks.push(ack);
    }

    let completion = DsrCompletionRecord::new(
        DsrCompletionRecordCreate {
            completion_id: request.completion_id,
            dsr_id: dsr_request.dsr_id.value.clone(),
            dispatches,
            acks,
            proofs,
            aggregate_proof_hash: request.aggregate_proof_hash,
            signer_ref: request.signer_ref,
            signature_ref: request.signature_ref,
            rekor_log_index: request.rekor_log_index,
            completed_at_epoch_seconds: request.completed_at_epoch_seconds,
        },
        &dsr_request,
    )
    .map_err(PlatformDsrApiError::Kernel)?;

    let dispatch_count = completion.dispatch_ids.value.len();
    Ok(PlatformDsrBuiltCascade {
        request: dsr_request,
        completion,
        dispatch_count,
    })
}

fn validate_boundary(
    boundary: &PlatformDsrCascadeBoundaryContext,
) -> Result<(), PlatformDsrApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(PlatformDsrApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(PlatformDsrApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(PlatformDsrApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_dsr_id(dsr_id: &str) -> Result<(), PlatformDsrApiError> {
    if dsr_id.trim().is_empty() {
        return Err(PlatformDsrApiError::InvalidDsrId {
            dsr_id: dsr_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    header_tenant_id: &str,
    principal: &PlatformDsrApiPrincipal,
    body_tenant_id: Option<&str>,
) -> Result<(), PlatformDsrApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(PlatformDsrApiError::EmptyPrincipalId);
    }
    if header_tenant_id != principal.tenant_id
        || body_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id)
    {
        return Err(PlatformDsrApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.map(str::to_string),
        });
    }
    Ok(())
}

/// Cross-check the VERIFIED principal against the caller-asserted identity.
///
/// The verified principal (minted from an unforgeable credential) is
/// authoritative. The caller-asserted `request.principal` and the body tenant
/// are forgeable inputs; if any of them disagrees with the verified identity the
/// request is rejected (403) — the caller cannot substitute a different identity
/// than the one the credential proves. This binds the idempotency key and the
/// completion record to the verified principal.
fn cross_check_verified_principal(
    verified: &VerifiedDsrPrincipal,
    principal: &PlatformDsrApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), PlatformDsrApiError> {
    if verified.tenant_id() != principal.tenant_id || verified.tenant_id() != body_tenant_id {
        return Err(PlatformDsrApiError::VerifiedTenantMismatch {
            verified_tenant_id: verified.tenant_id().to_string(),
            claimed_tenant_id: principal.tenant_id.clone(),
        });
    }
    if verified.principal_id() != principal.principal_id {
        return Err(PlatformDsrApiError::VerifiedPrincipalMismatch {
            verified_principal_id: verified.principal_id().to_string(),
            claimed_principal_id: principal.principal_id.clone(),
        });
    }
    Ok(())
}

/// Run the server-side PDP decision (fail-closed). The PDP resource is bound to
/// the VERIFIED principal's tenant (the target tenant whose stores the cascade
/// erases — cross-checked equal to `body.tenant_id` by
/// [`cross_check_verified_principal`]) and the TARGET dsr id from the trusted
/// path binding. It is NOT bound to a forged `allowed_surfaces` blob, so a
/// cross-tenant erasure (a principal targeting another tenant's stores) is
/// deniable: the PDP sees the actual `{principal, target tenant, dsr}` tuple.
///
/// A deny OR any PDP fault maps to
/// [`PlatformDsrApiError::CascadeAuthorizationDenied`] (403, NOT 500).
fn ensure_cascade_authorized(
    authz: &DsrCascadeAuthzProvider,
    verified: &VerifiedDsrPrincipal,
    target_dsr_id: &str,
    request_id: &str,
) -> Result<(), PlatformDsrApiError> {
    let resource = DsrCascadeResource {
        tenant_id: verified.tenant_id().to_string(),
        dsr_id: target_dsr_id.to_string(),
        surface: PLATFORM_DSR_CASCADE_EXECUTE_SURFACE.to_string(),
        request_id: request_id.to_string(),
    };
    match authz.ensure_authorized(verified, &resource) {
        Ok(()) => Ok(()),
        Err(DsrCascadeAuthorizationError::Denied | DsrCascadeAuthorizationError::Refused) => {
            Err(PlatformDsrApiError::CascadeAuthorizationDenied {
                surface: PLATFORM_DSR_CASCADE_EXECUTE_SURFACE.to_string(),
            })
        }
    }
}

fn store_ref_from_target(
    target: &PlatformDsrCascadeTargetRequest,
    tenant_id: &str,
) -> Result<DsrStoreRef, PlatformDsrApiError> {
    DsrStoreRef::new(DsrStoreRefCreate {
        axis: dsr_axis_from_label(&target.axis)?,
        kind: dsr_store_kind_from_label(&target.store_kind)?,
        store_id: target.store_id.clone(),
        tenant_id: tenant_id.to_string(),
        region: target.region.clone(),
        cell_id: target.cell_id.clone(),
        record_ref: target.record_ref.clone(),
        data_class: parse_privacy_data_class_label(&target.data_class)?,
    })
    .map_err(PlatformDsrApiError::Kernel)
}

fn parse_privacy_data_class_label(
    data_class: &str,
) -> Result<oya_data_boundary_kernel::PrivacyDataClass, PlatformDsrApiError> {
    let parsed = parse_data_class_label(data_class).ok_or_else(|| {
        PlatformDsrApiError::InvalidDataClassLabel {
            data_class: data_class.to_string(),
        }
    })?;
    platform_dsr_data_class_from_legacy(parsed).map_err(|_| {
        PlatformDsrApiError::InvalidDataClassLabel {
            data_class: data_class.to_string(),
        }
    })
}

fn dsr_action_from_label(action: &str) -> Result<DsrAction, PlatformDsrApiError> {
    match action.trim() {
        "erase" => Ok(DsrAction::Erase),
        "correct" => Ok(DsrAction::Correct),
        "export" => Ok(DsrAction::Export),
        "restrict" => Ok(DsrAction::Restrict),
        "object_to_processing" => Ok(DsrAction::ObjectToProcessing),
        "automated_decision_opt_out" => Ok(DsrAction::AutomatedDecisionOptOut),
        _ => Err(PlatformDsrApiError::InvalidActionLabel {
            action: action.to_string(),
        }),
    }
}

fn action_label(action: DsrAction) -> &'static str {
    match action {
        DsrAction::Erase => "erase",
        DsrAction::Correct => "correct",
        DsrAction::Export => "export",
        DsrAction::Restrict => "restrict",
        DsrAction::ObjectToProcessing => "object_to_processing",
        DsrAction::AutomatedDecisionOptOut => "automated_decision_opt_out",
    }
}

fn dsr_sla_tier_from_label(sla_tier: &str) -> Result<DsrSlaTier, PlatformDsrApiError> {
    match sla_tier.trim() {
        "preview" => Ok(DsrSlaTier::Preview),
        "stable" => Ok(DsrSlaTier::Stable),
        "ga" => Ok(DsrSlaTier::Ga),
        _ => Err(PlatformDsrApiError::InvalidSlaTierLabel {
            sla_tier: sla_tier.to_string(),
        }),
    }
}

fn dsr_axis_from_label(axis: &str) -> Result<DsrAxis, PlatformDsrApiError> {
    match axis.trim() {
        "saas" => Ok(DsrAxis::Saas),
        "workspace" => Ok(DsrAxis::Workspace),
        "vertical" => Ok(DsrAxis::Vertical),
        "foundry" => Ok(DsrAxis::Foundry),
        "cloud" => Ok(DsrAxis::Cloud),
        "search" => Ok(DsrAxis::Search),
        "ads" => Ok(DsrAxis::Ads),
        "analytics" => Ok(DsrAxis::Analytics),
        _ => Err(PlatformDsrApiError::InvalidAxisLabel {
            axis: axis.to_string(),
        }),
    }
}

fn dsr_store_kind_from_label(store_kind: &str) -> Result<DsrStoreKind, PlatformDsrApiError> {
    match store_kind.trim() {
        "tenant_table" => Ok(DsrStoreKind::TenantTable),
        "workspace_object" => Ok(DsrStoreKind::WorkspaceObject),
        "vertical_record" => Ok(DsrStoreKind::VerticalRecord),
        "foundry_memory" => Ok(DsrStoreKind::FoundryMemory),
        "cloud_resource" => Ok(DsrStoreKind::CloudResource),
        "search_index" => Ok(DsrStoreKind::SearchIndex),
        "ads_attribution" => Ok(DsrStoreKind::AdsAttribution),
        "analytics_warehouse" => Ok(DsrStoreKind::AnalyticsWarehouse),
        _ => Err(PlatformDsrApiError::InvalidStoreKindLabel {
            store_kind: store_kind.to_string(),
        }),
    }
}

fn proof_method_from_label(proof_method: &str) -> Result<DsrProofMethod, PlatformDsrApiError> {
    match proof_method.trim() {
        "kms_shred" => Ok(DsrProofMethod::KmsShred),
        "record_delete" => Ok(DsrProofMethod::RecordDelete),
        "index_rebuild" => Ok(DsrProofMethod::IndexRebuild),
        "cold_storage_purge" => Ok(DsrProofMethod::ColdStoragePurge),
        "correction_applied" => Ok(DsrProofMethod::CorrectionApplied),
        "export_produced" => Ok(DsrProofMethod::ExportProduced),
        "restrict_applied" => Ok(DsrProofMethod::RestrictApplied),
        "objection_applied" => Ok(DsrProofMethod::ObjectionApplied),
        "automated_decision_opt_out_applied" => Ok(DsrProofMethod::AutomatedDecisionOptOutApplied),
        _ => Err(PlatformDsrApiError::InvalidProofMethodLabel {
            proof_method: proof_method.to_string(),
        }),
    }
}

fn ack_status_from_label(ack_status: &str) -> Result<DsrAckStatus, PlatformDsrApiError> {
    match ack_status.trim() {
        "accepted" => Ok(DsrAckStatus::Accepted),
        "completed" => Ok(DsrAckStatus::Completed),
        "retryable_failure" => Ok(DsrAckStatus::RetryableFailure),
        "permanent_block" => Ok(DsrAckStatus::PermanentBlock),
        _ => Err(PlatformDsrApiError::InvalidAckStatusLabel {
            ack_status: ack_status.to_string(),
        }),
    }
}

fn ack_reason_from_label(ack_reason: &str) -> Result<DsrAckReason, PlatformDsrApiError> {
    match ack_reason.trim() {
        "lawful_retention" => Ok(DsrAckReason::LawfulRetention),
        "subject_identity_unverified" => Ok(DsrAckReason::SubjectIdentityUnverified),
        "store_unavailable" => Ok(DsrAckReason::StoreUnavailable),
        "unsupported_action" => Ok(DsrAckReason::UnsupportedAction),
        "residency_conflict" => Ok(DsrAckReason::ResidencyConflict),
        "integrity_check_failed" => Ok(DsrAckReason::IntegrityCheckFailed),
        _ => Err(PlatformDsrApiError::InvalidAckReasonLabel {
            ack_reason: ack_reason.to_string(),
        }),
    }
}

fn completion_status_label(status: compliance_dsr::DsrCompletionStatus) -> &'static str {
    match status {
        compliance_dsr::DsrCompletionStatus::Completed => "completed",
        compliance_dsr::DsrCompletionStatus::CompletedWithBlocks => "completed_with_blocks",
    }
}

fn sla_status_label(status: DsrSlaStatus) -> &'static str {
    match status {
        DsrSlaStatus::WithinSla => "within_sla",
        DsrSlaStatus::Breached => "breached",
    }
}

fn require_target_field(
    field: &Option<String>,
    field_name: &str,
) -> Result<String, PlatformDsrApiError> {
    field
        .clone()
        .ok_or_else(|| PlatformDsrApiError::MissingCompletedProofField {
            field: field_name.to_string(),
        })
}

fn idempotency_key_for(
    boundary: &PlatformDsrCascadeBoundaryContext,
    verified: &VerifiedDsrPrincipal,
    surface: &str,
) -> PlatformDsrIdempotencyLedgerKey {
    // Keyed on the VERIFIED identity, never the caller-asserted blob: a forged
    // principal cannot collide with (or replay) another principal's ledger entry.
    PlatformDsrIdempotencyLedgerKey {
        tenant_id: verified.tenant_id().to_string(),
        principal_id: verified.principal_id().to_string(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn cascade_fingerprint_for(
    verified: &VerifiedDsrPrincipal,
    request: &PlatformDsrCascadeExecuteApiRequest,
) -> PlatformDsrRequestFingerprint {
    let targets = request
        .body
        .targets
        .iter()
        .map(|target| {
            [
                format!("dispatch_id={}", target.dispatch_id),
                format!(
                    "dispatch_idempotency_key={}",
                    target.dispatch_idempotency_key
                ),
                format!("ack_id={}", target.ack_id),
                format!("ack_status={}", target.ack_status),
                format!("ack_reason={:?}", target.ack_reason),
                format!("axis={}", target.axis),
                format!("store_kind={}", target.store_kind),
                format!("store_id={}", target.store_id),
                format!("region={}", target.region),
                format!("cell_id={}", target.cell_id),
                format!("record_ref={}", target.record_ref),
                format!("data_class={}", target.data_class),
                format!("proof_id={:?}", target.proof_id),
                format!("proof_method={:?}", target.proof_method),
                format!("evidence_hash={:?}", target.evidence_hash),
                format!("witness_ref={:?}", target.witness_ref),
                format!("signer_ref={:?}", target.signer_ref),
                format!("signature_ref={:?}", target.signature_ref),
                format!("rekor_log_index={:?}", target.rekor_log_index),
                format!(
                    "processed_at_epoch_seconds={}",
                    target.processed_at_epoch_seconds
                ),
            ]
            .join(",")
        })
        .collect::<Vec<_>>()
        .join(";");
    PlatformDsrRequestFingerprint {
        canonical: [
            format!("path.dsr_id={}", request.path_dsr_id),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("verified.tenant_id={}", verified.tenant_id()),
            format!("verified.principal_id={}", verified.principal_id()),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!("body.dsr_id={}", request.body.dsr_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.region={}", request.body.region),
            format!("body.subject_ref={}", request.body.subject_ref),
            format!("body.action={}", request.body.action),
            format!("body.sla_tier={}", request.body.sla_tier),
            format!("body.data_classes={}", request.body.data_classes.join(",")),
            format!(
                "body.received_at_epoch_seconds={}",
                request.body.received_at_epoch_seconds
            ),
            format!(
                "body.deadline_epoch_seconds={}",
                request.body.deadline_epoch_seconds
            ),
            format!("body.completion_id={}", request.body.completion_id),
            format!(
                "body.aggregate_proof_hash={}",
                request.body.aggregate_proof_hash
            ),
            format!("body.signer_ref={}", request.body.signer_ref),
            format!("body.signature_ref={}", request.body.signature_ref),
            format!("body.rekor_log_index={}", request.body.rekor_log_index),
            format!(
                "body.completed_at_epoch_seconds={}",
                request.body.completed_at_epoch_seconds
            ),
            format!("body.targets={targets}"),
        ]
        .join("|"),
    }
}

fn completion_record(cascade: &PlatformDsrBuiltCascade) -> PlatformDsrCompletionRecord {
    PlatformDsrCompletionRecord {
        dsr_id: cascade.request.dsr_id.value.clone(),
        tenant_id: cascade.request.tenant_id.value.clone(),
        subject_ref: cascade.request.subject_ref.value.clone(),
        action: action_label(cascade.request.action.value).to_string(),
        completion_id: cascade.completion.completion_id.value.clone(),
        completion_status: completion_status_label(cascade.completion.completion_status.value)
            .to_string(),
        sla_status: sla_status_label(cascade.completion.sla_status.value).to_string(),
        dispatch_ids: cascade.completion.dispatch_ids.value.clone(),
        ack_ids: cascade.completion.ack_ids.value.clone(),
        proof_ids: cascade.completion.proof_ids.value.clone(),
        aggregate_proof_hash: cascade.completion.aggregate_proof_hash.value.clone(),
        signer_ref: cascade.completion.signer_ref.value.clone(),
        signature_ref: cascade.completion.signature_ref.value.clone(),
        rekor_log_index: cascade.completion.rekor_log_index.value,
        completed_at_epoch_seconds: cascade.completion.completed_at_epoch_seconds.value,
        schema_version: cascade.completion.schema_version.value,
        store_count: cascade.dispatch_count as u64,
    }
}

fn detail(field: &str, issue: &str) -> PlatformDsrApiErrorDetail {
    PlatformDsrApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

fn platform_dsr_error_message(error: &PlatformDsrError) -> &'static str {
    match error {
        PlatformDsrError::InvalidDsrId => "DSR id is invalid",
        PlatformDsrError::InvalidTenantId => "DSR tenant id is invalid",
        PlatformDsrError::InvalidRegion => "DSR region is invalid",
        PlatformDsrError::InvalidSubjectRef => "DSR subject ref is invalid",
        PlatformDsrError::InvalidStoreId => "DSR store id is invalid",
        PlatformDsrError::InvalidCellId => "DSR cell id is invalid",
        PlatformDsrError::InvalidRecordRef => "DSR record ref is invalid",
        PlatformDsrError::InvalidDispatchId => "DSR dispatch id is invalid",
        PlatformDsrError::InvalidIdempotencyKey => "DSR dispatch idempotency key is invalid",
        PlatformDsrError::InvalidAckId => "DSR acknowledgement id is invalid",
        PlatformDsrError::InvalidProofId => "DSR proof id is invalid",
        PlatformDsrError::InvalidCompletionId => "DSR completion id is invalid",
        PlatformDsrError::InvalidWitnessRef => "DSR witness ref is invalid",
        PlatformDsrError::InvalidSignerRef => "DSR signer ref is invalid",
        PlatformDsrError::InvalidSignatureRef => "DSR signature ref is invalid",
        PlatformDsrError::InvalidEvidenceHash => "DSR evidence hash is invalid",
        PlatformDsrError::InvalidAggregateProofHash => "DSR aggregate proof hash is invalid",
        PlatformDsrError::EmptyDataClassSet => "DSR data-class set is required",
        PlatformDsrError::DuplicateDataClass => "DSR data classes must be unique",
        PlatformDsrError::DeadlineExceedsSla => "DSR deadline exceeds SLA tier",
        PlatformDsrError::InvalidTimeOrder => "DSR timestamps are out of order",
        PlatformDsrError::StoreOutOfScope => "DSR target store is out of scope",
        PlatformDsrError::DataClassOutOfScope => "DSR target data class is out of scope",
        PlatformDsrError::ProofDispatchMismatch => "DSR proof does not match dispatch",
        PlatformDsrError::ProofMethodMismatch => "DSR proof method does not match action",
        PlatformDsrError::AckDispatchMismatch => "DSR acknowledgement does not match dispatch",
        PlatformDsrError::AckStatusInvalid => "DSR acknowledgement status is invalid",
        PlatformDsrError::AckProofMismatch => "DSR acknowledgement proof does not match",
        PlatformDsrError::EmptyDispatchSet => "DSR dispatch set is required",
        PlatformDsrError::DuplicateDispatchId => "DSR dispatch ids must be unique",
        PlatformDsrError::DuplicateStoreRef => "DSR store refs must be unique",
        PlatformDsrError::EmptyAckSet => "DSR acknowledgement set is required",
        PlatformDsrError::MissingDispatchAck => "DSR dispatch acknowledgement is missing",
        PlatformDsrError::DuplicateAckDispatchId => {
            "DSR acknowledgements must target dispatches once"
        }
        PlatformDsrError::NonTerminalAck => "DSR acknowledgement must be terminal",
        PlatformDsrError::MissingCompletedProof => "Completed DSR acknowledgement needs proof",
        PlatformDsrError::DuplicateProofId => "DSR proof ids must be unique",
        PlatformDsrError::ProofCoverageMismatch => "DSR proof coverage does not match cascade",
        PlatformDsrError::InvalidDataClass => "DSR data class is invalid",
    }
}

fn platform_dsr_error_issue(error: &PlatformDsrError) -> &'static str {
    match error {
        PlatformDsrError::InvalidDsrId => "dsr_id must be non-empty",
        PlatformDsrError::InvalidTenantId => "tenant_id must be non-empty",
        PlatformDsrError::InvalidRegion => "region must be non-empty",
        PlatformDsrError::InvalidSubjectRef => "subject_ref must be non-empty",
        PlatformDsrError::InvalidStoreId => "store_id must be non-empty",
        PlatformDsrError::InvalidCellId => "cell_id must be non-empty",
        PlatformDsrError::InvalidRecordRef => "record_ref must be non-empty",
        PlatformDsrError::InvalidDispatchId => "dispatch_id must be non-empty",
        PlatformDsrError::InvalidIdempotencyKey => "dispatch idempotency key must be non-empty",
        PlatformDsrError::InvalidAckId => "ack_id must be non-empty",
        PlatformDsrError::InvalidProofId => "proof_id must be non-empty",
        PlatformDsrError::InvalidCompletionId => "completion_id must be non-empty",
        PlatformDsrError::InvalidWitnessRef => "witness_ref must be non-empty",
        PlatformDsrError::InvalidSignerRef => "signer_ref must be non-empty",
        PlatformDsrError::InvalidSignatureRef => "signature_ref must be non-empty",
        PlatformDsrError::InvalidEvidenceHash => "evidence hash must be sha256-prefixed",
        PlatformDsrError::InvalidAggregateProofHash => {
            "aggregate proof hash must be sha256-prefixed"
        }
        PlatformDsrError::EmptyDataClassSet => "data_classes must be non-empty",
        PlatformDsrError::DuplicateDataClass => "data_classes cannot repeat",
        PlatformDsrError::DeadlineExceedsSla => "deadline must fit the SLA tier",
        PlatformDsrError::InvalidTimeOrder => "timestamps must be monotonic",
        PlatformDsrError::StoreOutOfScope => "store tenant and region must match request",
        PlatformDsrError::DataClassOutOfScope => "store data class must be requested",
        PlatformDsrError::ProofDispatchMismatch => {
            "proof must match dispatch id, action, and store"
        }
        PlatformDsrError::ProofMethodMismatch => "proof method must match action",
        PlatformDsrError::AckDispatchMismatch => "ack must match a dispatch for this DSR",
        PlatformDsrError::AckStatusInvalid => "ack status and proof fields must align",
        PlatformDsrError::AckProofMismatch => "ack proof id must target the same dispatch",
        PlatformDsrError::EmptyDispatchSet => "targets must be non-empty",
        PlatformDsrError::DuplicateDispatchId => "dispatch ids cannot repeat",
        PlatformDsrError::DuplicateStoreRef => "store refs cannot repeat",
        PlatformDsrError::EmptyAckSet => "acks must be non-empty",
        PlatformDsrError::MissingDispatchAck => "every dispatch requires terminal ack",
        PlatformDsrError::DuplicateAckDispatchId => "acks cannot repeat dispatch ids",
        PlatformDsrError::NonTerminalAck => "completion only accepts terminal acks",
        PlatformDsrError::MissingCompletedProof => "completed acks require proofs",
        PlatformDsrError::DuplicateProofId => "proof ids cannot repeat",
        PlatformDsrError::ProofCoverageMismatch => "proof coverage must match completed acks",
        PlatformDsrError::InvalidDataClass => "data class must be a privacy data class",
    }
}
