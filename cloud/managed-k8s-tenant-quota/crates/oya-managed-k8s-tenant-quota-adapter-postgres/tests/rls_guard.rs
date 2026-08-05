#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use oya_data_sql_kernel::clock::HlcTimestamp;
use oya_data_sql_kernel::{
    CommitReceipt, DataSession, DataSqlError, DataStore, ReadQuery, RowSet, SessionDescriptor,
    SessionScope, SqlValue, Statement, WriteBatch,
};
use oya_managed_k8s_tenant_quota_adapter_postgres::RlsGuardedPostgresQuotaStore;
use oya_managed_k8s_tenant_quota_api::{
    ProvisionRequest, QuotaAdminPort, QuotaDecisionPort, QuotaPortError, TenantId, TenantQuota,
    TenantUsage,
};

#[derive(Debug, Default)]
struct FixtureData {
    quotas: BTreeMap<String, TenantQuota>,
    usages: BTreeMap<String, TenantUsage>,
    statements: Vec<Statement>,
}

#[derive(Debug)]
struct RlsFixtureSession {
    descriptor: SessionDescriptor,
    data: Arc<Mutex<FixtureData>>,
}

impl RlsFixtureSession {
    fn tenant(tenant_id: &str, data: Arc<Mutex<FixtureData>>) -> Self {
        Self {
            descriptor: SessionDescriptor::tenant_data(
                tenant_id,
                "cell-001",
                "managed-k8s-tenant-quota-test",
            )
            .unwrap(),
            data,
        }
    }
}

impl DataSession for RlsFixtureSession {
    fn descriptor(&self) -> &SessionDescriptor {
        &self.descriptor
    }

    fn execute_write(&mut self, batch: &WriteBatch) -> Result<CommitReceipt, DataSqlError> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| DataSqlError::Adapter("fixture mutex poisoned".to_owned()))?;
        for statement in &batch.statements {
            let statement_tenant = first_text_param(statement)?;
            ensure_fixture_rls(&self.descriptor, statement_tenant)?;
            match statement.name.as_str() {
                "managed_k8s_tenant_quota.upsert_quota" => {
                    let quota = TenantQuota::new(
                        statement_tenant.to_owned(),
                        u32_param(statement, 1)?,
                        u32_param(statement, 2)?,
                        u32_param(statement, 3)?,
                        u32_param(statement, 4)?,
                    )
                    .map_err(|error| DataSqlError::Adapter(error.to_string()))?;
                    data.quotas.insert(statement_tenant.to_owned(), quota);
                }
                "managed_k8s_tenant_quota.upsert_usage" => {
                    let usage = TenantUsage::new(
                        statement_tenant.to_owned(),
                        u32_param(statement, 1)?,
                        u32_param(statement, 2)?,
                        u32_param(statement, 3)?,
                        u32_param(statement, 4)?,
                    )
                    .map_err(|error| DataSqlError::Adapter(error.to_string()))?;
                    data.usages.insert(statement_tenant.to_owned(), usage);
                }
                other => {
                    return Err(DataSqlError::Adapter(format!(
                        "unexpected write statement {other}"
                    )));
                }
            }
            data.statements.push(statement.clone());
        }
        Ok(CommitReceipt {
            store: DataStore::TenantData,
            commit_timestamp: HlcTimestamp::new(data.statements.len() as u64, 0),
            statement_names: batch.statement_names(),
        })
    }

    fn execute_read(&mut self, query: &ReadQuery) -> Result<RowSet, DataSqlError> {
        let statement_tenant = first_text_param(&query.statement)?;
        ensure_fixture_rls(&self.descriptor, statement_tenant)?;
        let data = self
            .data
            .lock()
            .map_err(|_| DataSqlError::Adapter("fixture mutex poisoned".to_owned()))?;
        match query.statement.name.as_str() {
            "managed_k8s_tenant_quota.read_quota" => {
                data.quotas
                    .get(statement_tenant)
                    .map_or_else(empty_quota_rows, |quota| {
                        RowSet::new(
                            quota_columns(),
                            vec![vec![
                                SqlValue::Text(quota.tenant_id.as_str().to_owned()),
                                SqlValue::Int64(i64::from(quota.max_clusters)),
                                SqlValue::Int64(i64::from(quota.max_nodes_per_cluster)),
                                SqlValue::Int64(i64::from(quota.max_vcpu_per_cluster)),
                                SqlValue::Int64(i64::from(quota.max_ram_gib_per_cluster)),
                            ]],
                        )
                    })
            }
            "managed_k8s_tenant_quota.read_usage" => {
                data.usages
                    .get(statement_tenant)
                    .map_or_else(empty_usage_rows, |usage| {
                        RowSet::new(
                            usage_columns(),
                            vec![vec![
                                SqlValue::Text(usage.tenant_id.as_str().to_owned()),
                                SqlValue::Int64(i64::from(usage.current_clusters)),
                                SqlValue::Int64(i64::from(usage.max_nodes_in_any_cluster)),
                                SqlValue::Int64(i64::from(usage.max_vcpu_in_any_cluster)),
                                SqlValue::Int64(i64::from(usage.max_ram_gib_in_any_cluster)),
                            ]],
                        )
                    })
            }
            other => Err(DataSqlError::Adapter(format!(
                "unexpected read statement {other}"
            ))),
        }
    }
}

#[test]
fn tenant_scoped_quota_store_round_trips_and_denies_cross_tenant_access() {
    let fixture = Arc::new(Mutex::new(FixtureData::default()));
    let session = RlsFixtureSession::tenant("ten_acme", Arc::clone(&fixture));
    let store = RlsGuardedPostgresQuotaStore::new(session).unwrap();

    let acme_quota = TenantQuota::new("ten_acme", 5, 10, 32, 128).unwrap();
    store.set_quota(acme_quota.clone()).unwrap();
    let fetched = store
        .get_quota(&TenantId::new("ten_acme").unwrap())
        .unwrap();
    assert_eq!(fetched, acme_quota);

    let decision = store
        .check_quota(&ProvisionRequest::new("ten_acme", 1, 3, 8, 32).unwrap())
        .unwrap();
    assert!(decision.is_allow());

    let cross_tenant_write = store
        .set_quota(TenantQuota::new("ten_globex", 3, 5, 16, 64).unwrap())
        .unwrap_err();
    assert_cross_tenant_error(cross_tenant_write);

    let cross_tenant_read = store
        .get_quota(&TenantId::new("ten_globex").unwrap())
        .unwrap_err();
    assert_cross_tenant_error(cross_tenant_read);

    let data = fixture.lock().unwrap();
    let statement = data
        .statements
        .iter()
        .find(|statement| statement.name == "managed_k8s_tenant_quota.upsert_quota")
        .expect("quota upsert statement recorded");
    assert!(
        statement.sql.contains("tenant_id"),
        "quota SQL must carry the tenant RLS column"
    );
    assert_eq!(
        statement.params.first(),
        Some(&SqlValue::Text("ten_acme".to_owned())),
        "tenant_id must be the first bound parameter so the data adapter can apply RLS"
    );
}

fn assert_cross_tenant_error(error: QuotaPortError) {
    let rendered = error.to_string();
    assert!(
        rendered.contains("cross-tenant quota access denied"),
        "unexpected error: {rendered}"
    );
}

fn ensure_fixture_rls(descriptor: &SessionDescriptor, requested: &str) -> Result<(), DataSqlError> {
    match &descriptor.scope {
        SessionScope::Tenant { tenant_id, .. } if tenant_id == requested => Ok(()),
        SessionScope::Tenant { tenant_id, .. } => Err(DataSqlError::Adapter(format!(
            "fixture RLS denied session tenant {tenant_id} accessing tenant {requested}"
        ))),
        SessionScope::ControlPlane { .. } => Err(DataSqlError::Adapter(
            "fixture RLS requires tenant scope".to_owned(),
        )),
    }
}

fn first_text_param(statement: &Statement) -> Result<&str, DataSqlError> {
    match statement.params.first() {
        Some(SqlValue::Text(value)) => Ok(value.as_str()),
        other => Err(DataSqlError::Adapter(format!(
            "expected first param to be tenant text, got {other:?}"
        ))),
    }
}

fn u32_param(statement: &Statement, index: usize) -> Result<u32, DataSqlError> {
    match statement.params.get(index) {
        Some(SqlValue::Int64(value)) => u32::try_from(*value)
            .map_err(|_| DataSqlError::Adapter(format!("param {index} out of u32 range"))),
        other => Err(DataSqlError::Adapter(format!(
            "expected int64 param at {index}, got {other:?}"
        ))),
    }
}

fn quota_columns() -> Vec<String> {
    [
        "tenant_id",
        "max_clusters",
        "max_nodes_per_cluster",
        "max_vcpu_per_cluster",
        "max_ram_gib_per_cluster",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn usage_columns() -> Vec<String> {
    [
        "tenant_id",
        "current_clusters",
        "max_nodes_in_any_cluster",
        "max_vcpu_in_any_cluster",
        "max_ram_gib_in_any_cluster",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn empty_quota_rows() -> Result<RowSet, DataSqlError> {
    RowSet::new(quota_columns(), vec![])
}

fn empty_usage_rows() -> Result<RowSet, DataSqlError> {
    RowSet::new(usage_columns(), vec![])
}
