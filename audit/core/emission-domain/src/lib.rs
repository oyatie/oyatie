//! Audit-chain emission domain: canonical envelope construction.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Validation rules,
//! deterministic fingerprinting, and period bucketing tracked under IP-004.
#![allow(dead_code)]

use audit_emission_kernel::ChainCoordinate;

/// Domain-level envelope produced after validation. Full schema in IP-004.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalEnvelope {
    pub coordinate: ChainCoordinate,
    pub event_id: String,
    pub fingerprint: String,
}

/// Domain-level error. Full variant set arrives with IP-004.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmissionDomainError {
    EmptyEventId,
    EmptyPack,
    FingerprintMismatch,
}
