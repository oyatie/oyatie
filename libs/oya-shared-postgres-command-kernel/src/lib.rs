//! Runtime-neutral Postgres command kernel for tenant-scoped adapters.
//!
//! The kernel models parameterized SQL commands and the tenant-session command
//! that must run before RLS-protected statements. Concrete adapters may bind
//! these commands through sqlx/tokio-postgres later; this crate deliberately
//! performs no database I/O.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const SET_LOCAL_TENANT_SQL: &str = "SELECT set_config('oyatie.tenant_id', $1, true)";

/// Probe the connected role's RLS-relevant flags + membership of the runtime
/// (RLS policy-subject) role. `$1` binds the runtime role name.
///
/// `pg_has_role(user, role, 'USAGE')` follows Postgres's own
/// `has_privs_of_role` predicate: it is true iff `user` IS `role` or inherits
/// its privileges (transitive membership, respecting INHERIT). This is exactly
/// the predicate Postgres uses internally to decide whether a `TO <role>` RLS
/// policy clause applies to the current user. `'USAGE'` (NOT `'MEMBER'`) is the
/// correct keyword: `'MEMBER'` tests bare set-membership and returns true even
/// for a NOINHERIT member, which would silently get deny-all in practice; only
/// `'USAGE'` answers "this role's policies apply to me". (See the Postgres
/// `pg_has_role` docs: `USAGE` = `has_privs_of_role`, `MEMBER` =
/// `is_member_of_role`.)
///
/// `session_user` is fetched so the executor can detect a `SET ROLE` switch:
/// these adapters never issue `SET ROLE`, so `session_user` must equal
/// `current_user` on every serving connection.
pub const RLS_ROLE_PROBE_SQL: &str = "SELECT current_user::text AS role_name, \
     session_user::text AS session_role, \
     rolsuper, rolbypassrls, \
     pg_has_role(current_user, $1, 'USAGE') AS is_runtime_member \
     FROM pg_roles WHERE rolname = current_user";

/// Probe whether a governed table has ROW LEVEL SECURITY enabled AND forced.
/// `$1` binds the schema name (`nspname`), `$2` binds the table name
/// (`relname`). Returns no row when the table does not exist.
pub const RLS_TABLE_FORCED_PROBE_SQL: &str = "SELECT c.relrowsecurity AS row_security, \
     c.relforcerowsecurity AS force_row_security \
     FROM pg_class c JOIN pg_namespace n ON c.relnamespace = n.oid \
     WHERE n.nspname = $1 AND c.relname = $2";

/// A boot-time RLS-enforceability failure surfaced by the shared guard. Pure
/// (no sqlx); the `&PgPool` executor lives in the sqlx adapter and maps these
/// into each adapter's own connect-error vocabulary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RlsEnforceabilityError {
    /// The connected role carries `rolsuper` or `rolbypassrls`: either flag
    /// silently skips Postgres RLS, turning tenant isolation into a no-op.
    Unenforceable { role: String }, // data_class: INTERNAL_ONLY
    /// The connected role is neither the policy-subject role nor a USAGE-member
    /// of it: a `TO <role>` policy would not apply, yielding deny-all under
    /// FORCE RLS (a safe outage, but still a misconfiguration).
    RoleMismatch {
        role: String,     // data_class: INTERNAL_ONLY
        expected: String, // data_class: INTERNAL_ONLY
    },
    /// `session_user` differs from `current_user`: a `SET ROLE` switch is in
    /// effect and the guard cannot safely check the effective role.
    RoleSwitchInEffect {
        session_role: String, // data_class: INTERNAL_ONLY
        current_role: String, // data_class: INTERNAL_ONLY
    },
    /// A governed table does not have BOTH `ENABLE` and `FORCE ROW LEVEL
    /// SECURITY`, so the table owner (or this role) could read rows without the
    /// tenant-isolation policies applying.
    RlsNotForced {
        table: String, // data_class: INTERNAL_ONLY
        row_security: bool,
        force_row_security: bool,
    },
    /// A governed table is absent from `pg_class` (the migration has not run, or
    /// the schema/table name drifted): the guard refuses to serve a store whose
    /// isolation it cannot verify.
    GovernedTableMissing { table: String }, // data_class: INTERNAL_ONLY
    /// The probe query (or a column decode) failed before the guard could reach
    /// a decision — e.g. the runtime role does not yet exist in the database.
    /// Fail-closed: the caller must NOT proceed. Carries the underlying sqlx
    /// detail so adapters can preserve their existing `Sqlx(detail)` surface.
    ProbeFailed { detail: String }, // data_class: INTERNAL_ONLY
}

impl core::fmt::Display for RlsEnforceabilityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unenforceable { role } => write!(
                f,
                "runtime role '{role}' can bypass RLS (rolsuper/rolbypassrls); \
                 refusing to serve a multi-tenant store"
            ),
            Self::RoleMismatch { role, expected } => write!(
                f,
                "connected role '{role}' is not a USAGE-member of the RLS \
                 policy-subject role '{expected}'; tenant isolation policies \
                 would not apply — refusing to serve"
            ),
            Self::RoleSwitchInEffect {
                session_role,
                current_role,
            } => write!(
                f,
                "session_user '{session_role}' != current_user '{current_role}': \
                 a SET ROLE switch is in effect; this adapter does not issue \
                 SET ROLE and cannot safely check the effective role"
            ),
            Self::RlsNotForced {
                table,
                row_security,
                force_row_security,
            } => write!(
                f,
                "governed table '{table}' is not under ENABLE+FORCE ROW LEVEL \
                 SECURITY (row_security={row_security}, \
                 force_row_security={force_row_security}); refusing to serve a \
                 multi-tenant store whose isolation is not forced"
            ),
            Self::GovernedTableMissing { table } => write!(
                f,
                "governed table '{table}' is missing from pg_class (migration \
                 not applied or name drift); refusing to serve"
            ),
            Self::ProbeFailed { detail } => write!(
                f,
                "RLS-enforceability probe failed before a decision: {detail}"
            ),
        }
    }
}

impl std::error::Error for RlsEnforceabilityError {}

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

/// Pure RLS role-flag decision — no DB, no async, fully unit-testable.
///
/// Returns `Ok(())` iff the role is safe to serve multi-tenant traffic:
/// - not a superuser (`rolsuper = false`),
/// - not bypass-RLS capable (`rolbypassrls = false`), AND
/// - is a USAGE-member of the policy-subject role (`is_runtime_member = true`).
///
/// The three conditions map to two distinct error variants so callers and logs
/// can distinguish a bypass risk (leak vector) from a membership gap (outage):
/// - `rolsuper || rolbypassrls` → [`RlsEnforceabilityError::Unenforceable`]
///   (bypass-capable role; the isolation guarantee is violated).
/// - `!is_runtime_member` → [`RlsEnforceabilityError::RoleMismatch`]
///   (non-member; under FORCE RLS the policies simply would not apply, yielding
///   deny-all — a safe fail-closed outage, but still a misconfiguration).
pub fn evaluate_rls_role_flags(
    role: &str,
    expected_role: &str,
    rolsuper: bool,
    rolbypassrls: bool,
    is_runtime_member: bool,
) -> Result<(), RlsEnforceabilityError> {
    if rolsuper || rolbypassrls {
        return Err(RlsEnforceabilityError::Unenforceable {
            role: role.to_owned(),
        });
    }
    if !is_runtime_member {
        return Err(RlsEnforceabilityError::RoleMismatch {
            role: role.to_owned(),
            expected: expected_role.to_owned(),
        });
    }
    Ok(())
}

/// Pure FORCE-RLS decision for one governed table — no DB, no async.
///
/// Returns `Ok(())` iff the table has BOTH `relrowsecurity` (ENABLE ROW LEVEL
/// SECURITY) and `relforcerowsecurity` (FORCE ROW LEVEL SECURITY). FORCE is
/// required so the policies apply even to the table owner; without it a
/// privileged owner role would read every row. A table missing either flag
/// yields [`RlsEnforceabilityError::RlsNotForced`].
pub fn evaluate_rls_forced(
    qualified_table: &str,
    row_security: bool,
    force_row_security: bool,
) -> Result<(), RlsEnforceabilityError> {
    if row_security && force_row_security {
        Ok(())
    } else {
        Err(RlsEnforceabilityError::RlsNotForced {
            table: qualified_table.to_owned(),
            row_security,
            force_row_security,
        })
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

    // --- RLS-enforceability probe SQL shape ----------------------------------

    #[test]
    fn rls_role_probe_uses_usage_keyword_not_member() {
        // The 'MEMBER'->'USAGE' correctness fix: RLS policy applicability is the
        // has_privs_of_role (USAGE) question, not bare is_member_of_role (MEMBER).
        assert!(
            RLS_ROLE_PROBE_SQL.contains("pg_has_role(current_user, $1, 'USAGE')"),
            "role probe must use the 'USAGE' (has_privs_of_role) keyword"
        );
        assert!(
            !RLS_ROLE_PROBE_SQL.contains("'MEMBER'"),
            "role probe must NOT use the 'MEMBER' (is_member_of_role) keyword"
        );
        assert!(RLS_ROLE_PROBE_SQL.contains("session_user"));
        assert!(RLS_ROLE_PROBE_SQL.contains("rolsuper"));
        assert!(RLS_ROLE_PROBE_SQL.contains("rolbypassrls"));
    }

    #[test]
    fn rls_table_forced_probe_reads_relrowsecurity_and_force() {
        assert!(RLS_TABLE_FORCED_PROBE_SQL.contains("c.relrowsecurity"));
        assert!(RLS_TABLE_FORCED_PROBE_SQL.contains("c.relforcerowsecurity"));
        assert!(RLS_TABLE_FORCED_PROBE_SQL.contains("n.nspname = $1"));
        assert!(RLS_TABLE_FORCED_PROBE_SQL.contains("c.relname = $2"));
    }

    // --- DB-free evaluate_rls_role_flags predicate tests ---------------------
    // These cover all meaningful combinations of (rolsuper, rolbypassrls,
    // is_runtime_member) without a database. Only the SQL wiring into pg_roles /
    // pg_has_role requires a live DB; the boolean DECISION is always tested here.

    const EXPECTED_ROLE: &str = "tenancy_lifecycle_runtime";

    #[test]
    fn rls_role_flags_superuser_is_rejected_as_unenforceable() {
        assert_eq!(
            evaluate_rls_role_flags("pg_superuser", EXPECTED_ROLE, true, false, true),
            Err(RlsEnforceabilityError::Unenforceable {
                role: "pg_superuser".to_owned()
            })
        );
    }

    #[test]
    fn rls_role_flags_bypassrls_is_rejected_as_unenforceable() {
        assert_eq!(
            evaluate_rls_role_flags("bypass_role", EXPECTED_ROLE, false, true, true),
            Err(RlsEnforceabilityError::Unenforceable {
                role: "bypass_role".to_owned()
            })
        );
    }

    #[test]
    fn rls_role_flags_super_and_bypass_is_rejected_as_unenforceable() {
        // Both flags set still classifies as the bypass risk (not a membership
        // gap): the leak vector dominates the decision.
        assert_eq!(
            evaluate_rls_role_flags("both_flags", EXPECTED_ROLE, true, true, false),
            Err(RlsEnforceabilityError::Unenforceable {
                role: "both_flags".to_owned()
            })
        );
    }

    #[test]
    fn rls_role_flags_non_member_is_role_mismatch() {
        assert_eq!(
            evaluate_rls_role_flags("some_role", EXPECTED_ROLE, false, false, false),
            Err(RlsEnforceabilityError::RoleMismatch {
                role: "some_role".to_owned(),
                expected: EXPECTED_ROLE.to_owned(),
            })
        );
    }

    #[test]
    fn rls_role_flags_non_super_non_bypass_member_is_ok() {
        assert_eq!(
            evaluate_rls_role_flags(EXPECTED_ROLE, EXPECTED_ROLE, false, false, true),
            Ok(())
        );
    }

    // --- DB-free evaluate_rls_forced predicate tests -------------------------

    #[test]
    fn rls_forced_enabled_and_forced_is_ok() {
        assert_eq!(evaluate_rls_forced("schema.table", true, true), Ok(()));
    }

    #[test]
    fn rls_forced_enabled_but_not_forced_is_rejected() {
        assert_eq!(
            evaluate_rls_forced("schema.table", true, false),
            Err(RlsEnforceabilityError::RlsNotForced {
                table: "schema.table".to_owned(),
                row_security: true,
                force_row_security: false,
            })
        );
    }

    #[test]
    fn rls_forced_not_enabled_is_rejected() {
        assert_eq!(
            evaluate_rls_forced("schema.table", false, false),
            Err(RlsEnforceabilityError::RlsNotForced {
                table: "schema.table".to_owned(),
                row_security: false,
                force_row_security: false,
            })
        );
    }
}
