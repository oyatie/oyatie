//! Tenant RBAC Postgres/RLS durable storage schema foundation.
//!
//! This crate models the declarative Postgres table, idempotency, and row-level
//! security plan required before Tenant RBAC metadata can move from the
//! in-memory reference store to durable storage. It deliberately does not open a
//! database connection, run migrations, verify live RLS behavior, attach a cloud
//! database, persist records, emit runtime audit-chain events, or claim durable
//! storage readiness.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

const SCHEMA_VERSION: u32 = 1;
const MIN_TABLE_COUNT: usize = 5;
const TENANT_CONTEXT_SETTING: &str = "app.tenant_id";
const RUNTIME_ROLE: &str = "tenant_rbac_runtime";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TenantRbacPostgresRecordKind {
    PolicyAdmission,
    GroupCloseRollup,
    CrossServiceWorkflowPlan,
    IncidentRollbackPlan,
    OpsCommand,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacPostgresRlsColumn {
    pub name: &'static str,       // data_class: PUBLIC
    pub sql_type: &'static str,   // data_class: PUBLIC
    pub required: bool,           // data_class: PUBLIC
    pub data_class: &'static str, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacPostgresRlsTable {
    pub table_name: &'static str,                  // data_class: PUBLIC
    pub record_kind: TenantRbacPostgresRecordKind, // data_class: PUBLIC
    pub columns: Vec<TenantRbacPostgresRlsColumn>, // data_class: PUBLIC
    pub primary_key_columns: Vec<&'static str>,    // data_class: PUBLIC
    pub unique_idempotency_scope_columns: Vec<&'static str>, // data_class: PUBLIC
    pub rls_policy_name: &'static str,             // data_class: PUBLIC
    pub tenant_context_setting: &'static str,      // data_class: INTERNAL_ONLY
    pub enable_row_level_security: bool,           // data_class: PUBLIC
    pub force_row_level_security: bool,            // data_class: PUBLIC
    pub select_policy_required: bool,              // data_class: PUBLIC
    pub insert_policy_required: bool,              // data_class: PUBLIC
    pub update_policy_required: bool,              // data_class: PUBLIC
    pub delete_allowed: bool,                      // data_class: PUBLIC
    pub append_only: bool,                         // data_class: PUBLIC
    pub payload_encryption_required: bool,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacPostgresRlsStoragePlan {
    pub plan_name: &'static str,                     // data_class: PUBLIC
    pub schema_name: &'static str,                   // data_class: PUBLIC
    pub runtime_role: &'static str,                  // data_class: INTERNAL_ONLY
    pub tenant_context_setting: &'static str,        // data_class: INTERNAL_ONLY
    pub tables: Vec<TenantRbacPostgresRlsTable>,     // data_class: PUBLIC
    pub default_deny_when_policy_missing: bool,      // data_class: PUBLIC
    pub owner_force_rls_required: bool,              // data_class: PUBLIC
    pub bypassrls_role_forbidden: bool,              // data_class: PUBLIC
    pub migration_sql_review_only: bool,             // data_class: PUBLIC
    pub runtime_database_attached: bool,             // data_class: INTERNAL_ONLY
    pub postgres_connection_attached: bool,          // data_class: INTERNAL_ONLY
    pub migration_applied_attached: bool,            // data_class: INTERNAL_ONLY
    pub rls_runtime_verified_attached: bool,         // data_class: INTERNAL_ONLY
    pub durable_storage_runtime_attached: bool,      // data_class: INTERNAL_ONLY
    pub cloud_database_attached: bool,               // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacPostgresRlsStorageError {
    InvalidPlan,
    InvalidTable,
    MissingTenantColumn,
    MissingIdempotencyColumn,
    MissingPayloadColumn,
    MissingSchemaVersionColumn,
    InvalidPrimaryKey,
    InvalidIdempotencyScope,
    MissingRlsPolicy,
    DuplicateTable(String),
    DeletePolicyForbidden,
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_postgres_rls_storage_plan() -> TenantRbacPostgresRlsStoragePlan {
    TenantRbacPostgresRlsStoragePlan {
        plan_name: "tenant-rbac-postgres-rls-storage",
        schema_name: "tenant_rbac",
        runtime_role: RUNTIME_ROLE,
        tenant_context_setting: TENANT_CONTEXT_SETTING,
        tables: tenant_rbac_postgres_rls_tables(),
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
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn tenant_rbac_postgres_rls_tables() -> Vec<TenantRbacPostgresRlsTable> {
    vec![
        table(
            "tenant_rbac_policy_admissions",
            TenantRbacPostgresRecordKind::PolicyAdmission,
            "tenant_rbac_policy_admissions_tenant_rls",
        ),
        table(
            "tenant_rbac_group_close_rollups",
            TenantRbacPostgresRecordKind::GroupCloseRollup,
            "tenant_rbac_group_close_rollups_tenant_rls",
        ),
        table(
            "tenant_rbac_cross_service_workflow_plans",
            TenantRbacPostgresRecordKind::CrossServiceWorkflowPlan,
            "tenant_rbac_cross_service_workflow_plans_tenant_rls",
        ),
        table(
            "tenant_rbac_incident_rollback_plans",
            TenantRbacPostgresRecordKind::IncidentRollbackPlan,
            "tenant_rbac_incident_rollback_plans_tenant_rls",
        ),
        table(
            "tenant_rbac_ops_commands",
            TenantRbacPostgresRecordKind::OpsCommand,
            "tenant_rbac_ops_commands_tenant_rls",
        ),
    ]
}

pub fn validate_tenant_rbac_postgres_rls_storage_plan(
    plan: &TenantRbacPostgresRlsStoragePlan,
) -> Result<(), TenantRbacPostgresRlsStorageError> {
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
        return Err(TenantRbacPostgresRlsStorageError::InvalidPlan);
    }
    if plan.runtime_database_attached
        || plan.postgres_connection_attached
        || plan.migration_applied_attached
        || plan.rls_runtime_verified_attached
        || plan.durable_storage_runtime_attached
        || plan.cloud_database_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(TenantRbacPostgresRlsStorageError::RuntimeAttachmentOverclaim);
    }

    let mut seen_tables = BTreeSet::new();
    for table in &plan.tables {
        validate_table(table)?;
        if !seen_tables.insert(table.table_name.to_owned()) {
            return Err(TenantRbacPostgresRlsStorageError::DuplicateTable(
                table.table_name.to_owned(),
            ));
        }
    }
    Ok(())
}

pub fn render_tenant_rbac_postgres_rls_migration(
    plan: &TenantRbacPostgresRlsStoragePlan,
) -> Result<String, TenantRbacPostgresRlsStorageError> {
    validate_tenant_rbac_postgres_rls_storage_plan(plan)?;
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

fn table(
    table_name: &'static str,
    record_kind: TenantRbacPostgresRecordKind,
    rls_policy_name: &'static str,
) -> TenantRbacPostgresRlsTable {
    TenantRbacPostgresRlsTable {
        table_name,
        record_kind,
        columns: common_columns(),
        primary_key_columns: vec!["tenant_id", "idempotency_key"],
        unique_idempotency_scope_columns: vec!["tenant_id", "idempotency_key"],
        rls_policy_name,
        tenant_context_setting: TENANT_CONTEXT_SETTING,
        enable_row_level_security: true,
        force_row_level_security: true,
        select_policy_required: true,
        insert_policy_required: true,
        update_policy_required: true,
        delete_allowed: false,
        append_only: true,
        payload_encryption_required: true,
    }
}

fn common_columns() -> Vec<TenantRbacPostgresRlsColumn> {
    vec![
        column("tenant_id", "text", "INTERNAL_ONLY"),
        column("idempotency_key", "text", "INTERNAL_ONLY"),
        column("primary_ref", "text", "INTERNAL_ONLY"),
        column("payload_json", "jsonb", "INTERNAL_ONLY"),
        column("payload_data_class", "text", "INTERNAL_ONLY"),
        column("schema_version", "integer", "PUBLIC"),
        column("created_at", "timestamptz", "INTERNAL_ONLY"),
        column("trace_id", "text", "INTERNAL_ONLY"),
        column("audit_evidence_ref", "text", "INTERNAL_ONLY"),
    ]
}

fn column(
    name: &'static str,
    sql_type: &'static str,
    data_class: &'static str,
) -> TenantRbacPostgresRlsColumn {
    TenantRbacPostgresRlsColumn {
        name,
        sql_type,
        required: true,
        data_class,
    }
}

fn validate_table(
    table: &TenantRbacPostgresRlsTable,
) -> Result<(), TenantRbacPostgresRlsStorageError> {
    if !valid_identifier(table.table_name)
        || !valid_identifier(table.rls_policy_name)
        || table.tenant_context_setting != TENANT_CONTEXT_SETTING
        || table.columns.is_empty()
        || !table.enable_row_level_security
        || !table.force_row_level_security
        || !table.select_policy_required
        || !table.insert_policy_required
        || !table.update_policy_required
        || !table.append_only
        || !table.payload_encryption_required
    {
        return Err(TenantRbacPostgresRlsStorageError::InvalidTable);
    }
    if table.delete_allowed {
        return Err(TenantRbacPostgresRlsStorageError::DeletePolicyForbidden);
    }
    require_column(
        table,
        "tenant_id",
        TenantRbacPostgresRlsStorageError::MissingTenantColumn,
    )?;
    require_column(
        table,
        "idempotency_key",
        TenantRbacPostgresRlsStorageError::MissingIdempotencyColumn,
    )?;
    require_column(
        table,
        "payload_json",
        TenantRbacPostgresRlsStorageError::MissingPayloadColumn,
    )?;
    require_column(
        table,
        "schema_version",
        TenantRbacPostgresRlsStorageError::MissingSchemaVersionColumn,
    )?;
    if table.primary_key_columns != ["tenant_id", "idempotency_key"] {
        return Err(TenantRbacPostgresRlsStorageError::InvalidPrimaryKey);
    }
    if table.unique_idempotency_scope_columns != ["tenant_id", "idempotency_key"] {
        return Err(TenantRbacPostgresRlsStorageError::InvalidIdempotencyScope);
    }
    if table.rls_policy_name.is_empty() {
        return Err(TenantRbacPostgresRlsStorageError::MissingRlsPolicy);
    }
    Ok(())
}

fn require_column(
    table: &TenantRbacPostgresRlsTable,
    name: &str,
    error: TenantRbacPostgresRlsStorageError,
) -> Result<(), TenantRbacPostgresRlsStorageError> {
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
    plan: &TenantRbacPostgresRlsStoragePlan,
    table: &TenantRbacPostgresRlsTable,
) -> String {
    let qualified = format!("{}.{}", plan.schema_name, table.table_name);
    let column_sql = table
        .columns
        .iter()
        .map(|column| format!("    {} {} NOT NULL", column.name, column.sql_type))
        .collect::<Vec<_>>()
        .join(",\n");
    let tenant_predicate = format!(
        "tenant_id = current_setting('{}', true)",
        table.tenant_context_setting
    );
    format!(
        "CREATE TABLE IF NOT EXISTS {qualified} (\n{column_sql},\n    PRIMARY KEY (tenant_id, idempotency_key)\n);\nALTER TABLE {qualified} ENABLE ROW LEVEL SECURITY;\nALTER TABLE {qualified} FORCE ROW LEVEL SECURITY;\nCREATE POLICY {policy} ON {qualified} AS RESTRICTIVE FOR ALL TO {role} USING ({predicate}) WITH CHECK ({predicate});\nCOMMENT ON TABLE {qualified} IS 'Tenant RBAC review-only Postgres/RLS storage plan; migrations are not applied by this crate.';\n",
        policy = table.rls_policy_name,
        role = plan.runtime_role,
        predicate = tenant_predicate,
    )
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
