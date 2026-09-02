use crate::{
    BoxCellFuture, CellId, Digest32, PlacementAuditRecordV1, PlacementContractError,
    PlacementOperationKey, PlacementOperationRevision, PlacementOperationV1,
    PlacementReconciliationPersistenceAuthorityV1, PlacementReconciliationReadAuthorityV1,
    ReservationRefV1, ReservationStatusV1,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CellReconciliationPartitionKey(String);

impl CellReconciliationPartitionKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, crate::ProofConstructionError> {
        Err(crate::ProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellReconciliationPageTokenV1(Vec<u8>);

impl CellReconciliationPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellReconciliationWorkClassV1 {
    TentativeReservationExpired,
    AwaitingBindingOutcome,
    PendingReservationRelease,
    NonterminalOperation,
    PendingMovementPermitPublication,
    PendingMovementBudgetSettlement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellReconciliationQueryV1 {
    pub partition: CellReconciliationPartitionKey,
    pub work_class: CellReconciliationWorkClassV1,
    pub changed_before_unix_seconds: u64,
    pub page_size: u32,
    pub page_token: Option<CellReconciliationPageTokenV1>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CellReconciliationSubjectV1 {
    Reservation {
        reference: ReservationRefV1,
        expected_revision: u64,
        status: ReservationStatusV1,
    },
    Operation {
        key: PlacementOperationKey,
        expected_revision: PlacementOperationRevision,
        operation: PlacementOperationV1,
    },
    MovementPermitIssuance {
        authority_partition: crate::MovementBudgetAuthorityPartition,
        issuance_record_digest: Digest32,
        expected_revision: crate::MovementPermitIssuanceRevision,
        expected_status: crate::MovementPermitIssuanceStatusV1,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellReconciliationCandidateV1 {
    pub cell_id: CellId,
    pub work_class: CellReconciliationWorkClassV1,
    pub subject: CellReconciliationSubjectV1,
    pub durable_checkpoint_digest: Digest32,
    pub candidate_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellReconciliationPageV1 {
    pub candidates: Vec<CellReconciliationCandidateV1>,
    pub next_page_token: Option<CellReconciliationPageTokenV1>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ReconciliationWorkerId(String);

impl ReconciliationWorkerId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, crate::ProofConstructionError> {
        Err(crate::ProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellReconciliationLeaseV1 {
    pub candidate_digest: Digest32,
    pub worker_id: ReconciliationWorkerId,
    pub lease_epoch: u64,
    pub not_before_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub lease_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClaimCellReconciliationWriteSetV1 {
    parts: ClaimCellReconciliationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClaimCellReconciliationWriteSetPartsV1 {
    pub authority: PlacementReconciliationPersistenceAuthorityV1,
    pub candidate: CellReconciliationCandidateV1,
    pub expected_lease_epoch: Option<u64>,
    pub lease: CellReconciliationLeaseV1,
    pub drain_mutations: crate::DrainContributorMutationSetV1,
    pub audit_outbox: PlacementAuditRecordV1,
}

impl ClaimCellReconciliationWriteSetV1 {
    pub fn assemble(
        _parts: ClaimCellReconciliationWriteSetPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &ClaimCellReconciliationWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteCellReconciliationV1 {
    pub candidate_digest: Digest32,
    pub lease_digest: Digest32,
    pub applied_checkpoint_digest: Digest32,
    pub result_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompleteCellReconciliationWriteSetV1 {
    parts: CompleteCellReconciliationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompleteCellReconciliationWriteSetPartsV1 {
    pub authority: PlacementReconciliationPersistenceAuthorityV1,
    pub expected_lease: CellReconciliationLeaseV1,
    pub completion: CompleteCellReconciliationV1,
    pub drain_mutations: crate::DrainContributorMutationSetV1,
    pub audit_outbox: PlacementAuditRecordV1,
}

impl CompleteCellReconciliationWriteSetV1 {
    pub fn assemble(
        _parts: CompleteCellReconciliationWriteSetPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &CompleteCellReconciliationWriteSetPartsV1 {
        &self.parts
    }
}

pub trait CellReconciliationStore: Send + Sync {
    fn list_candidates<'a>(
        &'a self,
        authority: &'a PlacementReconciliationReadAuthorityV1,
        query: &'a CellReconciliationQueryV1,
    ) -> BoxCellFuture<'a, Result<CellReconciliationPageV1, PlacementContractError>>;

    fn claim<'a>(
        &'a self,
        write_set: &'a ClaimCellReconciliationWriteSetV1,
    ) -> BoxCellFuture<'a, Result<CellReconciliationLeaseV1, PlacementContractError>>;

    fn complete<'a>(
        &'a self,
        write_set: &'a CompleteCellReconciliationWriteSetV1,
    ) -> BoxCellFuture<'a, Result<CompleteCellReconciliationV1, PlacementContractError>>;
}
