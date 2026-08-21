//! Foundry eval app/API boundary.
//!
//! This crate projects the pure `oya-governance-eval-domain` gate into the stable
//! `foundry.eval.run` REST surface: transport binding validation,
//! tenant/principal/authZ checks, idempotency, and response/error DTOs.

use std::collections::BTreeMap;

use check_eval_domain::{
    AdversarialKind, EvalCaseInput, EvalError, EvalGate, EvalMetric, EvalRunInput, EvalSetInput,
    REQUIRED_LINGUISTIC_COHORTS_DETAIL, REQUIRED_LINGUISTIC_COHORTS_MESSAGE,
};

const FOUNDRY_EVAL_RUN_SCHEMA_VERSION: u32 = 1;

pub const FOUNDRY_EVAL_RUN_SURFACE: &str = "foundry.eval.run";
pub const FOUNDRY_EVAL_RUN_OPENAPI_CONTRACT: &str = "contracts/openapi/foundry/eval-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundryEvalRunApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

impl FoundryEvalRunApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalCaseRequest {
    pub case_id: String,                  // data_class: INTERNAL_ONLY
    pub locale: String,                   // data_class: INTERNAL_ONLY
    pub input_ref: String,                // data_class: INTERNAL_ONLY
    pub expected_ref: String,             // data_class: INTERNAL_ONLY
    pub adversarial_kind: Option<String>, // data_class: INTERNAL_ONLY
    pub deterministic_seed: Option<u64>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalRunRequest {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub capability_id: String,              // data_class: INTERNAL_ONLY
    pub eval_set_version: String,           // data_class: INTERNAL_ONLY
    pub metric: String,                     // data_class: INTERNAL_ONLY
    pub min_pass_rate_percent: u8,          // data_class: INTERNAL_ONLY
    pub min_p95_score_percent: u8,          // data_class: INTERNAL_ONLY
    pub signed_eval_set: bool,              // data_class: AUDIT
    pub cases: Vec<FoundryEvalCaseRequest>, // data_class: AUDIT
    pub pass_rate_percent: u8,              // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub p95_score_percent: u8,              // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub adversarial_passed: bool,           // data_class: AUDIT
    pub linguistic_passed: bool,            // data_class: AUDIT
    pub signed_run: bool,                   // data_class: AUDIT
    pub run_started_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalRunApiRequest {
    pub path_capability_id: String, // data_class: INTERNAL_ONLY
    pub boundary: FoundryEvalApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: FoundryEvalApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: FoundryEvalApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: FoundryEvalRunRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalRunSuccessResponse {
    pub data: FoundryEvalRunRecord,       // data_class: INTERNAL_ONLY
    pub metadata: FoundryEvalRunMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalRunRecord {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub capability_id: String,             // data_class: INTERNAL_ONLY
    pub eval_set_version: String,          // data_class: INTERNAL_ONLY
    pub metric: String,                    // data_class: INTERNAL_ONLY
    pub pass_rate_percent: u8,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub p95_score_percent: u8,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub adversarial_passed: bool,          // data_class: AUDIT
    pub linguistic_passed: bool,           // data_class: AUDIT
    pub passed: bool,                      // data_class: AUDIT
    pub signed_eval_set: bool,             // data_class: AUDIT
    pub signed_run: bool,                  // data_class: AUDIT
    pub case_count: u64,                   // data_class: INTERNAL_ONLY
    pub run_started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalRunMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String,  // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalRunApiErrorResponse {
    pub error: FoundryEvalRunApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalRunApiErrorBody {
    pub code: String,                               // data_class: INTERNAL_ONLY
    pub message: String,                            // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,          // data_class: INTERNAL_ONLY
    pub request_id: String,                         // data_class: INTERNAL_ONLY
    pub details: Vec<FoundryEvalRunApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryEvalRunApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FoundryEvalRunApiError {
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
    IdempotencyKeyReused {
        idempotency_key: String, // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FoundryEvalRunApiErrorCode {
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
    EvalCapabilityIdInvalid,
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
    IdempotencyKeyReused,
}

impl FoundryEvalRunApiErrorCode {
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
            Self::EvalCapabilityIdInvalid => "EVAL_CAPABILITY_ID_INVALID",
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
            Self::IdempotencyKeyReused => "IDEMPOTENCY_KEY_REUSED",
        }
    }
}

impl FoundryEvalRunApiError {
    pub fn status(&self) -> FoundryEvalRunApiStatus {
        match self {
            Self::EmptyPrincipalTenantId | Self::EmptyPrincipalId => {
                FoundryEvalRunApiStatus::Unauthorized
            }
            Self::TenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationSurfaceDenied { .. }
            | Self::EmptyAuthorizationDecisionId => FoundryEvalRunApiStatus::Forbidden,
            Self::MissingAdversarialCoverage
            | Self::MissingLinguisticCoverage
            | Self::UnsignedEvalSet
            | Self::UnsignedEvalRun
            | Self::EvalRunBelowThreshold
            | Self::EvalRunVersionMismatch
            | Self::EvalSetNotFound
            | Self::MissingPassingEvalRun
            | Self::IdempotencyKeyReused { .. } => FoundryEvalRunApiStatus::UnprocessableEntity,
            Self::EmptyPathCapabilityId
            | Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::CapabilityIdMismatch { .. }
            | Self::InvalidCapabilityId
            | Self::EmptyEvalSetVersion
            | Self::EmptyEvalCaseId
            | Self::EmptyEvalLocale
            | Self::EmptyEvalInputRef
            | Self::EmptyEvalExpectedRef
            | Self::InvalidEvalThreshold
            | Self::InvalidEvalMetric { .. }
            | Self::InvalidAdversarialKind { .. }
            | Self::EmptyEvalSet => FoundryEvalRunApiStatus::BadRequest,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> FoundryEvalRunApiErrorCode {
        match self {
            Self::EmptyPathCapabilityId => FoundryEvalRunApiErrorCode::PathCapabilityIdEmpty,
            Self::EmptyRequestId => FoundryEvalRunApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => FoundryEvalRunApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => FoundryEvalRunApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalTenantId => FoundryEvalRunApiErrorCode::PrincipalTenantIdEmpty,
            Self::EmptyPrincipalId => FoundryEvalRunApiErrorCode::PrincipalIdEmpty,
            Self::EmptyAuthorizationDecisionId => {
                FoundryEvalRunApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::CapabilityIdMismatch { .. } => FoundryEvalRunApiErrorCode::CapabilityIdMismatch,
            Self::TenantMismatch { .. } => FoundryEvalRunApiErrorCode::TenantMismatch,
            Self::AuthorizationPrincipalMismatch { .. } => {
                FoundryEvalRunApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationSurfaceDenied { .. } => {
                FoundryEvalRunApiErrorCode::AuthorizationSurfaceDenied
            }
            Self::InvalidCapabilityId => FoundryEvalRunApiErrorCode::EvalCapabilityIdInvalid,
            Self::EmptyEvalSetVersion => FoundryEvalRunApiErrorCode::EvalSetVersionEmpty,
            Self::EmptyEvalCaseId => FoundryEvalRunApiErrorCode::EvalCaseIdEmpty,
            Self::EmptyEvalLocale => FoundryEvalRunApiErrorCode::EvalLocaleEmpty,
            Self::EmptyEvalInputRef => FoundryEvalRunApiErrorCode::EvalInputRefEmpty,
            Self::EmptyEvalExpectedRef => FoundryEvalRunApiErrorCode::EvalExpectedRefEmpty,
            Self::InvalidEvalThreshold => FoundryEvalRunApiErrorCode::EvalThresholdInvalid,
            Self::InvalidEvalMetric { .. } => FoundryEvalRunApiErrorCode::EvalMetricInvalid,
            Self::InvalidAdversarialKind { .. } => {
                FoundryEvalRunApiErrorCode::EvalAdversarialKindInvalid
            }
            Self::EmptyEvalSet => FoundryEvalRunApiErrorCode::EvalSetEmpty,
            Self::UnsignedEvalSet => FoundryEvalRunApiErrorCode::EvalSetUnsigned,
            Self::MissingAdversarialCoverage => {
                FoundryEvalRunApiErrorCode::EvalAdversarialCoverageMissing
            }
            Self::MissingLinguisticCoverage => {
                FoundryEvalRunApiErrorCode::EvalLinguisticCoverageMissing
            }
            Self::EvalSetNotFound => FoundryEvalRunApiErrorCode::EvalSetNotFound,
            Self::UnsignedEvalRun => FoundryEvalRunApiErrorCode::EvalRunUnsigned,
            Self::EvalRunVersionMismatch => FoundryEvalRunApiErrorCode::EvalRunVersionMismatch,
            Self::EvalRunBelowThreshold => FoundryEvalRunApiErrorCode::EvalRunBelowThreshold,
            Self::MissingPassingEvalRun => FoundryEvalRunApiErrorCode::EvalRunMissingPassing,
            Self::IdempotencyKeyReused { .. } => FoundryEvalRunApiErrorCode::IdempotencyKeyReused,
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> FoundryEvalRunApiErrorResponse {
        FoundryEvalRunApiErrorResponse {
            error: FoundryEvalRunApiErrorBody {
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
                "Authorization decision does not grant foundry.eval.run"
            }
            Self::InvalidCapabilityId => "Capability id must use the cap. namespace",
            Self::EmptyEvalSetVersion => "Eval set version is required",
            Self::EmptyEvalCaseId => "Eval case id is required",
            Self::EmptyEvalLocale => "Eval case locale is required",
            Self::EmptyEvalInputRef => "Eval case input reference is required",
            Self::EmptyEvalExpectedRef => "Eval case expected reference is required",
            Self::InvalidEvalThreshold => "Eval thresholds must be between 1 and 100",
            Self::InvalidEvalMetric { .. } => "Eval metric is not supported",
            Self::InvalidAdversarialKind { .. } => "Adversarial kind is not supported",
            Self::EmptyEvalSet => "Eval set must include at least one case",
            Self::UnsignedEvalSet => "Eval set must be signed before running",
            Self::MissingAdversarialCoverage => {
                "Eval set must include all mandatory adversarial cohorts"
            }
            Self::MissingLinguisticCoverage => REQUIRED_LINGUISTIC_COHORTS_MESSAGE,
            Self::EvalSetNotFound => "Eval set was not registered for this capability",
            Self::UnsignedEvalRun => "Eval run must be signed before recording",
            Self::EvalRunVersionMismatch => "Eval run version must match the registered eval set",
            Self::EvalRunBelowThreshold => "Eval run did not meet the registered pass threshold",
            Self::MissingPassingEvalRun => "No passing eval run is available for this capability",
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different eval run request"
            }
        }
    }

    fn details(&self) -> Vec<FoundryEvalRunApiErrorDetail> {
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
            Self::MissingAdversarialCoverage => vec![detail(
                "body.cases",
                "missing prompt-injection, data-class-violation, autonomy-bypass, or tool-exfiltration cohort",
            )],
            Self::MissingLinguisticCoverage => {
                vec![detail("body.cases", REQUIRED_LINGUISTIC_COHORTS_DETAIL)]
            }
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
            Self::EmptyEvalSetVersion => vec![detail("body.eval_set_version", "must not be empty")],
            Self::EmptyEvalCaseId => vec![detail("body.cases[].case_id", "must not be empty")],
            Self::EmptyEvalLocale => vec![detail("body.cases[].locale", "must not be empty")],
            Self::EmptyEvalInputRef => vec![detail("body.cases[].input_ref", "must not be empty")],
            Self::EmptyEvalExpectedRef => {
                vec![detail("body.cases[].expected_ref", "must not be empty")]
            }
            Self::InvalidEvalThreshold => vec![detail(
                "body.min_pass_rate_percent/body.min_p95_score_percent",
                "must be 1..=100",
            )],
            Self::InvalidEvalMetric { metric } => {
                vec![detail(
                    "body.metric",
                    format!("unsupported metric {metric}"),
                )]
            }
            Self::InvalidAdversarialKind { adversarial_kind } => vec![detail(
                "body.cases[].adversarial_kind",
                format!("unsupported adversarial kind {adversarial_kind}"),
            )],
            Self::EmptyEvalSet => vec![detail("body.cases", "must not be empty")],
            Self::UnsignedEvalSet => vec![detail("body.signed_eval_set", "must be true")],
            Self::EvalSetNotFound => vec![detail("body.capability_id", "eval set is missing")],
            Self::UnsignedEvalRun => vec![detail("body.signed_run", "must be true")],
            Self::EvalRunVersionMismatch => vec![detail(
                "body.eval_set_version",
                "run version must equal eval set version",
            )],
            Self::EvalRunBelowThreshold => vec![detail(
                "body.pass_rate_percent/body.p95_score_percent",
                "run scores or cohorts did not meet thresholds",
            )],
            Self::MissingPassingEvalRun => {
                vec![detail("body.capability_id", "no passing run recorded")]
            }
        }
    }
}

fn detail(field: impl Into<String>, issue: impl Into<String>) -> FoundryEvalRunApiErrorDetail {
    FoundryEvalRunApiErrorDetail {
        field: field.into(),
        issue: issue.into(),
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryEvalRunDirectory {
    eval_gate: EvalGate, // data_class: INTERNAL_ONLY
    records: BTreeMap<FoundryEvalRunRecordKey, FoundryEvalRunRecord>, // data_class: INTERNAL_ONLY
}

impl FoundryEvalRunDirectory {
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
        eval_set_version: &str,
    ) -> Option<&FoundryEvalRunRecord> {
        self.records.get(&FoundryEvalRunRecordKey {
            tenant_id: tenant_id.to_string(),
            capability_id: capability_id.to_string(),
            eval_set_version: eval_set_version.to_string(),
        })
    }

    fn run(
        &mut self,
        request: &FoundryEvalRunApiRequest,
    ) -> Result<FoundryEvalRunRecord, FoundryEvalRunApiError> {
        let mut working_gate = self.eval_gate.clone();
        let eval_set_input = EvalSetInput {
            capability_id: request.body.capability_id.clone(),
            version: request.body.eval_set_version.clone(),
            metric: parse_metric(&request.body.metric)?,
            min_pass_rate_percent: request.body.min_pass_rate_percent,
            min_p95_score_percent: request.body.min_p95_score_percent,
            signed: request.body.signed_eval_set,
            cases: request
                .body
                .cases
                .iter()
                .map(FoundryEvalCaseRequest::to_kernel)
                .collect::<Result<Vec<_>, _>>()?,
        };
        working_gate
            .register_eval_set(eval_set_input)
            .map_err(FoundryEvalRunApiError::from_kernel)?;
        let run = working_gate
            .record_run(EvalRunInput {
                capability_id: request.body.capability_id.clone(),
                eval_set_version: request.body.eval_set_version.clone(),
                pass_rate_percent: request.body.pass_rate_percent,
                p95_score_percent: request.body.p95_score_percent,
                adversarial_passed: request.body.adversarial_passed,
                linguistic_passed: request.body.linguistic_passed,
                signed: request.body.signed_run,
            })
            .map_err(FoundryEvalRunApiError::from_kernel)?;
        let record = FoundryEvalRunRecord {
            tenant_id: request.body.tenant_id.clone(),
            capability_id: request.body.capability_id.clone(),
            eval_set_version: request.body.eval_set_version.clone(),
            metric: request.body.metric.clone(),
            pass_rate_percent: request.body.pass_rate_percent,
            p95_score_percent: request.body.p95_score_percent,
            adversarial_passed: request.body.adversarial_passed,
            linguistic_passed: request.body.linguistic_passed,
            passed: run.passed.value,
            signed_eval_set: request.body.signed_eval_set,
            signed_run: request.body.signed_run,
            case_count: request.body.cases.len() as u64,
            run_started_at_epoch_seconds: request.body.run_started_at_epoch_seconds,
            schema_version: FOUNDRY_EVAL_RUN_SCHEMA_VERSION,
        };
        self.eval_gate = working_gate;
        self.records.insert(
            FoundryEvalRunRecordKey {
                tenant_id: record.tenant_id.clone(),
                capability_id: record.capability_id.clone(),
                eval_set_version: record.eval_set_version.clone(),
            },
            record.clone(),
        );
        Ok(record)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FoundryEvalRunRecordKey {
    tenant_id: String,        // data_class: INTERNAL_ONLY
    capability_id: String,    // data_class: INTERNAL_ONLY
    eval_set_version: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryEvalRunIdempotencyLedger {
    entries: BTreeMap<FoundryEvalRunIdempotencyLedgerKey, FoundryEvalRunIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl FoundryEvalRunIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FoundryEvalRunIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryEvalRunIdempotencyLedgerEntry {
    fingerprint: FoundryEvalRunRequestFingerprint, // data_class: INTERNAL_ONLY
    response: FoundryEvalRunSuccessResponse,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FoundryEvalRunRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

impl FoundryEvalCaseRequest {
    fn to_kernel(&self) -> Result<EvalCaseInput, FoundryEvalRunApiError> {
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

fn parse_metric(metric: &str) -> Result<EvalMetric, FoundryEvalRunApiError> {
    match metric {
        "ExactMatch" => Ok(EvalMetric::ExactMatch),
        "F1" => Ok(EvalMetric::F1),
        "Bleu" => Ok(EvalMetric::Bleu),
        "Rouge" => Ok(EvalMetric::Rouge),
        "HumanJudged" => Ok(EvalMetric::HumanJudged),
        "Composite" => Ok(EvalMetric::Composite),
        _ => Err(FoundryEvalRunApiError::InvalidEvalMetric {
            metric: metric.to_string(),
        }),
    }
}

fn parse_adversarial_kind(kind: &str) -> Result<AdversarialKind, FoundryEvalRunApiError> {
    match kind {
        "PromptInjection" => Ok(AdversarialKind::PromptInjection),
        "DataClassViolation" => Ok(AdversarialKind::DataClassViolation),
        "AutonomyBypass" => Ok(AdversarialKind::AutonomyBypass),
        "ToolExfiltration" => Ok(AdversarialKind::ToolExfiltration),
        _ => Err(FoundryEvalRunApiError::InvalidAdversarialKind {
            adversarial_kind: kind.to_string(),
        }),
    }
}

impl FoundryEvalRunApiError {
    fn from_kernel(error: EvalError) -> Self {
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

pub fn run_foundry_eval_from_api(
    directory: &mut FoundryEvalRunDirectory,
    idempotency_ledger: &mut FoundryEvalRunIdempotencyLedger,
    request: FoundryEvalRunApiRequest,
) -> Result<FoundryEvalRunSuccessResponse, FoundryEvalRunApiError> {
    validate_api_binding(&request)?;

    let ledger_key = FoundryEvalRunIdempotencyLedgerKey {
        tenant_id: request.boundary.tenant_id.clone(),
        principal_id: request.principal.principal_id.clone(),
        surface: FOUNDRY_EVAL_RUN_SURFACE.to_string(),
        idempotency_key: request.boundary.idempotency_key.clone(),
    };
    let fingerprint = FoundryEvalRunRequestFingerprint::from_request(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&ledger_key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.response.clone());
        }
        return Err(FoundryEvalRunApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let record = directory.run(&request)?;
    let response = FoundryEvalRunSuccessResponse {
        data: record,
        metadata: FoundryEvalRunMetadata {
            request_id: request.boundary.request_id,
            idempotency_key: request.boundary.idempotency_key,
            surface: FOUNDRY_EVAL_RUN_SURFACE.to_string(),
            openapi_contract: FOUNDRY_EVAL_RUN_OPENAPI_CONTRACT.to_string(),
        },
    };
    idempotency_ledger.entries.insert(
        ledger_key,
        FoundryEvalRunIdempotencyLedgerEntry {
            fingerprint,
            response: response.clone(),
        },
    );
    Ok(response)
}

fn validate_api_binding(request: &FoundryEvalRunApiRequest) -> Result<(), FoundryEvalRunApiError> {
    ensure_non_empty(
        &request.path_capability_id,
        FoundryEvalRunApiError::EmptyPathCapabilityId,
    )?;
    ensure_non_empty(
        &request.boundary.request_id,
        FoundryEvalRunApiError::EmptyRequestId,
    )?;
    ensure_non_empty(
        &request.boundary.tenant_id,
        FoundryEvalRunApiError::EmptyTenantHeader,
    )?;
    ensure_non_empty(
        &request.boundary.idempotency_key,
        FoundryEvalRunApiError::EmptyIdempotencyKey,
    )?;
    ensure_non_empty(
        &request.principal.tenant_id,
        FoundryEvalRunApiError::EmptyPrincipalTenantId,
    )?;
    ensure_non_empty(
        &request.principal.principal_id,
        FoundryEvalRunApiError::EmptyPrincipalId,
    )?;
    ensure_non_empty(
        &request.authorization.decision_id,
        FoundryEvalRunApiError::EmptyAuthorizationDecisionId,
    )?;
    if request.path_capability_id != request.body.capability_id {
        return Err(FoundryEvalRunApiError::CapabilityIdMismatch {
            path_capability_id: request.path_capability_id.clone(),
            body_capability_id: request.body.capability_id.clone(),
        });
    }
    if request.boundary.tenant_id != request.principal.tenant_id
        || request.boundary.tenant_id != request.authorization.tenant_id
        || request.boundary.tenant_id != request.body.tenant_id
    {
        return Err(FoundryEvalRunApiError::TenantMismatch {
            header_tenant_id: request.boundary.tenant_id.clone(),
            principal_tenant_id: request.principal.tenant_id.clone(),
            authorization_tenant_id: request.authorization.tenant_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
        });
    }
    if request.authorization.tenant_id != request.principal.tenant_id
        || request.authorization.principal_id != request.principal.principal_id
    {
        return Err(FoundryEvalRunApiError::AuthorizationPrincipalMismatch {
            principal_tenant_id: request.principal.tenant_id.clone(),
            principal_id: request.principal.principal_id.clone(),
            authorization_tenant_id: request.authorization.tenant_id.clone(),
            authorization_principal_id: request.authorization.principal_id.clone(),
        });
    }
    if !request
        .authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == FOUNDRY_EVAL_RUN_SURFACE)
    {
        return Err(FoundryEvalRunApiError::AuthorizationSurfaceDenied {
            decision_id: request.authorization.decision_id.clone(),
            surface: FOUNDRY_EVAL_RUN_SURFACE.to_string(),
        });
    }
    Ok(())
}

fn ensure_non_empty(
    value: &str,
    error: FoundryEvalRunApiError,
) -> Result<(), FoundryEvalRunApiError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

impl FoundryEvalRunRequestFingerprint {
    fn from_request(request: &FoundryEvalRunApiRequest) -> Self {
        let case_fingerprint = request
            .body
            .cases
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
                "path={}|tenant={}|principal={}|authz={}|body_tenant={}|capability={}|version={}|metric={}|min_pass={}|min_p95={}|signed_set={}|cases=[{}]|pass={}|p95={}|adv_passed={}|ling_passed={}|signed_run={}|started={}",
                request.path_capability_id,
                request.boundary.tenant_id,
                request.principal.principal_id,
                request.authorization.decision_id,
                request.body.tenant_id,
                request.body.capability_id,
                request.body.eval_set_version,
                request.body.metric,
                request.body.min_pass_rate_percent,
                request.body.min_p95_score_percent,
                request.body.signed_eval_set,
                case_fingerprint,
                request.body.pass_rate_percent,
                request.body.p95_score_percent,
                request.body.adversarial_passed,
                request.body.linguistic_passed,
                request.body.signed_run,
                request.body.run_started_at_epoch_seconds
            ),
        }
    }
}
