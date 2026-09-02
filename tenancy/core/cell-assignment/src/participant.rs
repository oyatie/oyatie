use cell_placement::{
    BindingOutcomeQueryRefV1, CellId, SignedBindingParticipantManifestCommitmentV1,
};

use crate::{
    BindingDigest32, BindingGeneration, BindingOperationKey, BindingProducerId,
    BindingProofConstructionError, BindingProofEnvelopeV1, BindingProofVerificationError,
    BindingProofVerifier, BindingRevision, CapabilityParticipantId, TenantId, WriteAuthorityEpoch,
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

            pub fn parse(_value: impl Into<String>) -> Result<Self, BindingProofConstructionError> {
                Err(BindingProofConstructionError::NotImplemented)
            }
        }
    };
}

opaque_id!(CapabilityInventoryAuthorityId);
opaque_id!(CapabilityInventoryRepositoryId);
opaque_id!(CapabilityInventoryObjectId);
opaque_id!(CapabilityId);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInventorySnapshotRefV1 {
    pub authority_id: CapabilityInventoryAuthorityId,
    pub repository_id: CapabilityInventoryRepositoryId,
    pub object_id: CapabilityInventoryObjectId,
    pub object_version: u64,
    pub content_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParticipantManifestBasisV1 {
    InitialBinding {
        tenant_birth_reference_digest: BindingDigest32,
        staged_tenant_record_digest: BindingDigest32,
        assurance_requirements_digest: BindingDigest32,
        capability_requirements_digest: BindingDigest32,
    },
    ExistingBinding {
        binding_record_digest: BindingDigest32,
        binding_generation: BindingGeneration,
        binding_revision: BindingRevision,
        predecessor_manifest_digest: BindingDigest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantManifestCoverageV1 {
    pub required_writable_capability_root_digest: BindingDigest32,
    pub required_writable_capability_count: u64,
    pub covered_writable_capability_root_digest: BindingDigest32,
    pub covered_writable_capability_count: u64,
    pub coverage_proof_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantManifestPayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub basis: ParticipantManifestBasisV1,
    pub inventory_snapshot: CapabilityInventorySnapshotRefV1,
    pub compiler_version: String,
    pub ordered_participant_root_digest: BindingDigest32,
    pub participant_count: u64,
    pub coverage: ParticipantManifestCoverageV1,
    pub member_snapshot: crate::ParticipantManifestMemberSnapshotV1,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedParticipantManifestV1 {
    pub payload: ParticipantManifestPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParticipantWriteRoleV1 {
    SourceAndTargetWriter,
    SourceFenceOnly,
    TargetWriterOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantManifestMemberV1 {
    pub ordinal: u64,
    pub participant_id: CapabilityParticipantId,
    pub capability_id: CapabilityId,
    pub role: ParticipantWriteRoleV1,
    pub member_digest: BindingDigest32,
    pub inclusion_path: Vec<BindingDigest32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantManifestExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub basis: ParticipantManifestBasisV1,
    pub inventory_snapshot: CapabilityInventorySnapshotRefV1,
    pub member_snapshot: crate::ParticipantManifestMemberSnapshotV1,
    pub maximum_inclusion_path_depth: u32,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedParticipantManifest(SignedParticipantManifestV1);

impl VerifiedParticipantManifest {
    #[must_use]
    pub fn signed(&self) -> &SignedParticipantManifestV1 {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedParticipantManifestMember(ParticipantManifestMemberV1);

impl VerifiedParticipantManifestMember {
    #[must_use]
    pub fn member(&self) -> &ParticipantManifestMemberV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantManifestCommitmentExpectationV1 {
    pub binding_operation: BindingOutcomeQueryRefV1,
    pub binding_attempt_digest: BindingDigest32,
    pub committed_at_unix_seconds: u64,
}

pub trait ParticipantManifestCommitmentAuthority: Send + Sync {
    fn issue_cell_commitment<'a>(
        &'a self,
        authority: &'a crate::BindingWorkSnapshotMutationAuthorityV1,
        manifest: &'a VerifiedParticipantManifest,
        expectation: &'a ParticipantManifestCommitmentExpectationV1,
    ) -> crate::BoxTenancyFuture<
        'a,
        Result<SignedBindingParticipantManifestCommitmentV1, BindingProofVerificationError>,
    >;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParticipantReceiptPhaseV1 {
    Prepared,
    SourceFenced,
    TargetActivated,
    SourceReleased,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParticipantReceiptEvidenceV1 {
    Prepared {
        preparation_digest: BindingDigest32,
    },
    SourceFenced {
        source_fence_directive_digest: BindingDigest32,
        fenced_authority_state_digest: BindingDigest32,
        write_authority_epoch: WriteAuthorityEpoch,
    },
    TargetActivated {
        binding_record_digest: BindingDigest32,
        generation: BindingGeneration,
        revision: BindingRevision,
        write_authority_epoch: WriteAuthorityEpoch,
        write_token_digest: BindingDigest32,
        write_authority_lease_digest: BindingDigest32,
        activated_authority_state_digest: BindingDigest32,
    },
    SourceReleased {
        successor_binding_record_digest: BindingDigest32,
        release_digest: BindingDigest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParticipantPlacementContextV1 {
    InitialBinding {
        target_cell_id: CellId,
        binding_attempt_digest: BindingDigest32,
        placement_decision_digest: BindingDigest32,
        reservation_commit_permit_digest: BindingDigest32,
    },
    MigrationPreparation {
        source_cell_id: CellId,
        target_cell_id: CellId,
        source_generation: BindingGeneration,
        source_revision: BindingRevision,
        source_binding_record_digest: BindingDigest32,
        binding_attempt_digest: BindingDigest32,
        placement_decision_digest: BindingDigest32,
        reservation_commit_permit_digest: BindingDigest32,
    },
    Migration {
        source_cell_id: CellId,
        target_cell_id: CellId,
        source_generation: BindingGeneration,
        source_revision: BindingRevision,
        source_binding_record_digest: BindingDigest32,
        migration_fence_claim_digest: BindingDigest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptPayloadV1 {
    pub schema_version: u32,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub placement_context: ParticipantPlacementContextV1,
    pub participant_manifest_digest: BindingDigest32,
    pub member: ParticipantManifestMemberV1,
    pub phase: ParticipantReceiptPhaseV1,
    pub evidence: ParticipantReceiptEvidenceV1,
    pub occurred_at_unix_seconds: u64,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedParticipantReceiptV1 {
    pub payload: ParticipantReceiptPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedParticipantReceipt(SignedParticipantReceiptV1);

impl VerifiedParticipantReceipt {
    #[must_use]
    pub fn signed(&self) -> &SignedParticipantReceiptV1 {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParticipantReceiptLedgerRevision(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptLedgerV1 {
    pub operation: BindingOperationKey,
    pub phase: ParticipantReceiptPhaseV1,
    pub manifest_digest: BindingDigest32,
    pub placement_context_digest: BindingDigest32,
    pub expected_participant_root_digest: BindingDigest32,
    pub expected_participant_count: u64,
    pub next_participant_ordinal: u64,
    pub applied_participant_root_digest: BindingDigest32,
    pub applied_receipt_root_digest: BindingDigest32,
    pub revision: ParticipantReceiptLedgerRevision,
    pub record_digest: BindingDigest32,
}

pub fn verify_participant_manifest(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedParticipantManifestV1,
    _expectation: &ParticipantManifestExpectationV1,
) -> Result<VerifiedParticipantManifest, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

pub fn verify_participant_manifest_member(
    _manifest: &VerifiedParticipantManifest,
    _member: ParticipantManifestMemberV1,
    _maximum_path_depth: u32,
) -> Result<VerifiedParticipantManifestMember, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptExpectationV1 {
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub placement_context: ParticipantPlacementContextV1,
    pub manifest_digest: BindingDigest32,
    pub participant_id: CapabilityParticipantId,
    pub member_digest: BindingDigest32,
    pub phase: ParticipantReceiptPhaseV1,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ParticipantReceiptPhaseAuthorityV1<'a> {
    Prepared {
        attempt: &'a crate::BindingReservationAttemptV1,
        placement_decision: &'a cell_placement::VerifiedCellPlacementDecision,
        reservation_commit_permit: &'a cell_placement::VerifiedReservationCommitPermit,
    },
    SourceFenced {
        directive: &'a crate::VerifiedSourceFenceDirective,
    },
    TargetActivated {
        lease: &'a crate::VerifiedWriteAuthorityLease,
        token: &'a crate::VerifiedWriteAuthorityToken,
    },
    SourceReleased {
        source_fence_directive: &'a crate::VerifiedSourceFenceDirective,
        successor_lease: &'a crate::VerifiedWriteAuthorityLease,
        target_activation: &'a crate::VerifiedParticipantPhaseClosure,
        projection_convergence: &'a crate::VerifiedProjectionConvergence,
        rollback_window: &'a crate::VerifiedRollbackWindowElapsed,
    },
}

pub fn verify_participant_receipt(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedParticipantReceiptV1,
    _manifest: &VerifiedParticipantManifest,
    _phase_authority: &ParticipantReceiptPhaseAuthorityV1<'_>,
    _expectation: &ParticipantReceiptExpectationV1,
) -> Result<VerifiedParticipantReceipt, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
