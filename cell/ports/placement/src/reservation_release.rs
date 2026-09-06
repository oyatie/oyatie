use crate::{
    Digest32, HistoricalProofVerificationContextV1, ProducerId, ProofConstructionError,
    ProofVerificationError, ReservationRefV1, TenantId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReservationReleaseIntentV1 {
    pub schema_version: u32,
    pub tenant_id: TenantId,
    pub binding_operation_id: String,
    pub tenancy_shard_key: String,
    pub source_binding_generation: u64,
    pub source_binding_revision: u64,
    pub source_binding_record_digest: Digest32,
    pub successor_binding_generation: u64,
    pub successor_binding_revision: u64,
    pub successor_binding_record_digest: Digest32,
    pub migration_commit_seal_digest: Digest32,
    pub source_home_reservation: ReservationRefV1,
    pub source_warm_recovery_reservation: Option<ReservationRefV1>,
    pub source_reservation_set_digest: Digest32,
    pub target_activation_closure_digest: Digest32,
    pub source_release_closure_digest: Digest32,
    pub routing_convergence_digest: Digest32,
    pub rollback_policy_generation: u64,
    pub rollback_policy_digest: Digest32,
    pub rollback_window_duration_seconds: u64,
    pub successor_binding_committed_at_unix_seconds: u64,
    pub release_not_before_unix_seconds: u64,
    pub trusted_time_evidence_digest: Digest32,
    pub clock_authority_digest: Digest32,
    pub observed_clock_uncertainty_millis: u64,
    pub intent_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReservationReleasePermitPayloadV1 {
    pub schema_version: u32,
    pub intent: SourceReservationReleaseIntentV1,
    pub release_issuance_revision: u64,
    pub release_issuance_record_digest: Digest32,
    pub tenancy_commit_attestation_digest: Digest32,
    pub permitted_at_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenancyReleaseProducerId(String);

impl TenancyReleaseProducerId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, ProofConstructionError> {
        Err(ProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenancyReleaseProofDomainV1 {
    SourceReservationReleasePermit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenancyReleaseProofEnvelopeV1 {
    pub schema_version: u32,
    pub signing_format_version: u32,
    pub domain: TenancyReleaseProofDomainV1,
    pub producer: TenancyReleaseProducerId,
    pub audience: ProducerId,
    pub key_id: String,
    pub nonce: String,
    pub issued_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub payload_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedSourceReservationReleasePermitV1 {
    pub payload: SourceReservationReleasePermitPayloadV1,
    pub envelope: TenancyReleaseProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenancyReleaseProofSigningPreimageV1 {
    pub domain_prefix: &'static str,
    pub envelope: TenancyReleaseProofEnvelopeV1,
    pub canonical_payload: Vec<u8>,
}

pub trait TenancyReleaseProofVerifier: Send + Sync {
    fn verify_signature(
        &self,
        preimage: &TenancyReleaseProofSigningPreimageV1,
        signature: &[u8],
    ) -> Result<(), ProofVerificationError>;
}

pub trait HistoricalTenancyReleaseProofVerifier: Send + Sync {
    fn verify_historical_signature(
        &self,
        preimage: &TenancyReleaseProofSigningPreimageV1,
        signature: &[u8],
        context: &HistoricalProofVerificationContextV1,
    ) -> Result<(), ProofVerificationError>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct TenancyReleaseProofConsumptionV1 {
    pub reservation: ReservationRefV1,
    pub producer: TenancyReleaseProducerId,
    pub nonce: String,
    pub payload_digest: Digest32,
    pub envelope_digest: Digest32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedSourceReservationReleasePermit(SignedSourceReservationReleasePermitV1);

impl VerifiedSourceReservationReleasePermit {
    #[must_use]
    pub fn signed(&self) -> &SignedSourceReservationReleasePermitV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceReservationReleasePermitExpectationV1 {
    pub tenant_id: TenantId,
    pub reservation: ReservationRefV1,
    pub expected_issuance_revision: u64,
    pub expected_issuance_record_digest: Digest32,
    pub expected_commit_attestation_digest: Digest32,
    pub expected_producer: TenancyReleaseProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

pub fn verify_source_reservation_release_permit(
    _verifier: &dyn TenancyReleaseProofVerifier,
    _signed: SignedSourceReservationReleasePermitV1,
    _expectation: &SourceReservationReleasePermitExpectationV1,
) -> Result<VerifiedSourceReservationReleasePermit, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn verify_historical_source_reservation_release_permit(
    _verifier: &dyn HistoricalTenancyReleaseProofVerifier,
    _signed: SignedSourceReservationReleasePermitV1,
    _expectation: &SourceReservationReleasePermitExpectationV1,
    _context: &HistoricalProofVerificationContextV1,
) -> Result<VerifiedSourceReservationReleasePermit, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn bind_tenancy_release_proof_consumption(
    _permit: &VerifiedSourceReservationReleasePermit,
    _reservation: ReservationRefV1,
) -> Result<TenancyReleaseProofConsumptionV1, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}
