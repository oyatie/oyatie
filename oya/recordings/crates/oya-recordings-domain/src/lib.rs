//! Workspace recordings archive kernel.
//!
//! Typed kernel records for the W-Workspace-GA Recordings adjunct surface named
//! by `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns archived
//! Meet recording metadata, transcoded variants, retention/legal-hold policy,
//! and purge eligibility without owning cold storage, transcoding workers, or
//! trust-portal UI.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod fhir_resource_type;

pub use fhir_resource_type::FhirResourceType;

use std::collections::BTreeSet;

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use oya_meet_domain::{RecordingAccessMode, RecordingRef, RecordingStatus};

const RECORDING_ARCHIVE_SCHEMA_VERSION: u32 = 1;
const RECORDING_INGEST_SCHEMA_VERSION: u32 = 1;
const RETENTION_POLICY_SCHEMA_VERSION: u32 = 1;
const MIN_ARCHIVE_BYTES: u64 = 1;
const SHA256_PREFIX: &str = "sha256:";
const SHA256_HEX_LENGTH: usize = 64;
const SPIFFE_PREFIX: &str = "spiffe://";
const AUDIT_CHAIN_PREFIX: &str = "audit-chain:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordingArchiveError {
    InvalidArchiveId,
    InvalidSessionId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidRecordingId,
    RecordingNotCompleted,
    RecordingNotTrustPortalOnly,
    EmptyRecordingBytes,
    InvalidRetentionPolicyId,
    InvalidKmsShredKeyId,
    RetentionPolicyMismatch,
    KmsShredKeyMismatch,
    InvalidLegalHoldId,
    EmptyVariantSet,
    InvalidVariantId,
    DuplicateVariantId,
    DuplicateVariantFormat,
    InvalidVariantStorageKey,
    InvalidChecksum,
    EmptyVariantBytes,
    InvalidDuration,
    InvalidTranscriptRef,
    InvalidSummaryRef,
    InvalidTimeOrder,
    InvalidDataClass,
    InvalidIdempotencyKey,
    InvalidSourceRef,
    InvalidProducerSpiffeId,
    InvalidParentAuditChainRef,
    MissingConsentBanner,
    DuplicateIngestContentHashMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecordingVariantFormat {
    Mp4Video,
    WebmVideo,
    OpusAudio,
    TranscriptJson,
    SummaryMarkdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecordingSourceKind {
    Meet,
    MessengerHuddle,
    LiveBroadcast,
    ManualUpload,
    WorkflowScreenCapture,
    LegacyWorkspaceRecording,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecordingContextKind {
    Personal,
    Professional,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingIngestRequestCreate {
    pub idempotency_key: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub source_kind: RecordingSourceKind,    // data_class: INTERNAL_ONLY
    pub source_ref: String,                  // data_class: INTERNAL_ONLY
    pub content_hash: String,                // data_class: INTERNAL_ONLY
    pub byte_len: u64,                       // data_class: INTERNAL_ONLY
    pub duration_seconds: u64,               // data_class: INTERNAL_ONLY
    pub ingested_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub ended_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub consent_banner_confirmed: bool,      // data_class: INTERNAL_ONLY
    pub context_kind: RecordingContextKind,  // data_class: INTERNAL_ONLY
    pub author_ref: Option<String>,          // data_class: INTERNAL_ONLY
    pub cell_id: String,                     // data_class: INTERNAL_ONLY
    pub producer_spiffe_id: String,          // data_class: INTERNAL_ONLY
    pub parent_audit_chain_ref: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingIngestRequest {
    pub idempotency_key: Classified<String>,
    pub tenant_id: Classified<String>,
    pub source_kind: Classified<RecordingSourceKind>,
    pub source_ref: Classified<String>,
    pub content_hash: Classified<String>,
    pub byte_len: Classified<u64>,
    pub duration_seconds: Classified<u64>,
    pub ingested_at_epoch_seconds: Classified<u64>,
    pub ended_at_epoch_seconds: Classified<Option<u64>>,
    pub consent_banner_confirmed: Classified<bool>,
    pub context_kind: Classified<RecordingContextKind>,
    pub author_ref: Classified<Option<String>>,
    pub cell_id: Classified<String>,
    pub producer_spiffe_id: Classified<String>,
    pub parent_audit_chain_ref: Classified<String>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingIngestReceipt {
    pub recording_id: Classified<String>,
    pub tenant_id: Classified<String>,
    pub idempotency_key: Classified<String>,
    pub content_hash: Classified<String>,
    pub content_hash_verified: Classified<String>,
    pub ingest_audit_chain_ref: Classified<String>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordingIngestDecision {
    AcceptNew,
    AlreadyAccepted { recording_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveRetentionPolicyCreate {
    pub retention_policy_id: String,    // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: String,       // data_class: INTERNAL_ONLY
    pub purge_after_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub legal_hold_id: Option<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveRetentionPolicy {
    pub retention_policy_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub purge_after_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub legal_hold_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingVariantCreate {
    pub variant_id: String,             // data_class: INTERNAL_ONLY
    pub format: RecordingVariantFormat, // data_class: INTERNAL_ONLY
    pub storage_key: String,            // data_class: INTERNAL_ONLY
    pub checksum: String,               // data_class: INTERNAL_ONLY
    pub byte_len: u64,                  // data_class: INTERNAL_ONLY
    pub duration_seconds: u64,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecordingVariant {
    pub variant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub format: Classified<RecordingVariantFormat>, // data_class: INTERNAL_ONLY
    pub storage_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub checksum: Classified<String>,   // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,      // data_class: INTERNAL_ONLY
    pub duration_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingArchiveEntryCreate {
    pub archive_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub region: String,                           // data_class: INTERNAL_ONLY
    pub cell_id: String,                          // data_class: INTERNAL_ONLY
    pub session_id: String,                       // data_class: INTERNAL_ONLY
    pub recording: RecordingRef,                  // data_class: PII_IDENTIFYING
    pub retention_policy: ArchiveRetentionPolicy, // data_class: INTERNAL_ONLY
    pub variants: Vec<RecordingVariant>,          // data_class: INTERNAL_ONLY
    pub transcript_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub summary_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,     // data_class: INTERNAL_ONLY
    pub archived_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingArchiveEntry {
    pub archive_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<String>,     // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub session_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub recording: Classified<RecordingRef>, // data_class: PII_IDENTIFYING
    pub retention_policy: Classified<ArchiveRetentionPolicy>, // data_class: INTERNAL_ONLY
    pub variants: Classified<Vec<RecordingVariant>>, // data_class: INTERNAL_ONLY
    pub transcript_ref: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub summary_ref: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub archived_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

pub trait RecordingArchiveReader {
    fn read_archive_entry(
        &self,
        tenant_id: &str,
        archive_id: &str,
    ) -> Result<Option<RecordingArchiveEntry>, RecordingArchiveError>;
}

impl RecordingIngestRequest {
    pub fn new(input: RecordingIngestRequestCreate) -> Result<Self, RecordingArchiveError> {
        validate_non_empty(
            &input.idempotency_key,
            RecordingArchiveError::InvalidIdempotencyKey,
        )?;
        validate_non_empty(&input.tenant_id, RecordingArchiveError::InvalidTenantId)?;
        validate_non_empty(&input.source_ref, RecordingArchiveError::InvalidSourceRef)?;
        validate_checksum(&input.content_hash)?;
        if input.byte_len < MIN_ARCHIVE_BYTES {
            return Err(RecordingArchiveError::EmptyRecordingBytes);
        }
        if input.duration_seconds == 0 {
            return Err(RecordingArchiveError::InvalidDuration);
        }
        if !input.consent_banner_confirmed {
            return Err(RecordingArchiveError::MissingConsentBanner);
        }
        validate_non_empty(&input.cell_id, RecordingArchiveError::InvalidCellId)?;
        validate_spiffe_id(
            &input.producer_spiffe_id,
            RecordingArchiveError::InvalidProducerSpiffeId,
        )?;
        validate_audit_chain_ref(
            &input.parent_audit_chain_ref,
            RecordingArchiveError::InvalidParentAuditChainRef,
        )?;
        if let Some(ended_at) = input.ended_at_epoch_seconds
            && input.ingested_at_epoch_seconds < ended_at
        {
            return Err(RecordingArchiveError::InvalidTimeOrder);
        }
        validate_optional_ref(
            input.author_ref.as_deref(),
            RecordingArchiveError::InvalidSourceRef,
        )?;

        Ok(Self {
            idempotency_key: internal(input.idempotency_key),
            tenant_id: internal(input.tenant_id),
            source_kind: internal(input.source_kind),
            source_ref: internal(input.source_ref),
            content_hash: internal(input.content_hash),
            byte_len: internal(input.byte_len),
            duration_seconds: internal(input.duration_seconds),
            ingested_at_epoch_seconds: internal(input.ingested_at_epoch_seconds),
            ended_at_epoch_seconds: internal(input.ended_at_epoch_seconds),
            consent_banner_confirmed: internal(input.consent_banner_confirmed),
            context_kind: internal(input.context_kind),
            author_ref: internal(input.author_ref),
            cell_id: internal(input.cell_id),
            producer_spiffe_id: internal(input.producer_spiffe_id),
            parent_audit_chain_ref: internal(input.parent_audit_chain_ref),
            schema_version: internal(RECORDING_INGEST_SCHEMA_VERSION),
        })
    }

    pub fn idempotency_scope(&self) -> (&str, &str) {
        (&self.tenant_id.value, &self.idempotency_key.value)
    }
}

impl RecordingIngestReceipt {
    pub fn accepted(
        recording_id: String,
        request: &RecordingIngestRequest,
        content_hash_verified: String,
        ingest_audit_chain_ref: String,
    ) -> Result<Self, RecordingArchiveError> {
        validate_non_empty(&recording_id, RecordingArchiveError::InvalidRecordingId)?;
        validate_checksum(&content_hash_verified)?;
        if content_hash_verified != request.content_hash.value {
            return Err(RecordingArchiveError::DuplicateIngestContentHashMismatch);
        }
        validate_audit_chain_ref(
            &ingest_audit_chain_ref,
            RecordingArchiveError::InvalidParentAuditChainRef,
        )?;

        Ok(Self {
            recording_id: internal(recording_id),
            tenant_id: internal(request.tenant_id.value.clone()),
            idempotency_key: internal(request.idempotency_key.value.clone()),
            content_hash: internal(request.content_hash.value.clone()),
            content_hash_verified: internal(content_hash_verified),
            ingest_audit_chain_ref: internal(ingest_audit_chain_ref),
            schema_version: internal(RECORDING_INGEST_SCHEMA_VERSION),
        })
    }
}

pub fn classify_idempotent_ingest(
    existing: Option<&RecordingIngestReceipt>,
    request: &RecordingIngestRequest,
) -> Result<RecordingIngestDecision, RecordingArchiveError> {
    let Some(existing) = existing else {
        return Ok(RecordingIngestDecision::AcceptNew);
    };
    if existing.tenant_id.value != request.tenant_id.value
        || existing.idempotency_key.value != request.idempotency_key.value
    {
        return Ok(RecordingIngestDecision::AcceptNew);
    }
    if existing.content_hash.value != request.content_hash.value {
        return Err(RecordingArchiveError::DuplicateIngestContentHashMismatch);
    }
    Ok(RecordingIngestDecision::AlreadyAccepted {
        recording_id: existing.recording_id.value.clone(),
    })
}

impl ArchiveRetentionPolicy {
    pub fn new(input: ArchiveRetentionPolicyCreate) -> Result<Self, RecordingArchiveError> {
        validate_non_empty(
            &input.retention_policy_id,
            RecordingArchiveError::InvalidRetentionPolicyId,
        )?;
        validate_non_empty(
            &input.kms_shred_key_id,
            RecordingArchiveError::InvalidKmsShredKeyId,
        )?;
        if let Some(legal_hold_id) = input.legal_hold_id.as_deref() {
            validate_non_empty(legal_hold_id, RecordingArchiveError::InvalidLegalHoldId)?;
        }
        Ok(Self {
            retention_policy_id: internal(input.retention_policy_id),
            kms_shred_key_id: internal(input.kms_shred_key_id),
            purge_after_epoch_seconds: internal(input.purge_after_epoch_seconds),
            legal_hold_id: internal(input.legal_hold_id),
            schema_version: internal(RETENTION_POLICY_SCHEMA_VERSION),
        })
    }

    pub fn can_purge_at(&self, now_epoch_seconds: u64) -> bool {
        self.legal_hold_id.value.is_none()
            && now_epoch_seconds >= self.purge_after_epoch_seconds.value
    }
}

impl RecordingVariant {
    pub fn new(input: RecordingVariantCreate) -> Result<Self, RecordingArchiveError> {
        validate_non_empty(&input.variant_id, RecordingArchiveError::InvalidVariantId)?;
        validate_storage_key(&input.storage_key)?;
        validate_checksum(&input.checksum)?;
        if input.byte_len < MIN_ARCHIVE_BYTES {
            return Err(RecordingArchiveError::EmptyVariantBytes);
        }
        if input.duration_seconds == 0 {
            return Err(RecordingArchiveError::InvalidDuration);
        }
        Ok(Self {
            variant_id: internal(input.variant_id),
            format: internal(input.format),
            storage_key: internal(input.storage_key),
            checksum: internal(input.checksum),
            byte_len: internal(input.byte_len),
            duration_seconds: internal(input.duration_seconds),
        })
    }
}

impl RecordingArchiveEntry {
    pub fn new(input: RecordingArchiveEntryCreate) -> Result<Self, RecordingArchiveError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_recording_data_class());
        validate_non_empty(&input.archive_id, RecordingArchiveError::InvalidArchiveId)?;
        validate_non_empty(&input.tenant_id, RecordingArchiveError::InvalidTenantId)?;
        validate_non_empty(&input.region, RecordingArchiveError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, RecordingArchiveError::InvalidCellId)?;
        validate_non_empty(&input.session_id, RecordingArchiveError::InvalidSessionId)?;
        validate_recording(&input.recording)?;
        validate_retention_matches_recording(&input.retention_policy, &input.recording)?;
        validate_variants(&input.variants)?;
        validate_optional_ref(
            input.transcript_ref.as_deref(),
            RecordingArchiveError::InvalidTranscriptRef,
        )?;
        validate_optional_ref(
            input.summary_ref.as_deref(),
            RecordingArchiveError::InvalidSummaryRef,
        )?;
        validate_archive_time(input.archived_at_epoch_seconds, &input.recording)?;

        Ok(Self {
            archive_id: internal(input.archive_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            session_id: internal(input.session_id),
            recording: Classified::new(input.recording, recording_archive_data_class()),
            retention_policy: internal(input.retention_policy),
            variants: internal(input.variants),
            transcript_ref: internal(input.transcript_ref),
            summary_ref: internal(input.summary_ref),
            data_class: internal(data_class),
            archived_at_epoch_seconds: internal(input.archived_at_epoch_seconds),
            schema_version: internal(RECORDING_ARCHIVE_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

pub fn default_workspace_recording_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn recording_archive_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_recording_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, RecordingArchiveError> {
    PrivacyDataClass::new(data_class).map_err(|_| RecordingArchiveError::InvalidDataClass)
}

fn validate_recording(recording: &RecordingRef) -> Result<(), RecordingArchiveError> {
    validate_non_empty(
        &recording.recording_id.value,
        RecordingArchiveError::InvalidRecordingId,
    )?;
    if recording.status.value != RecordingStatus::Completed {
        return Err(RecordingArchiveError::RecordingNotCompleted);
    }
    if recording.access_mode.value != RecordingAccessMode::TrustPortalOnly {
        return Err(RecordingArchiveError::RecordingNotTrustPortalOnly);
    }
    if recording.byte_len.value < MIN_ARCHIVE_BYTES {
        return Err(RecordingArchiveError::EmptyRecordingBytes);
    }
    Ok(())
}

fn validate_retention_matches_recording(
    policy: &ArchiveRetentionPolicy,
    recording: &RecordingRef,
) -> Result<(), RecordingArchiveError> {
    validate_non_empty(
        &policy.retention_policy_id.value,
        RecordingArchiveError::InvalidRetentionPolicyId,
    )?;
    validate_non_empty(
        &policy.kms_shred_key_id.value,
        RecordingArchiveError::InvalidKmsShredKeyId,
    )?;
    if policy.retention_policy_id.value != recording.retention_policy_id.value {
        return Err(RecordingArchiveError::RetentionPolicyMismatch);
    }
    if policy.kms_shred_key_id.value != recording.kms_shred_key_id.value {
        return Err(RecordingArchiveError::KmsShredKeyMismatch);
    }
    Ok(())
}

fn validate_variants(variants: &[RecordingVariant]) -> Result<(), RecordingArchiveError> {
    if variants.is_empty() {
        return Err(RecordingArchiveError::EmptyVariantSet);
    }
    let mut ids = BTreeSet::new();
    let mut formats = BTreeSet::new();
    for variant in variants {
        validate_non_empty(
            &variant.variant_id.value,
            RecordingArchiveError::InvalidVariantId,
        )?;
        validate_storage_key(&variant.storage_key.value)?;
        validate_checksum(&variant.checksum.value)?;
        if variant.byte_len.value < MIN_ARCHIVE_BYTES {
            return Err(RecordingArchiveError::EmptyVariantBytes);
        }
        if variant.duration_seconds.value == 0 {
            return Err(RecordingArchiveError::InvalidDuration);
        }
        if !ids.insert(variant.variant_id.value.clone()) {
            return Err(RecordingArchiveError::DuplicateVariantId);
        }
        if !formats.insert(variant.format.value) {
            return Err(RecordingArchiveError::DuplicateVariantFormat);
        }
    }
    Ok(())
}

fn validate_archive_time(
    archived_at_epoch_seconds: u64,
    recording: &RecordingRef,
) -> Result<(), RecordingArchiveError> {
    if let Some(ended_at) = recording.ended_at_epoch_seconds.value
        && archived_at_epoch_seconds < ended_at
    {
        return Err(RecordingArchiveError::InvalidTimeOrder);
    }
    Ok(())
}

fn validate_storage_key(storage_key: &str) -> Result<(), RecordingArchiveError> {
    if storage_key.trim() != storage_key
        || storage_key.is_empty()
        || storage_key.starts_with('/')
        || storage_key.contains("//")
        || storage_key.chars().any(char::is_control)
    {
        return Err(RecordingArchiveError::InvalidVariantStorageKey);
    }
    if storage_key
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(RecordingArchiveError::InvalidVariantStorageKey);
    }
    Ok(())
}

fn validate_checksum(checksum: &str) -> Result<(), RecordingArchiveError> {
    let Some(hex) = checksum.strip_prefix(SHA256_PREFIX) else {
        return Err(RecordingArchiveError::InvalidChecksum);
    };
    if checksum.trim() != checksum
        || hex.len() != SHA256_HEX_LENGTH
        || !hex.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
    {
        return Err(RecordingArchiveError::InvalidChecksum);
    }
    Ok(())
}

fn validate_spiffe_id(
    value: &str,
    error: RecordingArchiveError,
) -> Result<(), RecordingArchiveError> {
    validate_prefixed_ref(value, SPIFFE_PREFIX, error)
}

fn validate_audit_chain_ref(
    value: &str,
    error: RecordingArchiveError,
) -> Result<(), RecordingArchiveError> {
    validate_prefixed_ref(value, AUDIT_CHAIN_PREFIX, error)
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: RecordingArchiveError,
) -> Result<(), RecordingArchiveError> {
    if value.trim() != value
        || !value.starts_with(prefix)
        || value.len() == prefix.len()
        || value.chars().any(char::is_control)
    {
        return Err(error);
    }
    Ok(())
}

fn validate_optional_ref(
    value: Option<&str>,
    error: RecordingArchiveError,
) -> Result<(), RecordingArchiveError> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_non_empty(value, error)
}

fn validate_non_empty(
    value: &str,
    error: RecordingArchiveError,
) -> Result<(), RecordingArchiveError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_boundary_kernel::{DataClassification, OperationalDataClass};
    use oya_meet_domain::{RecordingRefCreate, RecordingStatus};

    fn recording(status: RecordingStatus) -> RecordingRef {
        RecordingRef::new(RecordingRefCreate {
            recording_id: "rec-1".into(),
            archive_storage_key: "tenant-1/meet/session-1/rec-1/source.webm".into(),
            kms_shred_key_id: "kms-rec-1".into(),
            retention_policy_id: "retention-7y".into(),
            status,
            byte_len: 4096,
            started_at_epoch_seconds: 1_700_000_000,
            ended_at_epoch_seconds: Some(1_700_000_900),
        })
        .unwrap()
    }

    fn policy() -> ArchiveRetentionPolicy {
        ArchiveRetentionPolicy::new(ArchiveRetentionPolicyCreate {
            retention_policy_id: "retention-7y".into(),
            kms_shred_key_id: "kms-rec-1".into(),
            purge_after_epoch_seconds: 1_920_000_000,
            legal_hold_id: None,
        })
        .unwrap()
    }

    fn variant(format: RecordingVariantFormat, variant_id: &str) -> RecordingVariant {
        RecordingVariant::new(RecordingVariantCreate {
            variant_id: variant_id.into(),
            format,
            storage_key: format!("tenant-1/meet/session-1/rec-1/{variant_id}.bin"),
            checksum: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into(),
            byte_len: 2048,
            duration_seconds: 900,
        })
        .unwrap()
    }

    fn archive_input() -> RecordingArchiveEntryCreate {
        RecordingArchiveEntryCreate {
            archive_id: "archive-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            session_id: "session-1".into(),
            recording: recording(RecordingStatus::Completed),
            retention_policy: policy(),
            variants: vec![variant(RecordingVariantFormat::Mp4Video, "mp4")],
            transcript_ref: Some("transcript-1".into()),
            summary_ref: Some("summary-1".into()),
            data_class: None,
            archived_at_epoch_seconds: 1_700_001_000,
        }
    }

    #[test]
    fn archive_entry_requires_completed_trust_portal_recording() {
        let archive = RecordingArchiveEntry::new(archive_input()).unwrap();
        assert_eq!(
            archive.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            archive.recording.data_class,
            DataClassification::Privacy(recording_archive_data_class())
        );
        assert_eq!(archive.schema_version.value, 1);

        let mut active = archive_input();
        active.recording = recording(RecordingStatus::Recording);
        assert_eq!(
            RecordingArchiveEntry::new(active),
            Err(RecordingArchiveError::RecordingNotCompleted)
        );
    }

    #[test]
    fn retention_policy_must_match_recording_kms_and_policy() {
        let mut wrong_policy = archive_input();
        wrong_policy.retention_policy = ArchiveRetentionPolicy::new(ArchiveRetentionPolicyCreate {
            retention_policy_id: "retention-30d".into(),
            kms_shred_key_id: "kms-rec-1".into(),
            purge_after_epoch_seconds: 1_800_000_000,
            legal_hold_id: None,
        })
        .unwrap();
        assert_eq!(
            RecordingArchiveEntry::new(wrong_policy),
            Err(RecordingArchiveError::RetentionPolicyMismatch)
        );

        let held = ArchiveRetentionPolicy::new(ArchiveRetentionPolicyCreate {
            retention_policy_id: "retention-7y".into(),
            kms_shred_key_id: "kms-rec-1".into(),
            purge_after_epoch_seconds: 1_700_000_000,
            legal_hold_id: Some("legal-hold-1".into()),
        })
        .unwrap();
        assert!(!held.can_purge_at(1_800_000_000));
        assert!(policy().can_purge_at(1_930_000_000));
    }

    #[test]
    fn variants_reject_duplicates_unsafe_keys_and_empty_bytes() {
        let mut duplicate_format = archive_input();
        duplicate_format.variants = vec![
            variant(RecordingVariantFormat::Mp4Video, "mp4-a"),
            variant(RecordingVariantFormat::Mp4Video, "mp4-b"),
        ];
        assert_eq!(
            RecordingArchiveEntry::new(duplicate_format),
            Err(RecordingArchiveError::DuplicateVariantFormat)
        );

        assert_eq!(
            RecordingVariant::new(RecordingVariantCreate {
                variant_id: "bad".into(),
                format: RecordingVariantFormat::WebmVideo,
                storage_key: "tenant-1/../bad.webm".into(),
                checksum: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
                byte_len: 1,
                duration_seconds: 1,
            }),
            Err(RecordingArchiveError::InvalidVariantStorageKey)
        );

        assert_eq!(
            RecordingVariant::new(RecordingVariantCreate {
                variant_id: "empty".into(),
                format: RecordingVariantFormat::OpusAudio,
                storage_key: "tenant-1/meet/session-1/empty.opus".into(),
                checksum: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
                byte_len: 0,
                duration_seconds: 1,
            }),
            Err(RecordingArchiveError::EmptyVariantBytes)
        );
    }

    #[test]
    fn archive_time_cannot_precede_recording_end() {
        let mut input = archive_input();
        input.archived_at_epoch_seconds = 1_700_000_800;
        assert_eq!(
            RecordingArchiveEntry::new(input),
            Err(RecordingArchiveError::InvalidTimeOrder)
        );
    }

    fn ingest_input() -> RecordingIngestRequestCreate {
        RecordingIngestRequestCreate {
            idempotency_key: "tenant-1:manual-upload-1".into(),
            tenant_id: "tenant-1".into(),
            source_kind: RecordingSourceKind::ManualUpload,
            source_ref: "upload-1".into(),
            content_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .into(),
            byte_len: 4096,
            duration_seconds: 900,
            ingested_at_epoch_seconds: 1_700_001_000,
            ended_at_epoch_seconds: Some(1_700_000_900),
            consent_banner_confirmed: true,
            context_kind: RecordingContextKind::Professional,
            author_ref: Some("user:host@example.com".into()),
            cell_id: "cell-a".into(),
            producer_spiffe_id: "spiffe://oyatie.local/ns/meet/sa/recording-producer".into(),
            parent_audit_chain_ref: "audit-chain:meet-session-1:seal-9".into(),
        }
    }

    #[test]
    fn ingest_request_requires_source_locked_identity_consent_and_audit_fields() {
        let request = RecordingIngestRequest::new(ingest_input()).unwrap();

        assert_eq!(
            request.idempotency_scope(),
            ("tenant-1", "tenant-1:manual-upload-1")
        );
        assert_eq!(
            request.content_hash.value,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(
            request.context_kind.value,
            RecordingContextKind::Professional
        );
        assert_eq!(
            request.producer_spiffe_id.value,
            "spiffe://oyatie.local/ns/meet/sa/recording-producer"
        );
        assert_eq!(
            request.parent_audit_chain_ref.value,
            "audit-chain:meet-session-1:seal-9"
        );
        assert_eq!(request.schema_version.value, 1);

        let mut missing_consent = ingest_input();
        missing_consent.consent_banner_confirmed = false;
        assert_eq!(
            RecordingIngestRequest::new(missing_consent),
            Err(RecordingArchiveError::MissingConsentBanner)
        );

        let mut missing_producer = ingest_input();
        missing_producer.producer_spiffe_id.clear();
        assert_eq!(
            RecordingIngestRequest::new(missing_producer),
            Err(RecordingArchiveError::InvalidProducerSpiffeId)
        );

        let mut malformed_hash = ingest_input();
        malformed_hash.content_hash = "sha256:abc123".into();
        assert_eq!(
            RecordingIngestRequest::new(malformed_hash),
            Err(RecordingArchiveError::InvalidChecksum)
        );

        let mut malformed_producer = ingest_input();
        malformed_producer.producer_spiffe_id = "meet-recording-producer".into();
        assert_eq!(
            RecordingIngestRequest::new(malformed_producer),
            Err(RecordingArchiveError::InvalidProducerSpiffeId)
        );

        let mut missing_audit_parent = ingest_input();
        missing_audit_parent.parent_audit_chain_ref = "meet-session-1:seal-9".into();
        assert_eq!(
            RecordingIngestRequest::new(missing_audit_parent),
            Err(RecordingArchiveError::InvalidParentAuditChainRef)
        );

        let mut invalid_time = ingest_input();
        invalid_time.ended_at_epoch_seconds = Some(1_700_001_001);
        assert_eq!(
            RecordingIngestRequest::new(invalid_time),
            Err(RecordingArchiveError::InvalidTimeOrder)
        );
    }

    #[test]
    fn idempotent_ingest_reuses_same_hash_and_rejects_mismatched_replay() {
        let request = RecordingIngestRequest::new(ingest_input()).unwrap();
        let receipt = RecordingIngestReceipt::accepted(
            "recording-1".into(),
            &request,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            "audit-chain:recordings:ingest:seal-1".into(),
        )
        .unwrap();

        assert_eq!(
            classify_idempotent_ingest(Some(&receipt), &request),
            Ok(RecordingIngestDecision::AlreadyAccepted {
                recording_id: "recording-1".into()
            })
        );

        let mut mismatched = ingest_input();
        mismatched.content_hash =
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
        let mismatched = RecordingIngestRequest::new(mismatched).unwrap();
        assert_eq!(
            classify_idempotent_ingest(Some(&receipt), &mismatched),
            Err(RecordingArchiveError::DuplicateIngestContentHashMismatch)
        );

        assert_eq!(
            classify_idempotent_ingest(None, &request),
            Ok(RecordingIngestDecision::AcceptNew)
        );

        assert_eq!(
            RecordingIngestReceipt::accepted(
                "recording-1".into(),
                &request,
                "sha256:abc123".into(),
                "audit-chain:recordings:ingest:seal-1".into(),
            ),
            Err(RecordingArchiveError::InvalidChecksum)
        );

        assert_eq!(
            RecordingIngestReceipt::accepted(
                "recording-1".into(),
                &request,
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                "recordings:ingest:seal-1".into(),
            ),
            Err(RecordingArchiveError::InvalidParentAuditChainRef)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_recording_data_class_from_legacy(DataClass::Audit),
            Err(RecordingArchiveError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.recordings STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingsSurfaceStaging {
    pub recording_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub kms_shred_ref: Classified<String>, // data_class: INTERNAL_ONLY
}

impl RecordingsSurfaceStaging {
    pub fn new(recording_id: String, tenant_id: String, kms_shred_ref: String) -> Self {
        Self {
            recording_id: Classified::new(recording_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            kms_shred_ref: Classified::new(kms_shred_ref, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> RecordingsSurfaceStaging {
        RecordingsSurfaceStaging::new(
            "recordings-1".into(),
            "recordings-1".into(),
            "recordings-1".into(),
        )
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.recording_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
