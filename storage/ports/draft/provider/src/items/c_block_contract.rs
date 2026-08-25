#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderBlockCreateVolumeRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_volume_ref: String,     // data_class: INTERNAL_ONLY
    pub volume_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub name: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                  // data_class: PUBLIC
    pub az: String,                      // data_class: PUBLIC
    pub cell_id: String,                 // data_class: PUBLIC
    pub residency: ResidencyClass,       // data_class: INTERNAL_ONLY
    pub tier: VolumeTier,                // data_class: PUBLIC
    pub size_gib: u64,                   // data_class: INTERNAL_ONLY
    pub performance: VolumePerformance,  // data_class: PUBLIC
    pub encryption: EncryptionMode,      // data_class: PUBLIC
    pub kms_key: Option<String>,         // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderBlockReceipt {
    pub provider: StorageProviderKind,    // data_class: PUBLIC
    pub operation: StorageBlockOperation, // data_class: PUBLIC
    pub request_id: String,               // data_class: INTERNAL_ONLY
    pub provider_request_id: String,      // data_class: INTERNAL_ONLY
    pub provider_volume_ref: String,      // data_class: INTERNAL_ONLY
    pub volume_id: String,                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub name: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: PUBLIC
    pub az: String,                       // data_class: PUBLIC
    pub cell_id: String,                  // data_class: PUBLIC
    pub residency: ResidencyClass,        // data_class: INTERNAL_ONLY
    pub tier: VolumeTier,                 // data_class: PUBLIC
    pub size_gib: u64,                    // data_class: INTERNAL_ONLY
    pub performance: VolumePerformance,   // data_class: PUBLIC
    pub encryption: EncryptionMode,       // data_class: PUBLIC
    pub kms_key: Option<String>,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,            // data_class: INTERNAL_ONLY
    pub actor: String,                    // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,    // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub schema_version: u32,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageProviderBlockError {
    InvalidProviderVolumeRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
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

pub trait StorageProviderBlockPort {
    fn provider_kind(&self) -> StorageProviderKind;

    fn create_volume(
        &self,
        input: StorageProviderBlockCreateVolumeRequest,
    ) -> Result<StorageProviderBlockReceipt, StorageProviderBlockError>;
}
