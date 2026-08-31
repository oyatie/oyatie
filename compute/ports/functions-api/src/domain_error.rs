fn cloud_compute_status_kind(error: &CloudComputeError) -> CloudComputeFunctionsApiStatusKind {
    match error {
        CloudComputeError::DuplicateInstance
        | CloudComputeError::DuplicateKubernetesCluster
        | CloudComputeError::DuplicateFunction
        | CloudComputeError::DuplicateInvocation
        | CloudComputeError::FunctionNotActive => CloudComputeFunctionsApiStatusKind::Conflict,
        CloudComputeError::UnknownFunction => CloudComputeFunctionsApiStatusKind::NotFound,
        CloudComputeError::ResourceTenantMismatch
        | CloudComputeError::ResourceRegionMismatch
        | CloudComputeError::ResidencyRegionMismatch
        | CloudComputeError::QuotaExceeded
        | CloudComputeError::PayloadDataClassNotAllowed => {
            CloudComputeFunctionsApiStatusKind::Forbidden
        }
        CloudComputeError::InvalidTenantId
        | CloudComputeError::InvalidResourceId
        | CloudComputeError::ResourceKindMismatch
        | CloudComputeError::InvalidAzCode
        | CloudComputeError::AzRegionMismatch
        | CloudComputeError::InvalidCellId
        | CloudComputeError::CellAzMismatch
        | CloudComputeError::InvalidDataClass
        | CloudComputeError::InvalidImageRef
        | CloudComputeError::InvalidKeyPairId
        | CloudComputeError::InvalidUserDataUri
        | CloudComputeError::InvalidWorkloadIdentityPolicy
        | CloudComputeError::InvalidRuntimeIsolationPolicy
        | CloudComputeError::InvalidSchedulingPolicy
        | CloudComputeError::InvalidAuditEvidenceRef
        | CloudComputeError::InvalidFlavor
        | CloudComputeError::InvalidQuota
        | CloudComputeError::InvalidInstanceState
        | CloudComputeError::InvalidKubernetesState
        | CloudComputeError::InvalidFunctionState
        | CloudComputeError::InvalidNodePoolId
        | CloudComputeError::DuplicateNodePool
        | CloudComputeError::InvalidNodePoolShape
        | CloudComputeError::KubernetesHaRequiresThreeAzs
        | CloudComputeError::InvalidControlPlaneVersion
        | CloudComputeError::InvalidFunctionName
        | CloudComputeError::InvalidFunctionBudget
        | CloudComputeError::InvalidInvocationId
        | CloudComputeError::InvalidIdempotencyKey => {
            CloudComputeFunctionsApiStatusKind::BadRequest
        }
    }
}

fn cloud_compute_message(error: &CloudComputeError) -> &'static str {
    match cloud_compute_status_kind(error) {
        CloudComputeFunctionsApiStatusKind::BadRequest => {
            "Cloud Compute rejected the request shape"
        }
        CloudComputeFunctionsApiStatusKind::Unauthorized => {
            "Cloud Compute authentication evidence is missing"
        }
        CloudComputeFunctionsApiStatusKind::Forbidden => "Cloud Compute policy denied the request",
        CloudComputeFunctionsApiStatusKind::NotFound => "Cloud Compute function was not found",
        CloudComputeFunctionsApiStatusKind::Conflict => {
            "Cloud Compute function state conflicts with the request"
        }
        CloudComputeFunctionsApiStatusKind::UnprocessableEntity => {
            "Cloud Compute rejected request idempotency"
        }
    }
}

fn cloud_compute_issue(error: &CloudComputeError) -> &'static str {
    match error {
        CloudComputeError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudComputeError::InvalidResourceId => "function_id must be canonical cloud resource id",
        CloudComputeError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudComputeError::ResourceRegionMismatch => "resource region must match request region",
        CloudComputeError::ResourceKindMismatch => {
            "resource kind must match requested compute type"
        }
        CloudComputeError::InvalidAzCode => "AZ must be canonical lowercase ASCII",
        CloudComputeError::AzRegionMismatch => "AZ code must sit under its region code",
        CloudComputeError::InvalidCellId => "cell_id must be canonical and use the cell- prefix",
        CloudComputeError::CellAzMismatch => "cell_id must sit under its AZ namespace",
        CloudComputeError::ResidencyRegionMismatch => "region must satisfy residency policy",
        CloudComputeError::InvalidDataClass => "payload_data_class must be a privacy data class",
        CloudComputeError::InvalidImageRef => "function bundle must be a supported image ref",
        CloudComputeError::InvalidKeyPairId => "key_pair must use the key_ prefix",
        CloudComputeError::InvalidUserDataUri => "user_data_uri must use the userdata/ prefix",
        CloudComputeError::InvalidWorkloadIdentityPolicy => {
            "workload identity refs must be tenant/cell scoped and non-secret"
        }
        CloudComputeError::InvalidRuntimeIsolationPolicy => {
            "compute workloads require private and sandboxed runtime isolation"
        }
        CloudComputeError::InvalidSchedulingPolicy => {
            "compute scheduling evidence must require topology spread"
        }
        CloudComputeError::InvalidAuditEvidenceRef => {
            "compute audit evidence ref must be a non-secret evidence path"
        }
        CloudComputeError::InvalidFlavor => {
            "flavor resources must be positive and class-consistent"
        }
        CloudComputeError::InvalidQuota => "quota envelope must not start beyond its limits",
        CloudComputeError::QuotaExceeded => {
            "requested function exceeds tenant quota envelope or concurrency limit"
        }
        CloudComputeError::InvalidInstanceState => "VM create requests must start in Pending state",
        CloudComputeError::InvalidKubernetesState => {
            "Kubernetes create requests must start in Creating state"
        }
        CloudComputeError::InvalidFunctionState => {
            "function deployment state is not valid for this operation"
        }
        CloudComputeError::InvalidNodePoolId => "node pool id must use the np_ prefix",
        CloudComputeError::DuplicateNodePool => "node pool ids must be unique",
        CloudComputeError::InvalidNodePoolShape => "node pool shape must be canonical",
        CloudComputeError::KubernetesHaRequiresThreeAzs => {
            "HA Kubernetes requires at least three AZs"
        }
        CloudComputeError::InvalidControlPlaneVersion => "control plane version must be canonical",
        CloudComputeError::InvalidFunctionName => "function name must be canonical",
        CloudComputeError::InvalidFunctionBudget => {
            "function budget must be within platform bounds"
        }
        CloudComputeError::InvalidInvocationId => "invocation id must use the fninv_ prefix",
        CloudComputeError::InvalidIdempotencyKey => "idempotency key must be bounded",
        CloudComputeError::FunctionNotActive => "function must be active before invocation",
        CloudComputeError::PayloadDataClassNotAllowed => {
            "payload data class must be admitted by deployment policy"
        }
        CloudComputeError::DuplicateInstance => "instance resource id is already present",
        CloudComputeError::DuplicateKubernetesCluster => {
            "Kubernetes cluster resource id is already present"
        }
        CloudComputeError::DuplicateFunction => "function resource id is already present",
        CloudComputeError::DuplicateInvocation => "function invocation id is already present",
        CloudComputeError::UnknownFunction => "function resource must exist before invocation",
    }
}

fn detail(field: &str, issue: &str) -> CloudComputeFunctionsApiErrorDetail {
    CloudComputeFunctionsApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
