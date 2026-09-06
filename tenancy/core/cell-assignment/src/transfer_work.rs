use crate::{
    BindingDigest32, BindingOperationKey, BindingProofConstructionError, CapabilityParticipantId,
    ResidencyTransferEffectV1, SignedTransferExecutionOutcomeV1, SignedTransferExecutionPermitV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferWorkPageTokenV1(Vec<u8>);

impl TransferWorkPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, BindingProofConstructionError> {
        Err(BindingProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferEffectSnapshotV1 {
    pub operation: BindingOperationKey,
    pub object_authority: String,
    pub repository_id: String,
    pub object_id: String,
    pub object_version: u64,
    pub content_digest: BindingDigest32,
    pub ordered_effect_root_digest: BindingDigest32,
    pub effect_count: u64,
    pub snapshot_digest: BindingDigest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TransferEffectSetV1 {
    snapshot: TransferEffectSnapshotV1,
    sealed_progress: crate::BindingWorkSnapshotProgressV1,
}

impl TransferEffectSetV1 {
    pub fn assemble(
        _snapshot: TransferEffectSnapshotV1,
        _sealed_progress: crate::BindingWorkSnapshotProgressV1,
    ) -> Result<Self, crate::BindingStoreError> {
        Err(crate::BindingStoreError::NotImplemented)
    }

    #[must_use]
    pub fn snapshot(&self) -> &TransferEffectSnapshotV1 {
        &self.snapshot
    }

    #[must_use]
    pub fn sealed_progress(&self) -> &crate::BindingWorkSnapshotProgressV1 {
        &self.sealed_progress
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferEffectPageRequestV1 {
    pub snapshot: TransferEffectSnapshotV1,
    pub manifest_digest: BindingDigest32,
    pub start_ordinal: u64,
    pub page_size: u32,
    pub page_token: Option<TransferWorkPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferEffectPageV1 {
    pub snapshot: TransferEffectSnapshotV1,
    pub start_ordinal: u64,
    pub effects: Vec<ResidencyTransferEffectV1>,
    pub next_ordinal: u64,
    pub next_page_token: Option<TransferWorkPageTokenV1>,
    pub page_digest: BindingDigest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransferExecutionItemRevision(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TransferExecutionItemDispositionV1 {
    PendingPermit,
    PermitIssued,
    OutcomeRecorded,
    SettlementPending,
    Settled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferExecutionItemV1 {
    pub operation: BindingOperationKey,
    pub effect_ordinal: u64,
    pub effect_fingerprint: BindingDigest32,
    pub participant_id: CapabilityParticipantId,
    pub disposition: TransferExecutionItemDispositionV1,
    pub permit: Option<SignedTransferExecutionPermitV1>,
    pub outcome: Option<SignedTransferExecutionOutcomeV1>,
    pub idempotency_digest: BindingDigest32,
    pub worker_id: String,
    pub worker_lease_epoch: u64,
    pub worker_lease_expires_at_unix_seconds: u64,
    pub settlement_claim_digest: Option<BindingDigest32>,
    pub settlement_digest: Option<BindingDigest32>,
    pub revision: TransferExecutionItemRevision,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferExecutionItemPreconditionV1 {
    Absent,
    Matches {
        revision: TransferExecutionItemRevision,
        disposition: TransferExecutionItemDispositionV1,
        record_digest: BindingDigest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferExecutionItemPageRequestV1 {
    pub operation: BindingOperationKey,
    pub ledger_revision: crate::TransferExecutionLedgerRevision,
    pub ledger_record_digest: BindingDigest32,
    pub start_ordinal: u64,
    pub page_size: u32,
    pub page_token: Option<TransferWorkPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferExecutionItemPageV1 {
    pub operation: BindingOperationKey,
    pub items: Vec<TransferExecutionItemV1>,
    pub next_ordinal: u64,
    pub next_page_token: Option<TransferWorkPageTokenV1>,
    pub page_digest: BindingDigest32,
}
