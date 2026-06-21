//! SCIM identity Postgres/RLS durable-storage plan + write-statement contract
//! for the [`UserStore`] / [`GroupStore`] ports.
//!
//! This crate is the declarative durable-storage seam that must exist before
//! the SCIM provisioning surface (`iam/facade/identity-service/src/users`) can
//! move its records off the in-memory reference stores
//! (`InMemoryUserStore` / `InMemoryGroupStore` in
//! `libs/oya-shared-scim-server-kernel`) onto managed Postgres. It models the
//! tenant-scoped Users and Groups tables, their row-level-security policies,
//! the per-tenant `userName` uniqueness constraint, and the parameterized
//! tenant-scoped statements a future durable adapter must use.
//!
//! ## Doctrine: transient adapter behind owned-shaped ports
//!
//! The kernel ports `UserStore`/`GroupStore` are the OWNED-destination
//! contracts (tenant-scoped record stores). Postgres is a TRANSIENT adapter
//! behind them: the OWNED data substrate is G003/oya-data, which cuts over
//! later WITHOUT changing the ports. This crate therefore mirrors the named
//! precedent `iam/adapters/tenant-rbac-postgres-rls-storage`: a review-only
//! declarative plan, NOT a live driver. It deliberately does NOT open a
//! database connection, run migrations, prepare/execute statements, persist
//! records, attach a cloud database, emit runtime audit-chain events, or claim
//! durable-storage readiness — and it does NOT implement the synchronous kernel
//! traits over a live backend (that live-wiring is a later slice; the in-memory
//! stores remain the dev/test realization). The synchronous ports stay
//! unchanged; a future live execution seam, when it lands, follows the
//! env-gated async command-adapter pattern
//! (`libs/oya-shared-postgres-command-adapter-sqlx`) rather than blocking on
//! async inside the sync trait.
//!
//! ## What it proves (hermetically, as SHAPE)
//!
//! - CRUD round-trip: Users and Groups each have a tenant-scoped table with the
//!   indexed scalar columns plus the `payload_json` aggregate, a parameterized
//!   INSERT/upsert, a tenant-scoped point SELECT, a `find_by_user_name` lookup,
//!   and a tenant-scoped DELETE.
//! - Tenant isolation (RLS denies cross-tenant): both tables are
//!   `ENABLE`+`FORCE ROW LEVEL SECURITY` under a RESTRICTIVE policy keyed on
//!   `current_setting('app.tenant_id', true)`.
//! - `userName` uniqueness is per-tenant: a UNIQUE `(tenant_id, user_name)`
//!   constraint so two tenants may reuse a userName but one tenant may not.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

const SCHEMA_VERSION: u32 = 1;
const SCHEMA_NAME: &str = "identity_scim";
const RUNTIME_ROLE: &str = "identity_scim_runtime";
const TENANT_CONTEXT_SETTING: &str = "app.tenant_id";
const MIN_TABLE_COUNT: usize = 2;

const INSERT_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-insert.html";
const SET_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-set.html";
const RLS_DOC_URL: &str = "https://www.postgresql.org/docs/current/ddl-rowsecurity.html";
const SELECT_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-select.html";

/// The SCIM resource families this crate persists, one per kernel store port.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScimRecordKind {
    /// `UserStore`: tenant-scoped Users with per-tenant userName uniqueness.
    User,
    /// `GroupStore`: tenant-scoped Groups.
    Group,
}

/// A declared column in a SCIM storage table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScimPostgresColumn {
    pub name: &'static str,       // data_class: PUBLIC
    pub sql_type: &'static str,   // data_class: PUBLIC
    pub required: bool,           // data_class: PUBLIC
    pub data_class: &'static str, // data_class: PUBLIC
}

/// A declared tenant-scoped SCIM storage table with its RLS posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScimPostgresTable {
    pub table_name: &'static str,               // data_class: PUBLIC
    pub record_kind: ScimRecordKind,            // data_class: PUBLIC
    pub columns: Vec<ScimPostgresColumn>,       // data_class: PUBLIC
    pub primary_key_columns: Vec<&'static str>, // data_class: PUBLIC
    /// Per-tenant uniqueness scope, e.g. `(tenant_id, user_name)`; empty for
    /// tables with no secondary uniqueness beyond the primary key.
    pub unique_scope_columns: Vec<&'static str>, // data_class: PUBLIC
    pub rls_policy_name: &'static str,          // data_class: PUBLIC
    pub tenant_context_setting: &'static str,   // data_class: INTERNAL_ONLY
    pub enable_row_level_security: bool,        // data_class: PUBLIC
    pub force_row_level_security: bool,         // data_class: PUBLIC
    pub select_policy_required: bool,           // data_class: PUBLIC
    pub insert_policy_required: bool,           // data_class: PUBLIC
    pub update_policy_required: bool,           // data_class: PUBLIC
    pub delete_policy_required: bool,           // data_class: PUBLIC
}

/// The full declarative SCIM storage plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScimPostgresStoragePlan {
    pub plan_name: &'static str,                // data_class: PUBLIC
    pub schema_name: &'static str,              // data_class: PUBLIC
    pub runtime_role: &'static str,             // data_class: INTERNAL_ONLY
    pub tenant_context_setting: &'static str,   // data_class: INTERNAL_ONLY
    pub tables: Vec<ScimPostgresTable>,         // data_class: PUBLIC
    pub default_deny_when_policy_missing: bool, // data_class: PUBLIC
    pub owner_force_rls_required: bool,         // data_class: PUBLIC
    pub bypassrls_role_forbidden: bool,         // data_class: PUBLIC
    pub migration_sql_review_only: bool,        // data_class: PUBLIC
    pub runtime_database_attached: bool,        // data_class: INTERNAL_ONLY
    pub postgres_connection_attached: bool,     // data_class: INTERNAL_ONLY
    pub migration_applied_attached: bool,       // data_class: INTERNAL_ONLY
    pub rls_runtime_verified_attached: bool,    // data_class: INTERNAL_ONLY
    pub durable_storage_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub cloud_database_attached: bool,          // data_class: INTERNAL_ONLY
    pub kernel_trait_runtime_attached: bool,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

/// The parameterized tenant-scoped statement set for one SCIM table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScimPostgresWriteStatement {
    pub table_name: &'static str,    // data_class: PUBLIC
    pub record_kind: ScimRecordKind, // data_class: PUBLIC
    pub tenant_context_sql: String,  // data_class: INTERNAL_ONLY
    pub insert_sql: String,          // data_class: INTERNAL_ONLY
    pub select_by_id_sql: String,    // data_class: INTERNAL_ONLY
    pub list_by_tenant_sql: String,  // data_class: INTERNAL_ONLY
    pub delete_by_id_sql: String,    // data_class: INTERNAL_ONLY
    /// Present only for the Users table (the `find_by_user_name` port method).
    pub find_by_user_name_sql: Option<String>, // data_class: INTERNAL_ONLY
    pub official_doc_urls: Vec<&'static str>, // data_class: PUBLIC
    pub uses_set_local_tenant_context: bool, // data_class: PUBLIC
    pub uses_parameterized_values: bool, // data_class: PUBLIC
    pub select_scoped_by_tenant: bool, // data_class: PUBLIC
    pub runtime_execution_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,         // data_class: PUBLIC
}

/// Validation failures for the SCIM storage plan and write contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScimPostgresError {
    InvalidPlan,
    InvalidTable,
    MissingTenantColumn,
    MissingPayloadColumn,
    MissingIdColumn,
    MissingSchemaVersionColumn,
    InvalidPrimaryKey,
    MissingRlsPolicy,
    MissingUserNameUniqueness,
    DuplicateTable(String),
    MissingRecordKind(String),
    RuntimeAttachmentOverclaim,
}

/// The canonical SCIM storage plan.
#[must_use]
pub fn scim_postgres_storage_plan() -> ScimPostgresStoragePlan {
    ScimPostgresStoragePlan {
        plan_name: "identity-scim-store-postgres",
        schema_name: SCHEMA_NAME,
        runtime_role: RUNTIME_ROLE,
        tenant_context_setting: TENANT_CONTEXT_SETTING,
        tables: scim_postgres_tables(),
        default_deny_when_policy_missing: true,
        owner_force_rls_required: true,
        bypassrls_role_forbidden: true,
        migration_sql_review_only: true,
        runtime_database_attached: false,
        postgres_connection_attached: false,
        migration_applied_attached: false,
        rls_runtime_verified_attached: false,
        durable_storage_runtime_attached: false,
        cloud_database_attached: false,
        kernel_trait_runtime_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

/// The two tenant-scoped tables, one per SCIM store port.
#[must_use]
pub fn scim_postgres_tables() -> Vec<ScimPostgresTable> {
    vec![
        ScimPostgresTable {
            table_name: "identity_scim_users",
            record_kind: ScimRecordKind::User,
            columns: user_columns(),
            primary_key_columns: vec!["tenant_id", "scim_id"],
            unique_scope_columns: vec!["tenant_id", "user_name"],
            rls_policy_name: "identity_scim_users_tenant_rls",
            tenant_context_setting: TENANT_CONTEXT_SETTING,
            enable_row_level_security: true,
            force_row_level_security: true,
            select_policy_required: true,
            insert_policy_required: true,
            update_policy_required: true,
            delete_policy_required: true,
        },
        ScimPostgresTable {
            table_name: "identity_scim_groups",
            record_kind: ScimRecordKind::Group,
            columns: group_columns(),
            primary_key_columns: vec!["tenant_id", "scim_id"],
            unique_scope_columns: vec![],
            rls_policy_name: "identity_scim_groups_tenant_rls",
            tenant_context_setting: TENANT_CONTEXT_SETTING,
            enable_row_level_security: true,
            force_row_level_security: true,
            select_policy_required: true,
            insert_policy_required: true,
            update_policy_required: true,
            delete_policy_required: true,
        },
    ]
}

/// The parameterized tenant-scoped write contract derived from the plan.
pub fn scim_postgres_write_statements() -> Result<Vec<ScimPostgresWriteStatement>, ScimPostgresError>
{
    let plan = scim_postgres_storage_plan();
    validate_scim_postgres_storage_plan(&plan)?;
    Ok(plan
        .tables
        .iter()
        .map(|table| write_statement(&plan, table))
        .collect())
}

/// Validate the SCIM storage plan.
pub fn validate_scim_postgres_storage_plan(
    plan: &ScimPostgresStoragePlan,
) -> Result<(), ScimPostgresError> {
    if !valid_identifier(plan.plan_name)
        || !valid_identifier(plan.schema_name)
        || !valid_identifier(plan.runtime_role)
        || plan.tenant_context_setting != TENANT_CONTEXT_SETTING
        || plan.tables.len() < MIN_TABLE_COUNT
        || !plan.default_deny_when_policy_missing
        || !plan.owner_force_rls_required
        || !plan.bypassrls_role_forbidden
        || !plan.migration_sql_review_only
        || plan.schema_version != SCHEMA_VERSION
    {
        return Err(ScimPostgresError::InvalidPlan);
    }
    if plan.runtime_database_attached
        || plan.postgres_connection_attached
        || plan.migration_applied_attached
        || plan.rls_runtime_verified_attached
        || plan.durable_storage_runtime_attached
        || plan.cloud_database_attached
        || plan.kernel_trait_runtime_attached
    {
        return Err(ScimPostgresError::RuntimeAttachmentOverclaim);
    }

    let mut seen_tables = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for table in &plan.tables {
        validate_table(table)?;
        if !seen_tables.insert(table.table_name.to_owned()) {
            return Err(ScimPostgresError::DuplicateTable(
                table.table_name.to_owned(),
            ));
        }
        seen_kinds.insert(table.record_kind);
    }
    for kind in [ScimRecordKind::User, ScimRecordKind::Group] {
        if !seen_kinds.contains(&kind) {
            return Err(ScimPostgresError::MissingRecordKind(format!("{kind:?}")));
        }
    }
    Ok(())
}

/// Render the idempotent, review-only migration SQL for the whole plan.
pub fn render_scim_postgres_migration(
    plan: &ScimPostgresStoragePlan,
) -> Result<String, ScimPostgresError> {
    validate_scim_postgres_storage_plan(plan)?;
    let mut sql = String::new();
    sql.push_str(&format!(
        "CREATE SCHEMA IF NOT EXISTS {};\n",
        plan.schema_name
    ));
    for table in &plan.tables {
        sql.push('\n');
        sql.push_str(&render_table_sql(plan, table));
    }
    Ok(sql)
}

fn user_columns() -> Vec<ScimPostgresColumn> {
    vec![
        column("tenant_id", "text", "INTERNAL_ONLY"),
        column("scim_id", "text", "TENANT_SCOPED"),
        column("user_name", "text", "SENSITIVE_PIPA_ART23"),
        column("external_id", "text", "TENANT_SCOPED"),
        column("active", "boolean", "TENANT_SCOPED"),
        column("payload_json", "jsonb", "SENSITIVE_PIPA_ART23"),
        column("schema_version", "integer", "PUBLIC"),
        column("updated_at", "timestamptz", "INTERNAL_ONLY"),
    ]
}

fn group_columns() -> Vec<ScimPostgresColumn> {
    vec![
        column("tenant_id", "text", "INTERNAL_ONLY"),
        column("scim_id", "text", "TENANT_SCOPED"),
        column("display_name", "text", "TENANT_SCOPED"),
        column("payload_json", "jsonb", "TENANT_SCOPED"),
        column("schema_version", "integer", "PUBLIC"),
        column("updated_at", "timestamptz", "INTERNAL_ONLY"),
    ]
}

fn column(
    name: &'static str,
    sql_type: &'static str,
    data_class: &'static str,
) -> ScimPostgresColumn {
    ScimPostgresColumn {
        name,
        sql_type,
        required: true,
        data_class,
    }
}

fn validate_table(table: &ScimPostgresTable) -> Result<(), ScimPostgresError> {
    if !valid_identifier(table.table_name)
        || !valid_identifier(table.rls_policy_name)
        || table.tenant_context_setting != TENANT_CONTEXT_SETTING
        || table.columns.is_empty()
        || !table.enable_row_level_security
        || !table.force_row_level_security
        || !table.select_policy_required
        || !table.insert_policy_required
        || !table.update_policy_required
        || !table.delete_policy_required
    {
        return Err(ScimPostgresError::InvalidTable);
    }
    require_column(table, "tenant_id", ScimPostgresError::MissingTenantColumn)?;
    require_column(table, "scim_id", ScimPostgresError::MissingIdColumn)?;
    require_column(
        table,
        "payload_json",
        ScimPostgresError::MissingPayloadColumn,
    )?;
    require_column(
        table,
        "schema_version",
        ScimPostgresError::MissingSchemaVersionColumn,
    )?;
    if table.primary_key_columns != ["tenant_id", "scim_id"] {
        return Err(ScimPostgresError::InvalidPrimaryKey);
    }
    if table.rls_policy_name.is_empty() {
        return Err(ScimPostgresError::MissingRlsPolicy);
    }
    // Users carry a per-tenant userName uniqueness scope (the
    // `find_by_user_name` + 409 Uniqueness contract).
    if table.record_kind == ScimRecordKind::User
        && (table.unique_scope_columns != ["tenant_id", "user_name"]
            || !table.columns.iter().any(|c| c.name == "user_name"))
    {
        return Err(ScimPostgresError::MissingUserNameUniqueness);
    }
    Ok(())
}

fn require_column(
    table: &ScimPostgresTable,
    name: &str,
    error: ScimPostgresError,
) -> Result<(), ScimPostgresError> {
    if table
        .columns
        .iter()
        .any(|column| column.name == name && column.required)
    {
        Ok(())
    } else {
        Err(error)
    }
}

fn render_table_sql(plan: &ScimPostgresStoragePlan, table: &ScimPostgresTable) -> String {
    let qualified = format!("{}.{}", plan.schema_name, table.table_name);
    let column_sql = table
        .columns
        .iter()
        .map(|column| format!("    {} {} NOT NULL", column.name, column.sql_type))
        .collect::<Vec<_>>()
        .join(",\n");
    let primary_key = table.primary_key_columns.join(", ");
    let unique_sql = if table.unique_scope_columns.is_empty() {
        String::new()
    } else {
        format!(",\n    UNIQUE ({})", table.unique_scope_columns.join(", "))
    };
    let tenant_predicate = format!(
        "tenant_id = current_setting('{}', true)",
        table.tenant_context_setting
    );
    format!(
        "CREATE TABLE IF NOT EXISTS {qualified} (\n{column_sql},\n    PRIMARY KEY ({primary_key}){unique_sql}\n);\nALTER TABLE {qualified} ENABLE ROW LEVEL SECURITY;\nALTER TABLE {qualified} FORCE ROW LEVEL SECURITY;\nCREATE POLICY {policy} ON {qualified} AS RESTRICTIVE FOR ALL TO {role} USING ({predicate}) WITH CHECK ({predicate});\nCOMMENT ON TABLE {qualified} IS 'SCIM identity review-only Postgres/RLS storage plan; migrations are not applied by this crate.';\n",
        policy = table.rls_policy_name,
        role = plan.runtime_role,
        predicate = tenant_predicate,
    )
}

fn write_statement(
    plan: &ScimPostgresStoragePlan,
    table: &ScimPostgresTable,
) -> ScimPostgresWriteStatement {
    let qualified = format!("{}.{}", plan.schema_name, table.table_name);
    let insert_columns = table
        .columns
        .iter()
        .map(|column| column.name)
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders = (1..=table.columns.len())
        .map(|index| {
            let column = &table.columns[index - 1];
            if column.sql_type == "jsonb" {
                format!("${index}::jsonb")
            } else {
                format!("${index}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    // PUT is an upsert keyed on (tenant_id, scim_id): create-or-replace, the
    // `UserStore::put` / `GroupStore::put` contract.
    let update_assignments = table
        .columns
        .iter()
        .filter(|column| !table.primary_key_columns.contains(&column.name))
        .map(|column| format!("{0} = EXCLUDED.{0}", column.name))
        .collect::<Vec<_>>()
        .join(", ");
    let select_columns = insert_columns.clone();
    let find_by_user_name_sql = if table.record_kind == ScimRecordKind::User {
        Some(format!(
            "SELECT {select_columns} FROM {qualified} WHERE tenant_id = $1 AND user_name = $2"
        ))
    } else {
        None
    };
    ScimPostgresWriteStatement {
        table_name: table.table_name,
        record_kind: table.record_kind,
        tenant_context_sql: format!("SET LOCAL {} = $1", plan.tenant_context_setting),
        insert_sql: format!(
            "INSERT INTO {qualified} ({insert_columns}) VALUES ({placeholders}) ON CONFLICT (tenant_id, scim_id) DO UPDATE SET {update_assignments}"
        ),
        select_by_id_sql: format!(
            "SELECT {select_columns} FROM {qualified} WHERE tenant_id = $1 AND scim_id = $2"
        ),
        list_by_tenant_sql: format!(
            "SELECT {select_columns} FROM {qualified} WHERE tenant_id = $1 ORDER BY scim_id"
        ),
        delete_by_id_sql: format!("DELETE FROM {qualified} WHERE tenant_id = $1 AND scim_id = $2"),
        find_by_user_name_sql,
        official_doc_urls: vec![INSERT_DOC_URL, SET_DOC_URL, RLS_DOC_URL, SELECT_DOC_URL],
        uses_set_local_tenant_context: true,
        uses_parameterized_values: true,
        select_scoped_by_tenant: true,
        runtime_execution_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && !unsafe_text(value)
        && !value.contains("__")
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-'))
}

fn unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
        || value.contains("..")
        || value.contains('\\')
        || value.contains('/')
        || value.contains(';')
        || value.contains('\'')
        || value.to_ascii_lowercase().contains("bypassrls")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_plan_covers_users_and_groups_with_tenant_rls() {
        let plan = scim_postgres_storage_plan();
        validate_scim_postgres_storage_plan(&plan).expect("plan validates");

        assert_eq!(plan.schema_name, "identity_scim");
        assert_eq!(plan.tables.len(), 2);
        assert!(plan.owner_force_rls_required);
        assert!(plan.bypassrls_role_forbidden);
        assert!(plan.migration_sql_review_only);
        assert!(!plan.cloud_database_attached);
        assert!(!plan.kernel_trait_runtime_attached);

        let kinds: BTreeSet<_> = plan.tables.iter().map(|t| t.record_kind).collect();
        assert!(kinds.contains(&ScimRecordKind::User));
        assert!(kinds.contains(&ScimRecordKind::Group));

        for table in &plan.tables {
            assert!(table.enable_row_level_security);
            assert!(table.force_row_level_security);
            assert_eq!(table.primary_key_columns, ["tenant_id", "scim_id"]);
            assert!(table.columns.iter().any(|c| c.name == "tenant_id"));
            assert!(
                table
                    .columns
                    .iter()
                    .any(|c| c.name == "payload_json" && c.sql_type == "jsonb")
            );
        }
    }

    #[test]
    fn user_table_enforces_per_tenant_user_name_uniqueness() {
        let plan = scim_postgres_storage_plan();
        let users = plan
            .tables
            .iter()
            .find(|t| t.record_kind == ScimRecordKind::User)
            .expect("users table present");
        assert_eq!(users.unique_scope_columns, ["tenant_id", "user_name"]);

        let sql = render_scim_postgres_migration(&plan).expect("migration renders");
        assert!(sql.contains("UNIQUE (tenant_id, user_name)"));
    }

    #[test]
    fn migration_sql_enables_and_forces_tenant_scoped_rls() {
        let plan = scim_postgres_storage_plan();
        let sql = render_scim_postgres_migration(&plan).expect("migration renders");

        assert!(sql.contains("CREATE SCHEMA IF NOT EXISTS identity_scim"));
        for table in &plan.tables {
            let qualified = format!("identity_scim.{}", table.table_name);
            assert!(sql.contains(&format!("CREATE TABLE IF NOT EXISTS {qualified}")));
            assert!(sql.contains(&format!(
                "ALTER TABLE {qualified} ENABLE ROW LEVEL SECURITY"
            )));
            assert!(sql.contains(&format!("ALTER TABLE {qualified} FORCE ROW LEVEL SECURITY")));
            assert!(sql.contains(&format!("CREATE POLICY {}", table.rls_policy_name)));
        }
        assert!(sql.contains("AS RESTRICTIVE FOR ALL"));
        assert!(sql.contains("current_setting('app.tenant_id', true)"));
        assert!(sql.contains("WITH CHECK"));
        assert!(!sql.contains("DROP TABLE"));
        assert!(!sql.contains("BYPASSRLS"));
    }

    #[test]
    fn write_statements_cover_the_full_store_port_surface() {
        let statements = scim_postgres_write_statements().expect("statements build");
        assert_eq!(statements.len(), 2);
        for statement in &statements {
            assert_eq!(statement.tenant_context_sql, "SET LOCAL app.tenant_id = $1");
            assert!(statement.uses_parameterized_values);
            // put = create-or-replace upsert on the tenant-scoped key.
            assert!(
                statement
                    .insert_sql
                    .contains("ON CONFLICT (tenant_id, scim_id) DO UPDATE SET")
            );
            // get / list / delete / find_by_user_name are all tenant-scoped.
            assert!(
                statement
                    .select_by_id_sql
                    .ends_with("WHERE tenant_id = $1 AND scim_id = $2")
            );
            assert!(
                statement
                    .list_by_tenant_sql
                    .contains("WHERE tenant_id = $1")
            );
            assert!(
                statement
                    .delete_by_id_sql
                    .starts_with("DELETE FROM identity_scim.")
            );
            assert!(
                statement
                    .delete_by_id_sql
                    .ends_with("WHERE tenant_id = $1 AND scim_id = $2")
            );
            assert!(statement.select_scoped_by_tenant);
            assert!(!statement.runtime_execution_attached);
        }
        let users = statements
            .iter()
            .find(|s| s.record_kind == ScimRecordKind::User)
            .expect("user statement");
        let find = users
            .find_by_user_name_sql
            .as_deref()
            .expect("users carry find_by_user_name");
        assert!(find.ends_with("WHERE tenant_id = $1 AND user_name = $2"));
        let groups = statements
            .iter()
            .find(|s| s.record_kind == ScimRecordKind::Group)
            .expect("group statement");
        assert!(groups.find_by_user_name_sql.is_none());
    }

    #[test]
    fn committed_migration_file_matches_rendered_plan() {
        let plan = scim_postgres_storage_plan();
        let rendered = render_scim_postgres_migration(&plan).expect("migration renders");
        let committed = include_str!("../migrations/0001_identity_scim_store.sql");
        assert!(
            committed.ends_with(&rendered),
            "committed migration drifted from rendered plan"
        );
    }

    #[test]
    fn rejects_runtime_overclaims_and_missing_invariants() {
        let mut plan = scim_postgres_storage_plan();
        plan.migration_applied_attached = true;
        assert_eq!(
            validate_scim_postgres_storage_plan(&plan),
            Err(ScimPostgresError::RuntimeAttachmentOverclaim)
        );

        let mut plan = scim_postgres_storage_plan();
        plan.kernel_trait_runtime_attached = true;
        assert_eq!(
            validate_scim_postgres_storage_plan(&plan),
            Err(ScimPostgresError::RuntimeAttachmentOverclaim)
        );

        let mut plan = scim_postgres_storage_plan();
        plan.tables[0].force_row_level_security = false;
        assert_eq!(
            validate_scim_postgres_storage_plan(&plan),
            Err(ScimPostgresError::InvalidTable)
        );

        let mut plan = scim_postgres_storage_plan();
        plan.tables[0].unique_scope_columns = vec!["user_name"];
        assert_eq!(
            validate_scim_postgres_storage_plan(&plan),
            Err(ScimPostgresError::MissingUserNameUniqueness)
        );
    }

    #[test]
    fn rejects_missing_tenant_column_and_duplicate_tables() {
        let mut plan = scim_postgres_storage_plan();
        plan.tables[0].columns.retain(|c| c.name != "tenant_id");
        assert_eq!(
            validate_scim_postgres_storage_plan(&plan),
            Err(ScimPostgresError::MissingTenantColumn)
        );

        let mut plan = scim_postgres_storage_plan();
        let duplicate = plan.tables[0].clone();
        plan.tables.push(duplicate);
        assert!(matches!(
            validate_scim_postgres_storage_plan(&plan),
            Err(ScimPostgresError::DuplicateTable(_))
        ));
    }
}
