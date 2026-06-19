//! Audit-chain verification domain: pure proof + signature checks.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). The full verifier flow
//! (resolve key, verify signature, verify proof, walk prior root, reject
//! cross-pack mixtures) is tracked under IP-011.
#![allow(dead_code)]

pub use audit_verification_api::{VerificationFailureReason, VerificationVerdict};
