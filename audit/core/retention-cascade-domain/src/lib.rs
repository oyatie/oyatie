//! Audit-chain retention-cascade domain: pure window + redaction-token rules.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full HIPAA/KR/SOC2 minima and
//! DSR replay rules in IP-013.
#![allow(dead_code)]

pub use audit_retention_cascade_api::{DsrCascade, RedactionToken, RetentionPolicy, RetentionRun};

/// Domain-level retention error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetentionDomainError {
    PolicyShortenedBelowMinimum,
    CrossPackCascade,
    InvalidRedactionReason,
}
