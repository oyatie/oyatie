use crate::{
    BoxCellFuture, CellAdmissionEpoch, CellControlOperationKeyV1, CellControlOperationRevision,
    CellControlOperationV1, CellId, CellIdentityV1, CellLifecycleRevision, CellPageRequestV1,
    CellPageV1, CellReadinessRingV1, CellRevisionIdentityV1, CellSpecRevision, CellSpecV1,
    CellViewV1, Digest32, DrainProofLedgerV1, DrainTermV1, PlacementContractError,
    PlacementIdempotencyKey, PlacementPartitionV1, RebalanceJobId, RebalanceJobV1,
    RebalancePolicyV1, RebalanceTriggerV1, VerifiedCellControlInvocation,
    VerifiedCellDrainCompletion, VerifiedCellPromotionEvidence, VerifiedDrainContributorManifest,
    VerifiedDrainContributorProof,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateCellRequestV1 {
    pub identity: CellIdentityV1,
    pub spec: CellSpecV1,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RepairCellControlOperationRequestV1 {
    pub repair_operation: CellControlOperationKeyV1,
    pub target_operation: CellControlOperationKeyV1,
    pub target_precondition: crate::CellControlRepairTargetPreconditionV1,
    pub authority: crate::VerifiedCellControlRepairAuthority,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateCellRequestV1 {
    pub cell_id: CellId,
    pub expected_spec_revision: CellSpecRevision,
    pub expected_resource_digest: Digest32,
    pub expected_capacity_revision: crate::CellCapacityRevision,
    pub expected_capacity_record_digest: Digest32,
    pub spec: CellSpecV1,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct MutateCellReadinessRequestV1 {
    pub expected_cell: CellRevisionIdentityV1,
    pub target_readiness: CellReadinessRingV1,
    pub evidence: CellReadinessMutationEvidenceV1,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CellReadinessMutationEvidenceV1 {
    Promotion(Box<VerifiedCellPromotionEvidence>),
    Demotion { reason_digest: Digest32 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct BeginCellDrainRequestV1 {
    pub cell_id: CellId,
    pub expected_revision: CellLifecycleRevision,
    pub expected_admission_epoch: CellAdmissionEpoch,
    pub next_drain_term: DrainTermV1,
    pub contributor_manifest: VerifiedDrainContributorManifest,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AppendCellDrainProofRequestV1 {
    pub cell_id: CellId,
    pub drain_term: DrainTermV1,
    pub expected_ledger_revision: u64,
    pub proof: VerifiedDrainContributorProof,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompleteCellDrainRequestV1 {
    pub cell_id: CellId,
    pub drain_term: DrainTermV1,
    pub expected_revision: CellLifecycleRevision,
    pub expected_ledger_revision: u64,
    pub completion: VerifiedCellDrainCompletion,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DecommissionCellRequestV1 {
    pub cell_id: CellId,
    pub drain_term: DrainTermV1,
    pub expected_revision: CellLifecycleRevision,
    pub completion: VerifiedCellDrainCompletion,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateRebalanceJobRequestV1 {
    pub partition: PlacementPartitionV1,
    pub trigger: RebalanceTriggerV1,
    pub policy: RebalancePolicyV1,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetRebalanceJobRequestV1 {
    pub partition: PlacementPartitionV1,
    pub job_id: RebalanceJobId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelRebalanceJobRequestV1 {
    pub partition: PlacementPartitionV1,
    pub job_id: RebalanceJobId,
    pub expected_revision: u64,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GetCellControlOperationRequestV1 {
    pub operation: CellControlOperationKeyV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelCellControlOperationRequestV1 {
    pub operation: CellControlOperationKeyV1,
    pub expected_revision: CellControlOperationRevision,
    pub idempotency_key: PlacementIdempotencyKey,
    pub canonical_request_digest: Digest32,
}

pub trait CellControlService: Send + Sync {
    fn create<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: CreateCellRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn get<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        cell_id: &'a CellId,
    ) -> BoxCellFuture<'a, Result<CellViewV1, PlacementContractError>>;

    fn update<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: UpdateCellRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn list<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        page: &'a CellPageRequestV1,
    ) -> BoxCellFuture<'a, Result<CellPageV1, PlacementContractError>>;

    fn mutate_readiness<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: MutateCellReadinessRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn begin_drain<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: BeginCellDrainRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn append_drain_proof<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: AppendCellDrainProofRequestV1,
    ) -> BoxCellFuture<'a, Result<DrainProofLedgerV1, PlacementContractError>>;

    fn complete_drain<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: CompleteCellDrainRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn decommission<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: DecommissionCellRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn create_rebalance_job<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: CreateRebalanceJobRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn get_rebalance_job<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: &'a GetRebalanceJobRequestV1,
    ) -> BoxCellFuture<'a, Result<RebalanceJobV1, PlacementContractError>>;

    fn cancel_rebalance_job<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: CancelRebalanceJobRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn get_operation<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: &'a GetCellControlOperationRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn cancel_operation<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: CancelCellControlOperationRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;

    fn repair_operation<'a>(
        &'a self,
        invocation: VerifiedCellControlInvocation,
        request: RepairCellControlOperationRequestV1,
    ) -> BoxCellFuture<'a, Result<CellControlOperationV1, PlacementContractError>>;
}
