use crate::{
    BindingDigest32, BindingOperationKey, BindingProofConstructionError,
    ParticipantManifestMemberV1, ParticipantReceiptPhaseV1, SignedParticipantReceiptV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantWorkPageTokenV1(Vec<u8>);

impl ParticipantWorkPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, BindingProofConstructionError> {
        Err(BindingProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantManifestMemberSnapshotV1 {
    pub operation: BindingOperationKey,
    pub object_authority: String,
    pub repository_id: String,
    pub object_id: String,
    pub object_version: u64,
    pub content_digest: BindingDigest32,
    pub ordered_member_root_digest: BindingDigest32,
    pub member_count: u64,
    pub snapshot_digest: BindingDigest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParticipantManifestMemberSetV1 {
    snapshot: ParticipantManifestMemberSnapshotV1,
    sealed_progress: crate::BindingWorkSnapshotProgressV1,
}

impl ParticipantManifestMemberSetV1 {
    pub fn assemble(
        _snapshot: ParticipantManifestMemberSnapshotV1,
        _sealed_progress: crate::BindingWorkSnapshotProgressV1,
    ) -> Result<Self, crate::BindingStoreError> {
        Err(crate::BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn snapshot(&self) -> &ParticipantManifestMemberSnapshotV1 {
        &self.snapshot
    }

    #[must_use]
    pub fn sealed_progress(&self) -> &crate::BindingWorkSnapshotProgressV1 {
        &self.sealed_progress
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantManifestMemberPageRequestV1 {
    pub snapshot: ParticipantManifestMemberSnapshotV1,
    pub manifest_digest: BindingDigest32,
    pub start_ordinal: u64,
    pub page_size: u32,
    pub page_token: Option<ParticipantWorkPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantManifestMemberPageV1 {
    pub snapshot: ParticipantManifestMemberSnapshotV1,
    pub start_ordinal: u64,
    pub members: Vec<ParticipantManifestMemberV1>,
    pub next_ordinal: u64,
    pub next_page_token: Option<ParticipantWorkPageTokenV1>,
    pub page_digest: BindingDigest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParticipantReceiptWorkRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParticipantReceiptWorkDispositionV1 {
    Pending,
    ReceiptCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptWorkItemV1 {
    pub operation: BindingOperationKey,
    pub phase: ParticipantReceiptPhaseV1,
    pub placement_context_digest: BindingDigest32,
    pub participant_ordinal: u64,
    pub member_digest: BindingDigest32,
    pub disposition: ParticipantReceiptWorkDispositionV1,
    pub idempotency_digest: BindingDigest32,
    pub receipt: Option<SignedParticipantReceiptV1>,
    pub revision: ParticipantReceiptWorkRevision,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParticipantReceiptWorkPreconditionV1 {
    Absent,
    Matches {
        revision: ParticipantReceiptWorkRevision,
        placement_context_digest: BindingDigest32,
        record_digest: BindingDigest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptWorkPageRequestV1 {
    pub operation: BindingOperationKey,
    pub phase: ParticipantReceiptPhaseV1,
    pub placement_context_digest: BindingDigest32,
    pub ledger_revision: crate::ParticipantReceiptLedgerRevision,
    pub ledger_record_digest: BindingDigest32,
    pub start_ordinal: u64,
    pub page_size: u32,
    pub page_token: Option<ParticipantWorkPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantReceiptWorkPageV1 {
    pub operation: BindingOperationKey,
    pub phase: ParticipantReceiptPhaseV1,
    pub placement_context_digest: BindingDigest32,
    pub items: Vec<ParticipantReceiptWorkItemV1>,
    pub next_ordinal: u64,
    pub next_page_token: Option<ParticipantWorkPageTokenV1>,
    pub page_digest: BindingDigest32,
}
