//! SQLx-backed Postgres command executor adapter.
//!
//! This adapter is the first live SQLx execution seam for the shared
//! tenant-scoped command kernel. It owns `sqlx::PgPool` usage, begins a
//! transaction, executes the tenant `set_config` command before statements, and
//! commits only after every command succeeds. The default test set stays
//! database-free; an explicit environment-gated live probe can exercise
//! PostgreSQL RLS and optional Citus distribution when a caller supplies a
//! disposable database URL.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_postgres_command_kernel::{
    PostgresPoolConfig, RLS_ROLE_PROBE_SQL, RLS_TABLE_FORCED_PROBE_SQL, RlsEnforceabilityError,
    SET_LOCAL_TENANT_SQL, SqlCommand, SqlCommandError, SqlExecutionPlan, SqlExecutionReport,
    SqlParam, SqlWriteBatch, TenantSqlContext, evaluate_rls_forced, evaluate_rls_role_flags,
};
use sqlx::{Executor, PgPool, Postgres, Row, Transaction, postgres::PgPoolOptions};
use std::{env, time::Duration};

pub const LIVE_POSTGRES_ENABLE_ENV: &str = "OYATIE_BACKBONE_LIVE_POSTGRES";
pub const LIVE_POSTGRES_DATABASE_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_URL";
pub const LIVE_POSTGRES_APP_DATABASE_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_APP_URL";
pub const LIVE_POSTGRES_REQUIRE_TLS_ENV: &str = "OYATIE_BACKBONE_POSTGRES_REQUIRE_TLS";
pub const LIVE_POSTGRES_REQUIRE_CITUS_ENV: &str = "OYATIE_BACKBONE_REQUIRE_CITUS";
const LIVE_RLS_SCHEMA: &str = "live_rls_probe";
const LIVE_RLS_TABLE: &str = "tenant_rows";
const LIVE_RLS_INSERT_SQL: &str =
    "INSERT INTO live_rls_probe.tenant_rows (tenant_id, row_id, payload) VALUES ($1, $2, $3)";
const LIVE_RLS_SELECT_COUNT_SQL: &str = "SELECT count(*)::bigint FROM live_rls_probe.tenant_rows";
const LIVE_RLS_DROP_SCHEMA_SQL: &str = "DROP SCHEMA IF EXISTS live_rls_probe CASCADE";
const LIVE_RLS_CREATE_SCHEMA_SQL: &str = "CREATE SCHEMA live_rls_probe";
const LIVE_RLS_CREATE_TABLE_SQL: &str = "CREATE TABLE live_rls_probe.tenant_rows (tenant_id text NOT NULL, row_id text NOT NULL, payload text NOT NULL, PRIMARY KEY (tenant_id, row_id))";
const LIVE_RLS_ENABLE_SQL: &str =
    "ALTER TABLE live_rls_probe.tenant_rows ENABLE ROW LEVEL SECURITY";
const LIVE_RLS_FORCE_SQL: &str = "ALTER TABLE live_rls_probe.tenant_rows FORCE ROW LEVEL SECURITY";
const LIVE_RLS_CREATE_POLICY_SQL: &str = "CREATE POLICY tenant_isolation ON live_rls_probe.tenant_rows USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true))";
const LIVE_RLS_CITUS_AVAILABLE_SQL: &str =
    "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'citus')";
const LIVE_RLS_DISTRIBUTE_SQL: &str = "SELECT create_distributed_table('live_rls_probe.tenant_rows', 'tenant_id', colocate_with => 'none')";

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LivePostgresRlsHarnessError {
    Disabled {
        enable_env: &'static str,
    },
    MissingDatabaseUrl {
        database_url_env: &'static str,
    },
    MissingAppDatabaseUrl {
        app_database_url_env: &'static str,
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
    AppRoleBypassesRls {
        role_name: String, // data_class: INTERNAL_ONLY
        rolsuper: bool,
        rolbypassrls: bool,
    },
    AppRoleMatchesSetupRole {
        role_name: String, // data_class: INTERNAL_ONLY
    },
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
    pub database_url: String,     // data_class: INTERNAL_ONLY
    pub app_database_url: String, // data_class: INTERNAL_ONLY
    pub require_tls: bool,        // data_class: INTERNAL_ONLY
    pub require_citus: bool,      // data_class: INTERNAL_ONLY
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
    pub app_role_checked: bool,
    pub app_role_name: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LivePostgresRoleInfo {
    name: String, // data_class: INTERNAL_ONLY
    rolsuper: bool,
    rolbypassrls: bool,
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
        if self.pool.require_tls && disables_tls(&self.database_url) {
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
        let app_database_url = lookup(LIVE_POSTGRES_APP_DATABASE_URL_ENV).ok_or(
            LivePostgresRlsHarnessError::MissingAppDatabaseUrl {
                app_database_url_env: LIVE_POSTGRES_APP_DATABASE_URL_ENV,
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
            app_database_url,
            require_tls,
            require_citus,
        };
        config.connection_config()?;
        config.app_connection_config()?;
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

    pub fn app_connection_config(
        &self,
    ) -> Result<SqlxPostgresConnectionConfig, SqlxPostgresCommandError> {
        SqlxPostgresConnectionConfig::new(
            self.app_database_url.clone(),
            live_probe_pool_config(self.require_tls)?,
        )
    }
}

pub async fn run_live_postgres_rls_probe(
    config: &LivePostgresRlsHarnessConfig,
) -> Result<LivePostgresRlsProbeReport, LivePostgresRlsHarnessError> {
    let setup_connection = config.connection_config()?;
    let app_connection = config.app_connection_config()?;
    let setup_executor = SqlxPostgresBatchExecutor::connect(&setup_connection).await?;
    let app_executor = SqlxPostgresBatchExecutor::connect(&app_connection).await?;
    let setup_pool = setup_executor.pool.clone();
    let app_pool = app_executor.pool.clone();
    let setup_role = current_database_role(&setup_pool).await?;
    let app_role = validate_live_rls_app_role(&app_pool).await?;
    ensure_distinct_live_rls_roles(&setup_role, &app_role)?;

    if let Err(error) = setup_live_rls_probe_schema(&setup_pool, config.require_citus).await {
        let _cleanup_error = cleanup_live_rls_probe_schema(&setup_pool).await.err();
        return Err(error);
    }
    if let Err(error) = grant_live_rls_probe_privileges(&setup_pool, &app_role.name).await {
        let _cleanup_error = cleanup_live_rls_probe_schema(&setup_pool).await.err();
        return Err(error);
    }

    let report = run_live_rls_probe_in_schema(
        &app_executor,
        &app_pool,
        config.require_citus,
        config.require_tls,
        app_role.name,
    )
    .await;
    let cleanup = cleanup_live_rls_probe_schema(&setup_pool).await;
    match (report, cleanup) {
        (Ok(report), Ok(())) => Ok(report),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

/// Fail-closed boot guard for a multi-tenant Postgres store. Run AFTER connect
/// but BEFORE serving, so a mis-provisioned database/role fails the service at
/// boot rather than leaking at runtime. It asserts:
///
/// 1. the connected `current_user` cannot bypass RLS (no `rolsuper` /
///    `rolbypassrls`) AND is a USAGE-member of `runtime_role` (the RLS
///    policy-subject role), via [`RLS_ROLE_PROBE_SQL`] +
///    [`evaluate_rls_role_flags`];
/// 2. `session_user == current_user` (no `SET ROLE` switch — these adapters
///    never issue one, so a mismatch means the wrong role would be checked);
/// 3. every governed table has BOTH `ENABLE` and `FORCE ROW LEVEL SECURITY`,
///    via [`RLS_TABLE_FORCED_PROBE_SQL`] + [`evaluate_rls_forced`].
///
/// `governed_tables` are SCHEMA-qualified (`"schema.table"`); each is split on
/// the FIRST `.` into `nspname` + `relname`. A table absent from `pg_class`
/// (migration not applied / name drift) yields
/// [`RlsEnforceabilityError::GovernedTableMissing`].
///
/// This is the single shared SSOT behind the two adapters' ergonomic delegates
/// (the tenancy store method and the SCIM free function). ADR-0083 Tier-3: no
/// unwrap/expect/panic.
///
/// # Errors
/// Any [`RlsEnforceabilityError`]; an underlying sqlx failure (probe query or
/// column decode) is surfaced fail-closed as
/// [`RlsEnforceabilityError::Unenforceable`] carrying the runtime role and the
/// sqlx detail, so the caller never proceeds on a probe it could not complete.
pub async fn assert_rls_enforceable(
    pool: &PgPool,
    runtime_role: &str,
    governed_tables: &[&str],
) -> Result<(), RlsEnforceabilityError> {
    let row = sqlx::query(RLS_ROLE_PROBE_SQL)
        .bind(runtime_role)
        .fetch_one(pool)
        .await
        .map_err(rls_probe_failed)?;
    let role_name: String = row.try_get("role_name").map_err(rls_probe_failed)?;
    let session_role: String = row.try_get("session_role").map_err(rls_probe_failed)?;
    let rolsuper: bool = row.try_get("rolsuper").map_err(rls_probe_failed)?;
    let rolbypassrls: bool = row.try_get("rolbypassrls").map_err(rls_probe_failed)?;
    let is_runtime_member: bool = row.try_get("is_runtime_member").map_err(rls_probe_failed)?;

    // Defense-in-depth: these adapters never issue SET ROLE, so the pool's
    // current_user must always equal session_user. Detect role-switch confusion
    // early rather than checking the wrong role.
    if session_role != role_name {
        return Err(RlsEnforceabilityError::RoleSwitchInEffect {
            session_role,
            current_role: role_name,
        });
    }
    evaluate_rls_role_flags(
        &role_name,
        runtime_role,
        rolsuper,
        rolbypassrls,
        is_runtime_member,
    )?;

    for qualified_table in governed_tables {
        let (schema, table) = split_qualified_table(qualified_table);
        let probe = sqlx::query(RLS_TABLE_FORCED_PROBE_SQL)
            .bind(schema)
            .bind(table)
            .fetch_optional(pool)
            .await
            .map_err(rls_probe_failed)?;
        let Some(probe) = probe else {
            return Err(RlsEnforceabilityError::GovernedTableMissing {
                table: (*qualified_table).to_owned(),
            });
        };
        let row_security: bool = probe.try_get("row_security").map_err(rls_probe_failed)?;
        let force_row_security: bool = probe
            .try_get("force_row_security")
            .map_err(rls_probe_failed)?;
        evaluate_rls_forced(qualified_table, row_security, force_row_security)?;
    }
    Ok(())
}

/// Map an sqlx probe failure (query or column decode) to a fail-closed
/// [`RlsEnforceabilityError::ProbeFailed`]. The guard must never proceed on a
/// probe it could not complete; each adapter maps `ProbeFailed` back to its own
/// `Sqlx(detail)` surface, preserving the same fatal `Sqlx` variant
/// (refuse-to-serve unchanged); the detail string carries a fail-closed prefix
/// sourced from the shared kernel Display, not the prior adapter-local wording
/// (e.g. when the runtime role does not yet exist in the database).
fn rls_probe_failed(error: sqlx::Error) -> RlsEnforceabilityError {
    RlsEnforceabilityError::ProbeFailed {
        detail: error.to_string(),
    }
}

/// Split a schema-qualified table name on the FIRST `.` into `(nspname,
/// relname)`. A name without a `.` is treated as residing in the unspecified
/// (empty) schema, which the probe will report as missing — fail-closed.
fn split_qualified_table(qualified_table: &str) -> (&str, &str) {
    match qualified_table.split_once('.') {
        Some((schema, table)) => (schema, table),
        None => ("", qualified_table),
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

fn disables_tls(database_url: &str) -> bool {
    let lower = database_url.to_ascii_lowercase();
    lower.contains("sslmode=disable") || lower.contains("sslmode=prefer")
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

async fn current_database_role(
    pool: &PgPool,
) -> Result<LivePostgresRoleInfo, LivePostgresRlsHarnessError> {
    let (name, rolsuper, rolbypassrls) = sqlx::query_as::<_, (String, bool, bool)>(
        "SELECT current_user::text, rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(pool)
    .await?;
    Ok(LivePostgresRoleInfo {
        name,
        rolsuper,
        rolbypassrls,
    })
}

async fn validate_live_rls_app_role(
    pool: &PgPool,
) -> Result<LivePostgresRoleInfo, LivePostgresRlsHarnessError> {
    let role = current_database_role(pool).await?;
    ensure_live_rls_app_role(role)
}

fn ensure_live_rls_app_role(
    role: LivePostgresRoleInfo,
) -> Result<LivePostgresRoleInfo, LivePostgresRlsHarnessError> {
    if role.rolsuper || role.rolbypassrls {
        return Err(LivePostgresRlsHarnessError::AppRoleBypassesRls {
            role_name: role.name,
            rolsuper: role.rolsuper,
            rolbypassrls: role.rolbypassrls,
        });
    }
    Ok(role)
}

fn ensure_distinct_live_rls_roles(
    setup_role: &LivePostgresRoleInfo,
    app_role: &LivePostgresRoleInfo,
) -> Result<(), LivePostgresRlsHarnessError> {
    if setup_role.name == app_role.name {
        return Err(LivePostgresRlsHarnessError::AppRoleMatchesSetupRole {
            role_name: app_role.name.clone(),
        });
    }
    Ok(())
}

async fn grant_live_rls_probe_privileges(
    pool: &PgPool,
    app_role_name: &str,
) -> Result<(), LivePostgresRlsHarnessError> {
    let schema = quote_identifier(LIVE_RLS_SCHEMA);
    let table = quote_identifier(LIVE_RLS_TABLE);
    let app_role = quote_identifier(app_role_name);
    for sql in [
        format!("GRANT USAGE ON SCHEMA {schema} TO {app_role}"),
        format!("GRANT SELECT, INSERT ON TABLE {schema}.{table} TO {app_role}"),
    ] {
        sqlx::query(&sql).execute(pool).await?;
    }
    Ok(())
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

async fn run_live_rls_probe_in_schema(
    executor: &SqlxPostgresBatchExecutor,
    pool: &PgPool,
    citus_distribution_checked: bool,
    require_tls: bool,
    app_role_name: String,
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
        app_role_checked: true,
        app_role_name,
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
    use shared_postgres_command_kernel::{
        PostgresPoolConfig, SqlCommand, SqlExecutionPlan, SqlWriteBatch, TenantSqlContext,
    };
    use std::collections::BTreeMap;

    fn env_lookup<'a>(entries: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        let map: BTreeMap<&str, &str> = entries.iter().copied().collect();
        move |name| map.get(name).map(|value| (*value).to_string())
    }

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
    fn rejects_tls_disabled_database_url_when_pool_requires_tls() {
        let config = SqlxPostgresConnectionConfig::new(
            "postgres://user:pass@localhost/db?sslmode=disable",
            PostgresPoolConfig::for_microservice("messenger", 16).unwrap(),
        );

        assert_eq!(
            config,
            Err(SqlxPostgresCommandError::TlsRequiredButDisabled)
        );
    }

    #[test]
    fn accepts_tls_required_database_url_shape() {
        let config = SqlxPostgresConnectionConfig::new(
            "postgres://user:pass@localhost/db?sslmode=require",
            PostgresPoolConfig::for_microservice("messenger", 16).unwrap(),
        )
        .unwrap();

        assert_eq!(config.pool.application_name, "oyatie-messenger");
    }

    #[test]
    fn split_qualified_table_uses_first_dot_for_schema_and_table() {
        assert_eq!(
            split_qualified_table("tenancy_lifecycle.tenancy_lifecycle_tenants"),
            ("tenancy_lifecycle", "tenancy_lifecycle_tenants")
        );
        assert_eq!(split_qualified_table("schema.a.b"), ("schema", "a.b"));
        // A name with no schema qualifier is treated as the empty schema, which
        // the probe reports as missing — fail-closed.
        assert_eq!(split_qualified_table("bare_table"), ("", "bare_table"));
    }

    #[test]
    fn rls_probe_failure_is_fail_closed_probe_failed() {
        let mapped = rls_probe_failed(sqlx::Error::RowNotFound);
        assert!(matches!(mapped, RlsEnforceabilityError::ProbeFailed { .. }));
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
        assert_eq!(
            LivePostgresRlsHarnessConfig::from_env_map(env_lookup(&[
                (LIVE_POSTGRES_ENABLE_ENV, "true"),
                (
                    LIVE_POSTGRES_DATABASE_URL_ENV,
                    "postgres://postgres:postgres@localhost/oyatie",
                ),
            ])),
            Err(LivePostgresRlsHarnessError::MissingAppDatabaseUrl {
                app_database_url_env: LIVE_POSTGRES_APP_DATABASE_URL_ENV,
            })
        );

        let config = LivePostgresRlsHarnessConfig::from_env_map(env_lookup(&[
            (LIVE_POSTGRES_ENABLE_ENV, "yes"),
            (
                LIVE_POSTGRES_DATABASE_URL_ENV,
                "postgres://postgres:postgres@localhost/oyatie",
            ),
            (
                LIVE_POSTGRES_APP_DATABASE_URL_ENV,
                "postgres://oyatie_app:postgres@localhost/oyatie",
            ),
            (LIVE_POSTGRES_REQUIRE_TLS_ENV, "false"),
            (LIVE_POSTGRES_REQUIRE_CITUS_ENV, "on"),
        ]))
        .unwrap();

        assert_eq!(
            config.app_database_url,
            "postgres://oyatie_app:postgres@localhost/oyatie"
        );
        assert!(!config.require_tls);
        assert!(config.require_citus);
        assert_eq!(config.connection_config().unwrap().pool.max_connections, 2);
        assert_eq!(
            config
                .app_connection_config()
                .unwrap()
                .pool
                .application_name,
            "oyatie-live-rls-probe"
        );
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

    #[test]
    fn quote_identifier_escapes_embedded_quotes_for_role_grants() {
        assert_eq!(quote_identifier("oyatie_app"), "\"oyatie_app\"");
        assert_eq!(quote_identifier("tenant\"runtime"), "\"tenant\"\"runtime\"");
    }

    #[test]
    fn rejects_live_rls_app_roles_that_can_bypass_rls() {
        let normal_role = LivePostgresRoleInfo {
            name: "oyatie_app".into(),
            rolsuper: false,
            rolbypassrls: false,
        };
        assert_eq!(
            ensure_live_rls_app_role(normal_role.clone()),
            Ok(normal_role)
        );

        assert_eq!(
            ensure_live_rls_app_role(LivePostgresRoleInfo {
                name: "postgres".into(),
                rolsuper: true,
                rolbypassrls: false,
            }),
            Err(LivePostgresRlsHarnessError::AppRoleBypassesRls {
                role_name: "postgres".into(),
                rolsuper: true,
                rolbypassrls: false,
            })
        );
        assert_eq!(
            ensure_live_rls_app_role(LivePostgresRoleInfo {
                name: "rls_break_glass".into(),
                rolsuper: false,
                rolbypassrls: true,
            }),
            Err(LivePostgresRlsHarnessError::AppRoleBypassesRls {
                role_name: "rls_break_glass".into(),
                rolsuper: false,
                rolbypassrls: true,
            })
        );
    }

    #[test]
    fn rejects_live_rls_app_role_that_matches_setup_role() {
        let setup_role = LivePostgresRoleInfo {
            name: "oyatie_setup".into(),
            rolsuper: false,
            rolbypassrls: false,
        };
        let app_role = LivePostgresRoleInfo {
            name: "oyatie_app".into(),
            rolsuper: false,
            rolbypassrls: false,
        };
        assert_eq!(
            ensure_distinct_live_rls_roles(&setup_role, &app_role),
            Ok(())
        );

        assert_eq!(
            ensure_distinct_live_rls_roles(&setup_role, &setup_role),
            Err(LivePostgresRlsHarnessError::AppRoleMatchesSetupRole {
                role_name: "oyatie_setup".into(),
            })
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
        assert!(report.app_role_checked);
        assert!(!report.app_role_name.trim().is_empty());
        assert_eq!(report.citus_distribution_checked, config.require_citus);
    }
}
