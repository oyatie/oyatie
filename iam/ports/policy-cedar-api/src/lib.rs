//! Platform Cedar policy publish API boundary.
//!
//! This crate owns authenticated REST-boundary normalization, path/body
//! policy-version binding, request fingerprint idempotency, scope/effect parsing,
//! and stable public error projection for `cedar.policy.publish` before handing
//! typed policy publication to the platform Cedar policy kernel.
//!
//! ## Modules
//!
//! - [`authz`] — fail-closed principal-verification + PDP authorization PORTS
//!   for the publish control plane (AUTH-005 / task #124 / ADR-0572).
//! - [`rest`] — axum control-plane REST edge (ADR-0090 amendment).

pub mod authz;
pub mod rest;

use std::collections::BTreeMap;

use iam_policy_cedar_domain::{
    PolicyEffect, PolicyError, PolicyRuleInput, PolicyScope, PolicySet, PolicyVersion,
    PublishedPolicy,
};

pub const CEDAR_POLICY_PUBLISH_SURFACE: &str = "cedar.policy.publish";
pub const CEDAR_POLICY_PUBLISH_OPENAPI_CONTRACT: &str =
    "contracts/openapi/platform/platform-policy-cedar-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CedarPolicyPublishApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CedarPolicyPublishApiStatus {
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
pub enum CedarPolicyPublishApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathPolicyIdEmpty,
    PathVersionEmpty,
    PolicyPathBodyMismatch,
    VersionPathBodyMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    ScopeKindInvalid,
    ScopeTenantMissing,
    ScopeTenantForbidden,
    ScopeTenantInvalid,
    RuleEffectInvalid,
    RequiredAttributeKeyEmpty,
    RequiredAttributeValueEmpty,
    SupersedesInvalid,
    IdempotencyKeyReused,
    PolicyInvalidId,
    PolicyInvalidSemver,
    PolicyEmptyRules,
    PolicyEmptyRuleField,
    PolicyVersionAlreadyExists,
    PolicySupersedesSelf,
    PolicySupersedesMissing,
    PolicySupersedesScopeMismatch,
    PolicySupersedesNotOlder,
}

impl CedarPolicyPublishApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CEDAR_POLICY_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CEDAR_POLICY_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CEDAR_POLICY_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CEDAR_POLICY_PRINCIPAL_ID_EMPTY",
            Self::PathPolicyIdEmpty => "CEDAR_POLICY_PATH_POLICY_ID_EMPTY",
            Self::PathVersionEmpty => "CEDAR_POLICY_PATH_VERSION_EMPTY",
            Self::PolicyPathBodyMismatch => "CEDAR_POLICY_POLICY_PATH_BODY_MISMATCH",
            Self::VersionPathBodyMismatch => "CEDAR_POLICY_VERSION_PATH_BODY_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "CEDAR_POLICY_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "CEDAR_POLICY_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "CEDAR_POLICY_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "CEDAR_POLICY_AUTHORIZATION_DENIED",
            Self::ScopeKindInvalid => "CEDAR_POLICY_SCOPE_KIND_INVALID",
            Self::ScopeTenantMissing => "CEDAR_POLICY_SCOPE_TENANT_MISSING",
            Self::ScopeTenantForbidden => "CEDAR_POLICY_SCOPE_TENANT_FORBIDDEN",
            Self::ScopeTenantInvalid => "CEDAR_POLICY_SCOPE_TENANT_INVALID",
            Self::RuleEffectInvalid => "CEDAR_POLICY_RULE_EFFECT_INVALID",
            Self::RequiredAttributeKeyEmpty => "CEDAR_POLICY_REQUIRED_ATTRIBUTE_KEY_EMPTY",
            Self::RequiredAttributeValueEmpty => "CEDAR_POLICY_REQUIRED_ATTRIBUTE_VALUE_EMPTY",
            Self::SupersedesInvalid => "CEDAR_POLICY_SUPERSEDES_INVALID",
            Self::IdempotencyKeyReused => "CEDAR_POLICY_IDEMPOTENCY_KEY_REUSED",
            Self::PolicyInvalidId => "CEDAR_POLICY_KERNEL_INVALID_ID",
            Self::PolicyInvalidSemver => "CEDAR_POLICY_KERNEL_INVALID_SEMVER",
            Self::PolicyEmptyRules => "CEDAR_POLICY_KERNEL_EMPTY_RULES",
            Self::PolicyEmptyRuleField => "CEDAR_POLICY_KERNEL_EMPTY_RULE_FIELD",
            Self::PolicyVersionAlreadyExists => "CEDAR_POLICY_KERNEL_VERSION_ALREADY_EXISTS",
            Self::PolicySupersedesSelf => "CEDAR_POLICY_KERNEL_SUPERSEDES_SELF",
            Self::PolicySupersedesMissing => "CEDAR_POLICY_KERNEL_SUPERSEDES_MISSING",
            Self::PolicySupersedesScopeMismatch => "CEDAR_POLICY_KERNEL_SUPERSEDES_SCOPE_MISMATCH",
            Self::PolicySupersedesNotOlder => "CEDAR_POLICY_KERNEL_SUPERSEDES_NOT_OLDER",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyScopeRef {
    pub kind: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyRequiredAttribute {
    pub key: String,   // data_class: INTERNAL_ONLY
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyRuleRef {
    pub effect: String,          // data_class: INTERNAL_ONLY
    pub principal_role: String,  // data_class: INTERNAL_ONLY
    pub action: String,          // data_class: INTERNAL_ONLY
    pub resource_prefix: String, // data_class: INTERNAL_ONLY
    pub required_attribute: Option<CedarPolicyRequiredAttribute>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyPublishRequest {
    pub policy_id: String,              // data_class: INTERNAL_ONLY
    pub version: String,                // data_class: INTERNAL_ONLY
    pub scope: CedarPolicyScopeRef,     // data_class: INTERNAL_ONLY
    pub supersedes: Option<String>,     // data_class: INTERNAL_ONLY
    pub rules: Vec<CedarPolicyRuleRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyPublishApiRequest {
    pub path_policy_id: String,                  // data_class: INTERNAL_ONLY
    pub path_version: String,                    // data_class: INTERNAL_ONLY
    pub boundary: CedarPolicyApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CedarPolicyApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: CedarPolicyApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CedarPolicyPublishRequest,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CedarPolicyPublishIdempotencyLedger {
    entries:
        BTreeMap<CedarPolicyPublishIdempotencyLedgerKey, CedarPolicyPublishIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CedarPolicyPublishIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CedarPolicyPublishIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CedarPolicyPublishIdempotencyLedgerEntry {
    fingerprint: CedarPolicyPublishRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CedarPolicyPublishSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CedarPolicyPublishRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyPublishSuccessResponse {
    pub data: CedarPolicyRecord,              // data_class: INTERNAL_ONLY
    pub metadata: CedarPolicyPublishMetadata, // data_class: INTERNAL_ONLY
}

impl CedarPolicyPublishSuccessResponse {
    pub fn created(data: CedarPolicyRecord, request: &CedarPolicyPublishApiRequest) -> Self {
        Self {
            data,
            metadata: CedarPolicyPublishMetadata {
                request_id: request.boundary.request_id.clone(),
                operator_tenant_id: request.boundary.tenant_id.clone(),
                principal_id: request.principal.principal_id.clone(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyPublishMetadata {
    pub request_id: String,         // data_class: INTERNAL_ONLY
    pub operator_tenant_id: String, // data_class: INTERNAL_ONLY
    pub principal_id: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyRecord {
    pub policy_id: String,              // data_class: INTERNAL_ONLY
    pub version: String,                // data_class: INTERNAL_ONLY
    pub scope: CedarPolicyScopeRef,     // data_class: INTERNAL_ONLY
    pub supersedes: Option<String>,     // data_class: INTERNAL_ONLY
    pub rules: Vec<CedarPolicyRuleRef>, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyPublishApiErrorResponse {
    pub error: CedarPolicyPublishApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyPublishApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<CedarPolicyPublishApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarPolicyPublishApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CedarPolicyPublishApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathPolicyId,
    EmptyPathVersion,
    PolicyPathBodyMismatch {
        path_policy_id: String,
        body_policy_id: String,
    },
    VersionPathBodyMismatch {
        path_version: String,
        body_version: String,
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
    InvalidScopeKind {
        scope_kind: String,
    },
    MissingScopeTenant,
    ForbiddenScopeTenant {
        tenant_id: String,
    },
    InvalidScopeTenant {
        tenant_id: String,
    },
    InvalidRuleEffect {
        effect: String,
    },
    EmptyRequiredAttributeKey,
    EmptyRequiredAttributeValue,
    InvalidSupersedes {
        supersedes: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Policy(PolicyError),
}

impl CedarPolicyPublishApiError {
    pub fn cedar_policy_publish_status(&self) -> CedarPolicyPublishApiStatus {
        match self.status_kind() {
            CedarPolicyPublishApiStatusKind::BadRequest => CedarPolicyPublishApiStatus::BadRequest,
            CedarPolicyPublishApiStatusKind::Unauthorized => {
                CedarPolicyPublishApiStatus::Unauthorized
            }
            CedarPolicyPublishApiStatusKind::Forbidden => CedarPolicyPublishApiStatus::Forbidden,
            CedarPolicyPublishApiStatusKind::Conflict => CedarPolicyPublishApiStatus::Conflict,
            CedarPolicyPublishApiStatusKind::UnprocessableEntity => {
                CedarPolicyPublishApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn cedar_policy_publish_status_code(&self) -> u16 {
        self.cedar_policy_publish_status().code()
    }

    pub fn code(&self) -> CedarPolicyPublishApiErrorCode {
        match self {
            Self::EmptyRequestId => CedarPolicyPublishApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CedarPolicyPublishApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CedarPolicyPublishApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CedarPolicyPublishApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathPolicyId => CedarPolicyPublishApiErrorCode::PathPolicyIdEmpty,
            Self::EmptyPathVersion => CedarPolicyPublishApiErrorCode::PathVersionEmpty,
            Self::PolicyPathBodyMismatch { .. } => {
                CedarPolicyPublishApiErrorCode::PolicyPathBodyMismatch
            }
            Self::VersionPathBodyMismatch { .. } => {
                CedarPolicyPublishApiErrorCode::VersionPathBodyMismatch
            }
            Self::EmptyAuthorizationDecisionId => {
                CedarPolicyPublishApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CedarPolicyPublishApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CedarPolicyPublishApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CedarPolicyPublishApiErrorCode::AuthorizationDenied,
            Self::InvalidScopeKind { .. } => CedarPolicyPublishApiErrorCode::ScopeKindInvalid,
            Self::MissingScopeTenant => CedarPolicyPublishApiErrorCode::ScopeTenantMissing,
            Self::ForbiddenScopeTenant { .. } => {
                CedarPolicyPublishApiErrorCode::ScopeTenantForbidden
            }
            Self::InvalidScopeTenant { .. } => CedarPolicyPublishApiErrorCode::ScopeTenantInvalid,
            Self::InvalidRuleEffect { .. } => CedarPolicyPublishApiErrorCode::RuleEffectInvalid,
            Self::EmptyRequiredAttributeKey => {
                CedarPolicyPublishApiErrorCode::RequiredAttributeKeyEmpty
            }
            Self::EmptyRequiredAttributeValue => {
                CedarPolicyPublishApiErrorCode::RequiredAttributeValueEmpty
            }
            Self::InvalidSupersedes { .. } => CedarPolicyPublishApiErrorCode::SupersedesInvalid,
            Self::IdempotencyKeyReused { .. } => {
                CedarPolicyPublishApiErrorCode::IdempotencyKeyReused
            }
            Self::Policy(error) => policy_error_code(error),
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> CedarPolicyPublishApiErrorResponse {
        CedarPolicyPublishApiErrorResponse {
            error: CedarPolicyPublishApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CedarPolicyPublishApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CedarPolicyPublishApiStatusKind::Unauthorized,
            Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CedarPolicyPublishApiStatusKind::Forbidden,
            Self::Policy(PolicyError::VersionAlreadyExists) => {
                CedarPolicyPublishApiStatusKind::Conflict
            }
            Self::IdempotencyKeyReused { .. } => {
                CedarPolicyPublishApiStatusKind::UnprocessableEntity
            }
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathPolicyId
            | Self::EmptyPathVersion
            | Self::PolicyPathBodyMismatch { .. }
            | Self::VersionPathBodyMismatch { .. }
            | Self::InvalidScopeKind { .. }
            | Self::MissingScopeTenant
            | Self::ForbiddenScopeTenant { .. }
            | Self::InvalidScopeTenant { .. }
            | Self::InvalidRuleEffect { .. }
            | Self::EmptyRequiredAttributeKey
            | Self::EmptyRequiredAttributeValue
            | Self::InvalidSupersedes { .. }
            | Self::Policy(_) => CedarPolicyPublishApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathPolicyId => "Path policy id is required",
            Self::EmptyPathVersion => "Path policy version is required",
            Self::PolicyPathBodyMismatch { .. } => {
                "Path policy id must match request body policy_id"
            }
            Self::VersionPathBodyMismatch { .. } => "Path version must match request body version",
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cedar policy publish surface"
            }
            Self::InvalidScopeKind { .. } => "Policy scope kind must be global or tenant",
            Self::MissingScopeTenant => "Tenant-scoped policies require scope.tenant_id",
            Self::ForbiddenScopeTenant { .. } => "Global policies must not include scope.tenant_id",
            Self::InvalidScopeTenant { .. } => "Scope tenant id must be a ten_ identifier",
            Self::InvalidRuleEffect { .. } => "Rule effect must be allow or deny",
            Self::EmptyRequiredAttributeKey => "Required attribute key is required",
            Self::EmptyRequiredAttributeValue => "Required attribute value is required",
            Self::InvalidSupersedes { .. } => "Supersedes must be a semver version when present",
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::Policy(error) => policy_error_message(error),
        }
    }

    fn details(&self) -> Vec<CedarPolicyPublishApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathPolicyId => vec![detail("path.policy_id", "must be non-empty")],
            Self::EmptyPathVersion => vec![detail("path.version", "must be non-empty")],
            Self::PolicyPathBodyMismatch { .. } => vec![detail(
                "body.policy_id",
                "must match the policy_id path parameter",
            )],
            Self::VersionPathBodyMismatch { .. } => vec![detail(
                "body.version",
                "must match the version path parameter",
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
                "must include the requested cedar.policy.publish surface",
            )],
            Self::InvalidScopeKind { .. } => {
                vec![detail("body.scope.kind", "must be global or tenant")]
            }
            Self::MissingScopeTenant => vec![detail(
                "body.scope.tenant_id",
                "must be present for tenant scope",
            )],
            Self::ForbiddenScopeTenant { .. } => vec![detail(
                "body.scope.tenant_id",
                "must be absent for global scope",
            )],
            Self::InvalidScopeTenant { .. } => {
                vec![detail("body.scope.tenant_id", "must start with ten_")]
            }
            Self::InvalidRuleEffect { .. } => {
                vec![detail("body.rules[].effect", "must be allow or deny")]
            }
            Self::EmptyRequiredAttributeKey => vec![detail(
                "body.rules[].required_attribute.key",
                "must be non-empty",
            )],
            Self::EmptyRequiredAttributeValue => vec![detail(
                "body.rules[].required_attribute.value",
                "must be non-empty",
            )],
            Self::InvalidSupersedes { .. } => vec![detail(
                "body.supersedes",
                "must be a semantic version when present",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::Policy(error) => vec![detail("policy_kernel", policy_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CedarPolicyPublishApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cedar_policy_publish_request(
    request: &CedarPolicyPublishApiRequest,
) -> Result<(), CedarPolicyPublishApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_body_binding(request)?;
    validate_principal_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CEDAR_POLICY_PUBLISH_SURFACE,
    )?;
    parse_policy_scope(&request.body.scope)?;
    validate_supersedes(&request.body.supersedes)?;
    for rule in &request.body.rules {
        parse_policy_effect(&rule.effect)?;
        validate_required_attribute(rule.required_attribute.as_ref())?;
    }
    Ok(())
}

/// Publish a Cedar policy version, requiring a verified caller principal as a
/// **type-level precondition**. The `verified` token is only mintable by the
/// [`crate::authz::PrincipalVerifier`] port, so no in-process caller can reach
/// the mutation without first completing principal verification + PDP
/// authorization. The REST handler is NOT the only guard; this boundary API
/// enforces the same invariant for any future route or adapter that imports it.
pub fn publish_cedar_policy_from_api(
    _verified: &crate::authz::VerifiedPrincipal,
    policies: &mut PolicySet,
    idempotency_ledger: &mut CedarPolicyPublishIdempotencyLedger,
    request: CedarPolicyPublishApiRequest,
) -> Result<CedarPolicyPublishSuccessResponse, CedarPolicyPublishApiError> {
    validate_cedar_policy_publish_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CEDAR_POLICY_PUBLISH_SURFACE,
    );
    let fingerprint = cedar_policy_publish_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(CedarPolicyPublishApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let version = policy_version_from_request(&request.body)?;
    let published = policies
        .publish(version)
        .map_err(CedarPolicyPublishApiError::Policy)?;
    let response = CedarPolicyPublishSuccessResponse::created(policy_record(&published), &request);
    idempotency_ledger.entries.insert(
        key,
        CedarPolicyPublishIdempotencyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(
    boundary: &CedarPolicyApiBoundaryContext,
) -> Result<(), CedarPolicyPublishApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CedarPolicyPublishApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CedarPolicyPublishApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CedarPolicyPublishApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_body_binding(
    request: &CedarPolicyPublishApiRequest,
) -> Result<(), CedarPolicyPublishApiError> {
    if request.path_policy_id.trim().is_empty() {
        return Err(CedarPolicyPublishApiError::EmptyPathPolicyId);
    }
    if request.path_version.trim().is_empty() {
        return Err(CedarPolicyPublishApiError::EmptyPathVersion);
    }
    if request.path_policy_id != request.body.policy_id {
        return Err(CedarPolicyPublishApiError::PolicyPathBodyMismatch {
            path_policy_id: request.path_policy_id.clone(),
            body_policy_id: request.body.policy_id.clone(),
        });
    }
    if request.path_version != request.body.version {
        return Err(CedarPolicyPublishApiError::VersionPathBodyMismatch {
            path_version: request.path_version.clone(),
            body_version: request.body.version.clone(),
        });
    }
    Ok(())
}

fn validate_principal_binding(
    boundary: &CedarPolicyApiBoundaryContext,
    principal: &CedarPolicyApiPrincipal,
) -> Result<(), CedarPolicyPublishApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CedarPolicyPublishApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id {
        return Err(CedarPolicyPublishApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CedarPolicyApiPrincipal,
    authorization: &CedarPolicyApiAuthorization,
    surface: &str,
) -> Result<(), CedarPolicyPublishApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CedarPolicyPublishApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CedarPolicyPublishApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CedarPolicyPublishApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CedarPolicyPublishApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn parse_policy_scope(
    scope: &CedarPolicyScopeRef,
) -> Result<PolicyScope, CedarPolicyPublishApiError> {
    match scope.kind.as_str() {
        "global" => {
            if let Some(tenant_id) = scope
                .tenant_id
                .as_ref()
                .filter(|tenant_id| !tenant_id.is_empty())
            {
                return Err(CedarPolicyPublishApiError::ForbiddenScopeTenant {
                    tenant_id: tenant_id.clone(),
                });
            }
            Ok(PolicyScope::Global)
        }
        "tenant" => {
            let tenant_id = scope
                .tenant_id
                .as_ref()
                .filter(|tenant_id| !tenant_id.trim().is_empty())
                .ok_or(CedarPolicyPublishApiError::MissingScopeTenant)?;
            if !tenant_id.starts_with("ten_") || tenant_id.len() <= 4 {
                return Err(CedarPolicyPublishApiError::InvalidScopeTenant {
                    tenant_id: tenant_id.clone(),
                });
            }
            Ok(PolicyScope::Tenant(tenant_id.clone()))
        }
        _ => Err(CedarPolicyPublishApiError::InvalidScopeKind {
            scope_kind: scope.kind.clone(),
        }),
    }
}

fn validate_supersedes(supersedes: &Option<String>) -> Result<(), CedarPolicyPublishApiError> {
    if let Some(version) = supersedes.as_ref()
        && !is_semver(version)
    {
        return Err(CedarPolicyPublishApiError::InvalidSupersedes {
            supersedes: version.clone(),
        });
    }
    Ok(())
}

fn parse_policy_effect(effect: &str) -> Result<PolicyEffect, CedarPolicyPublishApiError> {
    match effect {
        "allow" => Ok(PolicyEffect::Allow),
        "deny" => Ok(PolicyEffect::Deny),
        _ => Err(CedarPolicyPublishApiError::InvalidRuleEffect {
            effect: effect.to_string(),
        }),
    }
}

fn validate_required_attribute(
    attribute: Option<&CedarPolicyRequiredAttribute>,
) -> Result<(), CedarPolicyPublishApiError> {
    if let Some(attribute) = attribute {
        if attribute.key.trim().is_empty() {
            return Err(CedarPolicyPublishApiError::EmptyRequiredAttributeKey);
        }
        if attribute.value.trim().is_empty() {
            return Err(CedarPolicyPublishApiError::EmptyRequiredAttributeValue);
        }
    }
    Ok(())
}

fn policy_version_from_request(
    body: &CedarPolicyPublishRequest,
) -> Result<PolicyVersion, CedarPolicyPublishApiError> {
    Ok(PolicyVersion {
        policy_id: body.policy_id.clone(),
        version: body.version.clone(),
        scope: parse_policy_scope(&body.scope)?,
        supersedes: body.supersedes.clone(),
        rules: body
            .rules
            .iter()
            .map(policy_rule_input_from_ref)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn policy_rule_input_from_ref(
    rule: &CedarPolicyRuleRef,
) -> Result<PolicyRuleInput, CedarPolicyPublishApiError> {
    validate_required_attribute(rule.required_attribute.as_ref())?;
    Ok(PolicyRuleInput {
        effect: parse_policy_effect(&rule.effect)?,
        principal_role: rule.principal_role.clone(),
        action: rule.action.clone(),
        resource_prefix: rule.resource_prefix.clone(),
        required_attribute: rule
            .required_attribute
            .as_ref()
            .map(|attribute| (attribute.key.clone(), attribute.value.clone())),
        annotations: Vec::new(),
    })
}

fn idempotency_key_for(
    boundary: &CedarPolicyApiBoundaryContext,
    principal: &CedarPolicyApiPrincipal,
    surface: &str,
) -> CedarPolicyPublishIdempotencyLedgerKey {
    CedarPolicyPublishIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn cedar_policy_publish_fingerprint_for(
    request: &CedarPolicyPublishApiRequest,
) -> CedarPolicyPublishRequestFingerprint {
    let rules = request
        .body
        .rules
        .iter()
        .map(|rule| {
            let attribute = rule
                .required_attribute
                .as_ref()
                .map(|attribute| format!("{}={}", attribute.key, attribute.value))
                .unwrap_or_else(|| "none".to_string());
            format!(
                "{}:{}:{}:{}:{}",
                rule.effect, rule.principal_role, rule.action, rule.resource_prefix, attribute
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    CedarPolicyPublishRequestFingerprint {
        canonical: [
            format!("path.policy_id={}", request.path_policy_id),
            format!("path.version={}", request.path_version),
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
            format!("body.policy_id={}", request.body.policy_id),
            format!("body.version={}", request.body.version),
            format!("body.scope.kind={}", request.body.scope.kind),
            format!(
                "body.scope.tenant_id={}",
                request.body.scope.tenant_id.clone().unwrap_or_default()
            ),
            format!(
                "body.supersedes={}",
                request.body.supersedes.clone().unwrap_or_default()
            ),
            format!("body.rules={rules}"),
        ]
        .join("\n"),
    }
}

fn policy_record(published: &PublishedPolicy) -> CedarPolicyRecord {
    CedarPolicyRecord {
        policy_id: published.policy_id.clone(),
        version: published.version.clone(),
        scope: scope_ref(&published.scope),
        supersedes: published.supersedes.clone(),
        rules: published
            .rules
            .iter()
            .map(|rule| CedarPolicyRuleRef {
                effect: effect_label(rule.effect).to_string(),
                principal_role: rule.principal_role.clone(),
                action: rule.action.clone(),
                resource_prefix: rule.resource_prefix.clone(),
                required_attribute: rule.required_attribute.as_ref().map(|(key, value)| {
                    CedarPolicyRequiredAttribute {
                        key: key.clone(),
                        value: value.clone(),
                    }
                }),
            })
            .collect(),
        schema_version: 1,
    }
}

fn scope_ref(scope: &PolicyScope) -> CedarPolicyScopeRef {
    match scope {
        PolicyScope::Global => CedarPolicyScopeRef {
            kind: "global".to_string(),
            tenant_id: None,
        },
        PolicyScope::Tenant(tenant_id) => CedarPolicyScopeRef {
            kind: "tenant".to_string(),
            tenant_id: Some(tenant_id.clone()),
        },
    }
}

fn effect_label(effect: PolicyEffect) -> &'static str {
    match effect {
        PolicyEffect::Allow => "allow",
        PolicyEffect::Deny => "deny",
    }
}

fn is_semver(version: &str) -> bool {
    let parts = version.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.parse::<u64>().is_ok())
}

fn policy_error_code(error: &PolicyError) -> CedarPolicyPublishApiErrorCode {
    match error {
        PolicyError::InvalidPolicyId => CedarPolicyPublishApiErrorCode::PolicyInvalidId,
        PolicyError::InvalidSemver => CedarPolicyPublishApiErrorCode::PolicyInvalidSemver,
        PolicyError::EmptyRules => CedarPolicyPublishApiErrorCode::PolicyEmptyRules,
        PolicyError::EmptyRuleField => CedarPolicyPublishApiErrorCode::PolicyEmptyRuleField,
        PolicyError::VersionAlreadyExists => {
            CedarPolicyPublishApiErrorCode::PolicyVersionAlreadyExists
        }
        PolicyError::SupersedesSelf => CedarPolicyPublishApiErrorCode::PolicySupersedesSelf,
        PolicyError::SupersedesMissing => CedarPolicyPublishApiErrorCode::PolicySupersedesMissing,
        PolicyError::SupersedesScopeMismatch => {
            CedarPolicyPublishApiErrorCode::PolicySupersedesScopeMismatch
        }
        PolicyError::SupersedesNotOlder => CedarPolicyPublishApiErrorCode::PolicySupersedesNotOlder,
    }
}

fn policy_error_message(error: &PolicyError) -> &'static str {
    match error {
        PolicyError::InvalidPolicyId => "Policy id must be a pol_ identifier",
        PolicyError::InvalidSemver => "Policy version must use semver major.minor.patch",
        PolicyError::EmptyRules => "Policy must contain at least one rule",
        PolicyError::EmptyRuleField => "Policy rules require role, action, and resource prefix",
        PolicyError::VersionAlreadyExists => "Policy version already exists",
        PolicyError::SupersedesSelf => "Policy version cannot supersede itself",
        PolicyError::SupersedesMissing => "Superseded policy version does not exist",
        PolicyError::SupersedesScopeMismatch => {
            "Superseded policy version must have matching scope"
        }
        PolicyError::SupersedesNotOlder => "Superseded policy version must be older",
    }
}

fn policy_error_issue(error: &PolicyError) -> &'static str {
    match error {
        PolicyError::InvalidPolicyId => "policy_id must start with pol_",
        PolicyError::InvalidSemver => "version must have three numeric components",
        PolicyError::EmptyRules => "rules must be non-empty",
        PolicyError::EmptyRuleField => "rule fields must be non-empty",
        PolicyError::VersionAlreadyExists => "policy_id/version pair must be immutable",
        PolicyError::SupersedesSelf => "supersedes must not equal version",
        PolicyError::SupersedesMissing => "supersedes must reference an existing policy_id/version",
        PolicyError::SupersedesScopeMismatch => {
            "supersedes must reference a policy version with the same scope"
        }
        PolicyError::SupersedesNotOlder => "supersedes must reference an older version",
    }
}

fn detail(field: &str, issue: &str) -> CedarPolicyPublishApiErrorDetail {
    CedarPolicyPublishApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
