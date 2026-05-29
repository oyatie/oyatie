//! Audit-chain query API: tenant-scoped query and export DTOs.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full DTO schema in IP-012.
#![allow(dead_code)]

/// Audit query filter. Full filter schema in IP-012.
#[derive(Clone, Debug, Default)]
pub struct AuditQuery {
    pub tenant_id: String,
    pub pack: Option<String>,
    pub event_type: Option<String>,
    pub principal: Option<String>,
    pub entity: Option<String>,
    pub period: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Sealed/unsealed lifecycle marker returned with each query result row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultSealState {
    Unsealed,
    Sealed,
    Published,
    Redacted,
}

/// Query result page. Full schema in IP-012.
#[derive(Clone, Debug, Default)]
pub struct QueryResult {
    pub rows: Vec<QueryRow>,
    pub next_cursor: Option<String>,
}

/// Single audit query row.
#[derive(Clone, Debug)]
pub struct QueryRow {
    pub audit_id: String,
    pub period_id: String,
    pub seal_state: ResultSealState,
}

/// Signed export bundle. Full schema in IP-012.
#[derive(Clone, Debug)]
pub struct ExportBundle {
    pub engagement_id: String,
    pub root_ref: String,
    pub public_key_ref: String,
    pub bundle_uri: String,
}

/// Auditor engagement. Full schema in IP-012.
#[derive(Clone, Debug)]
pub struct AuditorEngagement {
    pub engagement_id: String,
    pub tenant_id: String,
    pub expires_at: String,
}
