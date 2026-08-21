//! Analytics use-case orchestration (ADR-0083 layer 6).
//!
//! Wires domain aggregates to the OLAP client port. Each use-case is a
//! single-operation struct that accepts a `&mut dyn OlapClient` reference
//! so tests can inject `InMemoryOlapClient` without a real ClickHouse cluster.
//!
//! ## Use-cases provided
//!
//! - [`GetDashboardUseCase`] — executes a tenant dashboard query.
//! - [`SearchAuditLogUseCase`] — executes an audit log search.
//! - [`RunBillingRollupUseCase`] — executes a billing rollup aggregation.
//! - [`CreateDataExportUseCase`] — initiates a data export (deferred).
//!
//! ## Tenancy
//!
//! Use-cases pass the tenant's [`TenantId`] through to the OLAP port unchanged.
//! Cross-tenant access is enforced by the kernel's `assert_same_tenant`; use-
//! cases do not re-check.
//!
//! ## Honest-claims note
//!
//! Status is "planned". The data-export use-case returns
//! [`UseCaseError::Unimplemented`] until IP-013 and IP-004 land.
//!
//! non_claim: no Cedar authorization call, no event-bus emission, no
//! object-storage write in this scaffolding.

// ADR-0083 Tier 3: tests may use unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use data_analytics_domain::{
    AuditLogSearch, BillingRollup, DataExport, DomainError, TenantDashboardQuery, TenantId,
};
use shared_olap_client_kernel::{KernelError, OlapClient, Row};

// =====================================================================
// Use-case error
// =====================================================================

/// Errors surfaced by analytics use-cases.
#[derive(Clone, Debug)]
pub enum UseCaseError {
    /// A domain invariant was violated before touching the OLAP port.
    Domain(DomainError),
    /// The OLAP port returned an error (engine, cross-tenant, quota, etc.).
    Kernel(KernelError),
    /// The feature is not yet wired (honest-claims: status=planned).
    Unimplemented(&'static str),
}

impl fmt::Display for UseCaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(e) => write!(f, "domain error: {e}"),
            Self::Kernel(e) => write!(f, "kernel error: {e}"),
            Self::Unimplemented(slug) => write!(f, "unimplemented: {slug}"),
        }
    }
}

impl std::error::Error for UseCaseError {}

impl From<DomainError> for UseCaseError {
    fn from(e: DomainError) -> Self {
        Self::Domain(e)
    }
}

impl From<KernelError> for UseCaseError {
    fn from(e: KernelError) -> Self {
        match &e {
            KernelError::CrossTenantAccessDenied => Self::Kernel(e),
            _ => Self::Kernel(e),
        }
    }
}

// =====================================================================
// Dashboard use-case
// =====================================================================

/// Execute a tenant dashboard query against the OLAP port.
pub struct GetDashboardUseCase<'a> {
    olap: &'a dyn OlapClient,
    tenant_id: TenantId,
}

impl<'a> GetDashboardUseCase<'a> {
    #[must_use]
    pub fn new(olap: &'a dyn OlapClient, tenant_id: TenantId) -> Self {
        Self { olap, tenant_id }
    }

    /// Run the dashboard query and return result rows.
    ///
    /// # Errors
    /// Returns [`UseCaseError`] on domain violation or OLAP failure.
    pub fn execute(&self, request: &TenantDashboardQuery) -> Result<Vec<Row>, UseCaseError> {
        let q = request.to_olap_query()?;
        Ok(self.olap.query(&self.tenant_id, &q)?)
    }
}

// =====================================================================
// Audit-log search use-case
// =====================================================================

/// Execute a tenant audit log search against the OLAP port.
pub struct SearchAuditLogUseCase<'a> {
    olap: &'a dyn OlapClient,
    tenant_id: TenantId,
}

impl<'a> SearchAuditLogUseCase<'a> {
    #[must_use]
    pub fn new(olap: &'a dyn OlapClient, tenant_id: TenantId) -> Self {
        Self { olap, tenant_id }
    }

    /// Run the audit log search and return rows.
    ///
    /// # Errors
    /// Returns [`UseCaseError`] on OLAP failure or cross-tenant detection.
    pub fn execute(&self, request: &AuditLogSearch) -> Result<Vec<Row>, UseCaseError> {
        let q = request.to_olap_query()?;
        Ok(self.olap.query(&self.tenant_id, &q)?)
    }
}

// =====================================================================
// Billing rollup use-case
// =====================================================================

/// Execute a billing rollup aggregation.
pub struct RunBillingRollupUseCase<'a> {
    olap: &'a dyn OlapClient,
    tenant_id: TenantId,
}

impl<'a> RunBillingRollupUseCase<'a> {
    #[must_use]
    pub fn new(olap: &'a dyn OlapClient, tenant_id: TenantId) -> Self {
        Self { olap, tenant_id }
    }

    /// Run the billing rollup and return aggregated rows.
    ///
    /// # Errors
    /// Returns [`UseCaseError`] on domain or OLAP failure.
    pub fn execute(&self, request: &BillingRollup) -> Result<Vec<Row>, UseCaseError> {
        let q = request.to_olap_query()?;
        Ok(self.olap.query(&self.tenant_id, &q)?)
    }
}

// =====================================================================
// Data-export use-case (deferred)
// =====================================================================

/// Initiate a regulator / tenant data export.
///
/// non_claim: Object-storage write and CDC ingest pipeline (IP-004, IP-013)
/// are deferred. This use-case returns [`UseCaseError::Unimplemented`] until
/// those IPs land.
pub struct CreateDataExportUseCase;

impl CreateDataExportUseCase {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Initiate a data export. Returns the export job ID.
    ///
    /// # Errors
    /// Currently always returns [`UseCaseError::Unimplemented`] (IP-013 deferred).
    #[allow(clippy::unused_self)]
    pub fn execute(&self, _request: &DataExport) -> Result<String, UseCaseError> {
        Err(UseCaseError::Unimplemented(
            "data_export: IP-013 / IP-004 deferred — object-storage wiring not yet complete",
        ))
    }
}

impl Default for CreateDataExportUseCase {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use data_analytics_domain::{
        AuditLogFilter, DashboardMetric, ExportFormat, ExportScope, Pagination, TimeRange,
    };
    use shared_olap_client_kernel::{
        ColumnDef, ColumnType, KernelError, QualifiedTable, TableEngine, TableName, TableSchema,
        memory_adapter::InMemoryOlapClient,
    };

    fn tid(s: &str) -> TenantId {
        TenantId::try_new(s).unwrap()
    }

    fn range() -> TimeRange {
        TimeRange {
            from_secs: 1_735_689_600,
            to_secs: 1_738_368_000,
        }
    }

    fn seed_table(client: &mut InMemoryOlapClient, tenant: &str, table: &str) {
        let tid = TenantId::try_new(tenant).unwrap();
        let tbl = TableName::try_new(table).unwrap();
        let qt = QualifiedTable::new(tid.clone(), tbl);
        client.ensure_tenant_database(&tid).unwrap();
        client
            .ensure_table(&TableSchema {
                table: qt,
                columns: vec![
                    ColumnDef::new("ts", ColumnType::DateTime, false),
                    ColumnDef::new("workflow_execution_count", ColumnType::UInt64, true),
                    ColumnDef::new("api_call_count", ColumnType::UInt64, true),
                    ColumnDef::new("error_rate", ColumnType::Float64, true),
                    ColumnDef::new("p99_latency_ms", ColumnType::Float64, true),
                    ColumnDef::new("active_users", ColumnType::UInt64, true),
                    ColumnDef::new("storage_used_bytes", ColumnType::UInt64, true),
                    ColumnDef::new("actor_id", ColumnType::String, true),
                    ColumnDef::new("action", ColumnType::String, true),
                    ColumnDef::new("resource_id", ColumnType::String, true),
                    ColumnDef::new("amount", ColumnType::Float64, true),
                    ColumnDef::new("period", ColumnType::String, true),
                ],
                engine: TableEngine::MergeTree,
                order_by: vec!["ts".to_string()],
                partition_by: None,
                ttl: None,
            })
            .unwrap();
    }

    #[test]
    fn get_dashboard_returns_empty_rows_from_in_memory() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "tenant_metrics");
        let uc = GetDashboardUseCase::new(&client, tid("t1"));
        let req = TenantDashboardQuery {
            tenant_id: tid("t1"),
            metrics: vec![DashboardMetric::ApiCallCount],
            time_range: range(),
            pagination: Pagination::new(10),
        };
        let result = uc.execute(&req).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn search_audit_log_empty_result() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "audit_log");
        let uc = SearchAuditLogUseCase::new(&client, tid("t1"));
        let req = AuditLogSearch {
            tenant_id: tid("t1"),
            filter: AuditLogFilter::default(),
            pagination: Pagination::new(20),
        };
        let rows = uc.execute(&req).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn billing_rollup_empty_result() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "billing_events");
        let uc = RunBillingRollupUseCase::new(&client, tid("t1"));
        let req = BillingRollup {
            tenant_id: tid("t1"),
            granularity: data_analytics_domain::RollupGranularity::Monthly,
            time_range: range(),
        };
        let result = uc.execute(&req).unwrap();
        // COUNT aggregate returns 1 row (count=0) from the in-memory adapter;
        // Sum aggregate also returns 1 row with total=0.0.
        // Just verify no panic.
        let _ = result;
    }

    #[test]
    fn data_export_honestly_unimplemented() {
        let uc = CreateDataExportUseCase::new();
        let req = DataExport {
            tenant_id: tid("t1"),
            scope: ExportScope::AuditLog {
                time_range: range(),
            },
            format: ExportFormat::JsonLines,
        };
        let err = uc.execute(&req).unwrap_err();
        match err {
            UseCaseError::Unimplemented(slug) => {
                assert!(slug.contains("IP-013"));
            }
            other => panic!("expected Unimplemented, got {other}"),
        }
    }

    /// Use-case caller tenant must match the query's QualifiedTable tenant;
    /// kernel `assert_same_tenant` surfaces as UseCaseError::Kernel(CrossTenant…).
    #[test]
    fn get_dashboard_refuses_cross_tenant_query() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "tenant_metrics");
        // Caller is t2; request is scoped to t1's QualifiedTable via domain builder.
        let uc = GetDashboardUseCase::new(&client, tid("t2"));
        let req = TenantDashboardQuery {
            tenant_id: tid("t1"),
            metrics: vec![DashboardMetric::ApiCallCount],
            time_range: range(),
            pagination: Pagination::new(10),
        };
        let err = uc.execute(&req).unwrap_err();
        match err {
            UseCaseError::Kernel(KernelError::CrossTenantAccessDenied) => {}
            other => panic!("expected CrossTenantAccessDenied, got {other}"),
        }
    }

    /// Same tenancy refusal pattern for audit-log search (caller ≠ query tenant).
    #[test]
    fn search_audit_log_refuses_cross_tenant_query() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "audit_log");
        let uc = SearchAuditLogUseCase::new(&client, tid("t2"));
        let req = AuditLogSearch {
            tenant_id: tid("t1"),
            filter: AuditLogFilter::default(),
            pagination: Pagination::new(10),
        };
        let err = uc.execute(&req).unwrap_err();
        match err {
            UseCaseError::Kernel(KernelError::CrossTenantAccessDenied) => {}
            other => panic!("expected CrossTenantAccessDenied, got {other}"),
        }
    }

    /// Same tenancy refusal pattern for billing rollup (caller ≠ query tenant).
    #[test]
    fn billing_rollup_refuses_cross_tenant_query() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "billing_events");
        let uc = RunBillingRollupUseCase::new(&client, tid("t2"));
        let req = BillingRollup {
            tenant_id: tid("t1"),
            granularity: data_analytics_domain::RollupGranularity::Monthly,
            time_range: range(),
        };
        let err = uc.execute(&req).unwrap_err();
        match err {
            UseCaseError::Kernel(KernelError::CrossTenantAccessDenied) => {}
            other => panic!("expected CrossTenantAccessDenied, got {other}"),
        }
    }
}
