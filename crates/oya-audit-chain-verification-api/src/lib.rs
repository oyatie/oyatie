//! Audit-chain verification API: verdict DTOs with structured failure reasons.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full schema in IP-011.
#![allow(dead_code)]

/// Reason taxonomy for failed verification. Closed set extended in IP-011.
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
