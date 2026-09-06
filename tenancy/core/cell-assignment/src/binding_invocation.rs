use crate::{
    BindingActionV1, BindingDigest32, BindingOperationKey, BindingPolicyVersionToken,
    BindingProducerId, BindingProofEnvelopeV1, BindingProofVerificationError, BindingProofVerifier,
    TenantId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingAuthorizationDecisionReceiptV1 {
    pub decision_id: String,
    pub policy_version: BindingPolicyVersionToken,
    pub decision_digest: BindingDigest32,
    pub determining_policy_set_digest: BindingDigest32,
    pub obligations_digest: BindingDigest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingInvocationPayloadV1 {
    pub schema_version: u32,
    pub action: BindingActionV1,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub canonical_request_digest: BindingDigest32,
    pub actor_digest: BindingDigest32,
    pub authorization: BindingAuthorizationDecisionReceiptV1,
    pub deadline_unix_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedBindingInvocationV1 {
    pub payload: BindingInvocationPayloadV1,
    pub envelope: BindingProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingInvocationExpectation {
    pub action: BindingActionV1,
    pub operation: BindingOperationKey,
    pub tenant_id: TenantId,
    pub canonical_request_digest: BindingDigest32,
    pub expected_producer: BindingProducerId,
    pub expected_audience: BindingProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedBindingInvocation(SignedBindingInvocationV1);

#[derive(Debug, Eq, PartialEq)]
pub struct BindingPersistenceAuthorityV1(SignedBindingInvocationV1);

#[derive(Debug, Eq, PartialEq)]
pub struct BindingReadAuthorityV1(SignedBindingInvocationV1);

impl VerifiedBindingInvocation {
    #[must_use]
    pub fn signed(&self) -> &SignedBindingInvocationV1 {
        &self.0
    }

    pub fn into_persistence_authority(
        self,
    ) -> Result<BindingPersistenceAuthorityV1, BindingProofVerificationError> {
        Err(BindingProofVerificationError::NotImplemented)
    }

    pub fn into_read_authority(
        self,
    ) -> Result<BindingReadAuthorityV1, BindingProofVerificationError> {
        Err(BindingProofVerificationError::NotImplemented)
    }
}

impl BindingPersistenceAuthorityV1 {
    #[must_use]
    pub fn invocation(&self) -> &SignedBindingInvocationV1 {
        &self.0
    }
}

impl BindingReadAuthorityV1 {
    #[must_use]
    pub fn invocation(&self) -> &SignedBindingInvocationV1 {
        &self.0
    }
}

pub fn verify_binding_invocation(
    _verifier: &dyn BindingProofVerifier,
    _signed: SignedBindingInvocationV1,
    _expectation: &BindingInvocationExpectation,
) -> Result<VerifiedBindingInvocation, BindingProofVerificationError> {
    Err(BindingProofVerificationError::NotImplemented)
}
