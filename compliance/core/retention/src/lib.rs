//! Workspace retention and legal-hold kernel.
//!
//! This crate centralizes the W-Workspace-GA retention, legal-hold, and DSR
//! purge-decision invariants named by `docs/products/workspace/PRD.md`,
//! ADR-0029, and ADR-0038. It owns typed policy records and deterministic
//! purge decisions only; per-surface storage engines, audit emitters, trust
//! portal UI, and DSR orchestration remain outside this crate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use data_boundary_kernel::{Classified, DataClass, DataClassMatcher, PrivacyDataClass};

const RETENTION_POLICY_SCHEMA_VERSION: u32 = 1;
const RETENTION_RECORD_SCHEMA_VERSION: u32 = 1;
const LEGAL_HOLD_SCHEMA_VERSION: u32 = 1;
const RETENTION_DECISION_SCHEMA_VERSION: u32 = 1;
const SHA256_PREFIX: &str = "sha256:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetentionError {
    InvalidPolicyId,
    InvalidTenantId,
    InvalidRegion,
    InvalidRecordId,
    InvalidRequestId,
    InvalidDecisionId,
    InvalidActorRef,
    InvalidKmsShredKeyId,
    InvalidHoldId,
    InvalidAuthorityRef,
    InvalidReasonRef,
    InvalidReleaseActorRef,
    InvalidReleaseEvidenceHash,
    MissingReleaseActorRef,
    MissingReleaseEvidenceHash,
    InvalidRetentionHorizon,
    InvalidRetentionDeadline,
    InvalidTimeOrder,
    PolicyRecordMismatch,
    HoldRecordMismatch,
    DuplicateHoldId,
    HighRiskRecordRequiresKmsShred,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkspaceRetentionSurface {
    Mail,
    Calendar,
    Docs,
    Drive,
    Sheets,
    Slides,
    Meet,
    Chat,
    Forms,
    Sites,
    Tasks,
    Notes,
    Translate,
    Recordings,
    AddressBook,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RetentionHorizon {
    Seconds(u64),
    Indefinite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RetentionLawfulBasis {
    Consent,
    Contract,
    LegitimateInterest,
    LegalObligation,
    RegulatoryCompliance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RetentionDisposition {
    KmsShred,
    RecordDelete,
    ColdStoragePurge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RetentionRequestKind {
    ScheduledExpiry,
    DsrErase,
    TenantDeletion,
    AdminPurge,
    DsrRestrict,
    DsrExport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RetentionDecisionOutcome {
    RetainUntilDeadline,
    RetainUnderLegalHold,
    RetainUnderLawfulBasis,
    PermitKmsShred,
    PermitRecordDelete,
    PermitColdStoragePurge,
    RestrictProcessing,
    ExportOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EraseMethod {
    KmsShred,
    RecordDelete,
    ColdStoragePurge,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LegalHoldScope {
    TenantWide,
    Surface(WorkspaceRetentionSurface),
    Record {
        surface: WorkspaceRetentionSurface,
        record_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPolicyCreate {
    pub policy_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: INTERNAL_ONLY
    pub surface: WorkspaceRetentionSurface, // data_class: INTERNAL_ONLY
    pub horizon: RetentionHorizon,          // data_class: INTERNAL_ONLY
    pub lawful_basis: RetentionLawfulBasis, // data_class: INTERNAL_ONLY
    pub disposition: RetentionDisposition,  // data_class: INTERNAL_ONLY
    pub effective_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    pub policy_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub region: Classified<String>,    // data_class: INTERNAL_ONLY
    pub surface: Classified<WorkspaceRetentionSurface>, // data_class: INTERNAL_ONLY
    pub horizon: Classified<RetentionHorizon>, // data_class: INTERNAL_ONLY
    pub lawful_basis: Classified<RetentionLawfulBasis>, // data_class: INTERNAL_ONLY
    pub disposition: Classified<RetentionDisposition>, // data_class: INTERNAL_ONLY
    pub effective_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionRecordRefCreate {
    pub record_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: INTERNAL_ONLY
    pub surface: WorkspaceRetentionSurface, // data_class: INTERNAL_ONLY
    pub subject_ref: Option<String>,        // data_class: PII_IDENTIFYING
    pub data_class: PrivacyDataClass,       // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: Option<String>,   // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionRecordRef {
    pub record_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub region: Classified<String>,    // data_class: INTERNAL_ONLY
    pub surface: Classified<WorkspaceRetentionSurface>, // data_class: INTERNAL_ONLY
    pub subject_ref: Classified<Option<String>>, // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalHoldCreate {
    pub hold_id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub region: String,                         // data_class: INTERNAL_ONLY
    pub scope: LegalHoldScope,                  // data_class: INTERNAL_ONLY
    pub authority_ref: String,                  // data_class: INTERNAL_ONLY
    pub reason_ref: String,                     // data_class: INTERNAL_ONLY
    pub imposed_by_actor_ref: String,           // data_class: PII_IDENTIFYING
    pub imposed_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub released_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub released_by_actor_ref: Option<String>,  // data_class: PII_IDENTIFYING
    pub release_evidence_hash: Option<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalHold {
    pub hold_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub region: Classified<String>,        // data_class: INTERNAL_ONLY
    pub scope: Classified<LegalHoldScope>, // data_class: INTERNAL_ONLY
    pub authority_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub reason_ref: Classified<String>,    // data_class: INTERNAL_ONLY
    pub imposed_by_actor_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub imposed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub released_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub released_by_actor_ref: Classified<Option<String>>, // data_class: PII_IDENTIFYING
    pub release_evidence_hash: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionDecisionCreate {
    pub decision_id: String,                // data_class: INTERNAL_ONLY
    pub request_id: String,                 // data_class: INTERNAL_ONLY
    pub request_kind: RetentionRequestKind, // data_class: INTERNAL_ONLY
    pub requested_by_actor_ref: String,     // data_class: PII_IDENTIFYING
    pub decided_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionDecision {
    pub decision_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub request_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub request_kind: Classified<RetentionRequestKind>, // data_class: INTERNAL_ONLY
    pub requested_by_actor_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub region: Classified<String>,      // data_class: INTERNAL_ONLY
    pub surface: Classified<WorkspaceRetentionSurface>, // data_class: INTERNAL_ONLY
    pub record_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub policy_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub outcome: Classified<RetentionDecisionOutcome>, // data_class: INTERNAL_ONLY
    pub erase_method: Classified<Option<EraseMethod>>, // data_class: INTERNAL_ONLY
    pub retention_deadline_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub blocking_hold_ids: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

pub trait RetentionPolicyReader {
    fn read_policy(
        &self,
        tenant_id: &str,
        policy_id: &str,
    ) -> Result<Option<RetentionPolicy>, RetentionError>;
}

pub trait LegalHoldRegistry {
    fn active_holds_for_record(
        &self,
        record: &RetentionRecordRef,
        at_epoch_seconds: u64,
    ) -> Result<Vec<LegalHold>, RetentionError>;
}

impl RetentionPolicy {
    pub fn new(input: RetentionPolicyCreate) -> Result<Self, RetentionError> {
        validate_non_empty(&input.policy_id, RetentionError::InvalidPolicyId)?;
        validate_non_empty(&input.tenant_id, RetentionError::InvalidTenantId)?;
        validate_non_empty(&input.region, RetentionError::InvalidRegion)?;
        validate_horizon(input.horizon)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        validate_time_order(
            input.effective_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;

        Ok(Self {
            policy_id: internal(input.policy_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            surface: internal(input.surface),
            horizon: internal(input.horizon),
            lawful_basis: internal(input.lawful_basis),
            disposition: internal(input.disposition),
            effective_at_epoch_seconds: internal(input.effective_at_epoch_seconds),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(RETENTION_POLICY_SCHEMA_VERSION),
        })
    }

    pub fn retention_deadline_for(
        &self,
        record: &RetentionRecordRef,
    ) -> Result<Option<u64>, RetentionError> {
        match self.horizon.value {
            RetentionHorizon::Indefinite => Ok(None),
            RetentionHorizon::Seconds(seconds) => record
                .created_at_epoch_seconds
                .value
                .checked_add(seconds)
                .map(Some)
                .ok_or(RetentionError::InvalidRetentionDeadline),
        }
    }
}

impl RetentionRecordRef {
    pub fn new(input: RetentionRecordRefCreate) -> Result<Self, RetentionError> {
        validate_non_empty(&input.record_id, RetentionError::InvalidRecordId)?;
        validate_non_empty(&input.tenant_id, RetentionError::InvalidTenantId)?;
        validate_non_empty(&input.region, RetentionError::InvalidRegion)?;
        validate_optional_non_empty(
            input.subject_ref.as_deref(),
            RetentionError::InvalidActorRef,
        )?;
        validate_optional_non_empty(
            input.kms_shred_key_id.as_deref(),
            RetentionError::InvalidKmsShredKeyId,
        )?;

        Ok(Self {
            record_id: internal(input.record_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            surface: internal(input.surface),
            subject_ref: Classified::new(input.subject_ref, DataClass::PiiIdentifying),
            data_class: internal(input.data_class),
            kms_shred_key_id: internal(input.kms_shred_key_id),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(RETENTION_RECORD_SCHEMA_VERSION),
        })
    }
}

impl LegalHold {
    pub fn new(input: LegalHoldCreate) -> Result<Self, RetentionError> {
        validate_non_empty(&input.hold_id, RetentionError::InvalidHoldId)?;
        validate_non_empty(&input.tenant_id, RetentionError::InvalidTenantId)?;
        validate_non_empty(&input.region, RetentionError::InvalidRegion)?;
        validate_scope(&input.scope)?;
        validate_non_empty(&input.authority_ref, RetentionError::InvalidAuthorityRef)?;
        validate_non_empty(&input.reason_ref, RetentionError::InvalidReasonRef)?;
        validate_non_empty(&input.imposed_by_actor_ref, RetentionError::InvalidActorRef)?;
        validate_release_fields(&input)?;

        Ok(Self {
            hold_id: internal(input.hold_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            scope: internal(input.scope),
            authority_ref: internal(input.authority_ref),
            reason_ref: internal(input.reason_ref),
            imposed_by_actor_ref: Classified::new(
                input.imposed_by_actor_ref,
                DataClass::PiiIdentifying,
            ),
            imposed_at_epoch_seconds: internal(input.imposed_at_epoch_seconds),
            released_at_epoch_seconds: internal(input.released_at_epoch_seconds),
            released_by_actor_ref: Classified::new(
                input.released_by_actor_ref,
                DataClass::PiiIdentifying,
            ),
            release_evidence_hash: internal(input.release_evidence_hash),
            schema_version: internal(LEGAL_HOLD_SCHEMA_VERSION),
        })
    }

    pub fn is_active_at(&self, at_epoch_seconds: u64) -> bool {
        match self.released_at_epoch_seconds.value {
            Some(released_at) => at_epoch_seconds < released_at,
            None => true,
        }
    }

    pub fn applies_to(&self, record: &RetentionRecordRef) -> bool {
        if self.tenant_id.value != record.tenant_id.value
            || self.region.value != record.region.value
        {
            return false;
        }
        match &self.scope.value {
            LegalHoldScope::TenantWide => true,
            LegalHoldScope::Surface(surface) => *surface == record.surface.value,
            LegalHoldScope::Record { surface, record_id } => {
                *surface == record.surface.value && *record_id == record.record_id.value
            }
        }
    }
}

impl RetentionDecision {
    pub fn evaluate(
        input: RetentionDecisionCreate,
        policy: &RetentionPolicy,
        record: &RetentionRecordRef,
        legal_holds: &[LegalHold],
    ) -> Result<Self, RetentionError> {
        validate_non_empty(&input.decision_id, RetentionError::InvalidDecisionId)?;
        validate_non_empty(&input.request_id, RetentionError::InvalidRequestId)?;
        validate_non_empty(
            &input.requested_by_actor_ref,
            RetentionError::InvalidActorRef,
        )?;
        validate_policy_record(policy, record)?;
        validate_policy_record_data_class(policy, record)?;
        validate_decision_time(input.decided_at_epoch_seconds, policy, record)?;
        validate_holds_for_record(legal_holds, record)?;

        let deadline = policy.retention_deadline_for(record)?;
        let blocking_hold_ids = active_hold_ids(legal_holds, input.decided_at_epoch_seconds);
        let (outcome, erase_method) = evaluate_outcome(
            input.request_kind,
            policy,
            deadline,
            input.decided_at_epoch_seconds,
            !blocking_hold_ids.is_empty(),
        );

        Ok(Self {
            decision_id: internal(input.decision_id),
            request_id: internal(input.request_id),
            request_kind: internal(input.request_kind),
            requested_by_actor_ref: Classified::new(
                input.requested_by_actor_ref,
                DataClass::PiiIdentifying,
            ),
            tenant_id: internal(record.tenant_id.value.clone()),
            region: internal(record.region.value.clone()),
            surface: internal(record.surface.value),
            record_id: internal(record.record_id.value.clone()),
            policy_id: internal(policy.policy_id.value.clone()),
            outcome: internal(outcome),
            erase_method: internal(erase_method),
            retention_deadline_epoch_seconds: internal(deadline),
            blocking_hold_ids: internal(blocking_hold_ids),
            decided_at_epoch_seconds: internal(input.decided_at_epoch_seconds),
            schema_version: internal(RETENTION_DECISION_SCHEMA_VERSION),
        })
    }

    pub fn permits_erasure(&self) -> bool {
        self.erase_method.value.is_some()
    }
}

pub fn default_workspace_retention_data_class() -> PrivacyDataClass {
    // ADR-0083 Tier 1: use kernel's infallible `internal_only()` constructor.
    PrivacyDataClass::internal_only()
}

pub fn workspace_retention_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, RetentionError> {
    PrivacyDataClass::new(data_class).map_err(|_| RetentionError::InvalidDataClass)
}

fn evaluate_outcome(
    request_kind: RetentionRequestKind,
    policy: &RetentionPolicy,
    deadline: Option<u64>,
    now_epoch_seconds: u64,
    has_active_hold: bool,
) -> (RetentionDecisionOutcome, Option<EraseMethod>) {
    if request_kind == RetentionRequestKind::DsrRestrict {
        return (RetentionDecisionOutcome::RestrictProcessing, None);
    }
    if request_kind == RetentionRequestKind::DsrExport {
        return (RetentionDecisionOutcome::ExportOnly, None);
    }
    if has_active_hold {
        return (RetentionDecisionOutcome::RetainUnderLegalHold, None);
    }
    if law_requires_retention(policy.lawful_basis.value)
        && matches!(
            request_kind,
            RetentionRequestKind::DsrErase | RetentionRequestKind::TenantDeletion
        )
        && !deadline_has_passed(deadline, now_epoch_seconds)
    {
        return (RetentionDecisionOutcome::RetainUnderLawfulBasis, None);
    }
    if request_kind == RetentionRequestKind::ScheduledExpiry
        && !deadline_has_passed(deadline, now_epoch_seconds)
    {
        return (RetentionDecisionOutcome::RetainUntilDeadline, None);
    }

    let erase_method = erase_method_for(policy.disposition.value);
    (outcome_for(erase_method), Some(erase_method))
}

fn deadline_has_passed(deadline: Option<u64>, now_epoch_seconds: u64) -> bool {
    match deadline {
        Some(deadline) => now_epoch_seconds >= deadline,
        None => false,
    }
}

fn law_requires_retention(lawful_basis: RetentionLawfulBasis) -> bool {
    matches!(
        lawful_basis,
        RetentionLawfulBasis::LegalObligation | RetentionLawfulBasis::RegulatoryCompliance
    )
}

fn erase_method_for(disposition: RetentionDisposition) -> EraseMethod {
    match disposition {
        RetentionDisposition::KmsShred => EraseMethod::KmsShred,
        RetentionDisposition::RecordDelete => EraseMethod::RecordDelete,
        RetentionDisposition::ColdStoragePurge => EraseMethod::ColdStoragePurge,
    }
}

fn outcome_for(erase_method: EraseMethod) -> RetentionDecisionOutcome {
    match erase_method {
        EraseMethod::KmsShred => RetentionDecisionOutcome::PermitKmsShred,
        EraseMethod::RecordDelete => RetentionDecisionOutcome::PermitRecordDelete,
        EraseMethod::ColdStoragePurge => RetentionDecisionOutcome::PermitColdStoragePurge,
    }
}

fn validate_horizon(horizon: RetentionHorizon) -> Result<(), RetentionError> {
    match horizon {
        RetentionHorizon::Seconds(0) => Err(RetentionError::InvalidRetentionHorizon),
        RetentionHorizon::Seconds(_) | RetentionHorizon::Indefinite => Ok(()),
    }
}

fn validate_release_fields(input: &LegalHoldCreate) -> Result<(), RetentionError> {
    if let Some(released_at) = input.released_at_epoch_seconds {
        validate_time_order(input.imposed_at_epoch_seconds, released_at)?;
        match input.released_by_actor_ref.as_deref() {
            Some(actor_ref) => {
                validate_non_empty(actor_ref, RetentionError::InvalidReleaseActorRef)?;
            }
            None => return Err(RetentionError::MissingReleaseActorRef),
        }
        match input.release_evidence_hash.as_deref() {
            Some(hash) => validate_evidence_hash(hash)?,
            None => return Err(RetentionError::MissingReleaseEvidenceHash),
        }
    } else {
        validate_optional_non_empty(
            input.released_by_actor_ref.as_deref(),
            RetentionError::InvalidReleaseActorRef,
        )?;
        validate_optional_evidence_hash(input.release_evidence_hash.as_deref())?;
    }
    Ok(())
}

fn validate_policy_record(
    policy: &RetentionPolicy,
    record: &RetentionRecordRef,
) -> Result<(), RetentionError> {
    if policy.tenant_id.value != record.tenant_id.value
        || policy.region.value != record.region.value
        || policy.surface.value != record.surface.value
    {
        return Err(RetentionError::PolicyRecordMismatch);
    }
    Ok(())
}

fn validate_policy_record_data_class(
    policy: &RetentionPolicy,
    record: &RetentionRecordRef,
) -> Result<(), RetentionError> {
    if policy.disposition.value == RetentionDisposition::KmsShred {
        validate_optional_non_empty(
            record.kms_shred_key_id.value.as_deref(),
            RetentionError::InvalidKmsShredKeyId,
        )?;
    }
    if is_regulated_erasure_class(record.data_class.value)
        && policy.disposition.value != RetentionDisposition::KmsShred
    {
        return Err(RetentionError::HighRiskRecordRequiresKmsShred);
    }
    Ok(())
}

fn validate_decision_time(
    decided_at_epoch_seconds: u64,
    policy: &RetentionPolicy,
    record: &RetentionRecordRef,
) -> Result<(), RetentionError> {
    validate_time_order(
        record.created_at_epoch_seconds.value,
        decided_at_epoch_seconds,
    )?;
    validate_time_order(
        policy.effective_at_epoch_seconds.value,
        decided_at_epoch_seconds,
    )?;
    Ok(())
}

fn validate_holds_for_record(
    legal_holds: &[LegalHold],
    record: &RetentionRecordRef,
) -> Result<(), RetentionError> {
    let mut hold_ids = BTreeSet::new();
    for hold in legal_holds {
        if !hold_ids.insert(hold.hold_id.value.clone()) {
            return Err(RetentionError::DuplicateHoldId);
        }
        if !hold.applies_to(record) {
            return Err(RetentionError::HoldRecordMismatch);
        }
    }
    Ok(())
}

fn active_hold_ids(legal_holds: &[LegalHold], at_epoch_seconds: u64) -> Vec<String> {
    legal_holds
        .iter()
        .filter(|hold| hold.is_active_at(at_epoch_seconds))
        .map(|hold| hold.hold_id.value.clone())
        .collect()
}

fn validate_scope(scope: &LegalHoldScope) -> Result<(), RetentionError> {
    match scope {
        LegalHoldScope::TenantWide | LegalHoldScope::Surface(_) => Ok(()),
        LegalHoldScope::Record { record_id, .. } => {
            validate_non_empty(record_id, RetentionError::InvalidRecordId)
        }
    }
}

fn validate_evidence_hash(evidence_hash: &str) -> Result<(), RetentionError> {
    if evidence_hash.trim() != evidence_hash
        || !evidence_hash.starts_with(SHA256_PREFIX)
        || evidence_hash.len() == SHA256_PREFIX.len()
        || evidence_hash.chars().any(char::is_control)
    {
        Err(RetentionError::InvalidReleaseEvidenceHash)
    } else {
        Ok(())
    }
}

fn validate_optional_evidence_hash(value: Option<&str>) -> Result<(), RetentionError> {
    match value {
        Some(value) => validate_evidence_hash(value),
        None => Ok(()),
    }
}

fn validate_optional_non_empty(
    value: Option<&str>,
    error: RetentionError,
) -> Result<(), RetentionError> {
    match value {
        Some(value) => validate_non_empty(value, error),
        None => Ok(()),
    }
}

fn validate_time_order(start: u64, end: u64) -> Result<(), RetentionError> {
    if start > end {
        Err(RetentionError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: RetentionError) -> Result<(), RetentionError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn is_regulated_erasure_class(data_class: PrivacyDataClass) -> bool {
    DataClassMatcher::SearchIndexRestricted.matches(data_class.data_class())
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn privacy(data_class: DataClass) -> PrivacyDataClass {
        PrivacyDataClass::new(data_class).unwrap()
    }

    fn policy(
        lawful_basis: RetentionLawfulBasis,
        disposition: RetentionDisposition,
    ) -> RetentionPolicy {
        RetentionPolicy::new(RetentionPolicyCreate {
            policy_id: "retention-7d".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            surface: WorkspaceRetentionSurface::Mail,
            horizon: RetentionHorizon::Seconds(604_800),
            lawful_basis,
            disposition,
            effective_at_epoch_seconds: 1_699_999_000,
            created_at_epoch_seconds: 1_699_999_000,
            updated_at_epoch_seconds: 1_700_000_000,
        })
        .unwrap()
    }

    fn record(data_class: PrivacyDataClass) -> RetentionRecordRef {
        RetentionRecordRef::new(RetentionRecordRefCreate {
            record_id: "message-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            surface: WorkspaceRetentionSurface::Mail,
            subject_ref: Some("subject-1".into()),
            data_class,
            kms_shred_key_id: Some("kms-message-1".into()),
            created_at_epoch_seconds: 1_700_000_000,
        })
        .unwrap()
    }

    fn decision_input(
        request_kind: RetentionRequestKind,
        decided_at: u64,
    ) -> RetentionDecisionCreate {
        RetentionDecisionCreate {
            decision_id: "decision-1".into(),
            request_id: "request-1".into(),
            request_kind,
            requested_by_actor_ref: "privacy-operator-1".into(),
            decided_at_epoch_seconds: decided_at,
        }
    }

    fn active_hold() -> LegalHold {
        LegalHold::new(LegalHoldCreate {
            hold_id: "legal-hold-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            scope: LegalHoldScope::Record {
                surface: WorkspaceRetentionSurface::Mail,
                record_id: "message-1".into(),
            },
            authority_ref: "matter-2026-001".into(),
            reason_ref: "litigation-preservation".into(),
            imposed_by_actor_ref: "dpo-1".into(),
            imposed_at_epoch_seconds: 1_700_000_100,
            released_at_epoch_seconds: None,
            released_by_actor_ref: None,
            release_evidence_hash: None,
        })
        .unwrap()
    }

    #[test]
    fn active_legal_hold_blocks_purge_and_names_the_hold() {
        let decision = RetentionDecision::evaluate(
            decision_input(RetentionRequestKind::ScheduledExpiry, 1_701_000_000),
            &policy(
                RetentionLawfulBasis::Contract,
                RetentionDisposition::KmsShred,
            ),
            &record(privacy(DataClass::PiiIdentifying)),
            &[active_hold()],
        )
        .unwrap();

        assert_eq!(
            decision.outcome.value,
            RetentionDecisionOutcome::RetainUnderLegalHold
        );
        assert_eq!(decision.erase_method.value, None);
        assert_eq!(decision.blocking_hold_ids.value, vec!["legal-hold-1"]);
        assert!(!decision.permits_erasure());
    }

    #[test]
    fn scheduled_expiry_requires_deadline_then_permits_kms_shred() {
        let policy = policy(
            RetentionLawfulBasis::Contract,
            RetentionDisposition::KmsShred,
        );
        let record = record(privacy(DataClass::PiiIdentifying));

        let early = RetentionDecision::evaluate(
            decision_input(RetentionRequestKind::ScheduledExpiry, 1_700_100_000),
            &policy,
            &record,
            &[],
        )
        .unwrap();
        assert_eq!(
            early.outcome.value,
            RetentionDecisionOutcome::RetainUntilDeadline
        );
        assert_eq!(
            early.retention_deadline_epoch_seconds.value,
            Some(1_700_604_800)
        );

        let expired = RetentionDecision::evaluate(
            decision_input(RetentionRequestKind::ScheduledExpiry, 1_701_000_000),
            &policy,
            &record,
            &[],
        )
        .unwrap();
        assert_eq!(
            expired.outcome.value,
            RetentionDecisionOutcome::PermitKmsShred
        );
        assert_eq!(expired.erase_method.value, Some(EraseMethod::KmsShred));
        assert!(expired.permits_erasure());
    }

    #[test]
    fn legal_obligation_blocks_dsr_until_retention_deadline() {
        let policy = policy(
            RetentionLawfulBasis::LegalObligation,
            RetentionDisposition::KmsShred,
        );
        let record = record(privacy(DataClass::PiiIdentifying));

        let before_deadline = RetentionDecision::evaluate(
            decision_input(RetentionRequestKind::DsrErase, 1_700_100_000),
            &policy,
            &record,
            &[],
        )
        .unwrap();
        assert_eq!(
            before_deadline.outcome.value,
            RetentionDecisionOutcome::RetainUnderLawfulBasis
        );
        assert_eq!(before_deadline.erase_method.value, None);

        let after_deadline = RetentionDecision::evaluate(
            decision_input(RetentionRequestKind::DsrErase, 1_701_000_000),
            &policy,
            &record,
            &[],
        )
        .unwrap();
        assert_eq!(
            after_deadline.outcome.value,
            RetentionDecisionOutcome::PermitKmsShred
        );
    }

    #[test]
    fn legal_hold_release_requires_evidence_and_valid_time_order() {
        let missing_evidence = LegalHold::new(LegalHoldCreate {
            hold_id: "legal-hold-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            scope: LegalHoldScope::TenantWide,
            authority_ref: "matter-2026-001".into(),
            reason_ref: "litigation-preservation".into(),
            imposed_by_actor_ref: "dpo-1".into(),
            imposed_at_epoch_seconds: 20,
            released_at_epoch_seconds: Some(30),
            released_by_actor_ref: Some("dpo-2".into()),
            release_evidence_hash: None,
        });
        assert_eq!(
            missing_evidence,
            Err(RetentionError::MissingReleaseEvidenceHash)
        );

        let bad_time = LegalHold::new(LegalHoldCreate {
            hold_id: "legal-hold-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            scope: LegalHoldScope::TenantWide,
            authority_ref: "matter-2026-001".into(),
            reason_ref: "litigation-preservation".into(),
            imposed_by_actor_ref: "dpo-1".into(),
            imposed_at_epoch_seconds: 30,
            released_at_epoch_seconds: Some(20),
            released_by_actor_ref: Some("dpo-2".into()),
            release_evidence_hash: Some("sha256:release".into()),
        });
        assert_eq!(bad_time, Err(RetentionError::InvalidTimeOrder));

        let released = LegalHold::new(LegalHoldCreate {
            hold_id: "legal-hold-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            scope: LegalHoldScope::TenantWide,
            authority_ref: "matter-2026-001".into(),
            reason_ref: "litigation-preservation".into(),
            imposed_by_actor_ref: "dpo-1".into(),
            imposed_at_epoch_seconds: 20,
            released_at_epoch_seconds: Some(30),
            released_by_actor_ref: Some("dpo-2".into()),
            release_evidence_hash: Some("sha256:release".into()),
        })
        .unwrap();
        assert!(released.is_active_at(29));
        assert!(!released.is_active_at(30));
    }

    #[test]
    fn policy_record_and_hold_mismatches_are_rejected() {
        let mut wrong_tenant = RetentionRecordRefCreate {
            record_id: "message-1".into(),
            tenant_id: "tenant-2".into(),
            region: "region-alpha1".into(),
            surface: WorkspaceRetentionSurface::Mail,
            subject_ref: Some("subject-1".into()),
            data_class: privacy(DataClass::PiiIdentifying),
            kms_shred_key_id: Some("kms-message-1".into()),
            created_at_epoch_seconds: 1_700_000_000,
        };
        let wrong_record = RetentionRecordRef::new(wrong_tenant.clone()).unwrap();
        assert_eq!(
            RetentionDecision::evaluate(
                decision_input(RetentionRequestKind::ScheduledExpiry, 1_701_000_000),
                &policy(
                    RetentionLawfulBasis::Contract,
                    RetentionDisposition::KmsShred
                ),
                &wrong_record,
                &[],
            ),
            Err(RetentionError::PolicyRecordMismatch)
        );

        wrong_tenant.tenant_id = "tenant-1".into();
        let record = RetentionRecordRef::new(wrong_tenant).unwrap();
        let unrelated_hold = LegalHold::new(LegalHoldCreate {
            hold_id: "legal-hold-2".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            scope: LegalHoldScope::Record {
                surface: WorkspaceRetentionSurface::Mail,
                record_id: "other-message".into(),
            },
            authority_ref: "matter-2026-002".into(),
            reason_ref: "litigation-preservation".into(),
            imposed_by_actor_ref: "dpo-1".into(),
            imposed_at_epoch_seconds: 1_700_000_100,
            released_at_epoch_seconds: None,
            released_by_actor_ref: None,
            release_evidence_hash: None,
        })
        .unwrap();
        assert_eq!(
            RetentionDecision::evaluate(
                decision_input(RetentionRequestKind::ScheduledExpiry, 1_701_000_000),
                &policy(
                    RetentionLawfulBasis::Contract,
                    RetentionDisposition::KmsShred
                ),
                &record,
                &[unrelated_hold],
            ),
            Err(RetentionError::HoldRecordMismatch)
        );
    }

    #[test]
    fn regulated_records_require_kms_shred_policy() {
        assert_eq!(
            RetentionDecision::evaluate(
                decision_input(RetentionRequestKind::DsrErase, 1_701_000_000),
                &policy(
                    RetentionLawfulBasis::Contract,
                    RetentionDisposition::RecordDelete
                ),
                &record(privacy(DataClass::Pci)),
                &[],
            ),
            Err(RetentionError::HighRiskRecordRequiresKmsShred)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_retention_data_class_from_legacy(DataClass::Audit),
            Err(RetentionError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
        assert_eq!(
            default_workspace_retention_data_class().data_class(),
            DataClass::InternalOnly
        );
    }
}
