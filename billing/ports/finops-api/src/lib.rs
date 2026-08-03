//! Cloud FinOps report API boundary.
//!
//! This crate owns tenant/header/path/body normalization, idempotency, and
//! authenticated API projection before handing typed report generation requests
//! to the Cloud FinOps kernel.
//!
//! ## Modules
//!
//! - [`authz`] — fail-closed principal-verification + PDP authorization PORTS for
//!   the `cloud.finops.report` surface (AUTH-005 class; Wave-2 capability-billing
//!   remediation; ADR-0591). The report generation entry point
//!   [`generate_cloud_finops_report_from_api`] REQUIRES a verified principal and
//!   a PDP authorizer; the caller-supplied [`CloudFinopsApiAuthorization`] DTO no
//!   longer grants access — its `allowed_surfaces` self-assertion is demoted to a
//!   non-authoritative correlation hint.

pub mod authz;

use std::collections::BTreeMap;

use authz::{
    FinopsReportAuthorizationError, FinopsReportAuthorizer, FinopsReportResource,
    FinopsReportScope, VerifiedPrincipal,
};
use billing_domain::Money;
use billing_finops::{
    AnomalyPolicy, AxisCostBreakdown, CloudFinopsError, CloudFinopsLedger, CostAnomaly,
    CostAnomalyKind, FinopsPeriod, FinopsRecommendation, FinopsReport, FinopsReportRequest,
    RecommendationKind, ResourceCostBreakdown,
};
use data_boundary_kernel::{DataClass, parse_data_class_label};
use billing_metering::AxisId;

pub const CLOUD_FINOPS_REPORT_SURFACE: &str = "cloud.finops.report";

/// The reserved tenant id whose report is a PLATFORM-WIDE aggregate of every
/// tenant's cloud spend. A report targeting this tenant is presented to the PDP
/// as a [`authz::FinopsReportScope::Platform`] resource requiring platform-admin
/// authority — NOT a per-tenant resource (the #815 global-scope CRITICAL: never
/// let a tenant-finops-admin exfiltrate platform-wide spend by self-asserting the
/// platform tenant).
pub const PLATFORM_AGGREGATE_TENANT_ID: &str = "ten_platform";

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
    PrincipalUnverified,
    PathReportIdEmpty,
    ReportIdMismatch,
    TenantMismatch,
    VerifiedPrincipalMismatch,
    VerifiedTenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    PdpDenied,
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
            Self::PrincipalUnverified => "CLOUD_FINOPS_PRINCIPAL_UNVERIFIED",
            Self::PathReportIdEmpty => "CLOUD_FINOPS_PATH_REPORT_ID_EMPTY",
            Self::ReportIdMismatch => "CLOUD_FINOPS_REPORT_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_FINOPS_TENANT_MISMATCH",
            Self::VerifiedPrincipalMismatch => "CLOUD_FINOPS_VERIFIED_PRINCIPAL_MISMATCH",
            Self::VerifiedTenantMismatch => "CLOUD_FINOPS_VERIFIED_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "CLOUD_FINOPS_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "CLOUD_FINOPS_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "CLOUD_FINOPS_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::PdpDenied => "CLOUD_FINOPS_PDP_DENIED",
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

/// Caller-supplied authorization correlation, **NOT an authorization grant**
/// (ADR-0591).
///
/// Historically `allowed_surfaces` was a self-asserted grant: any caller could
/// set `allowed_surfaces = ["cloud.finops.report"]` and authorize itself to read
/// a tenant's cloud-spend report (the AUTH-005 forgeable-authz CRIT). That gap is
/// now closed: authorization is decided server-side by the
/// [`authz::FinopsReportAuthorizer`] PDP port bound to the report's TARGET tenant,
/// gated behind a [`authz::VerifiedPrincipal`].
///
/// These fields are retained only as a **non-authoritative correlation hint** for
/// log joins / fingerprinting. `decision_id` is a caller-supplied correlation id,
/// NOT a grant. `tenant_id` / `principal_id` are still cross-checked against the
/// verified identity (a mismatch is rejected), and `allowed_surfaces` no longer
/// grants anything — the PDP decision is authoritative.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY — cross-check only
    pub principal_id: String,          // data_class: INTERNAL_ONLY — cross-check only
    pub decision_id: String,           // data_class: INTERNAL_ONLY — correlation hint, NOT a grant
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY — NON-AUTHORITATIVE (no longer grants)
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
    pub min_absolute_delta_minor_units: u64, // data_class: FINANCIAL_REGULATED_CREDIT
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
    pub data: CloudFinopsReportRecord, // data_class: FINANCIAL_REGULATED_CREDIT
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
    pub minor_units: u64, // data_class: FINANCIAL_REGULATED_CREDIT
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsReportRecord {
    pub id: String,                                            // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                     // data_class: INTERNAL_ONLY
    pub region: String,                                        // data_class: PUBLIC
    pub period_start_epoch_seconds: u64,                       // data_class: INTERNAL_ONLY
    pub period_end_epoch_seconds: u64,                         // data_class: INTERNAL_ONLY
    pub axes: Vec<CloudFinopsAxisRef>,                         // data_class: INTERNAL_ONLY
    pub axis_costs: Vec<CloudFinopsAxisCostRecord>, // data_class: FINANCIAL_REGULATED_CREDIT
    pub resource_costs: Vec<CloudFinopsResourceCostRecord>, // data_class: FINANCIAL_REGULATED_CREDIT
    pub anomalies: Vec<CloudFinopsCostAnomalyRecord>, // data_class: FINANCIAL_REGULATED_CREDIT
    pub recommendations: Vec<CloudFinopsRecommendationRecord>, // data_class: INTERNAL_ONLY
    pub total_cost: CloudFinopsMoneyRecord,           // data_class: FINANCIAL_REGULATED_CREDIT
    pub total_cost_of_revenue: CloudFinopsMoneyRecord, // data_class: FINANCIAL_REGULATED_CREDIT
    pub gross_margin_bps: u16,                        // data_class: INTERNAL_ONLY
    pub minimum_gross_margin_bps: u16,                // data_class: INTERNAL_ONLY
    pub data_class: String,                           // data_class: INTERNAL_ONLY
    pub schema_version: u32,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsAxisCostRecord {
    pub axis: String,                            // data_class: INTERNAL_ONLY
    pub actual_cost: CloudFinopsMoneyRecord,     // data_class: FINANCIAL_REGULATED_CREDIT
    pub cost_of_revenue: CloudFinopsMoneyRecord, // data_class: FINANCIAL_REGULATED_CREDIT
    pub gross_margin_bps: u16,                   // data_class: INTERNAL_ONLY
    pub budget: Option<CloudFinopsMoneyRecord>,  // data_class: FINANCIAL_REGULATED_CREDIT
    pub budget_utilization_bps: Option<u16>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsResourceCostRecord {
    pub resource_id: String,                     // data_class: INTERNAL_ONLY
    pub axis: String,                            // data_class: INTERNAL_ONLY
    pub actual_cost: CloudFinopsMoneyRecord,     // data_class: FINANCIAL_REGULATED_CREDIT
    pub cost_of_revenue: CloudFinopsMoneyRecord, // data_class: FINANCIAL_REGULATED_CREDIT
    pub gross_margin_bps: u16,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFinopsCostAnomalyRecord {
    pub kind: String,                                  // data_class: INTERNAL_ONLY
    pub axis: String,                                  // data_class: INTERNAL_ONLY
    pub resource_id: Option<String>,                   // data_class: INTERNAL_ONLY
    pub actual_cost: CloudFinopsMoneyRecord,           // data_class: FINANCIAL_REGULATED_CREDIT
    pub baseline_cost: Option<CloudFinopsMoneyRecord>, // data_class: FINANCIAL_REGULATED_CREDIT
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
    /// 401 — no verifiable caller credential produced a [`VerifiedPrincipal`].
    PrincipalUnverified,
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
    /// 403 — the request's self-asserted principal id does not match the
    /// VERIFIED principal id (identity-substitution attempt).
    VerifiedPrincipalMismatch {
        verified_principal_id: String,
        request_principal_id: String,
    },
    /// 403 — the request's self-asserted tenant does not match the VERIFIED
    /// tenant (cross-tenant operate-as attempt).
    VerifiedTenantMismatch {
        verified_tenant_id: String,
        request_tenant_id: String,
    },
    /// 403 — the PDP denied or refused (fail-closed) the
    /// `cloud.finops.report` decision on the TARGET resource.
    PdpDenied {
        surface: String,
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
            Self::PrincipalUnverified => CloudFinopsApiErrorCode::PrincipalUnverified,
            Self::EmptyPathReportId => CloudFinopsApiErrorCode::PathReportIdEmpty,
            Self::ReportIdMismatch { .. } => CloudFinopsApiErrorCode::ReportIdMismatch,
            Self::TenantMismatch { .. } => CloudFinopsApiErrorCode::TenantMismatch,
            Self::VerifiedPrincipalMismatch { .. } => {
                CloudFinopsApiErrorCode::VerifiedPrincipalMismatch
            }
            Self::VerifiedTenantMismatch { .. } => CloudFinopsApiErrorCode::VerifiedTenantMismatch,
            Self::PdpDenied { .. } => CloudFinopsApiErrorCode::PdpDenied,
            Self::EmptyAuthorizationDecisionId => {
                CloudFinopsApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudFinopsApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudFinopsApiErrorCode::AuthorizationPrincipalMismatch
            }
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
            Self::EmptyPrincipalId | Self::PrincipalUnverified => {
                CloudFinopsReportApiStatusKind::Unauthorized
            }
            Self::TenantMismatch { .. }
            | Self::VerifiedPrincipalMismatch { .. }
            | Self::VerifiedTenantMismatch { .. }
            | Self::PdpDenied { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. } => {
                CloudFinopsReportApiStatusKind::Forbidden
            }
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
            Self::PrincipalUnverified => {
                "A verified caller principal is required to read a Cloud FinOps report"
            }
            Self::EmptyPathReportId => "Path report id is required",
            Self::ReportIdMismatch { .. } => "Path and body report ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::VerifiedPrincipalMismatch { .. } => {
                "Request principal id must match the verified caller principal"
            }
            Self::VerifiedTenantMismatch { .. } => {
                "Request tenant must match the verified caller tenant"
            }
            Self::PdpDenied { .. } => {
                "The verified principal is not authorized for the requested Cloud FinOps report"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
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
            Self::PrincipalUnverified => vec![detail(
                "header.Authorization",
                "must present a verifiable caller credential",
            )],
            Self::EmptyPathReportId => vec![detail("path.report_id", "must be non-empty")],
            Self::ReportIdMismatch { .. } => {
                vec![detail("id", "path report_id and body id must match")]
            }
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
            )],
            Self::VerifiedPrincipalMismatch { .. } => vec![detail(
                "principal.principal_id",
                "must match the verified caller principal id",
            )],
            Self::VerifiedTenantMismatch { .. } => vec![detail(
                "principal.tenant_id",
                "must match the verified caller tenant",
            )],
            Self::PdpDenied { .. } => vec![detail(
                "authorization",
                "denied by the cloud.finops.report policy decision on the target tenant",
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

/// Structural request validation: boundary headers, path/body binding, tenant
/// consistency, and label shapes.
///
/// This is the shape gate. It does NOT make an authorization decision — the
/// caller-supplied [`CloudFinopsApiAuthorization`] is cross-checked for internal
/// consistency only and never GRANTS access. The authoritative authorization is
/// the [`authz::VerifiedPrincipal`] cross-check + PDP decision performed in
/// [`generate_cloud_finops_report_from_api`].
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
    validate_authorization_correlation(&request.principal, &request.authorization)?;
    validate_body_labels(&request.body)
}

/// Generate a Cloud FinOps report — FAIL-CLOSED (ADR-0591).
///
/// ## `verified` — unforgeable identity + active cross-check
///
/// The first argument is a [`authz::VerifiedPrincipal`]. Its fields are private
/// and its constructor is `pub(crate)`, callable only by a
/// [`authz::PrincipalVerifier`] inside this crate — external crates CANNOT build
/// one by struct literal or any public API, so no in-process caller can reach
/// report generation without completing principal verification first. The
/// function then ACTIVELY cross-checks the request's self-asserted
/// `principal.principal_id` / `principal.tenant_id` against the verified identity
/// and rejects any mismatch (403) — a verified principal of tenant A may not
/// operate as tenant B.
///
/// ## `authorizer` — server-side PDP decision bound to the TARGET tenant
///
/// Authorization is decided by the [`authz::FinopsReportAuthorizer`] PDP port,
/// not by the caller-supplied `allowed_surfaces` (which no longer grants
/// anything). The PDP resource is the report's TARGET tenant derived from the
/// validated request — a cross-tenant read is DENIABLE at the PDP. The caller's
/// own tenant is never flattened onto the resource (the #817 blast-radius
/// lesson). Any deny/refusal/fault maps to 403 (fail-closed).
///
/// The gate runs BEFORE the idempotency ledger and the kernel call: a denied or
/// unverified request never mutates the ledger and never reads spend data.
pub fn generate_cloud_finops_report_from_api(
    verified: &VerifiedPrincipal,
    authorizer: &dyn FinopsReportAuthorizer,
    ledger: &mut CloudFinopsLedger,
    idempotency_ledger: &mut CloudFinopsReportGenerateIdempotencyLedger,
    request: CloudFinopsReportApiRequest,
) -> Result<CloudFinopsReportGenerateSuccessResponse, CloudFinopsReportApiError> {
    // (1) Structural shape validation (headers, path/body binding, labels).
    validate_cloud_finops_report_request(&request)?;

    // (2) UNFORGEABLE identity cross-check — the verified principal is
    // authoritative. Reject any attempt to assert a different principal/tenant
    // than the one the verifier bound (identity-substitution / cross-tenant
    // operate-as). The request's tenant fields were already cross-bound equal by
    // `validate_tenant_binding`, so checking the body tenant binds the whole set.
    enforce_verified_identity(verified, &request)?;

    // (3) SERVER-SIDE PDP decision bound to the TARGET tenant of the report.
    // The resource tenant comes from the validated request body (a trusted source
    // after the verified cross-check), NOT echoed from a caller header and NOT
    // flattened to the caller's own tenant. Cross-tenant is deniable here; a
    // platform-wide aggregate is presented as a Platform resource needing
    // platform-admin authority.
    let resource = finops_report_resource(&request);
    authorizer
        .ensure_authorized(verified, &resource)
        .map_err(pdp_error)?;

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

/// Cross-check the caller-supplied authorization correlation for internal
/// consistency. This is **NOT** an authorization decision (ADR-0591): the
/// `allowed_surfaces` self-assertion no longer grants access — the authoritative
/// decision is the [`authz::FinopsReportAuthorizer`] PDP call in
/// [`generate_cloud_finops_report_from_api`]. We still reject an empty
/// `decision_id` (it is a required correlation id) and an inconsistent
/// tenant/principal (a forged correlation tuple is a request defect), so the
/// audit/log join is coherent.
fn validate_authorization_correlation(
    principal: &CloudFinopsApiPrincipal,
    authorization: &CloudFinopsApiAuthorization,
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
    Ok(())
}

/// Cross-check the request's self-asserted identity against the VERIFIED
/// principal. The verified identity is authoritative — a caller may not assert a
/// different principal id (identity-substitution) or a different tenant
/// (cross-tenant operate-as) than the verifier bound. Both → 403.
fn enforce_verified_identity(
    verified: &VerifiedPrincipal,
    request: &CloudFinopsReportApiRequest,
) -> Result<(), CloudFinopsReportApiError> {
    if request.principal.principal_id != verified.principal_id() {
        return Err(CloudFinopsReportApiError::VerifiedPrincipalMismatch {
            verified_principal_id: verified.principal_id().to_string(),
            request_principal_id: request.principal.principal_id.clone(),
        });
    }
    // `validate_tenant_binding` already bound header/principal/body tenants equal;
    // checking the principal tenant against the verified tenant binds the whole
    // set (including the TARGET report tenant in `body.tenant_id`).
    if request.principal.tenant_id != verified.tenant_id() {
        return Err(CloudFinopsReportApiError::VerifiedTenantMismatch {
            verified_tenant_id: verified.tenant_id().to_string(),
            request_tenant_id: request.principal.tenant_id.clone(),
        });
    }
    Ok(())
}

/// Build the [`FinopsReportResource`] for the PDP decision from the validated
/// request, carrying the scope EXPLICITLY so the PDP sees the true blast radius.
///
/// The tenant comes from the request body (a trusted source after
/// [`validate_tenant_binding`] + [`enforce_verified_identity`] bound it equal to
/// the verified tenant), NOT echoed from an attacker-controlled header and NOT
/// flattened. A platform-wide aggregate report (the reserved `ten_platform`
/// target) is presented as a [`FinopsReportScope::Platform`] resource requiring
/// platform-admin authority, so a tenant-finops-admin cannot exfiltrate
/// platform-wide spend by self-asserting the platform tenant.
fn finops_report_resource(request: &CloudFinopsReportApiRequest) -> FinopsReportResource {
    let target_tenant = request.body.tenant_id.clone();
    if target_tenant == PLATFORM_AGGREGATE_TENANT_ID {
        FinopsReportResource {
            report_id: request.body.id.clone(),
            scope: FinopsReportScope::Platform,
            tenant_id: String::new(),
        }
    } else {
        FinopsReportResource {
            report_id: request.body.id.clone(),
            scope: FinopsReportScope::Tenant,
            tenant_id: target_tenant,
        }
    }
}

/// Map a fail-closed PDP outcome (deny or refusal/fault) to the 403
/// [`CloudFinopsReportApiError::PdpDenied`]. Both Denied and Refused collapse to
/// a single opaque 403 so probing cannot distinguish "policy says no" from "PDP
/// unavailable".
fn pdp_error(_err: FinopsReportAuthorizationError) -> CloudFinopsReportApiError {
    CloudFinopsReportApiError::PdpDenied {
        surface: CLOUD_FINOPS_REPORT_SURFACE.to_string(),
    }
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
