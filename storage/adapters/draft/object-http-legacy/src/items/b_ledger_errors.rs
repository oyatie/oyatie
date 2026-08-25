/// Public projection of a recorded ledger entry.
///
/// Does not expose private `CloudStorageObjectPutLedgerEntry` or
/// `CloudStorageObjectRequestFingerprint`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectPutIdempotencyEntry {
    pub idempotency_key: String,                  // data_class: INTERNAL_ONLY
    pub outcome: CloudStorageObjectReplayOutcome, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudStorageObjectPutIdempotencyLedger {
    entries: BTreeMap<CloudStorageObjectIdempotencyLedgerKey, CloudStorageObjectPutLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudStorageObjectPutIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return a public projection of the recorded entry for the given composite key,
    /// or `None` if no entry has been recorded yet.
    ///
    /// Does not mutate the ledger. Does not drive the catalog.
    /// The `outcome` field reflects the *recorded* result, not a re-evaluation.
    pub fn peek(
        &self,
        tenant_id: &str,
        principal_id: &str,
        surface: &str,
        idempotency_key: &str,
    ) -> Option<CloudStorageObjectPutIdempotencyEntry> {
        let key = CloudStorageObjectIdempotencyLedgerKey {
            tenant_id: tenant_id.to_string(),
            principal_id: principal_id.to_string(),
            surface: surface.to_string(),
            idempotency_key: idempotency_key.to_string(),
        };
        let entry = self.entries.get(&key)?;
        let outcome = match &entry.result {
            Ok(response) => CloudStorageObjectReplayOutcome::Replayed {
                response: Box::new(response.clone()),
            },
            Err(_) => CloudStorageObjectReplayOutcome::Conflict {
                idempotency_key: idempotency_key.to_string(),
            },
        };
        Some(CloudStorageObjectPutIdempotencyEntry {
            idempotency_key: idempotency_key.to_string(),
            outcome,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudStorageObjectIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudStorageObjectPutLedgerEntry {
    fingerprint: CloudStorageObjectRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudStorageObjectPutApiResult,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudStorageObjectRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudStorageObjectPutApiResult =
    Result<CloudStorageObjectPutSuccessResponse, CloudStorageObjectApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectPutSuccessResponse {
    pub data: CloudStorageObjectRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudStorageObjectMetadata, // data_class: INTERNAL_ONLY
}

impl CloudStorageObjectPutSuccessResponse {
    pub fn created(data: CloudStorageObjectRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudStorageObjectMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectGetSuccessResponse {
    pub data: CloudStorageObjectRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudStorageObjectMetadata, // data_class: INTERNAL_ONLY
}

impl CloudStorageObjectGetSuccessResponse {
    pub fn ok(data: CloudStorageObjectRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudStorageObjectMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectRecord {
    pub bucket_id: String,  // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
    pub key: String,        // data_class: INTERNAL_ONLY
    pub size_bytes: u64,    // data_class: INTERNAL_ONLY
    pub etag: String,       // data_class: INTERNAL_ONLY
    pub data_class: String, // data_class: INTERNAL_ONLY
    pub encryption: CloudStorageObjectEncryptionBindingRecord, // data_class: INTERNAL_ONLY
    pub stored_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub last_accessed_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: u32, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectEncryptionBindingRecord {
    pub kms_key: String,                 // data_class: INTERNAL_ONLY
    pub kms_key_version: u32,            // data_class: INTERNAL_ONLY
    pub material_ref: String,            // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub kms_encrypt_event_id: String,    // data_class: INTERNAL_ONLY
    pub purpose: String,                 // data_class: INTERNAL_ONLY
    pub shred_proof_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectApiErrorResponse {
    pub error: CloudStorageObjectApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<CloudStorageObjectApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudStorageObjectApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudStorageObjectApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    InvalidBucketId {
        bucket_id: String,
    },
    BucketKindMismatch {
        bucket_id: String,
        kind_label: String,
    },
    InvalidObjectKey {
        object_key: String,
    },
    BucketIdMismatch {
        path_bucket_id: String,
        body_bucket_id: String,
    },
    ObjectKeyMismatch {
        path_object_key: String,
        body_key: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        resource_tenant_id: String,
        body_tenant_id: Option<String>,
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
    InvalidDataClassLabel {
        data_class: String,
    },
    InvalidKmsPurposeLabel {
        purpose: String,
    },
    ObjectNotFound {
        bucket_id: String,
        key: String,
    },
    Storage(CloudStorageError),
}
