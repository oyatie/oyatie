use crate::{BindingProofEnvelopeV1, BindingProofVerificationError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingProofSigningPreimageV1 {
    domain_prefix: &'static str,
    envelope: BindingProofEnvelopeV1,
    canonical_payload: Vec<u8>,
}

impl BindingProofSigningPreimageV1 {
    #[must_use]
    pub fn domain_prefix(&self) -> &'static str {
        self.domain_prefix
    }

    #[must_use]
    pub fn envelope(&self) -> &BindingProofEnvelopeV1 {
        &self.envelope
    }

    #[must_use]
    pub fn canonical_payload(&self) -> &[u8] {
        &self.canonical_payload
    }
}

pub trait BindingProofVerifier: Send + Sync {
    fn verify_signature(
        &self,
        preimage: &BindingProofSigningPreimageV1,
        signature: &[u8],
    ) -> Result<(), BindingProofVerificationError>;
}

pub trait HistoricalBindingProofVerifier: Send + Sync {
    fn verify_historical_signature(
        &self,
        preimage: &BindingProofSigningPreimageV1,
        signature: &[u8],
        context: &cell_placement::HistoricalProofVerificationContextV1,
    ) -> Result<(), BindingProofVerificationError>;
}
