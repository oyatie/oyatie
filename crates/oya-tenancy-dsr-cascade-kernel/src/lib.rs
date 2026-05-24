//! DSR cascade kernel — DsrRequest, ErasureReceipt, ProofOfErasure + ports.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-009 execution.
//! GDPR Article 17 + LGPD + DPDPA + CCPA erasure cascade across µservices.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DsrRequestId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DsrKind {
    Access,
    Erasure,
    Rectification,
    Portability,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrRequest {
    pub id: DsrRequestId,   // data_class: AUDIT
    pub tenant_id: String,  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub subject_id: String, // data_class: SENSITIVE_PIPA_ART23
    pub kind: DsrKind,      // data_class: SENSITIVE_PIPA_ART23
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErasureReceipt {
    pub request: DsrRequestId, // data_class: AUDIT
    pub microservice: String,  // data_class: AUDIT
    pub merkle_leaf: [u8; 32], // data_class: AUDIT
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofOfErasure {
    pub request: DsrRequestId,         // data_class: AUDIT
    pub merkle_root: [u8; 32],         // data_class: AUDIT
    pub receipts: Vec<ErasureReceipt>, // data_class: AUDIT
}

pub trait DsrRequestRepository {
    fn open(&self, request: &DsrRequest) -> Result<(), DsrKernelError>;
    fn append_receipt(&self, receipt: &ErasureReceipt) -> Result<(), DsrKernelError>;
    fn finalize(&self, proof: &ProofOfErasure) -> Result<(), DsrKernelError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DsrKernelError {
    UnknownRequest,
    DuplicateReceipt,
    MerkleAggregationFailed,
    SlaBreached,
}
