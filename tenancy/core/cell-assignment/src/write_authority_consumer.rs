use cell_placement::CellId;

use crate::{
    BindingDigest32, BindingGeneration, BindingProofVerificationError, BindingRevision,
    BindingStoreError, BoxTenancyFuture, CapabilityParticipantId, TenantId,
    VerifiedSourceFenceDirective, VerifiedWriteAuthorityLease, VerifiedWriteAuthorityToken,
    WriteAuthorityEpoch,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapabilityWriteAuthorityRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CapabilityWriteAuthorityDispositionV1 {
    Writable,
    Fenced,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityWriteAuthorityStateV1 {
    tenant_id: TenantId,
    participant_id: CapabilityParticipantId,
    cell_id: CellId,
    binding_generation: BindingGeneration,
    binding_revision: BindingRevision,
    write_authority_epoch: WriteAuthorityEpoch,
    disposition: CapabilityWriteAuthorityDispositionV1,
    participant_manifest_digest: BindingDigest32,
    write_authority_lease_digest: BindingDigest32,
    authority_expires_at_unix_seconds: u64,
    revision: CapabilityWriteAuthorityRevision,
    last_authority_proof_digest: BindingDigest32,
    record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityWriteAuthorityStatePartsV1 {
    pub tenant_id: TenantId,
    pub participant_id: CapabilityParticipantId,
    pub cell_id: CellId,
    pub binding_generation: BindingGeneration,
    pub binding_revision: BindingRevision,
    pub write_authority_epoch: WriteAuthorityEpoch,
    pub disposition: CapabilityWriteAuthorityDispositionV1,
    pub participant_manifest_digest: BindingDigest32,
    pub write_authority_lease_digest: BindingDigest32,
    pub authority_expires_at_unix_seconds: u64,
    pub revision: CapabilityWriteAuthorityRevision,
    pub last_authority_proof_digest: BindingDigest32,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CapabilityWriteAuthorityStateConstructionErrorV1 {
    NotImplemented,
    InvalidRevision,
    InvalidAuthorityRelation,
    InvalidDisposition,
    RecordDigestMismatch,
}

impl CapabilityWriteAuthorityStateV1 {
    pub fn rehydrate(
        _parts: CapabilityWriteAuthorityStatePartsV1,
    ) -> Result<Self, CapabilityWriteAuthorityStateConstructionErrorV1> {
        Err(CapabilityWriteAuthorityStateConstructionErrorV1::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> CapabilityWriteAuthorityStatePartsRefV1<'_> {
        CapabilityWriteAuthorityStatePartsRefV1 {
            tenant_id: &self.tenant_id,
            participant_id: &self.participant_id,
            cell_id: &self.cell_id,
            binding_generation: self.binding_generation,
            binding_revision: self.binding_revision,
            write_authority_epoch: self.write_authority_epoch,
            disposition: self.disposition,
            participant_manifest_digest: self.participant_manifest_digest,
            write_authority_lease_digest: self.write_authority_lease_digest,
            authority_expires_at_unix_seconds: self.authority_expires_at_unix_seconds,
            revision: self.revision,
            last_authority_proof_digest: self.last_authority_proof_digest,
            record_digest: self.record_digest,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityWriteAuthorityStatePartsRefV1<'a> {
    pub tenant_id: &'a TenantId,
    pub participant_id: &'a CapabilityParticipantId,
    pub cell_id: &'a CellId,
    pub binding_generation: BindingGeneration,
    pub binding_revision: BindingRevision,
    pub write_authority_epoch: WriteAuthorityEpoch,
    pub disposition: CapabilityWriteAuthorityDispositionV1,
    pub participant_manifest_digest: BindingDigest32,
    pub write_authority_lease_digest: BindingDigest32,
    pub authority_expires_at_unix_seconds: u64,
    pub revision: CapabilityWriteAuthorityRevision,
    pub last_authority_proof_digest: BindingDigest32,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapabilityWriteAuthorityPreconditionV1 {
    Absent,
    Matches {
        revision: CapabilityWriteAuthorityRevision,
        binding_generation: BindingGeneration,
        write_authority_epoch: WriteAuthorityEpoch,
        record_digest: BindingDigest32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum CapabilityWriteAuthorityTransitionEvidenceV1 {
    Activate(Box<CapabilityWriteAuthorityActivationEvidenceV1>),
    FenceSource(Box<VerifiedSourceFenceDirective>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct CapabilityWriteAuthorityActivationEvidenceV1 {
    pub lease: VerifiedWriteAuthorityLease,
    pub token: VerifiedWriteAuthorityToken,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CapabilityWriteAuthorityTransitionWriteSetV1 {
    parts: CapabilityWriteAuthorityTransitionWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CapabilityWriteAuthorityTransitionWriteSetPartsV1 {
    pub precondition: CapabilityWriteAuthorityPreconditionV1,
    pub evidence: CapabilityWriteAuthorityTransitionEvidenceV1,
    pub next: CapabilityWriteAuthorityStateV1,
    pub drain_mutation: cell_placement::DrainContributorStateMutationV1,
    pub local_idempotency_digest: BindingDigest32,
    pub local_proof_consumption_digest: BindingDigest32,
    pub local_audit_record_digest: BindingDigest32,
}

impl CapabilityWriteAuthorityTransitionWriteSetV1 {
    pub fn assemble(
        _parts: CapabilityWriteAuthorityTransitionWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &CapabilityWriteAuthorityTransitionWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CapabilityAuthorizedWriteSetV1 {
    parts: CapabilityAuthorizedWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CapabilityAuthorizedWriteSetPartsV1 {
    pub authority: VerifiedWriteAuthorityToken,
    pub expected_authority_state: CapabilityWriteAuthorityStateV1,
    pub write_attempt_at_unix_seconds: u64,
    pub local_effect_digest: BindingDigest32,
    pub drain_mutation: cell_placement::DrainContributorStateMutationV1,
    pub local_idempotency_digest: BindingDigest32,
    pub local_proof_consumption_digest: BindingDigest32,
    pub local_audit_record_digest: BindingDigest32,
}

impl CapabilityAuthorizedWriteSetV1 {
    pub fn assemble(
        _parts: CapabilityAuthorizedWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &CapabilityAuthorizedWriteSetPartsV1 {
        &self.parts
    }
}

pub fn verify_capability_authority_transition(
    _precondition: &CapabilityWriteAuthorityPreconditionV1,
    _evidence: &CapabilityWriteAuthorityTransitionEvidenceV1,
    _next: &CapabilityWriteAuthorityStateV1,
    _now_unix_seconds: u64,
) -> Result<(), BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

pub trait CapabilityWriteAuthorityStore: Send + Sync {
    fn apply_transition<'a>(
        &'a self,
        write_set: &'a CapabilityWriteAuthorityTransitionWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<CapabilityWriteAuthorityStateV1, BindingStoreError>>;

    fn consume_before_write<'a>(
        &'a self,
        write_set: &'a CapabilityAuthorizedWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<BindingDigest32, BindingStoreError>>;
}
