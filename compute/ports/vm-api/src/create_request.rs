#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmCreateRequest {
    pub resource_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: PUBLIC
    pub az: String,                       // data_class: PUBLIC
    pub cell_id: String,                  // data_class: PUBLIC
    pub flavor: CloudComputeVmFlavorSpec, // data_class: PUBLIC
    pub image: String,                    // data_class: INTERNAL_ONLY
    pub key_pair: Option<String>,         // data_class: INTERNAL_ONLY
    pub vpc_id: String,                   // data_class: INTERNAL_ONLY
    pub subnet_id: String,                // data_class: INTERNAL_ONLY
    pub security_groups: Vec<CloudComputeVmSecurityGroupRef>, // data_class: INTERNAL_ONLY
    pub iam_role: Option<CloudComputeVmIamRoleRef>, // data_class: INTERNAL_ONLY
    pub user_data_uri: Option<String>,    // data_class: INTERNAL_ONLY
    pub quota: CloudComputeVmQuotaEnvelope, // data_class: INTERNAL_ONLY
    pub residency: String,                // data_class: INTERNAL_ONLY
    pub data_class: String,               // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmFlavorSpec {
    pub class: String,     // data_class: PUBLIC
    pub vcpu: u32,         // data_class: PUBLIC
    pub memory_gb: u32,    // data_class: PUBLIC
    pub gpu_count: u32,    // data_class: PUBLIC
    pub local_ssd_gb: u32, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloudComputeVmQuotaEnvelope {
    pub vcpu_limit: u32,           // data_class: INTERNAL_ONLY
    pub memory_gb_limit: u32,      // data_class: INTERNAL_ONLY
    pub gpu_limit: u32,            // data_class: INTERNAL_ONLY
    pub local_ssd_gb_limit: u32,   // data_class: INTERNAL_ONLY
    pub current_vcpu: u32,         // data_class: INTERNAL_ONLY
    pub current_memory_gb: u32,    // data_class: INTERNAL_ONLY
    pub current_gpu: u32,          // data_class: INTERNAL_ONLY
    pub current_local_ssd_gb: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmSecurityGroupRef {
    pub value: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub region: String,    // data_class: PUBLIC
    pub vpc_id: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmIamRoleRef {
    pub value: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub region: String,    // data_class: PUBLIC
    pub vpc_id: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmCreateApiRequest {
    pub path_instance_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudComputeVmApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudComputeVmApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudComputeVmApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudComputeVmCreateRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmCreateIdempotencyLedger {
    entries: BTreeMap<CloudComputeVmIdempotencyLedgerKey, CloudComputeVmCreateLedgerEntry>, // data_class: INTERNAL_ONLY
    max_entries: usize, // data_class: INTERNAL_ONLY
}

impl Default for CloudComputeVmCreateIdempotencyLedger {
    fn default() -> Self {
        Self::with_max_entries(DEFAULT_VM_CREATE_IDEMPOTENCY_LEDGER_MAX_ENTRIES)
    }
}

impl CloudComputeVmCreateIdempotencyLedger {
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            max_entries: max_entries.max(1),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn remember(
        &mut self,
        key: CloudComputeVmIdempotencyLedgerKey,
        entry: CloudComputeVmCreateLedgerEntry,
    ) {
        if self.entries.len() >= self.max_entries
            && let Some(evicted) = self.entries.keys().next().cloned()
        {
            self.entries.remove(&evicted);
        }
        self.entries.insert(key, entry);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudComputeVmIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeVmCreateLedgerEntry {
    fingerprint: CloudComputeVmRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudComputeVmCreateApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeVmRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudComputeVmCreateApiResult =
    Result<CloudComputeVmCreateSuccessResponse, CloudComputeVmApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmCreateSuccessResponse {
    pub data: CloudComputeVmRecord,       // data_class: INTERNAL_ONLY
    pub metadata: CloudComputeVmMetadata, // data_class: INTERNAL_ONLY
}

impl CloudComputeVmCreateSuccessResponse {
    pub fn created(data: CloudComputeVmRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudComputeVmMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmRecord {
    pub resource_id: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: PUBLIC
    pub az: String,                       // data_class: PUBLIC
    pub cell_id: String,                  // data_class: PUBLIC
    pub flavor: CloudComputeVmFlavorSpec, // data_class: PUBLIC
    pub image_kind: String,               // data_class: PUBLIC
    pub residency: String,                // data_class: INTERNAL_ONLY
    pub state: String,                    // data_class: PUBLIC
    pub data_class: String,               // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiErrorResponse {
    pub error: CloudComputeVmApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiErrorBody {
    pub code: String,                               // data_class: INTERNAL_ONLY
    pub message: String,                            // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,          // data_class: INTERNAL_ONLY
    pub request_id: String,                         // data_class: INTERNAL_ONLY
    pub details: Vec<CloudComputeVmApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeVmApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeVmApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathInstanceId,
    InvalidInstanceId {
        instance_id: String,
    },
    InstanceKindMismatch {
        instance_id: String,
        kind_label: String,
    },
    InstanceIdMismatch {
        path_instance_id: String,
        body_resource_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        resource_tenant_id: String,
        body_tenant_id: String,
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
    InvalidFlavorClassLabel {
        class: String,
    },
    InvalidResidencyLabel {
        residency: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    SecurityGroupBindingMismatch {
        security_group: String,
        tenant_id: String,
        region: String,
        vpc_id: String,
    },
    IamRoleBindingMismatch {
        role_id: String,
        tenant_id: String,
        region: String,
        vpc_id: String,
    },
    Compute(CloudComputeError),
}
