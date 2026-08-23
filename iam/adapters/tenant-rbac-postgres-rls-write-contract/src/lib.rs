//! Tenant RBAC Postgres/RLS parameterized write contract.
//!
//! This review-only crate defines the static SQL statement contract that later
//! durable Postgres adapters must use when Tenant RBAC metadata leaves the
//! in-memory reference store. It binds each storage-plan table to a tenant
//! context statement, parameterized INSERT, idempotency-conflict behavior, and
//! tenant-scoped read-back statement. It does not open a database connection,
//! prepare statements, execute SQL, persist records, attach Postgres/RLS runtime,
//! or claim durable storage readiness.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_postgres_rls_storage::{
    TenantRbacPostgresRecordKind, TenantRbacPostgresRlsStorageError,
    TenantRbacPostgresRlsStoragePlan, TenantRbacPostgresRlsTable,
    tenant_rbac_postgres_rls_storage_plan, validate_tenant_rbac_postgres_rls_storage_plan,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_STATEMENT_COUNT: usize = 5;
const CONTRACT_NAME: &str = "tenant-rbac-postgres-rls-write-contract";
const SERVICE_NAME: &str = "tenant-rbac";
const SCHEMA_NAME: &str = "tenant_rbac";
const RUNTIME_ROLE: &str = "tenant_rbac_runtime";
const TENANT_CONTEXT_SETTING: &str = "app.tenant_id";
const SOURCE_STORAGE_PLAN_REF: &str =
    "crates/tenant-rbac-postgres-rls-storage/src/lib.rs::tenant_rbac_postgres_rls_storage_plan";

const INSERT_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-insert.html";
const SET_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-set.html";
const RLS_DOC_URL: &str = "https://www.postgresql.org/docs/current/ddl-rowsecurity.html";
const LIBPQ_DOC_URL: &str = "https://www.postgresql.org/docs/current/libpq-exec.html";

const INSERT_PARAMETER_ORDER: [&str; 8] = [
    "tenant_id",
    "idempotency_key",
    "primary_ref",
    "payload_json",
    "payload_data_class",
    "schema_version",
    "trace_id",
    "audit_evidence_ref",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacPostgresRlsWriteStatement {
    pub table_name: &'static str,                      // data_class: PUBLIC
    pub record_kind: TenantRbacPostgresRecordKind,     // data_class: PUBLIC
    pub tenant_context_sql: String,                    // data_class: INTERNAL_ONLY
    pub insert_sql: String,                            // data_class: INTERNAL_ONLY
    pub select_by_idempotency_sql: String,             // data_class: INTERNAL_ONLY
    pub insert_parameter_order: Vec<&'static str>,     // data_class: INTERNAL_ONLY
    pub select_parameter_order: Vec<&'static str>,     // data_class: INTERNAL_ONLY
    pub official_doc_urls: Vec<&'static str>,          // data_class: PUBLIC
    pub source_storage_plan_ref: &'static str,         // data_class: INTERNAL_ONLY
    pub uses_set_local_tenant_context: bool,           // data_class: PUBLIC
    pub uses_parameterized_values: bool,               // data_class: PUBLIC
    pub uses_on_conflict_do_nothing: bool,             // data_class: PUBLIC
    pub select_scoped_by_tenant_and_idempotency: bool, // data_class: PUBLIC
    pub returns_schema_version: bool,                  // data_class: PUBLIC
    pub delete_statement_forbidden: bool,              // data_class: PUBLIC
    pub runtime_execution_attached: bool,              // data_class: INTERNAL_ONLY
    pub schema_version: u32,                           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacPostgresRlsWriteContract {
    pub contract_name: &'static str,          // data_class: PUBLIC
    pub service_name: &'static str,           // data_class: PUBLIC
    pub schema_name: &'static str,            // data_class: PUBLIC
    pub runtime_role: &'static str,           // data_class: INTERNAL_ONLY
    pub tenant_context_setting: &'static str, // data_class: INTERNAL_ONLY
    pub storage_plan_table_count: usize,      // data_class: PUBLIC
    pub statements: Vec<TenantRbacPostgresRlsWriteStatement>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub set_local_tenant_context_required: bool, // data_class: PUBLIC
    pub parameterized_insert_required: bool,  // data_class: PUBLIC
    pub idempotency_conflict_do_nothing_required: bool, // data_class: PUBLIC
    pub tenant_scoped_readback_required: bool, // data_class: PUBLIC
    pub schema_version_return_required: bool, // data_class: PUBLIC
    pub delete_statement_forbidden: bool,     // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub database_connection_attached: bool,   // data_class: INTERNAL_ONLY
    pub prepared_statement_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub write_runtime_attached: bool,         // data_class: INTERNAL_ONLY
    pub durable_storage_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacPostgresRlsWriteContractError {
    StoragePlan(TenantRbacPostgresRlsStorageError),
    InvalidContractName,
    InvalidServiceName,
    InvalidSchemaName,
    InvalidRuntimeRole,
    InvalidTenantContextSetting,
    InvalidStoragePlanTableCount,
    MissingStatements,
    DuplicateStatement(String),
    MissingStoragePlanTable(String),
    InvalidTableName,
    InvalidTenantContextSql,
    InvalidInsertSql,
    InvalidSelectSql,
    InvalidParameterOrder,
    InvalidOfficialDocUrl,
    InvalidSourceStoragePlanRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_postgres_rls_write_contract()
-> Result<TenantRbacPostgresRlsWriteContract, TenantRbacPostgresRlsWriteContractError> {
    let storage_plan = tenant_rbac_postgres_rls_storage_plan();
    validate_tenant_rbac_postgres_rls_storage_plan(&storage_plan)
        .map_err(TenantRbacPostgresRlsWriteContractError::StoragePlan)?;

    Ok(TenantRbacPostgresRlsWriteContract {
        contract_name: CONTRACT_NAME,
        service_name: SERVICE_NAME,
        schema_name: storage_plan.schema_name,
        runtime_role: storage_plan.runtime_role,
        tenant_context_setting: storage_plan.tenant_context_setting,
        storage_plan_table_count: storage_plan.tables.len(),
        statements: storage_plan
            .tables
            .iter()
            .map(|table| write_statement(&storage_plan, table))
            .collect(),
        official_docs_required: true,
        set_local_tenant_context_required: true,
        parameterized_insert_required: true,
        idempotency_conflict_do_nothing_required: true,
        tenant_scoped_readback_required: true,
        schema_version_return_required: true,
        delete_statement_forbidden: true,
        review_only_contract: true,
        database_connection_attached: false,
        prepared_statement_runtime_attached: false,
        write_runtime_attached: false,
        durable_storage_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_tenant_rbac_postgres_rls_write_contract(
    contract: &TenantRbacPostgresRlsWriteContract,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    let storage_plan = tenant_rbac_postgres_rls_storage_plan();
    validate_tenant_rbac_postgres_rls_storage_plan(&storage_plan)
        .map_err(TenantRbacPostgresRlsWriteContractError::StoragePlan)?;

    validate_slug(
        contract.contract_name,
        TenantRbacPostgresRlsWriteContractError::InvalidContractName,
    )?;
    if contract.service_name != SERVICE_NAME {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidServiceName);
    }
    if contract.schema_name != SCHEMA_NAME {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidSchemaName);
    }
    if contract.runtime_role != RUNTIME_ROLE {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidRuntimeRole);
    }
    if contract.tenant_context_setting != TENANT_CONTEXT_SETTING {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidTenantContextSetting);
    }
    if contract.storage_plan_table_count != storage_plan.tables.len() {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidStoragePlanTableCount);
    }
    if contract.statements.len() < MIN_STATEMENT_COUNT || contract.schema_version != SCHEMA_VERSION
    {
        return Err(TenantRbacPostgresRlsWriteContractError::MissingStatements);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_statements(contract, &storage_plan)?;
    Ok(())
}

pub fn postgres_rls_write_contract_doc_urls(
    contract: &TenantRbacPostgresRlsWriteContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for statement in &contract.statements {
        for url in &statement.official_doc_urls {
            docs.insert(*url);
        }
    }
    docs.into_iter().collect()
}

fn write_statement(
    storage_plan: &TenantRbacPostgresRlsStoragePlan,
    table: &TenantRbacPostgresRlsTable,
) -> TenantRbacPostgresRlsWriteStatement {
    let qualified = format!("{}.{}", storage_plan.schema_name, table.table_name);
    TenantRbacPostgresRlsWriteStatement {
        table_name: table.table_name,
        record_kind: table.record_kind,
        tenant_context_sql: format!("SET LOCAL {} = $1", storage_plan.tenant_context_setting),
        insert_sql: format!(
            "INSERT INTO {qualified} (tenant_id, idempotency_key, primary_ref, payload_json, payload_data_class, schema_version, created_at, trace_id, audit_evidence_ref) VALUES ($1, $2, $3, $4::jsonb, $5, $6, statement_timestamp(), $7, $8) ON CONFLICT (tenant_id, idempotency_key) DO NOTHING RETURNING tenant_id, idempotency_key, schema_version, created_at"
        ),
        select_by_idempotency_sql: format!(
            "SELECT tenant_id, idempotency_key, primary_ref, payload_json, payload_data_class, schema_version, created_at, trace_id, audit_evidence_ref FROM {qualified} WHERE tenant_id = $1 AND idempotency_key = $2"
        ),
        insert_parameter_order: INSERT_PARAMETER_ORDER.to_vec(),
        select_parameter_order: vec!["tenant_id", "idempotency_key"],
        official_doc_urls: vec![INSERT_DOC_URL, SET_DOC_URL, RLS_DOC_URL, LIBPQ_DOC_URL],
        source_storage_plan_ref: SOURCE_STORAGE_PLAN_REF,
        uses_set_local_tenant_context: true,
        uses_parameterized_values: true,
        uses_on_conflict_do_nothing: true,
        select_scoped_by_tenant_and_idempotency: true,
        returns_schema_version: true,
        delete_statement_forbidden: true,
        runtime_execution_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_required_controls(
    contract: &TenantRbacPostgresRlsWriteContract,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    for control in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.set_local_tenant_context_required,
            "set_local_tenant_context_required",
        ),
        (
            contract.parameterized_insert_required,
            "parameterized_insert_required",
        ),
        (
            contract.idempotency_conflict_do_nothing_required,
            "idempotency_conflict_do_nothing_required",
        ),
        (
            contract.tenant_scoped_readback_required,
            "tenant_scoped_readback_required",
        ),
        (
            contract.schema_version_return_required,
            "schema_version_return_required",
        ),
        (
            contract.delete_statement_forbidden,
            "delete_statement_forbidden",
        ),
        (contract.review_only_contract, "review_only_contract"),
    ] {
        require_control(control.0, control.1)?;
    }
    Ok(())
}

fn validate_nonclaims(
    contract: &TenantRbacPostgresRlsWriteContract,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    if contract.database_connection_attached
        || contract.prepared_statement_runtime_attached
        || contract.write_runtime_attached
        || contract.durable_storage_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(TenantRbacPostgresRlsWriteContractError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_statements(
    contract: &TenantRbacPostgresRlsWriteContract,
    storage_plan: &TenantRbacPostgresRlsStoragePlan,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    let storage_tables = storage_plan
        .tables
        .iter()
        .map(|table| table.table_name)
        .collect::<BTreeSet<_>>();
    let mut seen_statements = BTreeSet::new();
    for statement in &contract.statements {
        validate_statement(statement, &storage_tables)?;
        if !seen_statements.insert(statement.table_name) {
            return Err(TenantRbacPostgresRlsWriteContractError::DuplicateStatement(
                statement.table_name.to_owned(),
            ));
        }
    }
    for table_name in storage_tables {
        if !seen_statements.contains(table_name) {
            return Err(
                TenantRbacPostgresRlsWriteContractError::MissingStoragePlanTable(
                    table_name.to_owned(),
                ),
            );
        }
    }
    Ok(())
}

fn validate_statement(
    statement: &TenantRbacPostgresRlsWriteStatement,
    storage_tables: &BTreeSet<&'static str>,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    if !storage_tables.contains(statement.table_name) || !valid_identifier(statement.table_name) {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidTableName);
    }
    validate_sql_text(&statement.tenant_context_sql)?;
    validate_sql_text(&statement.insert_sql)?;
    validate_sql_text(&statement.select_by_idempotency_sql)?;
    if statement.tenant_context_sql != "SET LOCAL app.tenant_id = $1" {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidTenantContextSql);
    }
    validate_insert_sql(statement)?;
    validate_select_sql(statement)?;
    if statement.insert_parameter_order != INSERT_PARAMETER_ORDER
        || statement.select_parameter_order != ["tenant_id", "idempotency_key"]
    {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidParameterOrder);
    }
    for url in &statement.official_doc_urls {
        validate_doc_url(url)?;
    }
    validate_prefixed_ref(
        statement.source_storage_plan_ref,
        "crates/tenant-rbac-postgres-rls-storage/",
        TenantRbacPostgresRlsWriteContractError::InvalidSourceStoragePlanRef,
    )?;
    for control in [
        (
            statement.uses_set_local_tenant_context,
            "statement_uses_set_local_tenant_context",
        ),
        (
            statement.uses_parameterized_values,
            "statement_uses_parameterized_values",
        ),
        (
            statement.uses_on_conflict_do_nothing,
            "statement_uses_on_conflict_do_nothing",
        ),
        (
            statement.select_scoped_by_tenant_and_idempotency,
            "statement_select_scoped_by_tenant_and_idempotency",
        ),
        (
            statement.returns_schema_version,
            "statement_returns_schema_version",
        ),
        (
            statement.delete_statement_forbidden,
            "statement_delete_statement_forbidden",
        ),
    ] {
        require_control(control.0, control.1)?;
    }
    if statement.runtime_execution_attached || statement.schema_version != SCHEMA_VERSION {
        return Err(TenantRbacPostgresRlsWriteContractError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_insert_sql(
    statement: &TenantRbacPostgresRlsWriteStatement,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    let expected_prefix = format!("INSERT INTO tenant_rbac.{} ", statement.table_name);
    if !statement.insert_sql.starts_with(&expected_prefix)
        || !statement
            .insert_sql
            .contains("VALUES ($1, $2, $3, $4::jsonb, $5, $6, statement_timestamp(), $7, $8)")
        || !statement
            .insert_sql
            .contains("ON CONFLICT (tenant_id, idempotency_key) DO NOTHING")
        || !statement
            .insert_sql
            .contains("RETURNING tenant_id, idempotency_key, schema_version, created_at")
        || statement
            .insert_sql
            .to_ascii_uppercase()
            .contains(" DELETE ")
    {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidInsertSql);
    }
    Ok(())
}

fn validate_select_sql(
    statement: &TenantRbacPostgresRlsWriteStatement,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    let expected_from = format!(" FROM tenant_rbac.{} ", statement.table_name);
    if !statement
        .select_by_idempotency_sql
        .starts_with("SELECT tenant_id, idempotency_key")
        || !statement.select_by_idempotency_sql.contains(&expected_from)
        || !statement
            .select_by_idempotency_sql
            .ends_with("WHERE tenant_id = $1 AND idempotency_key = $2")
        || statement
            .select_by_idempotency_sql
            .to_ascii_uppercase()
            .contains(" DELETE ")
    {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidSelectSql);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    if !matches!(
        url,
        INSERT_DOC_URL | SET_DOC_URL | RLS_DOC_URL | LIBPQ_DOC_URL
    ) {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: TenantRbacPostgresRlsWriteContractError,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || value
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(error);
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && !has_unsafe_text(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn validate_sql_text(sql: &str) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    if sql.trim() != sql
        || sql.is_empty()
        || sql.contains(';')
        || sql.contains("--")
        || sql.contains("/*")
        || sql.contains("*/")
        || sql.to_ascii_lowercase().contains("password")
        || sql.to_ascii_lowercase().contains("secret")
        || sql.to_ascii_lowercase().contains("credential")
    {
        return Err(TenantRbacPostgresRlsWriteContractError::InvalidInsertSql);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: TenantRbacPostgresRlsWriteContractError,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), TenantRbacPostgresRlsWriteContractError> {
    if value {
        Ok(())
    } else {
        Err(TenantRbacPostgresRlsWriteContractError::MissingRequiredControl(control))
    }
}

fn has_unsafe_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.trim() != value
        || value.contains("..")
        || value.contains('\\')
        || value.contains('/')
        || value.chars().any(char::is_control)
        || lower.contains("pending")
        || lower.contains("todo")
        || lower.contains("fixme")
        || lower.contains("placeholder")
        || lower.contains("mock")
        || lower.contains("stub")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("credential")
        || lower.contains("api_key")
        || lower.contains("bearer")
}

fn has_unsafe_ref_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.trim() != value
        || value.contains("..")
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || lower.contains("pending")
        || lower.contains("todo")
        || lower.contains("fixme")
        || lower.contains("placeholder")
        || lower.contains("mock")
        || lower.contains("stub")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("credential")
        || lower.contains("api_key")
        || lower.contains("bearer")
}
