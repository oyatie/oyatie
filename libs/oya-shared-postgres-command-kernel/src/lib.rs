//! Runtime-neutral Postgres command kernel for tenant-scoped adapters.
//!
//! The kernel models parameterized SQL commands and the tenant-session command
//! that must run before RLS-protected statements. Concrete adapters may bind
//! these commands through sqlx/tokio-postgres later; this crate deliberately
//! performs no database I/O.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const SET_LOCAL_TENANT_SQL: &str = "SELECT set_config('oyatie.tenant_id', $1, true)";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SqlCommandError {
    MissingTenantId,
    MissingHomeCell,
    MissingShardKey,
    MissingJurisdictionCode,
    MissingApplicationName,
    MissingCommandName,
    MissingSql,
    EmptyStatementSet,
    InvalidPoolConfig { field: &'static str },
    TenantScopeMustPrecedeStatements,
    MissingField { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SqlParam {
    Text(String),
    TextArray(Vec<String>),
    NullableText(Option<String>),
}

impl SqlParam {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn text_array(values: impl Into<Vec<String>>) -> Self {
        Self::TextArray(values.into())
    }

    pub fn nullable_text(value: Option<String>) -> Self {
        Self::NullableText(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantSqlContext {
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub home_cell: String,         // data_class: INTERNAL_ONLY
    pub shard_key: String,         // data_class: INTERNAL_ONLY
    pub jurisdiction_code: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlCommand {
    pub name: &'static str,    // data_class: INTERNAL_ONLY
    pub sql: &'static str,     // data_class: INTERNAL_ONLY
    pub params: Vec<SqlParam>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlWriteBatch {
    pub tenant_scope: SqlCommand,    // data_class: INTERNAL_ONLY
    pub statements: Vec<SqlCommand>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresPoolConfig {
    pub application_name: String,  // data_class: INTERNAL_ONLY
    pub max_connections: u32,      // data_class: INTERNAL_ONLY
    pub acquire_timeout_ms: u64,   // data_class: INTERNAL_ONLY
    pub statement_timeout_ms: u64, // data_class: INTERNAL_ONLY
    pub require_tls: bool,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlExecutionPlan {
    pub pool: PostgresPoolConfig,    // data_class: INTERNAL_ONLY
    pub tenant_scope: SqlCommand,    // data_class: INTERNAL_ONLY
    pub statements: Vec<SqlCommand>, // data_class: INTERNAL_ONLY
    pub total_command_count: usize,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlExecutionReport {
    pub application_name: String,            // data_class: INTERNAL_ONLY
    pub executed_command_names: Vec<String>, // data_class: INTERNAL_ONLY
    pub transaction_committed: bool,         // data_class: INTERNAL_ONLY
}

pub trait SqlBatchExecutor {
    fn execute_batch(
        &mut self,
        plan: &SqlExecutionPlan,
    ) -> Result<SqlExecutionReport, SqlCommandError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordingSqlBatchExecutor {
    pub reports: Vec<SqlExecutionReport>, // data_class: INTERNAL_ONLY
}

impl TenantSqlContext {
    pub fn new(
        tenant_id: impl Into<String>,
        home_cell: impl Into<String>,
        shard_key: impl Into<String>,
        jurisdiction_code: impl Into<String>,
    ) -> Result<Self, SqlCommandError> {
        let context = Self {
            tenant_id: tenant_id.into(),
            home_cell: home_cell.into(),
            shard_key: shard_key.into(),
            jurisdiction_code: jurisdiction_code.into(),
        };
        context.validate()?;
        Ok(context)
    }

    pub fn validate(&self) -> Result<(), SqlCommandError> {
        require_non_empty_named(&self.tenant_id, SqlCommandError::MissingTenantId)?;
        require_non_empty_named(&self.home_cell, SqlCommandError::MissingHomeCell)?;
        require_non_empty_named(&self.shard_key, SqlCommandError::MissingShardKey)?;
        require_non_empty_named(
            &self.jurisdiction_code,
            SqlCommandError::MissingJurisdictionCode,
        )?;
        Ok(())
    }

    pub fn tenant_scope_command(&self) -> Result<SqlCommand, SqlCommandError> {
        tenant_scope_command(self)
    }

    pub fn routing_params(&self) -> Result<Vec<SqlParam>, SqlCommandError> {
        self.validate()?;
        Ok(vec![
            SqlParam::text(self.tenant_id.clone()),
            SqlParam::text(self.home_cell.clone()),
            SqlParam::text(self.shard_key.clone()),
            SqlParam::text(self.jurisdiction_code.clone()),
        ])
    }
}

impl SqlCommand {
    pub fn new(
        name: &'static str,
        sql: &'static str,
        params: Vec<SqlParam>,
    ) -> Result<Self, SqlCommandError> {
        if name.trim().is_empty() {
            return Err(SqlCommandError::MissingCommandName);
        }
        if sql.trim().is_empty() {
            return Err(SqlCommandError::MissingSql);
        }
        Ok(Self { name, sql, params })
    }
}

impl SqlWriteBatch {
    pub fn new(
        context: &TenantSqlContext,
        statements: Vec<SqlCommand>,
    ) -> Result<Self, SqlCommandError> {
        context.validate()?;
        if statements.is_empty() {
            return Err(SqlCommandError::EmptyStatementSet);
        }
        Ok(Self {
            tenant_scope: tenant_scope_command(context)?,
            statements,
        })
    }
}

impl PostgresPoolConfig {
    pub fn new(
        application_name: impl Into<String>,
        max_connections: u32,
        acquire_timeout_ms: u64,
        statement_timeout_ms: u64,
        require_tls: bool,
    ) -> Result<Self, SqlCommandError> {
        let config = Self {
            application_name: application_name.into(),
            max_connections,
            acquire_timeout_ms,
            statement_timeout_ms,
            require_tls,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn for_microservice(
        microservice: impl Into<String>,
        max_connections: u32,
    ) -> Result<Self, SqlCommandError> {
        let microservice = microservice.into();
        Self::new(
            format!("oyatie-{microservice}"),
            max_connections,
            500,
            2_000,
            true,
        )
    }

    pub fn validate(&self) -> Result<(), SqlCommandError> {
        require_non_empty_named(
            &self.application_name,
            SqlCommandError::MissingApplicationName,
        )?;
        if self.max_connections == 0 {
            return Err(SqlCommandError::InvalidPoolConfig {
                field: "max_connections",
            });
        }
        if self.acquire_timeout_ms == 0 {
            return Err(SqlCommandError::InvalidPoolConfig {
                field: "acquire_timeout_ms",
            });
        }
        if self.statement_timeout_ms == 0 {
            return Err(SqlCommandError::InvalidPoolConfig {
                field: "statement_timeout_ms",
            });
        }
        Ok(())
    }
}

impl SqlExecutionPlan {
    pub fn from_batch(
        pool: PostgresPoolConfig,
        batch: SqlWriteBatch,
    ) -> Result<Self, SqlCommandError> {
        pool.validate()?;
        if batch.tenant_scope.name != "set_local_oyatie_tenant"
            || batch.tenant_scope.sql != SET_LOCAL_TENANT_SQL
        {
            return Err(SqlCommandError::TenantScopeMustPrecedeStatements);
        }
        if batch.statements.is_empty() {
            return Err(SqlCommandError::EmptyStatementSet);
        }
        let total_command_count = 1 + batch.statements.len();
        Ok(Self {
            pool,
            tenant_scope: batch.tenant_scope,
            statements: batch.statements,
            total_command_count,
        })
    }

    pub fn ordered_command_names(&self) -> Vec<String> {
        std::iter::once(self.tenant_scope.name.to_string())
            .chain(
                self.statements
                    .iter()
                    .map(|statement| statement.name.to_string()),
            )
            .collect()
    }
}

impl SqlBatchExecutor for RecordingSqlBatchExecutor {
    fn execute_batch(
        &mut self,
        plan: &SqlExecutionPlan,
    ) -> Result<SqlExecutionReport, SqlCommandError> {
        if plan.tenant_scope.name != "set_local_oyatie_tenant"
            || plan.tenant_scope.sql != SET_LOCAL_TENANT_SQL
        {
            return Err(SqlCommandError::TenantScopeMustPrecedeStatements);
        }
        let report = SqlExecutionReport {
            application_name: plan.pool.application_name.clone(),
            executed_command_names: plan.ordered_command_names(),
            transaction_committed: true,
        };
        self.reports.push(report.clone());
        Ok(report)
    }
}

pub fn tenant_scope_command(context: &TenantSqlContext) -> Result<SqlCommand, SqlCommandError> {
    context.validate()?;
    SqlCommand::new(
        "set_local_oyatie_tenant",
        SET_LOCAL_TENANT_SQL,
        vec![SqlParam::text(context.tenant_id.clone())],
    )
}

pub fn required_field(value: &str, field: &'static str) -> Result<String, SqlCommandError> {
    if value.trim().is_empty() {
        Err(SqlCommandError::MissingField { field })
    } else {
        Ok(value.to_string())
    }
}

pub fn optional_field(
    value: &Option<String>,
    field: &'static str,
) -> Result<Option<String>, SqlCommandError> {
    match value {
        Some(value) => required_field(value, field).map(Some),
        None => Ok(None),
    }
}

pub fn required_values<'a>(
    values: impl IntoIterator<Item = (&'static str, &'a str)>,
) -> Result<Vec<String>, SqlCommandError> {
    values
        .into_iter()
        .map(|(field, value)| required_field(value, field))
        .collect()
}

pub fn text_array_values(
    field: &'static str,
    values: &[String],
) -> Result<Vec<String>, SqlCommandError> {
    values
        .iter()
        .map(|value| required_field(value, field))
        .collect()
}

fn require_non_empty_named(value: &str, error: SqlCommandError) -> Result<(), SqlCommandError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> TenantSqlContext {
        TenantSqlContext::new("tenant:t", "cell-a", "tenant:t#cell-a", "US").unwrap()
    }

    #[test]
    fn tenant_scope_uses_parameterized_set_config() {
        let command = tenant_scope_command(&ctx()).unwrap();
        assert_eq!(
            command.sql,
            "SELECT set_config('oyatie.tenant_id', $1, true)"
        );
        assert_eq!(command.params, vec![SqlParam::text("tenant:t")]);
        assert!(!command.sql.contains("tenant:t"));
    }

    #[test]
    fn tenant_context_requires_all_routing_fields() {
        assert_eq!(
            TenantSqlContext::new("", "cell", "shard", "US"),
            Err(SqlCommandError::MissingTenantId)
        );
        assert_eq!(
            TenantSqlContext::new("tenant:t", "", "shard", "US"),
            Err(SqlCommandError::MissingHomeCell)
        );
        assert_eq!(
            TenantSqlContext::new("tenant:t", "cell", "", "US"),
            Err(SqlCommandError::MissingShardKey)
        );
        assert_eq!(
            TenantSqlContext::new("tenant:t", "cell", "shard", ""),
            Err(SqlCommandError::MissingJurisdictionCode)
        );
    }

    #[test]
    fn write_batch_refuses_empty_statement_set() {
        assert_eq!(
            SqlWriteBatch::new(&ctx(), vec![]),
            Err(SqlCommandError::EmptyStatementSet)
        );
    }

    #[test]
    fn command_refuses_empty_name_and_sql() {
        assert_eq!(
            SqlCommand::new("", "SELECT 1", vec![]),
            Err(SqlCommandError::MissingCommandName)
        );
        assert_eq!(
            SqlCommand::new("name", " ", vec![]),
            Err(SqlCommandError::MissingSql)
        );
    }

    #[test]
    fn optional_and_array_values_reject_blank_entries() {
        assert_eq!(
            optional_field(&Some(" ".into()), "workflow_consent_ref"),
            Err(SqlCommandError::MissingField {
                field: "workflow_consent_ref"
            })
        );
        assert_eq!(
            text_array_values("media_refs", &["media:1".into(), "".into()]),
            Err(SqlCommandError::MissingField {
                field: "media_refs"
            })
        );
    }

    #[test]
    fn pool_config_requires_bounded_runtime_settings() {
        assert_eq!(
            PostgresPoolConfig::new("", 4, 500, 2_000, true),
            Err(SqlCommandError::MissingApplicationName)
        );
        assert_eq!(
            PostgresPoolConfig::new("oyatie-messenger", 0, 500, 2_000, true),
            Err(SqlCommandError::InvalidPoolConfig {
                field: "max_connections"
            })
        );
        assert_eq!(
            PostgresPoolConfig::for_microservice("messenger", 8)
                .unwrap()
                .application_name,
            "oyatie-messenger"
        );
    }

    #[test]
    fn execution_plan_keeps_tenant_scope_before_statements() {
        let batch = SqlWriteBatch::new(
            &ctx(),
            vec![
                SqlCommand::new("insert_message", "INSERT INTO messages VALUES ($1)", vec![])
                    .unwrap(),
            ],
        )
        .unwrap();
        let plan = SqlExecutionPlan::from_batch(
            PostgresPoolConfig::for_microservice("messenger", 4).unwrap(),
            batch,
        )
        .unwrap();

        assert_eq!(plan.total_command_count, 2);
        assert_eq!(
            plan.ordered_command_names(),
            vec!["set_local_oyatie_tenant", "insert_message"]
        );
    }

    #[test]
    fn recording_executor_commits_ordered_execution_report() {
        let batch = SqlWriteBatch::new(
            &ctx(),
            vec![
                SqlCommand::new("insert_message", "INSERT INTO messages VALUES ($1)", vec![])
                    .unwrap(),
            ],
        )
        .unwrap();
        let plan = SqlExecutionPlan::from_batch(
            PostgresPoolConfig::for_microservice("messenger", 4).unwrap(),
            batch,
        )
        .unwrap();
        let mut executor = RecordingSqlBatchExecutor::default();
        let report = executor.execute_batch(&plan).unwrap();

        assert!(report.transaction_committed);
        assert_eq!(executor.reports, vec![report.clone()]);
        assert_eq!(
            report.executed_command_names,
            vec!["set_local_oyatie_tenant", "insert_message"]
        );
    }
}
