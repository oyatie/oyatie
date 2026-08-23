//! ClickHouse 26.3 LTS adapter for the engine-agnostic OLAP port (ADR-0193).
//!
//! This crate is the **dependency-seam boundary** for the `clickhouse` client
//! crate (ADR-0092 `layer_seam = adapter-only`). No other analytics crate
//! depends on `clickhouse` directly.
//!
//! ## Tenancy isolation
//!
//! Every [`ClickHouseOlapClient`] enforces that queries use the caller's
//! `tenant_{tenant_id}` ClickHouse database. Cross-tenant queries are
//! rejected by the kernel's `assert_same_tenant` before any network call.
//!
//! ## Honest-claims note
//!
//! Status is "planned". The adapter struct and trait impl are scaffolded;
//! production wiring to a live ClickHouse cluster is deferred (IP-003).
//! All CI tests use `shared_olap_client_kernel::memory_adapter::InMemoryOlapClient`.
//!
//! non_claim: no live ClickHouse connection, no SLO enforcement, no
//! production deployment in this scaffolding.

// ADR-0083 Tier 3: production code stays panic-free; tests use unwrap/expect.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_olap_client_kernel::{
    InsertBatch, KernelError, MaterializedViewSchema, OlapClient, Query, QuotaProfile, Row,
    TableSchema, TenantId,
};

// =====================================================================
// ClickHouse adapter configuration
// =====================================================================

/// Connection configuration for the ClickHouse 26.3 LTS cluster.
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug)]
pub struct ClickHouseConfig {
    /// ClickHouse HTTP interface URL (e.g. `http://clickhouse.analytics.svc:8123`).
    pub url: String,
    /// ClickHouse user with per-tenant database access.
    pub user: String,
    /// Password — sourced from OpenBao at runtime; not stored in config files.
    /// data_class: INTERNAL_ONLY (secret at rest)
    pub password: String,
}

// =====================================================================
// Adapter
// =====================================================================

/// [`OlapClient`] implementation backed by ClickHouse 26.3 LTS.
///
/// ## Dependency seam
///
/// The `clickhouse` crate is isolated to this adapter crate per ADR-0092.
/// All outer crates depend on the kernel port trait only.
///
/// ## Status: planned
///
/// non_claim: All methods return [`KernelError::AdapterError`] with a clear
/// "IP-003 deferred" message until the ClickHouse HTTP client wiring is
/// complete. The struct scaffolding exists so the composition root can
/// reference the type at compile time.
pub struct ClickHouseOlapClient {
    config: ClickHouseConfig,
}

impl ClickHouseOlapClient {
    /// Construct an adapter from config.
    ///
    /// The adapter routes queries to per-tenant `tenant_{id}` databases.
    #[must_use]
    pub fn new(config: ClickHouseConfig) -> Self {
        Self { config }
    }

    /// Return the ClickHouse URL (for logging/diagnostics only).
    #[must_use]
    pub fn url(&self) -> &str {
        &self.config.url
    }
}

impl OlapClient for ClickHouseOlapClient {
    fn ensure_tenant_database(&mut self, _tenant_id: &TenantId) -> Result<(), KernelError> {
        // non_claim: live ClickHouse DDL not wired yet (IP-003).
        let _ = &self.config.url; // suppress unused-field lint; remove when wired
        Err(KernelError::AdapterError(
            "clickhouse ensure_tenant_database: IP-003 deferred".to_string(),
        ))
    }

    fn ensure_table(&mut self, _schema: &TableSchema) -> Result<(), KernelError> {
        Err(KernelError::AdapterError(
            "clickhouse ensure_table: IP-003 deferred".to_string(),
        ))
    }

    fn ensure_materialized_view(
        &mut self,
        _schema: &MaterializedViewSchema,
    ) -> Result<(), KernelError> {
        Err(KernelError::AdapterError(
            "clickhouse ensure_materialized_view: IP-003 deferred".to_string(),
        ))
    }

    fn apply_quota(&mut self, _profile: &QuotaProfile) -> Result<(), KernelError> {
        Err(KernelError::AdapterError(
            "clickhouse apply_quota: IP-003 deferred".to_string(),
        ))
    }

    fn insert(&mut self, _batch: &InsertBatch) -> Result<u64, KernelError> {
        Err(KernelError::AdapterError(
            "clickhouse insert: IP-003 deferred".to_string(),
        ))
    }

    fn query(&self, _caller: &TenantId, _query: &Query) -> Result<Vec<Row>, KernelError> {
        Err(KernelError::AdapterError(
            "clickhouse query: IP-003 deferred".to_string(),
        ))
    }

    fn drop_tenant_database(&mut self, _tenant_id: &TenantId) -> Result<(), KernelError> {
        Err(KernelError::AdapterError(
            "clickhouse drop_tenant_database: IP-003 deferred".to_string(),
        ))
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn adapter() -> ClickHouseOlapClient {
        ClickHouseOlapClient::new(ClickHouseConfig {
            url: "http://clickhouse.test:8123".to_string(),
            user: "default".to_string(),
            password: "test".to_string(),
        })
    }

    #[test]
    fn unimplemented_surfaced_honestly_on_query() {
        let tid = TenantId::try_new("t1").unwrap();
        let table = shared_olap_client_kernel::QualifiedTable::new(
            tid.clone(),
            shared_olap_client_kernel::TableName::try_new("events").unwrap(),
        );
        let q = Query {
            source: table,
            columns: vec!["id".to_string()],
            aggregates: vec![],
            filter: None,
            group_by: vec![],
            order_by: vec![],
            limit: None,
        };
        let a = adapter();
        let err = a.query(&tid, &q).unwrap_err();
        match err {
            KernelError::AdapterError(msg) => assert!(msg.contains("IP-003")),
            other => panic!("expected AdapterError, got {other}"),
        }
    }

    #[test]
    fn unimplemented_surfaced_on_ensure_database() {
        let mut a = adapter();
        let tid = TenantId::try_new("t1").unwrap();
        let err = a.ensure_tenant_database(&tid).unwrap_err();
        assert!(matches!(err, KernelError::AdapterError(_)));
    }

    #[test]
    fn url_accessor() {
        let a = adapter();
        assert_eq!(a.url(), "http://clickhouse.test:8123");
    }
}
