//! Tenant-scoped Trust Center evidence API/read-model boundary.
//!
//! This crate implements the TRUSTCENTER-API-001 first slice from
//! `specs/trust-center-compliance-evidence-portal.json#api_contract` as a
//! transport-neutral API/read-model contract. It deliberately does not publish
//! certifications, expose raw scanner output, expose operator-only evidence, or
//! claim a deployed listener. Tenant scope comes from trusted boundary +
//! principal context; payload/query `tenant_id` assertions are checked but are
//! never authority.
//!
//! non_claim: in-memory read-model fixture only; no storage adapter, no live
//! HTTP listener, no export package assembly, and no external certification
//! display workflow.
// ADR-0083 Tier 3: tests legitimately use unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

pub const TRUST_CENTER_SCHEMA_VERSION: u32 = 1;
pub const TRUST_CENTER_SERVICE_NAME: &str = "trust-center";
pub const TENANT_ID_PREFIX: &str = "ten_";

pub const TRUST_CENTER_EVIDENCE_INDEX_PATH: &str = "/trust-center/v1/evidence";
pub const TRUST_CENTER_EVIDENCE_DETAIL_PATH: &str = "/trust-center/v1/evidence/{evidence_id}";
pub const TRUST_CENTER_SBOM_VEX_PATH: &str = "/trust-center/v1/sbom-vex";
pub const TRUST_CENTER_CONTROL_FRESHNESS_PATH: &str = "/trust-center/v1/controls/freshness";
pub const TRUST_CENTER_COMPLIANCE_PACKS_PATH: &str = "/trust-center/v1/compliance-packs";
pub const TRUST_CENTER_EXPORTS_PATH: &str = "/trust-center/v1/exports";
pub const TRUST_CENTER_ACCESS_AUDIT_PATH: &str = "/trust-center/v1/access-audit";

pub const TRUST_CENTER_EVIDENCE_INDEX_SURFACE: &str = "trust-center.evidence.index.read";
pub const TRUST_CENTER_EVIDENCE_DETAIL_SURFACE: &str = "trust-center.evidence.detail.read";
pub const TRUST_CENTER_SBOM_VEX_SURFACE: &str = "trust-center.sbom-vex.read";
pub const TRUST_CENTER_CONTROL_FRESHNESS_SURFACE: &str = "trust-center.controls.freshness.read";
pub const TRUST_CENTER_COMPLIANCE_PACKS_SURFACE: &str = "trust-center.compliance-packs.read";
pub const TRUST_CENTER_EXPORT_REQUEST_SURFACE: &str = "trust-center.exports.request";
pub const TRUST_CENTER_ACCESS_AUDIT_SURFACE: &str = "trust-center.access-audit.read";
pub const TRUST_CENTER_GRANT_WRITE_SURFACE: &str = "trust-center.access-grants.write";
pub const TRUST_CENTER_EXPORT_DOWNLOAD_SURFACE: &str = "trust-center.exports.download";
pub const TRUST_CENTER_PUBLISHABILITY_SURFACE: &str = "trust-center.publishability.write";

pub const TRUST_CENTER_EVIDENCE_INDEX_RECORD_TYPE: &str = "trust_center_evidence_index.v1";
pub const TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE: &str = "trust_center_evidence_item.v1";
pub const TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE: &str = "trust_center_control_freshness.v1";
pub const TRUST_CENTER_SBOM_VEX_RECORD_TYPE: &str = "trust_center_sbom_vex_view.v1";
pub const TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE: &str = "trust_center_compliance_pack_view.v1";
pub const TRUST_CENTER_EXPORT_REQUEST_RECORD_TYPE: &str = "trust_center_export_request.v1";
pub const TRUST_CENTER_ACCESS_AUDIT_RECORD_TYPE: &str = "trust_center_access_audit.v1";
pub const TRUST_CENTER_PUBLISHABILITY_DECISION_RECORD_TYPE: &str =
    "trust_center_publishability_decision.v1";

const DEFAULT_PAGE_SIZE: usize = 100;
const MAX_PAGE_SIZE: usize = 1_000;
const TRUST_CENTER_CURSOR_PREFIX: &str = "tc_cur/";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustCenterApiStatus {
    Ok,
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl TrustCenterApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
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
pub enum TrustCenterApiErrorCode {
    RequestIdEmpty,
    TrustedTenantMissing,
    TrustedTenantMalformed,
    PrincipalMissing,
    PrincipalIdEmpty,
    PrincipalTenantMismatch,
    TenantAssertionMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    PurposeDenied,
    RoleDenied,
    DataClassDenied,
    PublishabilityDenied,
    OperatorOnlyDetailDenied,
    EvidenceNotFound,
    EvidenceTenantMismatch,
    EvidenceNotPublishable,
    EvidenceNotFresh,
    MissingEvidence,
    InvalidPageSize,
    InvalidCursor,
    ExportRequiresTenantAdmin,
    InvalidRecordShape,
}

impl TrustCenterApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "TRUST_CENTER_REQUEST_ID_EMPTY",
            Self::TrustedTenantMissing => "TRUST_CENTER_TRUSTED_TENANT_MISSING",
            Self::TrustedTenantMalformed => "TRUST_CENTER_TRUSTED_TENANT_MALFORMED",
            Self::PrincipalMissing => "TRUST_CENTER_PRINCIPAL_MISSING",
            Self::PrincipalIdEmpty => "TRUST_CENTER_PRINCIPAL_ID_EMPTY",
            Self::PrincipalTenantMismatch => "TRUST_CENTER_PRINCIPAL_TENANT_MISMATCH",
            Self::TenantAssertionMismatch => "TRUST_CENTER_TENANT_ASSERTION_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "TRUST_CENTER_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "TRUST_CENTER_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "TRUST_CENTER_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "TRUST_CENTER_AUTHORIZATION_DENIED",
            Self::PurposeDenied => "TRUST_CENTER_PURPOSE_DENIED",
            Self::RoleDenied => "TRUST_CENTER_ROLE_DENIED",
            Self::DataClassDenied => "TRUST_CENTER_DATA_CLASS_DENIED",
            Self::PublishabilityDenied => "TRUST_CENTER_PUBLISHABILITY_DENIED",
            Self::OperatorOnlyDetailDenied => "TRUST_CENTER_OPERATOR_ONLY_DETAIL_DENIED",
            Self::EvidenceNotFound => "TRUST_CENTER_EVIDENCE_NOT_FOUND",
            Self::EvidenceTenantMismatch => "TRUST_CENTER_EVIDENCE_TENANT_MISMATCH",
            Self::EvidenceNotPublishable => "TRUST_CENTER_EVIDENCE_NOT_PUBLISHABLE",
            Self::EvidenceNotFresh => "TRUST_CENTER_EVIDENCE_NOT_FRESH",
            Self::MissingEvidence => "TRUST_CENTER_MISSING_EVIDENCE",
            Self::InvalidPageSize => "TRUST_CENTER_INVALID_PAGE_SIZE",
            Self::InvalidCursor => "TRUST_CENTER_INVALID_CURSOR",
            Self::ExportRequiresTenantAdmin => "TRUST_CENTER_EXPORT_REQUIRES_TENANT_ADMIN",
            Self::InvalidRecordShape => "TRUST_CENTER_INVALID_RECORD_SHAPE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrustCenterApiError {
    EmptyRequestId,
    MissingTrustedTenant,
    MalformedTrustedTenant {
        tenant_id: String,
    },
    MissingPrincipal,
    EmptyPrincipalId,
    PrincipalTenantMismatch {
        trusted_tenant_id: String,
        principal_tenant_id: String,
    },
    TenantAssertionMismatch {
        trusted_tenant_id: String,
        asserted_tenant_id: String,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        trusted_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    AuthorizationDenied {
        endpoint: String,
    },
    PurposeDenied {
        required: TrustCenterPurpose,
        supplied: TrustCenterPurpose,
    },
    RoleDenied {
        role: TrustCenterRole,
        endpoint: String,
    },
    DataClassDenied {
        data_class: TrustCenterDataClass,
    },
    PublishabilityDenied {
        publishability_state: TrustCenterPublishabilityState,
    },
    OperatorOnlyDetailDenied,
    EvidenceNotFound {
        evidence_id: String,
    },
    EvidenceTenantMismatch {
        trusted_tenant_id: String,
        evidence_tenant_id: String,
    },
    EvidenceNotPublishable {
        publishability_state: TrustCenterPublishabilityState,
    },
    EvidenceNotFresh {
        record_id: String,
        freshness_state: TrustCenterFreshnessState,
    },
    MissingEvidence {
        endpoint: String,
    },
    InvalidPageSize {
        page_size: usize,
    },
    InvalidCursor {
        cursor: String,
    },
    ExportRequiresTenantAdmin,
    InvalidRecordShape {
        reason: String,
    },
}

impl TrustCenterApiError {
    pub fn status(&self) -> TrustCenterApiStatus {
        match self {
            Self::EmptyRequestId
            | Self::MissingTrustedTenant
            | Self::MalformedTrustedTenant { .. }
            | Self::TenantAssertionMismatch { .. }
            | Self::InvalidPageSize { .. }
            | Self::InvalidCursor { .. }
            | Self::InvalidRecordShape { .. } => TrustCenterApiStatus::BadRequest,
            Self::MissingPrincipal | Self::EmptyPrincipalId => TrustCenterApiStatus::Unauthorized,
            Self::EvidenceNotFound { .. } => TrustCenterApiStatus::NotFound,
            Self::EvidenceNotFresh { .. } | Self::MissingEvidence { .. } => {
                TrustCenterApiStatus::UnprocessableEntity
            }
            Self::PrincipalTenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::PurposeDenied { .. }
            | Self::RoleDenied { .. }
            | Self::DataClassDenied { .. }
            | Self::PublishabilityDenied { .. }
            | Self::OperatorOnlyDetailDenied
            | Self::EvidenceTenantMismatch { .. }
            | Self::EvidenceNotPublishable { .. }
            | Self::ExportRequiresTenantAdmin => TrustCenterApiStatus::Forbidden,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> TrustCenterApiErrorCode {
        match self {
            Self::EmptyRequestId => TrustCenterApiErrorCode::RequestIdEmpty,
            Self::MissingTrustedTenant => TrustCenterApiErrorCode::TrustedTenantMissing,
            Self::MalformedTrustedTenant { .. } => TrustCenterApiErrorCode::TrustedTenantMalformed,
            Self::MissingPrincipal => TrustCenterApiErrorCode::PrincipalMissing,
            Self::EmptyPrincipalId => TrustCenterApiErrorCode::PrincipalIdEmpty,
            Self::PrincipalTenantMismatch { .. } => {
                TrustCenterApiErrorCode::PrincipalTenantMismatch
            }
            Self::TenantAssertionMismatch { .. } => {
                TrustCenterApiErrorCode::TenantAssertionMismatch
            }
            Self::EmptyAuthorizationDecisionId => {
                TrustCenterApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                TrustCenterApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                TrustCenterApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => TrustCenterApiErrorCode::AuthorizationDenied,
            Self::PurposeDenied { .. } => TrustCenterApiErrorCode::PurposeDenied,
            Self::RoleDenied { .. } => TrustCenterApiErrorCode::RoleDenied,
            Self::DataClassDenied { .. } => TrustCenterApiErrorCode::DataClassDenied,
            Self::PublishabilityDenied { .. } => TrustCenterApiErrorCode::PublishabilityDenied,
            Self::OperatorOnlyDetailDenied => TrustCenterApiErrorCode::OperatorOnlyDetailDenied,
            Self::EvidenceNotFound { .. } => TrustCenterApiErrorCode::EvidenceNotFound,
            Self::EvidenceTenantMismatch { .. } => TrustCenterApiErrorCode::EvidenceTenantMismatch,
            Self::EvidenceNotPublishable { .. } => TrustCenterApiErrorCode::EvidenceNotPublishable,
            Self::EvidenceNotFresh { .. } => TrustCenterApiErrorCode::EvidenceNotFresh,
            Self::MissingEvidence { .. } => TrustCenterApiErrorCode::MissingEvidence,
            Self::InvalidPageSize { .. } => TrustCenterApiErrorCode::InvalidPageSize,
            Self::InvalidCursor { .. } => TrustCenterApiErrorCode::InvalidCursor,
            Self::ExportRequiresTenantAdmin => TrustCenterApiErrorCode::ExportRequiresTenantAdmin,
            Self::InvalidRecordShape { .. } => TrustCenterApiErrorCode::InvalidRecordShape,
        }
    }
}

impl fmt::Display for TrustCenterApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code().as_str())
    }
}

impl std::error::Error for TrustCenterApiError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustCenterRole {
    TenantAdmin,
    SecurityComplianceReviewer,
    OyatieOperator,
    Auditor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustCenterPurpose {
    EvidenceRead,
    SecurityEvidenceRead,
    ControlEvidenceRead,
    ComplianceRead,
    ExportRequest,
    AccessAuditRead,
    GrantManagement,
    Download,
    PublishabilityReview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrustCenterDataClass {
    PublicStatus,
    TenantTrustEvidence,
    RegulatedExportEvidence,
    OperatorSecurityInternal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustCenterClaimTier {
    TargetNonClaim,
    SpecReady,
    MechanicallyEnforced,
    ProductionReady,
    HyperscalerGrade,
    ExternallyCertified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustCenterFreshnessState {
    Current,
    AgingWarning,
    Stale,
    Missing,
    NotApplicableWithPolicyReason,
    BlockedPendingReview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustCenterPublishabilityState {
    PublishableCustomerSafe,
    PublishableSummaryOnly,
    TenantAdminOnly,
    OperatorOnly,
    BlockedMissingEvidence,
    BlockedStaleEvidence,
    BlockedSecurityPrivacyReview,
    NotApplicableWithPolicyReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustCenterAccessAuditEventKind {
    EvidenceIndexViewed,
    EvidenceItemViewed,
    EvidenceExportRequested,
    EvidenceExportApproved,
    EvidenceExportDownloaded,
    AccessGrantCreated,
    AccessGrantRevoked,
    PublishabilityStateChanged,
    RedactionPolicyApplied,
    ExceptionDisplayed,
}

impl TrustCenterAccessAuditEventKind {
    pub const fn as_event_type(self) -> &'static str {
        match self {
            Self::EvidenceIndexViewed => "trust_center.evidence_index_viewed",
            Self::EvidenceItemViewed => "trust_center.evidence_item_viewed",
            Self::EvidenceExportRequested => "trust_center.evidence_export_requested",
            Self::EvidenceExportApproved => "trust_center.evidence_export_approved",
            Self::EvidenceExportDownloaded => "trust_center.evidence_export_downloaded",
            Self::AccessGrantCreated => "trust_center.access_grant_created",
            Self::AccessGrantRevoked => "trust_center.access_grant_revoked",
            Self::PublishabilityStateChanged => "trust_center.publishability_state_changed",
            Self::RedactionPolicyApplied => "trust_center.redaction_policy_applied",
            Self::ExceptionDisplayed => "trust_center.exception_displayed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterApiRoute {
    pub method: &'static str,
    pub path: &'static str,
    pub endpoint: &'static str,
    pub response_records: Vec<&'static str>,
}

pub fn trust_center_api_routes() -> Vec<TrustCenterApiRoute> {
    vec![
        TrustCenterApiRoute {
            method: "GET",
            path: TRUST_CENTER_EVIDENCE_INDEX_PATH,
            endpoint: TRUST_CENTER_EVIDENCE_INDEX_SURFACE,
            response_records: vec![TRUST_CENTER_EVIDENCE_INDEX_RECORD_TYPE],
        },
        TrustCenterApiRoute {
            method: "GET",
            path: TRUST_CENTER_EVIDENCE_DETAIL_PATH,
            endpoint: TRUST_CENTER_EVIDENCE_DETAIL_SURFACE,
            response_records: vec![TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE],
        },
        TrustCenterApiRoute {
            method: "GET",
            path: TRUST_CENTER_SBOM_VEX_PATH,
            endpoint: TRUST_CENTER_SBOM_VEX_SURFACE,
            response_records: vec![TRUST_CENTER_SBOM_VEX_RECORD_TYPE],
        },
        TrustCenterApiRoute {
            method: "GET",
            path: TRUST_CENTER_CONTROL_FRESHNESS_PATH,
            endpoint: TRUST_CENTER_CONTROL_FRESHNESS_SURFACE,
            response_records: vec![TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE],
        },
        TrustCenterApiRoute {
            method: "GET",
            path: TRUST_CENTER_COMPLIANCE_PACKS_PATH,
            endpoint: TRUST_CENTER_COMPLIANCE_PACKS_SURFACE,
            response_records: vec![TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE],
        },
        TrustCenterApiRoute {
            method: "POST",
            path: TRUST_CENTER_EXPORTS_PATH,
            endpoint: TRUST_CENTER_EXPORT_REQUEST_SURFACE,
            response_records: vec![TRUST_CENTER_EXPORT_REQUEST_RECORD_TYPE],
        },
        TrustCenterApiRoute {
            method: "GET",
            path: TRUST_CENTER_ACCESS_AUDIT_PATH,
            endpoint: TRUST_CENTER_ACCESS_AUDIT_SURFACE,
            response_records: vec![TRUST_CENTER_ACCESS_AUDIT_RECORD_TYPE],
        },
    ]
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterCommonFields {
    pub record_id: String,
    pub record_type: String,
    pub schema_version: u32,
    pub tenant_id: String,
    pub audience_id: String,
    pub source_system: String,
    pub source_record_ref: String,
    pub evidence_class: String,
    pub data_class: TrustCenterDataClass,
    pub claim_tier: TrustCenterClaimTier,
    pub freshness_state: TrustCenterFreshnessState,
    pub publishability_state: TrustCenterPublishabilityState,
    pub redaction_policy_id: String,
    pub audit_event_ref: String,
    pub created_at_trusted: String,
    pub expires_at_trusted_or_retention_until: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterEvidenceIndexRecord {
    #[serde(flatten)]
    pub common: TrustCenterCommonFields,
    pub title: String,
    pub customer_safe_summary: String,
    pub compliance_pack_ids: Vec<String>,
    pub service_ids: Vec<String>,
    pub last_updated_trusted: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterEvidenceItemRecord {
    #[serde(flatten)]
    pub common: TrustCenterCommonFields,
    pub title: String,
    pub customer_safe_summary: String,
    pub source_links: Vec<String>,
    pub compliance_pack_ids: Vec<String>,
    pub service_ids: Vec<String>,
    pub redacted_fields: Vec<String>,
    pub operator_only_detail_present: bool,
    pub raw_operator_payload_exposed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterControlFreshnessRecord {
    #[serde(flatten)]
    pub common: TrustCenterCommonFields,
    pub control_id: String,
    pub lane_id: String,
    pub service_id: Option<String>,
    pub compliance_pack_ids: Vec<String>,
    pub last_observed_at_trusted: String,
    pub stale_after_trusted: String,
    pub source_evidence_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterSbomVexViewRecord {
    #[serde(flatten)]
    pub common: TrustCenterCommonFields,
    pub artifact_ref: String,
    pub signed_sbom_ref: Option<String>,
    pub vex_ref: Option<String>,
    pub vulnerability_status_counts: BTreeMap<String, u32>,
    pub exception_refs: Vec<String>,
    pub remediation_sla_class: String,
    pub raw_scanner_output_exposed: bool,
    pub exploit_detail_exposed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterCompliancePackViewRecord {
    #[serde(flatten)]
    pub common: TrustCenterCommonFields,
    pub compliance_pack_id: String,
    pub version: String,
    pub regulator_references: Vec<String>,
    pub data_classes: Vec<TrustCenterDataClass>,
    pub residency_summary: String,
    pub retention_days: u32,
    pub dr_floor_ref: Option<String>,
    pub breach_workflow_ref: Option<String>,
    pub activated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterExportRequestRecord {
    #[serde(flatten)]
    pub common: TrustCenterCommonFields,
    pub export_request_id: String,
    pub requested_by_principal_id: String,
    pub purpose: String,
    pub framework: String,
    pub time_window_start_trusted: String,
    pub time_window_end_trusted: String,
    pub evidence_ids: Vec<String>,
    pub approval_state: String,
    pub manifest_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterAccessAuditRecord {
    #[serde(flatten)]
    pub common: TrustCenterCommonFields,
    pub event_type: String,
    pub actor_principal_id: String,
    pub actor_role: TrustCenterRole,
    pub action: String,
    pub target_record_id: Option<String>,
    pub granted: bool,
    pub occurred_at_trusted: String,
    pub decision_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrustCenterPublishabilityDecisionRecord {
    #[serde(flatten)]
    pub common: TrustCenterCommonFields,
    pub decision_id: String,
    pub evidence_id: String,
    pub previous_state: TrustCenterPublishabilityState,
    pub new_state: TrustCenterPublishabilityState,
    pub reason: String,
    pub decided_by_principal_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterBoundaryContext {
    pub request_id: String,
    pub tenant_id: String,
    pub occurred_at_trusted: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterPrincipal {
    pub tenant_id: String,
    pub principal_id: String,
    pub role: TrustCenterRole,
    pub purpose: TrustCenterPurpose,
    pub audience_id: String,
    pub access_grant_id: Option<String>,
    pub expires_at_trusted: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterAuthorizationDecision {
    pub tenant_id: String,
    pub principal_id: String,
    pub decision_id: String,
    pub allowed_endpoints: Vec<String>,
    pub allowed_purposes: Vec<TrustCenterPurpose>,
    pub allowed_data_classes: Vec<TrustCenterDataClass>,
    pub allowed_publishability_states: Vec<TrustCenterPublishabilityState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterApiRequest<T> {
    pub boundary: TrustCenterBoundaryContext,
    pub principal: Option<TrustCenterPrincipal>,
    pub authorization: TrustCenterAuthorizationDecision,
    pub payload: T,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustCenterPage {
    pub cursor: Option<String>,
    pub page_size: Option<usize>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustCenterEvidenceIndexQuery {
    pub asserted_tenant_id: Option<String>,
    pub evidence_class: Option<String>,
    pub compliance_pack_id: Option<String>,
    pub source_system: Option<String>,
    pub freshness_state: Option<TrustCenterFreshnessState>,
    pub claim_tier: Option<TrustCenterClaimTier>,
    pub page: TrustCenterPage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterEvidenceDetailQuery {
    pub asserted_tenant_id: Option<String>,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustCenterControlFreshnessQuery {
    pub asserted_tenant_id: Option<String>,
    pub compliance_pack_id: Option<String>,
    pub service_id: Option<String>,
    pub page: TrustCenterPage,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustCenterSbomVexQuery {
    pub asserted_tenant_id: Option<String>,
    pub artifact_ref: Option<String>,
    pub page: TrustCenterPage,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustCenterCompliancePackQuery {
    pub asserted_tenant_id: Option<String>,
    pub compliance_pack_id: Option<String>,
    pub page: TrustCenterPage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterExportRequestInput {
    pub asserted_tenant_id: Option<String>,
    pub purpose: String,
    pub framework: String,
    pub time_window_start_trusted: String,
    pub time_window_end_trusted: String,
    pub evidence_ids: Vec<String>,
    pub expires_at_trusted: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustCenterAccessAuditQuery {
    pub asserted_tenant_id: Option<String>,
    pub event_type: Option<String>,
    pub target_record_id: Option<String>,
    pub page: TrustCenterPage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterGrantEventInput {
    pub asserted_tenant_id: Option<String>,
    pub grant_id: String,
    pub reviewer_principal_id: String,
    pub expires_at_trusted: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterExportDownloadInput {
    pub asserted_tenant_id: Option<String>,
    pub export_request_id: String,
    pub artifact_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterPublishabilityDecisionInput {
    pub asserted_tenant_id: Option<String>,
    pub decision_id: String,
    pub evidence_id: String,
    pub previous_state: TrustCenterPublishabilityState,
    pub new_state: TrustCenterPublishabilityState,
    pub reason: String,
    pub expires_at_trusted_or_retention_until: String,
}

trait TenantAssertion {
    fn asserted_tenant_id(&self) -> Option<&str>;
}

impl TenantAssertion for TrustCenterEvidenceIndexQuery {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterEvidenceDetailQuery {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterControlFreshnessQuery {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterSbomVexQuery {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterCompliancePackQuery {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterExportRequestInput {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterAccessAuditQuery {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterGrantEventInput {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterExportDownloadInput {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

impl TenantAssertion for TrustCenterPublishabilityDecisionInput {
    fn asserted_tenant_id(&self) -> Option<&str> {
        self.asserted_tenant_id.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterEvidenceIndexResponse {
    pub records: Vec<TrustCenterEvidenceIndexRecord>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterControlFreshnessResponse {
    pub records: Vec<TrustCenterControlFreshnessRecord>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterSbomVexResponse {
    pub records: Vec<TrustCenterSbomVexViewRecord>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterCompliancePackResponse {
    pub records: Vec<TrustCenterCompliancePackViewRecord>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustCenterAccessAuditResponse {
    pub records: Vec<TrustCenterAccessAuditRecord>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrustCenterReadModel {
    evidence_items: BTreeMap<String, TrustCenterEvidenceItemRecord>,
    control_freshness: BTreeMap<String, TrustCenterControlFreshnessRecord>,
    sbom_vex: BTreeMap<String, TrustCenterSbomVexViewRecord>,
    compliance_packs: BTreeMap<String, TrustCenterCompliancePackViewRecord>,
    export_requests: BTreeMap<String, TrustCenterExportRequestRecord>,
    access_audit: Vec<TrustCenterAccessAuditRecord>,
    publishability_decisions: Vec<TrustCenterPublishabilityDecisionRecord>,
    emitted_audit_event_refs: Vec<String>,
}

impl TrustCenterReadModel {
    pub fn upsert_evidence_item(
        &mut self,
        record: TrustCenterEvidenceItemRecord,
    ) -> Result<(), TrustCenterApiError> {
        validate_common(&record.common, TRUST_CENTER_EVIDENCE_ITEM_RECORD_TYPE)?;
        if record.raw_operator_payload_exposed {
            return Err(TrustCenterApiError::InvalidRecordShape {
                reason: "evidence item cannot expose raw operator payload".to_owned(),
            });
        }
        self.evidence_items
            .insert(record.common.record_id.clone(), record);
        Ok(())
    }

    pub fn upsert_control_freshness(
        &mut self,
        record: TrustCenterControlFreshnessRecord,
    ) -> Result<(), TrustCenterApiError> {
        validate_common(&record.common, TRUST_CENTER_CONTROL_FRESHNESS_RECORD_TYPE)?;
        self.control_freshness
            .insert(record.common.record_id.clone(), record);
        Ok(())
    }

    pub fn upsert_sbom_vex(
        &mut self,
        record: TrustCenterSbomVexViewRecord,
    ) -> Result<(), TrustCenterApiError> {
        validate_common(&record.common, TRUST_CENTER_SBOM_VEX_RECORD_TYPE)?;
        if record.raw_scanner_output_exposed || record.exploit_detail_exposed {
            return Err(TrustCenterApiError::InvalidRecordShape {
                reason: "SBOM/VEX view cannot expose raw scanner output or exploit detail"
                    .to_owned(),
            });
        }
        self.sbom_vex
            .insert(record.common.record_id.clone(), record);
        Ok(())
    }

    pub fn upsert_compliance_pack(
        &mut self,
        record: TrustCenterCompliancePackViewRecord,
    ) -> Result<(), TrustCenterApiError> {
        validate_common(&record.common, TRUST_CENTER_COMPLIANCE_PACK_RECORD_TYPE)?;
        self.compliance_packs
            .insert(record.common.record_id.clone(), record);
        Ok(())
    }

    pub fn export_requests(&self) -> &BTreeMap<String, TrustCenterExportRequestRecord> {
        &self.export_requests
    }

    pub fn access_audit_records(&self) -> &[TrustCenterAccessAuditRecord] {
        &self.access_audit
    }

    pub fn emitted_audit_event_refs(&self) -> &[String] {
        &self.emitted_audit_event_refs
    }

    pub fn publishability_decisions(&self) -> &[TrustCenterPublishabilityDecisionRecord] {
        &self.publishability_decisions
    }
}

pub fn list_trust_center_evidence(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterEvidenceIndexQuery>,
) -> Result<TrustCenterEvidenceIndexResponse, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_EVIDENCE_INDEX_SURFACE,
        TrustCenterPurpose::EvidenceRead,
    )?;
    let tenant_id = request.boundary.tenant_id.clone();
    let mut records = model
        .evidence_items
        .values()
        .filter(|record| record.common.tenant_id == tenant_id)
        .filter(|record| record.common.data_class != TrustCenterDataClass::OperatorSecurityInternal)
        .filter(|record| {
            record.common.publishability_state != TrustCenterPublishabilityState::OperatorOnly
        })
        .filter(|record| evidence_index_filter(record, &request.payload))
        .map(evidence_index_record)
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.common.record_id.cmp(&right.common.record_id));
    if records.is_empty() {
        return Err(TrustCenterApiError::MissingEvidence {
            endpoint: TRUST_CENTER_EVIDENCE_INDEX_SURFACE.to_owned(),
        });
    }
    for record in &records {
        ensure_summary_visible(principal, &request.authorization, &record.common)?;
    }
    let (records, next_cursor) = paginate(&records, &request.payload.page, &tenant_id)?;
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::EvidenceIndexViewed,
        None,
        true,
        TRUST_CENTER_EVIDENCE_INDEX_SURFACE,
    )?;
    Ok(TrustCenterEvidenceIndexResponse {
        records,
        next_cursor,
    })
}

pub fn get_trust_center_evidence_detail(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterEvidenceDetailQuery>,
) -> Result<TrustCenterEvidenceItemRecord, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_EVIDENCE_DETAIL_SURFACE,
        TrustCenterPurpose::EvidenceRead,
    )?;
    let record = model
        .evidence_items
        .get(&request.payload.evidence_id)
        .cloned()
        .ok_or_else(|| TrustCenterApiError::EvidenceNotFound {
            evidence_id: request.payload.evidence_id.clone(),
        })?;
    ensure_detail_visible(
        principal,
        &request.authorization,
        &request.boundary,
        &record.common,
    )?;
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::EvidenceItemViewed,
        Some(record.common.record_id.clone()),
        true,
        TRUST_CENTER_EVIDENCE_DETAIL_SURFACE,
    )?;
    Ok(record)
}

pub fn get_trust_center_control_freshness(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterControlFreshnessQuery>,
) -> Result<TrustCenterControlFreshnessResponse, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_CONTROL_FRESHNESS_SURFACE,
        TrustCenterPurpose::ControlEvidenceRead,
    )?;
    let tenant_id = request.boundary.tenant_id.clone();
    let mut records = model
        .control_freshness
        .values()
        .filter(|record| record.common.tenant_id == tenant_id)
        .filter(|record| control_freshness_filter(record, &request.payload))
        .cloned()
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.common.record_id.cmp(&right.common.record_id));
    if records.is_empty() {
        return Err(TrustCenterApiError::MissingEvidence {
            endpoint: TRUST_CENTER_CONTROL_FRESHNESS_SURFACE.to_owned(),
        });
    }
    for record in &records {
        ensure_detail_visible(
            principal,
            &request.authorization,
            &request.boundary,
            &record.common,
        )?;
    }
    let (records, next_cursor) = paginate(&records, &request.payload.page, &tenant_id)?;
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::EvidenceIndexViewed,
        None,
        true,
        TRUST_CENTER_CONTROL_FRESHNESS_SURFACE,
    )?;
    Ok(TrustCenterControlFreshnessResponse {
        records,
        next_cursor,
    })
}

pub fn get_trust_center_sbom_vex(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterSbomVexQuery>,
) -> Result<TrustCenterSbomVexResponse, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_SBOM_VEX_SURFACE,
        TrustCenterPurpose::SecurityEvidenceRead,
    )?;
    let tenant_id = request.boundary.tenant_id.clone();
    let mut records = model
        .sbom_vex
        .values()
        .filter(|record| record.common.tenant_id == tenant_id)
        .filter(|record| sbom_vex_filter(record, &request.payload))
        .cloned()
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.common.record_id.cmp(&right.common.record_id));
    if records.is_empty() {
        return Err(TrustCenterApiError::MissingEvidence {
            endpoint: TRUST_CENTER_SBOM_VEX_SURFACE.to_owned(),
        });
    }
    for record in &records {
        ensure_detail_visible(
            principal,
            &request.authorization,
            &request.boundary,
            &record.common,
        )?;
    }
    let (records, next_cursor) = paginate(&records, &request.payload.page, &tenant_id)?;
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::EvidenceItemViewed,
        None,
        true,
        TRUST_CENTER_SBOM_VEX_SURFACE,
    )?;
    Ok(TrustCenterSbomVexResponse {
        records,
        next_cursor,
    })
}

pub fn get_trust_center_compliance_packs(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterCompliancePackQuery>,
) -> Result<TrustCenterCompliancePackResponse, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_COMPLIANCE_PACKS_SURFACE,
        TrustCenterPurpose::ComplianceRead,
    )?;
    let tenant_id = request.boundary.tenant_id.clone();
    let mut records = model
        .compliance_packs
        .values()
        .filter(|record| record.common.tenant_id == tenant_id)
        .filter(|record| compliance_pack_filter(record, &request.payload))
        .cloned()
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.common.record_id.cmp(&right.common.record_id));
    if records.is_empty() {
        return Err(TrustCenterApiError::MissingEvidence {
            endpoint: TRUST_CENTER_COMPLIANCE_PACKS_SURFACE.to_owned(),
        });
    }
    for record in &records {
        ensure_detail_visible(
            principal,
            &request.authorization,
            &request.boundary,
            &record.common,
        )?;
    }
    let (records, next_cursor) = paginate(&records, &request.payload.page, &tenant_id)?;
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::EvidenceItemViewed,
        None,
        true,
        TRUST_CENTER_COMPLIANCE_PACKS_SURFACE,
    )?;
    Ok(TrustCenterCompliancePackResponse {
        records,
        next_cursor,
    })
}

pub fn create_trust_center_export_request(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterExportRequestInput>,
) -> Result<TrustCenterExportRequestRecord, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_EXPORT_REQUEST_SURFACE,
        TrustCenterPurpose::ExportRequest,
    )?;
    if principal.role != TrustCenterRole::TenantAdmin {
        return Err(TrustCenterApiError::ExportRequiresTenantAdmin);
    }
    if request.payload.evidence_ids.is_empty() {
        return Err(TrustCenterApiError::MissingEvidence {
            endpoint: TRUST_CENTER_EXPORT_REQUEST_SURFACE.to_owned(),
        });
    }
    let mut unique_ids = BTreeSet::new();
    for evidence_id in &request.payload.evidence_ids {
        if !unique_ids.insert(evidence_id.clone()) {
            return Err(TrustCenterApiError::InvalidRecordShape {
                reason: format!("duplicate evidence id {evidence_id}"),
            });
        }
        let evidence = model.evidence_items.get(evidence_id).ok_or_else(|| {
            TrustCenterApiError::EvidenceNotFound {
                evidence_id: evidence_id.clone(),
            }
        })?;
        ensure_detail_visible(
            principal,
            &request.authorization,
            &request.boundary,
            &evidence.common,
        )?;
    }
    let export_request_id = format!(
        "export_req_{}",
        slug_component(&request.boundary.request_id)
    );
    let record_id = export_request_id.clone();
    let common = TrustCenterCommonFields {
        record_id: record_id.clone(),
        record_type: TRUST_CENTER_EXPORT_REQUEST_RECORD_TYPE.to_owned(),
        schema_version: TRUST_CENTER_SCHEMA_VERSION,
        tenant_id: request.boundary.tenant_id.clone(),
        audience_id: principal.audience_id.clone(),
        source_system: TRUST_CENTER_SERVICE_NAME.to_owned(),
        source_record_ref: request.boundary.request_id.clone(),
        evidence_class: "regulated_export_request".to_owned(),
        data_class: TrustCenterDataClass::RegulatedExportEvidence,
        claim_tier: TrustCenterClaimTier::SpecReady,
        freshness_state: TrustCenterFreshnessState::Current,
        publishability_state: TrustCenterPublishabilityState::TenantAdminOnly,
        redaction_policy_id: "redact_trust_center_export_manifest_v1".to_owned(),
        audit_event_ref: format!("audit/{record_id}"),
        created_at_trusted: request.boundary.occurred_at_trusted.clone(),
        expires_at_trusted_or_retention_until: request.payload.expires_at_trusted.clone(),
    };
    validate_common(&common, TRUST_CENTER_EXPORT_REQUEST_RECORD_TYPE)?;
    let record = TrustCenterExportRequestRecord {
        common,
        export_request_id,
        requested_by_principal_id: principal.principal_id.clone(),
        purpose: request.payload.purpose.clone(),
        framework: request.payload.framework.clone(),
        time_window_start_trusted: request.payload.time_window_start_trusted.clone(),
        time_window_end_trusted: request.payload.time_window_end_trusted.clone(),
        evidence_ids: request.payload.evidence_ids.clone(),
        approval_state: "operator_review_required".to_owned(),
        manifest_ref: None,
    };
    model
        .export_requests
        .insert(record_id.clone(), record.clone());
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::EvidenceExportRequested,
        Some(record_id),
        true,
        TRUST_CENTER_EXPORT_REQUEST_SURFACE,
    )?;
    Ok(record)
}

pub fn get_trust_center_access_audit(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterAccessAuditQuery>,
) -> Result<TrustCenterAccessAuditResponse, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_ACCESS_AUDIT_SURFACE,
        TrustCenterPurpose::AccessAuditRead,
    )?;
    let tenant_id = request.boundary.tenant_id.clone();
    let mut records = model
        .access_audit
        .iter()
        .filter(|record| record.common.tenant_id == tenant_id)
        .filter(|record| access_audit_filter(record, &request.payload))
        .cloned()
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.common.record_id.cmp(&right.common.record_id));
    for record in &records {
        ensure_summary_visible(principal, &request.authorization, &record.common)?;
    }
    let (records, next_cursor) = paginate(&records, &request.payload.page, &tenant_id)?;
    Ok(TrustCenterAccessAuditResponse {
        records,
        next_cursor,
    })
}

pub fn record_trust_center_access_grant_created(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterGrantEventInput>,
) -> Result<TrustCenterAccessAuditRecord, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_GRANT_WRITE_SURFACE,
        TrustCenterPurpose::GrantManagement,
    )?;
    if principal.role != TrustCenterRole::TenantAdmin
        && principal.role != TrustCenterRole::OyatieOperator
    {
        return Err(TrustCenterApiError::RoleDenied {
            role: principal.role,
            endpoint: TRUST_CENTER_GRANT_WRITE_SURFACE.to_owned(),
        });
    }
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::AccessGrantCreated,
        Some(request.payload.grant_id.clone()),
        true,
        TRUST_CENTER_GRANT_WRITE_SURFACE,
    )
}

pub fn record_trust_center_export_downloaded(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterExportDownloadInput>,
) -> Result<TrustCenterAccessAuditRecord, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_EXPORT_DOWNLOAD_SURFACE,
        TrustCenterPurpose::Download,
    )?;
    let export_request = model
        .export_requests
        .get(&request.payload.export_request_id)
        .ok_or_else(|| TrustCenterApiError::EvidenceNotFound {
            evidence_id: request.payload.export_request_id.clone(),
        })?;
    ensure_tenant_match(&request.boundary.tenant_id, &export_request.common)?;
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::EvidenceExportDownloaded,
        Some(request.payload.export_request_id.clone()),
        true,
        TRUST_CENTER_EXPORT_DOWNLOAD_SURFACE,
    )
}

pub fn record_trust_center_publishability_state_changed(
    model: &mut TrustCenterReadModel,
    request: TrustCenterApiRequest<TrustCenterPublishabilityDecisionInput>,
) -> Result<TrustCenterPublishabilityDecisionRecord, TrustCenterApiError> {
    let principal = validate_api_request(
        &request,
        TRUST_CENTER_PUBLISHABILITY_SURFACE,
        TrustCenterPurpose::PublishabilityReview,
    )?;
    if principal.role != TrustCenterRole::OyatieOperator {
        return Err(TrustCenterApiError::RoleDenied {
            role: principal.role,
            endpoint: TRUST_CENTER_PUBLISHABILITY_SURFACE.to_owned(),
        });
    }
    let evidence = model
        .evidence_items
        .get(&request.payload.evidence_id)
        .ok_or_else(|| TrustCenterApiError::EvidenceNotFound {
            evidence_id: request.payload.evidence_id.clone(),
        })?;
    if evidence.common.tenant_id != request.boundary.tenant_id {
        return Err(TrustCenterApiError::EvidenceTenantMismatch {
            trusted_tenant_id: request.boundary.tenant_id.clone(),
            evidence_tenant_id: evidence.common.tenant_id.clone(),
        });
    }
    let record_id = format!("pub_dec_{}", slug_component(&request.payload.decision_id));
    let common = TrustCenterCommonFields {
        record_id: record_id.clone(),
        record_type: TRUST_CENTER_PUBLISHABILITY_DECISION_RECORD_TYPE.to_owned(),
        schema_version: TRUST_CENTER_SCHEMA_VERSION,
        tenant_id: request.boundary.tenant_id.clone(),
        audience_id: principal.audience_id.clone(),
        source_system: TRUST_CENTER_SERVICE_NAME.to_owned(),
        source_record_ref: request.payload.evidence_id.clone(),
        evidence_class: "publishability_decision".to_owned(),
        data_class: TrustCenterDataClass::TenantTrustEvidence,
        claim_tier: TrustCenterClaimTier::SpecReady,
        freshness_state: TrustCenterFreshnessState::Current,
        publishability_state: TrustCenterPublishabilityState::TenantAdminOnly,
        redaction_policy_id: "redact_trust_center_publishability_v1".to_owned(),
        audit_event_ref: format!("audit/{record_id}"),
        created_at_trusted: request.boundary.occurred_at_trusted.clone(),
        expires_at_trusted_or_retention_until: request
            .payload
            .expires_at_trusted_or_retention_until
            .clone(),
    };
    validate_common(&common, TRUST_CENTER_PUBLISHABILITY_DECISION_RECORD_TYPE)?;
    let decision = TrustCenterPublishabilityDecisionRecord {
        common,
        decision_id: request.payload.decision_id.clone(),
        evidence_id: request.payload.evidence_id.clone(),
        previous_state: request.payload.previous_state,
        new_state: request.payload.new_state,
        reason: request.payload.reason.clone(),
        decided_by_principal_id: principal.principal_id.clone(),
    };
    model.publishability_decisions.push(decision.clone());
    append_access_audit_event(
        model,
        &request.boundary,
        principal,
        TrustCenterAccessAuditEventKind::PublishabilityStateChanged,
        Some(request.payload.evidence_id.clone()),
        true,
        TRUST_CENTER_PUBLISHABILITY_SURFACE,
    )?;
    Ok(decision)
}

fn validate_api_request<'a, T: TenantAssertion>(
    request: &'a TrustCenterApiRequest<T>,
    endpoint: &str,
    required_purpose: TrustCenterPurpose,
) -> Result<&'a TrustCenterPrincipal, TrustCenterApiError> {
    validate_boundary(&request.boundary)?;
    validate_tenant_assertion(&request.boundary, request.payload.asserted_tenant_id())?;
    let principal = request
        .principal
        .as_ref()
        .ok_or(TrustCenterApiError::MissingPrincipal)?;
    validate_principal(&request.boundary, principal)?;
    validate_authorization(
        &request.boundary,
        principal,
        &request.authorization,
        endpoint,
        required_purpose,
    )?;
    Ok(principal)
}

fn validate_boundary(boundary: &TrustCenterBoundaryContext) -> Result<(), TrustCenterApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(TrustCenterApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(TrustCenterApiError::MissingTrustedTenant);
    }
    if !boundary.tenant_id.starts_with(TENANT_ID_PREFIX) {
        return Err(TrustCenterApiError::MalformedTrustedTenant {
            tenant_id: boundary.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_tenant_assertion(
    boundary: &TrustCenterBoundaryContext,
    asserted_tenant_id: Option<&str>,
) -> Result<(), TrustCenterApiError> {
    if let Some(asserted) = asserted_tenant_id
        && asserted != boundary.tenant_id
    {
        return Err(TrustCenterApiError::TenantAssertionMismatch {
            trusted_tenant_id: boundary.tenant_id.clone(),
            asserted_tenant_id: asserted.to_owned(),
        });
    }
    Ok(())
}

fn validate_principal(
    boundary: &TrustCenterBoundaryContext,
    principal: &TrustCenterPrincipal,
) -> Result<(), TrustCenterApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(TrustCenterApiError::EmptyPrincipalId);
    }
    if principal.tenant_id != boundary.tenant_id {
        return Err(TrustCenterApiError::PrincipalTenantMismatch {
            trusted_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    boundary: &TrustCenterBoundaryContext,
    principal: &TrustCenterPrincipal,
    authorization: &TrustCenterAuthorizationDecision,
    endpoint: &str,
    required_purpose: TrustCenterPurpose,
) -> Result<(), TrustCenterApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(TrustCenterApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != boundary.tenant_id {
        return Err(TrustCenterApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            trusted_tenant_id: boundary.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(TrustCenterApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_endpoints
        .iter()
        .any(|allowed| allowed == endpoint)
    {
        return Err(TrustCenterApiError::AuthorizationDenied {
            endpoint: endpoint.to_owned(),
        });
    }
    if principal.purpose != required_purpose {
        return Err(TrustCenterApiError::PurposeDenied {
            required: required_purpose,
            supplied: principal.purpose,
        });
    }
    if !authorization.allowed_purposes.contains(&required_purpose) {
        return Err(TrustCenterApiError::PurposeDenied {
            required: required_purpose,
            supplied: principal.purpose,
        });
    }
    if !role_allows_endpoint(principal, endpoint) {
        return Err(TrustCenterApiError::RoleDenied {
            role: principal.role,
            endpoint: endpoint.to_owned(),
        });
    }
    Ok(())
}

fn role_allows_endpoint(principal: &TrustCenterPrincipal, endpoint: &str) -> bool {
    match endpoint {
        TRUST_CENTER_EVIDENCE_INDEX_SURFACE | TRUST_CENTER_EVIDENCE_DETAIL_SURFACE => {
            match principal.role {
                TrustCenterRole::TenantAdmin | TrustCenterRole::OyatieOperator => true,
                TrustCenterRole::SecurityComplianceReviewer | TrustCenterRole::Auditor => {
                    has_active_grant(principal)
                }
            }
        }
        TRUST_CENTER_SBOM_VEX_SURFACE | TRUST_CENTER_CONTROL_FRESHNESS_SURFACE => {
            match principal.role {
                TrustCenterRole::TenantAdmin | TrustCenterRole::OyatieOperator => true,
                TrustCenterRole::SecurityComplianceReviewer | TrustCenterRole::Auditor => {
                    has_active_grant(principal)
                }
            }
        }
        TRUST_CENTER_COMPLIANCE_PACKS_SURFACE => match principal.role {
            TrustCenterRole::TenantAdmin | TrustCenterRole::OyatieOperator => true,
            TrustCenterRole::SecurityComplianceReviewer | TrustCenterRole::Auditor => {
                has_active_grant(principal)
            }
        },
        TRUST_CENTER_EXPORT_REQUEST_SURFACE => principal.role == TrustCenterRole::TenantAdmin,
        TRUST_CENTER_ACCESS_AUDIT_SURFACE => {
            principal.role == TrustCenterRole::TenantAdmin
                || principal.role == TrustCenterRole::OyatieOperator
        }
        TRUST_CENTER_GRANT_WRITE_SURFACE => {
            principal.role == TrustCenterRole::TenantAdmin
                || principal.role == TrustCenterRole::OyatieOperator
        }
        TRUST_CENTER_EXPORT_DOWNLOAD_SURFACE => match principal.role {
            TrustCenterRole::TenantAdmin | TrustCenterRole::OyatieOperator => true,
            TrustCenterRole::SecurityComplianceReviewer | TrustCenterRole::Auditor => {
                has_active_grant(principal)
            }
        },
        TRUST_CENTER_PUBLISHABILITY_SURFACE => principal.role == TrustCenterRole::OyatieOperator,
        _ => false,
    }
}

fn has_active_grant(principal: &TrustCenterPrincipal) -> bool {
    principal
        .access_grant_id
        .as_deref()
        .is_some_and(|grant| !grant.trim().is_empty())
        && principal
            .expires_at_trusted
            .as_deref()
            .is_some_and(|expires| !expires.trim().is_empty())
}

fn validate_common(
    common: &TrustCenterCommonFields,
    expected_record_type: &str,
) -> Result<(), TrustCenterApiError> {
    if common.record_id.trim().is_empty() {
        return Err(TrustCenterApiError::InvalidRecordShape {
            reason: "record_id must be non-empty".to_owned(),
        });
    }
    if common.record_type != expected_record_type {
        return Err(TrustCenterApiError::InvalidRecordShape {
            reason: format!(
                "record_type {} does not match expected {}",
                common.record_type, expected_record_type
            ),
        });
    }
    if common.schema_version != TRUST_CENTER_SCHEMA_VERSION {
        return Err(TrustCenterApiError::InvalidRecordShape {
            reason: format!("schema_version must be {TRUST_CENTER_SCHEMA_VERSION}"),
        });
    }
    if !common.tenant_id.starts_with(TENANT_ID_PREFIX) {
        return Err(TrustCenterApiError::InvalidRecordShape {
            reason: "tenant_id must come from trusted ten_ scope".to_owned(),
        });
    }
    for (field, value) in [
        ("audience_id", common.audience_id.as_str()),
        ("source_system", common.source_system.as_str()),
        ("source_record_ref", common.source_record_ref.as_str()),
        ("evidence_class", common.evidence_class.as_str()),
        ("redaction_policy_id", common.redaction_policy_id.as_str()),
        ("audit_event_ref", common.audit_event_ref.as_str()),
        ("created_at_trusted", common.created_at_trusted.as_str()),
        (
            "expires_at_trusted_or_retention_until",
            common.expires_at_trusted_or_retention_until.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(TrustCenterApiError::InvalidRecordShape {
                reason: format!("{field} must be non-empty"),
            });
        }
    }
    if common.data_class == TrustCenterDataClass::OperatorSecurityInternal
        && matches!(
            common.publishability_state,
            TrustCenterPublishabilityState::PublishableCustomerSafe
                | TrustCenterPublishabilityState::PublishableSummaryOnly
        )
    {
        return Err(TrustCenterApiError::InvalidRecordShape {
            reason: "operator-internal evidence cannot be marked customer-publishable".to_owned(),
        });
    }
    Ok(())
}

fn evidence_index_record(record: &TrustCenterEvidenceItemRecord) -> TrustCenterEvidenceIndexRecord {
    let mut common = record.common.clone();
    common.record_id = format!("idx_{}", record.common.record_id);
    common.record_type = TRUST_CENTER_EVIDENCE_INDEX_RECORD_TYPE.to_owned();
    TrustCenterEvidenceIndexRecord {
        common,
        title: record.title.clone(),
        customer_safe_summary: record.customer_safe_summary.clone(),
        compliance_pack_ids: record.compliance_pack_ids.clone(),
        service_ids: record.service_ids.clone(),
        last_updated_trusted: record.common.created_at_trusted.clone(),
    }
}

fn ensure_summary_visible(
    principal: &TrustCenterPrincipal,
    authorization: &TrustCenterAuthorizationDecision,
    common: &TrustCenterCommonFields,
) -> Result<(), TrustCenterApiError> {
    ensure_tenant_match(&principal.tenant_id, common)?;
    ensure_data_class_allowed(authorization, common.data_class)?;
    if common.data_class == TrustCenterDataClass::OperatorSecurityInternal
        || common.publishability_state == TrustCenterPublishabilityState::OperatorOnly
    {
        return Err(TrustCenterApiError::OperatorOnlyDetailDenied);
    }
    ensure_publishability_allowed_by_role(principal, common.publishability_state)?;
    if !authorization
        .allowed_publishability_states
        .contains(&common.publishability_state)
    {
        return Err(TrustCenterApiError::PublishabilityDenied {
            publishability_state: common.publishability_state,
        });
    }
    Ok(())
}

fn ensure_detail_visible(
    principal: &TrustCenterPrincipal,
    authorization: &TrustCenterAuthorizationDecision,
    boundary: &TrustCenterBoundaryContext,
    common: &TrustCenterCommonFields,
) -> Result<(), TrustCenterApiError> {
    ensure_tenant_match(&boundary.tenant_id, common)?;
    ensure_summary_visible(principal, authorization, common)?;
    if !is_fresh_enough(common.freshness_state) {
        return Err(TrustCenterApiError::EvidenceNotFresh {
            record_id: common.record_id.clone(),
            freshness_state: common.freshness_state,
        });
    }
    if is_blocked_publishability(common.publishability_state) {
        return Err(TrustCenterApiError::EvidenceNotPublishable {
            publishability_state: common.publishability_state,
        });
    }
    Ok(())
}

fn ensure_tenant_match(
    trusted_tenant_id: &str,
    common: &TrustCenterCommonFields,
) -> Result<(), TrustCenterApiError> {
    if common.tenant_id != trusted_tenant_id {
        return Err(TrustCenterApiError::EvidenceTenantMismatch {
            trusted_tenant_id: trusted_tenant_id.to_owned(),
            evidence_tenant_id: common.tenant_id.clone(),
        });
    }
    Ok(())
}

fn ensure_data_class_allowed(
    authorization: &TrustCenterAuthorizationDecision,
    data_class: TrustCenterDataClass,
) -> Result<(), TrustCenterApiError> {
    if !authorization.allowed_data_classes.contains(&data_class) {
        return Err(TrustCenterApiError::DataClassDenied { data_class });
    }
    Ok(())
}

fn ensure_publishability_allowed_by_role(
    principal: &TrustCenterPrincipal,
    state: TrustCenterPublishabilityState,
) -> Result<(), TrustCenterApiError> {
    match state {
        TrustCenterPublishabilityState::TenantAdminOnly
            if principal.role == TrustCenterRole::SecurityComplianceReviewer
                || principal.role == TrustCenterRole::Auditor =>
        {
            return Err(TrustCenterApiError::PublishabilityDenied {
                publishability_state: state,
            });
        }
        TrustCenterPublishabilityState::OperatorOnly => {
            return Err(TrustCenterApiError::OperatorOnlyDetailDenied);
        }
        _ => {}
    }
    Ok(())
}

fn is_fresh_enough(state: TrustCenterFreshnessState) -> bool {
    matches!(
        state,
        TrustCenterFreshnessState::Current
            | TrustCenterFreshnessState::AgingWarning
            | TrustCenterFreshnessState::NotApplicableWithPolicyReason
    )
}

fn is_blocked_publishability(state: TrustCenterPublishabilityState) -> bool {
    matches!(
        state,
        TrustCenterPublishabilityState::BlockedMissingEvidence
            | TrustCenterPublishabilityState::BlockedStaleEvidence
            | TrustCenterPublishabilityState::BlockedSecurityPrivacyReview
            | TrustCenterPublishabilityState::OperatorOnly
    )
}

fn evidence_index_filter(
    record: &TrustCenterEvidenceItemRecord,
    query: &TrustCenterEvidenceIndexQuery,
) -> bool {
    optional_eq(
        query.evidence_class.as_deref(),
        &record.common.evidence_class,
    ) && optional_vec_contains(
        query.compliance_pack_id.as_deref(),
        &record.compliance_pack_ids,
    ) && optional_eq(query.source_system.as_deref(), &record.common.source_system)
        && query
            .freshness_state
            .is_none_or(|state| state == record.common.freshness_state)
        && query
            .claim_tier
            .is_none_or(|tier| tier == record.common.claim_tier)
}

fn control_freshness_filter(
    record: &TrustCenterControlFreshnessRecord,
    query: &TrustCenterControlFreshnessQuery,
) -> bool {
    optional_vec_contains(
        query.compliance_pack_id.as_deref(),
        &record.compliance_pack_ids,
    ) && query
        .service_id
        .as_deref()
        .is_none_or(|service| record.service_id.as_deref() == Some(service))
}

fn sbom_vex_filter(record: &TrustCenterSbomVexViewRecord, query: &TrustCenterSbomVexQuery) -> bool {
    query
        .artifact_ref
        .as_deref()
        .is_none_or(|artifact| artifact == record.artifact_ref)
}

fn compliance_pack_filter(
    record: &TrustCenterCompliancePackViewRecord,
    query: &TrustCenterCompliancePackQuery,
) -> bool {
    query
        .compliance_pack_id
        .as_deref()
        .is_none_or(|pack| pack == record.compliance_pack_id)
}

fn access_audit_filter(
    record: &TrustCenterAccessAuditRecord,
    query: &TrustCenterAccessAuditQuery,
) -> bool {
    optional_eq(query.event_type.as_deref(), &record.event_type)
        && query
            .target_record_id
            .as_deref()
            .is_none_or(|target| record.target_record_id.as_deref() == Some(target))
}

fn optional_eq(expected: Option<&str>, actual: &str) -> bool {
    expected.is_none_or(|value| value == actual)
}

fn optional_vec_contains(expected: Option<&str>, actual: &[String]) -> bool {
    expected.is_none_or(|value| actual.iter().any(|candidate| candidate == value))
}

fn paginate<T: Clone>(
    records: &[T],
    page: &TrustCenterPage,
    tenant_id: &str,
) -> Result<(Vec<T>, Option<String>), TrustCenterApiError> {
    let page_size = page.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    if page_size == 0 || page_size > MAX_PAGE_SIZE {
        return Err(TrustCenterApiError::InvalidPageSize { page_size });
    }
    let offset = match &page.cursor {
        Some(cursor) => parse_cursor(cursor, tenant_id)?,
        None => 0,
    };
    if offset > records.len() {
        return Err(TrustCenterApiError::InvalidCursor {
            cursor: page.cursor.clone().unwrap_or_default(),
        });
    }
    let end = records.len().min(offset.saturating_add(page_size));
    let next_cursor = if end < records.len() {
        Some(format!("{TRUST_CENTER_CURSOR_PREFIX}{tenant_id}/{end}"))
    } else {
        None
    };
    Ok((records[offset..end].to_vec(), next_cursor))
}

fn parse_cursor(cursor: &str, tenant_id: &str) -> Result<usize, TrustCenterApiError> {
    let expected_prefix = format!("{TRUST_CENTER_CURSOR_PREFIX}{tenant_id}/");
    if !cursor.starts_with(&expected_prefix) {
        return Err(TrustCenterApiError::InvalidCursor {
            cursor: cursor.to_owned(),
        });
    }
    cursor[expected_prefix.len()..]
        .parse::<usize>()
        .map_err(|_| TrustCenterApiError::InvalidCursor {
            cursor: cursor.to_owned(),
        })
}

fn append_access_audit_event(
    model: &mut TrustCenterReadModel,
    boundary: &TrustCenterBoundaryContext,
    principal: &TrustCenterPrincipal,
    event_kind: TrustCenterAccessAuditEventKind,
    target_record_id: Option<String>,
    granted: bool,
    action: &str,
) -> Result<TrustCenterAccessAuditRecord, TrustCenterApiError> {
    let sequence = model.access_audit.len() + 1;
    let record_id = format!(
        "audit_{}_{}",
        slug_component(event_kind.as_event_type()),
        sequence
    );
    let target_ref = target_record_id
        .clone()
        .unwrap_or_else(|| boundary.request_id.clone());
    let common = TrustCenterCommonFields {
        record_id: record_id.clone(),
        record_type: TRUST_CENTER_ACCESS_AUDIT_RECORD_TYPE.to_owned(),
        schema_version: TRUST_CENTER_SCHEMA_VERSION,
        tenant_id: boundary.tenant_id.clone(),
        audience_id: principal.audience_id.clone(),
        source_system: TRUST_CENTER_SERVICE_NAME.to_owned(),
        source_record_ref: target_ref,
        evidence_class: "access_audit".to_owned(),
        data_class: TrustCenterDataClass::TenantTrustEvidence,
        claim_tier: TrustCenterClaimTier::SpecReady,
        freshness_state: TrustCenterFreshnessState::Current,
        publishability_state: TrustCenterPublishabilityState::TenantAdminOnly,
        redaction_policy_id: "redact_trust_center_access_audit_v1".to_owned(),
        audit_event_ref: format!("audit/{record_id}"),
        created_at_trusted: boundary.occurred_at_trusted.clone(),
        expires_at_trusted_or_retention_until: "P400D".to_owned(),
    };
    validate_common(&common, TRUST_CENTER_ACCESS_AUDIT_RECORD_TYPE)?;
    let record = TrustCenterAccessAuditRecord {
        common: common.clone(),
        event_type: event_kind.as_event_type().to_owned(),
        actor_principal_id: principal.principal_id.clone(),
        actor_role: principal.role,
        action: action.to_owned(),
        target_record_id,
        granted,
        occurred_at_trusted: boundary.occurred_at_trusted.clone(),
        decision_id: format!("{}:{}", boundary.request_id, event_kind.as_event_type()),
    };
    model.emitted_audit_event_refs.push(common.audit_event_ref);
    model.access_audit.push(record.clone());
    Ok(record)
}

fn slug_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_owned()
}
