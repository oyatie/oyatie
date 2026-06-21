//! Tenant lifecycle Postgres/RLS durable-storage plan + write-statement
//! contract for the [`TenantLifecycleStore`] port.
//!
//! This crate is the declarative durable-storage seam that must exist before
//! the tenant lifecycle control plane can move its records off the in-memory
//! reference adapter (`tenancy/adapters/tenant-lifecycle-store-inmemory`) onto
//! managed Postgres. It models the tenant-scoped tables, row-level-security
//! policies, the idempotency-key dedup table, and the parameterized
//! tenant-scoped statements a future durable adapter must use.
//!
//! ## Doctrine: transient adapter behind an owned-shaped port
//!
//! The kernel port [`TenantLifecycleStore`] is the OWNED-destination contract
//! (the oya-data ordered-keyed KV shape; see the kernel crate). Postgres is a
//! TRANSIENT adapter behind it: the OWNED data substrate is G003/oya-data,
//! which cuts over later WITHOUT changing the port. This crate therefore mirrors
//! the named precedent `iam/adapters/tenant-rbac-postgres-rls-storage`: a
//! review-only declarative plan, NOT a live driver. It deliberately does NOT
//! open a database connection, run migrations, prepare/execute statements,
//! persist records, attach a cloud database, emit runtime audit-chain events,
//! or claim durable-storage readiness — and it does NOT implement the
//! synchronous kernel trait over a live backend (that live-wiring is a later
//! slice; the in-memory adapter remains the dev/test realization). The synchronous
//! port stays unchanged; a future live execution seam, when it lands, follows
//! the env-gated async command-adapter pattern
//! (`libs/oya-shared-postgres-command-adapter-sqlx`) rather than blocking on
//! async inside the sync trait.
//!
//! ## What it proves (hermetically, as SHAPE)
//!
//! - CRUD round-trip: every record family has a tenant-scoped table with the
//!   indexed scalar columns plus the `payload_json` aggregate, a parameterized
//!   INSERT, and a tenant-scoped point SELECT read-back.
//! - Tenant isolation (RLS denies cross-tenant): every table is
//!   `ENABLE`+`FORCE ROW LEVEL SECURITY` under a RESTRICTIVE policy keyed on
//!   `current_setting('app.tenant_id', true)`.
//! - Idempotency-key replay: the applied-writes table is keyed on
//!   `(tenant_id, idempotency_key)` and its INSERT uses
//!   `ON CONFLICT (tenant_id, idempotency_key) DO NOTHING`, so a replay is a
//!   no-op and the read-back returns the originally-applied record.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

const SCHEMA_VERSION: u32 = 1;
const SCHEMA_NAME: &str = "tenancy_lifecycle";
const RUNTIME_ROLE: &str = "tenancy_lifecycle_runtime";
const TENANT_CONTEXT_SETTING: &str = "app.tenant_id";
const MIN_TABLE_COUNT: usize = 3;

const INSERT_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-insert.html";
const SET_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-set.html";
const RLS_DOC_URL: &str = "https://www.postgresql.org/docs/current/ddl-rowsecurity.html";
const SELECT_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-select.html";

/// The record families the lifecycle store persists, one per kernel-port
/// record kind (`Tenant`, `AppliedWriteRecord`, `OperationRecord`).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TenantLifecycleRecordKind {
    /// `tenants/<id>` -> tenant aggregate (point get/put/remove + range scan).
    Tenant,
    /// Idempotency dedup table: client-UUID key -> first-applied record.
    AppliedWrite,
    /// AIP-151 operation ledger: `operations/...` -> ledger entry.
    Operation,
}

/// A single declared column in a lifecycle storage table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantLifecyclePostgresColumn {
    pub name: &'static str,       // data_class: PUBLIC
    pub sql_type: &'static str,   // data_class: PUBLIC
    pub required: bool,           // data_class: PUBLIC
    pub data_class: &'static str, // data_class: PUBLIC
}

/// A declared tenant-scoped storage table with its RLS posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantLifecyclePostgresTable {
    pub table_name: &'static str,                    // data_class: PUBLIC
    pub record_kind: TenantLifecycleRecordKind,      // data_class: PUBLIC
    pub columns: Vec<TenantLifecyclePostgresColumn>, // data_class: PUBLIC
    pub primary_key_columns: Vec<&'static str>,      // data_class: PUBLIC
    pub rls_policy_name: &'static str,               // data_class: PUBLIC
    pub tenant_context_setting: &'static str,        // data_class: INTERNAL_ONLY
    pub enable_row_level_security: bool,             // data_class: PUBLIC
    pub force_row_level_security: bool,              // data_class: PUBLIC
    pub select_policy_required: bool,                // data_class: PUBLIC
    pub insert_policy_required: bool,                // data_class: PUBLIC
    pub update_policy_required: bool,                // data_class: PUBLIC
    pub delete_policy_required: bool,                // data_class: PUBLIC
    /// The idempotency dedup table relies on `ON CONFLICT DO NOTHING` for
    /// replay safety; only that table sets this.
    pub idempotency_conflict_do_nothing: bool, // data_class: PUBLIC
}

/// The full declarative storage plan for the lifecycle store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantLifecyclePostgresStoragePlan {
    pub plan_name: &'static str,                   // data_class: PUBLIC
    pub schema_name: &'static str,                 // data_class: PUBLIC
    pub runtime_role: &'static str,                // data_class: INTERNAL_ONLY
    pub tenant_context_setting: &'static str,      // data_class: INTERNAL_ONLY
    pub tables: Vec<TenantLifecyclePostgresTable>, // data_class: PUBLIC
    pub default_deny_when_policy_missing: bool,    // data_class: PUBLIC
    pub owner_force_rls_required: bool,            // data_class: PUBLIC
    pub bypassrls_role_forbidden: bool,            // data_class: PUBLIC
    pub migration_sql_review_only: bool,           // data_class: PUBLIC
    pub runtime_database_attached: bool,           // data_class: INTERNAL_ONLY
    pub postgres_connection_attached: bool,        // data_class: INTERNAL_ONLY
    pub migration_applied_attached: bool,          // data_class: INTERNAL_ONLY
    pub rls_runtime_verified_attached: bool,       // data_class: INTERNAL_ONLY
    pub durable_storage_runtime_attached: bool,    // data_class: INTERNAL_ONLY
    pub cloud_database_attached: bool,             // data_class: INTERNAL_ONLY
    pub kernel_trait_runtime_attached: bool,       // data_class: INTERNAL_ONLY
    pub schema_version: u32,                       // data_class: PUBLIC
}

/// The parameterized tenant-scoped statement set for one table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantLifecyclePostgresWriteStatement {
    pub table_name: &'static str,               // data_class: PUBLIC
    pub record_kind: TenantLifecycleRecordKind, // data_class: PUBLIC
    pub tenant_context_sql: String,             // data_class: INTERNAL_ONLY
    pub insert_sql: String,                     // data_class: INTERNAL_ONLY
    pub select_by_primary_key_sql: String,      // data_class: INTERNAL_ONLY
    pub official_doc_urls: Vec<&'static str>,   // data_class: PUBLIC
    pub uses_set_local_tenant_context: bool,    // data_class: PUBLIC
    pub uses_parameterized_values: bool,        // data_class: PUBLIC
    pub uses_on_conflict_do_nothing: bool,      // data_class: PUBLIC
    pub select_scoped_by_tenant: bool,          // data_class: PUBLIC
    pub runtime_execution_attached: bool,       // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

/// Validation failures for the storage plan and write contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantLifecyclePostgresError {
    InvalidPlan,
    InvalidTable,
    MissingTenantColumn,
    MissingPayloadColumn,
    MissingSchemaVersionColumn,
    InvalidPrimaryKey,
    MissingRlsPolicy,
    DuplicateTable(String),
    MissingRecordKind(String),
    InvalidStatement,
    InvalidTenantContextSql,
    InvalidInsertSql,
    InvalidSelectSql,
    MissingIdempotencyConflict,
    RuntimeAttachmentOverclaim,
}

/// The canonical lifecycle storage plan.
#[must_use]
pub fn tenant_lifecycle_postgres_storage_plan() -> TenantLifecyclePostgresStoragePlan {
    TenantLifecyclePostgresStoragePlan {
        plan_name: "tenant-lifecycle-store-postgres",
        schema_name: SCHEMA_NAME,
        runtime_role: RUNTIME_ROLE,
        tenant_context_setting: TENANT_CONTEXT_SETTING,
        tables: tenant_lifecycle_postgres_tables(),
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

/// The three tenant-scoped tables, one per kernel-port record family.
#[must_use]
pub fn tenant_lifecycle_postgres_tables() -> Vec<TenantLifecyclePostgresTable> {
    vec![
        TenantLifecyclePostgresTable {
            table_name: "tenancy_lifecycle_tenants",
            record_kind: TenantLifecycleRecordKind::Tenant,
            columns: tenant_columns(),
            primary_key_columns: vec!["tenant_id", "resource_name"],
            rls_policy_name: "tenancy_lifecycle_tenants_tenant_rls",
            tenant_context_setting: TENANT_CONTEXT_SETTING,
            enable_row_level_security: true,
            force_row_level_security: true,
            select_policy_required: true,
            insert_policy_required: true,
            update_policy_required: true,
            delete_policy_required: true,
            idempotency_conflict_do_nothing: false,
        },
        TenantLifecyclePostgresTable {
            table_name: "tenancy_lifecycle_applied_writes",
            record_kind: TenantLifecycleRecordKind::AppliedWrite,
            columns: applied_write_columns(),
            primary_key_columns: vec!["tenant_id", "idempotency_key"],
            rls_policy_name: "tenancy_lifecycle_applied_writes_tenant_rls",
            tenant_context_setting: TENANT_CONTEXT_SETTING,
            enable_row_level_security: true,
            force_row_level_security: true,
            select_policy_required: true,
            insert_policy_required: true,
            update_policy_required: true,
            delete_policy_required: true,
            idempotency_conflict_do_nothing: true,
        },
        TenantLifecyclePostgresTable {
            table_name: "tenancy_lifecycle_operations",
            record_kind: TenantLifecycleRecordKind::Operation,
            columns: operation_columns(),
            primary_key_columns: vec!["tenant_id", "operation_name"],
            rls_policy_name: "tenancy_lifecycle_operations_tenant_rls",
            tenant_context_setting: TENANT_CONTEXT_SETTING,
            enable_row_level_security: true,
            force_row_level_security: true,
            select_policy_required: true,
            insert_policy_required: true,
            update_policy_required: true,
            delete_policy_required: true,
            idempotency_conflict_do_nothing: false,
        },
    ]
}

/// The parameterized tenant-scoped write contract derived from the plan.
pub fn tenant_lifecycle_postgres_write_statements()
-> Result<Vec<TenantLifecyclePostgresWriteStatement>, TenantLifecyclePostgresError> {
    let plan = tenant_lifecycle_postgres_storage_plan();
    validate_tenant_lifecycle_postgres_storage_plan(&plan)?;
    Ok(plan
        .tables
        .iter()
        .map(|table| write_statement(&plan, table))
        .collect())
}

/// Validate the storage plan: identifiers safe, RLS forced, tenant + payload +
/// schema-version columns present, primary keys tenant-leading, no runtime
/// overclaims, no duplicate tables, and every record family covered.
pub fn validate_tenant_lifecycle_postgres_storage_plan(
    plan: &TenantLifecyclePostgresStoragePlan,
) -> Result<(), TenantLifecyclePostgresError> {
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
        return Err(TenantLifecyclePostgresError::InvalidPlan);
    }
    if plan.runtime_database_attached
        || plan.postgres_connection_attached
        || plan.migration_applied_attached
        || plan.rls_runtime_verified_attached
        || plan.durable_storage_runtime_attached
        || plan.cloud_database_attached
        || plan.kernel_trait_runtime_attached
    {
        return Err(TenantLifecyclePostgresError::RuntimeAttachmentOverclaim);
    }

    let mut seen_tables = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for table in &plan.tables {
        validate_table(table)?;
        if !seen_tables.insert(table.table_name.to_owned()) {
            return Err(TenantLifecyclePostgresError::DuplicateTable(
                table.table_name.to_owned(),
            ));
        }
        seen_kinds.insert(table.record_kind);
    }
    for kind in [
        TenantLifecycleRecordKind::Tenant,
        TenantLifecycleRecordKind::AppliedWrite,
        TenantLifecycleRecordKind::Operation,
    ] {
        if !seen_kinds.contains(&kind) {
            return Err(TenantLifecyclePostgresError::MissingRecordKind(format!(
                "{kind:?}"
            )));
        }
    }
    Ok(())
}

/// Render the idempotent, review-only migration SQL for the whole plan.
pub fn render_tenant_lifecycle_postgres_migration(
    plan: &TenantLifecyclePostgresStoragePlan,
) -> Result<String, TenantLifecyclePostgresError> {
    validate_tenant_lifecycle_postgres_storage_plan(plan)?;
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

fn tenant_columns() -> Vec<TenantLifecyclePostgresColumn> {
    vec![
        column("tenant_id", "text", "INTERNAL_ONLY"),
        column("resource_name", "text", "TENANT_SCOPED"),
        column("display_name", "text", "TENANT_SCOPED"),
        column("lifecycle_state", "text", "TENANT_SCOPED"),
        column("payload_json", "jsonb", "INTERNAL_ONLY"),
        column("schema_version", "integer", "PUBLIC"),
        column("updated_at", "timestamptz", "INTERNAL_ONLY"),
    ]
}

fn applied_write_columns() -> Vec<TenantLifecyclePostgresColumn> {
    vec![
        column("tenant_id", "text", "INTERNAL_ONLY"),
        column("idempotency_key", "text", "INTERNAL_ONLY"),
        column("payload_json", "jsonb", "INTERNAL_ONLY"),
        column("schema_version", "integer", "PUBLIC"),
        column("created_at", "timestamptz", "INTERNAL_ONLY"),
    ]
}

fn operation_columns() -> Vec<TenantLifecyclePostgresColumn> {
    vec![
        column("tenant_id", "text", "INTERNAL_ONLY"),
        column("operation_name", "text", "TENANT_SCOPED"),
        column("operation_seq", "bigint", "INTERNAL_ONLY"),
        column("payload_json", "jsonb", "INTERNAL_ONLY"),
        column("schema_version", "integer", "PUBLIC"),
        column("created_at", "timestamptz", "INTERNAL_ONLY"),
    ]
}

fn column(
    name: &'static str,
    sql_type: &'static str,
    data_class: &'static str,
) -> TenantLifecyclePostgresColumn {
    TenantLifecyclePostgresColumn {
        name,
        sql_type,
        required: true,
        data_class,
    }
}

fn validate_table(
    table: &TenantLifecyclePostgresTable,
) -> Result<(), TenantLifecyclePostgresError> {
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
        return Err(TenantLifecyclePostgresError::InvalidTable);
    }
    require_column(
        table,
        "tenant_id",
        TenantLifecyclePostgresError::MissingTenantColumn,
    )?;
    require_column(
        table,
        "payload_json",
        TenantLifecyclePostgresError::MissingPayloadColumn,
    )?;
    require_column(
        table,
        "schema_version",
        TenantLifecyclePostgresError::MissingSchemaVersionColumn,
    )?;
    if table.primary_key_columns.first() != Some(&"tenant_id")
        || table.primary_key_columns.len() < 2
    {
        return Err(TenantLifecyclePostgresError::InvalidPrimaryKey);
    }
    for pk in &table.primary_key_columns {
        if !table.columns.iter().any(|column| &column.name == pk) {
            return Err(TenantLifecyclePostgresError::InvalidPrimaryKey);
        }
    }
    if table.rls_policy_name.is_empty() {
        return Err(TenantLifecyclePostgresError::MissingRlsPolicy);
    }
    // The idempotency dedup table MUST carry an idempotency_key in its key.
    if table.record_kind == TenantLifecycleRecordKind::AppliedWrite
        && (!table.idempotency_conflict_do_nothing
            || !table.primary_key_columns.contains(&"idempotency_key"))
    {
        return Err(TenantLifecyclePostgresError::MissingIdempotencyConflict);
    }
    Ok(())
}

fn require_column(
    table: &TenantLifecyclePostgresTable,
    name: &str,
    error: TenantLifecyclePostgresError,
) -> Result<(), TenantLifecyclePostgresError> {
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

fn render_table_sql(
    plan: &TenantLifecyclePostgresStoragePlan,
    table: &TenantLifecyclePostgresTable,
) -> String {
    let qualified = format!("{}.{}", plan.schema_name, table.table_name);
    let column_sql = table
        .columns
        .iter()
        .map(|column| format!("    {} {} NOT NULL", column.name, column.sql_type))
        .collect::<Vec<_>>()
        .join(",\n");
    let primary_key = table.primary_key_columns.join(", ");
    let tenant_predicate = format!(
        "tenant_id = current_setting('{}', true)",
        table.tenant_context_setting
    );
    format!(
        "CREATE TABLE IF NOT EXISTS {qualified} (\n{column_sql},\n    PRIMARY KEY ({primary_key})\n);\nALTER TABLE {qualified} ENABLE ROW LEVEL SECURITY;\nALTER TABLE {qualified} FORCE ROW LEVEL SECURITY;\nCREATE POLICY {policy} ON {qualified} AS RESTRICTIVE FOR ALL TO {role} USING ({predicate}) WITH CHECK ({predicate});\nCOMMENT ON TABLE {qualified} IS 'Tenant lifecycle review-only Postgres/RLS storage plan; migrations are not applied by this crate.';\n",
        policy = table.rls_policy_name,
        role = plan.runtime_role,
        predicate = tenant_predicate,
    )
}

fn write_statement(
    plan: &TenantLifecyclePostgresStoragePlan,
    table: &TenantLifecyclePostgresTable,
) -> TenantLifecyclePostgresWriteStatement {
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
    let conflict_clause = if table.idempotency_conflict_do_nothing {
        format!(
            " ON CONFLICT ({}) DO NOTHING",
            table.primary_key_columns.join(", ")
        )
    } else {
        format!(
            " ON CONFLICT ({}) DO UPDATE SET payload_json = EXCLUDED.payload_json, schema_version = EXCLUDED.schema_version",
            table.primary_key_columns.join(", ")
        )
    };
    let select_columns = table
        .columns
        .iter()
        .map(|column| column.name)
        .collect::<Vec<_>>()
        .join(", ");
    let where_clause = table
        .primary_key_columns
        .iter()
        .enumerate()
        .map(|(index, key)| format!("{key} = ${}", index + 1))
        .collect::<Vec<_>>()
        .join(" AND ");
    TenantLifecyclePostgresWriteStatement {
        table_name: table.table_name,
        record_kind: table.record_kind,
        tenant_context_sql: format!("SET LOCAL {} = $1", plan.tenant_context_setting),
        insert_sql: format!(
            "INSERT INTO {qualified} ({insert_columns}) VALUES ({placeholders}){conflict_clause}"
        ),
        select_by_primary_key_sql: format!(
            "SELECT {select_columns} FROM {qualified} WHERE {where_clause}"
        ),
        official_doc_urls: vec![INSERT_DOC_URL, SET_DOC_URL, RLS_DOC_URL, SELECT_DOC_URL],
        uses_set_local_tenant_context: true,
        uses_parameterized_values: true,
        uses_on_conflict_do_nothing: table.idempotency_conflict_do_nothing,
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
    fn storage_plan_covers_every_record_family_with_tenant_rls() {
        let plan = tenant_lifecycle_postgres_storage_plan();
        validate_tenant_lifecycle_postgres_storage_plan(&plan).expect("plan validates");

        assert_eq!(plan.schema_name, "tenancy_lifecycle");
        assert_eq!(plan.tables.len(), 3);
        assert!(plan.default_deny_when_policy_missing);
        assert!(plan.owner_force_rls_required);
        assert!(plan.bypassrls_role_forbidden);
        assert!(plan.migration_sql_review_only);
        assert!(!plan.runtime_database_attached);
        assert!(!plan.postgres_connection_attached);
        assert!(!plan.migration_applied_attached);
        assert!(!plan.durable_storage_runtime_attached);
        assert!(!plan.cloud_database_attached);
        assert!(!plan.kernel_trait_runtime_attached);

        // Every record family the kernel port persists is covered.
        let kinds: BTreeSet<_> = plan.tables.iter().map(|t| t.record_kind).collect();
        assert!(kinds.contains(&TenantLifecycleRecordKind::Tenant));
        assert!(kinds.contains(&TenantLifecycleRecordKind::AppliedWrite));
        assert!(kinds.contains(&TenantLifecycleRecordKind::Operation));

        for table in &plan.tables {
            assert!(table.enable_row_level_security);
            assert!(table.force_row_level_security);
            assert!(table.select_policy_required);
            assert!(table.insert_policy_required);
            assert!(table.update_policy_required);
            assert!(table.delete_policy_required);
            assert_eq!(table.primary_key_columns.first(), Some(&"tenant_id"));
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
    fn migration_sql_enables_and_forces_tenant_scoped_rls() {
        let plan = tenant_lifecycle_postgres_storage_plan();
        let sql = render_tenant_lifecycle_postgres_migration(&plan).expect("migration renders");

        assert!(sql.contains("CREATE SCHEMA IF NOT EXISTS tenancy_lifecycle"));
        for table in &plan.tables {
            let qualified = format!("tenancy_lifecycle.{}", table.table_name);
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
        // Review-only: no destructive or runtime statements rendered.
        assert!(!sql.contains("DROP TABLE"));
        assert!(!sql.contains("DELETE FROM"));
        assert!(!sql.contains("BYPASSRLS"));
    }

    #[test]
    fn idempotency_replay_uses_conflict_do_nothing_on_tenant_scoped_key() {
        let statements = tenant_lifecycle_postgres_write_statements().expect("statements build");
        let applied = statements
            .iter()
            .find(|s| s.record_kind == TenantLifecycleRecordKind::AppliedWrite)
            .expect("applied-writes statement present");

        // Tenant scope set before any statement (RLS precondition).
        assert_eq!(applied.tenant_context_sql, "SET LOCAL app.tenant_id = $1");
        assert!(applied.uses_set_local_tenant_context);
        // Idempotency-key replay is a no-op insert on the tenant-scoped key.
        assert!(applied.uses_on_conflict_do_nothing);
        assert!(
            applied
                .insert_sql
                .contains("ON CONFLICT (tenant_id, idempotency_key) DO NOTHING"),
            "insert: {}",
            applied.insert_sql
        );
        assert!(
            applied
                .insert_sql
                .starts_with("INSERT INTO tenancy_lifecycle.tenancy_lifecycle_applied_writes ")
        );
        // Read-back is scoped by the tenant-leading primary key.
        assert!(applied.select_scoped_by_tenant);
        assert!(
            applied
                .select_by_primary_key_sql
                .ends_with("WHERE tenant_id = $1 AND idempotency_key = $2"),
            "select: {}",
            applied.select_by_primary_key_sql
        );
    }

    #[test]
    fn crud_round_trip_statements_are_parameterized_and_tenant_scoped() {
        let statements = tenant_lifecycle_postgres_write_statements().expect("statements build");
        assert_eq!(statements.len(), 3);
        for statement in &statements {
            assert_eq!(statement.tenant_context_sql, "SET LOCAL app.tenant_id = $1");
            assert!(statement.uses_parameterized_values);
            assert!(statement.insert_sql.contains("VALUES ($1"));
            // The aggregate payload column is bound as jsonb.
            assert!(statement.insert_sql.contains("::jsonb"));
            assert!(statement.select_by_primary_key_sql.starts_with("SELECT "));
            assert!(
                statement
                    .select_by_primary_key_sql
                    .contains("WHERE tenant_id = $1")
            );
            assert!(!statement.runtime_execution_attached);
            // Non-idempotency tables upsert; the dedup table never overwrites.
            if statement.record_kind == TenantLifecycleRecordKind::AppliedWrite {
                assert!(statement.insert_sql.contains("DO NOTHING"));
            } else {
                assert!(statement.insert_sql.contains("DO UPDATE SET"));
            }
        }
    }

    #[test]
    fn committed_migration_file_matches_rendered_plan() {
        // The checked-in migration is the artifact a future durable adapter
        // applies; it MUST equal what the plan renders, or it has drifted. The
        // committed file carries a leading comment header; the rendered body is
        // the suffix after it.
        let plan = tenant_lifecycle_postgres_storage_plan();
        let rendered =
            render_tenant_lifecycle_postgres_migration(&plan).expect("migration renders");
        let committed = include_str!("../migrations/0001_tenant_lifecycle_store.sql");
        assert!(
            committed.ends_with(&rendered),
            "committed migration drifted from rendered plan"
        );
    }

    #[test]
    fn rejects_runtime_overclaims_and_missing_rls() {
        let mut plan = tenant_lifecycle_postgres_storage_plan();
        plan.migration_applied_attached = true;
        assert_eq!(
            validate_tenant_lifecycle_postgres_storage_plan(&plan),
            Err(TenantLifecyclePostgresError::RuntimeAttachmentOverclaim)
        );

        let mut plan = tenant_lifecycle_postgres_storage_plan();
        plan.kernel_trait_runtime_attached = true;
        assert_eq!(
            validate_tenant_lifecycle_postgres_storage_plan(&plan),
            Err(TenantLifecyclePostgresError::RuntimeAttachmentOverclaim)
        );

        let mut plan = tenant_lifecycle_postgres_storage_plan();
        plan.tables[0].force_row_level_security = false;
        assert_eq!(
            validate_tenant_lifecycle_postgres_storage_plan(&plan),
            Err(TenantLifecyclePostgresError::InvalidTable)
        );
    }

    #[test]
    fn rejects_missing_tenant_column_and_duplicate_or_missing_families() {
        let mut plan = tenant_lifecycle_postgres_storage_plan();
        plan.tables[0].columns.retain(|c| c.name != "tenant_id");
        assert_eq!(
            validate_tenant_lifecycle_postgres_storage_plan(&plan),
            Err(TenantLifecyclePostgresError::MissingTenantColumn)
        );

        let mut plan = tenant_lifecycle_postgres_storage_plan();
        let duplicate = plan.tables[0].clone();
        plan.tables.push(duplicate);
        assert!(matches!(
            validate_tenant_lifecycle_postgres_storage_plan(&plan),
            Err(TenantLifecyclePostgresError::DuplicateTable(_))
        ));

        // Replace the operations family with a second tenants table (count
        // stays at the minimum, but a required record kind is now absent).
        let mut plan = tenant_lifecycle_postgres_storage_plan();
        let mut substitute = plan.tables[0].clone();
        substitute.table_name = "tenancy_lifecycle_tenants_extra";
        substitute.rls_policy_name = "tenancy_lifecycle_tenants_extra_tenant_rls";
        plan.tables[2] = substitute;
        assert!(matches!(
            validate_tenant_lifecycle_postgres_storage_plan(&plan),
            Err(TenantLifecyclePostgresError::MissingRecordKind(_))
        ));
    }
}
