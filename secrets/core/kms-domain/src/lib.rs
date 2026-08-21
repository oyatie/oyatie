//! Cloud KMS aggregate kernel.
//!
//! The kernel owns the typed control/data contract behind
//! `cloud.kms.encrypt` / `cloud.kms.decrypt`: per-tenant keys, per-cell HSM
//! partition binding, residency, pack-certified/FIPS validation, key-use receipts, and
//! key-destruction evidence. It does not perform cryptography or HSM I/O; those
//! belong in adapter/runtime crates that consume these invariants.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod envelope_keys;
pub use envelope_keys::{DekId, EnvelopeKeyError, KekId};

use std::collections::BTreeMap;

use cell_region::{CellId, RegionCode};
use compute_resource::{CloudResourceError, ResourceId, ResourceKind};
use network_residency::{ResidencyClass, residency_class_allows_home_region_label};
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const KMS_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const USER_PRINCIPAL_PREFIX: &str = "usr_";
const SERVICE_PRINCIPAL_PREFIX: &str = "sp_";
const EVENT_ID_PREFIX: &str = "kmsuse_";
const DESTRUCTION_PROOF_PREFIX: &str = "kproof_";
const SEALING_ROOT_PREFIX: &str = "sealing-root/";
const MATERIAL_REF_PREFIX: &str = "matref/";
const CIPHERTEXT_REF_PREFIX: &str = "ct/";
const KMS_KEY_PREFIX: &str = "kms";
const BYOK_KEY_PREFIX: &str = "byok";
const HYOK_KEY_PREFIX: &str = "hyok";
const HSM_PARTITION_PREFIX: &str = "hsm/";
const KMS_SYSTEM_ACTOR: &str = "sp_kms_control_plane";
const MAX_DESTRUCTION_SLA_SECONDS: u64 = 86_400;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KmsKeyId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HsmPartitionRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MaterialRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CiphertextRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ActorRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KmsUseEventId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DestructionProofRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SealingRootRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KmsKeyOrigin {
    OyatieManaged,
    Byok,
    Hyok,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KmsKeyUsage {
    EncryptDecrypt,
    SignVerify,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum HsmValidation {
    PackEnhancedFips1403Level3,
    Fips1403Level3,
    Cryptrec,
    CommonCriteriaEal4,
    PciHsm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KmsKeyState {
    PendingImport,
    Enabled,
    Disabled,
    PendingDeletion,
    Destroyed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KmsPurpose {
    CloudObjectStorage,
    CloudBlockStorage,
    CloudFileStorage,
    CloudArchiveStorage,
    WorkspaceDriveObject,
    WorkspaceRecording,
    SecretProvider,
    CrossRegionReplication,
    DatabaseBackup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KmsOperation {
    Encrypt,
    Decrypt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KmsProviderKind {
    OpenBaoTransit,
    OciKms,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudKmsEvidenceOperation {
    Encrypt,
    Decrypt,
    Rotate,
    Destroy,
    ProviderEncrypt,
    ProviderDecrypt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudKmsEvidenceStatus {
    Succeeded,
}

impl KmsProviderKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OpenBaoTransit => "openbao_transit",
            Self::OciKms => "oci_kms",
        }
    }
}

impl KmsOperation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Encrypt => "encrypt",
            Self::Decrypt => "decrypt",
        }
    }
}

impl KmsPurpose {
    pub const fn label(self) -> &'static str {
        match self {
            Self::CloudObjectStorage => "cloud_object_storage",
            Self::CloudBlockStorage => "cloud_block_storage",
            Self::CloudFileStorage => "cloud_file_storage",
            Self::CloudArchiveStorage => "cloud_archive_storage",
            Self::WorkspaceDriveObject => "workspace_drive_object",
            Self::WorkspaceRecording => "workspace_recording",
            Self::SecretProvider => "secret_provider",
            Self::CrossRegionReplication => "cross_region_replication",
            Self::DatabaseBackup => "database_backup",
        }
    }
}

impl CloudKmsEvidenceOperation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Encrypt => "encrypt",
            Self::Decrypt => "decrypt",
            Self::Rotate => "rotate",
            Self::Destroy => "destroy",
            Self::ProviderEncrypt => "provider_encrypt",
            Self::ProviderDecrypt => "provider_decrypt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsKeyCreate {
    pub resource_id: String,               // data_class: INTERNAL_ONLY
    pub key_id: String,                    // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub region: String,                    // data_class: PUBLIC
    pub cell_id: String,                   // data_class: PUBLIC
    pub hsm_partition_ref: String,         // data_class: INTERNAL_ONLY
    pub origin: KmsKeyOrigin,              // data_class: PUBLIC
    pub usage: KmsKeyUsage,                // data_class: PUBLIC
    pub hsm_validation: HsmValidation,     // data_class: PUBLIC
    pub residency: ResidencyClass,         // data_class: INTERNAL_ONLY
    pub data_class: DataClass,             // data_class: INTERNAL_ONLY
    pub state: KmsKeyState,                // data_class: PUBLIC
    pub rotation_period_days: Option<u16>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsKey {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub key_id: Classified<KmsKeyId>,        // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub cell_id: Classified<CellId>,         // data_class: PUBLIC
    pub hsm_partition_ref: Classified<HsmPartitionRef>, // data_class: INTERNAL_ONLY
    pub origin: Classified<KmsKeyOrigin>,    // data_class: PUBLIC
    pub usage: Classified<KmsKeyUsage>,      // data_class: PUBLIC
    pub hsm_validation: Classified<HsmValidation>, // data_class: PUBLIC
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<KmsKeyState>,      // data_class: PUBLIC
    pub current_version: Classified<u32>,    // data_class: INTERNAL_ONLY
    pub rotation_period_days: Classified<Option<u16>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KmsKeyVersionLifecycleState {
    Active,
    DecryptOnly,
    Quarantined,
    Destroyed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsKeyVersionLifecycle {
    pub key_id: Classified<KmsKeyId>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub version: Classified<u32>,      // data_class: INTERNAL_ONLY
    pub state: Classified<KmsKeyVersionLifecycleState>, // data_class: PUBLIC
    pub reason: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub activated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub decrypt_only_since_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub quarantined_since_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsSealingRootCreate {
    pub root_ref: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub cell_id: String,               // data_class: PUBLIC
    pub active_version: u32,           // data_class: INTERNAL_ONLY
    pub rotate_after_seconds: u64,     // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsSealingRoot {
    pub root_ref: Classified<SealingRootRef>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,       // data_class: PUBLIC
    pub cell_id: Classified<CellId>,          // data_class: PUBLIC
    pub active_version: Classified<u32>,      // data_class: INTERNAL_ONLY
    pub rotate_after_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyVersionDemotionRequest {
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub version: u32,                    // data_class: INTERNAL_ONLY
    pub reason: String,                  // data_class: INTERNAL_ONLY
    pub effective_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRingQuarantineRequest {
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub reason: String,                  // data_class: INTERNAL_ONLY
    pub effective_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsEncryptRequest {
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub region: String,                  // data_class: PUBLIC
    pub cell_id: String,                 // data_class: PUBLIC
    pub plaintext_ref: String,           // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: INTERNAL_ONLY
    pub purpose: KmsPurpose,             // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub aad_fingerprint: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsDecryptRequest {
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub region: String,                  // data_class: PUBLIC
    pub cell_id: String,                 // data_class: PUBLIC
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: INTERNAL_ONLY
    pub purpose: KmsPurpose,             // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsProviderEncryptRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_key_ref: String,        // data_class: INTERNAL_ONLY
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub plaintext_ref: String,           // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: INTERNAL_ONLY
    pub purpose: KmsPurpose,             // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub aad_fingerprint: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsProviderDecryptRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_key_ref: String,        // data_class: INTERNAL_ONLY
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: INTERNAL_ONLY
    pub purpose: KmsPurpose,             // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsProviderCryptoReceipt {
    pub provider: KmsProviderKind,      // data_class: PUBLIC
    pub operation: KmsOperation,        // data_class: PUBLIC
    pub request_id: String,             // data_class: INTERNAL_ONLY
    pub provider_request_id: String,    // data_class: INTERNAL_ONLY
    pub provider_key_ref: String,       // data_class: INTERNAL_ONLY
    pub key_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub material_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,         // data_class: INTERNAL_ONLY
    pub data_class: DataClass,          // data_class: INTERNAL_ONLY
    pub purpose: KmsPurpose,            // data_class: INTERNAL_ONLY
    pub actor: String,                  // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KmsProviderCryptoError {
    InvalidProviderKeyRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidRequestShape(CloudKmsError),
    ProviderRejected {
        provider: KmsProviderKind, // data_class: PUBLIC
        reason: String,            // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: KmsProviderKind, // data_class: PUBLIC
        reason: String,            // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsUseReceipt {
    pub event_id: Classified<KmsUseEventId>, // data_class: INTERNAL_ONLY
    pub key_id: Classified<KmsKeyId>,        // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub operation: Classified<KmsOperation>, // data_class: PUBLIC
    pub material_ref: Classified<Option<MaterialRef>>, // data_class: INTERNAL_ONLY
    pub ciphertext_ref: Classified<CiphertextRef>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub purpose: Classified<KmsPurpose>,     // data_class: INTERNAL_ONLY
    pub actor: Classified<ActorRef>,         // data_class: INTERNAL_ONLY
    pub key_version: Classified<u32>,        // data_class: INTERNAL_ONLY
    pub hsm_partition_ref: Classified<HsmPartitionRef>, // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyDestructionRequest {
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub proof_ref: String,               // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyDestructionReceipt {
    pub key_id: Classified<KmsKeyId>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub proof_ref: Classified<DestructionProofRef>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRotationRequest {
    pub key_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub rotation_evidence_ref: String,   // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRotationReceipt {
    pub key_id: Classified<KmsKeyId>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub actor: Classified<ActorRef>,   // data_class: INTERNAL_ONLY
    pub previous_key_version: Classified<u32>, // data_class: INTERNAL_ONLY
    pub new_key_version: Classified<u32>, // data_class: INTERNAL_ONLY
    pub rotation_evidence_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsEvidenceEvent {
    event_id: String,                     // data_class: INTERNAL_ONLY
    tenant_id: String,                    // data_class: INTERNAL_ONLY
    key_id: String,                       // data_class: INTERNAL_ONLY
    actor: String,                        // data_class: INTERNAL_ONLY
    operation: CloudKmsEvidenceOperation, // data_class: PUBLIC
    status: CloudKmsEvidenceStatus,       // data_class: PUBLIC
    evidence_ref: String,                 // data_class: INTERNAL_ONLY
    provider: Option<KmsProviderKind>,    // data_class: PUBLIC
    provider_request_id: Option<String>,  // data_class: INTERNAL_ONLY
    key_version: Option<u32>,             // data_class: INTERNAL_ONLY
    occurred_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudKmsEvidenceReceipt {
    event_id: String,                     // data_class: INTERNAL_ONLY
    tenant_id: String,                    // data_class: INTERNAL_ONLY
    key_id: String,                       // data_class: INTERNAL_ONLY
    operation: CloudKmsEvidenceOperation, // data_class: PUBLIC
    status: CloudKmsEvidenceStatus,       // data_class: PUBLIC
    evidence_ref: String,                 // data_class: INTERNAL_ONLY
    provider: Option<KmsProviderKind>,    // data_class: PUBLIC
    provider_request_id: Option<String>,  // data_class: INTERNAL_ONLY
    key_version: Option<u32>,             // data_class: INTERNAL_ONLY
    occurred_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudKmsError {
    InvalidTenantId,
    InvalidResourceId,
    ResourceTenantMismatch,
    ResourceRegionMismatch,
    ResourceKindMismatch,
    InvalidKeyId,
    KeyIdOriginMismatch,
    KeyIdTenantMismatch,
    KeyIdRegionMismatch,
    InvalidCellId,
    CellRegionMismatch,
    CellPlacementMismatch,
    InvalidHsmPartitionRef,
    HsmPartitionMismatch,
    HsmValidationDenied,
    ResidencyRegionMismatch,
    InvalidDataClass,
    InvalidKeyState,
    InvalidKeyUsage,
    InvalidRotationPeriod,
    InvalidEventId,
    InvalidMaterialRef,
    InvalidCiphertextRef,
    InvalidActorRef,
    InvalidAadFingerprint,
    InvalidTimeOrder,
    DestructionSlaExceeded,
    InvalidDestructionProofRef,
    ProviderMismatch,
    InvalidEvidenceRef,
    InvalidEvidenceSchemaVersion,
    DuplicateKey,
    UnknownKey,
    DuplicateUseEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KmsKeyIdParts {
    origin: KmsKeyOrigin, // data_class: PUBLIC
    region: RegionCode,   // data_class: PUBLIC
    tenant_id: String,    // data_class: INTERNAL_ONLY
    name: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudKmsDirectory {
    keys: BTreeMap<KmsKeyId, KmsKey>,
    key_version_lifecycle: BTreeMap<(KmsKeyId, u32), KmsKeyVersionLifecycle>,
    sealing_roots: BTreeMap<SealingRootRef, KmsSealingRoot>,
    receipts: BTreeMap<KmsUseEventId, KmsUseReceipt>,
    destruction_receipts: BTreeMap<KmsKeyId, KeyDestructionReceipt>,
}

pub trait KmsRepo {
    fn create_key(&mut self, input: KmsKeyCreate) -> Result<KmsKey, CloudKmsError>;

    /// Execute a KMS encrypt at the domain layer.
    ///
    /// # SECURITY — do NOT call directly from HTTP boundary code
    ///
    /// This method performs the domain-level crypto operation only.  It does NOT
    /// verify principal identity and does NOT run a PDP authorization decision.
    /// HTTP boundary code (i.e. `secrets/ports/kms-api`) MUST call
    /// [`secrets_kms_api::authorize_cloud_kms_encrypt_from_api`] instead, which
    /// enforces [`VerifiedKmsPrincipal`] + PDP [`KmsCryptoAuthorizer`] before
    /// reaching this layer.  Direct callers bypass the entire AUTH-005 / ADR-0573
    /// fail-closed seam and open a cross-tenant IDOR.
    fn authorize_encrypt(
        &mut self,
        input: KmsEncryptRequest,
    ) -> Result<KmsUseReceipt, CloudKmsError>;

    /// Execute a KMS decrypt at the domain layer.
    ///
    /// # SECURITY — do NOT call directly from HTTP boundary code
    ///
    /// Same constraint as [`authorize_encrypt`]: this method carries no identity
    /// or authorization guarantee.  HTTP boundary code MUST go through
    /// [`secrets_kms_api::authorize_cloud_kms_decrypt_from_api`] (ADR-0573).
    fn authorize_decrypt(
        &mut self,
        input: KmsDecryptRequest,
    ) -> Result<KmsUseReceipt, CloudKmsError>;
    fn rotate_key(
        &mut self,
        key_id: &KmsKeyId,
        updated_at_epoch_seconds: u64,
    ) -> Result<KmsKey, CloudKmsError>;
    fn destroy_key(
        &mut self,
        input: KeyDestructionRequest,
    ) -> Result<KeyDestructionReceipt, CloudKmsError>;
}

pub trait KmsProviderCryptoPort {
    fn provider_kind(&self) -> KmsProviderKind;

    fn encrypt(
        &self,
        input: KmsProviderEncryptRequest,
    ) -> Result<KmsProviderCryptoReceipt, KmsProviderCryptoError>;

    fn decrypt(
        &self,
        input: KmsProviderDecryptRequest,
    ) -> Result<KmsProviderCryptoReceipt, KmsProviderCryptoError>;
}

impl KmsKeyOrigin {
    pub const fn id_prefix(self) -> &'static str {
        match self {
            Self::OyatieManaged => KMS_KEY_PREFIX,
            Self::Byok => BYOK_KEY_PREFIX,
            Self::Hyok => HYOK_KEY_PREFIX,
        }
    }
}

impl KmsKeyState {
    pub const fn can_serve_crypto(self) -> bool {
        matches!(self, Self::Enabled)
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Destroyed)
    }
}

impl KmsKeyId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudKmsError> {
        let value = value.into();
        parse_kms_key_id(&value)?;
        Ok(Self { value })
    }

    pub fn origin(&self) -> Result<KmsKeyOrigin, CloudKmsError> {
        Ok(self.parts()?.origin)
    }

    pub fn region(&self) -> Result<RegionCode, CloudKmsError> {
        Ok(self.parts()?.region)
    }

    pub fn tenant_id(&self) -> Result<String, CloudKmsError> {
        Ok(self.parts()?.tenant_id)
    }

    pub fn name(&self) -> Result<String, CloudKmsError> {
        Ok(self.parts()?.name)
    }

    fn parts(&self) -> Result<KmsKeyIdParts, CloudKmsError> {
        parse_kms_key_id(&self.value)
    }
}

impl HsmPartitionRef {
    pub fn new(
        value: impl Into<String>,
        region: &RegionCode,
        cell_id: &CellId,
    ) -> Result<Self, CloudKmsError> {
        let value = value.into();
        let expected = format!("{HSM_PARTITION_PREFIX}{}/{}", region.value, cell_id.value);
        if value == expected {
            Ok(Self { value })
        } else {
            Err(CloudKmsError::HsmPartitionMismatch)
        }
    }
}

impl MaterialRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudKmsError> {
        let value = value.into();
        validate_prefixed_ref(
            &value,
            MATERIAL_REF_PREFIX,
            CloudKmsError::InvalidMaterialRef,
        )?;
        Ok(Self { value })
    }
}

impl CiphertextRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudKmsError> {
        let value = value.into();
        validate_prefixed_ref(
            &value,
            CIPHERTEXT_REF_PREFIX,
            CloudKmsError::InvalidCiphertextRef,
        )?;
        Ok(Self { value })
    }
}

impl ActorRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudKmsError> {
        let value = value.into();
        if (value.starts_with(USER_PRINCIPAL_PREFIX) && value.len() > USER_PRINCIPAL_PREFIX.len())
            || (value.starts_with(SERVICE_PRINCIPAL_PREFIX)
                && value.len() > SERVICE_PRINCIPAL_PREFIX.len())
        {
            Ok(Self { value })
        } else {
            Err(CloudKmsError::InvalidActorRef)
        }
    }
}

impl KmsUseEventId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudKmsError> {
        let value = value.into();
        if value.starts_with(EVENT_ID_PREFIX) && value.len() > EVENT_ID_PREFIX.len() {
            Ok(Self { value })
        } else {
            Err(CloudKmsError::InvalidEventId)
        }
    }
}

impl DestructionProofRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudKmsError> {
        let value = value.into();
        if value.starts_with(DESTRUCTION_PROOF_PREFIX)
            && value.len() > DESTRUCTION_PROOF_PREFIX.len()
        {
            Ok(Self { value })
        } else {
            Err(CloudKmsError::InvalidDestructionProofRef)
        }
    }
}

impl SealingRootRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudKmsError> {
        let value = value.into();
        validate_prefixed_ref(
            &value,
            SEALING_ROOT_PREFIX,
            CloudKmsError::InvalidEvidenceRef,
        )?;
        Ok(Self { value })
    }
}

impl KmsKeyVersionLifecycle {
    fn active(key: &KmsKey) -> Self {
        Self {
            key_id: internal(key.key_id.value.clone()),
            tenant_id: internal(key.tenant_id.value.clone()),
            version: internal(key.current_version.value),
            state: public(KmsKeyVersionLifecycleState::Active),
            reason: internal(None),
            created_at_epoch_seconds: internal(key.updated_at_epoch_seconds.value),
            activated_at_epoch_seconds: internal(key.updated_at_epoch_seconds.value),
            decrypt_only_since_epoch_seconds: internal(None),
            quarantined_since_epoch_seconds: internal(None),
            schema_version: public(KMS_SCHEMA_VERSION),
        }
    }

    fn decrypt_only(
        &self,
        reason: impl Into<String>,
        effective_at_epoch_seconds: u64,
    ) -> Result<Self, CloudKmsError> {
        validate_time_order(
            self.activated_at_epoch_seconds.value,
            effective_at_epoch_seconds,
        )?;
        let mut updated = self.clone();
        updated.state = public(KmsKeyVersionLifecycleState::DecryptOnly);
        updated.reason = internal(Some(reason.into()));
        updated.decrypt_only_since_epoch_seconds = internal(Some(effective_at_epoch_seconds));
        Ok(updated)
    }

    fn quarantined(
        &self,
        reason: impl Into<String>,
        effective_at_epoch_seconds: u64,
    ) -> Result<Self, CloudKmsError> {
        validate_time_order(
            self.activated_at_epoch_seconds.value,
            effective_at_epoch_seconds,
        )?;
        let mut updated = self.clone();
        updated.state = public(KmsKeyVersionLifecycleState::Quarantined);
        updated.reason = internal(Some(reason.into()));
        updated.quarantined_since_epoch_seconds = internal(Some(effective_at_epoch_seconds));
        Ok(updated)
    }
}

impl KmsSealingRoot {
    pub fn new(input: KmsSealingRootCreate) -> Result<Self, CloudKmsError> {
        validate_tenant_id(&input.tenant_id)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudKmsError::InvalidResourceId)?;
        let cell_id = CellId::new(input.cell_id).map_err(|_| CloudKmsError::InvalidCellId)?;
        validate_cell_region(&cell_id, &region)?;
        if input.active_version == 0 || input.rotate_after_seconds == 0 {
            return Err(CloudKmsError::InvalidKeyState);
        }
        Ok(Self {
            root_ref: internal(SealingRootRef::new(input.root_ref)?),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            cell_id: public(cell_id),
            active_version: internal(input.active_version),
            rotate_after_seconds: internal(input.rotate_after_seconds),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(KMS_SCHEMA_VERSION),
        })
    }
}

impl KmsKey {
    pub fn new(input: KmsKeyCreate) -> Result<Self, CloudKmsError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state.is_terminal() || matches!(input.state, KmsKeyState::PendingDeletion) {
            return Err(CloudKmsError::InvalidKeyState);
        }
        validate_rotation_period(input.rotation_period_days)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudKmsError::InvalidResourceId)?;
        let cell_id = CellId::new(input.cell_id).map_err(|_| CloudKmsError::InvalidCellId)?;
        validate_cell_region(&cell_id, &region)?;
        validate_hsm_for_residency(input.hsm_validation, &input.residency)?;
        if !residency_class_allows_home_region_label(&input.residency, &region.value) {
            return Err(CloudKmsError::ResidencyRegionMismatch);
        }
        let resource_id = resource_id_for(&input.resource_id, &input.tenant_id, &region)?;
        let key_id = KmsKeyId::new(input.key_id)?;
        validate_key_id_matches(&key_id, input.origin, &input.tenant_id, &region)?;
        let hsm_partition_ref = HsmPartitionRef::new(input.hsm_partition_ref, &region, &cell_id)?;
        Ok(Self {
            resource_id: internal(resource_id),
            key_id: internal(key_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            cell_id: public(cell_id),
            hsm_partition_ref: internal(hsm_partition_ref),
            origin: public(input.origin),
            usage: public(input.usage),
            hsm_validation: public(input.hsm_validation),
            residency: internal(input.residency),
            data_class: internal(privacy_class(input.data_class)?),
            state: public(input.state),
            current_version: internal(1),
            rotation_period_days: internal(input.rotation_period_days),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(KMS_SCHEMA_VERSION),
        })
    }

    pub fn rotate(&self, updated_at_epoch_seconds: u64) -> Result<Self, CloudKmsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !self.state.value.can_serve_crypto() {
            return Err(CloudKmsError::InvalidKeyState);
        }
        if matches!(self.origin.value, KmsKeyOrigin::Hyok) {
            return Err(CloudKmsError::InvalidKeyState);
        }
        let mut updated = self.clone();
        updated.current_version = internal(self.current_version.value + 1);
        updated.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(updated)
    }

    pub fn rotate_with_receipt(
        &self,
        input: KeyRotationRequest,
    ) -> Result<(Self, KeyRotationReceipt), CloudKmsError> {
        let key_id = KmsKeyId::new(input.key_id)?;
        if key_id != self.key_id.value {
            return Err(CloudKmsError::UnknownKey);
        }
        if input.tenant_id != self.tenant_id.value {
            return Err(CloudKmsError::ResourceTenantMismatch);
        }
        let actor = ActorRef::new(input.actor)?;
        validate_evidence_ref(&input.rotation_evidence_ref)?;
        validate_time_order(
            input.requested_at_epoch_seconds,
            input.completed_at_epoch_seconds,
        )?;
        let rotated = self.rotate(input.completed_at_epoch_seconds)?;
        let receipt = KeyRotationReceipt {
            key_id: internal(key_id),
            tenant_id: internal(input.tenant_id),
            actor: internal(actor),
            previous_key_version: internal(self.current_version.value),
            new_key_version: internal(rotated.current_version.value),
            rotation_evidence_ref: internal(input.rotation_evidence_ref),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            completed_at_epoch_seconds: internal(input.completed_at_epoch_seconds),
            schema_version: public(KMS_SCHEMA_VERSION),
        };
        Ok((rotated, receipt))
    }

    pub fn destroy(
        &self,
        input: KeyDestructionRequest,
    ) -> Result<(Self, KeyDestructionReceipt), CloudKmsError> {
        let key_id = KmsKeyId::new(input.key_id)?;
        if key_id != self.key_id.value {
            return Err(CloudKmsError::UnknownKey);
        }
        if input.tenant_id != self.tenant_id.value {
            return Err(CloudKmsError::ResourceTenantMismatch);
        }
        validate_time_order(
            input.requested_at_epoch_seconds,
            input.completed_at_epoch_seconds,
        )?;
        if input.completed_at_epoch_seconds - input.requested_at_epoch_seconds
            > MAX_DESTRUCTION_SLA_SECONDS
        {
            return Err(CloudKmsError::DestructionSlaExceeded);
        }
        let receipt = KeyDestructionReceipt {
            key_id: internal(key_id),
            tenant_id: internal(input.tenant_id),
            proof_ref: internal(DestructionProofRef::new(input.proof_ref)?),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            completed_at_epoch_seconds: internal(input.completed_at_epoch_seconds),
            schema_version: public(KMS_SCHEMA_VERSION),
        };
        let mut destroyed = self.clone();
        destroyed.state = public(KmsKeyState::Destroyed);
        destroyed.updated_at_epoch_seconds = internal(input.completed_at_epoch_seconds);
        Ok((destroyed, receipt))
    }
}

impl KmsProviderEncryptRequest {
    pub fn validate(&self) -> Result<(), KmsProviderCryptoError> {
        validate_provider_ref(
            &self.request_id,
            KmsProviderCryptoError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &self.provider_key_ref,
            KmsProviderCryptoError::InvalidProviderKeyRef,
        )?;
        KmsKeyId::new(self.key_id.clone()).map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        validate_tenant_id(&self.tenant_id).map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        MaterialRef::new(self.plaintext_ref.clone())
            .map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        CiphertextRef::new(self.ciphertext_ref.clone())
            .map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        privacy_class(self.data_class).map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        ActorRef::new(self.actor.clone()).map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        validate_aad_fingerprint(&self.aad_fingerprint)
            .map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        Ok(())
    }
}

impl KmsProviderDecryptRequest {
    pub fn validate(&self) -> Result<(), KmsProviderCryptoError> {
        validate_provider_ref(
            &self.request_id,
            KmsProviderCryptoError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &self.provider_key_ref,
            KmsProviderCryptoError::InvalidProviderKeyRef,
        )?;
        KmsKeyId::new(self.key_id.clone()).map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        validate_tenant_id(&self.tenant_id).map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        CiphertextRef::new(self.ciphertext_ref.clone())
            .map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        privacy_class(self.data_class).map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        ActorRef::new(self.actor.clone()).map_err(KmsProviderCryptoError::InvalidRequestShape)?;
        Ok(())
    }
}

impl KmsProviderCryptoReceipt {
    pub fn encrypt(
        provider: KmsProviderKind,
        input: KmsProviderEncryptRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, KmsProviderCryptoError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_provider_ref(
            &provider_request_id,
            KmsProviderCryptoError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &provider_evidence_ref,
            KmsProviderCryptoError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: KmsOperation::Encrypt,
            request_id: input.request_id,
            provider_request_id,
            provider_key_ref: input.provider_key_ref,
            key_id: input.key_id,
            tenant_id: input.tenant_id,
            material_ref: Some(input.plaintext_ref),
            ciphertext_ref: input.ciphertext_ref,
            data_class: input.data_class,
            purpose: input.purpose,
            actor: input.actor,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: KMS_SCHEMA_VERSION,
        })
    }

    pub fn decrypt(
        provider: KmsProviderKind,
        input: KmsProviderDecryptRequest,
        provider_request_id: impl Into<String>,
        provider_evidence_ref: impl Into<String>,
    ) -> Result<Self, KmsProviderCryptoError> {
        input.validate()?;
        let provider_request_id = provider_request_id.into();
        let provider_evidence_ref = provider_evidence_ref.into();
        validate_provider_ref(
            &provider_request_id,
            KmsProviderCryptoError::InvalidProviderRequestId,
        )?;
        validate_provider_ref(
            &provider_evidence_ref,
            KmsProviderCryptoError::InvalidProviderEvidenceRef,
        )?;
        Ok(Self {
            provider,
            operation: KmsOperation::Decrypt,
            request_id: input.request_id,
            provider_request_id,
            provider_key_ref: input.provider_key_ref,
            key_id: input.key_id,
            tenant_id: input.tenant_id,
            material_ref: None,
            ciphertext_ref: input.ciphertext_ref,
            data_class: input.data_class,
            purpose: input.purpose,
            actor: input.actor,
            provider_evidence_ref,
            occurred_at_epoch_seconds: input.requested_at_epoch_seconds,
            schema_version: KMS_SCHEMA_VERSION,
        })
    }
}

impl KmsUseReceipt {
    pub fn encrypt(key: &KmsKey, input: KmsEncryptRequest) -> Result<Self, CloudKmsError> {
        validate_use_key(
            key,
            &input.key_id,
            &input.tenant_id,
            &input.region,
            &input.cell_id,
            input.data_class,
        )?;
        validate_aad_fingerprint(&input.aad_fingerprint)?;
        Ok(Self {
            event_id: internal(KmsUseEventId::new(input.event_id)?),
            key_id: internal(KmsKeyId::new(input.key_id)?),
            tenant_id: internal(input.tenant_id),
            operation: public(KmsOperation::Encrypt),
            material_ref: internal(Some(MaterialRef::new(input.plaintext_ref)?)),
            ciphertext_ref: internal(CiphertextRef::new(input.ciphertext_ref)?),
            data_class: internal(privacy_class(input.data_class)?),
            purpose: internal(input.purpose),
            actor: internal(ActorRef::new(input.actor)?),
            key_version: internal(key.current_version.value),
            hsm_partition_ref: internal(key.hsm_partition_ref.value.clone()),
            occurred_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            schema_version: public(KMS_SCHEMA_VERSION),
        })
    }

    pub fn decrypt(key: &KmsKey, input: KmsDecryptRequest) -> Result<Self, CloudKmsError> {
        validate_use_key(
            key,
            &input.key_id,
            &input.tenant_id,
            &input.region,
            &input.cell_id,
            input.data_class,
        )?;
        Ok(Self {
            event_id: internal(KmsUseEventId::new(input.event_id)?),
            key_id: internal(KmsKeyId::new(input.key_id)?),
            tenant_id: internal(input.tenant_id),
            operation: public(KmsOperation::Decrypt),
            material_ref: internal(None),
            ciphertext_ref: internal(CiphertextRef::new(input.ciphertext_ref)?),
            data_class: internal(privacy_class(input.data_class)?),
            purpose: internal(input.purpose),
            actor: internal(ActorRef::new(input.actor)?),
            key_version: internal(key.current_version.value),
            hsm_partition_ref: internal(key.hsm_partition_ref.value.clone()),
            occurred_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            schema_version: public(KMS_SCHEMA_VERSION),
        })
    }
}

impl CloudKmsEvidenceEvent {
    pub fn from_use_receipt(
        expected_tenant_id: &str,
        receipt: KmsUseReceipt,
        evidence_ref: impl Into<String>,
    ) -> Result<Self, CloudKmsError> {
        validate_expected_tenant(expected_tenant_id)?;
        validate_evidence_schema_version(receipt.schema_version.value)?;
        let tenant_id = receipt.tenant_id.value;
        if tenant_id != expected_tenant_id {
            return Err(CloudKmsError::ResourceTenantMismatch);
        }
        let evidence_ref = evidence_ref.into();
        validate_evidence_ref(&evidence_ref)?;
        let operation = match receipt.operation.value {
            KmsOperation::Encrypt => CloudKmsEvidenceOperation::Encrypt,
            KmsOperation::Decrypt => CloudKmsEvidenceOperation::Decrypt,
        };
        let source_id = receipt.event_id.value.value;
        let event_id = evidence_event_id(&tenant_id, operation, &source_id);
        Ok(Self {
            event_id,
            tenant_id,
            key_id: receipt.key_id.value.value,
            actor: receipt.actor.value.value,
            operation,
            status: CloudKmsEvidenceStatus::Succeeded,
            evidence_ref,
            provider: None,
            provider_request_id: None,
            key_version: Some(receipt.key_version.value),
            occurred_at_epoch_seconds: receipt.occurred_at_epoch_seconds.value,
            schema_version: KMS_SCHEMA_VERSION,
        })
    }

    pub fn from_provider_crypto_receipt(
        expected_tenant_id: &str,
        expected_provider: KmsProviderKind,
        receipt: KmsProviderCryptoReceipt,
    ) -> Result<Self, CloudKmsError> {
        validate_expected_tenant(expected_tenant_id)?;
        validate_evidence_schema_version(receipt.schema_version)?;
        if receipt.tenant_id != expected_tenant_id {
            return Err(CloudKmsError::ResourceTenantMismatch);
        }
        if receipt.provider != expected_provider {
            return Err(CloudKmsError::ProviderMismatch);
        }
        validate_evidence_ref(&receipt.provider_evidence_ref)?;
        let operation = match receipt.operation {
            KmsOperation::Encrypt => CloudKmsEvidenceOperation::ProviderEncrypt,
            KmsOperation::Decrypt => CloudKmsEvidenceOperation::ProviderDecrypt,
        };
        let event_id =
            evidence_event_id(&receipt.tenant_id, operation, &receipt.provider_request_id);
        Ok(Self {
            event_id,
            tenant_id: receipt.tenant_id,
            key_id: receipt.key_id,
            actor: receipt.actor,
            operation,
            status: CloudKmsEvidenceStatus::Succeeded,
            evidence_ref: receipt.provider_evidence_ref,
            provider: Some(receipt.provider),
            provider_request_id: Some(receipt.provider_request_id),
            key_version: None,
            occurred_at_epoch_seconds: receipt.occurred_at_epoch_seconds,
            schema_version: KMS_SCHEMA_VERSION,
        })
    }

    pub fn from_key_rotation_receipt(
        expected_tenant_id: &str,
        receipt: KeyRotationReceipt,
    ) -> Result<Self, CloudKmsError> {
        validate_expected_tenant(expected_tenant_id)?;
        validate_evidence_schema_version(receipt.schema_version.value)?;
        let tenant_id = receipt.tenant_id.value;
        if tenant_id != expected_tenant_id {
            return Err(CloudKmsError::ResourceTenantMismatch);
        }
        let evidence_ref = receipt.rotation_evidence_ref.value;
        validate_evidence_ref(&evidence_ref)?;
        let operation = CloudKmsEvidenceOperation::Rotate;
        let event_id = evidence_event_id(&tenant_id, operation, &evidence_ref);
        Ok(Self {
            event_id,
            tenant_id,
            key_id: receipt.key_id.value.value,
            actor: receipt.actor.value.value,
            operation,
            status: CloudKmsEvidenceStatus::Succeeded,
            evidence_ref,
            provider: None,
            provider_request_id: None,
            key_version: Some(receipt.new_key_version.value),
            occurred_at_epoch_seconds: receipt.completed_at_epoch_seconds.value,
            schema_version: KMS_SCHEMA_VERSION,
        })
    }

    pub fn from_key_destruction_receipt(
        expected_tenant_id: &str,
        receipt: KeyDestructionReceipt,
    ) -> Result<Self, CloudKmsError> {
        validate_expected_tenant(expected_tenant_id)?;
        validate_evidence_schema_version(receipt.schema_version.value)?;
        let tenant_id = receipt.tenant_id.value;
        if tenant_id != expected_tenant_id {
            return Err(CloudKmsError::ResourceTenantMismatch);
        }
        let evidence_ref = receipt.proof_ref.value.value;
        validate_evidence_ref(&evidence_ref)?;
        let operation = CloudKmsEvidenceOperation::Destroy;
        let event_id = evidence_event_id(&tenant_id, operation, &evidence_ref);
        Ok(Self {
            event_id,
            tenant_id,
            key_id: receipt.key_id.value.value,
            actor: KMS_SYSTEM_ACTOR.to_string(),
            operation,
            status: CloudKmsEvidenceStatus::Succeeded,
            evidence_ref,
            provider: None,
            provider_request_id: None,
            key_version: None,
            occurred_at_epoch_seconds: receipt.completed_at_epoch_seconds.value,
            schema_version: KMS_SCHEMA_VERSION,
        })
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn actor(&self) -> &str {
        &self.actor
    }

    pub const fn operation(&self) -> CloudKmsEvidenceOperation {
        self.operation
    }

    pub const fn status(&self) -> CloudKmsEvidenceStatus {
        self.status
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
    }

    pub const fn provider(&self) -> Option<KmsProviderKind> {
        self.provider
    }

    pub fn provider_request_id(&self) -> Option<&str> {
        self.provider_request_id.as_deref()
    }

    pub const fn key_version(&self) -> Option<u32> {
        self.key_version
    }

    pub const fn occurred_at_epoch_seconds(&self) -> u64 {
        self.occurred_at_epoch_seconds
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn receipt(&self) -> CloudKmsEvidenceReceipt {
        CloudKmsEvidenceReceipt {
            event_id: self.event_id.clone(),
            tenant_id: self.tenant_id.clone(),
            key_id: self.key_id.clone(),
            operation: self.operation,
            status: self.status,
            evidence_ref: self.evidence_ref.clone(),
            provider: self.provider,
            provider_request_id: self.provider_request_id.clone(),
            key_version: self.key_version,
            occurred_at_epoch_seconds: self.occurred_at_epoch_seconds,
            schema_version: self.schema_version,
        }
    }
}

impl CloudKmsEvidenceReceipt {
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub const fn operation(&self) -> CloudKmsEvidenceOperation {
        self.operation
    }

    pub const fn status(&self) -> CloudKmsEvidenceStatus {
        self.status
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
    }

    pub const fn provider(&self) -> Option<KmsProviderKind> {
        self.provider
    }

    pub fn provider_request_id(&self) -> Option<&str> {
        self.provider_request_id.as_deref()
    }

    pub const fn key_version(&self) -> Option<u32> {
        self.key_version
    }

    pub const fn occurred_at_epoch_seconds(&self) -> u64 {
        self.occurred_at_epoch_seconds
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

impl KmsRepo for CloudKmsDirectory {
    fn create_key(&mut self, input: KmsKeyCreate) -> Result<KmsKey, CloudKmsError> {
        let key = KmsKey::new(input)?;
        if self.keys.contains_key(&key.key_id.value) {
            return Err(CloudKmsError::DuplicateKey);
        }
        self.keys.insert(key.key_id.value.clone(), key.clone());
        self.key_version_lifecycle.insert(
            (key.key_id.value.clone(), key.current_version.value),
            KmsKeyVersionLifecycle::active(&key),
        );
        Ok(key)
    }

    fn authorize_encrypt(
        &mut self,
        input: KmsEncryptRequest,
    ) -> Result<KmsUseReceipt, CloudKmsError> {
        let key_id = KmsKeyId::new(input.key_id.clone())?;
        let key = self.keys.get(&key_id).ok_or(CloudKmsError::UnknownKey)?;
        let receipt = KmsUseReceipt::encrypt(key, input)?;
        self.insert_receipt(receipt)
    }

    fn authorize_decrypt(
        &mut self,
        input: KmsDecryptRequest,
    ) -> Result<KmsUseReceipt, CloudKmsError> {
        let key_id = KmsKeyId::new(input.key_id.clone())?;
        let key = self.keys.get(&key_id).ok_or(CloudKmsError::UnknownKey)?;
        let receipt = KmsUseReceipt::decrypt(key, input)?;
        self.insert_receipt(receipt)
    }

    fn rotate_key(
        &mut self,
        key_id: &KmsKeyId,
        updated_at_epoch_seconds: u64,
    ) -> Result<KmsKey, CloudKmsError> {
        let current = self
            .keys
            .get(key_id)
            .cloned()
            .ok_or(CloudKmsError::UnknownKey)?;
        let previous_version = current.current_version.value;
        let updated = current.rotate(updated_at_epoch_seconds)?;
        self.keys.insert(key_id.clone(), updated.clone());
        if let Some(previous) = self
            .key_version_lifecycle
            .get(&(key_id.clone(), previous_version))
        {
            let demoted = previous.decrypt_only(
                format!("rotated to key version {}", updated.current_version.value),
                updated_at_epoch_seconds,
            )?;
            self.key_version_lifecycle
                .insert((key_id.clone(), previous_version), demoted);
        }
        self.key_version_lifecycle.insert(
            (key_id.clone(), updated.current_version.value),
            KmsKeyVersionLifecycle::active(&updated),
        );
        Ok(updated)
    }

    fn destroy_key(
        &mut self,
        input: KeyDestructionRequest,
    ) -> Result<KeyDestructionReceipt, CloudKmsError> {
        let key_id = KmsKeyId::new(input.key_id.clone())?;
        let current = self.keys.get(&key_id).ok_or(CloudKmsError::UnknownKey)?;
        let (destroyed, receipt) = current.destroy(input)?;
        self.keys.insert(key_id.clone(), destroyed);
        self.destruction_receipts.insert(key_id, receipt.clone());
        Ok(receipt)
    }
}

impl CloudKmsDirectory {
    pub fn keys(&self) -> impl Iterator<Item = &KmsKey> {
        self.keys.values()
    }

    pub fn key_version_lifecycle(&self) -> impl Iterator<Item = &KmsKeyVersionLifecycle> {
        self.key_version_lifecycle.values()
    }

    pub fn sealing_roots(&self) -> impl Iterator<Item = &KmsSealingRoot> {
        self.sealing_roots.values()
    }

    pub fn receipts(&self) -> impl Iterator<Item = &KmsUseReceipt> {
        self.receipts.values()
    }

    pub fn destruction_receipts(&self) -> impl Iterator<Item = &KeyDestructionReceipt> {
        self.destruction_receipts.values()
    }

    fn insert_receipt(&mut self, receipt: KmsUseReceipt) -> Result<KmsUseReceipt, CloudKmsError> {
        if self.receipts.contains_key(&receipt.event_id.value) {
            return Err(CloudKmsError::DuplicateUseEvent);
        }
        self.receipts
            .insert(receipt.event_id.value.clone(), receipt.clone());
        Ok(receipt)
    }

    pub fn create_sealing_root(
        &mut self,
        input: KmsSealingRootCreate,
    ) -> Result<KmsSealingRoot, CloudKmsError> {
        let sealing_root = KmsSealingRoot::new(input)?;
        if self
            .sealing_roots
            .contains_key(&sealing_root.root_ref.value)
        {
            return Err(CloudKmsError::DuplicateKey);
        }
        self.sealing_roots
            .insert(sealing_root.root_ref.value.clone(), sealing_root.clone());
        Ok(sealing_root)
    }

    pub fn demote_key_version(
        &mut self,
        input: KeyVersionDemotionRequest,
    ) -> Result<KmsKeyVersionLifecycle, CloudKmsError> {
        validate_tenant_id(&input.tenant_id)?;
        let key_id = KmsKeyId::new(input.key_id)?;
        self.validate_key_tenant(&key_id, &input.tenant_id)?;
        let lifecycle_key = (key_id, input.version);
        let current = self
            .key_version_lifecycle
            .get(&lifecycle_key)
            .ok_or(CloudKmsError::UnknownKey)?;
        let demoted = current.decrypt_only(input.reason, input.effective_at_epoch_seconds)?;
        self.key_version_lifecycle
            .insert(lifecycle_key, demoted.clone());
        Ok(demoted)
    }

    pub fn quarantine_key_ring(
        &mut self,
        input: KeyRingQuarantineRequest,
    ) -> Result<Vec<KmsKeyVersionLifecycle>, CloudKmsError> {
        validate_tenant_id(&input.tenant_id)?;
        let key_id = KmsKeyId::new(input.key_id)?;
        self.validate_key_tenant(&key_id, &input.tenant_id)?;
        if let Some(current) = self.keys.get(&key_id).cloned() {
            let mut disabled = current;
            disabled.state = public(KmsKeyState::Disabled);
            disabled.updated_at_epoch_seconds = internal(input.effective_at_epoch_seconds);
            self.keys.insert(key_id.clone(), disabled);
        }
        let version_keys = self
            .key_version_lifecycle
            .keys()
            .filter(|(candidate, _)| candidate == &key_id)
            .cloned()
            .collect::<Vec<_>>();
        if version_keys.is_empty() {
            return Err(CloudKmsError::UnknownKey);
        }
        let mut quarantined_versions = Vec::with_capacity(version_keys.len());
        for version_key in version_keys {
            let current = self
                .key_version_lifecycle
                .get(&version_key)
                .ok_or(CloudKmsError::UnknownKey)?;
            let quarantined =
                current.quarantined(input.reason.clone(), input.effective_at_epoch_seconds)?;
            self.key_version_lifecycle
                .insert(version_key, quarantined.clone());
            quarantined_versions.push(quarantined);
        }
        Ok(quarantined_versions)
    }

    fn validate_key_tenant(&self, key_id: &KmsKeyId, tenant_id: &str) -> Result<(), CloudKmsError> {
        let key = self.keys.get(key_id).ok_or(CloudKmsError::UnknownKey)?;
        if key.tenant_id.value == tenant_id {
            Ok(())
        } else {
            Err(CloudKmsError::ResourceTenantMismatch)
        }
    }
}

fn parse_kms_key_id(value: &str) -> Result<KmsKeyIdParts, CloudKmsError> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() != 4 || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(CloudKmsError::InvalidKeyId);
    }
    let origin = match parts[0] {
        KMS_KEY_PREFIX => KmsKeyOrigin::OyatieManaged,
        BYOK_KEY_PREFIX => KmsKeyOrigin::Byok,
        HYOK_KEY_PREFIX => KmsKeyOrigin::Hyok,
        _ => return Err(CloudKmsError::InvalidKeyId),
    };
    let region = RegionCode::new(parts[1]).map_err(|_| CloudKmsError::InvalidKeyId)?;
    validate_tenant_id(parts[2]).map_err(|_| CloudKmsError::InvalidKeyId)?;
    validate_canonical_segment(parts[3], CloudKmsError::InvalidKeyId)?;
    Ok(KmsKeyIdParts {
        origin,
        region,
        tenant_id: parts[2].to_string(),
        name: parts[3].to_string(),
    })
}

fn validate_key_id_matches(
    key_id: &KmsKeyId,
    origin: KmsKeyOrigin,
    tenant_id: &str,
    region: &RegionCode,
) -> Result<(), CloudKmsError> {
    let parts = key_id.parts()?;
    if parts.origin != origin {
        return Err(CloudKmsError::KeyIdOriginMismatch);
    }
    if parts.tenant_id != tenant_id {
        return Err(CloudKmsError::KeyIdTenantMismatch);
    }
    if parts.region != *region {
        return Err(CloudKmsError::KeyIdRegionMismatch);
    }
    Ok(())
}

fn resource_id_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
) -> Result<ResourceId, CloudKmsError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudKmsError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudKmsError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != ResourceKind::KmsKey.type_label() {
        return Err(CloudKmsError::ResourceKindMismatch);
    }
    Ok(id)
}

fn validate_use_key(
    key: &KmsKey,
    key_id: &str,
    tenant_id: &str,
    region: &str,
    cell_id: &str,
    data_class: DataClass,
) -> Result<(), CloudKmsError> {
    let key_id = KmsKeyId::new(key_id.to_string())?;
    if key_id != key.key_id.value {
        return Err(CloudKmsError::UnknownKey);
    }
    if tenant_id != key.tenant_id.value {
        return Err(CloudKmsError::ResourceTenantMismatch);
    }
    let region =
        RegionCode::new(region.to_string()).map_err(|_| CloudKmsError::InvalidResourceId)?;
    if region != key.region.value {
        return Err(CloudKmsError::ResourceRegionMismatch);
    }
    let cell_id = CellId::new(cell_id.to_string()).map_err(|_| CloudKmsError::InvalidCellId)?;
    validate_cell_region(&cell_id, &region)?;
    if cell_id != key.cell_id.value {
        return Err(CloudKmsError::CellPlacementMismatch);
    }
    if !key.state.value.can_serve_crypto() {
        return Err(CloudKmsError::InvalidKeyState);
    }
    if key.usage.value != KmsKeyUsage::EncryptDecrypt {
        return Err(CloudKmsError::InvalidKeyUsage);
    }
    let request_data_class = privacy_class(data_class)?;
    if request_data_class != key.data_class.value {
        return Err(CloudKmsError::InvalidDataClass);
    }
    Ok(())
}

fn validate_hsm_for_residency(
    validation: HsmValidation,
    residency: &ResidencyClass,
) -> Result<(), CloudKmsError> {
    let required = if matches!(residency, ResidencyClass::Global) {
        HsmValidation::Fips1403Level3
    } else {
        HsmValidation::PackEnhancedFips1403Level3
    };
    if validation == required {
        Ok(())
    } else {
        Err(CloudKmsError::HsmValidationDenied)
    }
}

fn validate_cell_region(cell_id: &CellId, region: &RegionCode) -> Result<(), CloudKmsError> {
    let expected_prefix = format!("cell-{}-", region.value);
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudKmsError::CellRegionMismatch)
    }
}

fn validate_rotation_period(value: Option<u16>) -> Result<(), CloudKmsError> {
    match value {
        Some(days) if !(30..=730).contains(&days) => Err(CloudKmsError::InvalidRotationPeriod),
        Some(_) | None => Ok(()),
    }
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: CloudKmsError,
) -> Result<(), CloudKmsError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(())
    } else {
        Err(error)
    }
}

fn validate_aad_fingerprint(value: &str) -> Result<(), CloudKmsError> {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(CloudKmsError::InvalidAadFingerprint)
    }
}

fn validate_provider_ref(
    value: &str,
    error: KmsProviderCryptoError,
) -> Result<(), KmsProviderCryptoError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_expected_tenant(value: &str) -> Result<(), CloudKmsError> {
    validate_tenant_id(value)
}

fn validate_evidence_schema_version(value: u32) -> Result<(), CloudKmsError> {
    if value == KMS_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(CloudKmsError::InvalidEvidenceSchemaVersion)
    }
}

fn validate_evidence_ref(value: &str) -> Result<(), CloudKmsError> {
    if value.trim().is_empty()
        || value.starts_with(MATERIAL_REF_PREFIX)
        || value.starts_with(CIPHERTEXT_REF_PREFIX)
        || looks_like_serialized_token(value)
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        return Err(CloudKmsError::InvalidEvidenceRef);
    }
    Ok(())
}

fn looks_like_serialized_token(value: &str) -> bool {
    value.starts_with("eyJ") && value.matches('.').count() >= 2
}

fn evidence_event_id(
    tenant_id: &str,
    operation: CloudKmsEvidenceOperation,
    source_id: &str,
) -> String {
    format!(
        "kmsevt_{}_{}_{}",
        evidence_event_segment(tenant_id),
        operation.label(),
        evidence_event_segment(source_id)
    )
}

fn evidence_event_segment(value: &str) -> String {
    let mut segment = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() {
            segment.push(char::from(byte.to_ascii_lowercase()));
        } else if !segment.ends_with('_') {
            segment.push('_');
        }
    }
    let segment = segment.trim_matches('_').to_string();
    if segment.is_empty() {
        "unknown".to_string()
    } else {
        segment
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudKmsError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudKmsError::InvalidTenantId)
    }
}

fn validate_time_order(start: u64, end: u64) -> Result<(), CloudKmsError> {
    if end >= start {
        Ok(())
    } else {
        Err(CloudKmsError::InvalidTimeOrder)
    }
}

fn validate_canonical_segment(value: &str, error: CloudKmsError) -> Result<(), CloudKmsError> {
    if value.trim().is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(error);
    }
    Ok(())
}

fn privacy_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudKmsError> {
    PrivacyDataClass::new(data_class).map_err(|_| CloudKmsError::InvalidDataClass)
}

fn map_resource_error(error: CloudResourceError) -> CloudKmsError {
    match error {
        CloudResourceError::InvalidResourceId => CloudKmsError::InvalidResourceId,
        CloudResourceError::ResourceIdTenantMismatch => CloudKmsError::ResourceTenantMismatch,
        CloudResourceError::ResourceIdRegionMismatch => CloudKmsError::ResourceRegionMismatch,
        CloudResourceError::ResourceIdKindMismatch => CloudKmsError::ResourceKindMismatch,
        CloudResourceError::InvalidTenantId => CloudKmsError::InvalidTenantId,
        _ => CloudKmsError::InvalidResourceId,
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_residency::{
        PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    };

    const TENANT: &str = "ten_alpha";
    const GLOBAL_TENANT: &str = "ten_beta";
    const REGION: &str = "region-alpha1";
    const GLOBAL_REGION: &str = "region-beta1";
    const FORBIDDEN_REGION: &str = "region-gamma1";
    const CELL: &str = "cell-region-alpha1-a-001";
    const GLOBAL_CELL: &str = "cell-region-beta1-a-001";
    const FORBIDDEN_CELL: &str = "cell-region-gamma1-a-001";
    const HSM_PARTITION: &str = "hsm/region-alpha1/cell-region-alpha1-a-001";
    const GLOBAL_HSM_PARTITION: &str = "hsm/region-beta1/cell-region-beta1-a-001";
    const FORBIDDEN_HSM_PARTITION: &str = "hsm/region-gamma1/cell-region-gamma1-a-001";
    const RESOURCE_ID: &str = "oya:cloud:region-alpha1:ten_alpha:kms-key:object-key";
    const KEY_ID: &str = "kms/region-alpha1/ten_alpha/object-key";
    const PLAINTEXT_REF: &str = "matref/ten_alpha/object/001";
    const CIPHERTEXT_REF: &str = "ct/ten_alpha/object/001";

    fn residency_class() -> ResidencyClass {
        ResidencyClass::PerPack(Box::new(
            PerPackResidency::new(PerPackResidencyCreate {
                allowed_primary_regions: vec![REGION.to_string()],
                allowed_replica_regions: vec![GLOBAL_REGION.to_string()],
                forbidden_regions: vec![FORBIDDEN_REGION.to_string()],
                regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                    regulator_refs: vec!["regulator/cloud-kms".to_string()],
                    evidence_ref: "evidence/residency/cloud-kms".to_string(),
                })
                .expect("regulator overlay fixture is valid"),
            })
            .expect("per-pack residency fixture is valid"),
        ))
    }

    fn key_create() -> KmsKeyCreate {
        KmsKeyCreate {
            resource_id: RESOURCE_ID.to_string(),
            key_id: KEY_ID.to_string(),
            tenant_id: TENANT.to_string(),
            region: REGION.to_string(),
            cell_id: CELL.to_string(),
            hsm_partition_ref: HSM_PARTITION.to_string(),
            origin: KmsKeyOrigin::OyatieManaged,
            usage: KmsKeyUsage::EncryptDecrypt,
            hsm_validation: HsmValidation::PackEnhancedFips1403Level3,
            residency: residency_class(),
            data_class: DataClass::PiiIdentifying,
            state: KmsKeyState::Enabled,
            rotation_period_days: Some(90),
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn encrypt_request(event_id: &str) -> KmsEncryptRequest {
        KmsEncryptRequest {
            event_id: event_id.to_string(),
            key_id: KEY_ID.to_string(),
            tenant_id: TENANT.to_string(),
            region: REGION.to_string(),
            cell_id: CELL.to_string(),
            plaintext_ref: PLAINTEXT_REF.to_string(),
            ciphertext_ref: CIPHERTEXT_REF.to_string(),
            data_class: DataClass::PiiIdentifying,
            purpose: KmsPurpose::CloudObjectStorage,
            actor: "sp_storage".to_string(),
            aad_fingerprint: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            requested_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn provider_encrypt_request() -> KmsProviderEncryptRequest {
        KmsProviderEncryptRequest {
            request_id: "kmsprov_req_encrypt_001".to_string(),
            provider_key_ref: "openbao/transit/object-key".to_string(),
            key_id: "kms/alpha-region/ten_alpha/object-key".to_string(),
            tenant_id: "ten_alpha".to_string(),
            plaintext_ref: "matref/ten_alpha/object/001".to_string(),
            ciphertext_ref: "ct/ten_alpha/object/001".to_string(),
            data_class: DataClass::PiiIdentifying,
            purpose: KmsPurpose::CloudObjectStorage,
            actor: "sp_storage".to_string(),
            aad_fingerprint: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .to_string(),
            requested_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn provider_decrypt_request() -> KmsProviderDecryptRequest {
        KmsProviderDecryptRequest {
            request_id: "kmsprov_req_decrypt_001".to_string(),
            provider_key_ref: "oci/kms/object-key".to_string(),
            key_id: "kms/alpha-region/ten_alpha/object-key".to_string(),
            tenant_id: "ten_alpha".to_string(),
            ciphertext_ref: "ct/ten_alpha/object/001".to_string(),
            data_class: DataClass::PiiIdentifying,
            purpose: KmsPurpose::CloudObjectStorage,
            actor: "sp_storage".to_string(),
            requested_at_epoch_seconds: 1_700_000_020,
        }
    }

    fn per_pack_residency(allowed_primary_regions: &[&str]) -> ResidencyClass {
        let regulator_overlay = RegulatorOverlay::new(RegulatorOverlayCreate {
            regulator_refs: vec!["regulator/baseline".to_string()],
            evidence_ref: "evidence/residency/pack-alpha".to_string(),
        })
        .expect("regulator overlay fixture is valid");
        ResidencyClass::PerPack(Box::new(
            PerPackResidency::new(PerPackResidencyCreate {
                allowed_primary_regions: allowed_primary_regions
                    .iter()
                    .map(|region| (*region).to_string())
                    .collect(),
                allowed_replica_regions: vec!["beta-region".to_string()],
                forbidden_regions: vec![],
                regulator_overlay,
            })
            .expect("per-pack residency fixture is valid"),
        ))
    }

    #[test]
    fn creates_kms_key_with_resource_hsm_residency_and_validation_binding() {
        let key = KmsKey::new(key_create()).expect("key is valid");

        assert_eq!(key.resource_id.value.kind_label().unwrap(), "kms-key");
        assert_eq!(
            key.key_id.value.origin().unwrap(),
            KmsKeyOrigin::OyatieManaged
        );
        assert_eq!(key.region.value.value, REGION);
        assert_eq!(key.cell_id.value.value, CELL);
        assert_eq!(key.hsm_partition_ref.value.value, HSM_PARTITION);
        assert_eq!(key.current_version.value, 1);
        assert_eq!(key.schema_version.value, KMS_SCHEMA_VERSION);
    }

    #[test]
    fn provider_encrypt_request_validates_refs_without_plaintext_material() {
        let request = provider_encrypt_request();

        request.validate().expect("provider request is valid");
        assert_eq!(request.purpose.label(), "cloud_object_storage");
        assert_eq!(KmsOperation::Encrypt.label(), "encrypt");
        assert_eq!(KmsProviderKind::OpenBaoTransit.label(), "openbao_transit");

        let mut bad_aad = request.clone();
        bad_aad.aad_fingerprint = "not-hex".to_string();
        assert_eq!(
            bad_aad.validate(),
            Err(KmsProviderCryptoError::InvalidRequestShape(
                CloudKmsError::InvalidAadFingerprint
            ))
        );

        let mut bad_key_ref = request;
        bad_key_ref.provider_key_ref = " ".to_string();
        assert_eq!(
            bad_key_ref.validate(),
            Err(KmsProviderCryptoError::InvalidProviderKeyRef)
        );
    }

    #[test]
    fn provider_receipts_preserve_operation_and_redacted_material_refs() {
        let encrypt = KmsProviderCryptoReceipt::encrypt(
            KmsProviderKind::OpenBaoTransit,
            provider_encrypt_request(),
            "openbao-audit-001",
            "openbao-transit://kms.oyatie.com/transit/object-key/kmsprov_req_encrypt_001",
        )
        .expect("encrypt provider receipt is valid");

        assert_eq!(encrypt.provider, KmsProviderKind::OpenBaoTransit);
        assert_eq!(encrypt.operation, KmsOperation::Encrypt);
        assert_eq!(
            encrypt.material_ref.as_deref(),
            Some("matref/ten_alpha/object/001")
        );
        assert_eq!(encrypt.schema_version, KMS_SCHEMA_VERSION);

        let decrypt = KmsProviderCryptoReceipt::decrypt(
            KmsProviderKind::OciKms,
            provider_decrypt_request(),
            "oci-opc-request-001",
            "oci-kms://vaults/ocid1.vault.oc1.ap-chuncheon-1.test/keys/ocid1.key.test",
        )
        .expect("decrypt provider receipt is valid");

        assert_eq!(decrypt.provider, KmsProviderKind::OciKms);
        assert_eq!(decrypt.operation, KmsOperation::Decrypt);
        assert_eq!(decrypt.material_ref, None);
        assert_eq!(decrypt.ciphertext_ref, "ct/ten_alpha/object/001");
    }

    #[test]
    fn kms_receipts_convert_to_metadata_only_evidence_events() {
        let key = KmsKey::new(key_create()).expect("key is valid");
        let encrypt = KmsUseReceipt::encrypt(&key, encrypt_request("kmsuse_encrypt_001"))
            .expect("encrypt receipt is valid");
        let encrypt_event = CloudKmsEvidenceEvent::from_use_receipt(
            TENANT,
            encrypt,
            "audit-chain://cloud-kms/kmsuse_encrypt_001",
        )
        .expect("use receipt converts to metadata-only evidence");
        assert_eq!(encrypt_event.tenant_id(), TENANT);
        assert_eq!(encrypt_event.key_id(), KEY_ID);
        assert_eq!(encrypt_event.actor(), "sp_storage");
        assert_eq!(
            encrypt_event.operation(),
            CloudKmsEvidenceOperation::Encrypt
        );
        assert_eq!(encrypt_event.status(), CloudKmsEvidenceStatus::Succeeded);
        assert_eq!(
            encrypt_event.evidence_ref(),
            "audit-chain://cloud-kms/kmsuse_encrypt_001"
        );
        assert_eq!(encrypt_event.provider(), None);
        let encrypt_evidence_receipt = encrypt_event.receipt();
        assert_eq!(
            encrypt_evidence_receipt.event_id(),
            encrypt_event.event_id()
        );
        assert_eq!(encrypt_evidence_receipt.tenant_id(), TENANT);
        assert_eq!(
            encrypt_evidence_receipt.operation(),
            CloudKmsEvidenceOperation::Encrypt
        );

        let provider_encrypt = KmsProviderCryptoReceipt::encrypt(
            KmsProviderKind::OpenBaoTransit,
            provider_encrypt_request(),
            "openbao-audit-001",
            "openbao-transit://kms.oyatie.com/transit/object-key/kmsprov_req_encrypt_001",
        )
        .expect("provider receipt is valid");
        let provider_event = CloudKmsEvidenceEvent::from_provider_crypto_receipt(
            TENANT,
            KmsProviderKind::OpenBaoTransit,
            provider_encrypt,
        )
        .expect("provider receipt converts to metadata-only evidence");
        assert_eq!(
            provider_event.operation(),
            CloudKmsEvidenceOperation::ProviderEncrypt
        );
        assert_eq!(
            provider_event.provider(),
            Some(KmsProviderKind::OpenBaoTransit)
        );
        assert_eq!(
            provider_event.provider_request_id(),
            Some("openbao-audit-001")
        );
        assert_eq!(
            provider_event.evidence_ref(),
            "openbao-transit://kms.oyatie.com/transit/object-key/kmsprov_req_encrypt_001"
        );

        let (rotated_key, rotation_receipt) = key
            .rotate_with_receipt(KeyRotationRequest {
                key_id: KEY_ID.to_string(),
                tenant_id: TENANT.to_string(),
                actor: "sp_kms_rotator".to_string(),
                rotation_evidence_ref: "kms-rotation://ten_alpha/object-key/2".to_string(),
                requested_at_epoch_seconds: 1_700_000_050,
                completed_at_epoch_seconds: 1_700_000_060,
            })
            .expect("rotation receipt is valid");
        assert_eq!(rotated_key.current_version.value, 2);
        let rotation_event =
            CloudKmsEvidenceEvent::from_key_rotation_receipt(TENANT, rotation_receipt)
                .expect("rotation receipt converts to evidence");
        assert_eq!(
            rotation_event.operation(),
            CloudKmsEvidenceOperation::Rotate
        );
        assert_eq!(rotation_event.actor(), "sp_kms_rotator");
        assert_eq!(rotation_event.key_version(), Some(2));

        let (_, destruction_receipt) = rotated_key
            .destroy(KeyDestructionRequest {
                key_id: KEY_ID.to_string(),
                tenant_id: TENANT.to_string(),
                proof_ref: "kproof_tenant_offboard_001".to_string(),
                requested_at_epoch_seconds: 1_700_000_200,
                completed_at_epoch_seconds: 1_700_000_300,
            })
            .expect("destruction receipt is valid");
        let destruction_event =
            CloudKmsEvidenceEvent::from_key_destruction_receipt(TENANT, destruction_receipt)
                .expect("destruction receipt converts to evidence");
        assert_eq!(
            destruction_event.operation(),
            CloudKmsEvidenceOperation::Destroy
        );
        assert_eq!(
            destruction_event.evidence_ref(),
            "kproof_tenant_offboard_001"
        );
    }

    #[test]
    fn kms_evidence_events_reject_tenant_schema_evidence_ref_and_raw_material_drift() {
        let key = KmsKey::new(key_create()).expect("key is valid");
        let encrypt = KmsUseReceipt::encrypt(&key, encrypt_request("kmsuse_encrypt_001"))
            .expect("encrypt receipt is valid");

        let tenant_mismatch = CloudKmsEvidenceEvent::from_use_receipt(
            GLOBAL_TENANT,
            encrypt.clone(),
            "audit-chain://cloud-kms/kmsuse_encrypt_001",
        )
        .expect_err("evidence event is tenant-bound");
        assert_eq!(tenant_mismatch, CloudKmsError::ResourceTenantMismatch);

        let mut schema_drift = encrypt.clone();
        schema_drift.schema_version = public(KMS_SCHEMA_VERSION + 1);
        let schema_error = CloudKmsEvidenceEvent::from_use_receipt(
            TENANT,
            schema_drift,
            "audit-chain://cloud-kms/kmsuse_encrypt_001",
        )
        .expect_err("evidence event rejects schema drift");
        assert_eq!(schema_error, CloudKmsError::InvalidEvidenceSchemaVersion);

        let raw_material_ref_error =
            CloudKmsEvidenceEvent::from_use_receipt(TENANT, encrypt.clone(), PLAINTEXT_REF)
                .expect_err("evidence ref cannot be a plaintext material ref");
        assert_eq!(raw_material_ref_error, CloudKmsError::InvalidEvidenceRef);

        let token_shaped_ref_error = CloudKmsEvidenceEvent::from_use_receipt(
            TENANT,
            encrypt,
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhbGljZSJ9.signature",
        )
        .expect_err("evidence ref cannot be a token");
        assert_eq!(token_shaped_ref_error, CloudKmsError::InvalidEvidenceRef);

        let mut provider_receipt = KmsProviderCryptoReceipt::encrypt(
            KmsProviderKind::OpenBaoTransit,
            provider_encrypt_request(),
            "openbao-audit-001",
            "openbao-transit://kms.oyatie.com/transit/object-key/kmsprov_req_encrypt_001",
        )
        .expect("provider receipt is valid");
        let provider_mismatch = CloudKmsEvidenceEvent::from_provider_crypto_receipt(
            TENANT,
            KmsProviderKind::OciKms,
            provider_receipt.clone(),
        )
        .expect_err("evidence event is provider-bound");
        assert_eq!(provider_mismatch, CloudKmsError::ProviderMismatch);

        provider_receipt.provider_evidence_ref = CIPHERTEXT_REF.to_string();
        let provider_raw_ref = CloudKmsEvidenceEvent::from_provider_crypto_receipt(
            TENANT,
            KmsProviderKind::OpenBaoTransit,
            provider_receipt,
        )
        .expect_err("provider evidence cannot point at ciphertext material");
        assert_eq!(provider_raw_ref, CloudKmsError::InvalidEvidenceRef);
    }

    #[test]
    fn rejects_key_id_resource_and_hsm_partition_drift() {
        let wrong_origin = KmsKey::new(KmsKeyCreate {
            key_id: format!("byok/{REGION}/{TENANT}/object-key"),
            ..key_create()
        })
        .expect_err("key id origin must match declared origin");
        assert_eq!(wrong_origin, CloudKmsError::KeyIdOriginMismatch);

        let wrong_kind = KmsKey::new(KmsKeyCreate {
            resource_id: format!("oya:cloud:{REGION}:{TENANT}:bucket:object-key"),
            ..key_create()
        })
        .expect_err("resource id kind must be kms-key");
        assert_eq!(wrong_kind, CloudKmsError::ResourceKindMismatch);

        let wrong_partition = KmsKey::new(KmsKeyCreate {
            hsm_partition_ref: format!("hsm/{REGION}/cell-region-alpha1-b-001"),
            ..key_create()
        })
        .expect_err("HSM partition is cell-bound");
        assert_eq!(wrong_partition, CloudKmsError::HsmPartitionMismatch);
    }

    #[test]
    fn enforces_pack_certified_global_fips_and_residency_rules() {
        let hsm_error = KmsKey::new(KmsKeyCreate {
            hsm_validation: HsmValidation::Fips1403Level3,
            ..key_create()
        })
        .expect_err("pack-bound keys require pack-certified validation");
        assert_eq!(hsm_error, CloudKmsError::HsmValidationDenied);

        for validation in [
            HsmValidation::Cryptrec,
            HsmValidation::CommonCriteriaEal4,
            HsmValidation::PciHsm,
        ] {
            let profile_error = KmsKey::new(KmsKeyCreate {
                resource_id: format!(
                    "oya:cloud:{GLOBAL_REGION}:{GLOBAL_TENANT}:kms-key:object-key"
                ),
                key_id: format!("kms/{GLOBAL_REGION}/{GLOBAL_TENANT}/object-key"),
                tenant_id: GLOBAL_TENANT.to_string(),
                region: GLOBAL_REGION.to_string(),
                cell_id: GLOBAL_CELL.to_string(),
                hsm_partition_ref: GLOBAL_HSM_PARTITION.to_string(),
                hsm_validation: validation,
                residency: ResidencyClass::Global,
                ..key_create()
            })
            .expect_err("KMS keys require a FIPS 140-3 Level 3 profile");
            assert_eq!(profile_error, CloudKmsError::HsmValidationDenied);
        }

        let baseline = KmsKey::new(KmsKeyCreate {
            resource_id: format!("oya:cloud:{GLOBAL_REGION}:{GLOBAL_TENANT}:kms-key:object-key"),
            key_id: format!("kms/{GLOBAL_REGION}/{GLOBAL_TENANT}/object-key"),
            tenant_id: GLOBAL_TENANT.to_string(),
            region: GLOBAL_REGION.to_string(),
            cell_id: GLOBAL_CELL.to_string(),
            hsm_partition_ref: GLOBAL_HSM_PARTITION.to_string(),
            hsm_validation: HsmValidation::Fips1403Level3,
            residency: ResidencyClass::Global,
            ..key_create()
        })
        .expect("baseline KMS key accepts FIPS 140-3 validation");
        assert_eq!(baseline.hsm_validation.value, HsmValidation::Fips1403Level3);

        let residency_error = KmsKey::new(KmsKeyCreate {
            resource_id: format!("oya:cloud:{FORBIDDEN_REGION}:{TENANT}:kms-key:object-key"),
            key_id: format!("kms/{FORBIDDEN_REGION}/{TENANT}/object-key"),
            region: FORBIDDEN_REGION.to_string(),
            cell_id: FORBIDDEN_CELL.to_string(),
            hsm_partition_ref: FORBIDDEN_HSM_PARTITION.to_string(),
            ..key_create()
        })
        .expect_err("pack-bound key cannot be created in a forbidden region");
        assert_eq!(residency_error, CloudKmsError::ResidencyRegionMismatch);
    }

    #[test]
    fn authorizes_encrypt_and_decrypt_as_auditable_receipts() {
        let key = KmsKey::new(key_create()).expect("key is valid");
        let encrypt = KmsUseReceipt::encrypt(&key, encrypt_request("kmsuse_encrypt_001"))
            .expect("encrypt request is valid");
        assert_eq!(encrypt.operation.value, KmsOperation::Encrypt);
        assert!(encrypt.material_ref.value.is_some());
        assert_eq!(encrypt.key_version.value, 1);

        let decrypt = KmsUseReceipt::decrypt(
            &key,
            KmsDecryptRequest {
                event_id: "kmsuse_decrypt_001".to_string(),
                key_id: KEY_ID.to_string(),
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: CELL.to_string(),
                ciphertext_ref: CIPHERTEXT_REF.to_string(),
                data_class: DataClass::PiiIdentifying,
                purpose: KmsPurpose::CloudObjectStorage,
                actor: "usr_alice".to_string(),
                requested_at_epoch_seconds: 1_700_000_020,
            },
        )
        .expect("decrypt request is valid");
        assert_eq!(decrypt.operation.value, KmsOperation::Decrypt);
        assert!(decrypt.material_ref.value.is_none());
    }

    #[test]
    fn rejects_crypto_use_when_request_region_or_cell_drift_from_key_placement() {
        let key = KmsKey::new(key_create()).expect("key is valid");

        let region_error = KmsUseReceipt::encrypt(
            &key,
            KmsEncryptRequest {
                region: GLOBAL_REGION.to_string(),
                cell_id: GLOBAL_CELL.to_string(),
                ..encrypt_request("kmsuse_region_drift")
            },
        )
        .expect_err("request region must match key placement");
        assert_eq!(region_error, CloudKmsError::ResourceRegionMismatch);

        let cell_error = KmsUseReceipt::decrypt(
            &key,
            KmsDecryptRequest {
                event_id: "kmsuse_cell_drift".to_string(),
                key_id: KEY_ID.to_string(),
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: "cell-region-alpha1-b-001".to_string(),
                ciphertext_ref: CIPHERTEXT_REF.to_string(),
                data_class: DataClass::PiiIdentifying,
                purpose: KmsPurpose::CloudObjectStorage,
                actor: "usr_alice".to_string(),
                requested_at_epoch_seconds: 1_700_000_020,
            },
        )
        .expect_err("request cell must match key placement");
        assert_eq!(cell_error, CloudKmsError::CellPlacementMismatch);
    }

    #[test]
    fn rejects_crypto_use_when_state_usage_actor_or_data_class_is_invalid() {
        let disabled = KmsKey::new(KmsKeyCreate {
            state: KmsKeyState::Disabled,
            ..key_create()
        })
        .expect("disabled key can exist");
        let state_error = KmsUseReceipt::encrypt(&disabled, encrypt_request("kmsuse_disabled"))
            .expect_err("disabled key cannot serve crypto");
        assert_eq!(state_error, CloudKmsError::InvalidKeyState);

        let key = KmsKey::new(key_create()).expect("key is valid");
        let actor_error = KmsUseReceipt::encrypt(
            &key,
            KmsEncryptRequest {
                actor: "tenant-admin".to_string(),
                ..encrypt_request("kmsuse_bad_actor")
            },
        )
        .expect_err("actor must be a typed principal ref");
        assert_eq!(actor_error, CloudKmsError::InvalidActorRef);

        let data_error = KmsUseReceipt::encrypt(
            &key,
            KmsEncryptRequest {
                data_class: DataClass::Pci,
                ..encrypt_request("kmsuse_bad_class")
            },
        )
        .expect_err("request class must match key policy class");
        assert_eq!(data_error, CloudKmsError::InvalidDataClass);
    }

    #[test]
    fn rotates_enabled_managed_keys_and_rejects_hyok_rotation() {
        let key = KmsKey::new(key_create()).expect("key is valid");
        let rotated = key.rotate(1_700_000_100).expect("managed key rotates");
        assert_eq!(rotated.current_version.value, 2);

        let hyok = KmsKey::new(KmsKeyCreate {
            resource_id: format!("oya:cloud:{REGION}:{TENANT}:kms-key:tenant-held"),
            key_id: format!("hyok/{REGION}/{TENANT}/tenant-held"),
            origin: KmsKeyOrigin::Hyok,
            rotation_period_days: None,
            ..key_create()
        })
        .expect("HYOK key can exist");
        let error = hyok
            .rotate(1_700_000_100)
            .expect_err("tenant-held key rotation is not controlled by Oyatie KMS");
        assert_eq!(error, CloudKmsError::InvalidKeyState);
    }

    #[test]
    fn directory_rejects_duplicate_use_events_and_records_key_destruction() {
        let mut directory = CloudKmsDirectory::default();
        let key = directory
            .create_key(key_create())
            .expect("create key succeeds");
        let receipt = directory
            .authorize_encrypt(encrypt_request("kmsuse_encrypt_001"))
            .expect("first use event succeeds");
        assert_eq!(receipt.key_id.value, key.key_id.value);
        assert_eq!(
            directory
                .authorize_encrypt(encrypt_request("kmsuse_encrypt_001"))
                .expect_err("event ids are idempotency/audit identities"),
            CloudKmsError::DuplicateUseEvent
        );

        let destruction = directory
            .destroy_key(KeyDestructionRequest {
                key_id: KEY_ID.to_string(),
                tenant_id: TENANT.to_string(),
                proof_ref: "kproof_tenant_offboard_001".to_string(),
                requested_at_epoch_seconds: 1_700_000_200,
                completed_at_epoch_seconds: 1_700_000_300,
            })
            .expect("destruction receipt is valid");
        assert_eq!(
            destruction.proof_ref.value.value,
            "kproof_tenant_offboard_001"
        );
    }

    #[test]
    fn rejects_destruction_outside_twenty_four_hour_evidence_sla() {
        let key = KmsKey::new(key_create()).expect("key is valid");
        let error = key
            .destroy(KeyDestructionRequest {
                key_id: KEY_ID.to_string(),
                tenant_id: TENANT.to_string(),
                proof_ref: "kproof_tenant_offboard_001".to_string(),
                requested_at_epoch_seconds: 1_700_000_000,
                completed_at_epoch_seconds: 1_700_090_401,
            })
            .expect_err("destruction proof must meet 24h evidence SLA");
        assert_eq!(error, CloudKmsError::DestructionSlaExceeded);
    }

    #[test]
    fn directory_operator_lifecycle_ports_record_sealing_root_demote_and_quarantine() {
        let mut directory = CloudKmsDirectory::default();
        let sealing_root = directory
            .create_sealing_root(KmsSealingRootCreate {
                root_ref: "sealing-root/tenant-a".to_string(),
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: CELL.to_string(),
                active_version: 1,
                rotate_after_seconds: 86_400,
                created_at_epoch_seconds: 1_700_000_000,
            })
            .expect("sealing root creation should be a domain mutation");
        assert_eq!(sealing_root.root_ref.value.value, "sealing-root/tenant-a");
        assert_eq!(directory.sealing_roots().count(), 1);

        let key = directory
            .create_key(key_create())
            .expect("key create should seed lifecycle version state");
        directory
            .rotate_key(&key.key_id.value, 1_700_000_100)
            .expect("rotation should add active version two");

        let demoted = directory
            .demote_key_version(KeyVersionDemotionRequest {
                key_id: KEY_ID.to_string(),
                tenant_id: TENANT.to_string(),
                version: 1,
                reason: "newer active key version 2 is present".to_string(),
                effective_at_epoch_seconds: 1_700_000_110,
            })
            .expect("operator should demote old active versions through the domain");
        assert_eq!(
            demoted.state.value,
            KmsKeyVersionLifecycleState::DecryptOnly
        );
        assert_eq!(
            demoted.decrypt_only_since_epoch_seconds.value,
            Some(1_700_000_110)
        );

        let quarantined = directory
            .quarantine_key_ring(KeyRingQuarantineRequest {
                key_id: KEY_ID.to_string(),
                tenant_id: TENANT.to_string(),
                reason: "compromised observation".to_string(),
                effective_at_epoch_seconds: 1_700_000_120,
            })
            .expect("operator should quarantine compromised key rings through the domain");
        assert!(
            quarantined
                .iter()
                .all(|version| version.state.value == KmsKeyVersionLifecycleState::Quarantined)
        );
        let disabled = directory.keys().next().expect("key remains present");
        assert_eq!(disabled.state.value, KmsKeyState::Disabled);
    }
}
