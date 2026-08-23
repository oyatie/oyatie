//! Foundry registry API boundary.
//!
//! This crate projects the pure capability/eval kernels into the stable
//! `foundry.capability.publish` REST surface: transport binding validation,
//! Cedar authorization evidence, idempotency, eval publish gate enforcement,
//! and response/error DTOs.

use std::collections::BTreeMap;

use data_boundary_kernel::{PrivacyDataClass, parse_data_class_label};
use governance_eval_domain::{
    AdversarialKind, EvalCaseInput, EvalError, EvalGate, EvalMetric, EvalRunInput, EvalSetInput,
};
use intelligence_capability_domain::{
    AutonomyTier, Capability, CapabilityCostProfile, CapabilityError, CapabilityMcpContract,
    CapabilityRegistry,
};

const FOUNDRY_CAPABILITY_PUBLISH_SCHEMA_VERSION: u32 = 1;

pub const FOUNDRY_CAPABILITY_PUBLISH_SURFACE: &str = "foundry.capability.publish";
pub const FOUNDRY_REGISTRY_OPENAPI_CONTRACT: &str = "contracts/openapi/foundry/registry-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundryCapabilityPublishApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl FoundryCapabilityPublishApiStatus {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityDescriptionRequest {
    pub agent_readable: String, // data_class: PUBLIC
    pub human_readable: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityProviderRequest {
    pub preferred: String,     // data_class: INTERNAL_ONLY
    pub fallback: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityCostProfileRequest {
    pub per_invocation_limit_micros: u64, // data_class: INTERNAL_ONLY
    pub per_tenant_monthly_limit_micros: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityEvalCaseRequest {
    pub case_id: String,                  // data_class: INTERNAL_ONLY
    pub locale: String,                   // data_class: INTERNAL_ONLY
    pub input_ref: String,                // data_class: INTERNAL_ONLY
    pub expected_ref: String,             // data_class: INTERNAL_ONLY
    pub adversarial_kind: Option<String>, // data_class: INTERNAL_ONLY
    pub deterministic_seed: Option<u64>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityPublishRequest {
    pub tenant_id: String,     // data_class: INTERNAL_ONLY
    pub capability_id: String, // data_class: INTERNAL_ONLY
    pub namespace: String,     // data_class: PUBLIC
    pub version: String,       // data_class: PUBLIC
    pub description: FoundryCapabilityDescriptionRequest, // data_class: PUBLIC
    pub provider: FoundryCapabilityProviderRequest, // data_class: INTERNAL_ONLY
    pub autonomy_tier_required: String, // data_class: INTERNAL_ONLY
    pub data_classes_touched: Vec<String>, // data_class: INTERNAL_ONLY
    pub evidence_emission_topic: String, // data_class: INTERNAL_ONLY
    pub cost_profile: FoundryCapabilityCostProfileRequest, // data_class: INTERNAL_ONLY
    pub input_schema_json: String, // data_class: PUBLIC
    pub output_schema_json: String, // data_class: PUBLIC
    pub eval_set_version: String, // data_class: INTERNAL_ONLY
    pub eval_metric: String,   // data_class: INTERNAL_ONLY
    pub min_pass_rate_percent: u8, // data_class: INTERNAL_ONLY
    pub min_p95_score_percent: u8, // data_class: INTERNAL_ONLY
    pub signed_eval_set: bool, // data_class: AUDIT
    pub eval_cases: Vec<FoundryCapabilityEvalCaseRequest>, // data_class: AUDIT
    pub eval_pass_rate_percent: u8, // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub eval_p95_score_percent: u8, // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub eval_adversarial_passed: bool, // data_class: AUDIT
    pub eval_linguistic_passed: bool, // data_class: AUDIT
    pub signed_eval_run: bool, // data_class: AUDIT
    pub owner_team: String,    // data_class: PUBLIC
    pub catalog_record_path: String, // data_class: PUBLIC
    pub docs_path: String,     // data_class: PUBLIC
    pub published_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityPublishApiRequest {
    pub path_capability_id: String, // data_class: INTERNAL_ONLY
    pub boundary: FoundryCapabilityApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: FoundryCapabilityApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: FoundryCapabilityApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: FoundryCapabilityPublishRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityPublishSuccessResponse {
    pub data: FoundryCapabilityPublishRecord, // data_class: INTERNAL_ONLY
    pub metadata: FoundryCapabilityPublishMetadata, // data_class: INTERNAL_ONLY
}

impl FoundryCapabilityPublishSuccessResponse {
    pub const fn status_code(&self) -> u16 {
        FoundryCapabilityPublishApiStatus::Created.code()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityPublishRecord {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub capability_id: String,             // data_class: INTERNAL_ONLY
    pub namespace: String,                 // data_class: PUBLIC
    pub version: String,                   // data_class: PUBLIC
    pub owner_team: String,                // data_class: PUBLIC
    pub autonomy_tier_required: String,    // data_class: INTERNAL_ONLY
    pub data_classes_touched: Vec<String>, // data_class: INTERNAL_ONLY
    pub provider_preference: Vec<String>,  // data_class: INTERNAL_ONLY
    pub evidence_emission_topic: String,   // data_class: INTERNAL_ONLY
    pub eval_set_version: String,          // data_class: INTERNAL_ONLY
    pub eval_pass_rate_percent: u8,        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub eval_p95_score_percent: u8,        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub eval_case_count: u64,              // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub schema_version: u32,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityPublishMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String,  // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityPublishApiErrorResponse {
    pub error: FoundryCapabilityPublishApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityPublishApiErrorBody {
    pub code: String,                      // data_class: INTERNAL_ONLY
    pub message: String,                   // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>, // data_class: INTERNAL_ONLY
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub details: Vec<FoundryCapabilityPublishApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryCapabilityPublishApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FoundryCapabilityPublishApiError {
    EmptyPathCapabilityId,
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalTenantId,
    EmptyPrincipalId,
    EmptyAuthorizationDecisionId,
    CapabilityIdMismatch {
        path_capability_id: String, // data_class: INTERNAL_ONLY
        body_capability_id: String, // data_class: INTERNAL_ONLY
    },
    TenantMismatch {
        header_tenant_id: String,        // data_class: INTERNAL_ONLY
        principal_tenant_id: String,     // data_class: INTERNAL_ONLY
        authorization_tenant_id: String, // data_class: INTERNAL_ONLY
        body_tenant_id: String,          // data_class: INTERNAL_ONLY
    },
    AuthorizationPrincipalMismatch {
        principal_tenant_id: String,        // data_class: INTERNAL_ONLY
        principal_id: String,               // data_class: INTERNAL_ONLY
        authorization_tenant_id: String,    // data_class: INTERNAL_ONLY
        authorization_principal_id: String, // data_class: INTERNAL_ONLY
    },
    AuthorizationSurfaceDenied {
        decision_id: String, // data_class: INTERNAL_ONLY
        surface: String,     // data_class: INTERNAL_ONLY
    },
    InvalidCapabilityId,
    EmptyVersion,
    EmptyOwnerTeam,
    EmptyCatalogRecordPath,
    EmptyDocsPath,
    EmptyNamespace,
    EmptyEvidenceTopic,
    MissingDataClasses,
    InvalidDataClass {
        data_class: String, // data_class: INTERNAL_ONLY
    },
    InvalidAutonomyTier {
        autonomy_tier: String, // data_class: INTERNAL_ONLY
    },
    InvalidCostProfile,
    MissingProviderPreference,
    InvalidProviderPreference,
    InvalidMcpContract,
    EmptyEvalSetVersion,
    EmptyEvalCaseId,
    EmptyEvalLocale,
    EmptyEvalInputRef,
    EmptyEvalExpectedRef,
    InvalidEvalThreshold,
    InvalidEvalMetric {
        metric: String, // data_class: INTERNAL_ONLY
    },
    InvalidAdversarialKind {
        adversarial_kind: String, // data_class: INTERNAL_ONLY
    },
    EmptyEvalSet,
    UnsignedEvalSet,
    MissingAdversarialCoverage,
    MissingLinguisticCoverage,
    EvalSetNotFound,
    UnsignedEvalRun,
    EvalRunVersionMismatch,
    EvalRunBelowThreshold,
    MissingPassingEvalRun,
    DuplicateCapability {
        capability_id: String, // data_class: INTERNAL_ONLY
    },
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundryCapabilityPublishApiErrorCode {
    PathCapabilityIdEmpty,
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalTenantIdEmpty,
    PrincipalIdEmpty,
    AuthorizationDecisionIdEmpty,
    CapabilityIdMismatch,
    TenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationSurfaceDenied,
    CapabilityIdInvalid,
    VersionEmpty,
    OwnerTeamEmpty,
    CatalogRecordPathEmpty,
    DocsPathEmpty,
    NamespaceEmpty,
    EvidenceTopicEmpty,
    DataClassesMissing,
    DataClassInvalid,
    AutonomyTierInvalid,
    CostProfileInvalid,
    ProviderPreferenceMissing,
    ProviderPreferenceInvalid,
    McpContractInvalid,
    EvalSetVersionEmpty,
    EvalCaseIdEmpty,
    EvalLocaleEmpty,
    EvalInputRefEmpty,
    EvalExpectedRefEmpty,
    EvalThresholdInvalid,
    EvalMetricInvalid,
    EvalAdversarialKindInvalid,
    EvalSetEmpty,
    EvalSetUnsigned,
    EvalAdversarialCoverageMissing,
    EvalLinguisticCoverageMissing,
    EvalSetNotFound,
    EvalRunUnsigned,
    EvalRunVersionMismatch,
    EvalRunBelowThreshold,
    EvalRunMissingPassing,
    DuplicateCapability,
    IdempotencyKeyReused,
}

impl FoundryCapabilityPublishApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PathCapabilityIdEmpty => "PATH_CAPABILITY_ID_EMPTY",
            Self::RequestIdEmpty => "REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalTenantIdEmpty => "PRINCIPAL_TENANT_ID_EMPTY",
            Self::PrincipalIdEmpty => "PRINCIPAL_ID_EMPTY",
            Self::AuthorizationDecisionIdEmpty => "AUTHORIZATION_DECISION_ID_EMPTY",
            Self::CapabilityIdMismatch => "CAPABILITY_ID_MISMATCH",
            Self::TenantMismatch => "TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationSurfaceDenied => "AUTHORIZATION_SURFACE_DENIED",
            Self::CapabilityIdInvalid => "CAPABILITY_ID_INVALID",
            Self::VersionEmpty => "VERSION_EMPTY",
            Self::OwnerTeamEmpty => "OWNER_TEAM_EMPTY",
            Self::CatalogRecordPathEmpty => "CATALOG_RECORD_PATH_EMPTY",
            Self::DocsPathEmpty => "DOCS_PATH_EMPTY",
            Self::NamespaceEmpty => "NAMESPACE_EMPTY",
            Self::EvidenceTopicEmpty => "EVIDENCE_TOPIC_EMPTY",
            Self::DataClassesMissing => "DATA_CLASSES_MISSING",
            Self::DataClassInvalid => "DATA_CLASS_INVALID",
            Self::AutonomyTierInvalid => "AUTONOMY_TIER_INVALID",
            Self::CostProfileInvalid => "COST_PROFILE_INVALID",
            Self::ProviderPreferenceMissing => "PROVIDER_PREFERENCE_MISSING",
            Self::ProviderPreferenceInvalid => "PROVIDER_PREFERENCE_INVALID",
            Self::McpContractInvalid => "MCP_CONTRACT_INVALID",
            Self::EvalSetVersionEmpty => "EVAL_SET_VERSION_EMPTY",
            Self::EvalCaseIdEmpty => "EVAL_CASE_ID_EMPTY",
            Self::EvalLocaleEmpty => "EVAL_LOCALE_EMPTY",
            Self::EvalInputRefEmpty => "EVAL_INPUT_REF_EMPTY",
            Self::EvalExpectedRefEmpty => "EVAL_EXPECTED_REF_EMPTY",
            Self::EvalThresholdInvalid => "EVAL_THRESHOLD_INVALID",
            Self::EvalMetricInvalid => "EVAL_METRIC_INVALID",
            Self::EvalAdversarialKindInvalid => "EVAL_ADVERSARIAL_KIND_INVALID",
            Self::EvalSetEmpty => "EVAL_SET_EMPTY",
            Self::EvalSetUnsigned => "EVAL_SET_UNSIGNED",
            Self::EvalAdversarialCoverageMissing => "EVAL_ADVERSARIAL_COVERAGE_MISSING",
            Self::EvalLinguisticCoverageMissing => "EVAL_LINGUISTIC_COVERAGE_MISSING",
            Self::EvalSetNotFound => "EVAL_SET_NOT_FOUND",
            Self::EvalRunUnsigned => "EVAL_RUN_UNSIGNED",
            Self::EvalRunVersionMismatch => "EVAL_RUN_VERSION_MISMATCH",
            Self::EvalRunBelowThreshold => "EVAL_RUN_BELOW_THRESHOLD",
            Self::EvalRunMissingPassing => "EVAL_RUN_MISSING_PASSING",
            Self::DuplicateCapability => "DUPLICATE_CAPABILITY",
            Self::IdempotencyKeyReused => "IDEMPOTENCY_KEY_REUSED",
        }
    }
}

impl FoundryCapabilityPublishApiError {
    pub fn status(&self) -> FoundryCapabilityPublishApiStatus {
        match self {
            Self::EmptyPrincipalTenantId | Self::EmptyPrincipalId => {
                FoundryCapabilityPublishApiStatus::Unauthorized
            }
            Self::TenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationSurfaceDenied { .. }
            | Self::EmptyAuthorizationDecisionId => FoundryCapabilityPublishApiStatus::Forbidden,
            Self::DuplicateCapability { .. } => FoundryCapabilityPublishApiStatus::Conflict,
            Self::UnsignedEvalSet
            | Self::UnsignedEvalRun
            | Self::EvalRunBelowThreshold
            | Self::EvalRunVersionMismatch
            | Self::EvalSetNotFound
            | Self::MissingPassingEvalRun
            | Self::MissingAdversarialCoverage
            | Self::MissingLinguisticCoverage
            | Self::IdempotencyKeyReused { .. } => {
                FoundryCapabilityPublishApiStatus::UnprocessableEntity
            }
            _ => FoundryCapabilityPublishApiStatus::BadRequest,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> FoundryCapabilityPublishApiErrorCode {
        match self {
            Self::EmptyPathCapabilityId => {
                FoundryCapabilityPublishApiErrorCode::PathCapabilityIdEmpty
            }
            Self::EmptyRequestId => FoundryCapabilityPublishApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => FoundryCapabilityPublishApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => FoundryCapabilityPublishApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalTenantId => {
                FoundryCapabilityPublishApiErrorCode::PrincipalTenantIdEmpty
            }
            Self::EmptyPrincipalId => FoundryCapabilityPublishApiErrorCode::PrincipalIdEmpty,
            Self::EmptyAuthorizationDecisionId => {
                FoundryCapabilityPublishApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::CapabilityIdMismatch { .. } => {
                FoundryCapabilityPublishApiErrorCode::CapabilityIdMismatch
            }
            Self::TenantMismatch { .. } => FoundryCapabilityPublishApiErrorCode::TenantMismatch,
            Self::AuthorizationPrincipalMismatch { .. } => {
                FoundryCapabilityPublishApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationSurfaceDenied { .. } => {
                FoundryCapabilityPublishApiErrorCode::AuthorizationSurfaceDenied
            }
            Self::InvalidCapabilityId => FoundryCapabilityPublishApiErrorCode::CapabilityIdInvalid,
            Self::EmptyVersion => FoundryCapabilityPublishApiErrorCode::VersionEmpty,
            Self::EmptyOwnerTeam => FoundryCapabilityPublishApiErrorCode::OwnerTeamEmpty,
            Self::EmptyCatalogRecordPath => {
                FoundryCapabilityPublishApiErrorCode::CatalogRecordPathEmpty
            }
            Self::EmptyDocsPath => FoundryCapabilityPublishApiErrorCode::DocsPathEmpty,
            Self::EmptyNamespace => FoundryCapabilityPublishApiErrorCode::NamespaceEmpty,
            Self::EmptyEvidenceTopic => FoundryCapabilityPublishApiErrorCode::EvidenceTopicEmpty,
            Self::MissingDataClasses => FoundryCapabilityPublishApiErrorCode::DataClassesMissing,
            Self::InvalidDataClass { .. } => FoundryCapabilityPublishApiErrorCode::DataClassInvalid,
            Self::InvalidAutonomyTier { .. } => {
                FoundryCapabilityPublishApiErrorCode::AutonomyTierInvalid
            }
            Self::InvalidCostProfile => FoundryCapabilityPublishApiErrorCode::CostProfileInvalid,
            Self::MissingProviderPreference => {
                FoundryCapabilityPublishApiErrorCode::ProviderPreferenceMissing
            }
            Self::InvalidProviderPreference => {
                FoundryCapabilityPublishApiErrorCode::ProviderPreferenceInvalid
            }
            Self::InvalidMcpContract => FoundryCapabilityPublishApiErrorCode::McpContractInvalid,
            Self::EmptyEvalSetVersion => FoundryCapabilityPublishApiErrorCode::EvalSetVersionEmpty,
            Self::EmptyEvalCaseId => FoundryCapabilityPublishApiErrorCode::EvalCaseIdEmpty,
            Self::EmptyEvalLocale => FoundryCapabilityPublishApiErrorCode::EvalLocaleEmpty,
            Self::EmptyEvalInputRef => FoundryCapabilityPublishApiErrorCode::EvalInputRefEmpty,
            Self::EmptyEvalExpectedRef => {
                FoundryCapabilityPublishApiErrorCode::EvalExpectedRefEmpty
            }
            Self::InvalidEvalThreshold => {
                FoundryCapabilityPublishApiErrorCode::EvalThresholdInvalid
            }
            Self::InvalidEvalMetric { .. } => {
                FoundryCapabilityPublishApiErrorCode::EvalMetricInvalid
            }
            Self::InvalidAdversarialKind { .. } => {
                FoundryCapabilityPublishApiErrorCode::EvalAdversarialKindInvalid
            }
            Self::EmptyEvalSet => FoundryCapabilityPublishApiErrorCode::EvalSetEmpty,
            Self::UnsignedEvalSet => FoundryCapabilityPublishApiErrorCode::EvalSetUnsigned,
            Self::MissingAdversarialCoverage => {
                FoundryCapabilityPublishApiErrorCode::EvalAdversarialCoverageMissing
            }
            Self::MissingLinguisticCoverage => {
                FoundryCapabilityPublishApiErrorCode::EvalLinguisticCoverageMissing
            }
            Self::EvalSetNotFound => FoundryCapabilityPublishApiErrorCode::EvalSetNotFound,
            Self::UnsignedEvalRun => FoundryCapabilityPublishApiErrorCode::EvalRunUnsigned,
            Self::EvalRunVersionMismatch => {
                FoundryCapabilityPublishApiErrorCode::EvalRunVersionMismatch
            }
            Self::EvalRunBelowThreshold => {
                FoundryCapabilityPublishApiErrorCode::EvalRunBelowThreshold
            }
            Self::MissingPassingEvalRun => {
                FoundryCapabilityPublishApiErrorCode::EvalRunMissingPassing
            }
            Self::DuplicateCapability { .. } => {
                FoundryCapabilityPublishApiErrorCode::DuplicateCapability
            }
            Self::IdempotencyKeyReused { .. } => {
                FoundryCapabilityPublishApiErrorCode::IdempotencyKeyReused
            }
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> FoundryCapabilityPublishApiErrorResponse {
        FoundryCapabilityPublishApiErrorResponse {
            error: FoundryCapabilityPublishApiErrorBody {
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
            Self::EmptyPrincipalTenantId => "Authenticated principal tenant is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::CapabilityIdMismatch { .. } => "Path and body capability ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, authorization, and body"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization principal binding must match authenticated principal"
            }
            Self::AuthorizationSurfaceDenied { .. } => {
                "Authorization decision does not grant foundry.capability.publish"
            }
            Self::InvalidCapabilityId => "Capability id must use the cap. namespace",
            Self::EmptyVersion => "Capability version is required",
            Self::EmptyOwnerTeam => "Capability owner team is required",
            Self::EmptyCatalogRecordPath => "Capability catalog record path is required",
            Self::EmptyDocsPath => "Capability docs path is required",
            Self::EmptyNamespace => "Capability namespace is required",
            Self::EmptyEvidenceTopic => "Capability evidence emission topic is required",
            Self::MissingDataClasses => "Capability data classes are required",
            Self::InvalidDataClass { .. } => "Capability data class is not a privacy data class",
            Self::InvalidAutonomyTier { .. } => "Capability autonomy tier is not supported",
            Self::InvalidCostProfile => "Capability cost profile is invalid",
            Self::MissingProviderPreference => "Capability provider preference is required",
            Self::InvalidProviderPreference => "Capability provider preference is invalid",
            Self::InvalidMcpContract => "Capability MCP contract is invalid",
            Self::EmptyEvalSetVersion => "Eval set version is required",
            Self::EmptyEvalCaseId => "Eval case id is required",
            Self::EmptyEvalLocale => "Eval case locale is required",
            Self::EmptyEvalInputRef => "Eval case input reference is required",
            Self::EmptyEvalExpectedRef => "Eval case expected reference is required",
            Self::InvalidEvalThreshold => "Eval thresholds must be between 1 and 100",
            Self::InvalidEvalMetric { .. } => "Eval metric is not supported",
            Self::InvalidAdversarialKind { .. } => "Adversarial kind is not supported",
            Self::EmptyEvalSet => "Capability eval set must include at least one case",
            Self::UnsignedEvalSet => "Capability eval set must be signed before publish",
            Self::MissingAdversarialCoverage => {
                "Capability eval set must include all mandatory adversarial cohorts"
            }
            Self::MissingLinguisticCoverage => {
                "Capability eval set must include all mandatory linguistic cohorts"
            }
            Self::EvalSetNotFound => "Capability eval set was not registered",
            Self::UnsignedEvalRun => "Capability eval run must be signed before publish",
            Self::EvalRunVersionMismatch => "Capability eval run version must match the eval set",
            Self::EvalRunBelowThreshold => "Capability eval run did not meet the publish threshold",
            Self::MissingPassingEvalRun => "No passing eval run is available for this capability",
            Self::DuplicateCapability { .. } => "Capability has already been published",
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different capability publish request"
            }
        }
    }

    fn details(&self) -> Vec<FoundryCapabilityPublishApiErrorDetail> {
        match self {
            Self::CapabilityIdMismatch {
                path_capability_id,
                body_capability_id,
            } => vec![detail(
                "path.capability_id",
                format!("{path_capability_id} does not match body capability {body_capability_id}"),
            )],
            Self::TenantMismatch {
                header_tenant_id,
                principal_tenant_id,
                authorization_tenant_id,
                body_tenant_id,
            } => vec![detail(
                "X-Tenant-Id",
                format!(
                    "header={header_tenant_id}; principal={principal_tenant_id}; authorization={authorization_tenant_id}; body={body_tenant_id}"
                ),
            )],
            Self::AuthorizationPrincipalMismatch {
                principal_tenant_id,
                principal_id,
                authorization_tenant_id,
                authorization_principal_id,
            } => vec![detail(
                "authorization.principal_id",
                format!(
                    "principal={principal_tenant_id}/{principal_id}; authorization={authorization_tenant_id}/{authorization_principal_id}"
                ),
            )],
            Self::AuthorizationSurfaceDenied {
                decision_id,
                surface,
            } => vec![detail(
                "authorization.allowed_surfaces",
                format!("decision {decision_id} does not include {surface}"),
            )],
            Self::InvalidDataClass { data_class } => vec![detail(
                "body.data_classes_touched",
                format!("unsupported privacy data class {data_class}"),
            )],
            Self::InvalidAutonomyTier { autonomy_tier } => vec![detail(
                "body.autonomy_tier_required",
                format!("unsupported autonomy tier {autonomy_tier}"),
            )],
            Self::InvalidEvalMetric { metric } => {
                vec![detail(
                    "body.eval_metric",
                    format!("unsupported metric {metric}"),
                )]
            }
            Self::InvalidAdversarialKind { adversarial_kind } => vec![detail(
                "body.eval_cases[].adversarial_kind",
                format!("unsupported adversarial kind {adversarial_kind}"),
            )],
            Self::MissingAdversarialCoverage => vec![detail(
                "body.eval_cases",
                "missing prompt-injection, data-class-violation, autonomy-bypass, or tool-exfiltration cohort",
            )],
            Self::MissingLinguisticCoverage => vec![detail(
                "body.eval_cases",
                "missing one or more mandatory locale cohorts",
            )],
            Self::DuplicateCapability { capability_id } => vec![detail(
                "body.capability_id",
                format!("{capability_id} already exists in the registry"),
            )],
            Self::IdempotencyKeyReused { idempotency_key } => vec![detail(
                "Idempotency-Key",
                format!("{idempotency_key} is already bound to a different fingerprint"),
            )],
            Self::EmptyPathCapabilityId => vec![detail("path.capability_id", "must not be empty")],
            Self::EmptyRequestId => vec![detail("X-Request-Id", "must not be empty")],
            Self::EmptyTenantHeader => vec![detail("X-Tenant-Id", "must not be empty")],
            Self::EmptyIdempotencyKey => vec![detail("Idempotency-Key", "must not be empty")],
            Self::EmptyPrincipalTenantId => {
                vec![detail("principal.tenant_id", "must not be empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must not be empty")],
            Self::EmptyAuthorizationDecisionId => {
                vec![detail("authorization.decision_id", "must not be empty")]
            }
            Self::InvalidCapabilityId => vec![detail("body.capability_id", "must start with cap.")],
            Self::EmptyVersion => vec![detail("body.version", "must not be empty")],
            Self::EmptyOwnerTeam => vec![detail("body.owner_team", "must not be empty")],
            Self::EmptyCatalogRecordPath => {
                vec![detail("body.catalog_record_path", "must not be empty")]
            }
            Self::EmptyDocsPath => vec![detail("body.docs_path", "must not be empty")],
            Self::EmptyNamespace => vec![detail("body.namespace", "must not be empty")],
            Self::EmptyEvidenceTopic => {
                vec![detail("body.evidence_emission_topic", "must not be empty")]
            }
            Self::MissingDataClasses => {
                vec![detail("body.data_classes_touched", "must not be empty")]
            }
            Self::InvalidCostProfile => {
                vec![detail("body.cost_profile", "invalid budget ceilings")]
            }
            Self::MissingProviderPreference => {
                vec![detail("body.provider.preferred", "must not be empty")]
            }
            Self::InvalidProviderPreference => vec![detail(
                "body.provider",
                "provider ids must be unique lowercase ids",
            )],
            Self::InvalidMcpContract => vec![detail(
                "body.input_schema_json/body.output_schema_json",
                "schemas must be JSON objects",
            )],
            Self::EmptyEvalSetVersion => vec![detail("body.eval_set_version", "must not be empty")],
            Self::EmptyEvalCaseId => vec![detail("body.eval_cases[].case_id", "must not be empty")],
            Self::EmptyEvalLocale => vec![detail("body.eval_cases[].locale", "must not be empty")],
            Self::EmptyEvalInputRef => {
                vec![detail("body.eval_cases[].input_ref", "must not be empty")]
            }
            Self::EmptyEvalExpectedRef => vec![detail(
                "body.eval_cases[].expected_ref",
                "must not be empty",
            )],
            Self::InvalidEvalThreshold => vec![detail(
                "body.min_pass_rate_percent/body.min_p95_score_percent",
                "must be 1..=100",
            )],
            Self::EmptyEvalSet => vec![detail("body.eval_cases", "must not be empty")],
            Self::UnsignedEvalSet => vec![detail("body.signed_eval_set", "must be true")],
            Self::EvalSetNotFound => vec![detail("body.capability_id", "eval set is missing")],
            Self::UnsignedEvalRun => vec![detail("body.signed_eval_run", "must be true")],
            Self::EvalRunVersionMismatch => vec![detail(
                "body.eval_set_version",
                "run version must equal eval set version",
            )],
            Self::EvalRunBelowThreshold => vec![detail(
                "body.eval_pass_rate_percent/body.eval_p95_score_percent",
                "run scores or cohorts did not meet thresholds",
            )],
            Self::MissingPassingEvalRun => {
                vec![detail("body.capability_id", "no passing run recorded")]
            }
        }
    }
}

fn detail(
    field: impl Into<String>,
    issue: impl Into<String>,
) -> FoundryCapabilityPublishApiErrorDetail {
    FoundryCapabilityPublishApiErrorDetail {
        field: field.into(),
        issue: issue.into(),
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryCapabilityPublishDirectory {
    capability_registry: CapabilityRegistry, // data_class: INTERNAL_ONLY
    eval_gate: EvalGate,                     // data_class: INTERNAL_ONLY
    records: BTreeMap<FoundryCapabilityPublishRecordKey, FoundryCapabilityPublishRecord>, // data_class: INTERNAL_ONLY
}

impl FoundryCapabilityPublishDirectory {
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn get(
        &self,
        tenant_id: &str,
        capability_id: &str,
    ) -> Option<&FoundryCapabilityPublishRecord> {
        self.records.get(&FoundryCapabilityPublishRecordKey {
            tenant_id: tenant_id.to_string(),
            capability_id: capability_id.to_string(),
        })
    }

    fn publish(
        &mut self,
        request: &FoundryCapabilityPublishApiRequest,
    ) -> Result<FoundryCapabilityPublishRecord, FoundryCapabilityPublishApiError> {
        ensure_body_non_empty(
            &request.body.version,
            FoundryCapabilityPublishApiError::EmptyVersion,
        )?;
        ensure_body_non_empty(
            &request.body.owner_team,
            FoundryCapabilityPublishApiError::EmptyOwnerTeam,
        )?;
        ensure_body_non_empty(
            &request.body.catalog_record_path,
            FoundryCapabilityPublishApiError::EmptyCatalogRecordPath,
        )?;
        ensure_body_non_empty(
            &request.body.docs_path,
            FoundryCapabilityPublishApiError::EmptyDocsPath,
        )?;
        if self
            .capability_registry
            .get(&request.body.capability_id)
            .is_some()
        {
            return Err(FoundryCapabilityPublishApiError::DuplicateCapability {
                capability_id: request.body.capability_id.clone(),
            });
        }
        let capability = request.body.to_kernel_capability()?;
        let mut working_eval_gate = self.eval_gate.clone();
        working_eval_gate
            .register_eval_set(request.body.to_eval_set_input()?)
            .map_err(FoundryCapabilityPublishApiError::from_eval_kernel)?;
        working_eval_gate
            .record_run(request.body.to_eval_run_input())
            .map_err(FoundryCapabilityPublishApiError::from_eval_kernel)?;
        working_eval_gate
            .assert_publish_ready(&request.body.capability_id)
            .map_err(FoundryCapabilityPublishApiError::from_eval_kernel)?;

        self.capability_registry
            .publish(capability)
            .map_err(|error| match error {
                CapabilityError::DuplicateCapability => {
                    FoundryCapabilityPublishApiError::DuplicateCapability {
                        capability_id: request.body.capability_id.clone(),
                    }
                }
                other => FoundryCapabilityPublishApiError::from_capability_kernel(other),
            })?;
        self.eval_gate = working_eval_gate;

        let record = FoundryCapabilityPublishRecord {
            tenant_id: request.body.tenant_id.clone(),
            capability_id: request.body.capability_id.clone(),
            namespace: request.body.namespace.clone(),
            version: request.body.version.clone(),
            owner_team: request.body.owner_team.clone(),
            autonomy_tier_required: request.body.autonomy_tier_required.clone(),
            data_classes_touched: request.body.data_classes_touched.clone(),
            provider_preference: request.body.provider.preference(),
            evidence_emission_topic: request.body.evidence_emission_topic.clone(),
            eval_set_version: request.body.eval_set_version.clone(),
            eval_pass_rate_percent: request.body.eval_pass_rate_percent,
            eval_p95_score_percent: request.body.eval_p95_score_percent,
            eval_case_count: request.body.eval_cases.len() as u64,
            published_at_epoch_seconds: request.body.published_at_epoch_seconds,
            schema_version: FOUNDRY_CAPABILITY_PUBLISH_SCHEMA_VERSION,
        };
        self.records.insert(
            FoundryCapabilityPublishRecordKey {
                tenant_id: record.tenant_id.clone(),
                capability_id: record.capability_id.clone(),
            },
            record.clone(),
        );
        Ok(record)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FoundryCapabilityPublishRecordKey {
    tenant_id: String,     // data_class: INTERNAL_ONLY
    capability_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryCapabilityPublishIdempotencyLedger {
    entries: BTreeMap<
        FoundryCapabilityPublishIdempotencyLedgerKey,
        FoundryCapabilityPublishIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl FoundryCapabilityPublishIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FoundryCapabilityPublishIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryCapabilityPublishIdempotencyLedgerEntry {
    fingerprint: FoundryCapabilityPublishRequestFingerprint, // data_class: INTERNAL_ONLY
    response: FoundryCapabilityPublishSuccessResponse,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryCapabilityPublishRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

impl FoundryCapabilityProviderRequest {
    fn preference(&self) -> Vec<String> {
        let mut preference = vec![self.preferred.clone()];
        preference.extend(self.fallback.clone());
        preference
    }
}

impl FoundryCapabilityPublishRequest {
    fn to_kernel_capability(&self) -> Result<Capability, FoundryCapabilityPublishApiError> {
        let required_tier = parse_autonomy_tier(&self.autonomy_tier_required)?;
        let data_classes = parse_privacy_data_classes(&self.data_classes_touched)?;
        let cost_profile = CapabilityCostProfile::new(
            self.cost_profile.per_invocation_limit_micros,
            self.cost_profile.per_tenant_monthly_limit_micros,
            self.provider.preference(),
        )
        .map_err(FoundryCapabilityPublishApiError::from_capability_kernel)?;
        let mcp_contract = CapabilityMcpContract::new(
            self.description.agent_readable.clone(),
            self.description.human_readable.clone(),
            self.input_schema_json.clone(),
            self.output_schema_json.clone(),
        )
        .map_err(FoundryCapabilityPublishApiError::from_capability_kernel)?;
        Capability::new_with_cost_profile_and_mcp_contract(
            self.capability_id.clone(),
            self.namespace.clone(),
            required_tier,
            data_classes,
            self.evidence_emission_topic.clone(),
            cost_profile,
            mcp_contract,
        )
        .map_err(FoundryCapabilityPublishApiError::from_capability_kernel)
    }

    fn to_eval_set_input(&self) -> Result<EvalSetInput, FoundryCapabilityPublishApiError> {
        Ok(EvalSetInput {
            capability_id: self.capability_id.clone(),
            version: self.eval_set_version.clone(),
            metric: parse_metric(&self.eval_metric)?,
            min_pass_rate_percent: self.min_pass_rate_percent,
            min_p95_score_percent: self.min_p95_score_percent,
            signed: self.signed_eval_set,
            cases: self
                .eval_cases
                .iter()
                .map(FoundryCapabilityEvalCaseRequest::to_kernel)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn to_eval_run_input(&self) -> EvalRunInput {
        EvalRunInput {
            capability_id: self.capability_id.clone(),
            eval_set_version: self.eval_set_version.clone(),
            pass_rate_percent: self.eval_pass_rate_percent,
            p95_score_percent: self.eval_p95_score_percent,
            adversarial_passed: self.eval_adversarial_passed,
            linguistic_passed: self.eval_linguistic_passed,
            signed: self.signed_eval_run,
        }
    }
}

impl FoundryCapabilityEvalCaseRequest {
    fn to_kernel(&self) -> Result<EvalCaseInput, FoundryCapabilityPublishApiError> {
        Ok(EvalCaseInput {
            case_id: self.case_id.clone(),
            locale: self.locale.clone(),
            input_ref: self.input_ref.clone(),
            expected_ref: self.expected_ref.clone(),
            adversarial_kind: self
                .adversarial_kind
                .as_deref()
                .map(parse_adversarial_kind)
                .transpose()?,
            deterministic_seed: self.deterministic_seed,
        })
    }
}

fn parse_privacy_data_classes(
    labels: &[String],
) -> Result<Vec<PrivacyDataClass>, FoundryCapabilityPublishApiError> {
    if labels.is_empty() {
        return Err(FoundryCapabilityPublishApiError::MissingDataClasses);
    }
    labels
        .iter()
        .map(|label| {
            let data_class = parse_data_class_label(label).ok_or_else(|| {
                FoundryCapabilityPublishApiError::InvalidDataClass {
                    data_class: label.clone(),
                }
            })?;
            PrivacyDataClass::try_from(data_class).map_err(|_| {
                FoundryCapabilityPublishApiError::InvalidDataClass {
                    data_class: label.clone(),
                }
            })
        })
        .collect()
}

fn parse_autonomy_tier(tier: &str) -> Result<AutonomyTier, FoundryCapabilityPublishApiError> {
    match tier {
        "T1" | "T1ViewOnly" | "t1_view_only" => Ok(AutonomyTier::T1ViewOnly),
        "T2" | "T2Advisory" | "t2_advisory" => Ok(AutonomyTier::T2Advisory),
        "T3" | "T3ExecuteWithApproval" | "t3_execute_with_approval" => {
            Ok(AutonomyTier::T3ExecuteWithApproval)
        }
        "T4" | "T4AutoExecute" | "t4_auto_execute" => Ok(AutonomyTier::T4AutoExecute),
        _ => Err(FoundryCapabilityPublishApiError::InvalidAutonomyTier {
            autonomy_tier: tier.to_string(),
        }),
    }
}

fn parse_metric(metric: &str) -> Result<EvalMetric, FoundryCapabilityPublishApiError> {
    match metric {
        "ExactMatch" => Ok(EvalMetric::ExactMatch),
        "F1" => Ok(EvalMetric::F1),
        "Bleu" => Ok(EvalMetric::Bleu),
        "Rouge" => Ok(EvalMetric::Rouge),
        "HumanJudged" => Ok(EvalMetric::HumanJudged),
        "Composite" => Ok(EvalMetric::Composite),
        _ => Err(FoundryCapabilityPublishApiError::InvalidEvalMetric {
            metric: metric.to_string(),
        }),
    }
}

fn parse_adversarial_kind(kind: &str) -> Result<AdversarialKind, FoundryCapabilityPublishApiError> {
    match kind {
        "PromptInjection" => Ok(AdversarialKind::PromptInjection),
        "DataClassViolation" => Ok(AdversarialKind::DataClassViolation),
        "AutonomyBypass" => Ok(AdversarialKind::AutonomyBypass),
        "ToolExfiltration" => Ok(AdversarialKind::ToolExfiltration),
        _ => Err(FoundryCapabilityPublishApiError::InvalidAdversarialKind {
            adversarial_kind: kind.to_string(),
        }),
    }
}

impl FoundryCapabilityPublishApiError {
    fn from_capability_kernel(error: CapabilityError) -> Self {
        match error {
            CapabilityError::InvalidCapabilityId => Self::InvalidCapabilityId,
            CapabilityError::InvalidTenantId => Self::TenantMismatch {
                header_tenant_id: String::new(),
                principal_tenant_id: String::new(),
                authorization_tenant_id: String::new(),
                body_tenant_id: String::new(),
            },
            CapabilityError::EmptyNamespace => Self::EmptyNamespace,
            CapabilityError::EmptyEvidenceTopic => Self::EmptyEvidenceTopic,
            CapabilityError::MissingDataClasses => Self::MissingDataClasses,
            CapabilityError::NonPrivacyDataClass => Self::InvalidDataClass {
                data_class: "non-privacy".to_string(),
            },
            CapabilityError::InvalidCostProfile => Self::InvalidCostProfile,
            CapabilityError::MissingProviderPreference => Self::MissingProviderPreference,
            CapabilityError::InvalidProviderPreference => Self::InvalidProviderPreference,
            CapabilityError::InvalidMcpContract => Self::InvalidMcpContract,
            CapabilityError::DuplicateCapability => Self::DuplicateCapability {
                capability_id: "duplicate".to_string(),
            },
            CapabilityError::CapabilityNotFound => Self::InvalidCapabilityId,
        }
    }

    fn from_eval_kernel(error: EvalError) -> Self {
        match error {
            EvalError::InvalidCapabilityId => Self::InvalidCapabilityId,
            EvalError::EmptyVersion => Self::EmptyEvalSetVersion,
            EvalError::EmptyCaseId => Self::EmptyEvalCaseId,
            EvalError::EmptyLocale => Self::EmptyEvalLocale,
            EvalError::EmptyInputRef => Self::EmptyEvalInputRef,
            EvalError::EmptyExpectedRef => Self::EmptyEvalExpectedRef,
            EvalError::InvalidThreshold => Self::InvalidEvalThreshold,
            EvalError::EmptyEvalSet => Self::EmptyEvalSet,
            EvalError::UnsignedEvalSet => Self::UnsignedEvalSet,
            EvalError::MissingAdversarialCoverage => Self::MissingAdversarialCoverage,
            EvalError::MissingLinguisticCoverage => Self::MissingLinguisticCoverage,
            EvalError::EvalSetNotFound => Self::EvalSetNotFound,
            EvalError::UnsignedEvalRun => Self::UnsignedEvalRun,
            EvalError::EvalRunVersionMismatch => Self::EvalRunVersionMismatch,
            EvalError::EvalRunBelowThreshold => Self::EvalRunBelowThreshold,
            EvalError::MissingPassingEvalRun => Self::MissingPassingEvalRun,
        }
    }
}

pub fn publish_foundry_capability_from_api(
    directory: &mut FoundryCapabilityPublishDirectory,
    idempotency_ledger: &mut FoundryCapabilityPublishIdempotencyLedger,
    request: FoundryCapabilityPublishApiRequest,
) -> Result<FoundryCapabilityPublishSuccessResponse, FoundryCapabilityPublishApiError> {
    validate_api_binding(&request)?;

    let ledger_key = FoundryCapabilityPublishIdempotencyLedgerKey {
        tenant_id: request.boundary.tenant_id.clone(),
        principal_id: request.principal.principal_id.clone(),
        surface: FOUNDRY_CAPABILITY_PUBLISH_SURFACE.to_string(),
        idempotency_key: request.boundary.idempotency_key.clone(),
    };
    let fingerprint = FoundryCapabilityPublishRequestFingerprint::from_request(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&ledger_key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.response.clone());
        }
        return Err(FoundryCapabilityPublishApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let record = directory.publish(&request)?;
    let response = FoundryCapabilityPublishSuccessResponse {
        data: record,
        metadata: FoundryCapabilityPublishMetadata {
            request_id: request.boundary.request_id,
            idempotency_key: request.boundary.idempotency_key,
            surface: FOUNDRY_CAPABILITY_PUBLISH_SURFACE.to_string(),
            openapi_contract: FOUNDRY_REGISTRY_OPENAPI_CONTRACT.to_string(),
        },
    };
    idempotency_ledger.entries.insert(
        ledger_key,
        FoundryCapabilityPublishIdempotencyLedgerEntry {
            fingerprint,
            response: response.clone(),
        },
    );
    Ok(response)
}

fn validate_api_binding(
    request: &FoundryCapabilityPublishApiRequest,
) -> Result<(), FoundryCapabilityPublishApiError> {
    ensure_non_empty(
        &request.path_capability_id,
        FoundryCapabilityPublishApiError::EmptyPathCapabilityId,
    )?;
    ensure_non_empty(
        &request.boundary.request_id,
        FoundryCapabilityPublishApiError::EmptyRequestId,
    )?;
    ensure_non_empty(
        &request.boundary.tenant_id,
        FoundryCapabilityPublishApiError::EmptyTenantHeader,
    )?;
    ensure_non_empty(
        &request.boundary.idempotency_key,
        FoundryCapabilityPublishApiError::EmptyIdempotencyKey,
    )?;
    ensure_non_empty(
        &request.principal.tenant_id,
        FoundryCapabilityPublishApiError::EmptyPrincipalTenantId,
    )?;
    ensure_non_empty(
        &request.principal.principal_id,
        FoundryCapabilityPublishApiError::EmptyPrincipalId,
    )?;
    ensure_non_empty(
        &request.authorization.decision_id,
        FoundryCapabilityPublishApiError::EmptyAuthorizationDecisionId,
    )?;
    if request.path_capability_id != request.body.capability_id {
        return Err(FoundryCapabilityPublishApiError::CapabilityIdMismatch {
            path_capability_id: request.path_capability_id.clone(),
            body_capability_id: request.body.capability_id.clone(),
        });
    }
    if request.boundary.tenant_id != request.principal.tenant_id
        || request.boundary.tenant_id != request.authorization.tenant_id
        || request.boundary.tenant_id != request.body.tenant_id
    {
        return Err(FoundryCapabilityPublishApiError::TenantMismatch {
            header_tenant_id: request.boundary.tenant_id.clone(),
            principal_tenant_id: request.principal.tenant_id.clone(),
            authorization_tenant_id: request.authorization.tenant_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
        });
    }
    if request.authorization.tenant_id != request.principal.tenant_id
        || request.authorization.principal_id != request.principal.principal_id
    {
        return Err(
            FoundryCapabilityPublishApiError::AuthorizationPrincipalMismatch {
                principal_tenant_id: request.principal.tenant_id.clone(),
                principal_id: request.principal.principal_id.clone(),
                authorization_tenant_id: request.authorization.tenant_id.clone(),
                authorization_principal_id: request.authorization.principal_id.clone(),
            },
        );
    }
    if !request
        .authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == FOUNDRY_CAPABILITY_PUBLISH_SURFACE)
    {
        return Err(
            FoundryCapabilityPublishApiError::AuthorizationSurfaceDenied {
                decision_id: request.authorization.decision_id.clone(),
                surface: FOUNDRY_CAPABILITY_PUBLISH_SURFACE.to_string(),
            },
        );
    }
    Ok(())
}

fn ensure_non_empty(
    value: &str,
    error: FoundryCapabilityPublishApiError,
) -> Result<(), FoundryCapabilityPublishApiError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn ensure_body_non_empty(
    value: &str,
    error: FoundryCapabilityPublishApiError,
) -> Result<(), FoundryCapabilityPublishApiError> {
    ensure_non_empty(value, error)
}

impl FoundryCapabilityPublishRequestFingerprint {
    fn from_request(request: &FoundryCapabilityPublishApiRequest) -> Self {
        let data_classes = request.body.data_classes_touched.join(",");
        let providers = request.body.provider.preference().join(",");
        let eval_cases = request
            .body
            .eval_cases
            .iter()
            .map(|case| {
                format!(
                    "{}|{}|{}|{}|{}|{}",
                    case.case_id,
                    case.locale,
                    case.input_ref,
                    case.expected_ref,
                    case.adversarial_kind.as_deref().unwrap_or("None"),
                    case.deterministic_seed
                        .map_or_else(|| "None".to_string(), |seed| seed.to_string())
                )
            })
            .collect::<Vec<_>>()
            .join(";");
        Self {
            canonical: format!(
                "path={}|tenant={}|principal={}|authz={}|body_tenant={}|capability={}|namespace={}|version={}|agent_desc={}|human_desc={}|providers={}|tier={}|classes={}|topic={}|per_invocation={}|monthly={}|input_schema={}|output_schema={}|eval_version={}|metric={}|min_pass={}|min_p95={}|signed_set={}|cases=[{}]|pass={}|p95={}|adv_passed={}|ling_passed={}|signed_run={}|owner={}|catalog={}|docs={}|published={}",
                request.path_capability_id,
                request.boundary.tenant_id,
                request.principal.principal_id,
                request.authorization.decision_id,
                request.body.tenant_id,
                request.body.capability_id,
                request.body.namespace,
                request.body.version,
                request.body.description.agent_readable,
                request.body.description.human_readable,
                providers,
                request.body.autonomy_tier_required,
                data_classes,
                request.body.evidence_emission_topic,
                request.body.cost_profile.per_invocation_limit_micros,
                request.body.cost_profile.per_tenant_monthly_limit_micros,
                request.body.input_schema_json,
                request.body.output_schema_json,
                request.body.eval_set_version,
                request.body.eval_metric,
                request.body.min_pass_rate_percent,
                request.body.min_p95_score_percent,
                request.body.signed_eval_set,
                eval_cases,
                request.body.eval_pass_rate_percent,
                request.body.eval_p95_score_percent,
                request.body.eval_adversarial_passed,
                request.body.eval_linguistic_passed,
                request.body.signed_eval_run,
                request.body.owner_team,
                request.body.catalog_record_path,
                request.body.docs_path,
                request.body.published_at_epoch_seconds
            ),
        }
    }
}
