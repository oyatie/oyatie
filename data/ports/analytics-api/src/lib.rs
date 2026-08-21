//! Analytics API surface — outward-facing layer 11 per ADR-0083.
//!
//! Exposes typed request/response structs for:
//! - [`DashboardQueryRequest`] / [`DashboardQueryResponse`] (IP-007)
//! - [`AuditLogSearchRequest`] / [`AuditLogSearchResponse`] (IP-008)
//! - [`BillingRollupRequest`] / [`BillingRollupResponse`] (IP-009)
//! - [`DataExportRequest`] / [`DataExportResponse`] (IP-013)
//!
//! The actual HTTP handlers (axum routes) are deferred per manifest status
//! (IP-015). This crate provides the typed envelope structs and a thin
//! orchestration layer that wires use-cases without being tied to a transport.
//!
//! ## Tenancy
//!
//! Every request carries a `tenant_id` field validated as non-empty before
//! calling use-cases. Cross-tenant enforcement is double-locked at the OLAP
//! kernel (`assert_same_tenant`).
//!
//! ## Contracts
//!
//! - [`ANALYTICS_OPENAPI_CONTRACT`]
//! - [`ANALYTICS_ASYNCAPI_CONTRACT`]
//! - [`ANALYTICS_PROTO_CONTRACT`]
//!
//! ## Honest-claims note
//!
//! non_claim: no live HTTP server, no gRPC server.

// ADR-0083 Tier 3: tests may use unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use serde::{Deserialize, Serialize};

use data_analytics_domain::{
    AuditLogFilter, AuditLogSearch, BillingRollup, DashboardMetric, DataExport, ExportFormat,
    ExportScope, Pagination, RollupGranularity, TenantDashboardQuery, TimeRange,
};
use data_analytics_usecase::{
    CreateDataExportUseCase, GetDashboardUseCase, RunBillingRollupUseCase, SearchAuditLogUseCase,
    UseCaseError,
};
use shared_olap_client_kernel::{KernelError, OlapClient, TenantId};

/// OpenAPI contract path (SSOT: `data/analytics/catalog/contracts.json`).
pub const ANALYTICS_OPENAPI_CONTRACT: &str = "data/analytics/contracts/openapi-v1.yaml";
/// AsyncAPI contract path (SSOT: `data/analytics/catalog/contracts.json`).
pub const ANALYTICS_ASYNCAPI_CONTRACT: &str = "data/analytics/contracts/asyncapi-v1.yaml";
/// gRPC proto contract path (SSOT: `data/analytics/catalog/contracts.json`).
pub const ANALYTICS_PROTO_CONTRACT: &str = "data/analytics/contracts/analytics.proto";

// =====================================================================
// API error
// =====================================================================

/// Errors surfaced at the API boundary.
#[derive(Clone, Debug)]
pub enum ApiError {
    /// The request failed domain / input validation.
    BadRequest(String),
    /// Cross-tenant access was denied.
    Forbidden(String),
    /// A use-case error propagated from the inner layer.
    UseCase(UseCaseError),
    /// Feature not yet implemented (honest-claims).
    Unimplemented(&'static str),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "bad request: {msg}"),
            Self::Forbidden(msg) => write!(f, "forbidden: {msg}"),
            Self::UseCase(e) => write!(f, "use-case error: {e}"),
            Self::Unimplemented(slug) => write!(f, "unimplemented: {slug}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<UseCaseError> for ApiError {
    fn from(e: UseCaseError) -> Self {
        match &e {
            UseCaseError::Kernel(KernelError::CrossTenantAccessDenied) => {
                Self::Forbidden(e.to_string())
            }
            UseCaseError::Unimplemented(slug) => Self::Unimplemented(slug),
            _ => Self::UseCase(e),
        }
    }
}

/// Parse and validate a `tenant_id` string into a [`TenantId`].
fn parse_tenant_id(s: &str) -> Result<TenantId, ApiError> {
    TenantId::try_new(s).map_err(|e| ApiError::BadRequest(format!("invalid tenant_id: {e}")))
}

// =====================================================================
// Dashboard query API
// =====================================================================

/// REST request body for `GET /analytics/v1/dashboard`.
///
/// data_class: TENANT_PUBLIC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardQueryRequest {
    pub tenant_id: String,
    pub metrics: Vec<DashboardMetric>,
    pub from_secs: u64,
    pub to_secs: u64,
    pub limit: Option<u64>,
}

/// REST response for a dashboard query.
///
/// data_class: TENANT_PUBLIC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardQueryResponse {
    pub row_count: usize,
}

/// Execute a dashboard query request.
///
/// # Errors
/// Returns [`ApiError::BadRequest`] on invalid tenant/metrics, or propagates
/// use-case errors.
pub fn handle_dashboard_query(
    req: &DashboardQueryRequest,
    olap: &dyn OlapClient,
) -> Result<DashboardQueryResponse, ApiError> {
    let tenant_id = parse_tenant_id(&req.tenant_id)?;
    if req.metrics.is_empty() {
        return Err(ApiError::BadRequest(
            "at least one metric must be requested".to_string(),
        ));
    }
    let domain_req = TenantDashboardQuery {
        tenant_id: tenant_id.clone(),
        metrics: req.metrics.clone(),
        time_range: TimeRange {
            from_secs: req.from_secs,
            to_secs: req.to_secs,
        },
        pagination: Pagination::new(req.limit.unwrap_or(Pagination::DEFAULT_LIMIT)),
    };
    let uc = GetDashboardUseCase::new(olap, tenant_id);
    let rows = uc.execute(&domain_req)?;
    Ok(DashboardQueryResponse {
        row_count: rows.len(),
    })
}

// =====================================================================
// Audit log search API
// =====================================================================

/// REST request body for `GET /analytics/v1/audit-log`.
///
/// data_class: AUDIT_INTERNAL, TENANT_AUDIT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogSearchRequest {
    pub tenant_id: String,
    pub actor_id: Option<String>,
    pub resource_type: Option<String>,
    pub action: Option<String>,
    pub from_secs: Option<u64>,
    pub to_secs: Option<u64>,
    pub limit: Option<u64>,
}

/// REST response for an audit log search.
///
/// data_class: AUDIT_INTERNAL, TENANT_AUDIT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogSearchResponse {
    pub row_count: usize,
}

/// Execute an audit log search request.
///
/// # Errors
/// Returns [`ApiError::BadRequest`] on invalid tenant ID.
pub fn handle_audit_log_search(
    req: &AuditLogSearchRequest,
    olap: &dyn OlapClient,
) -> Result<AuditLogSearchResponse, ApiError> {
    let tenant_id = parse_tenant_id(&req.tenant_id)?;
    let time_range = match (req.from_secs, req.to_secs) {
        (Some(from), Some(to)) => Some(TimeRange {
            from_secs: from,
            to_secs: to,
        }),
        _ => None,
    };
    let domain_req = AuditLogSearch {
        tenant_id: tenant_id.clone(),
        filter: AuditLogFilter {
            actor_id: req.actor_id.clone(),
            resource_type: req.resource_type.clone(),
            action: req.action.clone(),
            time_range,
        },
        pagination: Pagination::new(req.limit.unwrap_or(Pagination::DEFAULT_LIMIT)),
    };
    let uc = SearchAuditLogUseCase::new(olap, tenant_id);
    let rows = uc.execute(&domain_req)?;
    Ok(AuditLogSearchResponse {
        row_count: rows.len(),
    })
}

// =====================================================================
// Billing rollup API
// =====================================================================

/// REST request body for `GET /analytics/v1/billing/rollup`.
///
/// data_class: FINANCIAL
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingRollupRequest {
    pub tenant_id: String,
    pub granularity: RollupGranularity,
    pub from_secs: u64,
    pub to_secs: u64,
}

/// REST response for a billing rollup.
///
/// data_class: FINANCIAL
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingRollupResponse {
    pub row_count: usize,
}

/// Execute a billing rollup request.
///
/// # Errors
/// Returns [`ApiError::BadRequest`] on invalid tenant ID.
pub fn handle_billing_rollup(
    req: &BillingRollupRequest,
    olap: &dyn OlapClient,
) -> Result<BillingRollupResponse, ApiError> {
    let tenant_id = parse_tenant_id(&req.tenant_id)?;
    let domain_req = BillingRollup {
        tenant_id: tenant_id.clone(),
        granularity: req.granularity,
        time_range: TimeRange {
            from_secs: req.from_secs,
            to_secs: req.to_secs,
        },
    };
    let uc = RunBillingRollupUseCase::new(olap, tenant_id);
    let rows = uc.execute(&domain_req)?;
    Ok(BillingRollupResponse {
        row_count: rows.len(),
    })
}

// =====================================================================
// Data export API (deferred)
// =====================================================================

/// REST request body for `POST /analytics/v1/export`.
///
/// data_class: TENANT_PRIVATE, FINANCIAL, PII_IDENTIFYING (scope-dependent)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataExportRequest {
    pub tenant_id: String,
    pub scope: String, // "audit_log" | "billing_history" | "workflow_executions"
    pub format: ExportFormat,
    pub from_secs: u64,
    pub to_secs: u64,
}

/// REST response for a data export initiation.
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataExportResponse {
    /// Export job ID (non_claim: not yet persisted; IP-013 deferred).
    pub export_job_id: String,
}

/// Initiate a data export request.
///
/// # Errors
/// Currently always returns [`ApiError::Unimplemented`] (IP-013 deferred).
pub fn handle_data_export(
    req: &DataExportRequest,
    _olap: &dyn OlapClient,
) -> Result<DataExportResponse, ApiError> {
    let tenant_id = parse_tenant_id(&req.tenant_id)?;
    let time_range = TimeRange {
        from_secs: req.from_secs,
        to_secs: req.to_secs,
    };
    let scope = match req.scope.as_str() {
        "audit_log" => ExportScope::AuditLog { time_range },
        "billing_history" => ExportScope::BillingHistory { time_range },
        "workflow_executions" => ExportScope::WorkflowExecutions { time_range },
        other => {
            return Err(ApiError::BadRequest(format!(
                "unknown export scope: {other}"
            )));
        }
    };
    let domain_req = DataExport {
        tenant_id,
        scope,
        format: req.format,
    };
    let uc = CreateDataExportUseCase::new();
    let _job_id = uc.execute(&domain_req)?;
    Ok(DataExportResponse {
        export_job_id: "deferred".to_string(),
    })
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use shared_olap_client_kernel::{
        ColumnDef, ColumnType, QualifiedTable, TableEngine, TableName, TableSchema,
        memory_adapter::InMemoryOlapClient,
    };

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
    fn dashboard_empty_tenant_bad_request() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "tenant_metrics");
        let req = DashboardQueryRequest {
            tenant_id: "".to_string(),
            metrics: vec![DashboardMetric::ApiCallCount],
            from_secs: 1_000,
            to_secs: 2_000,
            limit: None,
        };
        let err = handle_dashboard_query(&req, &client).unwrap_err();
        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn dashboard_empty_metrics_bad_request() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "tenant_metrics");
        let req = DashboardQueryRequest {
            tenant_id: "t1".to_string(),
            metrics: vec![],
            from_secs: 1_000,
            to_secs: 2_000,
            limit: None,
        };
        let err = handle_dashboard_query(&req, &client).unwrap_err();
        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn dashboard_returns_ok() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "tenant_metrics");
        let req = DashboardQueryRequest {
            tenant_id: "t1".to_string(),
            metrics: vec![DashboardMetric::ApiCallCount],
            from_secs: 1_000,
            to_secs: 2_000,
            limit: Some(10),
        };
        let resp = handle_dashboard_query(&req, &client).unwrap();
        assert_eq!(resp.row_count, 0);
    }

    #[test]
    fn audit_log_search_returns_ok() {
        let mut client = InMemoryOlapClient::new();
        seed_table(&mut client, "t1", "audit_log");
        let req = AuditLogSearchRequest {
            tenant_id: "t1".to_string(),
            actor_id: None,
            resource_type: None,
            action: None,
            from_secs: None,
            to_secs: None,
            limit: Some(25),
        };
        let resp = handle_audit_log_search(&req, &client).unwrap();
        assert_eq!(resp.row_count, 0);
    }

    #[test]
    fn data_export_honestly_unimplemented() {
        let client = InMemoryOlapClient::new();
        let req = DataExportRequest {
            tenant_id: "t1".to_string(),
            scope: "audit_log".to_string(),
            format: ExportFormat::JsonLines,
            from_secs: 1_000,
            to_secs: 2_000,
        };
        let err = handle_data_export(&req, &client).unwrap_err();
        assert!(matches!(err, ApiError::Unimplemented(_)));
    }
}
