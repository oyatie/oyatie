//! Audit-chain query API: tenant-scoped query and export DTOs.
//!
//! Tenant-scoped query and export DTOs. Validation, cursor codec and pagination
//! rules live in `audit/core/query-domain`; the wire schema, including the
//! `limit` bounds that crate enforces, is `audit/contracts/openapi/audit-chain.yaml`.
#![allow(dead_code)]

/// Audit query filter. Validated by `audit/core/query-domain`.
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

/// Query result page. Built by `audit/core/query-domain`, which also mints
/// and validates `next_cursor`.
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

/// Signed export bundle.
#[derive(Clone, Debug)]
pub struct ExportBundle {
    pub engagement_id: String,
    pub root_ref: String,
    pub public_key_ref: String,
    pub bundle_uri: String,
}

/// Auditor engagement. Expiry and tenant-match are checked by
/// `audit/core/query-domain`.
#[derive(Clone, Debug)]
pub struct AuditorEngagement {
    pub engagement_id: String,
    pub tenant_id: String,
    pub expires_at: String,
}
