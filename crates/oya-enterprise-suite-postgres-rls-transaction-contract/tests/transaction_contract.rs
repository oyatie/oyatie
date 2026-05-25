use oya_enterprise_suite_postgres_rls_transaction_contract::{
    EnterprisePostgresRlsTransactionContractError, EnterprisePostgresRlsTransactionStepKind,
    enterprise_postgres_rls_transaction_contract, postgres_rls_transaction_contract_doc_urls,
    validate_enterprise_postgres_rls_transaction_contract,
};

#[test]
fn postgres_rls_transaction_contract_validates_controls_and_nonclaims() {
    let contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    validate_enterprise_postgres_rls_transaction_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "enterprise-suite-postgres-rls-transaction-contract"
    );
    assert_eq!(contract.service_name, "enterprise-suite");
    assert_eq!(contract.schema_name, "enterprise_suite");
    assert_eq!(contract.runtime_role, "enterprise_suite_runtime");
    assert_eq!(contract.tenant_context_setting, "app.tenant_id");
    assert_eq!(contract.write_contract_statement_count, 5);
    assert_eq!(contract.transaction_plans.len(), 5);
    assert!(contract.official_docs_required);
    assert!(contract.explicit_transaction_required);
    assert!(contract.transaction_local_tenant_context_required);
    assert!(contract.prepared_statement_required);
    assert!(contract.bound_parameter_execution_required);
    assert!(contract.tenant_scoped_readback_required);
    assert!(contract.commit_after_readback_required);
    assert!(contract.rollback_on_error_required);
    assert!(contract.autocommit_write_forbidden);
    assert!(contract.review_only_contract);
    assert!(!contract.database_connection_attached);
    assert!(!contract.transaction_runtime_attached);
    assert!(!contract.prepared_statement_runtime_attached);
    assert!(!contract.write_runtime_attached);
    assert!(!contract.durable_storage_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn postgres_rls_transaction_contract_covers_tables_steps_and_docs() {
    let contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    let tables = contract
        .transaction_plans
        .iter()
        .map(|plan| plan.table_name)
        .collect::<std::collections::BTreeSet<_>>();

    for table in [
        "enterprise_policy_admissions",
        "enterprise_group_close_rollups",
        "enterprise_cross_product_workflow_plans",
        "enterprise_incident_rollback_plans",
        "enterprise_ops_commands",
    ] {
        assert!(tables.contains(table), "missing {table}");
    }
    for plan in &contract.transaction_plans {
        let kinds = plan
            .steps
            .iter()
            .map(|step| step.step_kind)
            .collect::<std::collections::BTreeSet<_>>();
        for kind in [
            EnterprisePostgresRlsTransactionStepKind::BeginTransaction,
            EnterprisePostgresRlsTransactionStepKind::BindTransactionLocalTenantContext,
            EnterprisePostgresRlsTransactionStepKind::PrepareInsertStatement,
            EnterprisePostgresRlsTransactionStepKind::ExecutePreparedInsert,
            EnterprisePostgresRlsTransactionStepKind::SelectTenantScopedReadback,
            EnterprisePostgresRlsTransactionStepKind::CommitAfterReadback,
            EnterprisePostgresRlsTransactionStepKind::RollbackOnError,
        ] {
            assert!(kinds.contains(&kind), "missing {kind:?}");
        }
    }

    let docs = postgres_rls_transaction_contract_doc_urls(&contract);
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/sql-begin.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/sql-commit.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/sql-rollback.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/sql-prepare.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/libpq-exec.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/functions-admin.html"));
}

#[test]
fn postgres_rls_transaction_contract_preserves_transaction_local_bound_execution() {
    let contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");

    assert!(contract.transaction_plans.iter().all(|plan| {
        plan.begin_sql == "BEGIN"
            && plan.tenant_context_bind_sql == "SELECT set_config('app.tenant_id', $1, true)"
            && plan.prepare_operation_ref == "libpq:PQprepare"
            && plan.execute_operation_ref == "libpq:PQexecPrepared"
            && plan
                .prepared_insert_sql
                .starts_with("INSERT INTO enterprise_suite.")
            && plan
                .prepared_insert_sql
                .contains("VALUES ($1, $2, $3, $4::jsonb, $5, $6, statement_timestamp(), $7, $8)")
            && plan
                .prepared_insert_sql
                .contains("ON CONFLICT (tenant_id, idempotency_key) DO NOTHING")
            && plan
                .readback_sql
                .ends_with("WHERE tenant_id = $1 AND idempotency_key = $2")
            && plan.commit_sql == "COMMIT"
            && plan.rollback_sql == "ROLLBACK"
            && plan.execution_parameter_order
                == [
                    "tenant_id",
                    "idempotency_key",
                    "primary_ref",
                    "payload_json",
                    "payload_data_class",
                    "schema_version",
                    "trace_id",
                    "audit_evidence_ref",
                ]
            && plan.uses_explicit_transaction
            && plan.binds_tenant_context_transaction_local
            && plan.uses_prepared_statement
            && plan.executes_with_bound_parameters
            && plan.reads_back_by_tenant_and_idempotency
            && plan.commits_after_readback
            && plan.rolls_back_on_error
            && plan.forbids_autocommit_write
            && !plan.runtime_execution_attached
    }));
}

#[test]
fn postgres_rls_transaction_contract_rejects_missing_duplicate_and_doc_drift() {
    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.transaction_plans.truncate(2);
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(EnterprisePostgresRlsTransactionContractError::MissingTransactionPlans)
    );

    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.transaction_plans[1] = contract.transaction_plans[0].clone();
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(
            EnterprisePostgresRlsTransactionContractError::DuplicateTransactionPlan(
                "enterprise_policy_admissions".to_owned()
            )
        )
    );

    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.transaction_plans[0].steps[0].official_doc_url = "https://example.com/postgres";
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(EnterprisePostgresRlsTransactionContractError::InvalidOfficialDocUrl)
    );

    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.transaction_plans[0].steps[2].sql = contract.transaction_plans[0].readback_sql.clone();
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(EnterprisePostgresRlsTransactionContractError::InvalidPreparedInsertSql)
    );

    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.transaction_plans[0].steps[4].sql =
        contract.transaction_plans[0].prepared_insert_sql.clone();
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(EnterprisePostgresRlsTransactionContractError::InvalidReadbackSql)
    );
}

#[test]
fn postgres_rls_transaction_contract_rejects_autocommit_context_drift_and_overclaims() {
    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.transaction_plans[0].begin_sql = String::new();
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(EnterprisePostgresRlsTransactionContractError::InvalidTransactionSql)
    );

    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.transaction_plans[0].tenant_context_bind_sql =
        "SET LOCAL app.tenant_id = $1".to_owned();
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(EnterprisePostgresRlsTransactionContractError::InvalidTenantContextBindSql)
    );

    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.prepared_statement_required = false;
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(
            EnterprisePostgresRlsTransactionContractError::MissingRequiredControl(
                "prepared_statement_required"
            )
        )
    );

    let mut contract = enterprise_postgres_rls_transaction_contract().expect("contract builds");
    contract.transaction_runtime_attached = true;
    assert_eq!(
        validate_enterprise_postgres_rls_transaction_contract(&contract),
        Err(EnterprisePostgresRlsTransactionContractError::RuntimeAttachmentOverclaim)
    );
}
