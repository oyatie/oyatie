use crate::{
    AssuranceCompilerVersion, AssuranceGeneration, BoxCellFuture, CellId, CellProofEnvelopeV1,
    CellProofVerifier, Digest32, EncryptionRequirementV1, ExclusiveTenantKeyRequirementV1,
    HardwareClassV1, ImmutableAuditStorageRequirementV1, IsolationClassV1, PlacementLocationV1,
    PlacementOperationKey, PlacementReadAuthorityV1, ProducerId, ProofConstructionError,
    ProofVerificationError, ReservationRefV1, ResilienceObjectiveV1, TenantId,
};

macro_rules! opaque_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn parse(_value: impl Into<String>) -> Result<Self, ProofConstructionError> {
                Err(ProofConstructionError::NotImplemented)
            }
        }
    };
}

opaque_id!(EvidenceAuthorityId);
opaque_id!(EvidenceRepositoryId);
opaque_id!(EvidenceObjectId);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImmutableEvidenceRefV1 {
    pub authority_id: EvidenceAuthorityId,
    pub repository_id: EvidenceRepositoryId,
    pub object_id: EvidenceObjectId,
    pub object_version: u64,
    pub content_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSetCommitmentV1 {
    pub ordered_root_digest: Digest32,
    pub member_count: u64,
    pub immutable_object: ImmutableEvidenceRefV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSetMemberRefV1 {
    pub set: ImmutableEvidenceRefV1,
    pub ordinal: u64,
    pub member_digest: Digest32,
    pub inclusion_path: Vec<Digest32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityPlacementProofV1 {
    pub capability_id: String,
    pub cell_id: CellId,
    pub resilience_objective_digest: Digest32,
    pub placement_proof: ImmutableEvidenceRefV1,
    pub replication_proof: ImmutableEvidenceRefV1,
    pub capacity_proof: ImmutableEvidenceRefV1,
    pub member_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceEvidencePayloadV1 {
    pub schema_version: u32,
    pub operation: PlacementOperationKey,
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub location: PlacementLocationV1,
    pub assurance_generation: AssuranceGeneration,
    pub assurance_compiler_version: AssuranceCompilerVersion,
    pub assurance_requirements_digest: Digest32,
    pub resilience_objective: ResilienceObjectiveV1,
    pub resilience_proof: ImmutableEvidenceRefV1,
    pub required_certification_set: EvidenceSetCommitmentV1,
    pub certification_evidence_set: EvidenceSetCommitmentV1,
    pub required_capability_set: EvidenceSetCommitmentV1,
    pub capability_placement_proof_set: EvidenceSetCommitmentV1,
    pub isolation: IsolationClassV1,
    pub hardware: HardwareClassV1,
    pub encryption: EncryptionRequirementV1,
    pub key_custody_evidence_set: EvidenceSetCommitmentV1,
    pub exclusive_tenant_key: ExclusiveTenantKeyRequirementV1,
    pub minimum_audit_retention_seconds: u64,
    pub immutable_audit_storage: ImmutableAuditStorageRequirementV1,
    pub evidence_record: ImmutableEvidenceRefV1,
    pub observed_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardRecoveryEvidenceV1 {
    pub backup_storage_location: PlacementLocationV1,
    pub restore_staging_location: PlacementLocationV1,
    pub restore_location: PlacementLocationV1,
    pub backup_proof: ImmutableEvidenceRefV1,
    pub restore_exercise_proof: ImmutableEvidenceRefV1,
    pub capability_recovery_proof_set: EvidenceSetCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarmRecoveryEvidenceV1 {
    pub recovery_cell_id: CellId,
    pub recovery_location: PlacementLocationV1,
    pub recovery_reservation: ReservationRefV1,
    pub failure_independence_proof: ImmutableEvidenceRefV1,
    pub recovery_readiness_proof: ImmutableEvidenceRefV1,
    pub capability_recovery_proof_set: EvidenceSetCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryEvidenceModeV1 {
    Standard(StandardRecoveryEvidenceV1),
    Warm(WarmRecoveryEvidenceV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryEvidencePayloadV1 {
    pub schema_version: u32,
    pub operation: PlacementOperationKey,
    pub tenant_id: TenantId,
    pub home_cell_id: CellId,
    pub assurance_generation: AssuranceGeneration,
    pub assurance_compiler_version: AssuranceCompilerVersion,
    pub assurance_requirements_digest: Digest32,
    pub mode: RecoveryEvidenceModeV1,
    pub proven_rpo_seconds: u64,
    pub proven_rto_seconds: u64,
    pub evidence_record: ImmutableEvidenceRefV1,
    pub observed_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedAssuranceEvidenceV1 {
    pub payload: AssuranceEvidencePayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedRecoveryEvidenceV1 {
    pub payload: RecoveryEvidencePayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceEvidenceExpectationV1 {
    pub operation: PlacementOperationKey,
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub assurance_generation: AssuranceGeneration,
    pub assurance_compiler_version: AssuranceCompilerVersion,
    pub assurance_requirements_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryEvidenceExpectationV1 {
    pub operation: PlacementOperationKey,
    pub tenant_id: TenantId,
    pub home_cell_id: CellId,
    pub assurance_generation: AssuranceGeneration,
    pub assurance_compiler_version: AssuranceCompilerVersion,
    pub assurance_requirements_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedAssuranceEvidence(SignedAssuranceEvidenceV1);

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedRecoveryEvidence(SignedRecoveryEvidenceV1);

impl VerifiedAssuranceEvidence {
    #[must_use]
    pub fn signed(&self) -> &SignedAssuranceEvidenceV1 {
        &self.0
    }
}

impl VerifiedRecoveryEvidence {
    #[must_use]
    pub fn signed(&self) -> &SignedRecoveryEvidenceV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceAuditPolicyV1 {
    evidence_record: ImmutableEvidenceRefV1,
    requirements_digest: Digest32,
    minimum_retention_seconds: u64,
    immutable_storage: ImmutableAuditStorageRequirementV1,
}

impl AssuranceAuditPolicyV1 {
    #[must_use]
    pub fn evidence_record(&self) -> &ImmutableEvidenceRefV1 {
        &self.evidence_record
    }

    #[must_use]
    pub fn requirements_digest(&self) -> Digest32 {
        self.requirements_digest
    }

    #[must_use]
    pub fn minimum_retention_seconds(&self) -> u64 {
        self.minimum_retention_seconds
    }

    #[must_use]
    pub fn immutable_storage(&self) -> ImmutableAuditStorageRequirementV1 {
        self.immutable_storage
    }
}

pub fn verify_assurance_evidence(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedAssuranceEvidenceV1,
    _expectation: &AssuranceEvidenceExpectationV1,
) -> Result<VerifiedAssuranceEvidence, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn verify_recovery_evidence(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedRecoveryEvidenceV1,
    _expectation: &RecoveryEvidenceExpectationV1,
) -> Result<VerifiedRecoveryEvidence, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn derive_assurance_audit_policy(
    _evidence: &VerifiedAssuranceEvidence,
) -> Result<AssuranceAuditPolicyV1, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub trait PlacementEvidenceReader: Send + Sync {
    fn get_assurance_evidence<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        reference: &'a ImmutableEvidenceRefV1,
    ) -> BoxCellFuture<'a, Result<Option<SignedAssuranceEvidenceV1>, ProofVerificationError>>;

    fn get_recovery_evidence<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        reference: &'a ImmutableEvidenceRefV1,
    ) -> BoxCellFuture<'a, Result<Option<SignedRecoveryEvidenceV1>, ProofVerificationError>>;

    fn get_capability_placement_proof<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        reference: &'a EvidenceSetMemberRefV1,
        maximum_inclusion_path_depth: u32,
    ) -> BoxCellFuture<'a, Result<Option<CapabilityPlacementProofV1>, ProofVerificationError>>;
}
