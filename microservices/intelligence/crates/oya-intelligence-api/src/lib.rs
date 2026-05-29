//! Foundry API boundary for inbound capability invocation requests.
//!
//! This crate owns REST/MCP-adjacent request validation before handing a
//! normalized application request to Foundation orchestration.

use std::collections::BTreeMap;

pub use oya_application_app::{
    CapabilityInvocationPrincipal, CapabilityInvocationRequest, Foundation, FoundationError,
    InvocationReceipt,
};

pub const CAPABILITY_INVOKE_SURFACE: &str = "foundry.capability.invoke";

pub type CapabilityInvocationReceipt = InvocationReceipt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityInvokeApiStatus {
    Accepted,
    BadRequest,
    Forbidden,
}

impl CapabilityInvokeApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityInvokeApiErrorCode {
    CapabilityPathIdEmpty,
    CapabilityIdMismatch,
    CapabilityRequestIdEmpty,
    CapabilityTenantHeaderEmpty,
    CapabilityIdempotencyKeyEmpty,
    CapabilityTenantMismatch,
    CapabilityIdempotencyKeyReused,
    CapabilityInvalidRequest,
    CapabilityInvocationForbidden,
}

impl CapabilityInvokeApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CapabilityPathIdEmpty => "CAPABILITY_PATH_ID_EMPTY",
            Self::CapabilityIdMismatch => "CAPABILITY_ID_MISMATCH",
            Self::CapabilityRequestIdEmpty => "CAPABILITY_REQUEST_ID_EMPTY",
            Self::CapabilityTenantHeaderEmpty => "CAPABILITY_TENANT_HEADER_EMPTY",
            Self::CapabilityIdempotencyKeyEmpty => "CAPABILITY_IDEMPOTENCY_KEY_EMPTY",
            Self::CapabilityTenantMismatch => "CAPABILITY_TENANT_MISMATCH",
            Self::CapabilityIdempotencyKeyReused => "CAPABILITY_IDEMPOTENCY_KEY_REUSED",
            Self::CapabilityInvalidRequest => "CAPABILITY_INVALID_REQUEST",
            Self::CapabilityInvocationForbidden => "CAPABILITY_INVOCATION_FORBIDDEN",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvokeApiRequest {
    pub path_capability_id: String,   // data_class: INTERNAL_ONLY
    pub boundary: ApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CapabilityInvocationPrincipal, // data_class: INTERNAL_ONLY
    pub body: CapabilityInvocationRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilityInvokeIdempotencyLedger {
    entries: BTreeMap<CapabilityInvokeIdempotencyLedgerKey, CapabilityInvokeIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CapabilityInvokeIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CapabilityInvokeIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    user_id: String,         // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapabilityInvokeIdempotencyLedgerEntry {
    fingerprint: CapabilityInvokeRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CapabilityInvokeApiResult,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapabilityInvokeRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CapabilityInvokeApiResult =
    Result<CapabilityInvokeApiSuccessResponse, CapabilityInvokeApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvokeApiSuccessResponse {
    pub data: CapabilityInvocationReceipt, // data_class: INTERNAL_ONLY
    pub metadata: CapabilityInvokeApiResponseMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvokeApiResponseMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

impl CapabilityInvokeApiSuccessResponse {
    pub fn accepted(
        data: CapabilityInvocationReceipt,
        request_id: impl Into<String>,
    ) -> CapabilityInvokeApiSuccessResponse {
        CapabilityInvokeApiSuccessResponse {
            data,
            metadata: CapabilityInvokeApiResponseMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvokeApiErrorResponse {
    pub error: CapabilityInvokeApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvokeApiErrorBody {
    pub code: String,                                 // data_class: INTERNAL_ONLY
    pub message: String,                              // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,            // data_class: INTERNAL_ONLY
    pub request_id: String,                           // data_class: INTERNAL_ONLY
    pub details: Vec<CapabilityInvokeApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvokeApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapabilityInvokeApiError {
    EmptyPathCapabilityId,
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    CapabilityIdMismatch {
        path_capability_id: String, // data_class: INTERNAL_ONLY
        body_capability_id: String, // data_class: INTERNAL_ONLY
    },
    TenantMismatch {
        header_tenant_id: String,    // data_class: INTERNAL_ONLY
        principal_tenant_id: String, // data_class: INTERNAL_ONLY
        body_tenant_id: String,      // data_class: INTERNAL_ONLY
    },
    PrincipalMismatch {
        principal_tenant_id: String, // data_class: INTERNAL_ONLY
        principal_user_id: String,   // data_class: INTERNAL_ONLY
        body_tenant_id: String,      // data_class: INTERNAL_ONLY
        body_user_id: String,        // data_class: INTERNAL_ONLY
    },
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
    Foundation(FoundationError),
}

impl CapabilityInvokeApiError {
    pub fn status(&self) -> CapabilityInvokeApiStatus {
        match self {
            Self::EmptyPathCapabilityId
            | Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::CapabilityIdMismatch { .. } => CapabilityInvokeApiStatus::BadRequest,
            Self::IdempotencyKeyReused { .. } => CapabilityInvokeApiStatus::BadRequest,
            Self::TenantMismatch { .. } | Self::PrincipalMismatch { .. } => {
                CapabilityInvokeApiStatus::Forbidden
            }
            Self::Foundation(error) => foundation_error_status(error),
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> CapabilityInvokeApiErrorCode {
        match self {
            Self::EmptyPathCapabilityId => CapabilityInvokeApiErrorCode::CapabilityPathIdEmpty,
            Self::EmptyRequestId => CapabilityInvokeApiErrorCode::CapabilityRequestIdEmpty,
            Self::EmptyTenantHeader => CapabilityInvokeApiErrorCode::CapabilityTenantHeaderEmpty,
            Self::EmptyIdempotencyKey => {
                CapabilityInvokeApiErrorCode::CapabilityIdempotencyKeyEmpty
            }
            Self::CapabilityIdMismatch { .. } => CapabilityInvokeApiErrorCode::CapabilityIdMismatch,
            Self::TenantMismatch { .. } => CapabilityInvokeApiErrorCode::CapabilityTenantMismatch,
            Self::PrincipalMismatch { .. } => {
                CapabilityInvokeApiErrorCode::CapabilityInvocationForbidden
            }
            Self::IdempotencyKeyReused { .. } => {
                CapabilityInvokeApiErrorCode::CapabilityIdempotencyKeyReused
            }
            Self::Foundation(FoundationError::InvalidInput | FoundationError::TokenTtlTooLong) => {
                CapabilityInvokeApiErrorCode::CapabilityInvalidRequest
            }
            Self::Foundation(_) => CapabilityInvokeApiErrorCode::CapabilityInvocationForbidden,
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> CapabilityInvokeApiErrorResponse {
        CapabilityInvokeApiErrorResponse {
            error: CapabilityInvokeApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyPathCapabilityId => "Path capability id is required",
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::CapabilityIdMismatch { .. } => "Path and body capability ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and invocation body"
            }
            Self::PrincipalMismatch { .. } => "Authenticated principal must match invocation body",
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::Foundation(FoundationError::InvalidInput | FoundationError::TokenTtlTooLong) => {
                "Foundation rejected the request as invalid"
            }
            Self::Foundation(_) => "Foundation policy rejected the capability invocation",
        }
    }

    fn details(&self) -> Vec<CapabilityInvokeApiErrorDetail> {
        match self {
            Self::EmptyPathCapabilityId => vec![CapabilityInvokeApiErrorDetail {
                field: "path.capability_id".into(),
                issue: "must be non-empty".into(),
            }],
            Self::EmptyRequestId => vec![CapabilityInvokeApiErrorDetail {
                field: "header.X-Request-Id".into(),
                issue: "must be non-empty".into(),
            }],
            Self::EmptyTenantHeader => vec![CapabilityInvokeApiErrorDetail {
                field: "header.X-Tenant-Id".into(),
                issue: "must be non-empty".into(),
            }],
            Self::EmptyIdempotencyKey => vec![CapabilityInvokeApiErrorDetail {
                field: "header.Idempotency-Key".into(),
                issue: "must be non-empty".into(),
            }],
            Self::CapabilityIdMismatch { .. } => vec![CapabilityInvokeApiErrorDetail {
                field: "capability_id".into(),
                issue: "path and body capability_id must match".into(),
            }],
            Self::TenantMismatch { .. } => vec![CapabilityInvokeApiErrorDetail {
                field: "header.X-Tenant-Id".into(),
                issue: "must match authenticated tenant and body tenant_id".into(),
            }],
            Self::PrincipalMismatch { .. } => vec![CapabilityInvokeApiErrorDetail {
                field: "principal".into(),
                issue: "authenticated tenant and subject must match body tenant_id and user_id"
                    .into(),
            }],
            Self::IdempotencyKeyReused { .. } => vec![CapabilityInvokeApiErrorDetail {
                field: "header.Idempotency-Key".into(),
                issue: "same key cannot be reused with a different request fingerprint".into(),
            }],
            Self::Foundation(FoundationError::InvalidInput | FoundationError::TokenTtlTooLong) => {
                vec![CapabilityInvokeApiErrorDetail {
                    field: "request".into(),
                    issue: "foundation rejected invalid input".into(),
                }]
            }
            Self::Foundation(_) => vec![CapabilityInvokeApiErrorDetail {
                field: "foundation".into(),
                issue: "foundation policy rejected invocation".into(),
            }],
        }
    }
}

pub fn validate_capability_invoke_request(
    request: &CapabilityInvokeApiRequest,
) -> Result<(), CapabilityInvokeApiError> {
    if request.boundary.request_id.trim().is_empty() {
        return Err(CapabilityInvokeApiError::EmptyRequestId);
    }
    if request.boundary.tenant_id.trim().is_empty() {
        return Err(CapabilityInvokeApiError::EmptyTenantHeader);
    }
    if request.boundary.idempotency_key.trim().is_empty() {
        return Err(CapabilityInvokeApiError::EmptyIdempotencyKey);
    }
    if request.path_capability_id.trim().is_empty() {
        return Err(CapabilityInvokeApiError::EmptyPathCapabilityId);
    }

    if request.path_capability_id != request.body.capability_id {
        return Err(CapabilityInvokeApiError::CapabilityIdMismatch {
            path_capability_id: request.path_capability_id.clone(),
            body_capability_id: request.body.capability_id.clone(),
        });
    }
    if request.boundary.tenant_id != request.principal.tenant_id
        || request.boundary.tenant_id != request.body.tenant_id
    {
        return Err(CapabilityInvokeApiError::TenantMismatch {
            header_tenant_id: request.boundary.tenant_id.clone(),
            principal_tenant_id: request.principal.tenant_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
        });
    }
    if request.principal.tenant_id != request.body.tenant_id
        || request.principal.user_id != request.body.user_id
    {
        return Err(CapabilityInvokeApiError::PrincipalMismatch {
            principal_tenant_id: request.principal.tenant_id.clone(),
            principal_user_id: request.principal.user_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
            body_user_id: request.body.user_id.clone(),
        });
    }

    Ok(())
}

pub fn invoke_capability_from_api(
    foundation: &mut Foundation,
    idempotency_ledger: &mut CapabilityInvokeIdempotencyLedger,
    request: CapabilityInvokeApiRequest,
) -> Result<CapabilityInvokeApiSuccessResponse, CapabilityInvokeApiError> {
    validate_capability_invoke_request(&request)?;
    let key = idempotency_key_for(&request);
    let fingerprint = fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CapabilityInvokeApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let result = foundation
        .invoke_capability_as_principal(request.principal.clone(), request.body.clone())
        .map(|receipt| {
            CapabilityInvokeApiSuccessResponse::accepted(receipt, request.boundary.request_id)
        })
        .map_err(CapabilityInvokeApiError::Foundation);
    if should_cache_capability_invoke_result(&result) {
        idempotency_ledger.entries.insert(
            key,
            CapabilityInvokeIdempotencyLedgerEntry {
                fingerprint,
                result: result.clone(),
            },
        );
    }
    result
}

fn should_cache_capability_invoke_result(result: &CapabilityInvokeApiResult) -> bool {
    match result {
        Ok(_) => true,
        Err(CapabilityInvokeApiError::Foundation(error)) => matches!(
            foundation_error_status(error),
            CapabilityInvokeApiStatus::BadRequest | CapabilityInvokeApiStatus::Forbidden
        ),
        Err(_) => false,
    }
}

fn idempotency_key_for(
    request: &CapabilityInvokeApiRequest,
) -> CapabilityInvokeIdempotencyLedgerKey {
    CapabilityInvokeIdempotencyLedgerKey {
        tenant_id: request.boundary.tenant_id.clone(),
        user_id: request.principal.user_id.clone(),
        surface: CAPABILITY_INVOKE_SURFACE.to_string(),
        idempotency_key: request.boundary.idempotency_key.clone(),
    }
}

fn fingerprint_for(request: &CapabilityInvokeApiRequest) -> CapabilityInvokeRequestFingerprint {
    CapabilityInvokeRequestFingerprint {
        canonical: [
            format!("path_capability_id={}", request.path_capability_id),
            format!("context.tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.user_id={}", request.principal.user_id),
            format!(
                "principal.autonomy_ceiling={:?}",
                request.principal.autonomy_ceiling
            ),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.user_id={}", request.body.user_id),
            format!("body.capability_id={}", request.body.capability_id),
            format!("body.purpose={:?}", request.body.purpose),
            format!("body.subject_class={:?}", request.body.subject_class),
            format!("body.budget_window_id={}", request.body.budget_window_id),
            format!(
                "body.projected_cost_micros={}",
                request.body.projected_cost_micros
            ),
            format!(
                "body.started_at_epoch_seconds={}",
                request.body.started_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn foundation_error_status(error: &FoundationError) -> CapabilityInvokeApiStatus {
    match error {
        FoundationError::InvalidInput | FoundationError::TokenTtlTooLong => {
            CapabilityInvokeApiStatus::BadRequest
        }
        FoundationError::AutonomyCeilingExceeded
        | FoundationError::CapabilityAlreadyExists
        | FoundationError::CapabilityEvalGateNotReady
        | FoundationError::CapabilityInvocationUnauthorized
        | FoundationError::CapabilityNotFound
        | FoundationError::CapabilityNotLicensed
        | FoundationError::CellBindingImmutable
        | FoundationError::CostBudgetExceeded
        | FoundationError::CostBudgetNotConfigured
        | FoundationError::DataUseNotAllowed
        | FoundationError::McpAccessDenied
        | FoundationError::McpRateLimited
        | FoundationError::OutboxRecordNotFound
        | FoundationError::PolicyVersionAlreadyExists
        | FoundationError::RegionalPackAlreadyExists
        | FoundationError::TenantAlreadyExists
        | FoundationError::TenantNotFound
        | FoundationError::UserNotFound => CapabilityInvokeApiStatus::Forbidden,
        // fail-closed: audit-chain append failure denies the request (no 5xx status in this enum; TODO follow-up)
        FoundationError::AuditChainAppendFailed(_) => CapabilityInvokeApiStatus::Forbidden,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_application_app::{AutonomyTier, Purpose, SubjectClass};

    fn fingerprint_request() -> CapabilityInvokeApiRequest {
        CapabilityInvokeApiRequest {
            path_capability_id: "cap.workflow.approve-payroll".to_string(),
            boundary: ApiBoundaryContext {
                request_id: "req-fingerprint".to_string(),
                tenant_id: "ten_api".to_string(),
                idempotency_key: "idem-fingerprint".to_string(),
            },
            principal: CapabilityInvocationPrincipal {
                tenant_id: "ten_api".to_string(),
                user_id: "usr_api".to_string(),
                autonomy_ceiling: AutonomyTier::T2Advisory,
            },
            body: CapabilityInvocationRequest {
                tenant_id: "ten_api".to_string(),
                user_id: "usr_api".to_string(),
                capability_id: "cap.workflow.approve-payroll".to_string(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".to_string(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_778_413_600,
            },
        }
    }

    #[test]
    fn capability_invoke_fingerprint_material_uses_exact_contract_fields_in_order() {
        let fingerprint = fingerprint_for(&fingerprint_request());

        assert_eq!(
            fingerprint.canonical,
            "path_capability_id=cap.workflow.approve-payroll|\
context.tenant_id=ten_api|\
principal.tenant_id=ten_api|\
principal.user_id=usr_api|\
principal.autonomy_ceiling=T2Advisory|\
body.tenant_id=ten_api|\
body.user_id=usr_api|\
body.capability_id=cap.workflow.approve-payroll|\
body.purpose=CapabilityInvocation|\
body.subject_class=Adult|\
body.budget_window_id=2026-05|\
body.projected_cost_micros=10|\
body.started_at_epoch_seconds=1778413600"
        );
    }

    #[test]
    fn capability_invoke_fingerprint_is_sensitive_to_every_contract_field() {
        let baseline = fingerprint_for(&fingerprint_request());
        let mut cases: Vec<(&str, CapabilityInvokeApiRequest)> = Vec::new();

        let mut request = fingerprint_request();
        request.path_capability_id = "cap.workflow.export-ledger".to_string();
        cases.push(("path_capability_id", request));

        let mut request = fingerprint_request();
        request.boundary.tenant_id = "ten_other".to_string();
        cases.push(("context.tenant_id", request));

        let mut request = fingerprint_request();
        request.principal.tenant_id = "ten_other".to_string();
        cases.push(("principal.tenant_id", request));

        let mut request = fingerprint_request();
        request.principal.user_id = "usr_other".to_string();
        cases.push(("principal.user_id", request));

        let mut request = fingerprint_request();
        request.principal.autonomy_ceiling = AutonomyTier::T3ExecuteWithApproval;
        cases.push(("principal.autonomy_ceiling", request));

        let mut request = fingerprint_request();
        request.body.tenant_id = "ten_other".to_string();
        cases.push(("body.tenant_id", request));

        let mut request = fingerprint_request();
        request.body.user_id = "usr_other".to_string();
        cases.push(("body.user_id", request));

        let mut request = fingerprint_request();
        request.body.capability_id = "cap.workflow.export-ledger".to_string();
        cases.push(("body.capability_id", request));

        let mut request = fingerprint_request();
        request.body.purpose = Purpose::CoreService;
        cases.push(("body.purpose", request));

        let mut request = fingerprint_request();
        request.body.subject_class = SubjectClass::Authority;
        cases.push(("body.subject_class", request));

        let mut request = fingerprint_request();
        request.body.budget_window_id = "2026-06".to_string();
        cases.push(("body.budget_window_id", request));

        let mut request = fingerprint_request();
        request.body.projected_cost_micros = 11;
        cases.push(("body.projected_cost_micros", request));

        let mut request = fingerprint_request();
        request.body.started_at_epoch_seconds += 1;
        cases.push(("body.started_at_epoch_seconds", request));

        for (field, request) in cases {
            assert_ne!(
                fingerprint_for(&request),
                baseline,
                "{field} must affect fingerprint"
            );
        }
    }
}
