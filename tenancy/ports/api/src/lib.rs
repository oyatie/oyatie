//! Platform Tenant API boundary.
//!
//! This crate owns authenticated REST-boundary normalization, path/body tenant
//! binding, request fingerprint idempotency, and global tenant-id uniqueness for
//! `tenant.create` before handing typed construction to the platform tenant
//! kernel.
//!
//! ## Authorization posture (AUTH-005, SECURITY remediation — fail-closed)
//!
//! Authorization is decided SERVER-SIDE by a Policy Decision Point ([`TenantCreateAuthorizer`])
//! the boundary OWNS, NEVER from caller-supplied decision fields. The historical
//! `TenantApiAuthorization` request DTO (which carried a caller-supplied
//! `{decision_id, tenant_id, principal_id, allowed_surfaces}` grant the boundary
//! merely cross-checked against the caller-supplied principal) was a self-attested
//! authorization: any caller could supply `allowed_surfaces: ["tenant.create"]`
//! and a matching principal and be authorized. It is REMOVED.
//!
//! The boundary now:
//!   - takes an UNFORGEABLE [`VerifiedTenantPrincipal`] (private fields, no public
//!     constructor — only a credential verifier outside this crate can mint one;
//!     it is never deserialized from the request), and
//!   - asks the injected [`TenantCreateAuthorizer`] PDP to `decide()` the
//!     `tenant.create` surface against the TARGET tenant (the path tenant id,
//!     a trusted source) on a separate axis from the caller's own tenant.
//!
//! Default-deny: no path reaches the directory mutation without a PDP `Ok(true)`.
//! Any PDP fault (error/timeout/unavailability) maps to a fail-closed deny, never
//! an allow (see the [`TenantCreateAuthorizer`] adapter contract).

use std::collections::BTreeMap;

use network_residency::parse_residency_class_label;
use tenancy_domain::{Tenant, TenantError};

pub const TENANT_CREATE_SURFACE: &str = "tenant.create";
pub const TENANT_CREATE_OPENAPI_CONTRACT: &str =
    "contracts/openapi/platform/platform-tenant-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TenantCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl TenantCreateApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TenantCreateApiErrorCode {
    RequestIdEmpty,
    OperatorTenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathTenantIdEmpty,
    TenantPathBodyMismatch,
    /// The server-side PDP returned an explicit deny for the verified caller.
    AuthorizationDenied,
    /// The server-side PDP adapter faulted (error/timeout/unavailability);
    /// fail-closed → deny.
    AuthorizationFault,
    ResidencyClassInvalid,
    DuplicateTenant,
    IdempotencyKeyReused,
    TenantInvalidTenantId,
    TenantLegalNameEmpty,
    TenantHomeRegionEmpty,
    TenantHomeRegionDenied,
    TenantRegionalPackMissing,
}

impl TenantCreateApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "TENANT_CREATE_REQUEST_ID_EMPTY",
            Self::OperatorTenantHeaderEmpty => "TENANT_CREATE_OPERATOR_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "TENANT_CREATE_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "TENANT_CREATE_PRINCIPAL_ID_EMPTY",
            Self::PathTenantIdEmpty => "TENANT_CREATE_PATH_TENANT_ID_EMPTY",
            Self::TenantPathBodyMismatch => "TENANT_CREATE_PATH_BODY_MISMATCH",
            Self::AuthorizationDenied => "TENANT_CREATE_AUTHORIZATION_DENIED",
            Self::AuthorizationFault => "TENANT_CREATE_AUTHORIZATION_FAULT",
            Self::ResidencyClassInvalid => "TENANT_CREATE_RESIDENCY_CLASS_INVALID",
            Self::DuplicateTenant => "TENANT_CREATE_DUPLICATE_TENANT",
            Self::IdempotencyKeyReused => "TENANT_CREATE_IDEMPOTENCY_KEY_REUSED",
            Self::TenantInvalidTenantId => "TENANT_CREATE_TENANT_INVALID_ID",
            Self::TenantLegalNameEmpty => "TENANT_CREATE_LEGAL_NAME_EMPTY",
            Self::TenantHomeRegionEmpty => "TENANT_CREATE_HOME_REGION_EMPTY",
            Self::TenantHomeRegionDenied => "TENANT_CREATE_HOME_REGION_DENIED",
            Self::TenantRegionalPackMissing => "TENANT_CREATE_REGIONAL_PACK_MISSING",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

/// A caller principal whose credential a verifier OUTSIDE this crate has
/// PROVEN. The inner fields are private and there is NO public constructor, so a
/// [`VerifiedTenantPrincipal`] can ONLY be minted by a verifier that checked an
/// UNFORGEABLE credential (an mTLS/SPIFFE peer identity or a constant-time bearer
/// compare in the boundary adapter). The boundary NEVER deserializes one from the
/// request body or from caller-supplied `x-principal-*`/`x-authorization-*`
/// headers — it is the type-level proof that authentication ran before any
/// authorization or mutation, and the cross-tenant axis the PDP checks against.
///
/// Construction lives behind `pub(crate)` + a `#[cfg(test)]` test constructor so
/// no downstream crate can forge one (the SECURITY remediation lesson: a public
/// constructor / public fields is a forgeable token — do NOT repeat it).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedTenantPrincipal {
    /// The verified caller's own tenant (derived from the credential, NEVER from
    /// a request field). Distinct axis from the TARGET tenant the PDP authorizes.
    caller_tenant_id: String, // data_class: INTERNAL_ONLY
    /// A stable identity label for the verified caller (credential subject), for
    /// the PDP request + audit attribution.
    caller_principal_id: String, // data_class: INTERNAL_ONLY
}

impl VerifiedTenantPrincipal {
    /// Mint a verified principal. `pub(crate)` so ONLY this crate's verifier
    /// adapter (which proved an unforgeable credential) can construct one — a
    /// downstream handler cannot fabricate authority from caller-supplied input.
    #[must_use]
    pub(crate) fn new(
        caller_tenant_id: impl Into<String>,
        caller_principal_id: impl Into<String>,
    ) -> Self {
        Self {
            caller_tenant_id: caller_tenant_id.into(),
            caller_principal_id: caller_principal_id.into(),
        }
    }

    /// Test-only constructor: lets tests mint a verified principal WITHOUT a live
    /// credential verifier. Gated behind `#[cfg(test)]` so it never exists in a
    /// release build — production code can only obtain one from a real verifier.
    #[cfg(test)]
    #[must_use]
    pub fn for_test(
        caller_tenant_id: impl Into<String>,
        caller_principal_id: impl Into<String>,
    ) -> Self {
        Self::new(caller_tenant_id, caller_principal_id)
    }

    /// The verified caller's own tenant.
    #[must_use]
    pub fn caller_tenant_id(&self) -> &str {
        &self.caller_tenant_id
    }

    /// The verified caller's identity label.
    #[must_use]
    pub fn caller_principal_id(&self) -> &str {
        &self.caller_principal_id
    }
}

/// A fully-bound, server-side tenant-create authorization request handed to the
/// PDP. `caller_tenant_id`/`caller_principal_id` come from the VERIFIED principal
/// (trusted); `target_tenant_id` is the TARGET tenant (the request path tenant id
/// — a trusted source, NOT a caller-supplied grant) on a SEPARATE axis so a PDP
/// that enforces isolation can DENY a cross-tenant create (true blast radius / no
/// IDOR). `surface` is the action slug (`tenant.create`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateAuthzRequest<'a> {
    pub caller_tenant_id: &'a str, // data_class: INTERNAL_ONLY (verified)
    pub caller_principal_id: &'a str, // data_class: INTERNAL_ONLY (verified)
    pub target_tenant_id: &'a str, // data_class: INTERNAL_ONLY (path-derived)
    pub surface: &'a str,          // data_class: PUBLIC
}

/// A fail-closed PDP fault. Any adapter error/timeout/unavailability maps to this
/// and the boundary DENIES (403) — a PDP outage never allows and never panics
/// (release builds use `panic = "abort"`, so `catch_unwind` is NOT a backstop;
/// the adapter MUST surface failures as `Err(AuthzFault)`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthzFault {
    detail: String, // data_class: INTERNAL_ONLY
}

impl AuthzFault {
    /// Construct a fault with a human-facing detail.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    /// Borrow the detail string.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// The server-side tenant-create authorization PORT (the PDP seam). The boundary
/// OWNS this port; the concrete PDP client + credential store are adapters
/// OUTSIDE this crate (owned-W5 shape — a cloud-iam Cedar PDP client). Returns
/// `Ok(true)` to PERMIT, `Ok(false)` to DENY, and `Err(AuthzFault)` for any
/// adapter fault — the boundary maps BOTH `Ok(false)` and `Err(_)` to a 403
/// (fail-closed, default-deny).
///
/// # Adapter contract (MUST be upheld by every implementation)
///
/// 1. **Fault mapping**: every error, timeout, or unavailability MUST map to
///    `Err(AuthzFault)` — never panic, never return `Ok(true)` on failure.
/// 2. **Deadline enforcement**: the adapter MUST enforce its own call deadline.
/// 3. **No panics**: the adapter MUST NOT panic. Release builds abort on panic,
///    so `catch_unwind` is NOT a backstop — surface failures as `Err`.
/// 4. **Cross-tenant deny is authoritative**: a request whose `target_tenant_id`
///    the verified caller has no proven authority over MUST be denied regardless
///    of how many allow rules match.
pub trait TenantCreateAuthorizer {
    /// Decide whether the verified caller may create the target tenant.
    ///
    /// # Errors
    /// Returns [`AuthzFault`] on any PDP adapter failure; the boundary denies.
    fn decide(&self, request: &TenantCreateAuthzRequest<'_>) -> Result<bool, AuthzFault>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRegulatoryPackRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateRequest {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub legal_name: String,                             // data_class: INTERNAL_ONLY
    pub home_region: String,                            // data_class: INTERNAL_ONLY
    pub residency_class: String,                        // data_class: INTERNAL_ONLY
    pub regulatory_packs: Vec<TenantRegulatoryPackRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiRequest {
    pub path_tenant_id: String,             // data_class: INTERNAL_ONLY
    pub boundary: TenantApiBoundaryContext, // data_class: INTERNAL_ONLY
    /// The UNFORGEABLE verified caller. NOT deserialized from the request — it is
    /// minted by a credential verifier (outside this crate) and handed in. There
    /// is no caller-supplied `authorization` grant: authority is decided
    /// server-side by the [`TenantCreateAuthorizer`] PDP.
    pub principal: VerifiedTenantPrincipal, // data_class: INTERNAL_ONLY (verified)
    pub body: TenantCreateRequest,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantDirectory {
    tenants: BTreeMap<String, Tenant>, // data_class: INTERNAL_ONLY
}

impl TenantDirectory {
    pub fn len(&self) -> usize {
        self.tenants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tenants.is_empty()
    }

    pub fn get(&self, tenant_id: &str) -> Option<&Tenant> {
        self.tenants.get(tenant_id)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantCreateIdempotencyLedger {
    entries: BTreeMap<TenantCreateIdempotencyLedgerKey, TenantCreateIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl TenantCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TenantCreateIdempotencyLedgerKey {
    operator_tenant_id: String, // data_class: INTERNAL_ONLY
    principal_id: String,       // data_class: INTERNAL_ONLY
    surface: String,            // data_class: INTERNAL_ONLY
    idempotency_key: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TenantCreateIdempotencyLedgerEntry {
    fingerprint: TenantCreateRequestFingerprint, // data_class: INTERNAL_ONLY
    result: TenantCreateSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TenantCreateRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateSuccessResponse {
    pub data: TenantRecord,             // data_class: INTERNAL_ONLY
    pub metadata: TenantCreateMetadata, // data_class: INTERNAL_ONLY
}

impl TenantCreateSuccessResponse {
    pub fn created(data: TenantRecord, request: &TenantCreateApiRequest) -> Self {
        Self {
            data,
            metadata: TenantCreateMetadata {
                request_id: request.boundary.request_id.clone(),
                // Audit attribution reflects the VERIFIED caller, never a
                // caller-supplied header/grant.
                operator_tenant_id: request.principal.caller_tenant_id().to_owned(),
                principal_id: request.principal.caller_principal_id().to_owned(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateMetadata {
    pub request_id: String,         // data_class: INTERNAL_ONLY
    pub operator_tenant_id: String, // data_class: INTERNAL_ONLY
    pub principal_id: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRecord {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub legal_name: String,                             // data_class: INTERNAL_ONLY
    pub home_region: String,                            // data_class: INTERNAL_ONLY
    pub residency_class: String,                        // data_class: INTERNAL_ONLY
    pub regulatory_packs: Vec<TenantRegulatoryPackRef>, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorResponse {
    pub error: TenantCreateApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorBody {
    pub code: String,                             // data_class: INTERNAL_ONLY
    pub message: String,                          // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,        // data_class: INTERNAL_ONLY
    pub request_id: String,                       // data_class: INTERNAL_ONLY
    pub details: Vec<TenantCreateApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantCreateApiError {
    EmptyRequestId,
    EmptyOperatorTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathTenantId,
    TenantPathBodyMismatch {
        path_tenant_id: String,
        body_tenant_id: String,
    },
    /// The server-side PDP returned an explicit deny (`Ok(false)`).
    AuthorizationDenied {
        surface: String,
    },
    /// The server-side PDP adapter faulted; fail-closed → deny (403).
    AuthorizationFault {
        detail: String,
    },
    InvalidResidencyClass {
        residency_class: String,
    },
    DuplicateTenant {
        tenant_id: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Tenant(TenantError),
}

impl TenantCreateApiError {
    pub fn tenant_create_status(&self) -> TenantCreateApiStatus {
        match self.status_kind() {
            TenantCreateApiStatusKind::BadRequest => TenantCreateApiStatus::BadRequest,
            TenantCreateApiStatusKind::Unauthorized => TenantCreateApiStatus::Unauthorized,
            TenantCreateApiStatusKind::Forbidden => TenantCreateApiStatus::Forbidden,
            TenantCreateApiStatusKind::Conflict => TenantCreateApiStatus::Conflict,
            TenantCreateApiStatusKind::UnprocessableEntity => {
                TenantCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn tenant_create_status_code(&self) -> u16 {
        self.tenant_create_status().code()
    }

    pub fn code(&self) -> TenantCreateApiErrorCode {
        match self {
            Self::EmptyRequestId => TenantCreateApiErrorCode::RequestIdEmpty,
            Self::EmptyOperatorTenantHeader => TenantCreateApiErrorCode::OperatorTenantHeaderEmpty,
            Self::EmptyIdempotencyKey => TenantCreateApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => TenantCreateApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathTenantId => TenantCreateApiErrorCode::PathTenantIdEmpty,
            Self::TenantPathBodyMismatch { .. } => TenantCreateApiErrorCode::TenantPathBodyMismatch,
            Self::AuthorizationDenied { .. } => TenantCreateApiErrorCode::AuthorizationDenied,
            Self::AuthorizationFault { .. } => TenantCreateApiErrorCode::AuthorizationFault,
            Self::InvalidResidencyClass { .. } => TenantCreateApiErrorCode::ResidencyClassInvalid,
            Self::DuplicateTenant { .. } => TenantCreateApiErrorCode::DuplicateTenant,
            Self::IdempotencyKeyReused { .. } => TenantCreateApiErrorCode::IdempotencyKeyReused,
            Self::Tenant(error) => tenant_error_code(error),
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> TenantCreateApiErrorResponse {
        TenantCreateApiErrorResponse {
            error: TenantCreateApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> TenantCreateApiStatusKind {
        match self {
            Self::EmptyPrincipalId => TenantCreateApiStatusKind::Unauthorized,
            Self::AuthorizationDenied { .. } | Self::AuthorizationFault { .. } => {
                TenantCreateApiStatusKind::Forbidden
            }
            Self::DuplicateTenant { .. } => TenantCreateApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => TenantCreateApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyOperatorTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathTenantId
            | Self::TenantPathBodyMismatch { .. }
            | Self::InvalidResidencyClass { .. }
            | Self::Tenant(_) => TenantCreateApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyOperatorTenantHeader => "X-Tenant-Id operator header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathTenantId => "Path tenant id is required",
            Self::TenantPathBodyMismatch { .. } => {
                "Path tenant id must match request body tenant_id"
            }
            Self::AuthorizationDenied { .. } => {
                "The verified caller is not authorized for the requested tenant creation"
            }
            Self::AuthorizationFault { .. } => {
                "The authorization decision point is unavailable; request denied (fail-closed)"
            }
            Self::InvalidResidencyClass { .. } => {
                "Request residency_class must be a supported residency class label"
            }
            Self::DuplicateTenant { .. } => "Tenant id already exists",
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::Tenant(error) => tenant_error_message(error),
        }
    }

    fn details(&self) -> Vec<TenantCreateApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyOperatorTenantHeader => {
                vec![detail("header.X-Tenant-Id", "must be non-empty")]
            }
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathTenantId => vec![detail("path.tenant_id", "must be non-empty")],
            Self::TenantPathBodyMismatch { .. } => vec![detail(
                "body.tenant_id",
                "must match the tenant_id path parameter",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization.decision",
                "the server-side policy decision point denied tenant.create for the verified caller",
            )],
            Self::AuthorizationFault { .. } => vec![detail(
                "authorization.decision",
                "the server-side policy decision point faulted; request denied fail-closed",
            )],
            Self::InvalidResidencyClass { .. } => vec![detail(
                "body.residency_class",
                "must be one of strict_home_region, home_with_recovery_failover, or global",
            )],
            Self::DuplicateTenant { .. } => {
                vec![detail("body.tenant_id", "must be globally unique")]
            }
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::Tenant(error) => vec![detail("tenant_kernel", tenant_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TenantCreateApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

/// Validate the request shape (boundary headers, path/body binding, principal
/// presence, residency class) WITHOUT deciding authorization. Authorization is a
/// server-side PDP decision (see [`create_tenant_from_api`]); a pure shape check
/// must not be mistaken for an authorization. Kept public for callers that want
/// to surface a 400 before reaching the PDP.
pub fn validate_tenant_create_request(
    request: &TenantCreateApiRequest,
) -> Result<(), TenantCreateApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_body_binding(&request.path_tenant_id, &request.body.tenant_id)?;
    validate_principal_present(&request.principal)?;
    parse_api_residency_class(&request.body.residency_class)?;
    Ok(())
}

/// Create a tenant, authorizing the VERIFIED caller against the TARGET tenant via
/// the SERVER-SIDE `authorizer` PDP (fail-closed, default-deny). No path reaches
/// the directory mutation without an `Ok(true)` from the PDP: a deny (`Ok(false)`)
/// or any adapter fault (`Err`) returns a 403 before any state changes.
///
/// The TARGET tenant the PDP authorizes is `request.path_tenant_id` (a trusted,
/// path-derived source — verified equal to `body.tenant_id` first), held on a
/// SEPARATE axis from the verified caller's own tenant so a cross-tenant create
/// is deniable at the PDP (true blast radius / no IDOR).
pub fn create_tenant_from_api(
    directory: &mut TenantDirectory,
    idempotency_ledger: &mut TenantCreateIdempotencyLedger,
    authorizer: &dyn TenantCreateAuthorizer,
    request: TenantCreateApiRequest,
) -> Result<TenantCreateSuccessResponse, TenantCreateApiError> {
    validate_tenant_create_request(&request)?;
    // SERVER-SIDE authorization: ask the PDP, fail-closed. The decision is made
    // HERE from trusted inputs (verified principal + path-derived target), never
    // read from a caller-supplied grant.
    authorize_tenant_create(authorizer, &request)?;
    let key = idempotency_key_for(&request.boundary, &request.principal, TENANT_CREATE_SURFACE);
    let fingerprint = tenant_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(TenantCreateApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }
    if directory.tenants.contains_key(&request.body.tenant_id) {
        return Err(TenantCreateApiError::DuplicateTenant {
            tenant_id: request.body.tenant_id,
        });
    }

    let tenant = tenant_from_request(&request.body)?;
    let response = TenantCreateSuccessResponse::created(tenant_record(&tenant), &request);
    directory.tenants.insert(tenant.id.clone(), tenant);
    idempotency_ledger.entries.insert(
        key,
        TenantCreateIdempotencyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(boundary: &TenantApiBoundaryContext) -> Result<(), TenantCreateApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyOperatorTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_body_binding(
    path_tenant_id: &str,
    body_tenant_id: &str,
) -> Result<(), TenantCreateApiError> {
    if path_tenant_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyPathTenantId);
    }
    if path_tenant_id != body_tenant_id {
        return Err(TenantCreateApiError::TenantPathBodyMismatch {
            path_tenant_id: path_tenant_id.to_string(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

/// A verified principal must carry a non-empty identity. The principal is
/// UNFORGEABLE (only a verifier mints it), so an empty identity means a
/// misconfigured verifier rather than an authn bypass — surface it as a 401 so
/// the boundary refuses to act on a principal with no subject.
fn validate_principal_present(
    principal: &VerifiedTenantPrincipal,
) -> Result<(), TenantCreateApiError> {
    if principal.caller_principal_id().trim().is_empty() {
        return Err(TenantCreateApiError::EmptyPrincipalId);
    }
    Ok(())
}

/// Ask the SERVER-SIDE PDP whether the verified caller may create the target
/// tenant. Fail-closed: an explicit deny (`Ok(false)`) and any adapter fault
/// (`Err`) BOTH map to a 403 — only `Ok(true)` permits. The TARGET tenant is the
/// path-derived `path_tenant_id` (trusted), bound on a separate axis from the
/// caller's own tenant so the PDP can deny a cross-tenant create.
fn authorize_tenant_create(
    authorizer: &dyn TenantCreateAuthorizer,
    request: &TenantCreateApiRequest,
) -> Result<(), TenantCreateApiError> {
    let authz_request = TenantCreateAuthzRequest {
        caller_tenant_id: request.principal.caller_tenant_id(),
        caller_principal_id: request.principal.caller_principal_id(),
        target_tenant_id: &request.path_tenant_id,
        surface: TENANT_CREATE_SURFACE,
    };
    match authorizer.decide(&authz_request) {
        Ok(true) => Ok(()),
        Ok(false) => Err(TenantCreateApiError::AuthorizationDenied {
            surface: TENANT_CREATE_SURFACE.to_string(),
        }),
        Err(fault) => Err(TenantCreateApiError::AuthorizationFault {
            detail: fault.detail().to_string(),
        }),
    }
}

fn tenant_from_request(body: &TenantCreateRequest) -> Result<Tenant, TenantCreateApiError> {
    Tenant::new(
        body.tenant_id.clone(),
        body.legal_name.clone(),
        body.home_region.clone(),
        parse_api_residency_class(&body.residency_class)?,
        body.regulatory_packs
            .iter()
            .map(|pack| pack.value.clone())
            .collect(),
    )
    .map_err(TenantCreateApiError::Tenant)
}

fn parse_api_residency_class(
    label: &str,
) -> Result<network_residency::ResidencyClass, TenantCreateApiError> {
    parse_residency_class_label(label).ok_or(TenantCreateApiError::InvalidResidencyClass {
        residency_class: label.to_string(),
    })
}

fn idempotency_key_for(
    boundary: &TenantApiBoundaryContext,
    principal: &VerifiedTenantPrincipal,
    surface: &str,
) -> TenantCreateIdempotencyLedgerKey {
    // Key the idempotency ledger on the VERIFIED caller (tenant + principal),
    // never on a caller-supplied header/grant, so a replay is scoped to the
    // authenticated identity that issued the original request.
    TenantCreateIdempotencyLedgerKey {
        operator_tenant_id: principal.caller_tenant_id().to_owned(),
        principal_id: principal.caller_principal_id().to_owned(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn tenant_create_fingerprint_for(
    request: &TenantCreateApiRequest,
) -> TenantCreateRequestFingerprint {
    TenantCreateRequestFingerprint {
        canonical: [
            format!("path.tenant_id={}", request.path_tenant_id),
            format!("header.operator_tenant_id={}", request.boundary.tenant_id),
            // VERIFIED caller axes (not caller-supplied authorization fields).
            format!(
                "principal.caller_tenant_id={}",
                request.principal.caller_tenant_id()
            ),
            format!(
                "principal.caller_principal_id={}",
                request.principal.caller_principal_id()
            ),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.legal_name={}", request.body.legal_name),
            format!("body.home_region={}", request.body.home_region),
            format!("body.residency_class={}", request.body.residency_class),
            format!(
                "body.regulatory_packs={}",
                request
                    .body
                    .regulatory_packs
                    .iter()
                    .map(|pack| pack.value.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ]
        .join("|"),
    }
}

fn tenant_record(tenant: &Tenant) -> TenantRecord {
    TenantRecord {
        tenant_id: tenant.id.clone(),
        legal_name: tenant.legal_name.value.clone(),
        home_region: tenant.home_region.value.clone(),
        residency_class: tenant
            .residency_class
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        regulatory_packs: tenant
            .regulatory_packs
            .value
            .iter()
            .cloned()
            .map(|value| TenantRegulatoryPackRef { value })
            .collect(),
        schema_version: 1,
    }
}

fn tenant_error_code(error: &TenantError) -> TenantCreateApiErrorCode {
    match error {
        TenantError::InvalidTenantId => TenantCreateApiErrorCode::TenantInvalidTenantId,
        TenantError::EmptyLegalName => TenantCreateApiErrorCode::TenantLegalNameEmpty,
        TenantError::EmptyHomeRegion => TenantCreateApiErrorCode::TenantHomeRegionEmpty,
        TenantError::HomeRegionNotAllowedForResidency => {
            TenantCreateApiErrorCode::TenantHomeRegionDenied
        }
        TenantError::MissingRegionalPack => TenantCreateApiErrorCode::TenantRegionalPackMissing,
    }
}

fn tenant_error_message(error: &TenantError) -> &'static str {
    match error {
        TenantError::InvalidTenantId => "Tenant id must use the ten_ prefix",
        TenantError::EmptyLegalName => "Tenant legal name is required",
        TenantError::EmptyHomeRegion => "Tenant home region is required",
        TenantError::HomeRegionNotAllowedForResidency => {
            "Tenant home region is not allowed for the requested residency class"
        }
        TenantError::MissingRegionalPack => "At least one regulatory pack is required",
    }
}

fn tenant_error_issue(error: &TenantError) -> &'static str {
    match error {
        TenantError::InvalidTenantId => "tenant id must be globally canonical and ten_-prefixed",
        TenantError::EmptyLegalName => "legal_name must be non-empty",
        TenantError::EmptyHomeRegion => "home_region must be non-empty",
        TenantError::HomeRegionNotAllowedForResidency => {
            "strict home-region residency classes require a kr-* home region"
        }
        TenantError::MissingRegionalPack => "regulatory_packs must contain at least one pack",
    }
}

fn detail(field: impl Into<String>, issue: impl Into<String>) -> TenantCreateApiErrorDetail {
    TenantCreateApiErrorDetail {
        field: field.into(),
        issue: issue.into(),
    }
}

// ============================================================
// Reference credential verifier (the unforgeable-token minter)
// ============================================================

/// Constant-time byte comparison (no early-out on first mismatch): never leak a
/// token's length or content through timing. Mirrors the established
/// constant-time doctrine — do NOT hand-roll a naive `==` for a credential.
#[must_use]
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();
    for index in 0..max_len {
        let left = a.get(index).copied().unwrap_or(0);
        let right = b.get(index).copied().unwrap_or(0);
        diff |= (left ^ right) as usize;
    }
    diff == 0
}

/// Caller-authentication PORT for the tenant-create boundary: derive a
/// [`VerifiedTenantPrincipal`] from an UNFORGEABLE credential. Returns `None`
/// when no valid credential is present (the boundary maps that to a 401,
/// default-deny). Caller-supplied `x-principal-*`/`x-authorization-*` fields MUST
/// NOT authorize — only a verified credential mints a principal. A production
/// adapter (mTLS/SPIFFE peer identity, or a cloud-iam credential-store client)
/// implements this OUTSIDE the boundary; the reference
/// [`BearerTenantPrincipalVerifier`] below is the W5-shaped seed used by tests
/// and single-node bring-up.
pub trait TenantPrincipalVerifier {
    /// Verify a presented bearer credential; `None` ⇒ no verified principal.
    fn verify(&self, presented_bearer: Option<&str>) -> Option<VerifiedTenantPrincipal>;
}

/// Reference [`TenantPrincipalVerifier`]: one configured bearer token bound to a
/// single caller identity + tenant, compared in constant time. This is the ONLY
/// place outside the type itself that mints a [`VerifiedTenantPrincipal`] (via
/// the `pub(crate)` constructor), so a downstream crate cannot forge one. An
/// empty/unset configured token verifies NOTHING (every caller is unauthenticated):
/// there is no allow-all path.
#[derive(Clone, Debug)]
pub struct BearerTenantPrincipalVerifier {
    token: String,               // data_class: SECRET
    caller_tenant_id: String,    // data_class: INTERNAL_ONLY
    caller_principal_id: String, // data_class: INTERNAL_ONLY
}

impl BearerTenantPrincipalVerifier {
    /// Build a verifier for one configured bearer credential. The bound caller
    /// identity + tenant are what a successful verify returns (never a header).
    #[must_use]
    pub fn new(
        token: impl Into<String>,
        caller_tenant_id: impl Into<String>,
        caller_principal_id: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            caller_tenant_id: caller_tenant_id.into(),
            caller_principal_id: caller_principal_id.into(),
        }
    }
}

impl TenantPrincipalVerifier for BearerTenantPrincipalVerifier {
    fn verify(&self, presented_bearer: Option<&str>) -> Option<VerifiedTenantPrincipal> {
        // An unset configured token authenticates no one (no allow-all).
        if self.token.is_empty() {
            return None;
        }
        let presented = presented_bearer?;
        if constant_time_eq(presented.as_bytes(), self.token.as_bytes()) {
            // Mint the unforgeable principal ONLY after the credential verified.
            Some(VerifiedTenantPrincipal::new(
                self.caller_tenant_id.clone(),
                self.caller_principal_id.clone(),
            ))
        } else {
            None
        }
    }
}
