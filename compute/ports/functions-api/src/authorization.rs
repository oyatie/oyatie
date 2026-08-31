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
