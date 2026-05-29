//! Audit-chain retention-cascade API: policy and DSR-cascade DTOs.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full schema in IP-013.
#![allow(dead_code)]

/// Retention policy for a (pack, data_class) pair. Full schema in IP-013.
#[derive(Clone, Debug)]
pub struct RetentionPolicy {
    pub pack: String,
    pub data_class: String,
    pub retention_seconds: u64,
}

/// DSR cascade request entering audit-chain. Full schema in IP-013.
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
