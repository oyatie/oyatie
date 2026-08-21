//! Foundry autonomy policy API boundary.
//!
//! This crate owns the REST-boundary normalization, authorization proof checks,
//! idempotency, and typed OpenAPI DTO projection for the stable
//! `foundry.policy.autonomy-ceiling.publish` surface. The kernel remains the
//! source of truth for autonomy-ceiling resolution; durable Cedar storage,
//! bundle rollout, and audit-chain append are adapter/application concerns.

use std::collections::BTreeMap;

use intelligence_capability_domain::{AutonomyTier, Capability, CapabilityAction, CapabilityError};
use intelligence_policy_domain::{
    AutonomyCapReason, AutonomyCapSource, AutonomyDecision, AutonomyVerdict, TenantPolicy,
};
use data_boundary_kernel::{
    AgeBand, DataClass, PrivacyDataClass, SubjectClass, parse_data_class_label,
};

pub const FOUNDRY_POLICY_AUTONOMY_CEILING_PUBLISH_SURFACE: &str =
    "foundry.policy.autonomy-ceiling.publish";
pub const FOUNDRY_POLICY_OPENAPI_CONTRACT: &str = "contracts/openapi/foundry/policy-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundryPolicyAutonomyCeilingPublishApiStatus {
    Created,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl FoundryPolicyAutonomyCeilingPublishApiStatus {
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
pub enum FoundryPolicyApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PolicyIdInvalid,
    PolicyIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    PolicyAlreadyPublished,
    AutonomyTierInvalid,
    CapabilityActionInvalid,
    DataClassInvalid,
    SubjectClassInvalid,
    CedarPolicyRefsMissing,
    KernelInvalidCapability,
}

impl FoundryPolicyApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "FOUNDRY_POLICY_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "FOUNDRY_POLICY_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "FOUNDRY_POLICY_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "FOUNDRY_POLICY_PRINCIPAL_ID_EMPTY",
            Self::PolicyIdInvalid => "FOUNDRY_POLICY_ID_INVALID",
            Self::PolicyIdMismatch => "FOUNDRY_POLICY_ID_MISMATCH",
            Self::TenantMismatch => "FOUNDRY_POLICY_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "FOUNDRY_POLICY_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "FOUNDRY_POLICY_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "FOUNDRY_POLICY_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "FOUNDRY_POLICY_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "FOUNDRY_POLICY_IDEMPOTENCY_KEY_REUSED",
            Self::PolicyAlreadyPublished => "FOUNDRY_POLICY_ALREADY_PUBLISHED",
            Self::AutonomyTierInvalid => "FOUNDRY_POLICY_AUTONOMY_TIER_INVALID",
            Self::CapabilityActionInvalid => "FOUNDRY_POLICY_CAPABILITY_ACTION_INVALID",
            Self::DataClassInvalid => "FOUNDRY_POLICY_DATA_CLASS_INVALID",
            Self::SubjectClassInvalid => "FOUNDRY_POLICY_SUBJECT_CLASS_INVALID",
            Self::CedarPolicyRefsMissing => "FOUNDRY_POLICY_CEDAR_POLICY_REFS_MISSING",
            Self::KernelInvalidCapability => "FOUNDRY_POLICY_KERNEL_INVALID_CAPABILITY",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyAutonomyBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: PII_IDENTIFYING
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyAutonomyCeilingPublishRequest {
    pub policy_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub capability_id: String,             // data_class: INTERNAL_ONLY
    pub policy_version: String,            // data_class: INTERNAL_ONLY
    pub tenant_configured_ceiling: String, // data_class: INTERNAL_ONLY
    pub principal_ceiling: String,         // data_class: INTERNAL_ONLY
    pub capability_required_tier: String,  // data_class: INTERNAL_ONLY
    pub capability_action: String,         // data_class: INTERNAL_ONLY
    pub data_classes: Vec<String>,         // data_class: INTERNAL_ONLY
    pub regulatory_packs: Vec<String>,     // data_class: INTERNAL_ONLY
    pub subject_class: String,             // data_class: INTERNAL_ONLY
    pub cedar_policy_refs: Vec<String>,    // data_class: INTERNAL_ONLY
    pub evidence_event_hash: String,       // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyAutonomyCeilingPublishApiRequest {
    pub path_policy_id: String, // data_class: INTERNAL_ONLY
    pub boundary: FoundryPolicyAutonomyBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: FoundryPolicyApiPrincipal, // data_class: PII_IDENTIFYING
    pub authorization: FoundryPolicyApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: FoundryPolicyAutonomyCeilingPublishRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryPolicyAutonomyCeilingDirectory {
    records: BTreeMap<FoundryPolicyAutonomyCeilingKey, FoundryPolicyAutonomyCeilingRecord>, // data_class: INTERNAL_ONLY
}

impl FoundryPolicyAutonomyCeilingDirectory {
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn records(&self) -> impl Iterator<Item = &FoundryPolicyAutonomyCeilingRecord> {
        self.records.values()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FoundryPolicyAutonomyCeilingKey {
    tenant_id: String, // data_class: INTERNAL_ONLY
    policy_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryPolicyAutonomyCeilingPublishIdempotencyLedger {
    entries: BTreeMap<FoundryPolicyIdempotencyLedgerKey, FoundryPolicyAutonomyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl FoundryPolicyAutonomyCeilingPublishIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FoundryPolicyIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: PII_IDENTIFYING
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryPolicyAutonomyLedgerEntry {
    fingerprint: FoundryPolicyRequestFingerprint, // data_class: INTERNAL_ONLY
    result: FoundryPolicyAutonomyCeilingPublishSuccessResponse, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryPolicyRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyAutonomyCeilingPublishSuccessResponse {
    pub data: FoundryPolicyAutonomyCeilingRecord, // data_class: INTERNAL_ONLY
    pub metadata: FoundryPolicyAutonomyMetadata,  // data_class: INTERNAL_ONLY
}

impl FoundryPolicyAutonomyCeilingPublishSuccessResponse {
    pub fn created(
        data: FoundryPolicyAutonomyCeilingRecord,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            data,
            metadata: FoundryPolicyAutonomyMetadata {
                request_id: request_id.into(),
                surface: FOUNDRY_POLICY_AUTONOMY_CEILING_PUBLISH_SURFACE.to_string(),
                openapi_contract: FOUNDRY_POLICY_OPENAPI_CONTRACT.to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyAutonomyMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyAutonomyCeilingRecord {
    pub policy_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub capability_id: String,               // data_class: INTERNAL_ONLY
    pub policy_version: String,              // data_class: INTERNAL_ONLY
    pub capability_action: String,           // data_class: INTERNAL_ONLY
    pub data_classes: Vec<String>,           // data_class: INTERNAL_ONLY
    pub regulatory_packs: Vec<String>,       // data_class: INTERNAL_ONLY
    pub subject_class: String,               // data_class: INTERNAL_ONLY
    pub configured_ceiling: String,          // data_class: INTERNAL_ONLY
    pub principal_ceiling: String,           // data_class: INTERNAL_ONLY
    pub capability_required_tier: String,    // data_class: INTERNAL_ONLY
    pub agentic_ads_cap: String,             // data_class: INTERNAL_ONLY
    pub vertical_pack_cap: String,           // data_class: INTERNAL_ONLY
    pub subject_class_cap: String,           // data_class: INTERNAL_ONLY
    pub denial_threshold: String,            // data_class: INTERNAL_ONLY
    pub effective_ceiling: String,           // data_class: INTERNAL_ONLY
    pub verdict: String,                     // data_class: INTERNAL_ONLY
    pub blocking_cap_source: Option<String>, // data_class: INTERNAL_ONLY
    pub blocking_cap_reason: Option<String>, // data_class: INTERNAL_ONLY
    pub lowering_cap_source: String,         // data_class: INTERNAL_ONLY
    pub lowering_cap_reason: String,         // data_class: INTERNAL_ONLY
    pub cedar_policy_refs: Vec<String>,      // data_class: INTERNAL_ONLY
    pub evidence_event_hash: String,         // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub schema_version: u32,                 // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyApiErrorResponse {
    pub error: FoundryPolicyApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyApiErrorBody {
    pub code: String,                              // data_class: INTERNAL_ONLY
    pub message: String,                           // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,         // data_class: INTERNAL_ONLY
    pub request_id: String,                        // data_class: INTERNAL_ONLY
    pub details: Vec<FoundryPolicyApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryPolicyApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FoundryPolicyApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    InvalidPolicyId {
        policy_id: String,
    },
    PolicyIdMismatch {
        path_policy_id: String,
        body_policy_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        authorization_tenant_id: Option<String>,
        body_tenant_id: Option<String>,
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
    PolicyAlreadyPublished {
        tenant_id: String,
        policy_id: String,
    },
    InvalidAutonomyTierLabel {
        field: String,
        autonomy_tier: String,
    },
    InvalidCapabilityActionLabel {
        capability_action: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    InvalidSubjectClassLabel {
        subject_class: String,
    },
    MissingCedarPolicyRefs,
    InvalidCapability(CapabilityError),
}

impl FoundryPolicyApiError {
    pub fn status_code(&self) -> u16 {
        match self.status_kind() {
            FoundryPolicyApiStatusKind::BadRequest => 400,
            FoundryPolicyApiStatusKind::Forbidden => 403,
            FoundryPolicyApiStatusKind::Conflict => 409,
            FoundryPolicyApiStatusKind::UnprocessableEntity => 422,
        }
    }

    pub fn code(&self) -> FoundryPolicyApiErrorCode {
        match self {
            Self::EmptyRequestId => FoundryPolicyApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => FoundryPolicyApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => FoundryPolicyApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => FoundryPolicyApiErrorCode::PrincipalIdEmpty,
            Self::InvalidPolicyId { .. } => FoundryPolicyApiErrorCode::PolicyIdInvalid,
            Self::PolicyIdMismatch { .. } => FoundryPolicyApiErrorCode::PolicyIdMismatch,
            Self::TenantMismatch { .. } => FoundryPolicyApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                FoundryPolicyApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                FoundryPolicyApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                FoundryPolicyApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => FoundryPolicyApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => FoundryPolicyApiErrorCode::IdempotencyKeyReused,
            Self::PolicyAlreadyPublished { .. } => {
                FoundryPolicyApiErrorCode::PolicyAlreadyPublished
            }
            Self::InvalidAutonomyTierLabel { .. } => FoundryPolicyApiErrorCode::AutonomyTierInvalid,
            Self::InvalidCapabilityActionLabel { .. } => {
                FoundryPolicyApiErrorCode::CapabilityActionInvalid
            }
            Self::InvalidDataClassLabel { .. } => FoundryPolicyApiErrorCode::DataClassInvalid,
            Self::InvalidSubjectClassLabel { .. } => FoundryPolicyApiErrorCode::SubjectClassInvalid,
            Self::MissingCedarPolicyRefs => FoundryPolicyApiErrorCode::CedarPolicyRefsMissing,
            Self::InvalidCapability(_) => FoundryPolicyApiErrorCode::KernelInvalidCapability,
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> FoundryPolicyApiErrorResponse {
        FoundryPolicyApiErrorResponse {
            error: FoundryPolicyApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> FoundryPolicyApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => FoundryPolicyApiStatusKind::Forbidden,
            Self::PolicyAlreadyPublished { .. } => FoundryPolicyApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => FoundryPolicyApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::InvalidPolicyId { .. }
            | Self::PolicyIdMismatch { .. }
            | Self::InvalidAutonomyTierLabel { .. }
            | Self::InvalidCapabilityActionLabel { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidSubjectClassLabel { .. }
            | Self::MissingCedarPolicyRefs
            | Self::InvalidCapability(_) => FoundryPolicyApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::InvalidPolicyId { .. } => "Autonomy policy id is required",
            Self::PolicyIdMismatch { .. } => "Path and body autonomy policy ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, authorization, and body tenant"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the Foundry autonomy-ceiling publish surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::PolicyAlreadyPublished { .. } => "Foundry autonomy policy already published",
            Self::InvalidAutonomyTierLabel { .. } => "Autonomy tier label is unknown",
            Self::InvalidCapabilityActionLabel { .. } => "Capability action label is unknown",
            Self::InvalidDataClassLabel { .. } => {
                "Capability data_classes must be known privacy data classes"
            }
            Self::InvalidSubjectClassLabel { .. } => "Subject class label is unknown",
            Self::MissingCedarPolicyRefs => {
                "Cedar-backed autonomy policy publish requires policy refs"
            }
            Self::InvalidCapability(_) => "Capability policy input failed kernel validation",
        }
    }

    fn details(&self) -> Vec<FoundryPolicyApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::InvalidPolicyId { .. } => vec![detail("path.policy_id", "must be non-empty")],
            Self::PolicyIdMismatch { .. } => vec![detail(
                "policy_id",
                "path policy_id and body policy_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, authorization tenant, and body tenant_id must match",
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
                "must include foundry.policy.autonomy-ceiling.publish",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::PolicyAlreadyPublished { .. } => vec![detail(
                "body.policy_id",
                "autonomy policy already exists for tenant",
            )],
            Self::InvalidAutonomyTierLabel { field, .. } => vec![detail(
                field,
                "must be one of t1_view_only, t2_advisory, t3_execute_with_approval, t4_auto_execute",
            )],
            Self::InvalidCapabilityActionLabel { .. } => vec![detail(
                "body.capability_action",
                "must be one of other, ads_bid, ads_budget_adjust",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_classes",
                "must be canonical privacy data-class labels",
            )],
            Self::InvalidSubjectClassLabel { .. } => vec![detail(
                "body.subject_class",
                "must be one of adult, authority, elderly, vulnerable, minor_under13, minor_under14, minor_under16, minor_under19, minor_unknown",
            )],
            Self::MissingCedarPolicyRefs => vec![detail(
                "body.cedar_policy_refs",
                "must include at least one non-empty Cedar policy ref",
            )],
            Self::InvalidCapability(error) => {
                vec![detail("foundry_capability", capability_error_issue(error))]
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FoundryPolicyApiStatusKind {
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_foundry_policy_autonomy_ceiling_publish_request(
    request: &FoundryPolicyAutonomyCeilingPublishApiRequest,
) -> Result<(), FoundryPolicyApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_policy_id(&request.path_policy_id)?;
    if request.path_policy_id != request.body.policy_id {
        return Err(FoundryPolicyApiError::PolicyIdMismatch {
            path_policy_id: request.path_policy_id.clone(),
            body_policy_id: request.body.policy_id.clone(),
        });
    }
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &request.authorization,
        Some(&request.body.tenant_id),
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        FOUNDRY_POLICY_AUTONOMY_CEILING_PUBLISH_SURFACE,
    )?;
    Ok(())
}

pub fn publish_foundry_policy_autonomy_ceiling_from_api(
    directory: &mut FoundryPolicyAutonomyCeilingDirectory,
    idempotency_ledger: &mut FoundryPolicyAutonomyCeilingPublishIdempotencyLedger,
    request: FoundryPolicyAutonomyCeilingPublishApiRequest,
) -> Result<FoundryPolicyAutonomyCeilingPublishSuccessResponse, FoundryPolicyApiError> {
    validate_foundry_policy_autonomy_ceiling_publish_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        FOUNDRY_POLICY_AUTONOMY_CEILING_PUBLISH_SURFACE,
    );
    let fingerprint = autonomy_policy_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(FoundryPolicyApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let tenant_id = request.body.tenant_id.clone();
    let policy_id = request.body.policy_id.clone();
    let policy_key = FoundryPolicyAutonomyCeilingKey {
        tenant_id: tenant_id.clone(),
        policy_id: policy_id.clone(),
    };
    if directory.records.contains_key(&policy_key) {
        return Err(FoundryPolicyApiError::PolicyAlreadyPublished {
            tenant_id,
            policy_id,
        });
    }

    let record = autonomy_policy_record_from_request(request.body)?;
    let response =
        FoundryPolicyAutonomyCeilingPublishSuccessResponse::created(record.clone(), request_id);
    directory.records.insert(policy_key, record);
    idempotency_ledger.entries.insert(
        key,
        FoundryPolicyAutonomyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn autonomy_policy_record_from_request(
    request: FoundryPolicyAutonomyCeilingPublishRequest,
) -> Result<FoundryPolicyAutonomyCeilingRecord, FoundryPolicyApiError> {
    require_non_empty(&request.policy_id, "body.policy_id")?;
    require_non_empty(&request.tenant_id, "body.tenant_id")?;
    require_non_empty(&request.capability_id, "body.capability_id")?;
    require_non_empty(&request.policy_version, "body.policy_version")?;
    require_non_empty(&request.evidence_event_hash, "body.evidence_event_hash")?;
    validate_cedar_policy_refs(&request.cedar_policy_refs)?;

    let configured_ceiling = autonomy_tier_from_label(
        &request.tenant_configured_ceiling,
        "body.tenant_configured_ceiling",
    )?;
    let principal_ceiling =
        autonomy_tier_from_label(&request.principal_ceiling, "body.principal_ceiling")?;
    let capability_required_tier = autonomy_tier_from_label(
        &request.capability_required_tier,
        "body.capability_required_tier",
    )?;
    let capability_action = capability_action_from_label(&request.capability_action)?;
    let subject_class = subject_class_from_label(&request.subject_class)?;
    let data_classes = privacy_data_classes_from_labels(&request.data_classes)?;
    let capability = Capability::new_with_action(
        request.capability_id.clone(),
        capability_namespace(&request.capability_id),
        capability_action,
        capability_required_tier,
        data_classes,
        "oya.foundry.policy.autonomy_decision".to_string(),
    )
    .map_err(FoundryPolicyApiError::InvalidCapability)?;
    let decision = TenantPolicy::new(request.tenant_id.clone(), configured_ceiling)
        .evaluate_with_context(
            &capability,
            principal_ceiling,
            &request.regulatory_packs,
            subject_class,
        );

    Ok(record_from_decision(request, capability_action, decision))
}

fn record_from_decision(
    request: FoundryPolicyAutonomyCeilingPublishRequest,
    capability_action: CapabilityAction,
    decision: AutonomyDecision,
) -> FoundryPolicyAutonomyCeilingRecord {
    FoundryPolicyAutonomyCeilingRecord {
        policy_id: request.policy_id,
        tenant_id: request.tenant_id,
        capability_id: request.capability_id,
        policy_version: request.policy_version,
        capability_action: capability_action_label(capability_action).to_string(),
        data_classes: request.data_classes,
        regulatory_packs: request.regulatory_packs,
        subject_class: request.subject_class,
        configured_ceiling: tier_label(decision.configured_ceiling).to_string(),
        principal_ceiling: tier_label(decision.principal_ceiling).to_string(),
        capability_required_tier: tier_label(decision.capability_required_cap).to_string(),
        agentic_ads_cap: tier_label(decision.agentic_ads_cap).to_string(),
        vertical_pack_cap: tier_label(decision.vertical_pack_cap).to_string(),
        subject_class_cap: tier_label(decision.subject_class_cap).to_string(),
        denial_threshold: tier_label(decision.denial_threshold).to_string(),
        effective_ceiling: tier_label(decision.effective_ceiling).to_string(),
        verdict: autonomy_verdict_label(decision.verdict).to_string(),
        blocking_cap_source: decision
            .blocking_cap_source
            .map(|source| cap_source_label(source).to_string()),
        blocking_cap_reason: decision
            .blocking_cap_reason
            .map(|reason| cap_reason_label(reason).to_string()),
        lowering_cap_source: cap_source_label(decision.lowering_cap_source).to_string(),
        lowering_cap_reason: cap_reason_label(decision.lowering_cap_reason).to_string(),
        cedar_policy_refs: request.cedar_policy_refs,
        evidence_event_hash: request.evidence_event_hash,
        published_at_epoch_seconds: request.published_at_epoch_seconds,
        schema_version: 1,
    }
}

fn validate_boundary(
    boundary: &FoundryPolicyAutonomyBoundaryContext,
) -> Result<(), FoundryPolicyApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(FoundryPolicyApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(FoundryPolicyApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(FoundryPolicyApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_policy_id(policy_id: &str) -> Result<(), FoundryPolicyApiError> {
    if policy_id.trim().is_empty() {
        return Err(FoundryPolicyApiError::InvalidPolicyId {
            policy_id: policy_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    header_tenant_id: &str,
    principal: &FoundryPolicyApiPrincipal,
    authorization: &FoundryPolicyApiAuthorization,
    body_tenant_id: Option<&str>,
) -> Result<(), FoundryPolicyApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(FoundryPolicyApiError::EmptyPrincipalId);
    }
    if header_tenant_id != principal.tenant_id
        || header_tenant_id != authorization.tenant_id
        || body_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id)
    {
        return Err(FoundryPolicyApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            authorization_tenant_id: Some(authorization.tenant_id.clone()),
            body_tenant_id: body_tenant_id.map(str::to_string),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &FoundryPolicyApiPrincipal,
    authorization: &FoundryPolicyApiAuthorization,
    surface: &str,
) -> Result<(), FoundryPolicyApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(FoundryPolicyApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(FoundryPolicyApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(FoundryPolicyApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed| allowed == surface)
    {
        return Err(FoundryPolicyApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn require_non_empty(value: &str, field: &str) -> Result<(), FoundryPolicyApiError> {
    if value.trim().is_empty() {
        return Err(FoundryPolicyApiError::InvalidPolicyId {
            policy_id: field.to_string(),
        });
    }
    Ok(())
}

fn validate_cedar_policy_refs(refs: &[String]) -> Result<(), FoundryPolicyApiError> {
    if refs.is_empty() || refs.iter().any(|value| value.trim().is_empty()) {
        return Err(FoundryPolicyApiError::MissingCedarPolicyRefs);
    }
    Ok(())
}

fn privacy_data_classes_from_labels(
    labels: &[String],
) -> Result<Vec<PrivacyDataClass>, FoundryPolicyApiError> {
    if labels.is_empty() {
        return Err(FoundryPolicyApiError::InvalidDataClassLabel {
            data_class: String::new(),
        });
    }
    labels
        .iter()
        .map(|label| privacy_data_class_from_label(label))
        .collect()
}

fn privacy_data_class_from_label(label: &str) -> Result<PrivacyDataClass, FoundryPolicyApiError> {
    let parsed = parse_data_class_label(label).ok_or_else(|| {
        FoundryPolicyApiError::InvalidDataClassLabel {
            data_class: label.to_string(),
        }
    })?;
    PrivacyDataClass::try_from(parsed).map_err(|_| FoundryPolicyApiError::InvalidDataClassLabel {
        data_class: label.to_string(),
    })
}

fn autonomy_tier_from_label(
    autonomy_tier: &str,
    field: &str,
) -> Result<AutonomyTier, FoundryPolicyApiError> {
    match autonomy_tier.trim() {
        "t1_view_only" | "T1" | "T1ViewOnly" => Ok(AutonomyTier::T1ViewOnly),
        "t2_advisory" | "T2" | "T2Advisory" => Ok(AutonomyTier::T2Advisory),
        "t3_execute_with_approval" | "T3" | "T3ExecuteWithApproval" => {
            Ok(AutonomyTier::T3ExecuteWithApproval)
        }
        "t4_auto_execute" | "T4" | "T4AutoExecute" => Ok(AutonomyTier::T4AutoExecute),
        _ => Err(FoundryPolicyApiError::InvalidAutonomyTierLabel {
            field: field.to_string(),
            autonomy_tier: autonomy_tier.to_string(),
        }),
    }
}

fn tier_label(tier: AutonomyTier) -> &'static str {
    match tier {
        AutonomyTier::T1ViewOnly => "t1_view_only",
        AutonomyTier::T2Advisory => "t2_advisory",
        AutonomyTier::T3ExecuteWithApproval => "t3_execute_with_approval",
        AutonomyTier::T4AutoExecute => "t4_auto_execute",
    }
}

fn capability_action_from_label(
    capability_action: &str,
) -> Result<CapabilityAction, FoundryPolicyApiError> {
    match capability_action.trim() {
        "other" => Ok(CapabilityAction::Other),
        "ads_bid" => Ok(CapabilityAction::AdsBid),
        "ads_budget_adjust" => Ok(CapabilityAction::AdsBudgetAdjust),
        _ => Err(FoundryPolicyApiError::InvalidCapabilityActionLabel {
            capability_action: capability_action.to_string(),
        }),
    }
}

fn capability_action_label(capability_action: CapabilityAction) -> &'static str {
    match capability_action {
        CapabilityAction::Other => "other",
        CapabilityAction::AdsBid => "ads_bid",
        CapabilityAction::AdsBudgetAdjust => "ads_budget_adjust",
    }
}

fn subject_class_from_label(subject_class: &str) -> Result<SubjectClass, FoundryPolicyApiError> {
    match subject_class.trim() {
        "adult" => Ok(SubjectClass::Adult),
        "authority" => Ok(SubjectClass::Authority),
        "elderly" => Ok(SubjectClass::Elderly),
        "vulnerable" => Ok(SubjectClass::Vulnerable),
        "minor_under13" => Ok(SubjectClass::Minor {
            age_band: AgeBand::Under13,
        }),
        "minor_under14" => Ok(SubjectClass::Minor {
            age_band: AgeBand::Under14,
        }),
        "minor_under16" => Ok(SubjectClass::Minor {
            age_band: AgeBand::Under16,
        }),
        "minor_under19" => Ok(SubjectClass::Minor {
            age_band: AgeBand::Under19,
        }),
        "minor_unknown" => Ok(SubjectClass::Minor {
            age_band: AgeBand::UnknownMinor,
        }),
        _ => Err(FoundryPolicyApiError::InvalidSubjectClassLabel {
            subject_class: subject_class.to_string(),
        }),
    }
}

fn autonomy_verdict_label(verdict: AutonomyVerdict) -> &'static str {
    match verdict {
        AutonomyVerdict::Allow => "allow",
        AutonomyVerdict::Deny => "deny",
    }
}

fn cap_source_label(source: AutonomyCapSource) -> &'static str {
    source.as_str()
}

fn cap_reason_label(reason: AutonomyCapReason) -> &'static str {
    reason.as_str()
}

fn capability_namespace(capability_id: &str) -> String {
    capability_id
        .rsplit_once('.')
        .map(|(namespace, _)| namespace)
        .filter(|namespace| !namespace.trim().is_empty())
        .unwrap_or("foundry")
        .to_string()
}

fn capability_error_issue(error: &CapabilityError) -> &'static str {
    match error {
        CapabilityError::InvalidCapabilityId => "capability id must be non-empty and namespaced",
        CapabilityError::InvalidTenantId => "tenant id must be valid",
        CapabilityError::EmptyNamespace => "capability namespace must be non-empty",
        CapabilityError::EmptyEvidenceTopic => "evidence topic must be non-empty",
        CapabilityError::MissingDataClasses => {
            "capability must declare at least one privacy data class"
        }
        CapabilityError::NonPrivacyDataClass => {
            "capability data classes must be privacy-program classes"
        }
        CapabilityError::InvalidCostProfile => "capability cost profile is invalid",
        CapabilityError::MissingProviderPreference => "capability provider preference is required",
        CapabilityError::InvalidProviderPreference => "capability provider preference is invalid",
        CapabilityError::InvalidMcpContract => "capability MCP contract is invalid",
        CapabilityError::DuplicateCapability => "capability already exists",
        CapabilityError::CapabilityNotFound => "capability was not found",
    }
}

fn idempotency_key_for(
    boundary: &FoundryPolicyAutonomyBoundaryContext,
    principal: &FoundryPolicyApiPrincipal,
    surface: &str,
) -> FoundryPolicyIdempotencyLedgerKey {
    FoundryPolicyIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn autonomy_policy_fingerprint_for(
    request: &FoundryPolicyAutonomyCeilingPublishApiRequest,
) -> FoundryPolicyRequestFingerprint {
    FoundryPolicyRequestFingerprint {
        canonical: format!(
            "policy_id={}|tenant_id={}|capability_id={}|policy_version={}|configured={}|principal={}|required={}|action={}|data_classes={}|packs={}|subject={}|cedar={}|evidence={}|published_at={}",
            request.body.policy_id,
            request.body.tenant_id,
            request.body.capability_id,
            request.body.policy_version,
            request.body.tenant_configured_ceiling,
            request.body.principal_ceiling,
            request.body.capability_required_tier,
            request.body.capability_action,
            request.body.data_classes.join(","),
            request.body.regulatory_packs.join(","),
            request.body.subject_class,
            request.body.cedar_policy_refs.join(","),
            request.body.evidence_event_hash,
            request.body.published_at_epoch_seconds,
        ),
    }
}

fn detail(field: impl Into<String>, issue: impl Into<String>) -> FoundryPolicyApiErrorDetail {
    FoundryPolicyApiErrorDetail {
        field: field.into(),
        issue: issue.into(),
    }
}

#[allow(dead_code)]
fn data_class_label(data_class: DataClass) -> &'static str {
    data_class.label()
}
