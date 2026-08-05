//! Tenant-scoped Postgres/RLS statement adapter for managed-K8s quota.
//!
//! This crate is the service-owned persistence adapter slice for DATA-002. It
//! binds managed-K8s quota reads/writes to the owned `oya-data-sql-kernel` port
//! with a tenant-scoped [`SessionDescriptor`]. The default app must not boot on
//! the in-memory fake; this adapter is the durable-store family that carries the
//! tenant/RLS guard through every SQL statement.

// ADR-0083 Tier-3: production code stays panic-free; tests may use unwrap/expect.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use oya_data_sql_kernel::{
    DataSession, DataSqlError, DataStore, ReadConsistency, ReadQuery, RowSet, SessionScope,
    SqlValue, Statement, WriteBatch,
};
use oya_managed_k8s_tenant_quota_api::{
    ProvisionRequest, QuotaAdminPort, QuotaDecision, QuotaDecisionPort, QuotaPortError, TenantId,
    TenantQuota, TenantUsage, evaluate,
};

const UPSERT_QUOTA_STATEMENT: &str = "managed_k8s_tenant_quota.upsert_quota";
const READ_QUOTA_STATEMENT: &str = "managed_k8s_tenant_quota.read_quota";
const UPSERT_USAGE_STATEMENT: &str = "managed_k8s_tenant_quota.upsert_usage";
const READ_USAGE_STATEMENT: &str = "managed_k8s_tenant_quota.read_usage";

const UPSERT_QUOTA_SQL: &str = "INSERT INTO managed_k8s_tenant_quotas \
     (tenant_id, max_clusters, max_nodes_per_cluster, max_vcpu_per_cluster, max_ram_gib_per_cluster) \
     VALUES ($1, $2, $3, $4, $5) \
     ON CONFLICT (tenant_id) DO UPDATE SET \
       max_clusters = EXCLUDED.max_clusters, \
       max_nodes_per_cluster = EXCLUDED.max_nodes_per_cluster, \
       max_vcpu_per_cluster = EXCLUDED.max_vcpu_per_cluster, \
       max_ram_gib_per_cluster = EXCLUDED.max_ram_gib_per_cluster";
const READ_QUOTA_SQL: &str = "SELECT tenant_id, max_clusters, max_nodes_per_cluster, \
     max_vcpu_per_cluster, max_ram_gib_per_cluster \
     FROM managed_k8s_tenant_quotas WHERE tenant_id = $1";
const UPSERT_USAGE_SQL: &str = "INSERT INTO managed_k8s_tenant_quota_usage \
     (tenant_id, current_clusters, max_nodes_in_any_cluster, max_vcpu_in_any_cluster, \
      max_ram_gib_in_any_cluster) \
     VALUES ($1, $2, $3, $4, $5) \
     ON CONFLICT (tenant_id) DO UPDATE SET \
       current_clusters = EXCLUDED.current_clusters, \
       max_nodes_in_any_cluster = EXCLUDED.max_nodes_in_any_cluster, \
       max_vcpu_in_any_cluster = EXCLUDED.max_vcpu_in_any_cluster, \
       max_ram_gib_in_any_cluster = EXCLUDED.max_ram_gib_in_any_cluster";
const READ_USAGE_SQL: &str = "SELECT tenant_id, current_clusters, max_nodes_in_any_cluster, \
     max_vcpu_in_any_cluster, max_ram_gib_in_any_cluster \
     FROM managed_k8s_tenant_quota_usage WHERE tenant_id = $1";

/// Cloneable quota store backed by one tenant-scoped data SQL session.
///
/// The data session is expected to apply engine RLS (Postgres `SET LOCAL
/// oyatie.tenant_id` in the transitional adapter). This wrapper adds the
/// service-family guard: an app/session scoped to tenant A cannot issue quota
/// reads or writes for tenant B, even before the database policy fires.
pub struct RlsGuardedPostgresQuotaStore<S> {
    session: Arc<Mutex<S>>,
}

impl<S> Clone for RlsGuardedPostgresQuotaStore<S> {
    fn clone(&self) -> Self {
        Self {
            session: Arc::clone(&self.session),
        }
    }
}

impl<S> RlsGuardedPostgresQuotaStore<S>
where
    S: DataSession + Send,
{
    /// Construct a quota store over a tenant-data SQL session.
    ///
    /// # Errors
    /// Returns [`QuotaPortError`] if the session is not a tenant-scoped
    /// `TenantData` session.
    pub fn new(session: S) -> Result<Self, QuotaPortError> {
        session.descriptor().validate().map_err(map_data_error)?;
        if session.descriptor().store != DataStore::TenantData {
            return Err(QuotaPortError::Persistence(
                "managed-k8s quota store requires DataStore::TenantData".to_owned(),
            ));
        }
        if !matches!(session.descriptor().scope, SessionScope::Tenant { .. }) {
            return Err(QuotaPortError::Persistence(
                "managed-k8s quota store requires a tenant-scoped RLS session".to_owned(),
            ));
        }
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
        })
    }

    fn with_session<T>(
        &self,
        f: impl FnOnce(&mut S) -> Result<T, QuotaPortError>,
    ) -> Result<T, QuotaPortError> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| QuotaPortError::Persistence("quota session mutex poisoned".to_owned()))?;
        f(&mut session)
    }
}

impl<S> QuotaAdminPort for RlsGuardedPostgresQuotaStore<S>
where
    S: DataSession + Send,
{
    fn set_quota(&self, quota: TenantQuota) -> Result<(), QuotaPortError> {
        self.with_session(|session| {
            ensure_session_tenant(session, quota.tenant_id.as_str())?;
            let statement = quota_upsert_statement(&quota)?;
            let batch = WriteBatch::new(vec![statement]).map_err(map_data_error)?;
            session.execute_write(&batch).map_err(map_data_error)?;
            Ok(())
        })
    }

    fn get_quota(&self, tenant_id: &TenantId) -> Result<TenantQuota, QuotaPortError> {
        self.with_session(|session| {
            ensure_session_tenant(session, tenant_id.as_str())?;
            let query = quota_read_query(tenant_id)?;
            let row_set = session.execute_read(&query).map_err(map_data_error)?;
            quota_from_rows(&row_set)?
                .ok_or_else(|| QuotaPortError::NotFound(tenant_id.as_str().to_owned()))
        })
    }

    fn get_usage(&self, tenant_id: &TenantId) -> Result<TenantUsage, QuotaPortError> {
        self.with_session(|session| {
            ensure_session_tenant(session, tenant_id.as_str())?;
            let query = usage_read_query(tenant_id)?;
            let row_set = session.execute_read(&query).map_err(map_data_error)?;
            usage_from_rows(&row_set)?
                .ok_or_else(|| QuotaPortError::NotFound(tenant_id.as_str().to_owned()))
        })
    }

    fn set_usage(&self, usage: TenantUsage) -> Result<(), QuotaPortError> {
        self.with_session(|session| {
            ensure_session_tenant(session, usage.tenant_id.as_str())?;
            let statement = usage_upsert_statement(&usage)?;
            let batch = WriteBatch::new(vec![statement]).map_err(map_data_error)?;
            session.execute_write(&batch).map_err(map_data_error)?;
            Ok(())
        })
    }
}

impl<S> QuotaDecisionPort for RlsGuardedPostgresQuotaStore<S>
where
    S: DataSession + Send,
{
    fn check_quota(&self, request: &ProvisionRequest) -> Result<QuotaDecision, QuotaPortError> {
        let quota = self.get_quota(&request.tenant_id)?;
        let usage = match self.get_usage(&request.tenant_id) {
            Ok(usage) => usage,
            Err(QuotaPortError::NotFound(_)) => TenantUsage {
                tenant_id: request.tenant_id.clone(),
                current_clusters: 0,
                max_nodes_in_any_cluster: 0,
                max_vcpu_in_any_cluster: 0,
                max_ram_gib_in_any_cluster: 0,
            },
            Err(error) => return Err(error),
        };
        Ok(evaluate(&quota, &usage, request))
    }
}

fn ensure_session_tenant<S: DataSession>(
    session: &S,
    requested: &str,
) -> Result<(), QuotaPortError> {
    match &session.descriptor().scope {
        SessionScope::Tenant { tenant_id, .. } if tenant_id == requested => Ok(()),
        SessionScope::Tenant { tenant_id, .. } => Err(QuotaPortError::Persistence(format!(
            "cross-tenant quota access denied: session tenant {tenant_id} cannot access tenant {requested}"
        ))),
        SessionScope::ControlPlane { .. } => Err(QuotaPortError::Persistence(
            "tenant quota store requires a tenant-scoped RLS session".to_owned(),
        )),
    }
}

fn quota_upsert_statement(quota: &TenantQuota) -> Result<Statement, QuotaPortError> {
    Statement::new(
        UPSERT_QUOTA_STATEMENT,
        UPSERT_QUOTA_SQL,
        vec![
            SqlValue::Text(quota.tenant_id.as_str().to_owned()),
            SqlValue::Int64(i64::from(quota.max_clusters)),
            SqlValue::Int64(i64::from(quota.max_nodes_per_cluster)),
            SqlValue::Int64(i64::from(quota.max_vcpu_per_cluster)),
            SqlValue::Int64(i64::from(quota.max_ram_gib_per_cluster)),
        ],
    )
    .map_err(map_data_error)
}

fn quota_read_query(tenant_id: &TenantId) -> Result<ReadQuery, QuotaPortError> {
    ReadQuery::new(
        Statement::new(
            READ_QUOTA_STATEMENT,
            READ_QUOTA_SQL,
            vec![SqlValue::Text(tenant_id.as_str().to_owned())],
        )
        .map_err(map_data_error)?,
        ReadConsistency::Strong,
    )
    .map_err(map_data_error)
}

fn usage_upsert_statement(usage: &TenantUsage) -> Result<Statement, QuotaPortError> {
    Statement::new(
        UPSERT_USAGE_STATEMENT,
        UPSERT_USAGE_SQL,
        vec![
            SqlValue::Text(usage.tenant_id.as_str().to_owned()),
            SqlValue::Int64(i64::from(usage.current_clusters)),
            SqlValue::Int64(i64::from(usage.max_nodes_in_any_cluster)),
            SqlValue::Int64(i64::from(usage.max_vcpu_in_any_cluster)),
            SqlValue::Int64(i64::from(usage.max_ram_gib_in_any_cluster)),
        ],
    )
    .map_err(map_data_error)
}

fn usage_read_query(tenant_id: &TenantId) -> Result<ReadQuery, QuotaPortError> {
    ReadQuery::new(
        Statement::new(
            READ_USAGE_STATEMENT,
            READ_USAGE_SQL,
            vec![SqlValue::Text(tenant_id.as_str().to_owned())],
        )
        .map_err(map_data_error)?,
        ReadConsistency::Strong,
    )
    .map_err(map_data_error)
}

fn quota_from_rows(row_set: &RowSet) -> Result<Option<TenantQuota>, QuotaPortError> {
    let Some(row) = single_row(row_set)? else {
        return Ok(None);
    };
    let tenant_id = text_column(row_set, row, "tenant_id")?;
    TenantQuota::new(
        tenant_id,
        u32_column(row_set, row, "max_clusters")?,
        u32_column(row_set, row, "max_nodes_per_cluster")?,
        u32_column(row_set, row, "max_vcpu_per_cluster")?,
        u32_column(row_set, row, "max_ram_gib_per_cluster")?,
    )
    .map(Some)
    .map_err(QuotaPortError::from)
}

fn usage_from_rows(row_set: &RowSet) -> Result<Option<TenantUsage>, QuotaPortError> {
    let Some(row) = single_row(row_set)? else {
        return Ok(None);
    };
    let tenant_id = text_column(row_set, row, "tenant_id")?;
    TenantUsage::new(
        tenant_id,
        u32_column(row_set, row, "current_clusters")?,
        u32_column(row_set, row, "max_nodes_in_any_cluster")?,
        u32_column(row_set, row, "max_vcpu_in_any_cluster")?,
        u32_column(row_set, row, "max_ram_gib_in_any_cluster")?,
    )
    .map(Some)
    .map_err(QuotaPortError::from)
}

fn single_row(row_set: &RowSet) -> Result<Option<&[SqlValue]>, QuotaPortError> {
    match row_set.rows.as_slice() {
        [] => Ok(None),
        [row] => Ok(Some(row.as_slice())),
        rows => Err(QuotaPortError::Persistence(format!(
            "quota query returned {} rows; expected at most one",
            rows.len()
        ))),
    }
}

fn column_index(row_set: &RowSet, name: &str) -> Result<usize, QuotaPortError> {
    row_set
        .columns
        .iter()
        .position(|column| column == name)
        .ok_or_else(|| QuotaPortError::Persistence(format!("quota row missing column {name}")))
}

fn text_column(row_set: &RowSet, row: &[SqlValue], name: &str) -> Result<String, QuotaPortError> {
    match row.get(column_index(row_set, name)?) {
        Some(SqlValue::Text(value)) => Ok(value.clone()),
        other => Err(QuotaPortError::Persistence(format!(
            "quota column {name} expected text, got {other:?}"
        ))),
    }
}

fn u32_column(row_set: &RowSet, row: &[SqlValue], name: &str) -> Result<u32, QuotaPortError> {
    match row.get(column_index(row_set, name)?) {
        Some(SqlValue::Int64(value)) => u32::try_from(*value).map_err(|_| {
            QuotaPortError::Persistence(format!(
                "quota column {name} value {value} is outside u32 range"
            ))
        }),
        other => Err(QuotaPortError::Persistence(format!(
            "quota column {name} expected int64, got {other:?}"
        ))),
    }
}

fn map_data_error(error: DataSqlError) -> QuotaPortError {
    QuotaPortError::Persistence(error.to_string())
}
