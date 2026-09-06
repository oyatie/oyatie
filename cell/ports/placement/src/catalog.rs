use crate::{
    BoxCellFuture, CapacityVectorV1, CellAdmissionTermV1, CellCapacityLedgerV1, CellResourceV1,
    Digest32, ImmutableEvidenceRefV1, PlacementContractError, PlacementPartitionV1,
    PlacementReadAuthorityV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellCatalogCandidateV1 {
    pub resource: CellResourceV1,
    pub capacity: CellCapacityLedgerV1,
    pub admission_term: CellAdmissionTermV1,
    pub reservable_capacity: CapacityVectorV1,
    pub eligibility_evidence: ImmutableEvidenceRefV1,
    pub candidate_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellCatalogSnapshotV1 {
    pub snapshot_id: crate::CatalogSnapshotId,
    pub partition: PlacementPartitionV1,
    pub revision: u64,
    pub ordered_candidate_root_digest: Digest32,
    pub candidate_count: u64,
    pub observed_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub snapshot_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellCatalogPageTokenV1(Vec<u8>);

impl CellCatalogPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellCatalogPageRequestV1 {
    pub partition: PlacementPartitionV1,
    pub snapshot_id: crate::CatalogSnapshotId,
    pub page_size: u32,
    pub page_token: Option<CellCatalogPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellCatalogPageV1 {
    pub snapshot: CellCatalogSnapshotV1,
    pub candidates: Vec<CellCatalogCandidateV1>,
    pub next_page_token: Option<CellCatalogPageTokenV1>,
}

pub trait CellCatalogReader: Send + Sync {
    fn read_page<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        request: &'a CellCatalogPageRequestV1,
    ) -> BoxCellFuture<'a, Result<CellCatalogPageV1, PlacementContractError>>;
}
