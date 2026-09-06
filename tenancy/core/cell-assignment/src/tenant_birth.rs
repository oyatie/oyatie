use cell_placement::ImmutableEvidenceRefV1;

use crate::{
    BindingDigest32, BindingProducerId, BindingProofConstructionError, BindingProofEnvelopeV1,
    BindingProofVerificationError, BindingProofVerifier, BindingReadAuthorityV1, BindingStoreError,
    BoxTenancyFuture, TenantId,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StagedTenantRecordName(String);

impl StagedTenantRecordName {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, BindingProofConstructionError> {
        Err(BindingProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantBirthRefV1 {
    pub schema_version: u32,
    pub tenant_id: TenantId,
    pub staged_tenant_record_name: StagedTenantRecordName,
    pub tenant_record_digest: BindingDigest32,
    pub assurance_requirements_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedTenantBirthRecordPayloadV1 {
    pub schema_version: u32,
    pub reference: TenantBirthRefV1,
    pub staging_revision: u64,
    pub immutable_record: ImmutableEvidenceRefV1,
    pub staged_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub record_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedStagedTenantBirthRecordV1 {
    pub payload: StagedTenantBirthRecordPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedTenantBirthRecordExpectationV1 {
    pub reference: TenantBirthRefV1,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedStagedTenantBirthRecord(SignedStagedTenantBirthRecordV1);

impl VerifiedStagedTenantBirthRecord {
    #[must_use]
    pub fn signed(&self) -> &SignedStagedTenantBirthRecordV1 {
        &self.0
    }
}

pub fn verify_staged_tenant_birth_record(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedStagedTenantBirthRecordV1,
    _expectation: &StagedTenantBirthRecordExpectationV1,
) -> Result<VerifiedStagedTenantBirthRecord, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}

pub trait TenantBirthRecordReader: Send + Sync {
    fn load_staged<'a>(
        &'a self,
        authority: &'a BindingReadAuthorityV1,
        reference: &'a TenantBirthRefV1,
    ) -> BoxTenancyFuture<'a, Result<Option<SignedStagedTenantBirthRecordV1>, BindingStoreError>>;
}
