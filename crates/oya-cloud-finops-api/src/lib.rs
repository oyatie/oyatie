//! Cloud FinOps report API boundary.
//!
//! This crate owns tenant/header/path/body normalization, idempotency, and
//! authenticated API projection before handing typed report generation requests
//! to the Cloud FinOps kernel.

use std::collections::BTreeMap;

use oya_cloud_billing_domain::Money;
use oya_cloud_finops_domain::{
    AnomalyPolicy, AxisCostBreakdown, CloudFinopsError, CloudFinopsLedger, CostAnomaly,
    CostAnomalyKind, FinopsPeriod, FinopsRecommendation, FinopsReport, FinopsReportRequest,
    RecommendationKind, ResourceCostBreakdown,
};
use oya_data_boundary_kernel::{parse_data_class_label, DataClass};
use oya_metering_domain::AxisId;

pub const CLOUD_FINOPS_REPORT_SURFACE: &str = "cloud.finops.report";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudFinopsReportApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CloudFinopsReportApiStatus {
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
pub enum CloudFinopsApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathReportIdEmpty,
    ReportIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    AxisInvalid,
    DataClassInvalid,
    FinopsInvalidRequest,
    FinopsForbidden,
    FinopsConflict,
    FinopsUnprocessable,
}

impl CloudFinopsApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_FINOPS_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_FINOPS_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_FINOPS_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_FINOPS_PRINCIPAL_ID_EMPTY",
            Self::PathReportIdEmpty => "CLOUD_FINOPS_PATH_REPORT_ID_EMPTY",
            Self::ReportIdMismatch => "CLOUD_FINOPS_REPORT_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_FINOPS_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "CLOUD_FINOPS_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "CLOUD_FINOPS_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "CLOUD_FINOPS_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "CLOUD_FINOPS_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_FINOPS_IDEMPOTENCY_KEY_REUSED",
            Self::AxisInvalid => "CLOUD_FINOPS_AXIS_INVALID",
            Self::DataClassInvalid => "CLOUD_FINOPS_DATA_CLASS_INVALID",
            Self::FinopsInvalidRequest => "CLOUD_FINOPS_INVALID_REQUEST",
            Self::FinopsForbidden => "CLOUD_FINOPS_FORBIDDEN",
            Self::FinopsConflict => "CLOUD_FINOPS_CONFLICT",
            Self::FinopsUnprocessable => "CLOUD_FINOPS_UNPROCESSABLE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsPeriodRequest {
    pub start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsAxisRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsReportAnomalyPolicyRequest {
    pub spend_growth_threshold_bps: u16, // data_class: INTERNAL_ONLY
    pub min_absolute_delta_minor_units: u64, // data_class: FINANCIAL_KR_신용정보
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsReportGenerateRequest {
    pub id: String,                                            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                     // data_class: INTERNAL_ONLY
    pub region: String,                                        // data_class: PUBLIC
    pub period: CloudFinopsPeriodRequest,                      // data_class: INTERNAL_ONLY
    pub baseline_period: Option<CloudFinopsPeriodRequest>,     // data_class: INTERNAL_ONLY
    pub axes: Vec<CloudFinopsAxisRef>,                         // data_class: INTERNAL_ONLY
    pub anomaly_policy: CloudFinopsReportAnomalyPolicyRequest, // data_class: INTERNAL_ONLY
    pub minimum_gross_margin_bps: u16,                         // data_class: INTERNAL_ONLY
    pub data_class: String,                                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsReportApiRequest {
    pub path_report_id: String,                  // data_class: INTERNAL_ONLY
    pub boundary: CloudFinopsApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudFinopsApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: CloudFinopsApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudFinopsReportGenerateRequest,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudFinopsReportGenerateIdempotencyLedger {
    entries:
        BTreeMap<CloudFinopsReportIdempotencyLedgerKey, CloudFinopsReportIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudFinopsReportGenerateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudFinopsReportIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudFinopsReportIdempotencyLedgerEntry {
    fingerprint: CloudFinopsReportRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudFinopsReportApiResult,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudFinopsReportRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudFinopsReportApiResult =
    Result<CloudFinopsReportGenerateSuccessResponse, CloudFinopsReportApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsReportGenerateSuccessResponse {
    pub data: CloudFinopsReportRecord, // data_class: FINANCIAL_KR_신용정보
    pub metadata: CloudFinopsReportMetadata, // data_class: INTERNAL_ONLY
}

impl CloudFinopsReportGenerateSuccessResponse {
    pub fn created(
        data: CloudFinopsReportRecord,
        request_id: impl Into<String>,
        tenant_id: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        let axis_count = data.axis_costs.len() as u32;
        let resource_count = data.resource_costs.len() as u32;
        let anomaly_count = data.anomalies.len() as u32;
        let recommendation_count = data.recommendations.len() as u32;
        Self {
            data,
            metadata: CloudFinopsReportMetadata {
                request_id: request_id.into(),
                tenant_id: tenant_id.into(),
                region: region.into(),
                axis_count,
                resource_count,
                anomaly_count,
                recommendation_count,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsReportMetadata {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub region: String,            // data_class: PUBLIC
    pub axis_count: u32,           // data_class: INTERNAL_ONLY
    pub resource_count: u32,       // data_class: INTERNAL_ONLY
    pub anomaly_count: u32,        // data_class: INTERNAL_ONLY
    pub recommendation_count: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsMoneyRecord {
    pub currency: String, // data_class: INTERNAL_ONLY
    pub minor_units: u64, // data_class: FINANCIAL_KR_신용정보
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsReportRecord {
    pub id: String,                                            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                     // data_class: INTERNAL_ONLY
    pub region: String,                                        // data_class: PUBLIC
    pub period_start_epoch_seconds: u64,                       // data_class: INTERNAL_ONLY
    pub period_end_epoch_seconds: u64,                         // data_class: INTERNAL_ONLY
    pub axes: Vec<CloudFinopsAxisRef>,                         // data_class: INTERNAL_ONLY
    pub axis_costs: Vec<CloudFinopsAxisCostRecord>,            // data_class: FINANCIAL_KR_신용정보
    pub resource_costs: Vec<CloudFinopsResourceCostRecord>,    // data_class: FINANCIAL_KR_신용정보
    pub anomalies: Vec<CloudFinopsCostAnomalyRecord>,          // data_class: FINANCIAL_KR_신용정보
    pub recommendations: Vec<CloudFinopsRecommendationRecord>, // data_class: INTERNAL_ONLY
    pub total_cost: CloudFinopsMoneyRecord,                    // data_class: FINANCIAL_KR_신용정보
    pub total_cost_of_revenue: CloudFinopsMoneyRecord,         // data_class: FINANCIAL_KR_신용정보
    pub gross_margin_bps: u16,                                 // data_class: INTERNAL_ONLY
    pub minimum_gross_margin_bps: u16,                         // data_class: INTERNAL_ONLY
    pub data_class: String,                                    // data_class: INTERNAL_ONLY
    pub schema_version: u32,                                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsAxisCostRecord {
    pub axis: String,                            // data_class: INTERNAL_ONLY
    pub actual_cost: CloudFinopsMoneyRecord,     // data_class: FINANCIAL_KR_신용정보
    pub cost_of_revenue: CloudFinopsMoneyRecord, // data_class: FINANCIAL_KR_신용정보
    pub gross_margin_bps: u16,                   // data_class: INTERNAL_ONLY
    pub budget: Option<CloudFinopsMoneyRecord>,  // data_class: FINANCIAL_KR_신용정보
    pub budget_utilization_bps: Option<u16>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsResourceCostRecord {
    pub resource_id: String,                     // data_class: INTERNAL_ONLY
    pub axis: String,                            // data_class: INTERNAL_ONLY
    pub actual_cost: CloudFinopsMoneyRecord,     // data_class: FINANCIAL_KR_신용정보
    pub cost_of_revenue: CloudFinopsMoneyRecord, // data_class: FINANCIAL_KR_신용정보
    pub gross_margin_bps: u16,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsCostAnomalyRecord {
    pub kind: String,                                  // data_class: INTERNAL_ONLY
    pub axis: String,                                  // data_class: INTERNAL_ONLY
    pub resource_id: Option<String>,                   // data_class: INTERNAL_ONLY
    pub actual_cost: CloudFinopsMoneyRecord,           // data_class: FINANCIAL_KR_신용정보
    pub baseline_cost: Option<CloudFinopsMoneyRecord>, // data_class: FINANCIAL_KR_신용정보
    pub threshold_bps: u16,                            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsRecommendationRecord {
    pub id: String,                  // data_class: INTERNAL_ONLY
    pub kind: String,                // data_class: INTERNAL_ONLY
    pub axis: String,                // data_class: INTERNAL_ONLY
    pub resource_id: Option<String>, // data_class: INTERNAL_ONLY
    pub evidence_anomaly: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsApiErrorResponse {
    pub error: CloudFinopsApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsApiErrorBody {
    pub code: String,                            // data_class: INTERNAL_ONLY
    pub message: String,                         // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,       // data_class: INTERNAL_ONLY
    pub request_id: String,                      // data_class: INTERNAL_ONLY
    pub details: Vec<CloudFinopsApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudFinopsReportApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathReportId,
    ReportIdMismatch {
        path_report_id: String,
        body_report_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        body_tenant_id: String,
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
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidAxisLabel {
        axis: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    Finops(CloudFinopsError),
}

impl CloudFinopsReportApiError {
    pub fn finops_report_status(&self) -> CloudFinopsReportApiStatus {
        match self.status_kind() {
            CloudFinopsReportApiStatusKind::BadRequest => CloudFinopsReportApiStatus::BadRequest,
            CloudFinopsReportApiStatusKind::Unauthorized => {
                CloudFinopsReportApiStatus::Unauthorized
            }
            CloudFinopsReportApiStatusKind::Forbidden => CloudFinopsReportApiStatus::Forbidden,
            CloudFinopsReportApiStatusKind::Conflict => CloudFinopsReportApiStatus::Conflict,
            CloudFinopsReportApiStatusKind::UnprocessableEntity => {
                CloudFinopsReportApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn finops_report_status_code(&self) -> u16 {
        self.finops_report_status().code()
    }

    pub fn code(&self) -> CloudFinopsApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudFinopsApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudFinopsApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudFinopsApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudFinopsApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathReportId => CloudFinopsApiErrorCode::PathReportIdEmpty,
            Self::ReportIdMismatch { .. } => CloudFinopsApiErrorCode::ReportIdMismatch,
            Self::TenantMismatch { .. } => CloudFinopsApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudFinopsApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudFinopsApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudFinopsApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudFinopsApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudFinopsApiErrorCode::IdempotencyKeyReused,
            Self::InvalidAxisLabel { .. } => CloudFinopsApiErrorCode::AxisInvalid,
            Self::InvalidDataClassLabel { .. } => CloudFinopsApiErrorCode::DataClassInvalid,
            Self::Finops(error) => match cloud_finops_status_kind(error) {
                CloudFinopsReportApiStatusKind::BadRequest => {
                    CloudFinopsApiErrorCode::FinopsInvalidRequest
                }
                CloudFinopsReportApiStatusKind::Forbidden => {
                    CloudFinopsApiErrorCode::FinopsForbidden
                }
                CloudFinopsReportApiStatusKind::Conflict => CloudFinopsApiErrorCode::FinopsConflict,
                CloudFinopsReportApiStatusKind::UnprocessableEntity => {
                    CloudFinopsApiErrorCode::FinopsUnprocessable
                }
                CloudFinopsReportApiStatusKind::Unauthorized => {
                    CloudFinopsApiErrorCode::FinopsInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudFinopsApiErrorResponse {
        CloudFinopsApiErrorResponse {
            error: CloudFinopsApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudFinopsReportApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudFinopsReportApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudFinopsReportApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => {
                CloudFinopsReportApiStatusKind::UnprocessableEntity
            }
            Self::Finops(error) => cloud_finops_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathReportId
            | Self::ReportIdMismatch { .. }
            | Self::InvalidAxisLabel { .. }
            | Self::InvalidDataClassLabel { .. } => CloudFinopsReportApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathReportId => "Path report id is required",
            Self::ReportIdMismatch { .. } => "Path and body report ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud FinOps report surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidAxisLabel { .. } => "Axis must be a known Oyatie meter axis label",
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::Finops(error) => cloud_finops_message(error),
        }
    }

    fn details(&self) -> Vec<CloudFinopsApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathReportId => vec![detail("path.report_id", "must be non-empty")],
            Self::ReportIdMismatch { .. } => {
                vec![detail("id", "path report_id and body id must match")]
            }
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
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
                "must include the requested Cloud FinOps report surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidAxisLabel { .. } => vec![detail(
                "body.axes.value",
                "must be one of saas, foundry, cloud, search, ads, marketplace, or vertical",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::Finops(error) => vec![detail("cloud_finops", cloud_finops_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudFinopsReportApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_finops_report_request(
    request: &CloudFinopsReportApiRequest,
) -> Result<(), CloudFinopsReportApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_report_id(&request.path_report_id, &request.body.id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_FINOPS_REPORT_SURFACE,
    )?;
    validate_body_labels(&request.body)
}

pub fn generate_cloud_finops_report_from_api(
    ledger: &mut CloudFinopsLedger,
    idempotency_ledger: &mut CloudFinopsReportGenerateIdempotencyLedger,
    request: CloudFinopsReportApiRequest,
) -> Result<CloudFinopsReportGenerateSuccessResponse, CloudFinopsReportApiError> {
    validate_cloud_finops_report_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_FINOPS_REPORT_SURFACE,
    );
    let fingerprint = finops_report_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudFinopsReportApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let body = request.body;
    let result = report_generate_input(body)
        .and_then(|input| {
            ledger
                .generate_report(input)
                .map_err(CloudFinopsReportApiError::Finops)
        })
        .map(|report| {
            let tenant_id = report.tenant_id.value.clone();
            let region = report.region.value.value.clone();
            CloudFinopsReportGenerateSuccessResponse::created(
                report_record(report),
                request_id,
                tenant_id,
                region,
            )
        });
    idempotency_ledger.entries.insert(
        key,
        CloudFinopsReportIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudFinopsApiBoundaryContext,
) -> Result<(), CloudFinopsReportApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudFinopsReportApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudFinopsReportApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudFinopsReportApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_report_id(
    path_report_id: &str,
    body_report_id: &str,
) -> Result<(), CloudFinopsReportApiError> {
    if path_report_id.trim().is_empty() {
        return Err(CloudFinopsReportApiError::EmptyPathReportId);
    }
    if path_report_id != body_report_id {
        return Err(CloudFinopsReportApiError::ReportIdMismatch {
            path_report_id: path_report_id.to_string(),
            body_report_id: body_report_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudFinopsApiBoundaryContext,
    principal: &CloudFinopsApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudFinopsReportApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudFinopsReportApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudFinopsReportApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudFinopsApiPrincipal,
    authorization: &CloudFinopsApiAuthorization,
    surface: &str,
) -> Result<(), CloudFinopsReportApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudFinopsReportApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudFinopsReportApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudFinopsReportApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudFinopsReportApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn validate_body_labels(
    body: &CloudFinopsReportGenerateRequest,
) -> Result<(), CloudFinopsReportApiError> {
    parse_api_data_class(body.data_class.clone())?;
    for axis in &body.axes {
        parse_axis(axis.value.clone())?;
    }
    Ok(())
}

fn report_generate_input(
    input: CloudFinopsReportGenerateRequest,
) -> Result<FinopsReportRequest, CloudFinopsReportApiError> {
    Ok(FinopsReportRequest {
        id: input.id,
        tenant_id: input.tenant_id,
        region: input.region,
        period: period_input(input.period)?,
        baseline_period: input.baseline_period.map(period_input).transpose()?,
        axes: input
            .axes
            .into_iter()
            .map(axis_ref_input)
            .collect::<Result<Vec<_>, _>>()?,
        anomaly_policy: AnomalyPolicy::new(
            input.anomaly_policy.spend_growth_threshold_bps,
            input.anomaly_policy.min_absolute_delta_minor_units,
        )
        .map_err(CloudFinopsReportApiError::Finops)?,
        minimum_gross_margin_bps: input.minimum_gross_margin_bps,
        data_class: parse_api_data_class(input.data_class)?,
    })
}

fn period_input(
    input: CloudFinopsPeriodRequest,
) -> Result<FinopsPeriod, CloudFinopsReportApiError> {
    FinopsPeriod::new(input.start_epoch_seconds, input.end_epoch_seconds)
        .map_err(CloudFinopsReportApiError::Finops)
}

fn axis_ref_input(input: CloudFinopsAxisRef) -> Result<AxisId, CloudFinopsReportApiError> {
    parse_axis(input.value)
}

fn parse_axis(label: String) -> Result<AxisId, CloudFinopsReportApiError> {
    match label.as_str() {
        "saas" => Ok(AxisId::Saas),
        "foundry" => Ok(AxisId::Foundry),
        "cloud" => Ok(AxisId::Cloud),
        "search" => Ok(AxisId::Search),
        "ads" => Ok(AxisId::Ads),
        "marketplace" => Ok(AxisId::Marketplace),
        "vertical" => Ok(AxisId::Vertical),
        _ => Err(CloudFinopsReportApiError::InvalidAxisLabel { axis: label }),
    }
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudFinopsReportApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudFinopsReportApiError::InvalidDataClassLabel { data_class: label })
}

fn idempotency_key_for(
    boundary: &CloudFinopsApiBoundaryContext,
    principal: &CloudFinopsApiPrincipal,
    surface: &str,
) -> CloudFinopsReportIdempotencyLedgerKey {
    CloudFinopsReportIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn finops_report_fingerprint_for(
    request: &CloudFinopsReportApiRequest,
) -> CloudFinopsReportRequestFingerprint {
    CloudFinopsReportRequestFingerprint {
        canonical: [
            format!("path.report_id={}", request.path_report_id),
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
            format!("body.id={}", request.body.id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.region={}", request.body.region),
            format!("body.period={:?}", request.body.period),
            format!("body.baseline_period={:?}", request.body.baseline_period),
            format!("body.axes={:?}", request.body.axes),
            format!("body.anomaly_policy={:?}", request.body.anomaly_policy),
            format!(
                "body.minimum_gross_margin_bps={}",
                request.body.minimum_gross_margin_bps
            ),
            format!("body.data_class={}", request.body.data_class),
        ]
        .join("|"),
    }
}

fn report_record(report: FinopsReport) -> CloudFinopsReportRecord {
    CloudFinopsReportRecord {
        id: report.id.value.value,
        tenant_id: report.tenant_id.value,
        region: report.region.value.value,
        period_start_epoch_seconds: report.period.value.start_epoch_seconds,
        period_end_epoch_seconds: report.period.value.end_epoch_seconds,
        axes: report
            .axes
            .value
            .into_iter()
            .map(|axis| CloudFinopsAxisRef {
                value: axis_label_string(axis),
            })
            .collect(),
        axis_costs: report
            .axis_costs
            .value
            .into_iter()
            .map(axis_cost_record)
            .collect(),
        resource_costs: report
            .resource_costs
            .value
            .into_iter()
            .map(resource_cost_record)
            .collect(),
        anomalies: report
            .anomalies
            .value
            .into_iter()
            .map(anomaly_record)
            .collect(),
        recommendations: report
            .recommendations
            .value
            .into_iter()
            .map(recommendation_record)
            .collect(),
        total_cost: money_record(&report.total_cost.value),
        total_cost_of_revenue: money_record(&report.total_cost_of_revenue.value),
        gross_margin_bps: report.gross_margin_bps.value,
        minimum_gross_margin_bps: report.minimum_gross_margin_bps.value,
        data_class: report.data_class.value.label().to_string(),
        schema_version: report.schema_version.value,
    }
}

fn axis_cost_record(cost: AxisCostBreakdown) -> CloudFinopsAxisCostRecord {
    CloudFinopsAxisCostRecord {
        axis: axis_label_string(cost.axis),
        actual_cost: money_record(&cost.actual_cost),
        cost_of_revenue: money_record(&cost.cost_of_revenue),
        gross_margin_bps: cost.gross_margin_bps,
        budget: cost.budget.as_ref().map(money_record),
        budget_utilization_bps: cost.budget_utilization_bps,
    }
}

fn resource_cost_record(cost: ResourceCostBreakdown) -> CloudFinopsResourceCostRecord {
    CloudFinopsResourceCostRecord {
        resource_id: cost.resource_id.value,
        axis: axis_label_string(cost.axis),
        actual_cost: money_record(&cost.actual_cost),
        cost_of_revenue: money_record(&cost.cost_of_revenue),
        gross_margin_bps: cost.gross_margin_bps,
    }
}

fn anomaly_record(anomaly: CostAnomaly) -> CloudFinopsCostAnomalyRecord {
    CloudFinopsCostAnomalyRecord {
        kind: anomaly_kind_label(anomaly.kind).to_string(),
        axis: axis_label_string(anomaly.axis),
        resource_id: anomaly.resource_id.map(|resource_id| resource_id.value),
        actual_cost: money_record(&anomaly.actual_cost),
        baseline_cost: anomaly.baseline_cost.as_ref().map(money_record),
        threshold_bps: anomaly.threshold_bps,
    }
}

fn recommendation_record(recommendation: FinopsRecommendation) -> CloudFinopsRecommendationRecord {
    CloudFinopsRecommendationRecord {
        id: recommendation.id.value,
        kind: recommendation_kind_label(recommendation.kind).to_string(),
        axis: axis_label_string(recommendation.axis),
        resource_id: recommendation
            .resource_id
            .map(|resource_id| resource_id.value),
        evidence_anomaly: anomaly_kind_label(recommendation.evidence_anomaly).to_string(),
    }
}

fn money_record(money: &Money) -> CloudFinopsMoneyRecord {
    CloudFinopsMoneyRecord {
        currency: money.currency.value.clone(),
        minor_units: money.minor_units,
    }
}

fn axis_label(axis: AxisId) -> &'static str {
    match axis {
        AxisId::Saas => "saas",
        AxisId::Foundry => "foundry",
        AxisId::Cloud => "cloud",
        AxisId::Search => "search",
        AxisId::Ads => "ads",
        AxisId::Marketplace => "marketplace",
        AxisId::Vertical => "vertical",
    }
}

fn axis_label_string(axis: AxisId) -> String {
    axis_label(axis).to_string()
}

fn anomaly_kind_label(kind: CostAnomalyKind) -> &'static str {
    match kind {
        CostAnomalyKind::SpendSpike => "spend_spike",
        CostAnomalyKind::BudgetSoftLimit => "budget_soft_limit",
        CostAnomalyKind::BudgetHardLimit => "budget_hard_limit",
        CostAnomalyKind::MarginBelowTarget => "margin_below_target",
    }
}

fn recommendation_kind_label(kind: RecommendationKind) -> &'static str {
    match kind {
        RecommendationKind::InvestigateSpendSpike => "investigate_spend_spike",
        RecommendationKind::PurchaseCommitment => "purchase_commitment",
        RecommendationKind::DownsizeResource => "downsize_resource",
        RecommendationKind::ReviewRateCard => "review_rate_card",
    }
}

fn cloud_finops_status_kind(error: &CloudFinopsError) -> CloudFinopsReportApiStatusKind {
    match error {
        CloudFinopsError::DuplicateReport
        | CloudFinopsError::DuplicateAllocation
        | CloudFinopsError::DuplicateMeterEvent
        | CloudFinopsError::DuplicateBudget
        | CloudFinopsError::DuplicateRateCardLine => CloudFinopsReportApiStatusKind::Conflict,
        CloudFinopsError::ResourceTenantMismatch | CloudFinopsError::ResourceRegionMismatch => {
            CloudFinopsReportApiStatusKind::Forbidden
        }
        CloudFinopsError::NoReportData => CloudFinopsReportApiStatusKind::UnprocessableEntity,
        CloudFinopsError::InvalidReportId
        | CloudFinopsError::InvalidCostAllocationId
        | CloudFinopsError::InvalidBudgetId
        | CloudFinopsError::InvalidRecommendationId
        | CloudFinopsError::InvalidTenantId
        | CloudFinopsError::InvalidRegion
        | CloudFinopsError::InvalidResourceId
        | CloudFinopsError::InvalidRateCardRef
        | CloudFinopsError::InvalidRateCardLine
        | CloudFinopsError::InvalidCurrency
        | CloudFinopsError::InvalidPeriod
        | CloudFinopsError::InvalidAxisSet
        | CloudFinopsError::InvalidBudget
        | CloudFinopsError::InvalidBudgetThreshold
        | CloudFinopsError::InvalidAnomalyPolicy
        | CloudFinopsError::InvalidGrossMarginTarget
        | CloudFinopsError::InvalidDataClass
        | CloudFinopsError::InvalidMeterEvent
        | CloudFinopsError::MissingRateCardLine
        | CloudFinopsError::CurrencyMismatch
        | CloudFinopsError::CostOverflow
        | CloudFinopsError::NonIntegralCost
        | CloudFinopsError::NegativeGrossMargin => CloudFinopsReportApiStatusKind::BadRequest,
    }
}

fn cloud_finops_message(error: &CloudFinopsError) -> &'static str {
    match cloud_finops_status_kind(error) {
        CloudFinopsReportApiStatusKind::BadRequest => "Cloud FinOps rejected the request shape",
        CloudFinopsReportApiStatusKind::Unauthorized => "Cloud FinOps authentication is required",
        CloudFinopsReportApiStatusKind::Forbidden => "Cloud FinOps policy denied the request",
        CloudFinopsReportApiStatusKind::Conflict => "Cloud FinOps report already exists",
        CloudFinopsReportApiStatusKind::UnprocessableEntity => {
            "Cloud FinOps cannot produce a report for the requested inputs"
        }
    }
}

fn cloud_finops_issue(error: &CloudFinopsError) -> &'static str {
    match error {
        CloudFinopsError::InvalidReportId => "report id must use the finr_ prefix",
        CloudFinopsError::InvalidCostAllocationId => "allocation id must use the fca_ prefix",
        CloudFinopsError::InvalidBudgetId => "budget id must use the fbg_ prefix",
        CloudFinopsError::InvalidRecommendationId => "recommendation id must use the frec_ prefix",
        CloudFinopsError::InvalidTenantId => "tenant_id must use the ten_ prefix",
        CloudFinopsError::InvalidRegion => "region must be a supported cloud region code",
        CloudFinopsError::InvalidResourceId => "resource id must be canonical cloud resource id",
        CloudFinopsError::ResourceTenantMismatch => "resource tenant must match report tenant",
        CloudFinopsError::ResourceRegionMismatch => "resource region must match report region",
        CloudFinopsError::InvalidRateCardRef => "rate card reference must use the rate/ prefix",
        CloudFinopsError::InvalidRateCardLine => {
            "rate-card line must have a positive integral rate"
        }
        CloudFinopsError::InvalidCurrency => "currency must be a three-letter uppercase code",
        CloudFinopsError::InvalidPeriod => {
            "periods must be ordered and within the maximum report window"
        }
        CloudFinopsError::InvalidAxisSet => "axes must be non-empty and unique",
        CloudFinopsError::InvalidBudget => "budget must be positive and computable",
        CloudFinopsError::InvalidBudgetThreshold => "budget thresholds must be ordered and bounded",
        CloudFinopsError::InvalidAnomalyPolicy => "anomaly policy thresholds must be positive",
        CloudFinopsError::InvalidGrossMarginTarget => {
            "gross margin target must not exceed 10000 bps"
        }
        CloudFinopsError::InvalidDataClass => "FinOps report data must use a financial data class",
        CloudFinopsError::InvalidMeterEvent => "meter event must be valid and timestamped",
        CloudFinopsError::MissingRateCardLine => {
            "matching rate-card line is required for all meter units"
        }
        CloudFinopsError::CurrencyMismatch => "money values must use one currency",
        CloudFinopsError::CostOverflow => "cost computation overflowed",
        CloudFinopsError::NonIntegralCost => "rate-card pricing must produce integral minor units",
        CloudFinopsError::NegativeGrossMargin => "cost of revenue must not exceed actual cost",
        CloudFinopsError::DuplicateRateCardLine => "rate-card line already exists",
        CloudFinopsError::DuplicateAllocation => "allocation id already exists",
        CloudFinopsError::DuplicateMeterEvent => "meter event already exists",
        CloudFinopsError::DuplicateBudget => "budget already exists",
        CloudFinopsError::DuplicateReport => "report id already exists",
        CloudFinopsError::NoReportData => "no allocations match the requested report scope",
    }
}

fn detail(field: &str, issue: &str) -> CloudFinopsApiErrorDetail {
    CloudFinopsApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
