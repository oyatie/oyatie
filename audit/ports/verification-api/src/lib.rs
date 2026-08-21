//! Audit-chain verification API: verdict DTOs with structured failure reasons.
//!
//! Verdict DTOs with structured failure reasons. The verifier that produces
//! these verdicts lives in `audit/core/verification-domain`.
#![allow(dead_code)]

/// Reason taxonomy for failed verification. Closed set; every variant is driven
/// by a test in `audit/core/verification-domain`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationFailureReason {
    KeyEpochMismatch,
    SignatureInvalid,
    ProofInvalid,
    PriorRootMissing,
    PackMismatch,
    RedactedEvent,
}

/// Verifier verdict. The verifier never mutates state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationVerdict {
    Verified,
    Failed(VerificationFailureReason),
}
