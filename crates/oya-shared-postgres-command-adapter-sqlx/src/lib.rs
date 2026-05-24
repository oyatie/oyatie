//! SQLx-backed Postgres command executor adapter.
//!
//! This adapter is the first live SQLx execution seam for the shared
//! tenant-scoped command kernel. It owns `sqlx::PgPool` usage, begins a
//! transaction, executes the tenant `set_config` command before statements, and
//! commits only after every command succeeds. The default test suite stays
//! database-free; an explicit environment-gated live probe can exercise
//! PostgreSQL RLS and optional Citus distribution when a caller supplies a
//! disposable database URL.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_shared_postgres_command_kernel::{
    PostgresPoolConfig, SET_LOCAL_TENANT_SQL, SqlCommand, SqlCommandError, SqlExecutionPlan,
    SqlExecutionReport, SqlParam, SqlWriteBatch, TenantSqlContext,
};
use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use std::{env, time::Duration};

pub const LIVE_POSTGRES_ENABLE_ENV: &str = "OYA_BACKBONE_LIVE_POSTGRES";
pub const LIVE_POSTGRES_DATABASE_URL_ENV: &str = "OYA_BACKBONE_POSTGRES_URL";
pub const LIVE_POSTGRES_REQUIRE_TLS_ENV: &str = "OYA_BACKBONE_POSTGRES_REQUIRE_TLS";
pub const LIVE_POSTGRES_REQUIRE_CITUS_ENV: &str = "OYA_BACKBONE_REQUIRE_CITUS";
const LIVE_RLS_SCHEMA: &str = "oya_live_rls_probe";
const LIVE_RLS_TABLE: &str = "tenant_rows";
const LIVE_RLS_INSERT_SQL: &str =
    "INSERT INTO oya_live_rls_probe.tenant_rows (tenant_id, row_id, payload) VALUES ($1, $2, $3)";
const LIVE_RLS_SELECT_COUNT_SQL: &str =
    "SELECT count(*)::bigint FROM oya_live_rls_probe.tenant_rows";
const LIVE_RLS_DROP_SCHEMA_SQL: &str = "DROP SCHEMA IF EXISTS oya_live_rls_probe CASCADE";
const LIVE_RLS_CREATE_SCHEMA_SQL: &str = "CREATE SCHEMA oya_live_rls_probe";
const LIVE_RLS_CREATE_TABLE_SQL: &str = "CREATE TABLE oya_live_rls_probe.tenant_rows (tenant_id text NOT NULL, row_id text NOT NULL, payload text NOT NULL, PRIMARY KEY (tenant_id, row_id))";
const LIVE_RLS_ENABLE_SQL: &str =
    "ALTER TABLE oya_live_rls_probe.tenant_rows ENABLE ROW LEVEL SECURITY";
const LIVE_RLS_FORCE_SQL: &str =
    "ALTER TABLE oya_live_rls_probe.tenant_rows FORCE ROW LEVEL SECURITY";
const LIVE_RLS_CREATE_POLICY_SQL: &str = "CREATE POLICY tenant_isolation ON oya_live_rls_probe.tenant_rows USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true))";
const LIVE_RLS_CITUS_AVAILABLE_SQL: &str =
    "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'citus')";
const LIVE_RLS_DISTRIBUTE_SQL: &str = "SELECT create_distributed_table('oya_live_rls_probe.tenant_rows', 'tenant_id', colocate_with => 'none')";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SqlxPostgresCommandError {
    Kernel(SqlCommandError),
    MissingDatabaseUrl,
    TlsRequiredButDisabled,
    UnsafeSql {
        command_name: &'static str,
    },
    ParameterCountMismatch {
        command_name: &'static str,
        expected_highest_placeholder: usize,
        actual_params: usize,
    },
    Sqlx(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LivePostgresRlsHarnessError {
    Disabled {
        enable_env: &'static str,
    },
    MissingDatabaseUrl {
        database_url_env: &'static str,
    },
    InvalidBooleanEnv {
        env_name: &'static str,
        value: String, // data_class: INTERNAL_ONLY
    },
    Config(SqlxPostgresCommandError),
    Sqlx(String), // data_class: INTERNAL_ONLY
    CitusExtensionUnavailable,
    RlsIsolationFailed {
        tenant_scope_ref: String, // data_class: INTERNAL_ONLY
        visible_rows: i64,
    },
}

impl From<SqlCommandError> for SqlxPostgresCommandError {
    fn from(error: SqlCommandError) -> Self {
        Self::Kernel(error)
    }
}

impl From<sqlx::Error> for SqlxPostgresCommandError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error.to_string())
    }
}

impl From<SqlxPostgresCommandError> for LivePostgresRlsHarnessError {
    fn from(error: SqlxPostgresCommandError) -> Self {
        Self::Config(error)
    }
}

impl From<sqlx::Error> for LivePostgresRlsHarnessError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxPostgresConnectionConfig {
    pub database_url: String,     // data_class: INTERNAL_ONLY
    pub pool: PostgresPoolConfig, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlxExecutionPlanSummary {
    pub application_name: String,            // data_class: INTERNAL_ONLY
    pub total_command_count: usize,          // data_class: INTERNAL_ONLY
    pub executed_command_names: Vec<String>, // data_class: INTERNAL_ONLY
    pub require_tls: bool,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LivePostgresRlsHarnessConfig {
    pub database_url: String, // data_class: INTERNAL_ONLY
    pub require_tls: bool,    // data_class: INTERNAL_ONLY
    pub require_citus: bool,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LivePostgresRlsProbeReport {
    pub schema_name: &'static str, // data_class: INTERNAL_ONLY
    pub table_name: &'static str,  // data_class: INTERNAL_ONLY
    pub tenant_a_visible_count: i64,
    pub tenant_b_visible_count: i64,
    pub no_tenant_visible_count: i64,
    pub citus_distribution_checked: bool,
    pub rls_policy_checked: bool,
}

pub struct SqlxPostgresBatchExecutor {
    pool: PgPool,
}

impl SqlxPostgresConnectionConfig {
    pub fn new(
        database_url: impl Into<String>,
        pool: PostgresPoolConfig,
    ) -> Result<Self, SqlxPostgresCommandError> {
        let config = Self {
            database_url: database_url.into(),
            pool,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), SqlxPostgresCommandError> {
        self.pool.validate()?;
        if self.database_url.trim().is_empty() {
            return Err(SqlxPostgresCommandError::MissingDatabaseUrl);
        }
        if self.pool.require_tls && !requires_strict_tls(&self.database_url) {
            return Err(SqlxPostgresCommandError::TlsRequiredButDisabled);
        }
        Ok(())
    }
}

impl SqlxPostgresBatchExecutor {
    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(
        config: &SqlxPostgresConnectionConfig,
    ) -> Result<Self, SqlxPostgresCommandError> {
        config.validate()?;
        let pool = PgPoolOptions::new()
            .max_connections(config.pool.max_connections)
            .acquire_timeout(Duration::from_millis(config.pool.acquire_timeout_ms))
            .connect(&config.database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn execute_batch(
        &self,
        plan: &SqlExecutionPlan,
    ) -> Result<SqlExecutionReport, SqlxPostgresCommandError> {
        let summary = validate_plan_for_sqlx(plan)?;
        let mut transaction = self.pool.begin().await?;
        execute_command(&mut transaction, &plan.tenant_scope).await?;
        for statement in &plan.statements {
            execute_command(&mut transaction, statement).await?;
        }
        transaction.commit().await?;
        Ok(SqlExecutionReport {
            application_name: summary.application_name,
            executed_command_names: summary.executed_command_names,
            transaction_committed: true,
        })
    }
}

impl LivePostgresRlsHarnessConfig {
    pub fn from_env() -> Result<Self, LivePostgresRlsHarnessError> {
        Self::from_env_map(|name| env::var(name).ok())
    }

    pub fn from_env_map(
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, LivePostgresRlsHarnessError> {
        let enabled = lookup(LIVE_POSTGRES_ENABLE_ENV)
            .as_deref()
            .map(|value| parse_env_bool(LIVE_POSTGRES_ENABLE_ENV, value))
            .transpose()?
            .unwrap_or(false);
        if !enabled {
            return Err(LivePostgresRlsHarnessError::Disabled {
                enable_env: LIVE_POSTGRES_ENABLE_ENV,
            });
        }
        let database_url = lookup(LIVE_POSTGRES_DATABASE_URL_ENV).ok_or(
            LivePostgresRlsHarnessError::MissingDatabaseUrl {
                database_url_env: LIVE_POSTGRES_DATABASE_URL_ENV,
            },
        )?;
        let require_tls = lookup(LIVE_POSTGRES_REQUIRE_TLS_ENV)
            .as_deref()
            .map(|value| parse_env_bool(LIVE_POSTGRES_REQUIRE_TLS_ENV, value))
            .transpose()?
            .unwrap_or(true);
        let require_citus = lookup(LIVE_POSTGRES_REQUIRE_CITUS_ENV)
            .as_deref()
            .map(|value| parse_env_bool(LIVE_POSTGRES_REQUIRE_CITUS_ENV, value))
            .transpose()?
            .unwrap_or(false);
        let config = Self {
            database_url,
            require_tls,
            require_citus,
        };
        config.connection_config()?;
        Ok(config)
    }

    pub fn connection_config(
        &self,
    ) -> Result<SqlxPostgresConnectionConfig, SqlxPostgresCommandError> {
        SqlxPostgresConnectionConfig::new(
            self.database_url.clone(),
            live_probe_pool_config(self.require_tls)?,
        )
    }
}

pub async fn run_live_postgres_rls_probe(
    config: &LivePostgresRlsHarnessConfig,
) -> Result<LivePostgresRlsProbeReport, LivePostgresRlsHarnessError> {
    let connection = config.connection_config()?;
    let executor = SqlxPostgresBatchExecutor::connect(&connection).await?;
    let pool = executor.pool.clone();
    setup_live_rls_probe_schema(&pool, config.require_citus).await?;
    let report =
        run_live_rls_probe_in_schema(&executor, &pool, config.require_citus, config.require_tls)
            .await;
    let cleanup = cleanup_live_rls_probe_schema(&pool).await;
    match (report, cleanup) {
        (Ok(report), Ok(())) => Ok(report),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

pub fn validate_plan_for_sqlx(
    plan: &SqlExecutionPlan,
) -> Result<SqlxExecutionPlanSummary, SqlxPostgresCommandError> {
    plan.pool.validate()?;
    if plan.tenant_scope.name != "set_local_oyatie_tenant"
        || plan.tenant_scope.sql != SET_LOCAL_TENANT_SQL
    {
        return Err(SqlxPostgresCommandError::Kernel(
            SqlCommandError::TenantScopeMustPrecedeStatements,
        ));
    }
    validate_command_for_sqlx(&plan.tenant_scope, true)?;
    if plan.statements.is_empty() {
        return Err(SqlxPostgresCommandError::Kernel(
            SqlCommandError::EmptyStatementSet,
        ));
    }
    for statement in &plan.statements {
        validate_command_for_sqlx(statement, false)?;
    }
    let expected_total = 1 + plan.statements.len();
    if plan.total_command_count != expected_total {
        return Err(SqlxPostgresCommandError::Kernel(
            SqlCommandError::TenantScopeMustPrecedeStatements,
        ));
    }
    Ok(SqlxExecutionPlanSummary {
        application_name: plan.pool.application_name.clone(),
        total_command_count: expected_total,
        executed_command_names: plan.ordered_command_names(),
        require_tls: plan.pool.require_tls,
    })
}

fn validate_command_for_sqlx(
    command: &SqlCommand,
    tenant_scope: bool,
) -> Result<(), SqlxPostgresCommandError> {
    if !sql_shape_is_safe(command.sql, tenant_scope) {
        return Err(SqlxPostgresCommandError::UnsafeSql {
            command_name: command.name,
        });
    }
    let highest = highest_placeholder(command.sql);
    if highest != command.params.len() {
        return Err(SqlxPostgresCommandError::ParameterCountMismatch {
            command_name: command.name,
            expected_highest_placeholder: highest,
            actual_params: command.params.len(),
        });
    }
    Ok(())
}

async fn execute_command(
    transaction: &mut Transaction<'_, Postgres>,
    command: &SqlCommand,
) -> Result<(), SqlxPostgresCommandError> {
    let mut query = sqlx::query(command.sql);
    for param in &command.params {
        query = match param {
            SqlParam::Text(value) => query.bind(value.clone()),
            SqlParam::TextArray(values) => query.bind(values.clone()),
            SqlParam::NullableText(value) => query.bind(value.clone()),
        };
    }
    query.execute(&mut **transaction).await?;
    Ok(())
}

fn sql_shape_is_safe(sql: &str, tenant_scope: bool) -> bool {
    let trimmed = sql.trim();
    if trimmed.contains(';') || trimmed.is_empty() {
        return false;
    }
    if tenant_scope {
        return trimmed == SET_LOCAL_TENANT_SQL;
    }
    let upper = trimmed.to_ascii_uppercase();
    upper.starts_with("INSERT ") || upper.starts_with("UPDATE ") || upper.starts_with("DELETE ")
}

fn highest_placeholder(sql: &str) -> usize {
    let bytes = sql.as_bytes();
    let mut index = 0usize;
    let mut highest = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'$' {
            let mut cursor = index + 1;
            let mut value = 0usize;
            let mut saw_digit = false;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                saw_digit = true;
                value = value
                    .saturating_mul(10)
                    .saturating_add((bytes[cursor] - b'0') as usize);
                cursor += 1;
            }
            if saw_digit {
                highest = highest.max(value);
                index = cursor;
                continue;
            }
        }
        index += 1;
    }
    highest
}

fn requires_strict_tls(database_url: &str) -> bool {
    let Some((_, query)) = database_url.split_once('?') else {
        return false;
    };
    query
        .split('#')
        .next()
        .unwrap_or_default()
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .any(|(key, value)| {
            key.eq_ignore_ascii_case("sslmode")
                && matches!(
                    value.to_ascii_lowercase().as_str(),
                    "require" | "verify-ca" | "verify-full"
                )
        })
}

fn parse_env_bool(
    env_name: &'static str,
    value: &str,
) -> Result<bool, LivePostgresRlsHarnessError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(LivePostgresRlsHarnessError::InvalidBooleanEnv {
            env_name,
            value: value.to_string(),
        }),
    }
}

fn live_probe_pool_config(require_tls: bool) -> Result<PostgresPoolConfig, SqlCommandError> {
    PostgresPoolConfig::new("oyatie-live-rls-probe", 2, 1_000, 2_000, require_tls)
}

async fn setup_live_rls_probe_schema(
    pool: &PgPool,
    require_citus: bool,
) -> Result<(), LivePostgresRlsHarnessError> {
    cleanup_live_rls_probe_schema(pool).await?;
    for sql in [
        LIVE_RLS_CREATE_SCHEMA_SQL,
        LIVE_RLS_CREATE_TABLE_SQL,
        LIVE_RLS_ENABLE_SQL,
        LIVE_RLS_FORCE_SQL,
        LIVE_RLS_CREATE_POLICY_SQL,
    ] {
        sqlx::query(sql).execute(pool).await?;
    }
    let citus_available = citus_extension_available(pool).await?;
    if require_citus && !citus_available {
        return Err(LivePostgresRlsHarnessError::CitusExtensionUnavailable);
    }
    if require_citus {
        sqlx::query(LIVE_RLS_DISTRIBUTE_SQL).execute(pool).await?;
    }
    Ok(())
}

async fn cleanup_live_rls_probe_schema(pool: &PgPool) -> Result<(), LivePostgresRlsHarnessError> {
    sqlx::query(LIVE_RLS_DROP_SCHEMA_SQL).execute(pool).await?;
    Ok(())
}

async fn citus_extension_available(pool: &PgPool) -> Result<bool, LivePostgresRlsHarnessError> {
    let available = sqlx::query_scalar::<_, bool>(LIVE_RLS_CITUS_AVAILABLE_SQL)
        .fetch_one(pool)
        .await?;
    Ok(available)
}

async fn run_live_rls_probe_in_schema(
    executor: &SqlxPostgresBatchExecutor,
    pool: &PgPool,
    citus_distribution_checked: bool,
    require_tls: bool,
) -> Result<LivePostgresRlsProbeReport, LivePostgresRlsHarnessError> {
    executor
        .execute_batch(&live_probe_insert_plan(
            "tenant:live-a",
            "row:a",
            "payload:a",
            require_tls,
        )?)
        .await?;
    executor
        .execute_batch(&live_probe_insert_plan(
            "tenant:live-b",
            "row:b",
            "payload:b",
            require_tls,
        )?)
        .await?;

    let tenant_a_visible_count = visible_count_for_tenant(pool, Some("tenant:live-a")).await?;
    let tenant_b_visible_count = visible_count_for_tenant(pool, Some("tenant:live-b")).await?;
    let no_tenant_visible_count = visible_count_for_tenant(pool, None).await?;
    ensure_visible_count("tenant:live-a", tenant_a_visible_count, 1)?;
    ensure_visible_count("tenant:live-b", tenant_b_visible_count, 1)?;
    ensure_visible_count("tenant:none", no_tenant_visible_count, 0)?;

    Ok(LivePostgresRlsProbeReport {
        schema_name: LIVE_RLS_SCHEMA,
        table_name: LIVE_RLS_TABLE,
        tenant_a_visible_count,
        tenant_b_visible_count,
        no_tenant_visible_count,
        citus_distribution_checked,
        rls_policy_checked: true,
    })
}

fn live_probe_insert_plan(
    tenant_scope_ref: &str,
    row_id: &str,
    payload: &str,
    require_tls: bool,
) -> Result<SqlExecutionPlan, SqlxPostgresCommandError> {
    let tenant = TenantSqlContext::new(
        tenant_scope_ref,
        "cell-live",
        format!("{tenant_scope_ref}#cell-live"),
        "US",
    )?;
    let statement = SqlCommand::new(
        "insert_live_rls_probe",
        LIVE_RLS_INSERT_SQL,
        vec![
            SqlParam::text(tenant_scope_ref),
            SqlParam::text(row_id),
            SqlParam::text(payload),
        ],
    )?;
    let batch = SqlWriteBatch::new(&tenant, vec![statement])?;
    let pool = live_probe_pool_config(require_tls)?;
    Ok(SqlExecutionPlan::from_batch(pool, batch)?)
}

async fn visible_count_for_tenant(
    pool: &PgPool,
    tenant_scope_ref: Option<&str>,
) -> Result<i64, LivePostgresRlsHarnessError> {
    let mut transaction = pool.begin().await?;
    if let Some(tenant_scope_ref) = tenant_scope_ref {
        sqlx::query(SET_LOCAL_TENANT_SQL)
            .bind(tenant_scope_ref)
            .execute(&mut *transaction)
            .await?;
    }
    let count = sqlx::query_scalar::<_, i64>(LIVE_RLS_SELECT_COUNT_SQL)
        .fetch_one(&mut *transaction)
        .await?;
    transaction.rollback().await?;
    Ok(count)
}

fn ensure_visible_count(
    tenant_scope_ref: &str,
    visible_rows: i64,
    expected: i64,
) -> Result<(), LivePostgresRlsHarnessError> {
    if visible_rows == expected {
        Ok(())
    } else {
        Err(LivePostgresRlsHarnessError::RlsIsolationFailed {
            tenant_scope_ref: tenant_scope_ref.to_string(),
            visible_rows,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_shared_postgres_command_kernel::{
        PostgresPoolConfig, SqlCommand, SqlExecutionPlan, SqlWriteBatch, TenantSqlContext,
    };
    use std::collections::BTreeMap;

    fn plan() -> SqlExecutionPlan {
        let tenant = TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap();
        let statement = SqlCommand::new(
            "insert_probe",
            "INSERT INTO messenger_messages (tenant_id, message_id) VALUES ($1, $2)",
            vec![SqlParam::text("tenant:t"), SqlParam::text("message:m")],
        )
        .unwrap();
        let batch = SqlWriteBatch::new(&tenant, vec![statement]).unwrap();
        SqlExecutionPlan::from_batch(
            PostgresPoolConfig::for_microservice("messenger", 16).unwrap(),
            batch,
        )
        .unwrap()
    }

    fn env_lookup<'a>(entries: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        let map: BTreeMap<&str, &str> = entries.iter().copied().collect();
        move |name| map.get(name).map(|value| (*value).to_string())
    }

    #[test]
    fn validates_sqlx_execution_plan_summary() {
        let summary = validate_plan_for_sqlx(&plan()).unwrap();

        assert_eq!(summary.application_name, "oyatie-messenger");
        assert_eq!(summary.total_command_count, 2);
        assert_eq!(
            summary.executed_command_names,
            vec!["set_local_oyatie_tenant", "insert_probe"]
        );
        assert!(summary.require_tls);
    }

    #[test]
    fn rejects_unsafe_sql_and_parameter_count_mismatch() {
        let mut unsafe_plan = plan();
        unsafe_plan.statements[0] = SqlCommand::new(
            "select_probe",
            "SELECT * FROM messenger_messages WHERE tenant_id = $1",
            vec![SqlParam::text("tenant:t")],
        )
        .unwrap();
        assert_eq!(
            validate_plan_for_sqlx(&unsafe_plan),
            Err(SqlxPostgresCommandError::UnsafeSql {
                command_name: "select_probe"
            })
        );

        let mut mismatch_plan = plan();
        mismatch_plan.statements[0] = SqlCommand::new(
            "insert_probe",
            "INSERT INTO messenger_messages (tenant_id, message_id) VALUES ($1, $2)",
            vec![SqlParam::text("tenant:t")],
        )
        .unwrap();
        assert_eq!(
            validate_plan_for_sqlx(&mismatch_plan),
            Err(SqlxPostgresCommandError::ParameterCountMismatch {
                command_name: "insert_probe",
                expected_highest_placeholder: 2,
                actual_params: 1,
            })
        );
    }

    #[test]
    fn rejects_non_strict_tls_database_url_when_pool_requires_tls() {
        for database_url in [
            "postgres://user:pass@localhost/db",
            "postgres://user:pass@localhost/db?sslmode=disable",
            "postgres://user:pass@localhost/db?sslmode=allow",
            "postgres://user:pass@localhost/db?sslmode=prefer",
        ] {
            let config = SqlxPostgresConnectionConfig::new(
                database_url,
                PostgresPoolConfig::for_microservice("messenger", 16).unwrap(),
            );

            assert_eq!(
                config,
                Err(SqlxPostgresCommandError::TlsRequiredButDisabled)
            );
        }
    }

    #[test]
    fn accepts_strict_tls_database_url_shape() {
        for database_url in [
            "postgres://user:pass@localhost/db?sslmode=require",
            "postgres://user:pass@localhost/db?sslmode=verify-ca",
            "postgres://user:pass@localhost/db?sslmode=verify-full",
        ] {
            let config = SqlxPostgresConnectionConfig::new(
                database_url,
                PostgresPoolConfig::for_microservice("messenger", 16).unwrap(),
            )
            .unwrap();

            assert_eq!(config.pool.application_name, "oyatie-messenger");
        }
    }

    #[test]
    fn highest_placeholder_tracks_postgres_bind_numbers() {
        assert_eq!(
            highest_placeholder("INSERT INTO t VALUES ($1, $2, $10)"),
            10
        );
        assert_eq!(highest_placeholder("INSERT INTO t VALUES ($1, $1)"), 1);
        assert_eq!(highest_placeholder("UPDATE t SET a = true"), 0);
    }

    #[test]
    fn live_rls_harness_config_is_env_gated_and_validated() {
        assert_eq!(
            LivePostgresRlsHarnessConfig::from_env_map(env_lookup(&[])),
            Err(LivePostgresRlsHarnessError::Disabled {
                enable_env: LIVE_POSTGRES_ENABLE_ENV,
            })
        );
        assert_eq!(
            LivePostgresRlsHarnessConfig::from_env_map(env_lookup(&[(
                LIVE_POSTGRES_ENABLE_ENV,
                "true",
            )])),
            Err(LivePostgresRlsHarnessError::MissingDatabaseUrl {
                database_url_env: LIVE_POSTGRES_DATABASE_URL_ENV,
            })
        );

        let config = LivePostgresRlsHarnessConfig::from_env_map(env_lookup(&[
            (LIVE_POSTGRES_ENABLE_ENV, "yes"),
            (
                LIVE_POSTGRES_DATABASE_URL_ENV,
                "postgres://postgres:postgres@localhost/oyatie",
            ),
            (LIVE_POSTGRES_REQUIRE_TLS_ENV, "false"),
            (LIVE_POSTGRES_REQUIRE_CITUS_ENV, "on"),
        ]))
        .unwrap();

        assert!(!config.require_tls);
        assert!(config.require_citus);
        assert_eq!(config.connection_config().unwrap().pool.max_connections, 2);
    }

    #[test]
    fn live_rls_harness_rejects_invalid_boolean_env_values() {
        assert_eq!(
            LivePostgresRlsHarnessConfig::from_env_map(env_lookup(&[(
                LIVE_POSTGRES_ENABLE_ENV,
                "maybe",
            )])),
            Err(LivePostgresRlsHarnessError::InvalidBooleanEnv {
                env_name: LIVE_POSTGRES_ENABLE_ENV,
                value: "maybe".into(),
            })
        );
    }

    #[test]
    fn live_rls_probe_plan_sets_tenant_scope_before_insert() {
        let plan = live_probe_insert_plan("tenant:live-a", "row:a", "payload:a", false).unwrap();

        assert_eq!(plan.tenant_scope.sql, SET_LOCAL_TENANT_SQL);
        assert_eq!(plan.statements[0].sql, LIVE_RLS_INSERT_SQL);
        assert_eq!(
            plan.ordered_command_names(),
            vec!["set_local_oyatie_tenant", "insert_live_rls_probe"]
        );
    }

    #[tokio::test]
    async fn live_postgres_rls_probe_runs_when_enabled_by_environment() {
        let config = match LivePostgresRlsHarnessConfig::from_env() {
            Ok(config) => config,
            Err(LivePostgresRlsHarnessError::Disabled { .. }) => return,
            Err(error) => panic!("live Postgres harness was enabled but misconfigured: {error:?}"),
        };

        let report = run_live_postgres_rls_probe(&config)
            .await
            .expect("live Postgres RLS probe should pass when explicitly enabled");

        assert_eq!(report.schema_name, LIVE_RLS_SCHEMA);
        assert_eq!(report.table_name, LIVE_RLS_TABLE);
        assert_eq!(report.tenant_a_visible_count, 1);
        assert_eq!(report.tenant_b_visible_count, 1);
        assert_eq!(report.no_tenant_visible_count, 0);
        assert!(report.rls_policy_checked);
        assert_eq!(report.citus_distribution_checked, config.require_citus);
    }
}
