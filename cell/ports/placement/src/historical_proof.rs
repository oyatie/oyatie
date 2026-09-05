use crate::{CellProofSigningPreimageV1, Digest32, ProofVerificationError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoricalSigningKeyEvidenceRefV1 {
    pub authority_id: String,
    pub repository_id: String,
    pub object_id: String,
    pub object_version: u64,
    pub producer_id: String,
    pub key_id: String,
    pub key_epoch: u64,
    pub content_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoricalRevocationEvidenceRefV1 {
    pub authority_id: String,
    pub repository_id: String,
    pub object_id: String,
    pub object_version: u64,
    pub observed_through_unix_seconds: u64,
    pub content_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoricalProofVerificationContextV1 {
    pub proof_valid_at_unix_seconds: u64,
    pub audited_at_unix_seconds: u64,
    pub signing_key_evidence: HistoricalSigningKeyEvidenceRefV1,
    pub revocation_evidence: HistoricalRevocationEvidenceRefV1,
    pub verification_policy_digest: Digest32,
}

pub trait HistoricalCellProofVerifier: Send + Sync {
    fn verify_historical_signature(
        &self,
        preimage: &CellProofSigningPreimageV1,
        signature: &[u8],
        context: &HistoricalProofVerificationContextV1,
    ) -> Result<(), ProofVerificationError>;
}
