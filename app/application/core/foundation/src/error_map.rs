//! Domain-error to foundation-error mapping.

use crate::*;

use crate::error::FoundationError;

pub(crate) fn map_tenant_error(error: TenantError) -> FoundationError {
    match error {
        TenantError::InvalidTenantId
        | TenantError::EmptyLegalName
        | TenantError::EmptyHomeRegion
        | TenantError::HomeRegionNotAllowedForResidency
        | TenantError::MissingRegionalPack => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_identity_error(error: IdentityError) -> FoundationError {
    match error {
        IdentityError::TokenTtlTooLong => FoundationError::TokenTtlTooLong,
        IdentityError::InvalidTenantId
        | IdentityError::InvalidUserId
        | IdentityError::InvalidRegionPack
        | IdentityError::InvalidIdentityProviderId
        | IdentityError::InvalidServicePrincipalId
        | IdentityError::InvalidCapabilityId
        | IdentityError::EmptyPrimaryIdentifier
        | IdentityError::EmptyExternalSubject
        | IdentityError::TokenTtlZero
        | IdentityError::MissingCredentialScope
        | IdentityError::LongLivedCredentialForbidden => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_capability_error(error: CapabilityError) -> FoundationError {
    match error {
        CapabilityError::InvalidCapabilityId
        | CapabilityError::InvalidTenantId
        | CapabilityError::EmptyNamespace
        | CapabilityError::EmptyEvidenceTopic
        | CapabilityError::MissingDataClasses
        | CapabilityError::NonPrivacyDataClass
        | CapabilityError::InvalidCostProfile
        | CapabilityError::MissingProviderPreference
        | CapabilityError::InvalidProviderPreference
        | CapabilityError::InvalidMcpContract => FoundationError::InvalidInput,
        CapabilityError::DuplicateCapability => FoundationError::CapabilityAlreadyExists,
        CapabilityError::CapabilityNotFound => FoundationError::CapabilityNotFound,
    }
}

pub(crate) fn map_eval_error(error: EvalError) -> FoundationError {
    match error {
        EvalError::EvalSetNotFound
        | EvalError::MissingPassingEvalRun
        | EvalError::UnsignedEvalSet
        | EvalError::MissingAdversarialCoverage
        | EvalError::MissingLinguisticCoverage
        | EvalError::UnsignedEvalRun
        | EvalError::EvalRunVersionMismatch
        | EvalError::EvalRunBelowThreshold => FoundationError::CapabilityEvalGateNotReady,
        EvalError::InvalidCapabilityId
        | EvalError::EmptyVersion
        | EvalError::EmptyCaseId
        | EvalError::EmptyLocale
        | EvalError::EmptyInputRef
        | EvalError::EmptyExpectedRef
        | EvalError::InvalidThreshold
        | EvalError::EmptyEvalSet => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_mcp_error(error: McpGatewayError) -> FoundationError {
    match error {
        McpGatewayError::TenantMismatch | McpGatewayError::MissingScope => {
            FoundationError::McpAccessDenied
        }
        McpGatewayError::TokenAudienceMismatch
        | McpGatewayError::TokenIssuerMismatch
        | McpGatewayError::TokenExpired => FoundationError::McpAccessDenied,
        McpGatewayError::RateLimitExceeded => FoundationError::McpRateLimited,
        McpGatewayError::AutonomyCeilingExceeded => FoundationError::AutonomyCeilingExceeded,
        McpGatewayError::InvalidTenantId
        | McpGatewayError::EmptySubjectId
        | McpGatewayError::EmptyRegion
        | McpGatewayError::EmptyTld
        | McpGatewayError::InvalidHostSegment
        | McpGatewayError::EmptyAuthorizationServer
        | McpGatewayError::InvalidAuthorizationServer
        | McpGatewayError::InvalidRateLimitPolicy
        | McpGatewayError::EmptyToolName
        | McpGatewayError::InvalidToolName
        | McpGatewayError::ToolNameTooLong => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_budget_error(error: BudgetError) -> FoundationError {
    match error {
        BudgetError::MissingBudgetCeiling => FoundationError::CostBudgetNotConfigured,
        BudgetError::PerInvocationLimitExceeded
        | BudgetError::TenantMonthlyLimitExceeded
        | BudgetError::CapabilityMonthlyLimitExceeded => FoundationError::CostBudgetExceeded,
        BudgetError::InvalidTenantId
        | BudgetError::InvalidCapabilityId
        | BudgetError::InvalidWindowId
        | BudgetError::InvalidBudgetCeiling
        | BudgetError::NonPositiveAmount
        | BudgetError::ReservationNotFound
        | BudgetError::ReservationNotPending => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_bypass_error(_error: BypassError) -> FoundationError {
    FoundationError::InvalidInput
}

pub(crate) fn map_run_error(error: RunError) -> FoundationError {
    match error {
        RunError::InvalidTenantId
        | RunError::InvalidCapabilityId
        | RunError::InvalidInitiatorId
        | RunError::InvalidRunHistory
        | RunError::MissingDataClasses
        | RunError::InvalidDataClass
        | RunError::EmptyRegion
        | RunError::EmptyIdempotencyKey
        | RunError::RunNotFound
        | RunError::RunNotRunning => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_step_error(error: StepError) -> FoundationError {
    match error {
        StepError::InvalidRunId
        | StepError::InvalidStepHistory
        | StepError::EmptyProviderKind
        | StepError::EmptyModelRef
        | StepError::MissingDataClasses
        | StepError::InvalidDataClass
        | StepError::StepNotFound
        | StepError::StepNotRunning => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_adapter_error(error: AdapterError) -> FoundationError {
    match error {
        AdapterError::DataClassNotAllowed => FoundationError::DataUseNotAllowed,
        AdapterError::InvalidCostCeiling | AdapterError::NoProviderAvailable => {
            FoundationError::CostBudgetExceeded
        }
        AdapterError::InvalidProviderId
        | AdapterError::InvalidTenantId
        | AdapterError::EmptyProviderAccount
        | AdapterError::EmptyFailoverChain
        | AdapterError::MissingDataClassAllowlist
        | AdapterError::MissingProviderRegion
        | AdapterError::MissingProviderCapability
        | AdapterError::AuthModeMismatch
        | AdapterError::InvalidRequiredRegion
        | AdapterError::EmptyProviderCallIdempotencyKey
        | AdapterError::EmptyProviderModelRef
        | AdapterError::InvalidProviderCallAttempt
        | AdapterError::EmptyProviderRequestId
        | AdapterError::EmptyProviderPromptRef
        | AdapterError::EmptyProviderToolName
        | AdapterError::ProviderAdapterMismatch
        | AdapterError::InvalidProviderEventSequence
        | AdapterError::ProviderRetryableFailure
        | AdapterError::ProviderNonRetryableFailure
        | AdapterError::ProviderCallRegionMismatch
        | AdapterError::InvalidDataClass => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_evidence_error(error: EvidenceError) -> FoundationError {
    match error {
        EvidenceError::InvalidEvidenceId
        | EvidenceError::InvalidTenantId
        | EvidenceError::InvalidRunId
        | EvidenceError::InvalidStepId
        | EvidenceError::InvalidCapabilityId
        | EvidenceError::EmptyFields
        | EvidenceError::MissingDataClasses
        | EvidenceError::InvalidDataClass => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_regional_pack_error(error: RegionalPackError) -> FoundationError {
    match error {
        RegionalPackError::InvalidPackId
        | RegionalPackError::EmptyRegion
        | RegionalPackError::EmptyResidencyClass
        | RegionalPackError::InvalidResidencyClass
        | RegionalPackError::MissingControls => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_object_graph_error(error: ObjectGraphError) -> FoundationError {
    match error {
        ObjectGraphError::InvalidEntityId
        | ObjectGraphError::EmptyEntityType
        | ObjectGraphError::MissingProperties
        | ObjectGraphError::EmptyPropertyName
        | ObjectGraphError::InvalidDataClass => FoundationError::InvalidInput,
    }
}

pub(crate) fn map_eventing_error(error: EventingError) -> FoundationError {
    match error {
        EventingError::EmptyTopic
        | EventingError::EmptyTopicAxis
        | EventingError::EmptyTopicDescription
        | EventingError::InvalidTopicName
        | EventingError::DuplicateTopic
        | EventingError::TopicNotFound
        | EventingError::EmptyIdempotencyKey
        | EventingError::EmptyPayloadRef
        | EventingError::IdempotencyReplayMismatch
        | EventingError::InvalidOutboxHistory => FoundationError::InvalidInput,
        EventingError::OutboxRecordNotFound => FoundationError::OutboxRecordNotFound,
    }
}
