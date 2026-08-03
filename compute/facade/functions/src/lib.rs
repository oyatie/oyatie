//! Cloud Compute Functions API boundary for invocation receipts.
//!
//! This crate owns request boundary normalization, authorization proof checks,
//! idempotent invoke semantics, and tenant-safe function invocation projection
//! around the Cloud compute kernel.

use std::collections::BTreeMap;

use compute_domain::{
    CloudComputeCatalog, CloudComputeError, ComputeRepo, FunctionInvocationReceipt,
    FunctionInvocationRequest,
};
use compute_resource::ResourceId;
use data_boundary_kernel::{DataClass, parse_data_class_label};

pub const CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE: &str = "cloud.compute.functions.invoke";
const DEFAULT_FUNCTIONS_INVOKE_IDEMPOTENCY_LEDGER_MAX_ENTRIES: usize = 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudComputeFunctionsInvokeApiStatus {
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudComputeFunctionsInvokeApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
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
pub enum CloudComputeFunctionsApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathFunctionIdEmpty,
    FunctionIdInvalid,
    FunctionKindMismatch,
    FunctionIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationVerifierMissing,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    PayloadDataClassInvalid,
    ComputeInvalidRequest,
    ComputeForbidden,
    ComputeNotFound,
    ComputeConflict,
}

impl CloudComputeFunctionsApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_COMPUTE_FUNCTIONS_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_COMPUTE_FUNCTIONS_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_COMPUTE_FUNCTIONS_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_COMPUTE_FUNCTIONS_PRINCIPAL_ID_EMPTY",
            Self::PathFunctionIdEmpty => "CLOUD_COMPUTE_FUNCTIONS_PATH_FUNCTION_ID_EMPTY",
            Self::FunctionIdInvalid => "CLOUD_COMPUTE_FUNCTIONS_FUNCTION_ID_INVALID",
            Self::FunctionKindMismatch => "CLOUD_COMPUTE_FUNCTIONS_FUNCTION_KIND_MISMATCH",
            Self::FunctionIdMismatch => "CLOUD_COMPUTE_FUNCTIONS_FUNCTION_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_COMPUTE_FUNCTIONS_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_COMPUTE_FUNCTIONS_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationVerifierMissing => {
                "CLOUD_COMPUTE_FUNCTIONS_AUTHORIZATION_VERIFIER_MISSING"
            }
            Self::AuthorizationTenantMismatch => {
                "CLOUD_COMPUTE_FUNCTIONS_AUTHORIZATION_TENANT_MISMATCH"
            }
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_COMPUTE_FUNCTIONS_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_COMPUTE_FUNCTIONS_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_COMPUTE_FUNCTIONS_IDEMPOTENCY_KEY_REUSED",
            Self::PayloadDataClassInvalid => "CLOUD_COMPUTE_FUNCTIONS_PAYLOAD_DATA_CLASS_INVALID",
            Self::ComputeInvalidRequest => "CLOUD_COMPUTE_FUNCTIONS_INVALID_REQUEST",
            Self::ComputeForbidden => "CLOUD_COMPUTE_FUNCTIONS_FORBIDDEN",
            Self::ComputeNotFound => "CLOUD_COMPUTE_FUNCTIONS_NOT_FOUND",
            Self::ComputeConflict => "CLOUD_COMPUTE_FUNCTIONS_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiAuthorization {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub principal_id: String,           // data_class: INTERNAL_ONLY
    pub decision_id: String,            // data_class: INTERNAL_ONLY
    pub requested_surface: String,      // data_class: INTERNAL_ONLY
    pub valid_until_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudComputeFunctionsAuthorizationDecision {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsTrustedAuthorizationDecision {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
    pub decision_id: String,  // data_class: INTERNAL_ONLY
    pub surface: String,      // data_class: INTERNAL_ONLY
    pub decision: CloudComputeFunctionsAuthorizationDecision, // data_class: INTERNAL_ONLY
    pub valid_until_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsAuthorizationVerifier {
    evaluation_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    decisions: BTreeMap<String, CloudComputeFunctionsTrustedAuthorizationDecision>, // data_class: INTERNAL_ONLY
}

impl Default for CloudComputeFunctionsAuthorizationVerifier {
    fn default() -> Self {
        Self {
            evaluation_epoch_seconds: u64::MAX,
            decisions: BTreeMap::new(),
        }
    }
}

impl CloudComputeFunctionsAuthorizationVerifier {
    pub fn new(evaluation_epoch_seconds: u64) -> Self {
        Self {
            evaluation_epoch_seconds,
            decisions: BTreeMap::new(),
        }
    }

    pub fn trust_decision(&mut self, decision: CloudComputeFunctionsTrustedAuthorizationDecision) {
        self.decisions
            .insert(decision.decision_id.clone(), decision);
    }

    pub fn with_trusted_decision(
        mut self,
        decision: CloudComputeFunctionsTrustedAuthorizationDecision,
    ) -> Self {
        self.trust_decision(decision);
        self
    }

    fn verify(
        &self,
        principal: &CloudComputeFunctionsApiPrincipal,
        decision_id: &str,
        surface: &str,
    ) -> Result<(), CloudComputeFunctionsApiError> {
        if decision_id.trim().is_empty() {
            return Err(CloudComputeFunctionsApiError::EmptyAuthorizationDecisionId);
        }
        let Some(decision) = self.decisions.get(decision_id) else {
            return Err(CloudComputeFunctionsApiError::AuthorizationDenied {
                surface: surface.to_string(),
            });
        };
        if decision.tenant_id != principal.tenant_id {
            return Err(CloudComputeFunctionsApiError::AuthorizationTenantMismatch {
                authorization_tenant_id: decision.tenant_id.clone(),
                principal_tenant_id: principal.tenant_id.clone(),
            });
        }
        if decision.principal_id != principal.principal_id {
            return Err(
                CloudComputeFunctionsApiError::AuthorizationPrincipalMismatch {
                    authorization_principal_id: decision.principal_id.clone(),
                    principal_id: principal.principal_id.clone(),
                },
            );
        }
        if decision.surface != surface
            || decision.decision != CloudComputeFunctionsAuthorizationDecision::Allow
            || decision.valid_until_epoch_seconds <= self.evaluation_epoch_seconds
        {
            return Err(CloudComputeFunctionsApiError::AuthorizationDenied {
                surface: surface.to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvokeRequest {
    pub invocation_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub function_id: String,                 // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: PUBLIC
    pub payload_data_class: String,          // data_class: INTERNAL_ONLY
    pub current_concurrent_invocations: u32, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvokeApiRequest {
    pub path_function_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudComputeFunctionsApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudComputeFunctionsApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudComputeFunctionsApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudComputeFunctionsInvokeRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvokeIdempotencyLedger {
    entries:
        BTreeMap<CloudComputeFunctionsIdempotencyLedgerKey, CloudComputeFunctionsInvokeLedgerEntry>, // data_class: INTERNAL_ONLY
    max_entries: usize, // data_class: INTERNAL_ONLY
}

impl Default for CloudComputeFunctionsInvokeIdempotencyLedger {
    fn default() -> Self {
        Self::with_max_entries(DEFAULT_FUNCTIONS_INVOKE_IDEMPOTENCY_LEDGER_MAX_ENTRIES)
    }
}

impl CloudComputeFunctionsInvokeIdempotencyLedger {
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries: max_entries.max(1),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn remember(
        &mut self,
        key: CloudComputeFunctionsIdempotencyLedgerKey,
        entry: CloudComputeFunctionsInvokeLedgerEntry,
    ) {
        if self.entries.len() >= self.max_entries {
            if let Some(evicted) = self.entries.keys().next().cloned() {
                self.entries.remove(&evicted);
            }
        }
        self.entries.insert(key, entry);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudComputeFunctionsIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeFunctionsInvokeLedgerEntry {
    fingerprint: CloudComputeFunctionsRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudComputeFunctionsInvokeApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeFunctionsRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudComputeFunctionsInvokeApiResult =
    Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvokeSuccessResponse {
    pub data: CloudComputeFunctionsInvocationReceipt, // data_class: INTERNAL_ONLY
    pub metadata: CloudComputeFunctionsMetadata,      // data_class: INTERNAL_ONLY
}

impl CloudComputeFunctionsInvokeSuccessResponse {
    pub fn accepted(
        data: CloudComputeFunctionsInvocationReceipt,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            data,
            metadata: CloudComputeFunctionsMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvocationReceipt {
    pub invocation_id: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub function_id: String,            // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub payload_data_class: String,     // data_class: INTERNAL_ONLY
    pub cold_start_budget_ms: u32,      // data_class: PUBLIC
    pub accepted_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiErrorResponse {
    pub error: CloudComputeFunctionsApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiErrorBody {
    pub code: String,                                      // data_class: INTERNAL_ONLY
    pub message: String,                                   // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,                 // data_class: INTERNAL_ONLY
    pub request_id: String,                                // data_class: INTERNAL_ONLY
    pub details: Vec<CloudComputeFunctionsApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeFunctionsApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathFunctionId,
    InvalidFunctionId {
        function_id: String,
    },
    FunctionKindMismatch {
        function_id: String,
        kind_label: String,
    },
    FunctionIdMismatch {
        path_function_id: String,
        body_function_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        resource_tenant_id: String,
        body_tenant_id: String,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationVerifierMissing,
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
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidPayloadDataClassLabel {
        payload_data_class: String,
    },
    Compute(CloudComputeError),
}

impl CloudComputeFunctionsApiError {
    pub fn invoke_status(&self) -> CloudComputeFunctionsInvokeApiStatus {
        match self.status_kind() {
            CloudComputeFunctionsApiStatusKind::BadRequest => {
                CloudComputeFunctionsInvokeApiStatus::BadRequest
            }
            CloudComputeFunctionsApiStatusKind::Unauthorized => {
                CloudComputeFunctionsInvokeApiStatus::Unauthorized
            }
            CloudComputeFunctionsApiStatusKind::Forbidden => {
                CloudComputeFunctionsInvokeApiStatus::Forbidden
            }
            CloudComputeFunctionsApiStatusKind::NotFound => {
                CloudComputeFunctionsInvokeApiStatus::NotFound
            }
            CloudComputeFunctionsApiStatusKind::Conflict => {
                CloudComputeFunctionsInvokeApiStatus::Conflict
            }
            CloudComputeFunctionsApiStatusKind::UnprocessableEntity => {
                CloudComputeFunctionsInvokeApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn invoke_status_code(&self) -> u16 {
        self.invoke_status().code()
    }

    pub fn code(&self) -> CloudComputeFunctionsApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudComputeFunctionsApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudComputeFunctionsApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudComputeFunctionsApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudComputeFunctionsApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathFunctionId => CloudComputeFunctionsApiErrorCode::PathFunctionIdEmpty,
            Self::InvalidFunctionId { .. } => CloudComputeFunctionsApiErrorCode::FunctionIdInvalid,
            Self::FunctionKindMismatch { .. } => {
                CloudComputeFunctionsApiErrorCode::FunctionKindMismatch
            }
            Self::FunctionIdMismatch { .. } => {
                CloudComputeFunctionsApiErrorCode::FunctionIdMismatch
            }
            Self::TenantMismatch { .. } => CloudComputeFunctionsApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudComputeFunctionsApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationVerifierMissing => {
                CloudComputeFunctionsApiErrorCode::AuthorizationVerifierMissing
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudComputeFunctionsApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudComputeFunctionsApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => {
                CloudComputeFunctionsApiErrorCode::AuthorizationDenied
            }
            Self::IdempotencyKeyReused { .. } => {
                CloudComputeFunctionsApiErrorCode::IdempotencyKeyReused
            }
            Self::InvalidPayloadDataClassLabel { .. } => {
                CloudComputeFunctionsApiErrorCode::PayloadDataClassInvalid
            }
            Self::Compute(error) => match cloud_compute_status_kind(error) {
                CloudComputeFunctionsApiStatusKind::BadRequest
                | CloudComputeFunctionsApiStatusKind::Unauthorized
                | CloudComputeFunctionsApiStatusKind::UnprocessableEntity => {
                    CloudComputeFunctionsApiErrorCode::ComputeInvalidRequest
                }
                CloudComputeFunctionsApiStatusKind::Forbidden => {
                    CloudComputeFunctionsApiErrorCode::ComputeForbidden
                }
                CloudComputeFunctionsApiStatusKind::NotFound => {
                    CloudComputeFunctionsApiErrorCode::ComputeNotFound
                }
                CloudComputeFunctionsApiStatusKind::Conflict => {
                    CloudComputeFunctionsApiErrorCode::ComputeConflict
                }
            },
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> CloudComputeFunctionsApiErrorResponse {
        CloudComputeFunctionsApiErrorResponse {
            error: CloudComputeFunctionsApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudComputeFunctionsApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudComputeFunctionsApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationVerifierMissing
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudComputeFunctionsApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => {
                CloudComputeFunctionsApiStatusKind::UnprocessableEntity
            }
            Self::Compute(error) => cloud_compute_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathFunctionId
            | Self::InvalidFunctionId { .. }
            | Self::FunctionKindMismatch { .. }
            | Self::FunctionIdMismatch { .. }
            | Self::InvalidPayloadDataClassLabel { .. } => {
                CloudComputeFunctionsApiStatusKind::BadRequest
            }
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathFunctionId => "Path function id is required",
            Self::InvalidFunctionId { .. } => "Function id must be a canonical Cloud resource id",
            Self::FunctionKindMismatch { .. } => "Function id must identify a function resource",
            Self::FunctionIdMismatch { .. } => "Path and body function ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, function id, and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationVerifierMissing => "Compute authorization verifier is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Compute Functions surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidPayloadDataClassLabel { .. } => {
                "Payload data class must be a known data-class label"
            }
            Self::Compute(error) => cloud_compute_message(error),
        }
    }

    fn details(&self) -> Vec<CloudComputeFunctionsApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathFunctionId => vec![detail("path.function_id", "must be non-empty")],
            Self::InvalidFunctionId { .. } => vec![detail(
                "path.function_id",
                "must be a canonical oya:cloud function resource id",
            )],
            Self::FunctionKindMismatch { .. } => {
                vec![detail("path.function_id", "resource kind must be function")]
            }
            Self::FunctionIdMismatch { .. } => vec![detail(
                "function_id",
                "path function_id and body function_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, resource tenant, and body tenant_id must match",
            )],
            Self::EmptyAuthorizationDecisionId => vec![detail(
                "authorization.decision_id",
                "must be non-empty authorization evidence",
            )],
            Self::AuthorizationVerifierMissing => vec![detail(
                "authorization.verifier",
                "compute boundary must verify decision_id against trusted local authorization state",
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
                "authorization",
                "must bind tenant, principal, requested surface, and unexpired decision evidence",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidPayloadDataClassLabel { .. } => vec![detail(
                "body.payload_data_class",
                "must be a canonical data-class label",
            )],
            Self::Compute(error) => vec![detail("cloud_compute", cloud_compute_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudComputeFunctionsApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_compute_functions_invoke_request(
    request: &CloudComputeFunctionsInvokeApiRequest,
) -> Result<ResourceId, CloudComputeFunctionsApiError> {
    validate_cloud_compute_functions_invoke_request_with_optional_authorization_verifier(
        request, None,
    )
}

pub fn validate_cloud_compute_functions_invoke_request_with_authorization_verifier(
    request: &CloudComputeFunctionsInvokeApiRequest,
    authorization_verifier: &CloudComputeFunctionsAuthorizationVerifier,
) -> Result<ResourceId, CloudComputeFunctionsApiError> {
    validate_cloud_compute_functions_invoke_request_with_optional_authorization_verifier(
        request,
        Some(authorization_verifier),
    )
}

fn validate_cloud_compute_functions_invoke_request_with_optional_authorization_verifier(
    request: &CloudComputeFunctionsInvokeApiRequest,
    authorization_verifier: Option<&CloudComputeFunctionsAuthorizationVerifier>,
) -> Result<ResourceId, CloudComputeFunctionsApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_function_id(&request.path_function_id, &request.body.function_id)?;
    let resource_id = validate_function_resource_id(&request.path_function_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &resource_id,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        authorization_verifier,
        &request.principal,
        &request.authorization.decision_id,
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE,
    )?;
    Ok(resource_id)
}

pub fn invoke_cloud_compute_function_from_api(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    validate_cloud_compute_functions_invoke_request(&request)?;
    invoke_validated_cloud_compute_function_from_api(catalog, idempotency_ledger, request)
}

pub fn invoke_cloud_compute_function_from_api_with_authorization_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    authorization_verifier: &CloudComputeFunctionsAuthorizationVerifier,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    validate_cloud_compute_functions_invoke_request_with_authorization_verifier(
        &request,
        authorization_verifier,
    )?;
    invoke_validated_cloud_compute_function_from_api(catalog, idempotency_ledger, request)
}

fn invoke_validated_cloud_compute_function_from_api(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    let input = function_invoke_input(&request.boundary, &request.body)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE,
    );
    let fingerprint = function_invoke_fingerprint_for(&request.path_function_id, &input);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudComputeFunctionsApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = catalog
        .invoke_function(input)
        .map_err(CloudComputeFunctionsApiError::Compute)
        .map(|receipt| {
            CloudComputeFunctionsInvokeSuccessResponse::accepted(
                invocation_receipt(receipt),
                request_id,
            )
        });
    idempotency_ledger.remember(
        key,
        CloudComputeFunctionsInvokeLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

/// Stable legacy entrypoint for `cloud.compute.functions.invoke`.
///
/// This entrypoint intentionally fails closed because it has no compute-owned
/// authorization verifier. Use `invoke_with_authorization_verifier` for live
/// API-boundary invocation.
pub fn invoke(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    invoke_cloud_compute_function_from_api(catalog, idempotency_ledger, request)
}

pub fn invoke_with_authorization_verifier(
    catalog: &mut CloudComputeCatalog,
    idempotency_ledger: &mut CloudComputeFunctionsInvokeIdempotencyLedger,
    authorization_verifier: &CloudComputeFunctionsAuthorizationVerifier,
    request: CloudComputeFunctionsInvokeApiRequest,
) -> Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError> {
    invoke_cloud_compute_function_from_api_with_authorization_verifier(
        catalog,
        idempotency_ledger,
        authorization_verifier,
        request,
    )
}

fn validate_boundary(
    boundary: &CloudComputeFunctionsApiBoundaryContext,
) -> Result<(), CloudComputeFunctionsApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_function_id(
    path_function_id: &str,
    body_function_id: &str,
) -> Result<(), CloudComputeFunctionsApiError> {
    if path_function_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyPathFunctionId);
    }
    if path_function_id != body_function_id {
        return Err(CloudComputeFunctionsApiError::FunctionIdMismatch {
            path_function_id: path_function_id.to_string(),
            body_function_id: body_function_id.to_string(),
        });
    }
    Ok(())
}

fn validate_function_resource_id(value: &str) -> Result<ResourceId, CloudComputeFunctionsApiError> {
    let id = ResourceId::new(value.to_string()).map_err(|_| {
        CloudComputeFunctionsApiError::InvalidFunctionId {
            function_id: value.to_string(),
        }
    })?;
    let kind_label =
        id.kind_label()
            .map_err(|_| CloudComputeFunctionsApiError::InvalidFunctionId {
                function_id: value.to_string(),
            })?;
    if kind_label != "function" {
        return Err(CloudComputeFunctionsApiError::FunctionKindMismatch {
            function_id: value.to_string(),
            kind_label,
        });
    }
    Ok(id)
}

fn validate_tenant_binding(
    boundary: &CloudComputeFunctionsApiBoundaryContext,
    principal: &CloudComputeFunctionsApiPrincipal,
    resource_id: &ResourceId,
    body_tenant_id: &str,
) -> Result<(), CloudComputeFunctionsApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyPrincipalId);
    }
    let resource_tenant_id =
        resource_id
            .tenant_id()
            .map_err(|_| CloudComputeFunctionsApiError::InvalidFunctionId {
                function_id: resource_id.value.clone(),
            })?;
    if boundary.tenant_id != principal.tenant_id
        || boundary.tenant_id != resource_tenant_id
        || boundary.tenant_id != body_tenant_id
    {
        return Err(CloudComputeFunctionsApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            resource_tenant_id,
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    authorization_verifier: Option<&CloudComputeFunctionsAuthorizationVerifier>,
    principal: &CloudComputeFunctionsApiPrincipal,
    decision_id: &str,
    surface: &str,
) -> Result<(), CloudComputeFunctionsApiError> {
    if decision_id.trim().is_empty() {
        return Err(CloudComputeFunctionsApiError::EmptyAuthorizationDecisionId);
    }
    let verifier = authorization_verifier
        .ok_or(CloudComputeFunctionsApiError::AuthorizationVerifierMissing)?;
    verifier.verify(principal, decision_id, surface)
}

fn function_invoke_input(
    boundary: &CloudComputeFunctionsApiBoundaryContext,
    body: &CloudComputeFunctionsInvokeRequest,
) -> Result<FunctionInvocationRequest, CloudComputeFunctionsApiError> {
    Ok(FunctionInvocationRequest {
        invocation_id: body.invocation_id.clone(),
        tenant_id: body.tenant_id.clone(),
        function_id: body.function_id.clone(),
        region: body.region.clone(),
        payload_data_class: parse_api_data_class(body.payload_data_class.clone())?,
        idempotency_key: boundary.idempotency_key.clone(),
        current_concurrent_invocations: body.current_concurrent_invocations,
        requested_at_epoch_seconds: body.requested_at_epoch_seconds,
    })
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudComputeFunctionsApiError> {
    parse_data_class_label(&label).ok_or(
        CloudComputeFunctionsApiError::InvalidPayloadDataClassLabel {
            payload_data_class: label,
        },
    )
}

fn idempotency_key_for(
    boundary: &CloudComputeFunctionsApiBoundaryContext,
    principal: &CloudComputeFunctionsApiPrincipal,
    surface: &str,
) -> CloudComputeFunctionsIdempotencyLedgerKey {
    CloudComputeFunctionsIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn function_invoke_fingerprint_for(
    path_function_id: &str,
    input: &FunctionInvocationRequest,
) -> CloudComputeFunctionsRequestFingerprint {
    CloudComputeFunctionsRequestFingerprint {
        canonical: canonical_fields(&[
            ("path.function_id", path_function_id.to_string()),
            ("body.invocation_id", input.invocation_id.clone()),
            ("body.tenant_id", input.tenant_id.clone()),
            ("body.function_id", input.function_id.clone()),
            ("body.region", input.region.clone()),
            (
                "body.payload_data_class",
                input.payload_data_class.label().to_string(),
            ),
            (
                "body.current_concurrent_invocations",
                input.current_concurrent_invocations.to_string(),
            ),
            (
                "body.requested_at_epoch_seconds",
                input.requested_at_epoch_seconds.to_string(),
            ),
        ]),
    }
}

fn canonical_fields(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{}:{}={}:{}", name.len(), name, value.len(), value))
        .collect::<Vec<_>>()
        .join("")
}

fn invocation_receipt(
    receipt: FunctionInvocationReceipt,
) -> CloudComputeFunctionsInvocationReceipt {
    CloudComputeFunctionsInvocationReceipt {
        invocation_id: receipt.invocation_id.value.value,
        tenant_id: receipt.tenant_id.value,
        function_id: receipt.function_id.value.value,
        region: receipt.region.value.value,
        payload_data_class: receipt.payload_data_class.value.label().to_string(),
        cold_start_budget_ms: receipt.cold_start_budget_ms.value,
        accepted_at_epoch_seconds: receipt.accepted_at_epoch_seconds.value,
        schema_version: receipt.schema_version.value,
    }
}

fn cloud_compute_status_kind(error: &CloudComputeError) -> CloudComputeFunctionsApiStatusKind {
    match error {
        CloudComputeError::DuplicateInstance
        | CloudComputeError::DuplicateKubernetesCluster
        | CloudComputeError::DuplicateFunction
        | CloudComputeError::DuplicateInvocation
        | CloudComputeError::FunctionNotActive => CloudComputeFunctionsApiStatusKind::Conflict,
        CloudComputeError::UnknownFunction => CloudComputeFunctionsApiStatusKind::NotFound,
        CloudComputeError::ResourceTenantMismatch
        | CloudComputeError::ResourceRegionMismatch
        | CloudComputeError::ResidencyRegionMismatch
        | CloudComputeError::QuotaExceeded
        | CloudComputeError::PayloadDataClassNotAllowed => {
            CloudComputeFunctionsApiStatusKind::Forbidden
        }
        CloudComputeError::InvalidTenantId
        | CloudComputeError::InvalidResourceId
        | CloudComputeError::ResourceKindMismatch
        | CloudComputeError::InvalidAzCode
        | CloudComputeError::AzRegionMismatch
        | CloudComputeError::InvalidCellId
        | CloudComputeError::CellAzMismatch
        | CloudComputeError::InvalidDataClass
        | CloudComputeError::InvalidImageRef
        | CloudComputeError::InvalidKeyPairId
        | CloudComputeError::InvalidUserDataUri
        | CloudComputeError::InvalidWorkloadIdentityPolicy
        | CloudComputeError::InvalidRuntimeIsolationPolicy
        | CloudComputeError::InvalidSchedulingPolicy
        | CloudComputeError::InvalidAuditEvidenceRef
        | CloudComputeError::InvalidFlavor
        | CloudComputeError::InvalidQuota
        | CloudComputeError::InvalidInstanceState
        | CloudComputeError::InvalidKubernetesState
        | CloudComputeError::InvalidFunctionState
        | CloudComputeError::InvalidNodePoolId
        | CloudComputeError::DuplicateNodePool
        | CloudComputeError::InvalidNodePoolShape
        | CloudComputeError::KubernetesHaRequiresThreeAzs
        | CloudComputeError::InvalidControlPlaneVersion
        | CloudComputeError::InvalidFunctionName
        | CloudComputeError::InvalidFunctionBudget
        | CloudComputeError::InvalidInvocationId
        | CloudComputeError::InvalidIdempotencyKey => {
            CloudComputeFunctionsApiStatusKind::BadRequest
        }
    }
}

fn cloud_compute_message(error: &CloudComputeError) -> &'static str {
    match cloud_compute_status_kind(error) {
        CloudComputeFunctionsApiStatusKind::BadRequest => {
            "Cloud Compute rejected the request shape"
        }
        CloudComputeFunctionsApiStatusKind::Unauthorized => {
            "Cloud Compute authentication evidence is missing"
        }
        CloudComputeFunctionsApiStatusKind::Forbidden => "Cloud Compute policy denied the request",
        CloudComputeFunctionsApiStatusKind::NotFound => "Cloud Compute function was not found",
        CloudComputeFunctionsApiStatusKind::Conflict => {
            "Cloud Compute function state conflicts with the request"
        }
        CloudComputeFunctionsApiStatusKind::UnprocessableEntity => {
            "Cloud Compute rejected request idempotency"
        }
    }
}

fn cloud_compute_issue(error: &CloudComputeError) -> &'static str {
    match error {
        CloudComputeError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudComputeError::InvalidResourceId => "function_id must be canonical cloud resource id",
        CloudComputeError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudComputeError::ResourceRegionMismatch => "resource region must match request region",
        CloudComputeError::ResourceKindMismatch => {
            "resource kind must match requested compute type"
        }
        CloudComputeError::InvalidAzCode => "AZ must be canonical lowercase ASCII",
        CloudComputeError::AzRegionMismatch => "AZ code must sit under its region code",
        CloudComputeError::InvalidCellId => "cell_id must be canonical and use the cell- prefix",
        CloudComputeError::CellAzMismatch => "cell_id must sit under its AZ namespace",
        CloudComputeError::ResidencyRegionMismatch => "region must satisfy residency policy",
        CloudComputeError::InvalidDataClass => "payload_data_class must be a privacy data class",
        CloudComputeError::InvalidImageRef => "function bundle must be a supported image ref",
        CloudComputeError::InvalidKeyPairId => "key_pair must use the key_ prefix",
        CloudComputeError::InvalidUserDataUri => "user_data_uri must use the userdata/ prefix",
        CloudComputeError::InvalidWorkloadIdentityPolicy => {
            "workload identity refs must be tenant/cell scoped and non-secret"
        }
        CloudComputeError::InvalidRuntimeIsolationPolicy => {
            "compute workloads require private and sandboxed runtime isolation"
        }
        CloudComputeError::InvalidSchedulingPolicy => {
            "compute scheduling evidence must require topology spread"
        }
        CloudComputeError::InvalidAuditEvidenceRef => {
            "compute audit evidence ref must be a non-secret evidence path"
        }
        CloudComputeError::InvalidFlavor => {
            "flavor resources must be positive and class-consistent"
        }
        CloudComputeError::InvalidQuota => "quota envelope must not start beyond its limits",
        CloudComputeError::QuotaExceeded => {
            "requested function exceeds tenant quota envelope or concurrency limit"
        }
        CloudComputeError::InvalidInstanceState => "VM create requests must start in Pending state",
        CloudComputeError::InvalidKubernetesState => {
            "Kubernetes create requests must start in Creating state"
        }
        CloudComputeError::InvalidFunctionState => {
            "function deployment state is not valid for this operation"
        }
        CloudComputeError::InvalidNodePoolId => "node pool id must use the np_ prefix",
        CloudComputeError::DuplicateNodePool => "node pool ids must be unique",
        CloudComputeError::InvalidNodePoolShape => "node pool shape must be canonical",
        CloudComputeError::KubernetesHaRequiresThreeAzs => {
            "HA Kubernetes requires at least three AZs"
        }
        CloudComputeError::InvalidControlPlaneVersion => "control plane version must be canonical",
        CloudComputeError::InvalidFunctionName => "function name must be canonical",
        CloudComputeError::InvalidFunctionBudget => {
            "function budget must be within platform bounds"
        }
        CloudComputeError::InvalidInvocationId => "invocation id must use the fninv_ prefix",
        CloudComputeError::InvalidIdempotencyKey => "idempotency key must be bounded",
        CloudComputeError::FunctionNotActive => "function must be active before invocation",
        CloudComputeError::PayloadDataClassNotAllowed => {
            "payload data class must be admitted by deployment policy"
        }
        CloudComputeError::DuplicateInstance => "instance resource id is already present",
        CloudComputeError::DuplicateKubernetesCluster => {
            "Kubernetes cluster resource id is already present"
        }
        CloudComputeError::DuplicateFunction => "function resource id is already present",
        CloudComputeError::DuplicateInvocation => "function invocation id is already present",
        CloudComputeError::UnknownFunction => "function resource must exist before invocation",
    }
}

fn detail(field: &str, issue: &str) -> CloudComputeFunctionsApiErrorDetail {
    CloudComputeFunctionsApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
