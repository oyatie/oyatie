use cell_placement::{
    AssuranceCompilerVersion, AssuranceGeneration, CellId, PlacementLocationV1, ReservationRefV1,
};

use crate::{
    BindingDigest32, BindingOperationKey, BindingPersistenceAuthorityV1, BindingProducerId,
    BindingProofConstructionError, BindingProofEnvelopeV1, BindingProofVerificationError,
    BindingProofVerifier, CapabilityParticipantId, TenantId,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransferEffectId(String);

impl TransferEffectId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, BindingProofConstructionError> {
        Err(BindingProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TransferDataClassV1 {
    Public,
    Internal,
    Confidential,
    Regulated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TransferPurposeV1 {
    InitialCopy,
    FinalDelta,
    RecoveryPreparation,
    DisasterRecovery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyTransferEffectV1 {
    pub ordinal: u64,
    pub effect_id: TransferEffectId,
    pub participant_id: CapabilityParticipantId,
    pub source_cell_id: CellId,
    pub target_cell_id: CellId,
    pub source: PlacementLocationV1,
    pub destination: PlacementLocationV1,
    pub destination_reservation: ReservationRefV1,
    pub data_class: TransferDataClassV1,
    pub purpose: TransferPurposeV1,
    pub prepared_effect_digest: BindingDigest32,
    pub maximum_bytes: u64,
    pub maximum_cost_microunits: u64,
    pub inclusion_path: Vec<BindingDigest32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyTransferAuthorizationPayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub assurance_generation: AssuranceGeneration,
    pub assurance_compiler_version: AssuranceCompilerVersion,
    pub assurance_requirements_digest: BindingDigest32,
    pub effect: ResidencyTransferEffectV1,
    pub legal_basis_digest: BindingDigest32,
    pub consent_receipt_digest: Option<BindingDigest32>,
    pub policy_decision_digest: BindingDigest32,
    pub transport_policy_digest: BindingDigest32,
    pub destination_key_custody_digest: BindingDigest32,
    pub audit_receipt_digest: BindingDigest32,
    pub movement_permit_digest: BindingDigest32,
    pub scheduling_permit_id: String,
    pub parent_deadline_unix_seconds: u64,
    pub worker_id: String,
    pub worker_lease_epoch: u64,
    pub worker_lease_expires_at_unix_seconds: u64,
    pub ordinary_budget_relation_digest: BindingDigest32,
    pub forward_completion_reserve_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedResidencyTransferAuthorizationV1 {
    pub payload: ResidencyTransferAuthorizationPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyTransferExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub assurance_generation: AssuranceGeneration,
    pub assurance_compiler_version: AssuranceCompilerVersion,
    pub assurance_requirements_digest: BindingDigest32,
    pub effect: ResidencyTransferEffectV1,
    pub movement_permit_digest: BindingDigest32,
    pub scheduling_permit_id: String,
    pub worker_id: String,
    pub worker_lease_epoch: u64,
    pub maximum_effect_inclusion_path_depth: u32,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedResidencyTransferAuthorization(SignedResidencyTransferAuthorizationV1);

impl VerifiedResidencyTransferAuthorization {
    #[must_use]
    pub fn signed(&self) -> &SignedResidencyTransferAuthorizationV1 {
        &self.0
    }
}

pub fn verify_residency_transfer_authorization(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedResidencyTransferAuthorizationV1,
    _expectation: &ResidencyTransferExpectationV1,
) -> Result<VerifiedResidencyTransferAuthorization, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransferAuthorizationJournalRevision(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyTransferAuthorizationSetPayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub assurance_generation: AssuranceGeneration,
    pub assurance_compiler_version: AssuranceCompilerVersion,
    pub assurance_requirements_digest: BindingDigest32,
    pub required_effect_manifest_digest: BindingDigest32,
    pub ordered_authorization_root_digest: BindingDigest32,
    pub authorization_count: u64,
    pub journal_revision: TransferAuthorizationJournalRevision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedResidencyTransferAuthorizationSetV1 {
    pub payload: ResidencyTransferAuthorizationSetPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidencyTransferAuthorizationSetExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub assurance_generation: AssuranceGeneration,
    pub assurance_compiler_version: AssuranceCompilerVersion,
    pub assurance_requirements_digest: BindingDigest32,
    pub required_effect_manifest_digest: BindingDigest32,
    pub ordered_authorization_root_digest: BindingDigest32,
    pub authorization_count: u64,
    pub journal_revision: TransferAuthorizationJournalRevision,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedResidencyTransferAuthorizationSet(SignedResidencyTransferAuthorizationSetV1);

impl VerifiedResidencyTransferAuthorizationSet {
    #[must_use]
    pub fn signed(&self) -> &SignedResidencyTransferAuthorizationSetV1 {
        &self.0
    }
}

pub fn verify_residency_transfer_authorization_set(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedResidencyTransferAuthorizationSetV1,
    _expectation: &ResidencyTransferAuthorizationSetExpectationV1,
) -> Result<VerifiedResidencyTransferAuthorizationSet, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

pub trait ResidencyTransferAuthority: Send + Sync {
    fn authorize_effect<'a>(
        &'a self,
        authority: &'a BindingPersistenceAuthorityV1,
        expectation: &'a ResidencyTransferExpectationV1,
    ) -> crate::BoxTenancyFuture<
        'a,
        Result<SignedResidencyTransferAuthorizationV1, BindingProofVerificationError>,
    >;

    fn seal_authorization_set<'a>(
        &'a self,
        authority: &'a BindingPersistenceAuthorityV1,
        expectation: &'a ResidencyTransferAuthorizationSetExpectationV1,
    ) -> crate::BoxTenancyFuture<
        'a,
        Result<SignedResidencyTransferAuthorizationSetV1, BindingProofVerificationError>,
    >;
}
