//! Workspace e-discovery export kernel.
//!
//! Typed kernel records for the W-Workspace-GA e-discovery export surface named
//! by `docs/products/workspace/PRD.md`. The kernel owns e-discovery request
//! scope, package manifests, retention export-decision binding, and signed proof
//! references. Surface readers, renderers, storage, audit emission, and trust
//! portal UI remain outside this crate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use compliance_retention::{
    RetentionDecision, RetentionDecisionOutcome, RetentionDisposition, RetentionHorizon,
    RetentionLawfulBasis, RetentionPolicy, RetentionPolicyCreate, RetentionRequestKind,
    WorkspaceRetentionSurface,
};

const EDISCOVERY_REQUEST_SCHEMA_VERSION: u32 = 1;
const EDISCOVERY_ITEM_SCHEMA_VERSION: u32 = 1;
const EDISCOVERY_PACKAGE_SCHEMA_VERSION: u32 = 1;
const EDISCOVERY_PROOF_SCHEMA_VERSION: u32 = 1;
const MIN_PACKAGE_BYTES: u64 = 1;
const MIN_ITEM_BYTES: u64 = 1;
const SHA256_PREFIX: &str = "sha256:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EdiscoveryError {
    InvalidRequestId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidActorRef,
    InvalidMatterRef,
    InvalidPurposeRef,
    InvalidConsentReceiptRef,
    CrossRegionConsentRequired,
    UnauthorizedActorRole,
    EmptySurfaceSet,
    DuplicateSurface,
    EmptyDataClassSet,
    DuplicateDataClass,
    InvalidItemId,
    InvalidSourceRef,
    UnsupportedExportFormat,
    InvalidRetentionDecisionId,
    InvalidEvidenceHash,
    EmptyItemBytes,
    EmptyItemSet,
    DuplicateItemId,
    DuplicateSourceRef,
    ItemSurfaceOutOfScope,
    ItemDataClassOutOfScope,
    MissingRetentionDecision,
    RetentionDecisionNotExportOnly,
    RetentionDecisionMismatch,
    InvalidPackageId,
    InvalidManifestHash,
    InvalidPackageHash,
    InvalidEncryptionKeyRef,
    EmptyPackageBytes,
    InvalidProofId,
    InvalidSignerRef,
    InvalidSignatureRef,
    ProofItemHashMismatch,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EdiscoveryActorRole {
    DataSubject,
    TenantAdmin,
    Dpo,
    Auditor,
    LegalCounsel,
    RegulatorDelegate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EdiscoveryLegalBasis {
    DsrExport,
    Litigation,
    RegulatoryInquiry,
    InternalInvestigation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EdiscoveryExportFormat {
    Jsonl,
    Csv,
    Pdf,
    Eml,
    Mbox,
    Ics,
    Docx,
    Xlsx,
    Pptx,
    Hwpx,
    Html,
    NativeZip,
    Mp4,
    TranscriptJson,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdiscoveryRequestCreate {
    pub request_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub region: String,                           // data_class: INTERNAL_ONLY
    pub destination_region: String,               // data_class: INTERNAL_ONLY
    pub cell_id: String,                          // data_class: INTERNAL_ONLY
    pub actor_ref: String,                        // data_class: PII_IDENTIFYING
    pub actor_role: EdiscoveryActorRole,          // data_class: INTERNAL_ONLY
    pub legal_basis: EdiscoveryLegalBasis,        // data_class: INTERNAL_ONLY
    pub matter_ref: String,                       // data_class: INTERNAL_ONLY
    pub purpose_ref: String,                      // data_class: INTERNAL_ONLY
    pub consent_receipt_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub surfaces: Vec<WorkspaceRetentionSurface>, // data_class: INTERNAL_ONLY
    pub data_classes: Vec<PrivacyDataClass>,      // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdiscoveryRequest {
    pub request_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<String>,     // data_class: INTERNAL_ONLY
    pub destination_region: Classified<String>, // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub actor_ref: Classified<String>,  // data_class: PII_IDENTIFYING
    pub actor_role: Classified<EdiscoveryActorRole>, // data_class: INTERNAL_ONLY
    pub legal_basis: Classified<EdiscoveryLegalBasis>, // data_class: INTERNAL_ONLY
    pub matter_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub purpose_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub consent_receipt_ref: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub surfaces: Classified<Vec<WorkspaceRetentionSurface>>, // data_class: INTERNAL_ONLY
    pub data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdiscoveryExportItemCreate {
    pub item_id: String,                       // data_class: INTERNAL_ONLY
    pub source_ref: String,                    // data_class: INTERNAL_ONLY
    pub surface: WorkspaceRetentionSurface,    // data_class: INTERNAL_ONLY
    pub data_class: PrivacyDataClass,          // data_class: INTERNAL_ONLY
    pub export_format: EdiscoveryExportFormat, // data_class: INTERNAL_ONLY
    pub retention_decision_id: String,         // data_class: INTERNAL_ONLY
    pub evidence_hash: String,                 // data_class: INTERNAL_ONLY
    pub byte_len: u64,                         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EdiscoveryExportItem {
    pub item_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub source_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub surface: Classified<WorkspaceRetentionSurface>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub export_format: Classified<EdiscoveryExportFormat>, // data_class: INTERNAL_ONLY
    pub retention_decision_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub evidence_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,      // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdiscoveryExportPackageCreate {
    pub package_id: String,                // data_class: INTERNAL_ONLY
    pub items: Vec<EdiscoveryExportItem>,  // data_class: INTERNAL_ONLY
    pub manifest_hash: String,             // data_class: INTERNAL_ONLY
    pub package_sha256: String,            // data_class: INTERNAL_ONLY
    pub encrypted_package_key_ref: String, // data_class: SECRET
    pub byte_len: u64,                     // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdiscoveryExportPackage {
    pub package_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub request_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<String>,     // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub items: Classified<Vec<EdiscoveryExportItem>>, // data_class: INTERNAL_ONLY
    pub manifest_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub package_sha256: Classified<String>, // data_class: INTERNAL_ONLY
    pub encrypted_package_key_ref: Classified<String>, // data_class: SECRET
    pub byte_len: Classified<u64>,      // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdiscoveryExportProofCreate {
    pub proof_id: String,                  // data_class: INTERNAL_ONLY
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub package_id: String,                // data_class: INTERNAL_ONLY
    pub manifest_hash: String,             // data_class: INTERNAL_ONLY
    pub package_sha256: String,            // data_class: INTERNAL_ONLY
    pub item_evidence_hashes: Vec<String>, // data_class: INTERNAL_ONLY
    pub signer_ref: String,                // data_class: INTERNAL_ONLY
    pub signature_ref: String,             // data_class: INTERNAL_ONLY
    pub rekor_log_index: u64,              // data_class: INTERNAL_ONLY
    pub signed_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdiscoveryExportProof {
    pub proof_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub request_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub package_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub manifest_hash: Classified<String>,  // data_class: INTERNAL_ONLY
    pub package_sha256: Classified<String>, // data_class: INTERNAL_ONLY
    pub item_evidence_hashes: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub signer_ref: Classified<String>,     // data_class: INTERNAL_ONLY
    pub signature_ref: Classified<String>,  // data_class: INTERNAL_ONLY
    pub rekor_log_index: Classified<u64>,   // data_class: INTERNAL_ONLY
    pub signed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,    // data_class: INTERNAL_ONLY
}

pub trait EdiscoveryExporter {
    fn export(
        &self,
        request: &EdiscoveryRequest,
        retention_decisions: &[RetentionDecision],
    ) -> Result<EdiscoveryExportPackage, EdiscoveryError>;
}

impl EdiscoveryRequest {
    pub fn new(input: EdiscoveryRequestCreate) -> Result<Self, EdiscoveryError> {
        validate_non_empty(&input.request_id, EdiscoveryError::InvalidRequestId)?;
        validate_non_empty(&input.tenant_id, EdiscoveryError::InvalidTenantId)?;
        validate_non_empty(&input.region, EdiscoveryError::InvalidRegion)?;
        validate_non_empty(&input.destination_region, EdiscoveryError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, EdiscoveryError::InvalidCellId)?;
        validate_non_empty(&input.actor_ref, EdiscoveryError::InvalidActorRef)?;
        validate_non_empty(&input.matter_ref, EdiscoveryError::InvalidMatterRef)?;
        validate_non_empty(&input.purpose_ref, EdiscoveryError::InvalidPurposeRef)?;
        validate_optional_non_empty(
            input.consent_receipt_ref.as_deref(),
            EdiscoveryError::InvalidConsentReceiptRef,
        )?;
        validate_actor_authorized(input.actor_role, input.legal_basis)?;
        validate_surfaces(&input.surfaces)?;
        validate_data_classes(&input.data_classes)?;
        validate_cross_region_consent(
            &input.region,
            &input.destination_region,
            input.consent_receipt_ref.as_deref(),
        )?;

        Ok(Self {
            request_id: internal(input.request_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            destination_region: internal(input.destination_region),
            cell_id: internal(input.cell_id),
            actor_ref: Classified::new(input.actor_ref, actor_data_class()),
            actor_role: internal(input.actor_role),
            legal_basis: internal(input.legal_basis),
            matter_ref: internal(input.matter_ref),
            purpose_ref: internal(input.purpose_ref),
            consent_receipt_ref: internal(input.consent_receipt_ref),
            surfaces: internal(input.surfaces),
            data_classes: internal(input.data_classes),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            schema_version: internal(EDISCOVERY_REQUEST_SCHEMA_VERSION),
        })
    }
}

impl EdiscoveryExportItem {
    pub fn new(input: EdiscoveryExportItemCreate) -> Result<Self, EdiscoveryError> {
        validate_non_empty(&input.item_id, EdiscoveryError::InvalidItemId)?;
        validate_non_empty(&input.source_ref, EdiscoveryError::InvalidSourceRef)?;
        validate_non_empty(
            &input.retention_decision_id,
            EdiscoveryError::InvalidRetentionDecisionId,
        )?;
        validate_hash(&input.evidence_hash, EdiscoveryError::InvalidEvidenceHash)?;
        if input.byte_len < MIN_ITEM_BYTES {
            return Err(EdiscoveryError::EmptyItemBytes);
        }
        if !format_supported_for(input.surface, input.export_format) {
            return Err(EdiscoveryError::UnsupportedExportFormat);
        }

        Ok(Self {
            item_id: internal(input.item_id),
            source_ref: internal(input.source_ref),
            surface: internal(input.surface),
            data_class: internal(input.data_class),
            export_format: internal(input.export_format),
            retention_decision_id: internal(input.retention_decision_id),
            evidence_hash: internal(input.evidence_hash),
            byte_len: internal(input.byte_len),
            schema_version: internal(EDISCOVERY_ITEM_SCHEMA_VERSION),
        })
    }
}

impl EdiscoveryExportPackage {
    pub fn new(
        input: EdiscoveryExportPackageCreate,
        request: &EdiscoveryRequest,
        retention_decisions: &[RetentionDecision],
    ) -> Result<Self, EdiscoveryError> {
        validate_non_empty(&input.package_id, EdiscoveryError::InvalidPackageId)?;
        validate_hash(&input.manifest_hash, EdiscoveryError::InvalidManifestHash)?;
        validate_hash(&input.package_sha256, EdiscoveryError::InvalidPackageHash)?;
        validate_non_empty(
            &input.encrypted_package_key_ref,
            EdiscoveryError::InvalidEncryptionKeyRef,
        )?;
        if input.byte_len < MIN_PACKAGE_BYTES {
            return Err(EdiscoveryError::EmptyPackageBytes);
        }
        validate_time_order(
            input.created_at_epoch_seconds,
            input.completed_at_epoch_seconds,
        )?;
        validate_time_order(
            request.requested_at_epoch_seconds.value,
            input.completed_at_epoch_seconds,
        )?;
        validate_items(&input.items, request, retention_decisions)?;

        Ok(Self {
            package_id: internal(input.package_id),
            request_id: internal(request.request_id.value.clone()),
            tenant_id: internal(request.tenant_id.value.clone()),
            region: internal(request.region.value.clone()),
            cell_id: internal(request.cell_id.value.clone()),
            items: internal(input.items),
            manifest_hash: internal(input.manifest_hash),
            package_sha256: internal(input.package_sha256),
            encrypted_package_key_ref: Classified::new(
                input.encrypted_package_key_ref,
                DataClass::Secret,
            ),
            byte_len: internal(input.byte_len),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            completed_at_epoch_seconds: internal(input.completed_at_epoch_seconds),
            schema_version: internal(EDISCOVERY_PACKAGE_SCHEMA_VERSION),
        })
    }
}

impl EdiscoveryExportProof {
    pub fn new(
        input: EdiscoveryExportProofCreate,
        package: &EdiscoveryExportPackage,
    ) -> Result<Self, EdiscoveryError> {
        validate_non_empty(&input.proof_id, EdiscoveryError::InvalidProofId)?;
        validate_non_empty(&input.request_id, EdiscoveryError::InvalidRequestId)?;
        validate_non_empty(&input.package_id, EdiscoveryError::InvalidPackageId)?;
        validate_hash(&input.manifest_hash, EdiscoveryError::InvalidManifestHash)?;
        validate_hash(&input.package_sha256, EdiscoveryError::InvalidPackageHash)?;
        validate_non_empty(&input.signer_ref, EdiscoveryError::InvalidSignerRef)?;
        validate_non_empty(&input.signature_ref, EdiscoveryError::InvalidSignatureRef)?;
        validate_time_order(
            package.completed_at_epoch_seconds.value,
            input.signed_at_epoch_seconds,
        )?;
        validate_proof_matches_package(&input, package)?;

        Ok(Self {
            proof_id: internal(input.proof_id),
            request_id: internal(input.request_id),
            package_id: internal(input.package_id),
            manifest_hash: internal(input.manifest_hash),
            package_sha256: internal(input.package_sha256),
            item_evidence_hashes: internal(input.item_evidence_hashes),
            signer_ref: internal(input.signer_ref),
            signature_ref: internal(input.signature_ref),
            rekor_log_index: internal(input.rekor_log_index),
            signed_at_epoch_seconds: internal(input.signed_at_epoch_seconds),
            schema_version: internal(EDISCOVERY_PROOF_SCHEMA_VERSION),
        })
    }
}

pub fn default_workspace_ediscovery_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn actor_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_ediscovery_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, EdiscoveryError> {
    PrivacyDataClass::new(data_class).map_err(|_| EdiscoveryError::InvalidDataClass)
}

pub fn retention_policy_for_export(
    policy_id: String,
    tenant_id: String,
    region: String,
    surface: WorkspaceRetentionSurface,
    effective_at_epoch_seconds: u64,
) -> Result<RetentionPolicy, EdiscoveryError> {
    RetentionPolicy::new(RetentionPolicyCreate {
        policy_id,
        tenant_id,
        region,
        surface,
        horizon: RetentionHorizon::Indefinite,
        lawful_basis: RetentionLawfulBasis::LegalObligation,
        disposition: RetentionDisposition::KmsShred,
        effective_at_epoch_seconds,
        created_at_epoch_seconds: effective_at_epoch_seconds,
        updated_at_epoch_seconds: effective_at_epoch_seconds,
    })
    .map_err(|_| EdiscoveryError::RetentionDecisionMismatch)
}

fn validate_items(
    items: &[EdiscoveryExportItem],
    request: &EdiscoveryRequest,
    retention_decisions: &[RetentionDecision],
) -> Result<(), EdiscoveryError> {
    if items.is_empty() {
        return Err(EdiscoveryError::EmptyItemSet);
    }

    let mut item_ids = BTreeSet::new();
    let mut source_refs = BTreeSet::new();
    let mut used_decision_ids = BTreeSet::new();
    let decision_by_id = retention_decisions
        .iter()
        .map(|decision| (decision.decision_id.value.as_str(), decision))
        .collect::<BTreeMap<_, _>>();

    for item in items {
        if !item_ids.insert(item.item_id.value.clone()) {
            return Err(EdiscoveryError::DuplicateItemId);
        }
        if !source_refs.insert(item.source_ref.value.clone()) {
            return Err(EdiscoveryError::DuplicateSourceRef);
        }
        if !request.surfaces.value.contains(&item.surface.value) {
            return Err(EdiscoveryError::ItemSurfaceOutOfScope);
        }
        if !request.data_classes.value.contains(&item.data_class.value) {
            return Err(EdiscoveryError::ItemDataClassOutOfScope);
        }
        let Some(decision) = decision_by_id.get(item.retention_decision_id.value.as_str()) else {
            return Err(EdiscoveryError::MissingRetentionDecision);
        };
        validate_export_decision(item, request, decision)?;
        used_decision_ids.insert(item.retention_decision_id.value.clone());
    }

    if used_decision_ids.len() != retention_decisions.len() {
        return Err(EdiscoveryError::RetentionDecisionMismatch);
    }
    Ok(())
}

fn validate_export_decision(
    item: &EdiscoveryExportItem,
    request: &EdiscoveryRequest,
    decision: &RetentionDecision,
) -> Result<(), EdiscoveryError> {
    if decision.outcome.value != RetentionDecisionOutcome::ExportOnly
        || decision.erase_method.value.is_some()
        || decision.request_kind.value != RetentionRequestKind::DsrExport
    {
        return Err(EdiscoveryError::RetentionDecisionNotExportOnly);
    }
    if decision.request_id.value != request.request_id.value
        || decision.tenant_id.value != request.tenant_id.value
        || decision.region.value != request.region.value
        || decision.surface.value != item.surface.value
        || decision.record_id.value != item.source_ref.value
    {
        return Err(EdiscoveryError::RetentionDecisionMismatch);
    }
    Ok(())
}

fn validate_proof_matches_package(
    input: &EdiscoveryExportProofCreate,
    package: &EdiscoveryExportPackage,
) -> Result<(), EdiscoveryError> {
    if input.request_id != package.request_id.value
        || input.package_id != package.package_id.value
        || input.manifest_hash != package.manifest_hash.value
        || input.package_sha256 != package.package_sha256.value
    {
        return Err(EdiscoveryError::ProofItemHashMismatch);
    }

    let mut expected = BTreeSet::new();
    for item in &package.items.value {
        expected.insert(item.evidence_hash.value.clone());
    }
    let mut actual = BTreeSet::new();
    for hash in &input.item_evidence_hashes {
        validate_hash(hash, EdiscoveryError::InvalidEvidenceHash)?;
        if !actual.insert(hash.clone()) {
            return Err(EdiscoveryError::ProofItemHashMismatch);
        }
    }
    if actual != expected {
        return Err(EdiscoveryError::ProofItemHashMismatch);
    }
    Ok(())
}

fn validate_actor_authorized(
    role: EdiscoveryActorRole,
    legal_basis: EdiscoveryLegalBasis,
) -> Result<(), EdiscoveryError> {
    let authorized = match legal_basis {
        EdiscoveryLegalBasis::DsrExport => matches!(
            role,
            EdiscoveryActorRole::DataSubject
                | EdiscoveryActorRole::TenantAdmin
                | EdiscoveryActorRole::Dpo
        ),
        EdiscoveryLegalBasis::Litigation => matches!(
            role,
            EdiscoveryActorRole::LegalCounsel
                | EdiscoveryActorRole::Auditor
                | EdiscoveryActorRole::Dpo
        ),
        EdiscoveryLegalBasis::RegulatoryInquiry => matches!(
            role,
            EdiscoveryActorRole::RegulatorDelegate
                | EdiscoveryActorRole::Auditor
                | EdiscoveryActorRole::Dpo
                | EdiscoveryActorRole::LegalCounsel
        ),
        EdiscoveryLegalBasis::InternalInvestigation => matches!(
            role,
            EdiscoveryActorRole::TenantAdmin
                | EdiscoveryActorRole::Auditor
                | EdiscoveryActorRole::LegalCounsel
                | EdiscoveryActorRole::Dpo
        ),
    };
    if authorized {
        Ok(())
    } else {
        Err(EdiscoveryError::UnauthorizedActorRole)
    }
}

fn validate_cross_region_consent(
    source_region: &str,
    destination_region: &str,
    consent_receipt_ref: Option<&str>,
) -> Result<(), EdiscoveryError> {
    if source_region != destination_region && consent_receipt_ref.is_none() {
        Err(EdiscoveryError::CrossRegionConsentRequired)
    } else {
        Ok(())
    }
}

fn validate_surfaces(surfaces: &[WorkspaceRetentionSurface]) -> Result<(), EdiscoveryError> {
    if surfaces.is_empty() {
        return Err(EdiscoveryError::EmptySurfaceSet);
    }
    let mut seen = BTreeSet::new();
    for surface in surfaces {
        if !seen.insert(*surface) {
            return Err(EdiscoveryError::DuplicateSurface);
        }
    }
    Ok(())
}

fn validate_data_classes(data_classes: &[PrivacyDataClass]) -> Result<(), EdiscoveryError> {
    if data_classes.is_empty() {
        return Err(EdiscoveryError::EmptyDataClassSet);
    }
    let mut seen = BTreeSet::new();
    for data_class in data_classes {
        if !seen.insert(*data_class) {
            return Err(EdiscoveryError::DuplicateDataClass);
        }
    }
    Ok(())
}

fn format_supported_for(
    surface: WorkspaceRetentionSurface,
    format: EdiscoveryExportFormat,
) -> bool {
    match surface {
        WorkspaceRetentionSurface::Mail => matches!(
            format,
            EdiscoveryExportFormat::Eml
                | EdiscoveryExportFormat::Mbox
                | EdiscoveryExportFormat::Pdf
                | EdiscoveryExportFormat::Jsonl
        ),
        WorkspaceRetentionSurface::Calendar => {
            matches!(
                format,
                EdiscoveryExportFormat::Ics
                    | EdiscoveryExportFormat::Jsonl
                    | EdiscoveryExportFormat::Pdf
            )
        }
        WorkspaceRetentionSurface::Docs => matches!(
            format,
            EdiscoveryExportFormat::Pdf
                | EdiscoveryExportFormat::Docx
                | EdiscoveryExportFormat::Hwpx
                | EdiscoveryExportFormat::NativeZip
                | EdiscoveryExportFormat::Jsonl
        ),
        WorkspaceRetentionSurface::Sheets => matches!(
            format,
            EdiscoveryExportFormat::Pdf
                | EdiscoveryExportFormat::Xlsx
                | EdiscoveryExportFormat::Csv
                | EdiscoveryExportFormat::NativeZip
                | EdiscoveryExportFormat::Jsonl
        ),
        WorkspaceRetentionSurface::Slides => matches!(
            format,
            EdiscoveryExportFormat::Pdf
                | EdiscoveryExportFormat::Pptx
                | EdiscoveryExportFormat::NativeZip
                | EdiscoveryExportFormat::Jsonl
        ),
        WorkspaceRetentionSurface::Drive => matches!(
            format,
            EdiscoveryExportFormat::NativeZip
                | EdiscoveryExportFormat::Pdf
                | EdiscoveryExportFormat::Jsonl
        ),
        WorkspaceRetentionSurface::Meet | WorkspaceRetentionSurface::Recordings => matches!(
            format,
            EdiscoveryExportFormat::Mp4
                | EdiscoveryExportFormat::TranscriptJson
                | EdiscoveryExportFormat::NativeZip
                | EdiscoveryExportFormat::Jsonl
                | EdiscoveryExportFormat::Pdf
        ),
        WorkspaceRetentionSurface::Chat => {
            matches!(
                format,
                EdiscoveryExportFormat::Jsonl | EdiscoveryExportFormat::Pdf
            )
        }
        WorkspaceRetentionSurface::Forms => matches!(
            format,
            EdiscoveryExportFormat::Csv
                | EdiscoveryExportFormat::Jsonl
                | EdiscoveryExportFormat::Pdf
        ),
        WorkspaceRetentionSurface::Sites => matches!(
            format,
            EdiscoveryExportFormat::Html
                | EdiscoveryExportFormat::Pdf
                | EdiscoveryExportFormat::NativeZip
                | EdiscoveryExportFormat::Jsonl
        ),
        WorkspaceRetentionSurface::Tasks
        | WorkspaceRetentionSurface::Notes
        | WorkspaceRetentionSurface::AddressBook => matches!(
            format,
            EdiscoveryExportFormat::Jsonl
                | EdiscoveryExportFormat::Csv
                | EdiscoveryExportFormat::Pdf
        ),
        WorkspaceRetentionSurface::Translate => {
            matches!(
                format,
                EdiscoveryExportFormat::Jsonl | EdiscoveryExportFormat::Csv
            )
        }
    }
}

fn validate_hash(hash: &str, error: EdiscoveryError) -> Result<(), EdiscoveryError> {
    if hash.trim() != hash
        || !hash.starts_with(SHA256_PREFIX)
        || hash.len() == SHA256_PREFIX.len()
        || hash.chars().any(char::is_control)
    {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_optional_non_empty(
    value: Option<&str>,
    error: EdiscoveryError,
) -> Result<(), EdiscoveryError> {
    match value {
        Some(value) => validate_non_empty(value, error),
        None => Ok(()),
    }
}

fn validate_time_order(start: u64, end: u64) -> Result<(), EdiscoveryError> {
    if start > end {
        Err(EdiscoveryError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: EdiscoveryError) -> Result<(), EdiscoveryError> {
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
    use data_boundary_kernel::{DataClassification, OperationalDataClass};
    use compliance_retention::{
        RetentionDecisionCreate, RetentionRecordRef, RetentionRecordRefCreate,
    };

    fn privacy(data_class: DataClass) -> PrivacyDataClass {
        PrivacyDataClass::new(data_class).unwrap()
    }

    fn request() -> EdiscoveryRequest {
        EdiscoveryRequest::new(EdiscoveryRequestCreate {
            request_id: "ediscovery-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            destination_region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            actor_ref: "auditor-1".into(),
            actor_role: EdiscoveryActorRole::Auditor,
            legal_basis: EdiscoveryLegalBasis::RegulatoryInquiry,
            matter_ref: "matter-1".into(),
            purpose_ref: "regulator-production".into(),
            consent_receipt_ref: None,
            surfaces: vec![WorkspaceRetentionSurface::Mail],
            data_classes: vec![privacy(DataClass::PiiIdentifying)],
            requested_at_epoch_seconds: 1_700_000_000,
        })
        .unwrap()
    }

    fn record(source_ref: &str, surface: WorkspaceRetentionSurface) -> RetentionRecordRef {
        RetentionRecordRef::new(RetentionRecordRefCreate {
            record_id: source_ref.into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            surface,
            subject_ref: Some("subject-1".into()),
            data_class: privacy(DataClass::PiiIdentifying),
            kms_shred_key_id: Some("kms-record-1".into()),
            created_at_epoch_seconds: 1_699_999_000,
        })
        .unwrap()
    }

    fn export_decision(source_ref: &str, surface: WorkspaceRetentionSurface) -> RetentionDecision {
        let policy = retention_policy_for_export(
            "retention-export".into(),
            "tenant-1".into(),
            "region-alpha1".into(),
            surface,
            1_699_000_000,
        )
        .unwrap();
        RetentionDecision::evaluate(
            RetentionDecisionCreate {
                decision_id: format!("decision-{source_ref}"),
                request_id: "ediscovery-1".into(),
                request_kind: RetentionRequestKind::DsrExport,
                requested_by_actor_ref: "auditor-1".into(),
                decided_at_epoch_seconds: 1_700_000_100,
            },
            &policy,
            &record(source_ref, surface),
            &[],
        )
        .unwrap()
    }

    fn item(source_ref: &str) -> EdiscoveryExportItem {
        EdiscoveryExportItem::new(EdiscoveryExportItemCreate {
            item_id: format!("item-{source_ref}"),
            source_ref: source_ref.into(),
            surface: WorkspaceRetentionSurface::Mail,
            data_class: privacy(DataClass::PiiIdentifying),
            export_format: EdiscoveryExportFormat::Eml,
            retention_decision_id: format!("decision-{source_ref}"),
            evidence_hash: format!("sha256:evidence-{source_ref}"),
            byte_len: 1024,
        })
        .unwrap()
    }

    fn package() -> EdiscoveryExportPackage {
        EdiscoveryExportPackage::new(
            EdiscoveryExportPackageCreate {
                package_id: "package-1".into(),
                items: vec![item("message-1")],
                manifest_hash: "sha256:manifest".into(),
                package_sha256: "sha256:package".into(),
                encrypted_package_key_ref: "kms://tenant-1/package-1".into(),
                byte_len: 2048,
                created_at_epoch_seconds: 1_700_000_100,
                completed_at_epoch_seconds: 1_700_000_200,
            },
            &request(),
            &[export_decision(
                "message-1",
                WorkspaceRetentionSurface::Mail,
            )],
        )
        .unwrap()
    }

    #[test]
    fn request_requires_authorized_role_scope_and_cross_region_consent() {
        let valid = request();
        assert_eq!(valid.schema_version.value, 1);
        assert_eq!(
            valid.actor_ref.data_class,
            DataClassification::Privacy(actor_data_class())
        );

        let unauthorized = EdiscoveryRequest::new(EdiscoveryRequestCreate {
            actor_role: EdiscoveryActorRole::DataSubject,
            legal_basis: EdiscoveryLegalBasis::RegulatoryInquiry,
            ..request_create()
        });
        assert_eq!(unauthorized, Err(EdiscoveryError::UnauthorizedActorRole));

        let cross_region = EdiscoveryRequest::new(EdiscoveryRequestCreate {
            region: "region-beta1".into(),
            destination_region: "region-alpha1".into(),
            consent_receipt_ref: None,
            ..request_create()
        });
        assert_eq!(
            cross_region,
            Err(EdiscoveryError::CrossRegionConsentRequired)
        );

        let duplicate_surface = EdiscoveryRequest::new(EdiscoveryRequestCreate {
            surfaces: vec![
                WorkspaceRetentionSurface::Mail,
                WorkspaceRetentionSurface::Mail,
            ],
            ..request_create()
        });
        assert_eq!(duplicate_surface, Err(EdiscoveryError::DuplicateSurface));
    }

    #[test]
    fn items_are_surface_format_allowlisted() {
        assert_eq!(
            EdiscoveryExportItem::new(EdiscoveryExportItemCreate {
                item_id: "bad-format".into(),
                source_ref: "message-1".into(),
                surface: WorkspaceRetentionSurface::Mail,
                data_class: privacy(DataClass::PiiIdentifying),
                export_format: EdiscoveryExportFormat::Pptx,
                retention_decision_id: "decision-message-1".into(),
                evidence_hash: "sha256:evidence".into(),
                byte_len: 1,
            }),
            Err(EdiscoveryError::UnsupportedExportFormat)
        );
    }

    #[test]
    fn package_requires_export_only_retention_decisions() {
        let package = package();
        assert_eq!(package.items.value.len(), 1);
        assert_eq!(
            package.encrypted_package_key_ref.data_class,
            DataClassification::from(DataClass::Secret)
        );

        let missing_decision = EdiscoveryExportPackage::new(
            EdiscoveryExportPackageCreate {
                package_id: "package-2".into(),
                items: vec![item("message-1")],
                manifest_hash: "sha256:manifest".into(),
                package_sha256: "sha256:package".into(),
                encrypted_package_key_ref: "kms://tenant-1/package-2".into(),
                byte_len: 2048,
                created_at_epoch_seconds: 1_700_000_100,
                completed_at_epoch_seconds: 1_700_000_200,
            },
            &request(),
            &[],
        );
        assert_eq!(
            missing_decision,
            Err(EdiscoveryError::MissingRetentionDecision)
        );

        let wrong_decision = export_decision("other-message", WorkspaceRetentionSurface::Mail);
        assert_eq!(
            EdiscoveryExportPackage::new(
                EdiscoveryExportPackageCreate {
                    package_id: "package-3".into(),
                    items: vec![item("message-1")],
                    manifest_hash: "sha256:manifest".into(),
                    package_sha256: "sha256:package".into(),
                    encrypted_package_key_ref: "kms://tenant-1/package-3".into(),
                    byte_len: 2048,
                    created_at_epoch_seconds: 1_700_000_100,
                    completed_at_epoch_seconds: 1_700_000_200,
                },
                &request(),
                &[wrong_decision],
            ),
            Err(EdiscoveryError::MissingRetentionDecision)
        );
    }

    #[test]
    fn package_rejects_out_of_scope_item_data_class_and_duplicate_sources() {
        let out_of_scope_item = EdiscoveryExportItem::new(EdiscoveryExportItemCreate {
            item_id: "item-message-1".into(),
            source_ref: "message-1".into(),
            surface: WorkspaceRetentionSurface::Mail,
            data_class: privacy(DataClass::Phi),
            export_format: EdiscoveryExportFormat::Eml,
            retention_decision_id: "decision-message-1".into(),
            evidence_hash: "sha256:evidence".into(),
            byte_len: 1,
        })
        .unwrap();
        assert_eq!(
            EdiscoveryExportPackage::new(
                EdiscoveryExportPackageCreate {
                    package_id: "package-4".into(),
                    items: vec![out_of_scope_item],
                    manifest_hash: "sha256:manifest".into(),
                    package_sha256: "sha256:package".into(),
                    encrypted_package_key_ref: "kms://tenant-1/package-4".into(),
                    byte_len: 2048,
                    created_at_epoch_seconds: 1_700_000_100,
                    completed_at_epoch_seconds: 1_700_000_200,
                },
                &request(),
                &[export_decision(
                    "message-1",
                    WorkspaceRetentionSurface::Mail
                )],
            ),
            Err(EdiscoveryError::ItemDataClassOutOfScope)
        );

        assert_eq!(
            EdiscoveryExportPackage::new(
                EdiscoveryExportPackageCreate {
                    package_id: "package-5".into(),
                    items: vec![item("message-1"), item("message-1")],
                    manifest_hash: "sha256:manifest".into(),
                    package_sha256: "sha256:package".into(),
                    encrypted_package_key_ref: "kms://tenant-1/package-5".into(),
                    byte_len: 2048,
                    created_at_epoch_seconds: 1_700_000_100,
                    completed_at_epoch_seconds: 1_700_000_200,
                },
                &request(),
                &[export_decision(
                    "message-1",
                    WorkspaceRetentionSurface::Mail
                )],
            ),
            Err(EdiscoveryError::DuplicateItemId)
        );
    }

    #[test]
    fn proof_requires_exact_manifest_hashes_and_signature_refs() {
        let package = package();
        let proof = EdiscoveryExportProof::new(
            EdiscoveryExportProofCreate {
                proof_id: "proof-1".into(),
                request_id: "ediscovery-1".into(),
                package_id: "package-1".into(),
                manifest_hash: "sha256:manifest".into(),
                package_sha256: "sha256:package".into(),
                item_evidence_hashes: vec!["sha256:evidence-message-1".into()],
                signer_ref: "cosign://tenant-1/keyless".into(),
                signature_ref: "rekor://entry-1".into(),
                rekor_log_index: 42,
                signed_at_epoch_seconds: 1_700_000_300,
            },
            &package,
        )
        .unwrap();
        assert_eq!(proof.schema_version.value, 1);

        assert_eq!(
            EdiscoveryExportProof::new(
                EdiscoveryExportProofCreate {
                    proof_id: "proof-2".into(),
                    request_id: "ediscovery-1".into(),
                    package_id: "package-1".into(),
                    manifest_hash: "sha256:manifest".into(),
                    package_sha256: "sha256:package".into(),
                    item_evidence_hashes: vec!["sha256:other".into()],
                    signer_ref: "cosign://tenant-1/keyless".into(),
                    signature_ref: "rekor://entry-2".into(),
                    rekor_log_index: 43,
                    signed_at_epoch_seconds: 1_700_000_300,
                },
                &package,
            ),
            Err(EdiscoveryError::ProofItemHashMismatch)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_ediscovery_data_class_from_legacy(DataClass::Audit),
            Err(EdiscoveryError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
        assert_eq!(
            default_workspace_ediscovery_data_class().data_class(),
            DataClass::PiiIdentifying
        );
    }

    fn request_create() -> EdiscoveryRequestCreate {
        EdiscoveryRequestCreate {
            request_id: "ediscovery-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            destination_region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            actor_ref: "auditor-1".into(),
            actor_role: EdiscoveryActorRole::Auditor,
            legal_basis: EdiscoveryLegalBasis::RegulatoryInquiry,
            matter_ref: "matter-1".into(),
            purpose_ref: "regulator-production".into(),
            consent_receipt_ref: None,
            surfaces: vec![WorkspaceRetentionSurface::Mail],
            data_classes: vec![privacy(DataClass::PiiIdentifying)],
            requested_at_epoch_seconds: 1_700_000_000,
        }
    }
}
