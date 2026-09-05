#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sAcceptanceApiError {
    Boundary(CloudComputeK8sApiError),
    IdempotencyKeyReused,
    OperationContractMismatch,
    ResourceMismatch,
    RepositoryUnavailable,
    OutcomeUnknown,
    IntegrityViolation,
}
impl CloudComputeK8sAcceptanceApiError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Boundary(error) => error.cluster_create_status_code(),
            Self::IdempotencyKeyReused | Self::OperationContractMismatch | Self::ResourceMismatch => 422,
            Self::RepositoryUnavailable | Self::OutcomeUnknown | Self::IntegrityViolation => 503,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Boundary(error) => error.code().as_str(),
            Self::IdempotencyKeyReused => "CLOUD_COMPUTE_K8S_ACCEPTANCE_IDEMPOTENCY_KEY_REUSED",
            Self::OperationContractMismatch => "CLOUD_COMPUTE_K8S_ACCEPTANCE_CONTRACT_MISMATCH",
            Self::ResourceMismatch => "CLOUD_COMPUTE_K8S_ACCEPTANCE_RESOURCE_MISMATCH",
            Self::RepositoryUnavailable => "CLOUD_COMPUTE_K8S_ACCEPTANCE_REPOSITORY_UNAVAILABLE",
            Self::OutcomeUnknown => "CLOUD_COMPUTE_K8S_ACCEPTANCE_OUTCOME_UNKNOWN",
            Self::IntegrityViolation => "CLOUD_COMPUTE_K8S_ACCEPTANCE_INTEGRITY_VIOLATION",
        }
    }
}
fn acceptance_repository_error(error: CloudComputeK8sAcceptanceRepositoryError) -> CloudComputeK8sAcceptanceApiError {
    match error {
        CloudComputeK8sAcceptanceRepositoryError::IdempotencyKeyReused => CloudComputeK8sAcceptanceApiError::IdempotencyKeyReused,
        CloudComputeK8sAcceptanceRepositoryError::OperationContractMismatch => CloudComputeK8sAcceptanceApiError::OperationContractMismatch,
        CloudComputeK8sAcceptanceRepositoryError::ResourceMismatch => CloudComputeK8sAcceptanceApiError::ResourceMismatch,
        CloudComputeK8sAcceptanceRepositoryError::Unavailable => CloudComputeK8sAcceptanceApiError::RepositoryUnavailable,
        CloudComputeK8sAcceptanceRepositoryError::OutcomeUnknown => CloudComputeK8sAcceptanceApiError::OutcomeUnknown,
        CloudComputeK8sAcceptanceRepositoryError::IntegrityViolation => CloudComputeK8sAcceptanceApiError::IntegrityViolation,
    }
}
