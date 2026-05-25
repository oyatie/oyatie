//! Enterprise Suite Postgres/RLS transaction and prepared-statement contract.
//!
//! This review-only crate defines the transaction choreography that later
//! durable Postgres adapters must follow after the static RLS storage and write
//! contracts exist: begin an explicit transaction, bind tenant context with a
//! transaction-local `set_config` call, prepare the insert statement, execute it
//! through bound parameters, read back by tenant/idempotency key, commit only
//! after readback evidence, and rollback on any failure. It does not open a
//! database connection, prepare statements at runtime, execute SQL, persist
//! records, attach a durable storage runtime, or emit runtime audit-chain events.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use oya_enterprise_suite_postgres_rls_write_contract::{
    EnterprisePostgresRlsWriteContractError, EnterprisePostgresRlsWriteStatement,
    enterprise_postgres_rls_write_contract, validate_enterprise_postgres_rls_write_contract,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_TRANSACTION_PLAN_COUNT: usize = 5;
const CONTRACT_NAME: &str = "enterprise-suite-postgres-rls-transaction-contract";
const SERVICE_NAME: &str = "enterprise-suite";
const SCHEMA_NAME: &str = "enterprise_suite";
const RUNTIME_ROLE: &str = "enterprise_suite_runtime";
const TENANT_CONTEXT_SETTING: &str = "app.tenant_id";
const SOURCE_WRITE_CONTRACT_REF: &str = "crates/oya-enterprise-suite-postgres-rls-write-contract/src/lib.rs::enterprise_postgres_rls_write_contract";

const BEGIN_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-begin.html";
const COMMIT_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-commit.html";
const ROLLBACK_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-rollback.html";
const PREPARE_DOC_URL: &str = "https://www.postgresql.org/docs/current/sql-prepare.html";
const LIBPQ_EXEC_DOC_URL: &str = "https://www.postgresql.org/docs/current/libpq-exec.html";
const FUNCTIONS_ADMIN_DOC_URL: &str =
    "https://www.postgresql.org/docs/current/functions-admin.html";

const EXECUTION_PARAMETER_ORDER: [&str; 8] = [
    "tenant_id",
    "idempotency_key",
    "primary_ref",
    "payload_json",
    "payload_data_class",
    "schema_version",
    "trace_id",
    "audit_evidence_ref",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EnterprisePostgresRlsTransactionStepKind {
    BeginTransaction,
    BindTransactionLocalTenantContext,
    PrepareInsertStatement,
    ExecutePreparedInsert,
    SelectTenantScopedReadback,
    CommitAfterReadback,
    RollbackOnError,
}

impl EnterprisePostgresRlsTransactionStepKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BeginTransaction => "begin_transaction",
            Self::BindTransactionLocalTenantContext => "bind_transaction_local_tenant_context",
            Self::PrepareInsertStatement => "prepare_insert_statement",
            Self::ExecutePreparedInsert => "execute_prepared_insert",
            Self::SelectTenantScopedReadback => "select_tenant_scoped_readback",
            Self::CommitAfterReadback => "commit_after_readback",
            Self::RollbackOnError => "rollback_on_error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterprisePostgresRlsTransactionStep {
    pub step_kind: EnterprisePostgresRlsTransactionStepKind, // data_class: PUBLIC
    pub operation_ref: &'static str,                         // data_class: INTERNAL_ONLY
    pub sql: String,                                         // data_class: INTERNAL_ONLY
    pub parameter_order: Vec<&'static str>,                  // data_class: INTERNAL_ONLY
    pub official_doc_url: &'static str,                      // data_class: PUBLIC
    pub source_write_contract_ref: &'static str,             // data_class: INTERNAL_ONLY
    pub rollback_required_on_failure: bool,                  // data_class: PUBLIC
    pub runtime_execution_attached: bool,                    // data_class: INTERNAL_ONLY
    pub schema_version: u32,                                 // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterprisePostgresRlsTransactionPlan {
    pub table_name: &'static str,                     // data_class: PUBLIC
    pub prepared_statement_name: String,              // data_class: INTERNAL_ONLY
    pub begin_sql: String,                            // data_class: INTERNAL_ONLY
    pub tenant_context_bind_sql: String,              // data_class: INTERNAL_ONLY
    pub prepare_operation_ref: &'static str,          // data_class: INTERNAL_ONLY
    pub execute_operation_ref: &'static str,          // data_class: INTERNAL_ONLY
    pub prepared_insert_sql: String,                  // data_class: INTERNAL_ONLY
    pub readback_sql: String,                         // data_class: INTERNAL_ONLY
    pub commit_sql: String,                           // data_class: INTERNAL_ONLY
    pub rollback_sql: String,                         // data_class: INTERNAL_ONLY
    pub execution_parameter_order: Vec<&'static str>, // data_class: INTERNAL_ONLY
    pub steps: Vec<EnterprisePostgresRlsTransactionStep>, // data_class: INTERNAL_ONLY
    pub source_write_contract_ref: &'static str,      // data_class: INTERNAL_ONLY
    pub uses_explicit_transaction: bool,              // data_class: PUBLIC
    pub binds_tenant_context_transaction_local: bool, // data_class: PUBLIC
    pub uses_prepared_statement: bool,                // data_class: PUBLIC
    pub executes_with_bound_parameters: bool,         // data_class: PUBLIC
    pub reads_back_by_tenant_and_idempotency: bool,   // data_class: PUBLIC
    pub commits_after_readback: bool,                 // data_class: PUBLIC
    pub rolls_back_on_error: bool,                    // data_class: PUBLIC
    pub forbids_autocommit_write: bool,               // data_class: PUBLIC
    pub runtime_execution_attached: bool,             // data_class: INTERNAL_ONLY
    pub schema_version: u32,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterprisePostgresRlsTransactionContract {
    pub contract_name: &'static str,           // data_class: PUBLIC
    pub service_name: &'static str,            // data_class: PUBLIC
    pub schema_name: &'static str,             // data_class: PUBLIC
    pub runtime_role: &'static str,            // data_class: INTERNAL_ONLY
    pub tenant_context_setting: &'static str,  // data_class: INTERNAL_ONLY
    pub write_contract_statement_count: usize, // data_class: PUBLIC
    pub transaction_plans: Vec<EnterprisePostgresRlsTransactionPlan>, // data_class: INTERNAL_ONLY
    pub official_docs_required: bool,          // data_class: PUBLIC
    pub explicit_transaction_required: bool,   // data_class: PUBLIC
    pub transaction_local_tenant_context_required: bool, // data_class: PUBLIC
    pub prepared_statement_required: bool,     // data_class: PUBLIC
    pub bound_parameter_execution_required: bool, // data_class: PUBLIC
    pub tenant_scoped_readback_required: bool, // data_class: PUBLIC
    pub commit_after_readback_required: bool,  // data_class: PUBLIC
    pub rollback_on_error_required: bool,      // data_class: PUBLIC
    pub autocommit_write_forbidden: bool,      // data_class: PUBLIC
    pub review_only_contract: bool,            // data_class: PUBLIC
    pub database_connection_attached: bool,    // data_class: INTERNAL_ONLY
    pub transaction_runtime_attached: bool,    // data_class: INTERNAL_ONLY
    pub prepared_statement_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub write_runtime_attached: bool,          // data_class: INTERNAL_ONLY
    pub durable_storage_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnterprisePostgresRlsTransactionContractError {
    WriteContract(EnterprisePostgresRlsWriteContractError),
    InvalidContractName,
    InvalidServiceName,
    InvalidSchemaName,
    InvalidRuntimeRole,
    InvalidTenantContextSetting,
    InvalidWriteStatementCount,
    MissingTransactionPlans,
    DuplicateTransactionPlan(String),
    MissingWriteContractTable(String),
    InvalidTableName,
    InvalidPreparedStatementName,
    InvalidTransactionSql,
    InvalidTenantContextBindSql,
    InvalidPreparedInsertSql,
    InvalidReadbackSql,
    InvalidOperationRef,
    InvalidParameterOrder,
    InvalidOfficialDocUrl,
    InvalidSourceWriteContractRef,
    MissingStep(EnterprisePostgresRlsTransactionStepKind),
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn enterprise_postgres_rls_transaction_contract()
-> Result<EnterprisePostgresRlsTransactionContract, EnterprisePostgresRlsTransactionContractError> {
    let write_contract = enterprise_postgres_rls_write_contract()
        .map_err(EnterprisePostgresRlsTransactionContractError::WriteContract)?;
    validate_enterprise_postgres_rls_write_contract(&write_contract)
        .map_err(EnterprisePostgresRlsTransactionContractError::WriteContract)?;

    Ok(EnterprisePostgresRlsTransactionContract {
        contract_name: CONTRACT_NAME,
        service_name: SERVICE_NAME,
        schema_name: SCHEMA_NAME,
        runtime_role: RUNTIME_ROLE,
        tenant_context_setting: TENANT_CONTEXT_SETTING,
        write_contract_statement_count: write_contract.statements.len(),
        transaction_plans: write_contract
            .statements
            .iter()
            .map(transaction_plan)
            .collect(),
        official_docs_required: true,
        explicit_transaction_required: true,
        transaction_local_tenant_context_required: true,
        prepared_statement_required: true,
        bound_parameter_execution_required: true,
        tenant_scoped_readback_required: true,
        commit_after_readback_required: true,
        rollback_on_error_required: true,
        autocommit_write_forbidden: true,
        review_only_contract: true,
        database_connection_attached: false,
        transaction_runtime_attached: false,
        prepared_statement_runtime_attached: false,
        write_runtime_attached: false,
        durable_storage_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_enterprise_postgres_rls_transaction_contract(
    contract: &EnterprisePostgresRlsTransactionContract,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    let write_contract = enterprise_postgres_rls_write_contract()
        .map_err(EnterprisePostgresRlsTransactionContractError::WriteContract)?;
    validate_enterprise_postgres_rls_write_contract(&write_contract)
        .map_err(EnterprisePostgresRlsTransactionContractError::WriteContract)?;

    validate_slug(
        contract.contract_name,
        EnterprisePostgresRlsTransactionContractError::InvalidContractName,
    )?;
    if contract.service_name != SERVICE_NAME {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidServiceName);
    }
    if contract.schema_name != SCHEMA_NAME {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidSchemaName);
    }
    if contract.runtime_role != RUNTIME_ROLE {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidRuntimeRole);
    }
    if contract.tenant_context_setting != TENANT_CONTEXT_SETTING {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidTenantContextSetting);
    }
    if contract.write_contract_statement_count != write_contract.statements.len() {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidWriteStatementCount);
    }
    if contract.transaction_plans.len() < MIN_TRANSACTION_PLAN_COUNT
        || contract.schema_version != SCHEMA_VERSION
    {
        return Err(EnterprisePostgresRlsTransactionContractError::MissingTransactionPlans);
    }
    validate_required_controls(contract)?;
    validate_nonclaims(contract)?;
    validate_transaction_plans(contract, &write_contract.statements)?;
    Ok(())
}

pub fn postgres_rls_transaction_contract_doc_urls(
    contract: &EnterprisePostgresRlsTransactionContract,
) -> Vec<&'static str> {
    let mut docs = BTreeSet::new();
    for plan in &contract.transaction_plans {
        for step in &plan.steps {
            docs.insert(step.official_doc_url);
        }
    }
    docs.into_iter().collect()
}

fn transaction_plan(
    statement: &EnterprisePostgresRlsWriteStatement,
) -> EnterprisePostgresRlsTransactionPlan {
    let prepared_statement_name = format!("enterprise_suite_{}_insert_v1", statement.table_name);
    let tenant_context_bind_sql =
        format!("SELECT set_config('{}', $1, true)", TENANT_CONTEXT_SETTING);
    let steps = vec![
        step(
            EnterprisePostgresRlsTransactionStepKind::BeginTransaction,
            "sql:BEGIN",
            "BEGIN".to_owned(),
            Vec::new(),
            BEGIN_DOC_URL,
            false,
        ),
        step(
            EnterprisePostgresRlsTransactionStepKind::BindTransactionLocalTenantContext,
            "libpq:PQexecParams:set_config",
            tenant_context_bind_sql.clone(),
            vec!["tenant_id"],
            FUNCTIONS_ADMIN_DOC_URL,
            true,
        ),
        step(
            EnterprisePostgresRlsTransactionStepKind::PrepareInsertStatement,
            "libpq:PQprepare",
            statement.insert_sql.clone(),
            EXECUTION_PARAMETER_ORDER.to_vec(),
            PREPARE_DOC_URL,
            true,
        ),
        step(
            EnterprisePostgresRlsTransactionStepKind::ExecutePreparedInsert,
            "libpq:PQexecPrepared",
            prepared_statement_name.clone(),
            EXECUTION_PARAMETER_ORDER.to_vec(),
            LIBPQ_EXEC_DOC_URL,
            true,
        ),
        step(
            EnterprisePostgresRlsTransactionStepKind::SelectTenantScopedReadback,
            "libpq:PQexecParams:readback",
            statement.select_by_idempotency_sql.clone(),
            vec!["tenant_id", "idempotency_key"],
            LIBPQ_EXEC_DOC_URL,
            true,
        ),
        step(
            EnterprisePostgresRlsTransactionStepKind::CommitAfterReadback,
            "sql:COMMIT",
            "COMMIT".to_owned(),
            Vec::new(),
            COMMIT_DOC_URL,
            false,
        ),
        step(
            EnterprisePostgresRlsTransactionStepKind::RollbackOnError,
            "sql:ROLLBACK",
            "ROLLBACK".to_owned(),
            Vec::new(),
            ROLLBACK_DOC_URL,
            true,
        ),
    ];
    EnterprisePostgresRlsTransactionPlan {
        table_name: statement.table_name,
        prepared_statement_name,
        begin_sql: "BEGIN".to_owned(),
        tenant_context_bind_sql,
        prepare_operation_ref: "libpq:PQprepare",
        execute_operation_ref: "libpq:PQexecPrepared",
        prepared_insert_sql: statement.insert_sql.clone(),
        readback_sql: statement.select_by_idempotency_sql.clone(),
        commit_sql: "COMMIT".to_owned(),
        rollback_sql: "ROLLBACK".to_owned(),
        execution_parameter_order: EXECUTION_PARAMETER_ORDER.to_vec(),
        steps,
        source_write_contract_ref: SOURCE_WRITE_CONTRACT_REF,
        uses_explicit_transaction: true,
        binds_tenant_context_transaction_local: true,
        uses_prepared_statement: true,
        executes_with_bound_parameters: true,
        reads_back_by_tenant_and_idempotency: true,
        commits_after_readback: true,
        rolls_back_on_error: true,
        forbids_autocommit_write: true,
        runtime_execution_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn step(
    step_kind: EnterprisePostgresRlsTransactionStepKind,
    operation_ref: &'static str,
    sql: String,
    parameter_order: Vec<&'static str>,
    official_doc_url: &'static str,
    rollback_required_on_failure: bool,
) -> EnterprisePostgresRlsTransactionStep {
    EnterprisePostgresRlsTransactionStep {
        step_kind,
        operation_ref,
        sql,
        parameter_order,
        official_doc_url,
        source_write_contract_ref: SOURCE_WRITE_CONTRACT_REF,
        rollback_required_on_failure,
        runtime_execution_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_required_controls(
    contract: &EnterprisePostgresRlsTransactionContract,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    for control in [
        (contract.official_docs_required, "official_docs_required"),
        (
            contract.explicit_transaction_required,
            "explicit_transaction_required",
        ),
        (
            contract.transaction_local_tenant_context_required,
            "transaction_local_tenant_context_required",
        ),
        (
            contract.prepared_statement_required,
            "prepared_statement_required",
        ),
        (
            contract.bound_parameter_execution_required,
            "bound_parameter_execution_required",
        ),
        (
            contract.tenant_scoped_readback_required,
            "tenant_scoped_readback_required",
        ),
        (
            contract.commit_after_readback_required,
            "commit_after_readback_required",
        ),
        (
            contract.rollback_on_error_required,
            "rollback_on_error_required",
        ),
        (
            contract.autocommit_write_forbidden,
            "autocommit_write_forbidden",
        ),
        (contract.review_only_contract, "review_only_contract"),
    ] {
        require_control(control.0, control.1)?;
    }
    Ok(())
}

fn validate_nonclaims(
    contract: &EnterprisePostgresRlsTransactionContract,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if contract.database_connection_attached
        || contract.transaction_runtime_attached
        || contract.prepared_statement_runtime_attached
        || contract.write_runtime_attached
        || contract.durable_storage_runtime_attached
        || contract.runtime_audit_chain_emission_attached
    {
        return Err(EnterprisePostgresRlsTransactionContractError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_transaction_plans(
    contract: &EnterprisePostgresRlsTransactionContract,
    write_statements: &[EnterprisePostgresRlsWriteStatement],
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    let write_tables = write_statements
        .iter()
        .map(|statement| statement.table_name)
        .collect::<BTreeSet<_>>();
    let mut seen_plans = BTreeSet::new();
    for plan in &contract.transaction_plans {
        validate_transaction_plan(plan, &write_tables)?;
        if !seen_plans.insert(plan.table_name) {
            return Err(
                EnterprisePostgresRlsTransactionContractError::DuplicateTransactionPlan(
                    plan.table_name.to_owned(),
                ),
            );
        }
    }
    for table_name in write_tables {
        if !seen_plans.contains(table_name) {
            return Err(
                EnterprisePostgresRlsTransactionContractError::MissingWriteContractTable(
                    table_name.to_owned(),
                ),
            );
        }
    }
    Ok(())
}

fn validate_transaction_plan(
    plan: &EnterprisePostgresRlsTransactionPlan,
    write_tables: &BTreeSet<&'static str>,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if !write_tables.contains(plan.table_name) || !valid_identifier(plan.table_name) {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidTableName);
    }
    if !valid_identifier(&plan.prepared_statement_name)
        || !plan
            .prepared_statement_name
            .ends_with(&format!("{}_insert_v1", plan.table_name))
    {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidPreparedStatementName);
    }
    validate_transaction_sql(&plan.begin_sql, "BEGIN")?;
    validate_transaction_sql(&plan.commit_sql, "COMMIT")?;
    validate_transaction_sql(&plan.rollback_sql, "ROLLBACK")?;
    validate_tenant_context_bind_sql(&plan.tenant_context_bind_sql)?;
    validate_prepared_insert_sql(plan)?;
    validate_readback_sql(plan)?;
    validate_operation_ref(plan.prepare_operation_ref, "libpq:PQprepare")?;
    validate_operation_ref(plan.execute_operation_ref, "libpq:PQexecPrepared")?;
    if plan.execution_parameter_order != EXECUTION_PARAMETER_ORDER {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidParameterOrder);
    }
    validate_prefixed_ref(
        plan.source_write_contract_ref,
        "crates/oya-enterprise-suite-postgres-rls-write-contract/",
        EnterprisePostgresRlsTransactionContractError::InvalidSourceWriteContractRef,
    )?;
    for control in [
        (
            plan.uses_explicit_transaction,
            "plan_uses_explicit_transaction",
        ),
        (
            plan.binds_tenant_context_transaction_local,
            "plan_binds_tenant_context_transaction_local",
        ),
        (plan.uses_prepared_statement, "plan_uses_prepared_statement"),
        (
            plan.executes_with_bound_parameters,
            "plan_executes_with_bound_parameters",
        ),
        (
            plan.reads_back_by_tenant_and_idempotency,
            "plan_reads_back_by_tenant_and_idempotency",
        ),
        (plan.commits_after_readback, "plan_commits_after_readback"),
        (plan.rolls_back_on_error, "plan_rolls_back_on_error"),
        (
            plan.forbids_autocommit_write,
            "plan_forbids_autocommit_write",
        ),
    ] {
        require_control(control.0, control.1)?;
    }
    if plan.runtime_execution_attached || plan.schema_version != SCHEMA_VERSION {
        return Err(EnterprisePostgresRlsTransactionContractError::RuntimeAttachmentOverclaim);
    }
    validate_steps(plan)
}

fn validate_steps(
    plan: &EnterprisePostgresRlsTransactionPlan,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if plan.steps.len() != 7 {
        return Err(EnterprisePostgresRlsTransactionContractError::MissingTransactionPlans);
    }
    let kinds = plan
        .steps
        .iter()
        .map(|step| step.step_kind)
        .collect::<BTreeSet<_>>();
    for kind in [
        EnterprisePostgresRlsTransactionStepKind::BeginTransaction,
        EnterprisePostgresRlsTransactionStepKind::BindTransactionLocalTenantContext,
        EnterprisePostgresRlsTransactionStepKind::PrepareInsertStatement,
        EnterprisePostgresRlsTransactionStepKind::ExecutePreparedInsert,
        EnterprisePostgresRlsTransactionStepKind::SelectTenantScopedReadback,
        EnterprisePostgresRlsTransactionStepKind::CommitAfterReadback,
        EnterprisePostgresRlsTransactionStepKind::RollbackOnError,
    ] {
        if !kinds.contains(&kind) {
            return Err(EnterprisePostgresRlsTransactionContractError::MissingStep(
                kind,
            ));
        }
    }
    for step in &plan.steps {
        validate_operation_ref_prefix(step.operation_ref)?;
        validate_doc_url(step.official_doc_url)?;
        validate_prefixed_ref(
            step.source_write_contract_ref,
            "crates/oya-enterprise-suite-postgres-rls-write-contract/",
            EnterprisePostgresRlsTransactionContractError::InvalidSourceWriteContractRef,
        )?;
        validate_step_contract(step, plan)?;
        if step.runtime_execution_attached || step.schema_version != SCHEMA_VERSION {
            return Err(EnterprisePostgresRlsTransactionContractError::RuntimeAttachmentOverclaim);
        }
        if matches!(
            step.step_kind,
            EnterprisePostgresRlsTransactionStepKind::BindTransactionLocalTenantContext
                | EnterprisePostgresRlsTransactionStepKind::PrepareInsertStatement
                | EnterprisePostgresRlsTransactionStepKind::ExecutePreparedInsert
                | EnterprisePostgresRlsTransactionStepKind::SelectTenantScopedReadback
                | EnterprisePostgresRlsTransactionStepKind::RollbackOnError
        ) && !step.rollback_required_on_failure
        {
            return Err(
                EnterprisePostgresRlsTransactionContractError::MissingRequiredControl(
                    "step_rollback_required_on_failure",
                ),
            );
        }
    }
    Ok(())
}

fn validate_step_contract(
    step: &EnterprisePostgresRlsTransactionStep,
    plan: &EnterprisePostgresRlsTransactionPlan,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    match step.step_kind {
        EnterprisePostgresRlsTransactionStepKind::BeginTransaction => {
            validate_operation_ref(step.operation_ref, "sql:BEGIN")?;
            if step.sql != plan.begin_sql || step.official_doc_url != BEGIN_DOC_URL {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidTransactionSql);
            }
            validate_transaction_sql(&step.sql, "BEGIN")?;
            if !step.parameter_order.is_empty() {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidParameterOrder);
            }
        }
        EnterprisePostgresRlsTransactionStepKind::BindTransactionLocalTenantContext => {
            validate_operation_ref(step.operation_ref, "libpq:PQexecParams:set_config")?;
            if step.sql != plan.tenant_context_bind_sql
                || step.official_doc_url != FUNCTIONS_ADMIN_DOC_URL
            {
                return Err(
                    EnterprisePostgresRlsTransactionContractError::InvalidTenantContextBindSql,
                );
            }
            validate_tenant_context_bind_sql(&step.sql)?;
            if step.parameter_order != ["tenant_id"] {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidParameterOrder);
            }
        }
        EnterprisePostgresRlsTransactionStepKind::PrepareInsertStatement => {
            validate_operation_ref(step.operation_ref, plan.prepare_operation_ref)?;
            if step.sql != plan.prepared_insert_sql || step.official_doc_url != PREPARE_DOC_URL {
                return Err(
                    EnterprisePostgresRlsTransactionContractError::InvalidPreparedInsertSql,
                );
            }
            validate_prepared_insert_sql(plan)?;
            if step.parameter_order != EXECUTION_PARAMETER_ORDER {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidParameterOrder);
            }
        }
        EnterprisePostgresRlsTransactionStepKind::ExecutePreparedInsert => {
            validate_operation_ref(step.operation_ref, plan.execute_operation_ref)?;
            if step.sql != plan.prepared_statement_name
                || step.official_doc_url != LIBPQ_EXEC_DOC_URL
                || !valid_identifier(&step.sql)
            {
                return Err(
                    EnterprisePostgresRlsTransactionContractError::InvalidPreparedStatementName,
                );
            }
            if step.parameter_order != EXECUTION_PARAMETER_ORDER {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidParameterOrder);
            }
        }
        EnterprisePostgresRlsTransactionStepKind::SelectTenantScopedReadback => {
            validate_operation_ref(step.operation_ref, "libpq:PQexecParams:readback")?;
            if step.sql != plan.readback_sql || step.official_doc_url != LIBPQ_EXEC_DOC_URL {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidReadbackSql);
            }
            validate_readback_sql(plan)?;
            if step.parameter_order != ["tenant_id", "idempotency_key"] {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidParameterOrder);
            }
        }
        EnterprisePostgresRlsTransactionStepKind::CommitAfterReadback => {
            validate_operation_ref(step.operation_ref, "sql:COMMIT")?;
            if step.sql != plan.commit_sql || step.official_doc_url != COMMIT_DOC_URL {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidTransactionSql);
            }
            validate_transaction_sql(&step.sql, "COMMIT")?;
            if !step.parameter_order.is_empty() {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidParameterOrder);
            }
        }
        EnterprisePostgresRlsTransactionStepKind::RollbackOnError => {
            validate_operation_ref(step.operation_ref, "sql:ROLLBACK")?;
            if step.sql != plan.rollback_sql || step.official_doc_url != ROLLBACK_DOC_URL {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidTransactionSql);
            }
            validate_transaction_sql(&step.sql, "ROLLBACK")?;
            if !step.parameter_order.is_empty() {
                return Err(EnterprisePostgresRlsTransactionContractError::InvalidParameterOrder);
            }
        }
    }
    Ok(())
}

fn validate_tenant_context_bind_sql(
    sql: &str,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    validate_sql_text(sql)?;
    if sql != "SELECT set_config('app.tenant_id', $1, true)" {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidTenantContextBindSql);
    }
    Ok(())
}

fn validate_prepared_insert_sql(
    plan: &EnterprisePostgresRlsTransactionPlan,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    validate_sql_text(&plan.prepared_insert_sql)?;
    let expected_prefix = format!("INSERT INTO enterprise_suite.{} ", plan.table_name);
    if !plan.prepared_insert_sql.starts_with(&expected_prefix)
        || !plan
            .prepared_insert_sql
            .contains("VALUES ($1, $2, $3, $4::jsonb, $5, $6, statement_timestamp(), $7, $8)")
        || !plan
            .prepared_insert_sql
            .contains("ON CONFLICT (tenant_id, idempotency_key) DO NOTHING")
        || !plan
            .prepared_insert_sql
            .contains("RETURNING tenant_id, idempotency_key, schema_version, created_at")
    {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidPreparedInsertSql);
    }
    Ok(())
}

fn validate_readback_sql(
    plan: &EnterprisePostgresRlsTransactionPlan,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    validate_sql_text(&plan.readback_sql)?;
    let expected_from = format!(" FROM enterprise_suite.{} ", plan.table_name);
    if !plan
        .readback_sql
        .starts_with("SELECT tenant_id, idempotency_key")
        || !plan.readback_sql.contains(&expected_from)
        || !plan
            .readback_sql
            .ends_with("WHERE tenant_id = $1 AND idempotency_key = $2")
    {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidReadbackSql);
    }
    Ok(())
}

fn validate_transaction_sql(
    sql: &str,
    expected: &str,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if sql != expected {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidTransactionSql);
    }
    Ok(())
}

fn validate_operation_ref(
    value: &str,
    expected: &str,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if value != expected || has_unsafe_ref_text(value) {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidOperationRef);
    }
    Ok(())
}

fn validate_operation_ref_prefix(
    value: &str,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if !(value.starts_with("libpq:") || value.starts_with("sql:")) || has_unsafe_ref_text(value) {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidOperationRef);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if !matches!(
        url,
        BEGIN_DOC_URL
            | COMMIT_DOC_URL
            | ROLLBACK_DOC_URL
            | PREPARE_DOC_URL
            | LIBPQ_EXEC_DOC_URL
            | FUNCTIONS_ADMIN_DOC_URL
    ) {
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: EnterprisePostgresRlsTransactionContractError,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
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

fn validate_sql_text(sql: &str) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
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
        return Err(EnterprisePostgresRlsTransactionContractError::InvalidTransactionSql);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: EnterprisePostgresRlsTransactionContractError,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || has_unsafe_ref_text(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), EnterprisePostgresRlsTransactionContractError> {
    if value {
        Ok(())
    } else {
        Err(EnterprisePostgresRlsTransactionContractError::MissingRequiredControl(control))
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
