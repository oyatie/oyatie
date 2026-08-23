//! Cloud Observability audit read API boundary.
//!
//! This crate owns tenant/header/body normalization, authentication and
//! authorization evidence checks, fail-closed label parsing, and typed REST
//! projection for the `cloud.observability.audit.read` surface before handing
//! immutable reads to the Cloud observability kernel.

use audit_chain_domain::Plane;
use observability_aggregate::{
    AuditReadRequest, AuditReadScope, CloudAuditOperation, CloudAuditRecord, CloudAuditTopic,
    CloudObservabilityCatalog, CloudObservabilityError,
};
use data_boundary_kernel::{OperationalDataClass, Purpose};

pub mod authz;

pub use authz::{
    AuditReadAction, AuditReadAuthorizationError, AuditReadAuthorizer, AuditReadAuthzProvider,
    AuditReadResource, AuthzProviderConfigError, CallerCredential,
    ConfiguredBearerPrincipalVerifier, PrincipalVerificationError, PrincipalVerifier,
    VerifiedPrincipal, constant_time_eq,
};

pub const CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE: &str = "cloud.observability.audit.read";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudObservabilityAuditReadApiStatus {
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

impl CloudObservabilityAuditReadApiStatus {
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
pub enum CloudObservabilityApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    PrincipalMissing,
    PrincipalIdEmpty,
    TenantMismatch,
    PrincipalUnverified,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    AuditScopeInvalid,
    AuditTopicInvalid,
    ObservabilityInvalidRequest,
    ObservabilityForbidden,
    ObservabilityUnprocessable,
}

impl CloudObservabilityApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_OBSERVABILITY_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_OBSERVABILITY_TENANT_HEADER_EMPTY",
            Self::PrincipalMissing => "CLOUD_OBSERVABILITY_PRINCIPAL_MISSING",
            Self::PrincipalIdEmpty => "CLOUD_OBSERVABILITY_PRINCIPAL_ID_EMPTY",
            Self::TenantMismatch => "CLOUD_OBSERVABILITY_TENANT_MISMATCH",
            Self::PrincipalUnverified => "CLOUD_OBSERVABILITY_PRINCIPAL_UNVERIFIED",
            Self::AuthorizationTenantMismatch => {
                "CLOUD_OBSERVABILITY_AUTHORIZATION_TENANT_MISMATCH"
            }
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_OBSERVABILITY_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_OBSERVABILITY_AUTHORIZATION_DENIED",
            Self::AuditScopeInvalid => "CLOUD_OBSERVABILITY_AUDIT_SCOPE_INVALID",
            Self::AuditTopicInvalid => "CLOUD_OBSERVABILITY_AUDIT_TOPIC_INVALID",
            Self::ObservabilityInvalidRequest => "CLOUD_OBSERVABILITY_INVALID_REQUEST",
            Self::ObservabilityForbidden => "CLOUD_OBSERVABILITY_FORBIDDEN",
            Self::ObservabilityUnprocessable => "CLOUD_OBSERVABILITY_UNPROCESSABLE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiBoundaryContext {
    pub request_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

/// NON-AUTHORITATIVE caller correlation metadata.
///
/// ## ⚠ This is NOT an authorization grant (C18 / ADR-0590)
///
/// Before ADR-0590 the boundary trusted the `allowed_surfaces` list on this DTO
/// as the authorization decision: a caller who set
/// `allowed_surfaces = ["cloud.observability.audit.read"]` was "authorized".
/// That was **self-granting evidence** — the caller authored the very decision
/// meant to authorize them.  The `allowed_surfaces` field is REMOVED.
///
/// The remaining fields are a caller-supplied **correlation id only** for tracing
/// — they confer NO authority.  Authorization is decided server-side by the
/// [`authz::AuditReadAuthorizer`] PDP port against the
/// [`authz::VerifiedPrincipal`].  The `tenant_id` / `principal_id` here are
/// cross-checked against the verified principal and rejected on mismatch; they
/// never grant.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudObservabilityApiAuthorization {
    pub tenant_id: String, // data_class: INTERNAL_ONLY — cross-check only, never a grant
    pub principal_id: String, // data_class: INTERNAL_ONLY — cross-check only, never a grant
    pub correlation_id: String, // data_class: INTERNAL_ONLY — non-authoritative trace id
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadTopicRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadRequest {
    pub tenant_id: String,        // data_class: INTERNAL_ONLY
    pub region: String,           // data_class: PUBLIC
    pub cell_id: Option<String>,  // data_class: PUBLIC
    pub scope: String,            // data_class: INTERNAL_ONLY
    pub start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub topics: Vec<CloudObservabilityAuditReadTopicRef>, // data_class: INTERNAL_ONLY
    pub actor: Option<String>,    // data_class: INTERNAL_ONLY
    pub resource_id: Option<String>, // data_class: INTERNAL_ONLY
    pub cursor: Option<String>,   // data_class: INTERNAL_ONLY
    pub page_size: Option<u16>,   // data_class: INTERNAL_ONLY
    pub require_complete_chain: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadApiRequest {
    pub boundary: CloudObservabilityApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: Option<CloudObservabilityApiPrincipal>, // data_class: INTERNAL_ONLY
    pub authorization: CloudObservabilityApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudObservabilityAuditReadRequest,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadSuccessResponse {
    pub data: Vec<CloudObservabilityAuditRecord>, // data_class: INTERNAL_ONLY
    pub metadata: CloudObservabilityAuditReadMetadata, // data_class: INTERNAL_ONLY
}

impl CloudObservabilityAuditReadSuccessResponse {
    pub fn ok(
        data: Vec<CloudObservabilityAuditRecord>,
        metadata: CloudObservabilityAuditReadMetadata,
    ) -> Self {
        Self { data, metadata }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadMetadata {
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: PUBLIC
    pub record_count: u32,                    // data_class: INTERNAL_ONLY
    pub next_cursor: Option<String>,          // data_class: INTERNAL_ONLY
    pub chain_complete: bool,                 // data_class: INTERNAL_ONLY
    pub high_watermark_sequence: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditRecord {
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: PUBLIC
    pub cell_id: Option<String>,            // data_class: PUBLIC
    pub topic: String,                      // data_class: INTERNAL_ONLY
    pub operation: String,                  // data_class: INTERNAL_ONLY
    pub record_class: String,               // data_class: INTERNAL_ONLY
    pub source_resource_id: Option<String>, // data_class: INTERNAL_ONLY
    pub actor: String,                      // data_class: INTERNAL_ONLY
    pub iam_role: Option<String>,           // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub chain_sequence: u64,                // data_class: INTERNAL_ONLY
    pub previous_hash: String,              // data_class: INTERNAL_ONLY
    pub hash: String,                       // data_class: INTERNAL_ONLY
    pub payload_hash: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub decision: String,                   // data_class: INTERNAL_ONLY
    pub purpose: String,                    // data_class: INTERNAL_ONLY
    pub plane: String,                      // data_class: INTERNAL_ONLY
    pub data_classes_referenced: Vec<CloudObservabilityDataClassRef>, // data_class: INTERNAL_ONLY
    pub signed_export_uri: String,          // data_class: INTERNAL_ONLY
    pub audit_marker: String,               // data_class: INTERNAL_ONLY
    pub schema_version: u32,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityDataClassRef {
    pub label: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiErrorResponse {
    pub error: CloudObservabilityApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<CloudObservabilityApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudObservabilityApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    MissingPrincipal,
    EmptyPrincipalId,
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: Option<String>,
        body_tenant_id: String,
    },
    /// The presented credential did not verify into a real principal (401).
    PrincipalUnverified,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        principal_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    /// The server-side PDP denied (or refused, fail-closed) the audit read for
    /// the verified principal on the scope-derived action (403).
    AuthorizationDenied {
        action: String,
    },
    InvalidAuditScopeLabel {
        scope: String,
    },
    InvalidAuditTopicLabel {
        topic: String,
    },
    Observability(CloudObservabilityError),
}

impl CloudObservabilityApiError {
    pub fn audit_read_status(&self) -> CloudObservabilityAuditReadApiStatus {
        match self.status_kind() {
            CloudObservabilityApiStatusKind::BadRequest => {
                CloudObservabilityAuditReadApiStatus::BadRequest
            }
            CloudObservabilityApiStatusKind::Unauthorized => {
                CloudObservabilityAuditReadApiStatus::Unauthorized
            }
            CloudObservabilityApiStatusKind::Forbidden => {
                CloudObservabilityAuditReadApiStatus::Forbidden
            }
            CloudObservabilityApiStatusKind::UnprocessableEntity => {
                CloudObservabilityAuditReadApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn status_code(&self) -> u16 {
        self.audit_read_status().code()
    }

    pub fn code(&self) -> CloudObservabilityApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudObservabilityApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudObservabilityApiErrorCode::TenantHeaderEmpty,
            Self::MissingPrincipal => CloudObservabilityApiErrorCode::PrincipalMissing,
            Self::EmptyPrincipalId => CloudObservabilityApiErrorCode::PrincipalIdEmpty,
            Self::TenantMismatch { .. } => CloudObservabilityApiErrorCode::TenantMismatch,
            Self::PrincipalUnverified => CloudObservabilityApiErrorCode::PrincipalUnverified,
            Self::AuthorizationTenantMismatch { .. } => {
                CloudObservabilityApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudObservabilityApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudObservabilityApiErrorCode::AuthorizationDenied,
            Self::InvalidAuditScopeLabel { .. } => {
                CloudObservabilityApiErrorCode::AuditScopeInvalid
            }
            Self::InvalidAuditTopicLabel { .. } => {
                CloudObservabilityApiErrorCode::AuditTopicInvalid
            }
            Self::Observability(error) => match observability_status_kind(error) {
                CloudObservabilityApiStatusKind::BadRequest => {
                    CloudObservabilityApiErrorCode::ObservabilityInvalidRequest
                }
                CloudObservabilityApiStatusKind::Forbidden => {
                    CloudObservabilityApiErrorCode::ObservabilityForbidden
                }
                CloudObservabilityApiStatusKind::UnprocessableEntity => {
                    CloudObservabilityApiErrorCode::ObservabilityUnprocessable
                }
                CloudObservabilityApiStatusKind::Unauthorized => {
                    CloudObservabilityApiErrorCode::ObservabilityInvalidRequest
                }
            },
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> CloudObservabilityApiErrorResponse {
        CloudObservabilityApiErrorResponse {
            error: CloudObservabilityApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudObservabilityApiStatusKind {
        match self {
            Self::MissingPrincipal | Self::EmptyPrincipalId | Self::PrincipalUnverified => {
                CloudObservabilityApiStatusKind::Unauthorized
            }
            Self::TenantMismatch { .. }
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudObservabilityApiStatusKind::Forbidden,
            Self::Observability(error) => observability_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::InvalidAuditScopeLabel { .. }
            | Self::InvalidAuditTopicLabel { .. } => CloudObservabilityApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::MissingPrincipal => "Authenticated principal evidence is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::PrincipalUnverified => "Presented credential did not verify a principal",
            Self::AuthorizationTenantMismatch { .. } => {
                "Request tenant must match the verified principal tenant"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Request principal must match the verified principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Server-side authorization denied the requested Cloud Observability audit read"
            }
            Self::InvalidAuditScopeLabel { .. } => "Audit read scope label is not supported",
            Self::InvalidAuditTopicLabel { .. } => "Audit topic label is not supported",
            Self::Observability(error) => observability_message(error),
        }
    }

    fn details(&self) -> Vec<CloudObservabilityApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::MissingPrincipal => vec![detail("principal", "must be present")],
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and request body tenant must match",
            )],
            Self::PrincipalUnverified => vec![detail(
                "credential",
                "presented credential did not verify a principal",
            )],
            Self::AuthorizationTenantMismatch { .. } => vec![detail(
                "principal.tenant_id",
                "must match the verified principal tenant",
            )],
            Self::AuthorizationPrincipalMismatch { .. } => vec![detail(
                "principal.principal_id",
                "must match the verified principal id",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization",
                "server-side PDP denied the audit read for the verified principal",
            )],
            Self::InvalidAuditScopeLabel { .. } => vec![detail(
                "body.scope",
                "must be control_plane_mutations or all_tenant_audit",
            )],
            Self::InvalidAuditTopicLabel { .. } => {
                vec![detail(
                    "body.topics",
                    "must use a supported audit topic label",
                )]
            }
            Self::Observability(error) => {
                vec![detail("cloud_observability", observability_issue(error))]
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudObservabilityApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

/// Serve an immutable Cloud Observability audit read, fail-closed.
///
/// ## Authorization contract (C18 / ADR-0590)
///
/// The first argument is an [`authz::VerifiedPrincipal`] — a token whose private
/// fields and `pub(crate)` constructor mean it can ONLY be produced by running a
/// real [`authz::PrincipalVerifier`] inside this crate.  The boundary therefore
/// cannot be reached without a verified credential; an absent/forged credential
/// never mints a `VerifiedPrincipal` (the caller maps the verifier's 401-class
/// refusal before calling this fn).
///
/// The `authorizer` is the server-side PDP [`authz::AuditReadAuthorizer`] port.
/// There is NO default-allow path: the boundary takes the authorizer by
/// reference and ALWAYS calls `ensure_authorized` before any catalog read.  The
/// decision is bound to the scope-derived [`authz::AuditReadAction`] (so the
/// broader `all_tenant_audit` scope requires its own grant — the coarse-scope
/// fix) and to the TARGET tenant taken from the verified principal (so a
/// cross-tenant read is deniable at the PDP — the blast-radius binding).
///
/// The self-attested `request.authorization` / `request.principal` fields NEVER
/// authorize: they are cross-checked against the verified principal and rejected
/// on mismatch.  A request body tenant that differs from the verified tenant is a
/// `TenantMismatch` (403); a self-attested principal/tenant that differs from the
/// verified identity is an `AuthorizationPrincipalMismatch` /
/// `AuthorizationTenantMismatch` (403).
pub fn read_cloud_observability_audit_from_api(
    verified: &authz::VerifiedPrincipal,
    authorizer: &dyn authz::AuditReadAuthorizer,
    catalog: &CloudObservabilityCatalog,
    request: CloudObservabilityAuditReadApiRequest,
) -> Result<CloudObservabilityAuditReadSuccessResponse, CloudObservabilityApiError> {
    validate_boundary(&request.boundary)?;
    // The self-attested principal DTO is optional and NON-AUTHORITATIVE. When
    // present it must agree with the verified identity; when absent the verified
    // principal alone is authoritative. The verified principal is ALWAYS the
    // source of truth — never the DTO.
    if let Some(principal) = request.principal.as_ref() {
        if principal.principal_id.trim().is_empty() {
            return Err(CloudObservabilityApiError::EmptyPrincipalId);
        }
        cross_check_verified_principal(verified, principal)?;
    }
    // The verified tenant is the TRUSTED tenant axis. Bind the request body to
    // it; a body tenant that differs is a cross-tenant attempt (403). The header
    // tenant is likewise bound for defense in depth.
    bind_tenant_to_verified(verified, &request.boundary, &request.body.tenant_id)?;
    // Cross-check the non-authoritative correlation DTO when populated.
    cross_check_authorization_correlation(verified, &request.authorization)?;

    // Derive the action from the requested scope so the broader all-tenant-audit
    // scope requires strictly more authority than control-plane reads.
    let scope = parse_audit_scope(&request.body.scope)?;
    let action = audit_read_action(scope);
    let resource = authz::AuditReadResource {
        tenant_id: verified.tenant_id().to_string(),
        region: request.body.region.clone(),
        action,
        request_hash: audit_read_request_hash(verified, action, &request.body),
    };
    // SERVER-SIDE PDP decision. Fail-closed: deny and refuse both map to 403.
    authorizer
        .ensure_authorized(verified, &resource)
        .map_err(|_| CloudObservabilityApiError::AuthorizationDenied {
            action: action.as_str().to_string(),
        })?;

    let request_id = request.boundary.request_id.clone();
    // Project the metadata tenant from the VERIFIED principal, not the caller DTO.
    let tenant_id = verified.tenant_id().to_string();
    let region = request.body.region.clone();
    // Force the kernel read tenant to the verified tenant so the served data is
    // scoped to the authorized tenant regardless of any residual DTO value.
    let mut body = request.body;
    body.tenant_id = verified.tenant_id().to_string();
    let kernel_request = audit_read_request(body)?;
    let result = catalog
        .read_audit(kernel_request)
        .map_err(CloudObservabilityApiError::Observability)?;
    let data = result.records.iter().map(audit_record).collect::<Vec<_>>();
    let record_count = data.len() as u32;
    Ok(CloudObservabilityAuditReadSuccessResponse::ok(
        data,
        CloudObservabilityAuditReadMetadata {
            request_id,
            tenant_id,
            region,
            record_count,
            next_cursor: result.next_cursor.map(|cursor| cursor.value),
            chain_complete: result.chain_complete,
            high_watermark_sequence: result.high_watermark_sequence,
        },
    ))
}

/// Map an audit-read scope to its PDP action. The all-tenant scope exposes
/// data-plane-security / KMS-use / billing audit and therefore requires its OWN,
/// strictly-more-privileged action (the C18 coarse-scope fix).
const fn audit_read_action(scope: AuditReadScope) -> authz::AuditReadAction {
    match scope {
        AuditReadScope::ControlPlaneMutations => authz::AuditReadAction::ControlPlaneAuditRead,
        AuditReadScope::AllTenantAudit => authz::AuditReadAction::AllTenantAuditRead,
    }
}

/// A stable, collision-resistant-by-construction hash of the authorized request
/// shape. Binds the PDP decision to THIS request so it cannot be replayed against
/// a different body. Uses a length-prefixed field encoding (no separator
/// injection) hashed with FNV-1a — sufficient as a binding token (NOT a security
/// MAC; the decision itself is the authority).
fn audit_read_request_hash(
    verified: &authz::VerifiedPrincipal,
    action: authz::AuditReadAction,
    body: &CloudObservabilityAuditReadRequest,
) -> String {
    let mut hasher = Fnv1a::new();
    hasher.field(CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE.as_bytes());
    hasher.field(action.as_str().as_bytes());
    hasher.field(verified.principal_id().as_bytes());
    hasher.field(verified.tenant_id().as_bytes());
    hasher.field(body.region.as_bytes());
    hasher.field(body.scope.trim().as_bytes());
    let mut topics = body
        .topics
        .iter()
        .map(|topic| topic.value.trim().to_string())
        .collect::<Vec<_>>();
    topics.sort();
    for topic in &topics {
        hasher.field(topic.as_bytes());
    }
    format!("h:{:016x}", hasher.finish())
}

/// Minimal FNV-1a 64-bit hasher with length-prefixed field framing so distinct
/// field boundaries cannot be confused by concatenation.
struct Fnv1a {
    state: u64,
}

impl Fnv1a {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self {
            state: Self::OFFSET,
        }
    }

    fn byte(&mut self, b: u8) {
        self.state ^= u64::from(b);
        self.state = self.state.wrapping_mul(Self::PRIME);
    }

    fn field(&mut self, bytes: &[u8]) {
        for b in (bytes.len() as u64).to_le_bytes() {
            self.byte(b);
        }
        for &b in bytes {
            self.byte(b);
        }
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

/// Cross-check the self-attested principal DTO against the verified identity. A
/// mismatch means the caller substituted a different identity than the verifier
/// bound — reject as Forbidden (never trust the DTO over the credential).
fn cross_check_verified_principal(
    verified: &authz::VerifiedPrincipal,
    principal: &CloudObservabilityApiPrincipal,
) -> Result<(), CloudObservabilityApiError> {
    if principal.principal_id != verified.principal_id() {
        return Err(CloudObservabilityApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: verified.principal_id().to_string(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if principal.tenant_id != verified.tenant_id() {
        return Err(CloudObservabilityApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: verified.tenant_id().to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    Ok(())
}

/// Bind the header tenant and the request body tenant to the VERIFIED tenant. A
/// body or header tenant that differs from the verified tenant is a cross-tenant
/// attempt (the blast-radius binding) and is rejected.
fn bind_tenant_to_verified(
    verified: &authz::VerifiedPrincipal,
    boundary: &CloudObservabilityApiBoundaryContext,
    body_tenant_id: &str,
) -> Result<(), CloudObservabilityApiError> {
    if boundary.tenant_id != verified.tenant_id() || body_tenant_id != verified.tenant_id() {
        return Err(CloudObservabilityApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: Some(verified.tenant_id().to_string()),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

/// Cross-check the NON-AUTHORITATIVE correlation DTO. Its tenant/principal fields,
/// when populated, must agree with the verified identity; they NEVER grant.
fn cross_check_authorization_correlation(
    verified: &authz::VerifiedPrincipal,
    authorization: &CloudObservabilityApiAuthorization,
) -> Result<(), CloudObservabilityApiError> {
    if !authorization.tenant_id.is_empty() && authorization.tenant_id != verified.tenant_id() {
        return Err(CloudObservabilityApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: verified.tenant_id().to_string(),
        });
    }
    if !authorization.principal_id.is_empty()
        && authorization.principal_id != verified.principal_id()
    {
        return Err(CloudObservabilityApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: verified.principal_id().to_string(),
        });
    }
    Ok(())
}

fn validate_boundary(
    boundary: &CloudObservabilityApiBoundaryContext,
) -> Result<(), CloudObservabilityApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudObservabilityApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudObservabilityApiError::EmptyTenantHeader);
    }
    Ok(())
}

fn audit_read_request(
    input: CloudObservabilityAuditReadRequest,
) -> Result<AuditReadRequest, CloudObservabilityApiError> {
    Ok(AuditReadRequest {
        tenant_id: input.tenant_id,
        region: input.region,
        cell_id: input.cell_id,
        scope: parse_audit_scope(&input.scope)?,
        start_epoch_seconds: input.start_epoch_seconds,
        end_epoch_seconds: input.end_epoch_seconds,
        topics: input
            .topics
            .into_iter()
            .map(|topic| parse_audit_topic(&topic.value))
            .collect::<Result<Vec<_>, _>>()?,
        actor: input.actor,
        resource_id: input.resource_id,
        cursor: input.cursor,
        page_size: input.page_size,
        require_complete_chain: input.require_complete_chain,
    })
}

fn parse_audit_scope(value: &str) -> Result<AuditReadScope, CloudObservabilityApiError> {
    match value.trim() {
        "control_plane_mutations" => Ok(AuditReadScope::ControlPlaneMutations),
        "all_tenant_audit" => Ok(AuditReadScope::AllTenantAudit),
        _ => Err(CloudObservabilityApiError::InvalidAuditScopeLabel {
            scope: value.to_string(),
        }),
    }
}

fn parse_audit_topic(value: &str) -> Result<CloudAuditTopic, CloudObservabilityApiError> {
    match value.trim() {
        "oyatie.audit.cloud_resource_created" => Ok(CloudAuditTopic::CloudResourceCreated),
        "oyatie.audit.cloud_resource_terminated" => Ok(CloudAuditTopic::CloudResourceTerminated),
        "oyatie.audit.cloud_iam_assume" => Ok(CloudAuditTopic::CloudIamAssume),
        "oyatie.audit.cloud_iam_policy" => Ok(CloudAuditTopic::CloudIamPolicy),
        "oyatie.audit.cloud_region_register" => Ok(CloudAuditTopic::CloudRegionRegister),
        "oyatie.audit.cloud_kms_use" => Ok(CloudAuditTopic::CloudKmsUse),
        "oyatie.audit.cloud_replication" => Ok(CloudAuditTopic::CloudReplication),
        "oyatie.audit.cloud_flow_anomaly" => Ok(CloudAuditTopic::CloudFlowAnomaly),
        "oyatie.audit.cloud_invoice" => Ok(CloudAuditTopic::CloudInvoice),
        "oyatie.audit.cloud_interconnect" => Ok(CloudAuditTopic::CloudInterconnect),
        "oyatie.audit.cloud_cell_rebalanced" => Ok(CloudAuditTopic::CloudCellRebalanced),
        _ => Err(CloudObservabilityApiError::InvalidAuditTopicLabel {
            topic: value.to_string(),
        }),
    }
}

fn audit_record(record: &CloudAuditRecord) -> CloudObservabilityAuditRecord {
    CloudObservabilityAuditRecord {
        id: record.id.value.value.clone(),
        tenant_id: record.tenant_id.value.clone(),
        region: record.region.value.value.clone(),
        cell_id: record
            .cell_id
            .value
            .as_ref()
            .map(|cell_id| cell_id.value.clone()),
        topic: record.topic.value.as_str().to_string(),
        operation: audit_operation_label(record.operation.value).to_string(),
        record_class: audit_record_class_label(record.record_class.value).to_string(),
        source_resource_id: record
            .source_resource_id
            .value
            .as_ref()
            .map(|resource_id| resource_id.value.clone()),
        actor: record.actor.value.value.clone(),
        iam_role: record
            .iam_role
            .value
            .as_ref()
            .map(|role| role.value.clone()),
        occurred_at_epoch_seconds: record.occurred_at_epoch_seconds.value,
        chain_sequence: record.chain_sequence.value,
        previous_hash: record.previous_hash.value.value.clone(),
        hash: record.hash.value.value.clone(),
        payload_hash: record.payload_hash.value.value.clone(),
        idempotency_key: record.idempotency_key.value.value.clone(),
        decision: record.decision.value.clone(),
        purpose: purpose_label(record.purpose.value).to_string(),
        plane: plane_label(record.plane.value).to_string(),
        data_classes_referenced: record
            .data_classes_referenced
            .value
            .iter()
            .map(|classification| CloudObservabilityDataClassRef {
                label: classification.label().to_string(),
            })
            .collect(),
        signed_export_uri: record.signed_export_uri.value.value.clone(),
        audit_marker: operational_data_class_label(record.audit_marker.value).to_string(),
        schema_version: record.schema_version.value,
    }
}

fn audit_operation_label(operation: CloudAuditOperation) -> &'static str {
    match operation {
        CloudAuditOperation::ResourceCreated => "resource_created",
        CloudAuditOperation::ResourceTerminated => "resource_terminated",
        CloudAuditOperation::IamRoleAssumed => "iam_role_assumed",
        CloudAuditOperation::IamPolicyChanged => "iam_policy_changed",
        CloudAuditOperation::RegionRegistered => "region_registered",
        CloudAuditOperation::KmsKeyUsed => "kms_key_used",
        CloudAuditOperation::CrossRegionReplication => "cross_region_replication",
        CloudAuditOperation::NetworkFlowAnomaly => "network_flow_anomaly",
        CloudAuditOperation::InvoiceIssued => "invoice_issued",
        CloudAuditOperation::DirectInterconnectProvisioned => "direct_interconnect_provisioned",
        CloudAuditOperation::CellRebalanced => "cell_rebalanced",
    }
}

fn audit_record_class_label(
    record_class: observability_aggregate::AuditRecordClass,
) -> &'static str {
    match record_class {
        observability_aggregate::AuditRecordClass::ControlPlaneMutation => "control_plane_mutation",
        observability_aggregate::AuditRecordClass::DataPlaneSecurity => "data_plane_security",
        observability_aggregate::AuditRecordClass::BillingAnalytics => "billing_analytics",
        observability_aggregate::AuditRecordClass::Replication => "replication",
        observability_aggregate::AuditRecordClass::CapacityOperations => "capacity_operations",
    }
}

fn plane_label(plane: Plane) -> &'static str {
    match plane {
        Plane::Control => "control",
        Plane::Data => "data",
        Plane::Audit => "audit",
        Plane::Analytics => "analytics",
    }
}

fn purpose_label(purpose: Purpose) -> &'static str {
    match purpose {
        Purpose::CoreService => "core_service",
        Purpose::CapabilityInvocation => "capability_invocation",
        Purpose::SearchIndex => "search_index",
        Purpose::AdsTargeting => "ads_targeting",
        Purpose::Analytics => "analytics",
        Purpose::Support => "support",
        Purpose::TenantAnalyticsFirstParty => "tenant_analytics_first_party",
        Purpose::CrossTenantAggregateAnonymous => "cross_tenant_aggregate_anonymous",
        Purpose::PersonalizationInProduct => "personalization_in_product",
        Purpose::SearchIndexPrivate => "search_index_private",
        Purpose::SearchIndexPublic => "search_index_public",
        Purpose::AdTargetingDeclared => "ad_targeting_declared",
        Purpose::AdTargetingBehavioral => "ad_targeting_behavioral",
        Purpose::ModelTrainingOya => "model_training_oya",
        Purpose::ModelTrainingThirdParty => "model_training_third_party",
    }
}

fn operational_data_class_label(data_class: OperationalDataClass) -> &'static str {
    match data_class {
        OperationalDataClass::Audit => "AUDIT",
        OperationalDataClass::Secret => "SECRET",
    }
}

fn observability_status_kind(error: &CloudObservabilityError) -> CloudObservabilityApiStatusKind {
    match error {
        CloudObservabilityError::ResourceTenantMismatch
        | CloudObservabilityError::ResourceRegionMismatch => {
            CloudObservabilityApiStatusKind::Forbidden
        }
        CloudObservabilityError::InvalidCursor
        | CloudObservabilityError::CursorTenantMismatch
        | CloudObservabilityError::CursorRegionMismatch
        | CloudObservabilityError::IncompleteAuditChain
        | CloudObservabilityError::UnverifiedAuditChain => {
            CloudObservabilityApiStatusKind::UnprocessableEntity
        }
        _ => CloudObservabilityApiStatusKind::BadRequest,
    }
}

fn observability_message(error: &CloudObservabilityError) -> &'static str {
    match observability_status_kind(error) {
        CloudObservabilityApiStatusKind::BadRequest => {
            "Cloud Observability rejected the audit read request shape"
        }
        CloudObservabilityApiStatusKind::Forbidden => {
            "Cloud Observability policy denied the audit read request"
        }
        CloudObservabilityApiStatusKind::UnprocessableEntity => {
            "Cloud Observability could not process the supplied audit cursor or chain state"
        }
        CloudObservabilityApiStatusKind::Unauthorized => {
            "Cloud Observability authentication is required"
        }
    }
}

fn observability_issue(error: &CloudObservabilityError) -> &'static str {
    match error {
        CloudObservabilityError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudObservabilityError::InvalidRegion => "region must be canonical and residency-valid",
        CloudObservabilityError::InvalidCellId => "cell_id must be canonical for the region",
        CloudObservabilityError::InvalidAuditTopic => {
            "control-plane scope may only include control-plane audit topics"
        }
        CloudObservabilityError::InvalidReadWindow => {
            "start_epoch_seconds must be before end_epoch_seconds within the maximum read window"
        }
        CloudObservabilityError::InvalidPageSize => {
            "page_size must be between 1 and the maximum audit read page size"
        }
        CloudObservabilityError::InvalidCursor => "cursor must use the audit read cursor format",
        CloudObservabilityError::CursorTenantMismatch => {
            "cursor tenant must match the request tenant"
        }
        CloudObservabilityError::CursorRegionMismatch => {
            "cursor region must match the request region"
        }
        CloudObservabilityError::InvalidActorRef => {
            "actor filter must use an accepted principal reference"
        }
        CloudObservabilityError::InvalidResourceId => {
            "resource_id filter must be a valid Cloud resource id"
        }
        CloudObservabilityError::ResourceTenantMismatch => {
            "resource_id filter must belong to the request tenant"
        }
        CloudObservabilityError::ResourceRegionMismatch => {
            "resource_id filter must belong to the request region"
        }
        CloudObservabilityError::IncompleteAuditChain => {
            "audit chain completeness is required but unavailable"
        }
        CloudObservabilityError::UnverifiedAuditChain => "audit chain verification failed",
        _ => "cloud observability invariant rejected the request",
    }
}

fn detail(field: &str, issue: &str) -> CloudObservabilityApiErrorDetail {
    CloudObservabilityApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
