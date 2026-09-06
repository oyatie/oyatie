use cell_placement::CellId;

use crate::{
    BindingAuditRecordV1, BindingDigest32, BindingGeneration, BindingIdempotencyKey,
    BindingIdempotencyRecordV1, BindingOperationKey, BindingOperationRevision, BindingOperationV1,
    BindingPersistenceAuthorityV1, BindingProofConsumptionV1,
    BindingReconciliationPersistenceAuthorityV1, BindingRevision, BindingStoreError,
    BindingWritePrecondition, BoxTenancyFuture, SignedWriteAuthorityLeaseV1, TenantId,
    VerifiedBindingInvocation, VerifiedCommittedWriteAuthorityLeaseIssuance,
    VerifiedWriteAuthorityLease, WriteAuthorityEpoch, WriteAuthorityLeaseIssuancePreconditionV1,
    WriteAuthorityLeaseIssuanceRecordV1, WriteAuthorityLeaseStatePreconditionV1,
    WriteAuthorityLeaseStateV1,
};

#[derive(Debug, Eq, PartialEq)]
pub enum BindingWriteAuthorityLeaseMutationV1 {
    Initialize(Box<BindingWriteAuthorityLeaseInitializeV1>),
    Move(Box<BindingWriteAuthorityLeaseMoveV1>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingWriteAuthorityLeaseInitializeV1 {
    pub target_lease_issuance: WriteAuthorityLeaseIssuanceRecordV1,
    pub active_target_state: WriteAuthorityLeaseStateV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BindingWriteAuthorityLeaseMoveV1 {
    pub source_precondition: WriteAuthorityLeaseStatePreconditionV1,
    pub retired_source_state: WriteAuthorityLeaseStateV1,
    pub target_lease_issuance: WriteAuthorityLeaseIssuanceRecordV1,
    pub active_target_state: WriteAuthorityLeaseStateV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WriteAuthorityLeaseFreezeV1 {
    pub source_precondition: WriteAuthorityLeaseStatePreconditionV1,
    pub frozen_source_state: WriteAuthorityLeaseStateV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RenewWriteAuthorityLeaseRequestV1 {
    pub operation: BindingOperationKey,
    pub expected_cell_id: CellId,
    pub expected_generation: BindingGeneration,
    pub expected_binding_revision: BindingRevision,
    pub expected_binding_record_digest: BindingDigest32,
    pub expected_lease_state: WriteAuthorityLeaseStatePreconditionV1,
    pub requested_validity_seconds: u64,
    pub idempotency_key: BindingIdempotencyKey,
    pub canonical_request_digest: BindingDigest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WriteAuthorityLeaseRenewalWriteSetV1 {
    parts: WriteAuthorityLeaseRenewalWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WriteAuthorityLeaseRenewalWriteSetPartsV1 {
    pub authority: BindingPersistenceAuthorityV1,
    pub binding_precondition: BindingWritePrecondition,
    pub lease_state_precondition: WriteAuthorityLeaseStatePreconditionV1,
    pub next_lease_state: WriteAuthorityLeaseStateV1,
    pub lease_issuance: WriteAuthorityLeaseIssuanceRecordV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl WriteAuthorityLeaseRenewalWriteSetV1 {
    pub fn assemble(
        _parts: WriteAuthorityLeaseRenewalWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &WriteAuthorityLeaseRenewalWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WriteAuthorityLeaseRenewalResultV1 {
    pub lease_issuance: WriteAuthorityLeaseIssuanceRecordV1,
    pub state: WriteAuthorityLeaseStateV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PublishWriteAuthorityLeaseRequestV1 {
    pub operation: BindingOperationKey,
    pub lease_digest: BindingDigest32,
    pub expected_issuance: WriteAuthorityLeaseIssuancePreconditionV1,
    pub idempotency_key: BindingIdempotencyKey,
    pub canonical_request_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedWriteAuthorityLeaseIssuanceQueryV1 {
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub binding_generation: BindingGeneration,
    pub write_authority_epoch: WriteAuthorityEpoch,
    pub lease_digest: BindingDigest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WriteAuthorityLeasePublicationWriteSetV1 {
    parts: WriteAuthorityLeasePublicationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WriteAuthorityLeasePublicationAuthorityV1 {
    Request(BindingPersistenceAuthorityV1),
    Reconciler(BindingReconciliationPersistenceAuthorityV1),
}

#[derive(Debug, Eq, PartialEq)]
pub struct WriteAuthorityLeasePublicationWriteSetPartsV1 {
    pub authority: WriteAuthorityLeasePublicationAuthorityV1,
    pub issuance_precondition: WriteAuthorityLeaseIssuancePreconditionV1,
    pub committed_issuance: VerifiedCommittedWriteAuthorityLeaseIssuance,
    pub lease: VerifiedWriteAuthorityLease,
    pub published_issuance: WriteAuthorityLeaseIssuanceRecordV1,
    pub effect: WriteAuthorityLeasePublicationEffectV1,
    pub idempotency: BindingIdempotencyRecordV1,
    pub audit_outbox: BindingAuditRecordV1,
    pub proof_consumptions: Vec<BindingProofConsumptionV1>,
}

impl WriteAuthorityLeasePublicationWriteSetV1 {
    pub fn assemble(
        _parts: WriteAuthorityLeasePublicationWriteSetPartsV1,
    ) -> Result<Self, BindingStoreError> {
        Err(BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &WriteAuthorityLeasePublicationWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WriteAuthorityLeasePublicationOperationV1 {
    pub expected_operation_revision: BindingOperationRevision,
    pub operation: BindingOperationV1,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WriteAuthorityLeasePublicationEffectV1 {
    BindingActivation(Box<WriteAuthorityLeasePublicationOperationV1>),
    Renewal,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PublishedWriteAuthorityLeaseV1 {
    pub lease: SignedWriteAuthorityLeaseV1,
    pub issuance: WriteAuthorityLeaseIssuanceRecordV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct WriteAuthorityLeasePublicationResultV1 {
    pub published: PublishedWriteAuthorityLeaseV1,
    pub operation: Option<BindingOperationV1>,
}

pub trait WriteAuthorityLeaseAuthority: Send + Sync {
    fn sign_committed<'a>(
        &'a self,
        issuance: &'a VerifiedCommittedWriteAuthorityLeaseIssuance,
    ) -> BoxTenancyFuture<'a, Result<VerifiedWriteAuthorityLease, BindingStoreError>>;
}

pub trait TenancyWriteAuthorityLeaseService: Send + Sync {
    fn get_latest_published_write_authority_lease<'a>(
        &'a self,
        invocation: VerifiedBindingInvocation,
        tenant_id: &'a TenantId,
        cell_id: &'a CellId,
        generation: BindingGeneration,
        epoch: WriteAuthorityEpoch,
        minimum_valid_until_unix_seconds: u64,
    ) -> BoxTenancyFuture<'a, Result<PublishedWriteAuthorityLeaseV1, crate::BindingContractError>>;

    fn renew_write_authority_lease<'a>(
        &'a self,
        invocation: VerifiedBindingInvocation,
        request: RenewWriteAuthorityLeaseRequestV1,
    ) -> BoxTenancyFuture<'a, Result<WriteAuthorityLeaseRenewalResultV1, crate::BindingContractError>>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NotImplementedTenancyWriteAuthorityLeaseService;

impl TenancyWriteAuthorityLeaseService for NotImplementedTenancyWriteAuthorityLeaseService {
    fn get_latest_published_write_authority_lease<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: &'a TenantId,
        _: &'a CellId,
        _: BindingGeneration,
        _: WriteAuthorityEpoch,
        _: u64,
    ) -> BoxTenancyFuture<'a, Result<PublishedWriteAuthorityLeaseV1, crate::BindingContractError>>
    {
        Box::pin(async { Err(crate::BindingContractError::NotImplemented) })
    }

    fn renew_write_authority_lease<'a>(
        &'a self,
        _: VerifiedBindingInvocation,
        _: RenewWriteAuthorityLeaseRequestV1,
    ) -> BoxTenancyFuture<'a, Result<WriteAuthorityLeaseRenewalResultV1, crate::BindingContractError>>
    {
        Box::pin(async { Err(crate::BindingContractError::NotImplemented) })
    }
}
