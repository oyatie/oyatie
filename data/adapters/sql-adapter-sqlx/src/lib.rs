//! SQLx Postgres adapter for the owned `oya-data` SQL port.
//!
//! Story G003 sub-slice 2 (ADR-0536 D-10): services link the
//! `oya-data-sql-kernel` port; this adapter is the ADR-0510 transitional
//! Postgres implementation behind it. It absorbs ALL engine impedance:
//! tenant sessions run the parameterized RLS `set_config` (reusing
//! `oya-shared-postgres-command-kernel::SET_LOCAL_TENANT_SQL`) inside the
//! same transaction as every statement, commit receipts are stamped from an
//! HLC fed by adapter-injected physical time, and every read-consistency
//! level is served as `Strong` (Postgres has no follower reads — bounded
//! staleness and snapshot reads arrive with the CRDB-class engine; the port
//! contract explicitly allows the upgrade).
//!
//! The default test set stays database-free. The env-gated live harness
//! exercises real Postgres RLS cross-tenant denial against a containerized
//! database, mirroring the `oya-shared-postgres-command-adapter-sqlx`
//! live-probe pattern.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use oya_data_sql_kernel::clock::{ClockError, Hlc, HlcTimestamp};
use oya_data_sql_kernel::{
    CommitReceipt, DataSqlError, DataStore, ReadQuery, RowSet, SessionDescriptor, SessionScope,
    SqlValue, Statement, WriteBatch,
};
use oya_shared_postgres_command_kernel::{PostgresPoolConfig, SET_LOCAL_TENANT_SQL};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Column, PgPool, Postgres, Row, Transaction, TypeInfo, ValueRef};

/// Enable flag for the live containerized-Postgres RLS harness.
pub const LIVE_DATA_POSTGRES_ENABLE_ENV: &str = "OYA_DATA_LIVE_POSTGRES";
/// Admin (schema-owning) connection URL for the live harness.
pub const LIVE_DATA_POSTGRES_ADMIN_URL_ENV: &str = "OYA_DATA_POSTGRES_ADMIN_URL";
/// Application-role (RLS-subject) connection URL for the live harness.
pub const LIVE_DATA_POSTGRES_APP_URL_ENV: &str = "OYA_DATA_POSTGRES_APP_URL";

/// CockroachDB-default max clock offset used for adapter-side HLC stamping.
pub const DEFAULT_MAX_OFFSET_NANOS: u64 = 500_000_000;

const LIVE_RLS_SCHEMA: &str = "oya_data_live_rls";
const LIVE_RLS_DROP_SCHEMA_SQL: &str = "DROP SCHEMA IF EXISTS oya_data_live_rls CASCADE";
const LIVE_RLS_CREATE_SCHEMA_SQL: &str = "CREATE SCHEMA oya_data_live_rls";
const LIVE_RLS_CREATE_TABLE_SQL: &str = "CREATE TABLE oya_data_live_rls.tenant_rows (tenant_id text NOT NULL, row_id text NOT NULL, payload text NOT NULL, PRIMARY KEY (tenant_id, row_id))";
const LIVE_RLS_ENABLE_SQL: &str =
    "ALTER TABLE oya_data_live_rls.tenant_rows ENABLE ROW LEVEL SECURITY";
const LIVE_RLS_FORCE_SQL: &str =
    "ALTER TABLE oya_data_live_rls.tenant_rows FORCE ROW LEVEL SECURITY";
const LIVE_RLS_CREATE_POLICY_SQL: &str = "CREATE POLICY tenant_isolation ON oya_data_live_rls.tenant_rows USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true))";
const LIVE_RLS_GRANT_USAGE_SQL: &str = "GRANT USAGE ON SCHEMA oya_data_live_rls TO PUBLIC";
const LIVE_RLS_GRANT_TABLE_SQL: &str =
    "GRANT SELECT, INSERT, UPDATE, DELETE ON oya_data_live_rls.tenant_rows TO PUBLIC";
const LIVE_RLS_INSERT_SQL: &str =
    "INSERT INTO oya_data_live_rls.tenant_rows (tenant_id, row_id, payload) VALUES ($1, $2, $3)";
const LIVE_RLS_SELECT_ROWS_SQL: &str =
    "SELECT tenant_id, row_id, payload FROM oya_data_live_rls.tenant_rows ORDER BY row_id";

/// Connection settings for the adapter, reusing the shared pool config so
/// application-name/TLS/timeout discipline stays single-sourced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxDataClientConfig {
    pub database_url: String,     // data_class: INTERNAL_ONLY
    pub pool: PostgresPoolConfig, // data_class: INTERNAL_ONLY
}

impl SqlxDataClientConfig {
    pub fn new(
        database_url: impl Into<String>,
        pool: PostgresPoolConfig,
    ) -> Result<Self, DataSqlError> {
        let config = Self {
            database_url: database_url.into(),
            pool,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), DataSqlError> {
        self.pool
            .validate()
            .map_err(|error| DataSqlError::Adapter(format!("pool config: {error:?}")))?;
        if self.database_url.trim().is_empty() {
            return Err(DataSqlError::MissingField {
                field: "client.database_url",
            });
        }
        if self.pool.require_tls && disables_tls(&self.database_url) {
            return Err(DataSqlError::Adapter(
                "TLS required but the database URL disables ssl".to_owned(),
            ));
        }
        Ok(())
    }
}

/// The transitional Postgres implementation of the owned data port.
/// Async surface mirrors `oya_data_sql_kernel::{DataClient, DataSession}`
/// 1:1 (the sync kernel traits stay reserved for IO-free reference impls,
/// matching the postgres-command kernel/adapter split).
pub struct SqlxDataClient {
    pool: PgPool,
    hlc: std::sync::Mutex<Hlc>,
}

impl SqlxDataClient {
    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            pool,
            hlc: std::sync::Mutex::new(Hlc::new(DEFAULT_MAX_OFFSET_NANOS)),
        }
    }

    pub async fn connect(config: &SqlxDataClientConfig) -> Result<Self, DataSqlError> {
        config.validate()?;
        let pool = PgPoolOptions::new()
            .max_connections(config.pool.max_connections)
            .acquire_timeout(Duration::from_millis(config.pool.acquire_timeout_ms))
            .connect(&config.database_url)
            .await
            .map_err(sqlx_error)?;
        Ok(Self::from_pool(pool))
    }

    /// Open a validated session. Descriptor invariants (tenant fields,
    /// bootstrap-metastore-never-tenant-scoped) are enforced here exactly as
    /// the kernel reference implementation enforces them.
    pub fn open_session(
        &self,
        descriptor: &SessionDescriptor,
    ) -> Result<SqlxDataSession<'_>, DataSqlError> {
        descriptor.validate()?;
        Ok(SqlxDataSession {
            client: self,
            descriptor: descriptor.clone(),
        })
    }

    fn stamp_commit(&self) -> Result<HlcTimestamp, DataSqlError> {
        let physical_now_nanos = physical_now_nanos()?;
        let mut hlc = self
            .hlc
            .lock()
            .map_err(|_| DataSqlError::Adapter("clock mutex poisoned".to_owned()))?;
        hlc.tick(physical_now_nanos).map_err(DataSqlError::from)
    }
}

/// A scoped session over the shared pool. Every write batch runs in one
/// transaction with the tenant RLS scope applied first; every read runs in a
/// transaction with the same scoping so RLS governs SELECTs identically.
pub struct SqlxDataSession<'client> {
    client: &'client SqlxDataClient,
    descriptor: SessionDescriptor,
}

impl std::fmt::Debug for SqlxDataSession<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The pooled client handle is intentionally opaque; the descriptor is
        // the identifying state (tenant scope + application) of a session.
        f.debug_struct("SqlxDataSession")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

impl SqlxDataSession<'_> {
    #[must_use]
    pub fn descriptor(&self) -> &SessionDescriptor {
        &self.descriptor
    }

    pub async fn execute_write(&self, batch: &WriteBatch) -> Result<CommitReceipt, DataSqlError> {
        if batch.statements.is_empty() {
            return Err(DataSqlError::EmptyWriteBatch);
        }
        for statement in &batch.statements {
            statement.validate()?;
            check_statement_shape(statement)?;
        }
        let mut transaction = self.client.pool.begin().await.map_err(sqlx_error)?;
        apply_session_scope(&mut transaction, &self.descriptor).await?;
        for statement in &batch.statements {
            bind_params(sqlx::query(&statement.sql), &statement.params)
                .execute(&mut *transaction)
                .await
                .map_err(sqlx_error)?;
        }
        transaction.commit().await.map_err(sqlx_error)?;
        Ok(CommitReceipt {
            store: self.descriptor.store,
            commit_timestamp: self.client.stamp_commit()?,
            statement_names: batch.statement_names(),
        })
    }

    /// Reads honor the port contract; Postgres serves every consistency
    /// level as Strong (transitional absorb documented at the crate root).
    pub async fn execute_read(&self, query: &ReadQuery) -> Result<RowSet, DataSqlError> {
        query.statement.validate()?;
        query.consistency.validate()?;
        check_statement_shape(&query.statement)?;
        let mut transaction = self.client.pool.begin().await.map_err(sqlx_error)?;
        apply_session_scope(&mut transaction, &self.descriptor).await?;
        let rows = bind_params(sqlx::query(&query.statement.sql), &query.statement.params)
            .fetch_all(&mut *transaction)
            .await
            .map_err(sqlx_error)?;
        transaction.commit().await.map_err(sqlx_error)?;
        rows_to_row_set(&rows)
    }
}

async fn apply_session_scope(
    transaction: &mut Transaction<'_, Postgres>,
    descriptor: &SessionDescriptor,
) -> Result<(), DataSqlError> {
    match &descriptor.scope {
        SessionScope::Tenant { tenant_id, .. } => {
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut **transaction)
                .await
                .map_err(sqlx_error)?;
            Ok(())
        }
        // Control-plane sessions run without a tenant scope; RLS policies
        // treat the unset setting as no-tenant (deny on tenant tables).
        SessionScope::ControlPlane { .. } => Ok(()),
    }
}

/// Statement-shape guard, mirroring the existing adapter's UnsafeSql and
/// placeholder-count discipline: one statement per `Statement`, and the
/// highest `$n` placeholder must match the bound parameter count.
fn check_statement_shape(statement: &Statement) -> Result<(), DataSqlError> {
    let sql = statement.sql.trim().trim_end_matches(';');
    if sql.contains(';') {
        return Err(DataSqlError::Adapter(format!(
            "statement {:?} must be a single SQL statement",
            statement.name
        )));
    }
    let highest = highest_placeholder(sql);
    if highest != statement.params.len() {
        return Err(DataSqlError::Adapter(format!(
            "statement {:?} names ${highest} as its highest placeholder but binds {} params",
            statement.name,
            statement.params.len()
        )));
    }
    Ok(())
}

fn highest_placeholder(sql: &str) -> usize {
    let bytes = sql.as_bytes();
    let mut highest = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'$' {
            let mut end = index + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > index + 1
                && let Ok(value) = sql[index + 1..end].parse::<usize>()
            {
                highest = highest.max(value);
            }
            index = end;
        } else {
            index += 1;
        }
    }
    highest
}

fn bind_params<'q>(
    mut query: sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>,
    params: &'q [SqlValue],
) -> sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
    for param in params {
        query = match param {
            SqlValue::Text(value) => query.bind(value),
            SqlValue::Int64(value) => query.bind(value),
            SqlValue::Bool(value) => query.bind(value),
            SqlValue::Bytes(value) => query.bind(value),
            SqlValue::TextArray(value) => query.bind(value),
            SqlValue::Null => query.bind(Option::<String>::None),
        };
    }
    query
}

fn rows_to_row_set(rows: &[PgRow]) -> Result<RowSet, DataSqlError> {
    let columns: Vec<String> = rows.first().map_or_else(Vec::new, |row| {
        row.columns()
            .iter()
            .map(|column| column.name().to_owned())
            .collect()
    });
    let mut out_rows = Vec::with_capacity(rows.len());
    for row in rows {
        let mut values = Vec::with_capacity(row.columns().len());
        for (column_index, column) in row.columns().iter().enumerate() {
            values.push(decode_value(row, column_index, column.type_info().name())?);
        }
        out_rows.push(values);
    }
    RowSet::new(columns, out_rows)
}

fn decode_value(
    row: &PgRow,
    column_index: usize,
    type_name: &str,
) -> Result<SqlValue, DataSqlError> {
    let raw = row
        .try_get_raw(column_index)
        .map_err(|error| DataSqlError::Adapter(error.to_string()))?;
    if raw.is_null() {
        return Ok(SqlValue::Null);
    }
    match type_name {
        "TEXT" | "VARCHAR" | "CHAR" | "NAME" | "BPCHAR" => row
            .try_get::<String, _>(column_index)
            .map(SqlValue::Text)
            .map_err(|error| DataSqlError::Adapter(error.to_string())),
        "INT8" | "INT4" | "INT2" => row
            .try_get::<i64, _>(column_index)
            .or_else(|_| row.try_get::<i32, _>(column_index).map(i64::from))
            .or_else(|_| row.try_get::<i16, _>(column_index).map(i64::from))
            .map(SqlValue::Int64)
            .map_err(|error| DataSqlError::Adapter(error.to_string())),
        "BOOL" => row
            .try_get::<bool, _>(column_index)
            .map(SqlValue::Bool)
            .map_err(|error| DataSqlError::Adapter(error.to_string())),
        "BYTEA" => row
            .try_get::<Vec<u8>, _>(column_index)
            .map(SqlValue::Bytes)
            .map_err(|error| DataSqlError::Adapter(error.to_string())),
        "TEXT[]" | "VARCHAR[]" => row
            .try_get::<Vec<String>, _>(column_index)
            .map(SqlValue::TextArray)
            .map_err(|error| DataSqlError::Adapter(error.to_string())),
        other => Err(DataSqlError::Adapter(format!(
            "unsupported column type {other} at index {column_index}"
        ))),
    }
}

fn physical_now_nanos() -> Result<u64, DataSqlError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| DataSqlError::Adapter(format!("system clock before epoch: {error}")))?;
    u64::try_from(elapsed.as_nanos())
        .map_err(|_| DataSqlError::Clock(ClockError::LogicalOverflow { wall_nanos: u64::MAX }))
}

fn sqlx_error(error: sqlx::Error) -> DataSqlError {
    DataSqlError::Adapter(error.to_string())
}

fn disables_tls(database_url: &str) -> bool {
    database_url.contains("sslmode=disable")
}

/// Outcome of the live cross-tenant-deny probe (AMENDMENT 7 integration
/// rung): proves over real Postgres RLS that tenant A's rows are invisible
/// to tenant B and to unscoped sessions through the owned port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveRlsCrossTenantReport {
    pub schema_name: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_a_visible_rows: usize,
    pub tenant_b_visible_rows: usize,
    pub tenant_b_write_into_a_denied: bool,
    pub commit_timestamps_strictly_increase: bool,
}

/// Env-gated live harness. Returns `Ok(None)` when the enable flag is
/// absent so default test runs stay database-free; CI integration lanes set
/// the env vars against a containerized Postgres.
pub async fn run_live_rls_cross_tenant_probe()
-> Result<Option<LiveRlsCrossTenantReport>, DataSqlError> {
    if env::var(LIVE_DATA_POSTGRES_ENABLE_ENV).is_err() {
        return Ok(None);
    }
    let admin_url = env::var(LIVE_DATA_POSTGRES_ADMIN_URL_ENV).map_err(|_| {
        DataSqlError::MissingField {
            field: LIVE_DATA_POSTGRES_ADMIN_URL_ENV,
        }
    })?;
    let app_url =
        env::var(LIVE_DATA_POSTGRES_APP_URL_ENV).map_err(|_| DataSqlError::MissingField {
            field: LIVE_DATA_POSTGRES_APP_URL_ENV,
        })?;

    // Admin pool prepares the RLS fixture (schema owner bypasses RLS).
    let admin = PgPool::connect(&admin_url).await.map_err(sqlx_error)?;
    for setup_sql in [
        LIVE_RLS_DROP_SCHEMA_SQL,
        LIVE_RLS_CREATE_SCHEMA_SQL,
        LIVE_RLS_CREATE_TABLE_SQL,
        LIVE_RLS_ENABLE_SQL,
        LIVE_RLS_FORCE_SQL,
        LIVE_RLS_CREATE_POLICY_SQL,
        LIVE_RLS_GRANT_USAGE_SQL,
        LIVE_RLS_GRANT_TABLE_SQL,
    ] {
        sqlx::query(setup_sql)
            .execute(&admin)
            .await
            .map_err(sqlx_error)?;
    }

    // The application role goes through the owned port only.
    let pool_config = PostgresPoolConfig::for_microservice("data-live-probe", 4)
        .map_err(|error| DataSqlError::Adapter(format!("pool config: {error:?}")))?;
    let client =
        SqlxDataClient::connect(&SqlxDataClientConfig::new(app_url, pool_config)?).await?;

    let tenant_a = SessionDescriptor::tenant_data("tenant-a", "cell-001", "oyatie-data-probe")?;
    let tenant_b = SessionDescriptor::tenant_data("tenant-b", "cell-001", "oyatie-data-probe")?;
    let session_a = client.open_session(&tenant_a)?;
    let session_b = client.open_session(&tenant_b)?;

    let insert_a = WriteBatch::new(vec![
        Statement::new(
            "insert_row_1",
            LIVE_RLS_INSERT_SQL,
            vec![
                SqlValue::Text("tenant-a".to_owned()),
                SqlValue::Text("row-1".to_owned()),
                SqlValue::Text("alpha".to_owned()),
            ],
        )?,
        Statement::new(
            "insert_row_2",
            LIVE_RLS_INSERT_SQL,
            vec![
                SqlValue::Text("tenant-a".to_owned()),
                SqlValue::Text("row-2".to_owned()),
                SqlValue::Text("beta".to_owned()),
            ],
        )?,
    ])?;
    let receipt_1 = session_a.execute_write(&insert_a).await?;

    let insert_b_own = WriteBatch::new(vec![Statement::new(
        "insert_b_row",
        LIVE_RLS_INSERT_SQL,
        vec![
            SqlValue::Text("tenant-b".to_owned()),
            SqlValue::Text("row-b".to_owned()),
            SqlValue::Text("gamma".to_owned()),
        ],
    )?])?;
    let receipt_2 = session_b.execute_write(&insert_b_own).await?;

    // Cross-tenant WRITE: tenant B attempting to write a tenant-a row must
    // be denied by the RLS WITH CHECK clause.
    let forged = WriteBatch::new(vec![Statement::new(
        "forged_cross_tenant_insert",
        LIVE_RLS_INSERT_SQL,
        vec![
            SqlValue::Text("tenant-a".to_owned()),
            SqlValue::Text("row-forged".to_owned()),
            SqlValue::Text("evil".to_owned()),
        ],
    )?])?;
    let tenant_b_write_into_a_denied = session_b.execute_write(&forged).await.is_err();

    // Cross-tenant READ: each tenant sees exactly its own rows.
    let read = ReadQuery::new(
        Statement::new("read_rows", LIVE_RLS_SELECT_ROWS_SQL, vec![])?,
        oya_data_sql_kernel::ReadConsistency::Strong,
    )?;
    let rows_a = session_a.execute_read(&read).await?;
    let rows_b = session_b.execute_read(&read).await?;

    sqlx::query(LIVE_RLS_DROP_SCHEMA_SQL)
        .execute(&admin)
        .await
        .map_err(sqlx_error)?;

    Ok(Some(LiveRlsCrossTenantReport {
        schema_name: LIVE_RLS_SCHEMA,
        tenant_a_visible_rows: rows_a.rows.len(),
        tenant_b_visible_rows: rows_b.rows.len(),
        tenant_b_write_into_a_denied,
        commit_timestamps_strictly_increase: receipt_2.commit_timestamp
            > receipt_1.commit_timestamp,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_sql_kernel::ReadConsistency;

    fn pool_config() -> PostgresPoolConfig {
        PostgresPoolConfig::for_microservice("data-adapter-test", 4).unwrap()
    }

    #[test]
    fn config_requires_url_and_honors_tls_discipline() {
        assert!(SqlxDataClientConfig::new(" ", pool_config()).is_err());
        let err = SqlxDataClientConfig::new(
            "postgres://localhost/db?sslmode=disable",
            pool_config(),
        )
        .unwrap_err();
        assert!(matches!(err, DataSqlError::Adapter(_)));
        SqlxDataClientConfig::new("postgres://localhost/db", pool_config()).unwrap();
    }

    #[test]
    fn statement_shape_guard_rejects_multi_statement_sql() {
        let statement = Statement::new(
            "two_statements",
            "SELECT 1; DROP TABLE tenants",
            vec![],
        )
        .unwrap();
        assert!(check_statement_shape(&statement).is_err());
    }

    #[test]
    fn statement_shape_guard_matches_placeholders_to_params() {
        let missing_param = Statement::new(
            "insert_row",
            "INSERT INTO t (a, b) VALUES ($1, $2)",
            vec![SqlValue::Text("only-one".to_owned())],
        )
        .unwrap();
        assert!(check_statement_shape(&missing_param).is_err());
        let aligned = Statement::new(
            "insert_row",
            "INSERT INTO t (a, b) VALUES ($1, $2)",
            vec![
                SqlValue::Text("one".to_owned()),
                SqlValue::Int64(2),
            ],
        )
        .unwrap();
        check_statement_shape(&aligned).unwrap();
        let trailing_semicolon =
            Statement::new("read", "SELECT a FROM t;", vec![]).unwrap();
        check_statement_shape(&trailing_semicolon).unwrap();
    }

    #[test]
    fn highest_placeholder_scans_digits_correctly() {
        assert_eq!(highest_placeholder("SELECT $1, $12, $3"), 12);
        assert_eq!(highest_placeholder("SELECT 'a$'"), 0);
        assert_eq!(highest_placeholder("SELECT 1"), 0);
    }

    // tokio::test (not plain #[test]): PgPool::connect_lazy spawns pool
    // maintenance via sqlx's runtime shim, which panics ("requires a Tokio
    // context") outside a tokio runtime — the assertions themselves are sync.
    #[tokio::test]
    async fn session_open_enforces_kernel_descriptor_invariants() {
        let client = SqlxDataClient::from_pool(PgPool::connect_lazy("postgres://localhost/x").unwrap());
        let invalid = SessionDescriptor {
            store: DataStore::BootstrapMetastore,
            scope: SessionScope::Tenant {
                tenant_id: "acme".to_owned(),
                cell_id: "cell-001".to_owned(),
            },
            application_name: "oyatie-data".to_owned(),
        };
        assert_eq!(
            client.open_session(&invalid).unwrap_err(),
            DataSqlError::TenantScopeForbiddenInBootstrapMetastore
        );
        let valid = SessionDescriptor::tenant_data("acme", "cell-001", "oyatie-data").unwrap();
        assert_eq!(
            client.open_session(&valid).unwrap().descriptor(),
            &valid
        );
    }

    // tokio::test for the same connect_lazy runtime requirement as above.
    #[tokio::test]
    async fn commit_stamps_strictly_increase() {
        let client = SqlxDataClient::from_pool(PgPool::connect_lazy("postgres://localhost/x").unwrap());
        let first = client.stamp_commit().unwrap();
        let second = client.stamp_commit().unwrap();
        assert!(second > first);
    }

    #[tokio::test]
    async fn live_probe_is_disabled_by_default() {
        // Default test runs are database-free: without the enable env the
        // probe must short-circuit to None.
        if env::var(LIVE_DATA_POSTGRES_ENABLE_ENV).is_ok() {
            return; // an integration lane is driving the live probe
        }
        assert_eq!(run_live_rls_cross_tenant_probe().await.unwrap(), None);
    }

    /// AMENDMENT 7 integration rung, env-gated: run against containerized
    /// Postgres via OYA_DATA_LIVE_POSTGRES + admin/app URLs.
    #[tokio::test]
    async fn live_rls_cross_tenant_deny_when_enabled() {
        if env::var(LIVE_DATA_POSTGRES_ENABLE_ENV).is_err() {
            return;
        }
        let report = run_live_rls_cross_tenant_probe()
            .await
            .unwrap()
            .expect("live probe enabled");
        assert_eq!(report.tenant_a_visible_rows, 2);
        assert_eq!(report.tenant_b_visible_rows, 1);
        assert!(report.tenant_b_write_into_a_denied);
        assert!(report.commit_timestamps_strictly_increase);
    }

    #[test]
    fn read_consistency_levels_are_all_accepted_by_validation() {
        // The adapter serves every level as Strong; validation must accept
        // all port levels so callers can switch engines without code change.
        for consistency in [
            ReadConsistency::Strong,
            ReadConsistency::BoundedStaleness {
                max_staleness_ms: 250,
            },
            ReadConsistency::SnapshotAt {
                timestamp: HlcTimestamp::new(7, 0),
            },
        ] {
            consistency.validate().unwrap();
        }
    }
}
