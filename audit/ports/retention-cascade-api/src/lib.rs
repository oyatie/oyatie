//! Audit-chain retention-cascade API: policy and DSR-cascade DTOs.
//!
//! Retention-policy and DSR-cascade DTOs. The retention floors, redaction rules
//! and cascade constraints applied to these types live in
//! `audit/core/retention-cascade-domain`, whose authority is
//! `audit/policy/retention-matrix.yaml`.
#![allow(dead_code)]

/// Retention policy for a (pack, data_class) pair. Floors are enforced by
/// `audit/core/retention-cascade-domain` against `audit/policy/retention-matrix.yaml`.
#[derive(Clone, Debug)]
pub struct RetentionPolicy {
    pub pack: String,
    pub data_class: String,
    pub retention_seconds: u64,
}

/// DSR cascade request entering audit-chain. Confinement to a single pack is
/// enforced by `audit/core/retention-cascade-domain`.
#[derive(Clone, Debug)]
pub struct DsrCascade {
    pub tenant_id: String,
    pub subject_id: String,
    pub source_microservice: String,
}

/// Redaction token written in place of erased payload material.
#[derive(Clone, Debug)]
pub struct RedactionToken {
    pub audit_id: String,
    pub reason: String,
    pub lawful_basis: String,
}

/// One run of the retention worker.
#[derive(Clone, Debug)]
pub struct RetentionRun {
    pub run_id: String,
    pub pack: String,
    pub started_at: String,
}
