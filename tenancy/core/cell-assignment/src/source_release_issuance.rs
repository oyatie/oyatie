use cell_placement::{
    SignedSourceReservationReleasePermitV1, SourceReservationReleaseIntentV1,
    VerifiedSourceReservationReleasePermit,
};

use crate::{
    BindingAuditRecordV1, BindingDigest32, BindingIdempotencyRecordV1, BindingOperationKey,
    BindingPersistenceAuthorityV1, BindingProducerId, BindingProofConsumptionV1,
    BindingProofEnvelopeV1, BindingProofVerificationError, BindingProofVerifier,
    BindingReconciliationLeaseV1, BindingReconciliationPersistenceAuthorityV1, BindingStoreError,
    BoxTenancyFuture, TenantCellBinding, TenantId, VerifiedRollbackWindowElapsed,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceReservationReleaseIssuanceRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceReservationReleaseIssuanceStatusV1 {
    PendingSignature,
    Published,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceReleaseCommitContextV1 {
    Request,
    Reconciliation {
        candidate_digest: BindingDigest32,
        lease_epoch: u64,
        lease_digest: BindingDigest32,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum SourceReleaseClaimContextV1 {
    Request,
    Reconciliation(BindingReconciliationLeaseV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReservationReleaseIssuanceRecordV1 {
    intent: SourceReservationReleaseIntentV1,
    status: SourceReservationReleaseIssuanceStatusV1,
    signed_permit: Option<SignedSourceReservationReleasePermitV1>,
    revision: SourceReservationReleaseIssuanceRevision,
    record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReservationReleaseIssuanceRecordPartsV1 {
    pub intent: SourceReservationReleaseIntentV1,
    pub status: SourceReservationReleaseIssuanceStatusV1,
    pub signed_permit: Option<SignedSourceReservationReleasePermitV1>,
    pub revision: SourceReservationReleaseIssuanceRevision,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceReservationReleaseIssuanceConstructionErrorV1 {
    NotImplemented,
    InvalidRevision,
    InvalidStatus,
    InvalidPermitRelation,
    RecordDigestMismatch,
}

impl SourceReservationReleaseIssuanceRecordV1 {
    pub fn rehydrate(
        _parts: SourceReservationReleaseIssuanceRecordPartsV1,
    ) -> Result<Self, SourceReservationReleaseIssuanceConstructionErrorV1> {
        Err(SourceReservationReleaseIssuanceConstructionErrorV1::NotImplemented)
    }

    #[must_use]
    pub fn intent(&self) -> &SourceReservationReleaseIntentV1 {
        &self.intent
    }

    #[must_use]
    pub fn status(&self) -> SourceReservationReleaseIssuanceStatusV1 {
        self.status
    }

    #[must_use]
    pub fn signed_permit(&self) -> Option<&SignedSourceReservationReleasePermitV1> {
        self.signed_permit.as_ref()
    }

    #[must_use]
    pub fn revision(&self) -> SourceReservationReleaseIssuanceRevision {
        self.revision
    }

    #[must_use]
    pub fn record_digest(&self) -> BindingDigest32 {
        self.record_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReservationReleaseIssuancePreconditionV1 {
    pub revision: SourceReservationReleaseIssuanceRevision,
    pub status: SourceReservationReleaseIssuanceStatusV1,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReleaseCommitAttestationPayloadV1 {
    pub schema_version: u32,
    pub tenant_id: TenantId,
    pub operation: BindingOperationKey,
    pub successor_binding_generation: crate::BindingGeneration,
    pub successor_binding_revision: crate::BindingRevision,
    pub successor_binding_record_digest: BindingDigest32,
    pub issuance_revision: SourceReservationReleaseIssuanceRevision,
    pub issuance_record_digest: BindingDigest32,
    pub rollback_window_elapsed_digest: BindingDigest32,
    pub context: SourceReleaseCommitContextV1,
    pub committed_transaction_digest: BindingDigest32,
    pub committed_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedSourceReleaseCommitAttestationV1 {
    pub payload: SourceReleaseCommitAttestationPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommittedSourceReservationReleaseIssuanceClaimV1 {
    pub successor_binding: TenantCellBinding,
    pub issuance: SourceReservationReleaseIssuanceRecordV1,
    pub rollback_window: VerifiedRollbackWindowElapsed,
    pub context: SourceReleaseClaimContextV1,
    pub attestation: SignedSourceReleaseCommitAttestationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReleaseCommitAttestationExpectationV1 {
    pub tenant_id: TenantId,
    pub operation: BindingOperationKey,
    pub successor_binding_generation: crate::BindingGeneration,
    pub successor_binding_revision: crate::BindingRevision,
    pub successor_binding_record_digest: BindingDigest32,
    pub issuance_revision: SourceReservationReleaseIssuanceRevision,
    pub issuance_record_digest: BindingDigest32,
    pub rollback_window_elapsed_digest: BindingDigest32,
    pub context: SourceReleaseCommitContextV1,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedCommittedSourceReservationReleaseIssuance(
    CommittedSourceReservationReleaseIssuanceClaimV1,
);

impl VerifiedCommittedSourceReservationReleaseIssuance {
    #[must_use]
    pub fn claim(&self) -> &CommittedSourceReservationReleaseIssuanceClaimV1 {
        &self.0
    }
}

pub fn verify_committed_source_release_issuance(
    _verifier: &dyn BindingProofVerifier,
    _claim: CommittedSourceReservationReleaseIssuanceClaimV1,
    _expectation: &SourceReleaseCommitAttestationExpectationV1,
) -> Result<VerifiedCommittedSourceReservationReleaseIssuance, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

#[derive(Debug, Eq, PartialEq)]
pub enum SourceReleasePublicationAuthorityV1 {
    Request(BindingPersistenceAuthorityV1),
    Reconciler(BindingReconciliationPersistenceAuthorityV1),
}

#[derive(Debug, Eq, PartialEq)]
pub struct SourceReleasePublicationWriteSetV1 {
    parts: SourceReleasePublicationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SourceReleasePublicationWriteSetPartsV1 {
    pub authority: SourceReleasePublicationAuthorityV1,
    pub issuance_precondition: SourceReservationReleaseIssuancePreconditionV1,
    pub committed_issuance: VerifiedCommittedSourceReservationReleaseIssuance,
    pub permit: VerifiedSourceReservationReleasePermit,
    pub published_issuance: SourceReservationReleaseIssuanceRecordV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl SourceReleasePublicationWriteSetV1 {
    pub fn assemble(
        _parts: SourceReleasePublicationWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &SourceReleasePublicationWriteSetPartsV1 {
        &self.parts
    }
}

pub trait MigrationReleaseStore: Send + Sync {
    fn commit_release_issuance<'a>(
        &'a self,
        write_set: &'a crate::MigrationReleaseWriteSetV1,
    ) -> BoxTenancyFuture<
        'a,
        Result<CommittedSourceReservationReleaseIssuanceClaimV1, BindingStoreError>,
    >;

    fn load_committed_release_issuance<'a>(
        &'a self,
        authority: &'a BindingReconciliationPersistenceAuthorityV1,
        operation: &'a BindingOperationKey,
        reconciliation_lease: &'a BindingReconciliationLeaseV1,
    ) -> BoxTenancyFuture<
        'a,
        Result<Option<CommittedSourceReservationReleaseIssuanceClaimV1>, BindingStoreError>,
    >;

    fn publish_release_permit<'a>(
        &'a self,
        write_set: &'a SourceReleasePublicationWriteSetV1,
    ) -> BoxTenancyFuture<'a, Result<SignedSourceReservationReleasePermitV1, BindingStoreError>>;
}

pub trait SourceReservationReleaseAuthority: Send + Sync {
    fn sign_committed<'a>(
        &'a self,
        issuance: &'a VerifiedCommittedSourceReservationReleaseIssuance,
    ) -> BoxTenancyFuture<'a, Result<VerifiedSourceReservationReleasePermit, BindingStoreError>>;
}
