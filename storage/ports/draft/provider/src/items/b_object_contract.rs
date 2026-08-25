#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderObjectPutRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_bucket_ref: String,     // data_class: INTERNAL_ONLY
    pub bucket_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub object_key: String,              // data_class: INTERNAL_ONLY
    pub object_body_ref: String,         // data_class: INTERNAL_ONLY
    pub size_bytes: u64,                 // data_class: INTERNAL_ONLY
    pub etag: String,                    // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: INTERNAL_ONLY
    pub kms_key: String,                 // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderObjectGetRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_bucket_ref: String,     // data_class: INTERNAL_ONLY
    pub bucket_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub object_key: String,              // data_class: INTERNAL_ONLY
    pub result_body_ref: String,         // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderObjectReceipt {
    pub provider: StorageProviderKind,     // data_class: PUBLIC
    pub operation: StorageObjectOperation, // data_class: PUBLIC
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub provider_request_id: String,       // data_class: INTERNAL_ONLY
    pub provider_bucket_ref: String,       // data_class: INTERNAL_ONLY
    pub bucket_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub object_key: String,                // data_class: INTERNAL_ONLY
    pub object_body_ref: String,           // data_class: INTERNAL_ONLY
    pub size_bytes: Option<u64>,           // data_class: INTERNAL_ONLY
    pub etag: Option<String>,              // data_class: INTERNAL_ONLY
    pub data_class: Option<DataClass>,     // data_class: INTERNAL_ONLY
    pub kms_key: Option<String>,           // data_class: INTERNAL_ONLY
    pub ciphertext_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub actor: String,                     // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,     // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,               // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageProviderObjectError {
    InvalidProviderBucketRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidObjectBodyRef,
    InvalidIdempotencyKey,
    InvalidActorRef,
    InvalidRequestShape(CloudStorageError),
    ProviderRejected {
        provider: StorageProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: StorageProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
}

pub trait StorageProviderObjectPort {
    fn provider_kind(&self) -> StorageProviderKind;

    fn put_object(
        &self,
        input: StorageProviderObjectPutRequest,
    ) -> Result<StorageProviderObjectReceipt, StorageProviderObjectError>;

    fn get_object(
        &self,
        input: StorageProviderObjectGetRequest,
    ) -> Result<StorageProviderObjectReceipt, StorageProviderObjectError>;
}
