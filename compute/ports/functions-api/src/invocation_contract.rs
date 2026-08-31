#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvokeRequest {
    pub invocation_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub function_id: String,                 // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: PUBLIC
    pub payload_data_class: String,          // data_class: INTERNAL_ONLY
    pub current_concurrent_invocations: u32, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvokeApiRequest {
    pub path_function_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudComputeFunctionsApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudComputeFunctionsApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudComputeFunctionsApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudComputeFunctionsInvokeRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvokeIdempotencyLedger {
    entries:
        BTreeMap<CloudComputeFunctionsIdempotencyLedgerKey, CloudComputeFunctionsInvokeLedgerEntry>, // data_class: INTERNAL_ONLY
    max_entries: usize, // data_class: INTERNAL_ONLY
}

impl Default for CloudComputeFunctionsInvokeIdempotencyLedger {
    fn default() -> Self {
        Self::with_max_entries(DEFAULT_FUNCTIONS_INVOKE_IDEMPOTENCY_LEDGER_MAX_ENTRIES)
    }
}

impl CloudComputeFunctionsInvokeIdempotencyLedger {
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
        key: CloudComputeFunctionsIdempotencyLedgerKey,
        entry: CloudComputeFunctionsInvokeLedgerEntry,
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
struct CloudComputeFunctionsIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeFunctionsInvokeLedgerEntry {
    fingerprint: CloudComputeFunctionsRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudComputeFunctionsInvokeApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudComputeFunctionsRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudComputeFunctionsInvokeApiResult =
    Result<CloudComputeFunctionsInvokeSuccessResponse, CloudComputeFunctionsApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvokeSuccessResponse {
    pub data: CloudComputeFunctionsInvocationReceipt, // data_class: INTERNAL_ONLY
    pub metadata: CloudComputeFunctionsMetadata,      // data_class: INTERNAL_ONLY
}

impl CloudComputeFunctionsInvokeSuccessResponse {
    pub fn accepted(
        data: CloudComputeFunctionsInvocationReceipt,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            data,
            metadata: CloudComputeFunctionsMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsInvocationReceipt {
    pub invocation_id: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub function_id: String,            // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub payload_data_class: String,     // data_class: INTERNAL_ONLY
    pub cold_start_budget_ms: u32,      // data_class: PUBLIC
    pub accepted_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiErrorResponse {
    pub error: CloudComputeFunctionsApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiErrorBody {
    pub code: String,                                      // data_class: INTERNAL_ONLY
    pub message: String,                                   // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,                 // data_class: INTERNAL_ONLY
    pub request_id: String,                                // data_class: INTERNAL_ONLY
    pub details: Vec<CloudComputeFunctionsApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeFunctionsApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeFunctionsApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathFunctionId,
    InvalidFunctionId {
        function_id: String,
    },
    FunctionKindMismatch {
        function_id: String,
        kind_label: String,
    },
    FunctionIdMismatch {
        path_function_id: String,
        body_function_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        resource_tenant_id: String,
        body_tenant_id: String,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationVerifierMissing,
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
    InvalidPayloadDataClassLabel {
        payload_data_class: String,
    },
    Compute(CloudComputeError),
}
