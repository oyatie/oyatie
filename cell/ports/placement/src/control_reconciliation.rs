use crate::{
    BoxCellFuture, CellControlAuditRecordV1, CellControlOperationV1,
    CellControlReconciliationPersistenceAuthorityV1, CellControlReconciliationReadAuthorityV1,
    CellId, Digest32, DrainProofLedgerV1, PlacementContractError, RebalanceJobV1,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CellControlReconciliationPartitionKey(String);

impl CellControlReconciliationPartitionKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, crate::ProofConstructionError> {
        Err(crate::ProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellControlReconciliationPageTokenV1(Vec<u8>);

impl CellControlReconciliationPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellControlReconciliationWorkClassV1 {
    NonterminalControlOperation,
    IncompleteDrainProof,
    NonterminalRebalanceJob,
    RebalanceCandidateSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellControlReconciliationQueryV1 {
    pub partition: CellControlReconciliationPartitionKey,
    pub work_class: CellControlReconciliationWorkClassV1,
    pub changed_before_unix_seconds: u64,
    pub page_size: u32,
    pub page_token: Option<CellControlReconciliationPageTokenV1>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CellControlReconciliationSubjectV1 {
    Operation(CellControlOperationV1),
    DrainLedger(DrainProofLedgerV1),
    RebalanceJob(Box<RebalanceJobV1>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellControlReconciliationCandidateV1 {
    pub cell_id: Option<CellId>,
    pub work_class: CellControlReconciliationWorkClassV1,
    pub subject: CellControlReconciliationSubjectV1,
    pub durable_checkpoint_digest: Digest32,
    pub candidate_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellControlReconciliationPageV1 {
    pub candidates: Vec<CellControlReconciliationCandidateV1>,
    pub next_page_token: Option<CellControlReconciliationPageTokenV1>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CellControlReconciliationWorkerId(String);

impl CellControlReconciliationWorkerId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, crate::ProofConstructionError> {
        Err(crate::ProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellControlReconciliationLeaseV1 {
    pub candidate_digest: Digest32,
    pub worker_id: CellControlReconciliationWorkerId,
    pub lease_epoch: u64,
    pub not_before_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub lease_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClaimCellControlReconciliationWriteSetV1 {
    parts: ClaimCellControlReconciliationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClaimCellControlReconciliationWriteSetPartsV1 {
    pub authority: CellControlReconciliationPersistenceAuthorityV1,
    pub candidate: CellControlReconciliationCandidateV1,
    pub expected_lease_epoch: Option<u64>,
    pub lease: CellControlReconciliationLeaseV1,
    pub drain_mutations: crate::DrainContributorMutationSetV1,
    pub audit_outbox: CellControlAuditRecordV1,
}

impl ClaimCellControlReconciliationWriteSetV1 {
    pub fn assemble(
        _parts: ClaimCellControlReconciliationWriteSetPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &ClaimCellControlReconciliationWriteSetPartsV1 {
        &self.parts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteCellControlReconciliationV1 {
    pub candidate_digest: Digest32,
    pub lease_digest: Digest32,
    pub applied_checkpoint_digest: Digest32,
    pub result_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompleteCellControlReconciliationWriteSetV1 {
    parts: CompleteCellControlReconciliationWriteSetPartsV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CompleteCellControlReconciliationWriteSetPartsV1 {
    pub authority: CellControlReconciliationPersistenceAuthorityV1,
    pub expected_lease: CellControlReconciliationLeaseV1,
    pub completion: CompleteCellControlReconciliationV1,
    pub drain_mutations: crate::DrainContributorMutationSetV1,
    pub audit_outbox: CellControlAuditRecordV1,
}

impl CompleteCellControlReconciliationWriteSetV1 {
    pub fn assemble(
        _parts: CompleteCellControlReconciliationWriteSetPartsV1,
    ) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn parts(&self) -> &CompleteCellControlReconciliationWriteSetPartsV1 {
        &self.parts
    }
}

pub trait CellControlReconciliationStore: Send + Sync {
    fn list_candidates<'a>(
        &'a self,
        authority: &'a CellControlReconciliationReadAuthorityV1,
        query: &'a CellControlReconciliationQueryV1,
    ) -> BoxCellFuture<'a, Result<CellControlReconciliationPageV1, PlacementContractError>>;

    fn claim<'a>(
        &'a self,
        write_set: &'a ClaimCellControlReconciliationWriteSetV1,
    ) -> BoxCellFuture<'a, Result<CellControlReconciliationLeaseV1, PlacementContractError>>;

    fn complete<'a>(
        &'a self,
        write_set: &'a CompleteCellControlReconciliationWriteSetV1,
    ) -> BoxCellFuture<'a, Result<CompleteCellControlReconciliationV1, PlacementContractError>>;
}
