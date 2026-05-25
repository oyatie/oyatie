//! SQLx-backed Postgres command executor adapter.
//!
//! This adapter is the first live SQLx execution seam for the shared
//! tenant-scoped command kernel. It owns `sqlx::PgPool` usage, begins a
//! transaction, executes the tenant `set_config` command before statements, and
//! commits only after every command succeeds. Tests stay database-free; live
//! RLS/backup/Citus drills remain a later environment slice.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_shared_postgres_command_kernel::{
    PostgresPoolConfig, SET_LOCAL_TENANT_SQL, SqlCommand, SqlCommandError, SqlExecutionPlan,
    SqlExecutionReport, SqlParam,
};
use sqlx::{Executor, PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use std::time::Duration;

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

#[cfg(test)]
mod tests {
    use super::*;
    use oya_shared_postgres_command_kernel::{
        PostgresPoolConfig, SqlCommand, SqlExecutionPlan, SqlWriteBatch, TenantSqlContext,
    };

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
    fn highest_placeholder_tracks_postgres_bind_numbers() {
        assert_eq!(
            highest_placeholder("INSERT INTO t VALUES ($1, $2, $10)"),
            10
        );
        assert_eq!(highest_placeholder("INSERT INTO t VALUES ($1, $1)"), 1);
        assert_eq!(highest_placeholder("UPDATE t SET a = true"), 0);
    }
}
