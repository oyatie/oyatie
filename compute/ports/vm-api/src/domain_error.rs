fn vm_record(instance: Instance) -> CloudComputeVmRecord {
    let flavor = instance.flavor.value;
    let image = instance.image.value;
    CloudComputeVmRecord {
        resource_id: instance.resource_id.value.value,
        tenant_id: instance.tenant_id.value,
        region: instance.region.value.value,
        az: instance.az.value.value,
        cell_id: instance.cell_id.value.value,
        flavor: CloudComputeVmFlavorSpec {
            class: flavor_class_label(flavor.class).to_string(),
            vcpu: flavor.vcpu,
            memory_gb: flavor.memory_gb,
            gpu_count: flavor.gpu_count,
            local_ssd_gb: flavor.local_ssd_gb,
        },
        image_kind: image_kind_label(image.kind).to_string(),
        residency: instance
            .residency
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        state: instance_state_label(instance.state.value).to_string(),
        data_class: instance.data_class.value.label().to_string(),
        created_at_epoch_seconds: instance.created_at_epoch_seconds.value,
        schema_version: instance.schema_version.value,
    }
}

fn cloud_compute_status_kind(error: &CloudComputeError) -> CloudComputeVmApiStatusKind {
    match error {
        CloudComputeError::DuplicateInstance
        | CloudComputeError::DuplicateKubernetesCluster
        | CloudComputeError::DuplicateFunction
        | CloudComputeError::DuplicateInvocation => CloudComputeVmApiStatusKind::Conflict,
        CloudComputeError::UnknownFunction => CloudComputeVmApiStatusKind::NotFound,
        CloudComputeError::ResourceTenantMismatch
        | CloudComputeError::ResourceRegionMismatch
        | CloudComputeError::ResidencyRegionMismatch
        | CloudComputeError::QuotaExceeded
        | CloudComputeError::PayloadDataClassNotAllowed => CloudComputeVmApiStatusKind::Forbidden,
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
        | CloudComputeError::InvalidIdempotencyKey
        | CloudComputeError::FunctionNotActive => CloudComputeVmApiStatusKind::BadRequest,
    }
}

fn cloud_compute_message(error: &CloudComputeError) -> &'static str {
    match cloud_compute_status_kind(error) {
        CloudComputeVmApiStatusKind::BadRequest => "Cloud Compute rejected the request shape",
        CloudComputeVmApiStatusKind::Unauthorized => {
            "Cloud Compute authentication evidence is missing"
        }
        CloudComputeVmApiStatusKind::Forbidden => "Cloud Compute policy denied the request",
        CloudComputeVmApiStatusKind::NotFound => "Cloud Compute resource was not found",
        CloudComputeVmApiStatusKind::Conflict => "Cloud Compute resource already exists",
        CloudComputeVmApiStatusKind::UnprocessableEntity => {
            "Cloud Compute rejected request idempotency"
        }
    }
}

fn cloud_compute_issue(error: &CloudComputeError) -> &'static str {
    match error {
        CloudComputeError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudComputeError::InvalidResourceId => "resource_id must be canonical cloud resource id",
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
        CloudComputeError::InvalidDataClass => "data_class must be public metadata class",
        CloudComputeError::InvalidImageRef => "image must be a supported digest-pinned image ref",
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
        CloudComputeError::QuotaExceeded => "requested VM exceeds tenant quota envelope",
        CloudComputeError::InvalidInstanceState => "VM create requests must start in Pending state",
        CloudComputeError::InvalidKubernetesState => {
            "Kubernetes create requests must start in Creating state"
        }
        CloudComputeError::InvalidFunctionState => {
            "function create requests must start in Deploying state"
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
        CloudComputeError::InvalidIdempotencyKey => "function idempotency key must be bounded",
        CloudComputeError::FunctionNotActive => "function must be active before invocation",
        CloudComputeError::PayloadDataClassNotAllowed => {
            "payload data_class must be admitted by deployment policy"
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

fn detail(field: &str, issue: &str) -> CloudComputeVmApiErrorDetail {
    CloudComputeVmApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
