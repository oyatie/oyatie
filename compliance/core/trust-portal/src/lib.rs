//! Workspace trust portal kernel.
//!
//! Tenant-visible trust portal records for ADR-0038. This crate owns typed
//! publication contracts for lineage, DSR queue state, proof archive entries,
//! API stability, SLA windows, override packs, consent receipts, subprocessors,
//! residency declarations, and plugin trust tiers. Apps own rendering, storage,
//! authorization, and audit-chain append.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use compliance_dsr::{
    DsrAction, DsrAxis, DsrCompletionRecord, DsrRequest, DsrSlaStatus, DsrStoreKind, ErasureProof,
};
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use network_residency::ResidencyClass;

const TRUST_PORTAL_SECTION_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_LINEAGE_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_DSR_QUEUE_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_PROOF_ARCHIVE_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_API_STABILITY_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_AXIS_SLO_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_OVERRIDE_PACK_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_CONSENT_RECEIPT_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_SUBPROCESSOR_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_RESIDENCY_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_PLUGIN_TRUST_SCHEMA_VERSION: u32 = 1;
const TRUST_PORTAL_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
const SHA256_PREFIX: &str = "sha256:";
const MAX_AVAILABILITY_BASIS_POINTS: u32 = 10_000;
const TRUST_PORTAL_SECTIONS: [TrustPortalSection; 10] = [
    TrustPortalSection::LineageView,
    TrustPortalSection::DsrQueue,
    TrustPortalSection::ProofArchive,
    TrustPortalSection::ApiStabilityMirror,
    TrustPortalSection::AxisSlaAndUptime,
    TrustPortalSection::OverridePackView,
    TrustPortalSection::ConsentReceiptArchive,
    TrustPortalSection::SubprocessorList,
    TrustPortalSection::ResidencyDeclaration,
    TrustPortalSection::PluginTrustTierMatrix,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrustPortalError {
    InvalidSnapshotId,
    InvalidTenantId,
    InvalidRegion,
    InvalidSectionEvidenceRef,
    EmptySectionSummarySet,
    DuplicateSectionSummary,
    MissingSectionSummary,
    SectionCountMismatch,
    InvalidLineageId,
    InvalidStoreId,
    InvalidFlowRef,
    InvalidQueueEntryId,
    InvalidDsrId,
    InvalidCompletionId,
    InvalidArchiveEntryId,
    InvalidProofId,
    InvalidDispatchId,
    InvalidEvidenceHash,
    InvalidSignatureRef,
    InvalidDownloadableRef,
    InvalidApiId,
    InvalidApiName,
    InvalidApiUsageSummaryRef,
    InvalidApiSunset,
    InvalidSloWindow,
    InvalidAvailability,
    InvalidPackId,
    InvalidJurisdiction,
    EmptyDeniedPurposeSet,
    InvalidDeniedPurpose,
    DuplicateDeniedPurpose,
    InvalidReceiptId,
    InvalidPurposeRef,
    InvalidProcessorId,
    InvalidLegalName,
    InvalidNoticeRef,
    InvalidResidencyEvidenceRef,
    EmptyResidencyDataClassSet,
    DuplicateDataClass,
    ResidencyCrossRegionMismatch,
    InvalidPluginId,
    InvalidSandboxProfileRef,
    InvalidTimeOrder,
    EmptyLineageView,
    EmptyApiStabilityMirror,
    EmptyAxisSloView,
    EmptySubprocessorList,
    EmptyPluginTrustMatrix,
    DuplicateLineageId,
    DuplicateDsrId,
    DuplicateProofId,
    DuplicateApiId,
    DuplicateOverridePackId,
    DuplicateReceiptId,
    DuplicateProcessorId,
    DuplicatePluginId,
    DsrCompletionMismatch,
    DsrCompletionStatusMismatch,
    MissingDsrCompletion,
    MissingProofArchiveEntry,
    ProofCompletionMismatch,
    TenantScopeMismatch,
    RegionScopeMismatch,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TrustPortalSection {
    LineageView,
    DsrQueue,
    ProofArchive,
    ApiStabilityMirror,
    AxisSlaAndUptime,
    OverridePackView,
    ConsentReceiptArchive,
    SubprocessorList,
    ResidencyDeclaration,
    PluginTrustTierMatrix,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TrustPortalDsrQueueStatus {
    Open,
    InProgress,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApiStabilityTier {
    Preview,
    Stable,
    Ga,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApiDeprecationStatus {
    Current,
    Deprecated,
    Sunset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PluginTrustTier {
    UnsignedReviewRequired,
    Signed,
    Reviewed,
    FirstParty,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalSectionSummaryCreate {
    pub section: TrustPortalSection,     // data_class: INTERNAL_ONLY
    pub published_record_count: u32,     // data_class: INTERNAL_ONLY
    pub refreshed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalSectionSummary {
    pub section: Classified<TrustPortalSection>, // data_class: INTERNAL_ONLY
    pub published_record_count: Classified<u32>, // data_class: INTERNAL_ONLY
    pub refreshed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<String>,        // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalLineageEntryCreate {
    pub lineage_id: String,            // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass,  // data_class: INTERNAL_ONLY
    pub axis: DsrAxis,                 // data_class: INTERNAL_ONLY
    pub store_kind: DsrStoreKind,      // data_class: INTERNAL_ONLY
    pub store_id: String,              // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: INTERNAL_ONLY
    pub flow_ref: String,              // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalLineageEntry {
    pub lineage_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub axis: Classified<DsrAxis>,      // data_class: INTERNAL_ONLY
    pub store_kind: Classified<DsrStoreKind>, // data_class: INTERNAL_ONLY
    pub store_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub region: Classified<String>,     // data_class: INTERNAL_ONLY
    pub flow_ref: Classified<String>,   // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalDsrQueueEntryCreate {
    pub queue_entry_id: String,                  // data_class: INTERNAL_ONLY
    pub request: DsrRequest,                     // data_class: INTERNAL_ONLY
    pub status: TrustPortalDsrQueueStatus,       // data_class: INTERNAL_ONLY
    pub completion: Option<DsrCompletionRecord>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalDsrQueueEntry {
    pub queue_entry_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub region: Classified<String>,         // data_class: INTERNAL_ONLY
    pub subject_ref: Classified<String>,    // data_class: PII_IDENTIFYING
    pub action: Classified<DsrAction>,      // data_class: INTERNAL_ONLY
    pub status: Classified<TrustPortalDsrQueueStatus>, // data_class: INTERNAL_ONLY
    pub sla_status: Classified<DsrSlaStatus>, // data_class: INTERNAL_ONLY
    pub data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub received_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub deadline_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub completion_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub completion_proof_ids: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalProofArchiveEntryCreate {
    pub archive_entry_id: String,        // data_class: INTERNAL_ONLY
    pub proof: ErasureProof,             // data_class: INTERNAL_ONLY
    pub completion_id: String,           // data_class: INTERNAL_ONLY
    pub downloadable_ref: String,        // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalProofArchiveEntry {
    pub archive_entry_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub proof_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub dispatch_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub dsr_id: Classified<String>,           // data_class: INTERNAL_ONLY
    pub completion_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub action: Classified<DsrAction>,        // data_class: INTERNAL_ONLY
    pub axis: Classified<DsrAxis>,            // data_class: INTERNAL_ONLY
    pub store_kind: Classified<DsrStoreKind>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub region: Classified<String>,           // data_class: INTERNAL_ONLY
    pub store_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub record_ref: Classified<String>,       // data_class: INTERNAL_ONLY
    pub evidence_hash: Classified<String>,    // data_class: INTERNAL_ONLY
    pub signature_ref: Classified<String>,    // data_class: INTERNAL_ONLY
    pub rekor_log_index: Classified<u64>,     // data_class: INTERNAL_ONLY
    pub downloadable_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub published_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalApiStabilityEntryCreate {
    pub api_id: String,                           // data_class: INTERNAL_ONLY
    pub api_name: String,                         // data_class: INTERNAL_ONLY
    pub tier: ApiStabilityTier,                   // data_class: INTERNAL_ONLY
    pub deprecation_status: ApiDeprecationStatus, // data_class: INTERNAL_ONLY
    pub sunset_at_epoch_seconds: Option<u64>,     // data_class: INTERNAL_ONLY
    pub usage_summary_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub changed_at_epoch_seconds: u64,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalApiStabilityEntry {
    pub api_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub api_name: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tier: Classified<ApiStabilityTier>, // data_class: INTERNAL_ONLY
    pub deprecation_status: Classified<ApiDeprecationStatus>, // data_class: INTERNAL_ONLY
    pub sunset_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub usage_summary_ref: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub changed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalAxisSloEntryCreate {
    pub axis: DsrAxis,                     // data_class: INTERNAL_ONLY
    pub availability_basis_points: u32,    // data_class: INTERNAL_ONLY
    pub window_start_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub window_end_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub last_incident_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalAxisSloEntry {
    pub axis: Classified<DsrAxis>, // data_class: INTERNAL_ONLY
    pub availability_basis_points: Classified<u32>, // data_class: INTERNAL_ONLY
    pub window_start_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub window_end_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub last_incident_ref: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalOverridePackEntryCreate {
    pub pack_id: String,                // data_class: INTERNAL_ONLY
    pub jurisdiction: String,           // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass,   // data_class: INTERNAL_ONLY
    pub denied_purposes: Vec<String>,   // data_class: INTERNAL_ONLY
    pub reviewed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalOverridePackEntry {
    pub pack_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub jurisdiction: Classified<String>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub denied_purposes: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub reviewed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalConsentReceiptEntryCreate {
    pub receipt_id: String,                    // data_class: INTERNAL_ONLY
    pub purpose_ref: String,                   // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass,          // data_class: INTERNAL_ONLY
    pub granted_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub revoked_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalConsentReceiptEntry {
    pub receipt_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub purpose_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub granted_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub revoked_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalSubprocessorEntryCreate {
    pub processor_id: String,                // data_class: INTERNAL_ONLY
    pub legal_name: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: INTERNAL_ONLY
    pub purpose_ref: String,                 // data_class: INTERNAL_ONLY
    pub data_classes: Vec<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub notice_ref: String,                  // data_class: INTERNAL_ONLY
    pub notice_sent_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub effective_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalSubprocessorEntry {
    pub processor_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub legal_name: Classified<String>,   // data_class: INTERNAL_ONLY
    pub region: Classified<String>,       // data_class: INTERNAL_ONLY
    pub purpose_ref: Classified<String>,  // data_class: INTERNAL_ONLY
    pub data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub notice_ref: Classified<String>,   // data_class: INTERNAL_ONLY
    pub notice_sent_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub effective_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalResidencyDeclarationCreate {
    pub residency_class: ResidencyClass, // data_class: INTERNAL_ONLY
    pub primary_region: String,          // data_class: INTERNAL_ONLY
    pub data_classes: Vec<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub cross_region_allowed: bool,      // data_class: INTERNAL_ONLY
    pub cross_region_regions: Vec<String>, // data_class: INTERNAL_ONLY
    pub consent_receipt_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub declared_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalResidencyDeclaration {
    pub residency_class: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub primary_region: Classified<String>,          // data_class: INTERNAL_ONLY
    pub data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub cross_region_allowed: Classified<bool>,      // data_class: INTERNAL_ONLY
    pub cross_region_regions: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub consent_receipt_refs: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<String>,            // data_class: INTERNAL_ONLY
    pub declared_at_epoch_seconds: Classified<u64>,  // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalPluginTrustEntryCreate {
    pub plugin_id: String,              // data_class: INTERNAL_ONLY
    pub tier: PluginTrustTier,          // data_class: INTERNAL_ONLY
    pub signature_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub sandbox_profile_ref: String,    // data_class: INTERNAL_ONLY
    pub reviewed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalPluginTrustEntry {
    pub plugin_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub tier: Classified<PluginTrustTier>, // data_class: INTERNAL_ONLY
    pub signature_ref: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub sandbox_profile_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub reviewed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalTenantSnapshotCreate {
    pub snapshot_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub region: String,      // data_class: INTERNAL_ONLY
    pub section_summaries: Vec<TrustPortalSectionSummary>, // data_class: INTERNAL_ONLY
    pub lineage_entries: Vec<TrustPortalLineageEntry>, // data_class: INTERNAL_ONLY
    pub dsr_queue_entries: Vec<TrustPortalDsrQueueEntry>, // data_class: INTERNAL_ONLY
    pub proof_archive_entries: Vec<TrustPortalProofArchiveEntry>, // data_class: INTERNAL_ONLY
    pub api_stability_entries: Vec<TrustPortalApiStabilityEntry>, // data_class: INTERNAL_ONLY
    pub axis_slo_entries: Vec<TrustPortalAxisSloEntry>, // data_class: INTERNAL_ONLY
    pub override_pack_entries: Vec<TrustPortalOverridePackEntry>, // data_class: INTERNAL_ONLY
    pub consent_receipt_entries: Vec<TrustPortalConsentReceiptEntry>, // data_class: INTERNAL_ONLY
    pub subprocessor_entries: Vec<TrustPortalSubprocessorEntry>, // data_class: INTERNAL_ONLY
    pub residency_declaration: TrustPortalResidencyDeclaration, // data_class: INTERNAL_ONLY
    pub plugin_trust_entries: Vec<TrustPortalPluginTrustEntry>, // data_class: INTERNAL_ONLY
    pub generated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPortalTenantSnapshot {
    pub snapshot_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub region: Classified<String>,      // data_class: INTERNAL_ONLY
    pub section_summaries: Classified<Vec<TrustPortalSectionSummary>>, // data_class: INTERNAL_ONLY
    pub lineage_entries: Classified<Vec<TrustPortalLineageEntry>>, // data_class: INTERNAL_ONLY
    pub dsr_queue_entries: Classified<Vec<TrustPortalDsrQueueEntry>>, // data_class: INTERNAL_ONLY
    pub proof_archive_entries: Classified<Vec<TrustPortalProofArchiveEntry>>, // data_class: INTERNAL_ONLY
    pub api_stability_entries: Classified<Vec<TrustPortalApiStabilityEntry>>, // data_class: INTERNAL_ONLY
    pub axis_slo_entries: Classified<Vec<TrustPortalAxisSloEntry>>, // data_class: INTERNAL_ONLY
    pub override_pack_entries: Classified<Vec<TrustPortalOverridePackEntry>>, // data_class: INTERNAL_ONLY
    pub consent_receipt_entries: Classified<Vec<TrustPortalConsentReceiptEntry>>, // data_class: INTERNAL_ONLY
    pub subprocessor_entries: Classified<Vec<TrustPortalSubprocessorEntry>>, // data_class: INTERNAL_ONLY
    pub residency_declaration: Classified<TrustPortalResidencyDeclaration>, // data_class: INTERNAL_ONLY
    pub plugin_trust_entries: Classified<Vec<TrustPortalPluginTrustEntry>>, // data_class: INTERNAL_ONLY
    pub generated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,             // data_class: INTERNAL_ONLY
}

impl TrustPortalSectionSummary {
    pub fn new(input: TrustPortalSectionSummaryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(
            &input.evidence_ref,
            TrustPortalError::InvalidSectionEvidenceRef,
        )?;
        Ok(Self {
            section: internal(input.section),
            published_record_count: internal(input.published_record_count),
            refreshed_at_epoch_seconds: internal(input.refreshed_at_epoch_seconds),
            evidence_ref: internal(input.evidence_ref),
            schema_version: internal(TRUST_PORTAL_SECTION_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalLineageEntry {
    pub fn new(input: TrustPortalLineageEntryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.lineage_id, TrustPortalError::InvalidLineageId)?;
        validate_non_empty(&input.store_id, TrustPortalError::InvalidStoreId)?;
        validate_non_empty(&input.region, TrustPortalError::InvalidRegion)?;
        validate_non_empty(&input.flow_ref, TrustPortalError::InvalidFlowRef)?;
        Ok(Self {
            lineage_id: internal(input.lineage_id),
            data_class: internal(input.data_class),
            axis: internal(input.axis),
            store_kind: internal(input.store_kind),
            store_id: internal(input.store_id),
            region: internal(input.region),
            flow_ref: internal(input.flow_ref),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_LINEAGE_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalDsrQueueEntry {
    pub fn new(input: TrustPortalDsrQueueEntryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.queue_entry_id, TrustPortalError::InvalidQueueEntryId)?;
        validate_time_order(
            input.request.received_at_epoch_seconds.value,
            input.updated_at_epoch_seconds,
        )?;
        let (completion_id, completion_proof_ids, sla_status) = validate_queue_completion(&input)?;
        Ok(Self {
            queue_entry_id: internal(input.queue_entry_id),
            dsr_id: internal(input.request.dsr_id.value),
            tenant_id: internal(input.request.tenant_id.value),
            region: internal(input.request.region.value),
            subject_ref: subject(input.request.subject_ref.value),
            action: internal(input.request.action.value),
            status: internal(input.status),
            sla_status: internal(sla_status),
            data_classes: internal(input.request.data_classes.value),
            received_at_epoch_seconds: internal(input.request.received_at_epoch_seconds.value),
            deadline_epoch_seconds: internal(input.request.deadline_epoch_seconds.value),
            completion_id: internal(completion_id),
            completion_proof_ids: internal(completion_proof_ids),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_DSR_QUEUE_SCHEMA_VERSION),
        })
    }

    pub fn is_completed(&self) -> bool {
        self.status.value == TrustPortalDsrQueueStatus::Completed
    }
}

impl TrustPortalProofArchiveEntry {
    pub fn new(input: TrustPortalProofArchiveEntryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(
            &input.archive_entry_id,
            TrustPortalError::InvalidArchiveEntryId,
        )?;
        validate_non_empty(&input.completion_id, TrustPortalError::InvalidCompletionId)?;
        validate_non_empty(
            &input.proof.proof_id.value,
            TrustPortalError::InvalidProofId,
        )?;
        validate_non_empty(
            &input.proof.dispatch_id.value,
            TrustPortalError::InvalidDispatchId,
        )?;
        validate_hash(
            &input.proof.evidence_hash.value,
            TrustPortalError::InvalidEvidenceHash,
        )?;
        validate_non_empty(
            &input.proof.signature_ref.value,
            TrustPortalError::InvalidSignatureRef,
        )?;
        validate_non_empty(
            &input.downloadable_ref,
            TrustPortalError::InvalidDownloadableRef,
        )?;
        validate_time_order(
            input.proof.proved_at_epoch_seconds.value,
            input.published_at_epoch_seconds,
        )?;
        Ok(Self {
            archive_entry_id: internal(input.archive_entry_id),
            proof_id: internal(input.proof.proof_id.value),
            dispatch_id: internal(input.proof.dispatch_id.value),
            dsr_id: internal(input.proof.dsr_id.value),
            completion_id: internal(input.completion_id),
            action: internal(input.proof.action.value),
            axis: internal(input.proof.store.value.axis.value),
            store_kind: internal(input.proof.store.value.kind.value),
            tenant_id: internal(input.proof.store.value.tenant_id.value),
            region: internal(input.proof.store.value.region.value),
            store_id: internal(input.proof.store.value.store_id.value),
            record_ref: internal(input.proof.store.value.record_ref.value),
            evidence_hash: internal(input.proof.evidence_hash.value),
            signature_ref: internal(input.proof.signature_ref.value),
            rekor_log_index: internal(input.proof.rekor_log_index.value),
            downloadable_ref: internal(input.downloadable_ref),
            published_at_epoch_seconds: internal(input.published_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_PROOF_ARCHIVE_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalApiStabilityEntry {
    pub fn new(input: TrustPortalApiStabilityEntryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.api_id, TrustPortalError::InvalidApiId)?;
        validate_non_empty(&input.api_name, TrustPortalError::InvalidApiName)?;
        validate_optional_non_empty(
            input.usage_summary_ref.as_deref(),
            TrustPortalError::InvalidApiUsageSummaryRef,
        )?;
        validate_api_sunset(&input)?;
        Ok(Self {
            api_id: internal(input.api_id),
            api_name: internal(input.api_name),
            tier: internal(input.tier),
            deprecation_status: internal(input.deprecation_status),
            sunset_at_epoch_seconds: internal(input.sunset_at_epoch_seconds),
            usage_summary_ref: internal(input.usage_summary_ref),
            changed_at_epoch_seconds: internal(input.changed_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_API_STABILITY_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalAxisSloEntry {
    pub fn new(input: TrustPortalAxisSloEntryCreate) -> Result<Self, TrustPortalError> {
        if input.availability_basis_points > MAX_AVAILABILITY_BASIS_POINTS {
            return Err(TrustPortalError::InvalidAvailability);
        }
        if input.window_start_epoch_seconds >= input.window_end_epoch_seconds {
            return Err(TrustPortalError::InvalidSloWindow);
        }
        validate_optional_non_empty(
            input.last_incident_ref.as_deref(),
            TrustPortalError::InvalidNoticeRef,
        )?;
        Ok(Self {
            axis: internal(input.axis),
            availability_basis_points: internal(input.availability_basis_points),
            window_start_epoch_seconds: internal(input.window_start_epoch_seconds),
            window_end_epoch_seconds: internal(input.window_end_epoch_seconds),
            last_incident_ref: internal(input.last_incident_ref),
            schema_version: internal(TRUST_PORTAL_AXIS_SLO_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalOverridePackEntry {
    pub fn new(input: TrustPortalOverridePackEntryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.pack_id, TrustPortalError::InvalidPackId)?;
        validate_non_empty(&input.jurisdiction, TrustPortalError::InvalidJurisdiction)?;
        validate_denied_purposes(&input.denied_purposes)?;
        Ok(Self {
            pack_id: internal(input.pack_id),
            jurisdiction: internal(input.jurisdiction),
            data_class: internal(input.data_class),
            denied_purposes: internal(input.denied_purposes),
            reviewed_at_epoch_seconds: internal(input.reviewed_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_OVERRIDE_PACK_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalConsentReceiptEntry {
    pub fn new(input: TrustPortalConsentReceiptEntryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.receipt_id, TrustPortalError::InvalidReceiptId)?;
        validate_non_empty(&input.purpose_ref, TrustPortalError::InvalidPurposeRef)?;
        validate_non_empty(
            &input.evidence_ref,
            TrustPortalError::InvalidSectionEvidenceRef,
        )?;
        if let Some(revoked_at) = input.revoked_at_epoch_seconds {
            validate_time_order(input.granted_at_epoch_seconds, revoked_at)?;
        }
        Ok(Self {
            receipt_id: internal(input.receipt_id),
            purpose_ref: internal(input.purpose_ref),
            data_class: internal(input.data_class),
            granted_at_epoch_seconds: internal(input.granted_at_epoch_seconds),
            revoked_at_epoch_seconds: internal(input.revoked_at_epoch_seconds),
            evidence_ref: internal(input.evidence_ref),
            schema_version: internal(TRUST_PORTAL_CONSENT_RECEIPT_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalSubprocessorEntry {
    pub fn new(input: TrustPortalSubprocessorEntryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.processor_id, TrustPortalError::InvalidProcessorId)?;
        validate_non_empty(&input.legal_name, TrustPortalError::InvalidLegalName)?;
        validate_non_empty(&input.region, TrustPortalError::InvalidRegion)?;
        validate_non_empty(&input.purpose_ref, TrustPortalError::InvalidPurposeRef)?;
        validate_non_empty(&input.notice_ref, TrustPortalError::InvalidNoticeRef)?;
        validate_data_classes(&input.data_classes)?;
        validate_time_order(
            input.notice_sent_at_epoch_seconds,
            input.effective_at_epoch_seconds,
        )?;
        Ok(Self {
            processor_id: internal(input.processor_id),
            legal_name: internal(input.legal_name),
            region: internal(input.region),
            purpose_ref: internal(input.purpose_ref),
            data_classes: internal(input.data_classes),
            notice_ref: internal(input.notice_ref),
            notice_sent_at_epoch_seconds: internal(input.notice_sent_at_epoch_seconds),
            effective_at_epoch_seconds: internal(input.effective_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_SUBPROCESSOR_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalResidencyDeclaration {
    pub fn new(input: TrustPortalResidencyDeclarationCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.primary_region, TrustPortalError::InvalidRegion)?;
        validate_non_empty(
            &input.evidence_ref,
            TrustPortalError::InvalidResidencyEvidenceRef,
        )?;
        validate_data_classes(&input.data_classes)?;
        validate_cross_region_residency(&input)?;
        Ok(Self {
            residency_class: internal(input.residency_class),
            primary_region: internal(input.primary_region),
            data_classes: internal(input.data_classes),
            cross_region_allowed: internal(input.cross_region_allowed),
            cross_region_regions: internal(input.cross_region_regions),
            consent_receipt_refs: internal(input.consent_receipt_refs),
            evidence_ref: internal(input.evidence_ref),
            declared_at_epoch_seconds: internal(input.declared_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_RESIDENCY_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalPluginTrustEntry {
    pub fn new(input: TrustPortalPluginTrustEntryCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.plugin_id, TrustPortalError::InvalidPluginId)?;
        validate_non_empty(
            &input.sandbox_profile_ref,
            TrustPortalError::InvalidSandboxProfileRef,
        )?;
        validate_plugin_signature(input.tier, input.signature_ref.as_deref())?;
        Ok(Self {
            plugin_id: internal(input.plugin_id),
            tier: internal(input.tier),
            signature_ref: internal(input.signature_ref),
            sandbox_profile_ref: internal(input.sandbox_profile_ref),
            reviewed_at_epoch_seconds: internal(input.reviewed_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_PLUGIN_TRUST_SCHEMA_VERSION),
        })
    }
}

impl TrustPortalTenantSnapshot {
    pub fn new(input: TrustPortalTenantSnapshotCreate) -> Result<Self, TrustPortalError> {
        validate_non_empty(&input.snapshot_id, TrustPortalError::InvalidSnapshotId)?;
        validate_non_empty(&input.tenant_id, TrustPortalError::InvalidTenantId)?;
        validate_non_empty(&input.region, TrustPortalError::InvalidRegion)?;
        validate_required_views(&input)?;
        validate_section_summaries(&input)?;
        validate_scope(&input)?;
        validate_unique_ids(&input)?;
        validate_dsr_proof_archive(&input)?;
        validate_generated_at(&input)?;
        Ok(Self {
            snapshot_id: internal(input.snapshot_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            section_summaries: internal(input.section_summaries),
            lineage_entries: internal(input.lineage_entries),
            dsr_queue_entries: internal(input.dsr_queue_entries),
            proof_archive_entries: internal(input.proof_archive_entries),
            api_stability_entries: internal(input.api_stability_entries),
            axis_slo_entries: internal(input.axis_slo_entries),
            override_pack_entries: internal(input.override_pack_entries),
            consent_receipt_entries: internal(input.consent_receipt_entries),
            subprocessor_entries: internal(input.subprocessor_entries),
            residency_declaration: internal(input.residency_declaration),
            plugin_trust_entries: internal(input.plugin_trust_entries),
            generated_at_epoch_seconds: internal(input.generated_at_epoch_seconds),
            schema_version: internal(TRUST_PORTAL_SNAPSHOT_SCHEMA_VERSION),
        })
    }
}

pub trait TrustPortalPublisher {
    fn publish_snapshot(
        &self,
        snapshot: &TrustPortalTenantSnapshot,
    ) -> Result<(), TrustPortalError>;
}

pub fn trust_portal_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, TrustPortalError> {
    PrivacyDataClass::new(data_class).map_err(|_| TrustPortalError::InvalidDataClass)
}

pub fn default_trust_portal_subject_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

fn validate_queue_completion(
    input: &TrustPortalDsrQueueEntryCreate,
) -> Result<(Option<String>, Vec<String>, DsrSlaStatus), TrustPortalError> {
    match (&input.status, input.completion.as_ref()) {
        (TrustPortalDsrQueueStatus::Completed, Some(completion)) => {
            if completion.dsr_id.value != input.request.dsr_id.value {
                return Err(TrustPortalError::DsrCompletionMismatch);
            }
            validate_time_order(
                completion.completed_at_epoch_seconds.value,
                input.updated_at_epoch_seconds,
            )?;
            Ok((
                Some(completion.completion_id.value.clone()),
                completion.proof_ids.value.clone(),
                completion.sla_status.value,
            ))
        }
        (TrustPortalDsrQueueStatus::Completed, None) => Err(TrustPortalError::MissingDsrCompletion),
        (_, Some(_)) => Err(TrustPortalError::DsrCompletionStatusMismatch),
        (_, None) => {
            let sla_status =
                if input.updated_at_epoch_seconds <= input.request.deadline_epoch_seconds.value {
                    DsrSlaStatus::WithinSla
                } else {
                    DsrSlaStatus::Breached
                };
            Ok((None, Vec::new(), sla_status))
        }
    }
}

fn validate_api_sunset(input: &TrustPortalApiStabilityEntryCreate) -> Result<(), TrustPortalError> {
    match (input.deprecation_status, input.sunset_at_epoch_seconds) {
        (ApiDeprecationStatus::Current, None) => Ok(()),
        (ApiDeprecationStatus::Current, Some(_)) => Err(TrustPortalError::InvalidApiSunset),
        (ApiDeprecationStatus::Deprecated | ApiDeprecationStatus::Sunset, Some(sunset_at)) => {
            validate_time_order(input.changed_at_epoch_seconds, sunset_at)
                .map_err(|_| TrustPortalError::InvalidApiSunset)
        }
        (ApiDeprecationStatus::Deprecated | ApiDeprecationStatus::Sunset, None) => {
            Err(TrustPortalError::InvalidApiSunset)
        }
    }
}

fn validate_denied_purposes(denied_purposes: &[String]) -> Result<(), TrustPortalError> {
    if denied_purposes.is_empty() {
        return Err(TrustPortalError::EmptyDeniedPurposeSet);
    }
    let mut seen = BTreeSet::new();
    for purpose in denied_purposes {
        validate_non_empty(purpose, TrustPortalError::InvalidDeniedPurpose)?;
        if !seen.insert(purpose.as_str()) {
            return Err(TrustPortalError::DuplicateDeniedPurpose);
        }
    }
    Ok(())
}

fn validate_cross_region_residency(
    input: &TrustPortalResidencyDeclarationCreate,
) -> Result<(), TrustPortalError> {
    for region in &input.cross_region_regions {
        validate_non_empty(region, TrustPortalError::InvalidRegion)?;
    }
    for receipt_ref in &input.consent_receipt_refs {
        validate_non_empty(receipt_ref, TrustPortalError::InvalidReceiptId)?;
    }
    if input.cross_region_allowed {
        if input.cross_region_regions.is_empty() || input.consent_receipt_refs.is_empty() {
            return Err(TrustPortalError::ResidencyCrossRegionMismatch);
        }
    } else if !input.cross_region_regions.is_empty() || !input.consent_receipt_refs.is_empty() {
        return Err(TrustPortalError::ResidencyCrossRegionMismatch);
    }
    Ok(())
}

fn residency_class_forbids_cross_region(residency_class: &ResidencyClass) -> bool {
    residency_class
        .label()
        .is_some_and(|label| label.starts_with("strict_"))
}

fn validate_plugin_signature(
    tier: PluginTrustTier,
    signature_ref: Option<&str>,
) -> Result<(), TrustPortalError> {
    match tier {
        PluginTrustTier::UnsignedReviewRequired => {
            if signature_ref.is_some() {
                return Err(TrustPortalError::InvalidSignatureRef);
            }
        }
        PluginTrustTier::Signed | PluginTrustTier::Reviewed | PluginTrustTier::FirstParty => {
            validate_optional_non_empty(signature_ref, TrustPortalError::InvalidSignatureRef)?;
            if signature_ref.is_none() {
                return Err(TrustPortalError::InvalidSignatureRef);
            }
        }
    }
    Ok(())
}

fn validate_required_views(
    input: &TrustPortalTenantSnapshotCreate,
) -> Result<(), TrustPortalError> {
    if input.lineage_entries.is_empty() {
        return Err(TrustPortalError::EmptyLineageView);
    }
    if input.api_stability_entries.is_empty() {
        return Err(TrustPortalError::EmptyApiStabilityMirror);
    }
    if input.axis_slo_entries.is_empty() {
        return Err(TrustPortalError::EmptyAxisSloView);
    }
    if input.subprocessor_entries.is_empty() {
        return Err(TrustPortalError::EmptySubprocessorList);
    }
    if input.plugin_trust_entries.is_empty() {
        return Err(TrustPortalError::EmptyPluginTrustMatrix);
    }
    Ok(())
}

fn validate_section_summaries(
    input: &TrustPortalTenantSnapshotCreate,
) -> Result<(), TrustPortalError> {
    if input.section_summaries.is_empty() {
        return Err(TrustPortalError::EmptySectionSummarySet);
    }
    let mut by_section = BTreeMap::new();
    for summary in &input.section_summaries {
        if summary.refreshed_at_epoch_seconds.value > input.generated_at_epoch_seconds {
            return Err(TrustPortalError::InvalidTimeOrder);
        }
        if by_section.insert(summary.section.value, summary).is_some() {
            return Err(TrustPortalError::DuplicateSectionSummary);
        }
    }
    for section in TRUST_PORTAL_SECTIONS {
        let Some(summary) = by_section.get(&section) else {
            return Err(TrustPortalError::MissingSectionSummary);
        };
        if summary.published_record_count.value as usize != section_record_count(input, section) {
            return Err(TrustPortalError::SectionCountMismatch);
        }
    }
    if by_section.len() != TRUST_PORTAL_SECTIONS.len() {
        return Err(TrustPortalError::DuplicateSectionSummary);
    }
    Ok(())
}

fn section_record_count(
    input: &TrustPortalTenantSnapshotCreate,
    section: TrustPortalSection,
) -> usize {
    match section {
        TrustPortalSection::LineageView => input.lineage_entries.len(),
        TrustPortalSection::DsrQueue => input.dsr_queue_entries.len(),
        TrustPortalSection::ProofArchive => input.proof_archive_entries.len(),
        TrustPortalSection::ApiStabilityMirror => input.api_stability_entries.len(),
        TrustPortalSection::AxisSlaAndUptime => input.axis_slo_entries.len(),
        TrustPortalSection::OverridePackView => input.override_pack_entries.len(),
        TrustPortalSection::ConsentReceiptArchive => input.consent_receipt_entries.len(),
        TrustPortalSection::SubprocessorList => input.subprocessor_entries.len(),
        TrustPortalSection::ResidencyDeclaration => 1,
        TrustPortalSection::PluginTrustTierMatrix => input.plugin_trust_entries.len(),
    }
}

fn validate_scope(input: &TrustPortalTenantSnapshotCreate) -> Result<(), TrustPortalError> {
    if input.residency_declaration.primary_region.value != input.region {
        return Err(TrustPortalError::RegionScopeMismatch);
    }
    for lineage in &input.lineage_entries {
        if lineage.region.value != input.region {
            return Err(TrustPortalError::RegionScopeMismatch);
        }
    }
    for queue_entry in &input.dsr_queue_entries {
        if queue_entry.tenant_id.value != input.tenant_id {
            return Err(TrustPortalError::TenantScopeMismatch);
        }
        if queue_entry.region.value != input.region {
            return Err(TrustPortalError::RegionScopeMismatch);
        }
    }
    for proof in &input.proof_archive_entries {
        if proof.tenant_id.value != input.tenant_id {
            return Err(TrustPortalError::TenantScopeMismatch);
        }
        if proof.region.value != input.region {
            return Err(TrustPortalError::RegionScopeMismatch);
        }
    }
    Ok(())
}

fn validate_unique_ids(input: &TrustPortalTenantSnapshotCreate) -> Result<(), TrustPortalError> {
    validate_unique(
        input
            .lineage_entries
            .iter()
            .map(|entry| entry.lineage_id.value.as_str()),
        TrustPortalError::DuplicateLineageId,
    )?;
    validate_unique(
        input
            .dsr_queue_entries
            .iter()
            .map(|entry| entry.dsr_id.value.as_str()),
        TrustPortalError::DuplicateDsrId,
    )?;
    validate_unique(
        input
            .proof_archive_entries
            .iter()
            .map(|entry| entry.proof_id.value.as_str()),
        TrustPortalError::DuplicateProofId,
    )?;
    validate_unique(
        input
            .api_stability_entries
            .iter()
            .map(|entry| entry.api_id.value.as_str()),
        TrustPortalError::DuplicateApiId,
    )?;
    validate_unique(
        input
            .override_pack_entries
            .iter()
            .map(|entry| entry.pack_id.value.as_str()),
        TrustPortalError::DuplicateOverridePackId,
    )?;
    validate_unique(
        input
            .consent_receipt_entries
            .iter()
            .map(|entry| entry.receipt_id.value.as_str()),
        TrustPortalError::DuplicateReceiptId,
    )?;
    validate_unique(
        input
            .subprocessor_entries
            .iter()
            .map(|entry| entry.processor_id.value.as_str()),
        TrustPortalError::DuplicateProcessorId,
    )?;
    validate_unique(
        input
            .plugin_trust_entries
            .iter()
            .map(|entry| entry.plugin_id.value.as_str()),
        TrustPortalError::DuplicatePluginId,
    )
}

fn validate_dsr_proof_archive(
    input: &TrustPortalTenantSnapshotCreate,
) -> Result<(), TrustPortalError> {
    let completed_queue_by_dsr = input
        .dsr_queue_entries
        .iter()
        .filter(|entry| entry.is_completed())
        .map(|entry| (entry.dsr_id.value.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut proofs_by_dsr = BTreeMap::<&str, BTreeSet<String>>::new();
    for proof in &input.proof_archive_entries {
        let Some(queue_entry) = completed_queue_by_dsr.get(proof.dsr_id.value.as_str()) else {
            return Err(TrustPortalError::ProofCompletionMismatch);
        };
        if queue_entry.completion_id.value.as_deref() != Some(proof.completion_id.value.as_str()) {
            return Err(TrustPortalError::ProofCompletionMismatch);
        }
        proofs_by_dsr
            .entry(proof.dsr_id.value.as_str())
            .or_default()
            .insert(proof.proof_id.value.clone());
    }
    for queue_entry in completed_queue_by_dsr.values() {
        let expected = queue_entry
            .completion_proof_ids
            .value
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let actual = proofs_by_dsr
            .get(queue_entry.dsr_id.value.as_str())
            .cloned()
            .unwrap_or_default();
        if expected != actual {
            return Err(TrustPortalError::MissingProofArchiveEntry);
        }
    }
    Ok(())
}

fn validate_generated_at(input: &TrustPortalTenantSnapshotCreate) -> Result<(), TrustPortalError> {
    for entry in &input.lineage_entries {
        validate_time_order(
            entry.updated_at_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    for entry in &input.dsr_queue_entries {
        validate_time_order(
            entry.updated_at_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    for entry in &input.proof_archive_entries {
        validate_time_order(
            entry.published_at_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    for entry in &input.api_stability_entries {
        validate_time_order(
            entry.changed_at_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    for entry in &input.axis_slo_entries {
        validate_time_order(
            entry.window_end_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    for entry in &input.override_pack_entries {
        validate_time_order(
            entry.reviewed_at_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    for entry in &input.consent_receipt_entries {
        validate_time_order(
            entry.granted_at_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    for entry in &input.subprocessor_entries {
        validate_time_order(
            entry.notice_sent_at_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    validate_time_order(
        input.residency_declaration.declared_at_epoch_seconds.value,
        input.generated_at_epoch_seconds,
    )?;
    for entry in &input.plugin_trust_entries {
        validate_time_order(
            entry.reviewed_at_epoch_seconds.value,
            input.generated_at_epoch_seconds,
        )?;
    }
    Ok(())
}

fn validate_data_classes(data_classes: &[PrivacyDataClass]) -> Result<(), TrustPortalError> {
    if data_classes.is_empty() {
        return Err(TrustPortalError::EmptyResidencyDataClassSet);
    }
    let mut seen = BTreeSet::new();
    for data_class in data_classes {
        if !seen.insert(*data_class) {
            return Err(TrustPortalError::DuplicateDataClass);
        }
    }
    Ok(())
}

fn validate_unique<'a>(
    values: impl Iterator<Item = &'a str>,
    duplicate_error: TrustPortalError,
) -> Result<(), TrustPortalError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(duplicate_error);
        }
    }
    Ok(())
}

fn validate_non_empty(value: &str, error: TrustPortalError) -> Result<(), TrustPortalError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_optional_non_empty(
    value: Option<&str>,
    error: TrustPortalError,
) -> Result<(), TrustPortalError> {
    match value {
        Some(value) => validate_non_empty(value, error),
        None => Ok(()),
    }
}

fn validate_hash(value: &str, error: TrustPortalError) -> Result<(), TrustPortalError> {
    if value.starts_with(SHA256_PREFIX) && value.len() > SHA256_PREFIX.len() {
        Ok(())
    } else {
        Err(error)
    }
}

fn validate_time_order(first: u64, second: u64) -> Result<(), TrustPortalError> {
    if first <= second {
        Ok(())
    } else {
        Err(TrustPortalError::InvalidTimeOrder)
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, internal_data_class())
}

fn subject<T>(value: T) -> Classified<T> {
    Classified::new(value, default_trust_portal_subject_data_class())
}

fn internal_data_class() -> PrivacyDataClass {
    // ADR-0083 Tier 1: use the infallible kernel constructor; the previous
    // `.expect()` proved a statically known invariant that the kernel now
    // encodes at the type level.
    PrivacyDataClass::internal_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use compliance_dsr::{
        DsrAckStatus, DsrCascadeAck, DsrCascadeAckCreate, DsrCompletionRecordCreate, DsrDispatch,
        DsrDispatchCreate, DsrProofMethod, DsrRequestCreate, DsrSlaTier, DsrStoreRef,
        DsrStoreRefCreate, ErasureProofCreate,
    };

    fn privacy(data_class: DataClass) -> PrivacyDataClass {
        PrivacyDataClass::new(data_class).expect("test fixture uses privacy class")
    }

    fn cross_region_residency_class() -> ResidencyClass {
        ResidencyClass::HomeWithRecoveryFailover
    }

    fn strict_residency_class() -> ResidencyClass {
        ResidencyClass::StrictHomeRegion
    }

    fn platform_dsr_bundle() -> (DsrRequest, DsrCompletionRecord, ErasureProof) {
        let subject_class = privacy(DataClass::PiiIdentifying);
        let request = DsrRequest::new(DsrRequestCreate {
            dsr_id: "dsr-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            region: "region-alpha1".to_string(),
            subject_ref: "subject-1".to_string(),
            action: DsrAction::Erase,
            sla_tier: DsrSlaTier::Ga,
            data_classes: vec![subject_class],
            received_at_epoch_seconds: 100,
            deadline_epoch_seconds: 100 + DsrSlaTier::Ga.max_seconds(),
        })
        .expect("request fixture is valid");
        let store = DsrStoreRef::new(DsrStoreRefCreate {
            axis: DsrAxis::Workspace,
            kind: DsrStoreKind::WorkspaceObject,
            store_id: "mail-store".to_string(),
            tenant_id: "tenant-1".to_string(),
            region: "region-alpha1".to_string(),
            cell_id: "cell-1".to_string(),
            record_ref: "mail/message-1".to_string(),
            data_class: subject_class,
        })
        .expect("store fixture is valid");
        let dispatch = DsrDispatch::new(
            DsrDispatchCreate {
                dispatch_id: "dispatch-1".to_string(),
                idempotency_key: "idem-1".to_string(),
                store: store.clone(),
                dispatched_at_epoch_seconds: 120,
            },
            &request,
        )
        .expect("dispatch fixture is valid");
        let proof = ErasureProof::new(
            ErasureProofCreate {
                proof_id: "proof-1".to_string(),
                dispatch_id: "dispatch-1".to_string(),
                dsr_id: "dsr-1".to_string(),
                action: DsrAction::Erase,
                store,
                method: DsrProofMethod::RecordDelete,
                evidence_hash: "sha256:proof1".to_string(),
                witness_ref: "witness/audit-chain/1".to_string(),
                signer_ref: "kms/cosign/workspace".to_string(),
                signature_ref: "sigstore/cosign/proof-1".to_string(),
                rekor_log_index: 42,
                proved_at_epoch_seconds: 140,
            },
            &dispatch,
        )
        .expect("proof fixture is valid");
        let ack = DsrCascadeAck::new(
            DsrCascadeAckCreate {
                ack_id: "ack-1".to_string(),
                dispatch_id: "dispatch-1".to_string(),
                dsr_id: "dsr-1".to_string(),
                status: DsrAckStatus::Completed,
                reason: None,
                proof_id: Some("proof-1".to_string()),
                evidence_hash: Some("sha256:proof1".to_string()),
                acknowledged_at_epoch_seconds: 150,
            },
            &dispatch,
            Some(&proof),
        )
        .expect("ack fixture is valid");
        let completion = DsrCompletionRecord::new(
            DsrCompletionRecordCreate {
                completion_id: "completion-1".to_string(),
                dsr_id: "dsr-1".to_string(),
                dispatches: vec![dispatch],
                acks: vec![ack],
                proofs: vec![proof.clone()],
                aggregate_proof_hash: "sha256:aggregate".to_string(),
                signer_ref: "kms/cosign/workspace".to_string(),
                signature_ref: "sigstore/cosign/completion-1".to_string(),
                rekor_log_index: 43,
                completed_at_epoch_seconds: 160,
            },
            &request,
        )
        .expect("completion fixture is valid");
        (request, completion, proof)
    }

    fn completed_queue_entry() -> TrustPortalDsrQueueEntry {
        let (request, completion, _) = platform_dsr_bundle();
        TrustPortalDsrQueueEntry::new(TrustPortalDsrQueueEntryCreate {
            queue_entry_id: "queue-1".to_string(),
            request,
            status: TrustPortalDsrQueueStatus::Completed,
            completion: Some(completion),
            updated_at_epoch_seconds: 170,
        })
        .expect("queue entry fixture is valid")
    }

    fn proof_archive_entry() -> TrustPortalProofArchiveEntry {
        let (_, completion, proof) = platform_dsr_bundle();
        TrustPortalProofArchiveEntry::new(TrustPortalProofArchiveEntryCreate {
            archive_entry_id: "archive-1".to_string(),
            proof,
            completion_id: completion.completion_id.value,
            downloadable_ref: "trust-portal/proofs/proof-1.json".to_string(),
            published_at_epoch_seconds: 175,
        })
        .expect("proof archive fixture is valid")
    }

    fn lineage_entry() -> TrustPortalLineageEntry {
        TrustPortalLineageEntry::new(TrustPortalLineageEntryCreate {
            lineage_id: "lineage-1".to_string(),
            data_class: privacy(DataClass::PiiIdentifying),
            axis: DsrAxis::Workspace,
            store_kind: DsrStoreKind::WorkspaceObject,
            store_id: "mail-store".to_string(),
            region: "region-alpha1".to_string(),
            flow_ref: "lineage/flow/mail-to-retention".to_string(),
            updated_at_epoch_seconds: 180,
        })
        .expect("lineage fixture is valid")
    }

    fn api_entry() -> TrustPortalApiStabilityEntry {
        TrustPortalApiStabilityEntry::new(TrustPortalApiStabilityEntryCreate {
            api_id: "retention-dsr-v1".to_string(),
            api_name: "Workspace DSR API".to_string(),
            tier: ApiStabilityTier::Preview,
            deprecation_status: ApiDeprecationStatus::Current,
            sunset_at_epoch_seconds: None,
            usage_summary_ref: Some("usage/retention-dsr-v1".to_string()),
            changed_at_epoch_seconds: 181,
        })
        .expect("api fixture is valid")
    }

    fn slo_entry() -> TrustPortalAxisSloEntry {
        TrustPortalAxisSloEntry::new(TrustPortalAxisSloEntryCreate {
            axis: DsrAxis::Workspace,
            availability_basis_points: 9_999,
            window_start_epoch_seconds: 1,
            window_end_epoch_seconds: 182,
            last_incident_ref: None,
        })
        .expect("slo fixture is valid")
    }

    fn override_entry() -> TrustPortalOverridePackEntry {
        TrustPortalOverridePackEntry::new(TrustPortalOverridePackEntryCreate {
            pack_id: "pack-alpha-healthcare".to_string(),
            jurisdiction: "JURISDICTION_ALPHA".to_string(),
            data_class: privacy(DataClass::SensitivePipaArticle23),
            denied_purposes: vec!["ads-targeting".to_string()],
            reviewed_at_epoch_seconds: 183,
        })
        .expect("override fixture is valid")
    }

    fn consent_entry() -> TrustPortalConsentReceiptEntry {
        TrustPortalConsentReceiptEntry::new(TrustPortalConsentReceiptEntryCreate {
            receipt_id: "receipt-1".to_string(),
            purpose_ref: "cross-region-dr".to_string(),
            data_class: privacy(DataClass::PiiIdentifying),
            granted_at_epoch_seconds: 184,
            revoked_at_epoch_seconds: None,
            evidence_ref: "consent/receipt-1".to_string(),
        })
        .expect("consent fixture is valid")
    }

    fn subprocessor_entry() -> TrustPortalSubprocessorEntry {
        TrustPortalSubprocessorEntry::new(TrustPortalSubprocessorEntryCreate {
            processor_id: "processor-1".to_string(),
            legal_name: "Region Alpha Operator".to_string(),
            region: "region-alpha1".to_string(),
            purpose_ref: "workspace-hosting".to_string(),
            data_classes: vec![privacy(DataClass::PiiIdentifying)],
            notice_ref: "notice/subprocessor-1".to_string(),
            notice_sent_at_epoch_seconds: 185,
            effective_at_epoch_seconds: 186,
        })
        .expect("subprocessor fixture is valid")
    }

    fn residency_declaration() -> TrustPortalResidencyDeclaration {
        TrustPortalResidencyDeclaration::new(TrustPortalResidencyDeclarationCreate {
            residency_class: ResidencyClass::Global,
            primary_region: "region-alpha1".to_string(),
            data_classes: vec![privacy(DataClass::PiiIdentifying)],
            cross_region_allowed: true,
            cross_region_regions: vec!["region-beta1".to_string()],
            consent_receipt_refs: vec!["receipt-1".to_string()],
            evidence_ref: "residency/evidence-1".to_string(),
            declared_at_epoch_seconds: 187,
        })
        .expect("residency fixture is valid")
    }

    fn plugin_entry() -> TrustPortalPluginTrustEntry {
        TrustPortalPluginTrustEntry::new(TrustPortalPluginTrustEntryCreate {
            plugin_id: "plugin-1".to_string(),
            tier: PluginTrustTier::Reviewed,
            signature_ref: Some("sigstore/plugin-1".to_string()),
            sandbox_profile_ref: "wasm/sandbox/reviewed".to_string(),
            reviewed_at_epoch_seconds: 188,
        })
        .expect("plugin fixture is valid")
    }

    #[derive(Clone, Copy)]
    struct SectionCounts {
        lineage: u32,
        queue: u32,
        proof: u32,
        api: u32,
        slo: u32,
        override_pack: u32,
        consent: u32,
        subprocessor: u32,
        plugin: u32,
    }

    fn complete_section_counts() -> SectionCounts {
        SectionCounts {
            lineage: 1,
            queue: 1,
            proof: 1,
            api: 1,
            slo: 1,
            override_pack: 1,
            consent: 1,
            subprocessor: 1,
            plugin: 1,
        }
    }

    fn section_summaries(counts: SectionCounts) -> Vec<TrustPortalSectionSummary> {
        [
            (TrustPortalSection::LineageView, counts.lineage),
            (TrustPortalSection::DsrQueue, counts.queue),
            (TrustPortalSection::ProofArchive, counts.proof),
            (TrustPortalSection::ApiStabilityMirror, counts.api),
            (TrustPortalSection::AxisSlaAndUptime, counts.slo),
            (TrustPortalSection::OverridePackView, counts.override_pack),
            (TrustPortalSection::ConsentReceiptArchive, counts.consent),
            (TrustPortalSection::SubprocessorList, counts.subprocessor),
            (TrustPortalSection::ResidencyDeclaration, 1),
            (TrustPortalSection::PluginTrustTierMatrix, counts.plugin),
        ]
        .into_iter()
        .map(|(section, count)| {
            TrustPortalSectionSummary::new(TrustPortalSectionSummaryCreate {
                section,
                published_record_count: count,
                refreshed_at_epoch_seconds: 190,
                evidence_ref: format!("trust-portal/section/{section:?}"),
            })
            .expect("section summary fixture is valid")
        })
        .collect()
    }

    fn snapshot_create() -> TrustPortalTenantSnapshotCreate {
        TrustPortalTenantSnapshotCreate {
            snapshot_id: "snapshot-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            region: "region-alpha1".to_string(),
            section_summaries: section_summaries(complete_section_counts()),
            lineage_entries: vec![lineage_entry()],
            dsr_queue_entries: vec![completed_queue_entry()],
            proof_archive_entries: vec![proof_archive_entry()],
            api_stability_entries: vec![api_entry()],
            axis_slo_entries: vec![slo_entry()],
            override_pack_entries: vec![override_entry()],
            consent_receipt_entries: vec![consent_entry()],
            subprocessor_entries: vec![subprocessor_entry()],
            residency_declaration: residency_declaration(),
            plugin_trust_entries: vec![plugin_entry()],
            generated_at_epoch_seconds: 200,
        }
    }

    #[test]
    fn snapshot_requires_all_sections_and_exact_dsr_proof_coverage() {
        let snapshot = TrustPortalTenantSnapshot::new(snapshot_create())
            .expect("complete trust portal snapshot should build");
        assert_eq!(
            snapshot.section_summaries.value.len(),
            TRUST_PORTAL_SECTIONS.len()
        );
        assert_eq!(
            snapshot.dsr_queue_entries.value[0].sla_status.value,
            DsrSlaStatus::WithinSla
        );
        assert_eq!(
            snapshot.schema_version.value,
            TRUST_PORTAL_SNAPSHOT_SCHEMA_VERSION
        );
    }

    #[test]
    fn completed_dsr_queue_entry_requires_completion_record() {
        let (request, _, _) = platform_dsr_bundle();
        let error = TrustPortalDsrQueueEntry::new(TrustPortalDsrQueueEntryCreate {
            queue_entry_id: "queue-1".to_string(),
            request,
            status: TrustPortalDsrQueueStatus::Completed,
            completion: None,
            updated_at_epoch_seconds: 170,
        })
        .expect_err("completed queue entries require completion");
        assert_eq!(error, TrustPortalError::MissingDsrCompletion);
    }

    #[test]
    fn completed_dsr_requires_published_proof_archive_coverage() {
        let mut input = snapshot_create();
        input.proof_archive_entries = Vec::new();
        input.section_summaries = section_summaries(SectionCounts {
            proof: 0,
            ..complete_section_counts()
        });
        let error = TrustPortalTenantSnapshot::new(input)
            .expect_err("completed DSR needs proof archive entries");
        assert_eq!(error, TrustPortalError::MissingProofArchiveEntry);
    }

    #[test]
    fn api_deprecation_status_requires_sunset_date() {
        let error = TrustPortalApiStabilityEntry::new(TrustPortalApiStabilityEntryCreate {
            api_id: "retention-dsr-v1".to_string(),
            api_name: "Workspace DSR API".to_string(),
            tier: ApiStabilityTier::Stable,
            deprecation_status: ApiDeprecationStatus::Deprecated,
            sunset_at_epoch_seconds: None,
            usage_summary_ref: Some("usage/retention-dsr-v1".to_string()),
            changed_at_epoch_seconds: 181,
        })
        .expect_err("deprecated APIs require a sunset date");
        assert_eq!(error, TrustPortalError::InvalidApiSunset);
    }

    #[test]
    fn slo_entries_bound_availability_and_window_order() {
        let availability_error = TrustPortalAxisSloEntry::new(TrustPortalAxisSloEntryCreate {
            axis: DsrAxis::Workspace,
            availability_basis_points: 10_001,
            window_start_epoch_seconds: 1,
            window_end_epoch_seconds: 2,
            last_incident_ref: None,
        })
        .expect_err("availability cannot exceed one hundred percent");
        assert_eq!(availability_error, TrustPortalError::InvalidAvailability);

        let window_error = TrustPortalAxisSloEntry::new(TrustPortalAxisSloEntryCreate {
            axis: DsrAxis::Workspace,
            availability_basis_points: 9_999,
            window_start_epoch_seconds: 2,
            window_end_epoch_seconds: 2,
            last_incident_ref: None,
        })
        .expect_err("SLO windows need positive duration");
        assert_eq!(window_error, TrustPortalError::InvalidSloWindow);
    }

    #[test]
    fn snapshot_rejects_duplicate_customer_visible_ids() {
        let mut input = snapshot_create();
        input.consent_receipt_entries.push(consent_entry());
        input.section_summaries = section_summaries(SectionCounts {
            consent: 2,
            ..complete_section_counts()
        });
        let error = TrustPortalTenantSnapshot::new(input)
            .expect_err("duplicate consent receipt IDs are rejected");
        assert_eq!(error, TrustPortalError::DuplicateReceiptId);
    }

    #[test]
    fn residency_declaration_requires_consent_for_cross_region_replication() {
        let error = TrustPortalResidencyDeclaration::new(TrustPortalResidencyDeclarationCreate {
            residency_class: ResidencyClass::Global,
            primary_region: "region-alpha1".to_string(),
            data_classes: vec![privacy(DataClass::PiiIdentifying)],
            cross_region_allowed: true,
            cross_region_regions: vec!["region-beta1".to_string()],
            consent_receipt_refs: Vec::new(),
            evidence_ref: "residency/evidence-1".to_string(),
            declared_at_epoch_seconds: 187,
        })
        .expect_err("cross-region replication requires consent references");
        assert_eq!(error, TrustPortalError::ResidencyCrossRegionMismatch);

        let disabled_cross_region_error =
            TrustPortalResidencyDeclaration::new(TrustPortalResidencyDeclarationCreate {
                residency_class: ResidencyClass::Global,
                primary_region: "region-alpha1".to_string(),
                data_classes: vec![privacy(DataClass::PiiIdentifying)],
                cross_region_allowed: false,
                cross_region_regions: vec!["region-beta1".to_string()],
                consent_receipt_refs: vec!["receipt-1".to_string()],
                evidence_ref: "residency/evidence-1".to_string(),
                declared_at_epoch_seconds: 187,
            })
            .expect_err("disabled cross-region declarations reject replica metadata");
        assert_eq!(
            disabled_cross_region_error,
            TrustPortalError::ResidencyCrossRegionMismatch
        );
    }

    #[test]
    fn legacy_operational_data_class_is_not_accepted_as_privacy_scope() {
        let error = trust_portal_data_class_from_legacy(DataClass::Audit)
            .expect_err("operational labels cannot enter privacy-scoped portal records");
        assert_eq!(error, TrustPortalError::InvalidDataClass);
    }
}
