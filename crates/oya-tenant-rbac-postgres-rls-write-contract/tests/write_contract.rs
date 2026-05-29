use oya_tenant_rbac_postgres_rls_storage::TenantRbacPostgresRecordKind;
use oya_tenant_rbac_postgres_rls_write_contract::{
    TenantRbacPostgresRlsWriteContractError, postgres_rls_write_contract_doc_urls,
    tenant_rbac_postgres_rls_write_contract, validate_tenant_rbac_postgres_rls_write_contract,
};

#[test]
fn postgres_rls_write_contract_validates_controls_and_nonclaims() {
    let contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");
    validate_tenant_rbac_postgres_rls_write_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "tenant-rbac-postgres-rls-write-contract"
    );
    assert_eq!(contract.service_name, "tenant-rbac");
    assert_eq!(contract.schema_name, "tenant_rbac");
    assert_eq!(contract.runtime_role, "tenant_rbac_runtime");
    assert_eq!(contract.tenant_context_setting, "app.tenant_id");
    assert_eq!(contract.storage_plan_table_count, 5);
    assert_eq!(contract.statements.len(), 5);
    assert!(contract.official_docs_required);
    assert!(contract.set_local_tenant_context_required);
    assert!(contract.parameterized_insert_required);
    assert!(contract.idempotency_conflict_do_nothing_required);
    assert!(contract.tenant_scoped_readback_required);
    assert!(contract.schema_version_return_required);
    assert!(contract.delete_statement_forbidden);
    assert!(contract.review_only_contract);
    assert!(!contract.database_connection_attached);
    assert!(!contract.prepared_statement_runtime_attached);
    assert!(!contract.write_runtime_attached);
    assert!(!contract.durable_storage_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn postgres_rls_write_contract_covers_storage_plan_tables_and_docs() {
    let contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");
    let tables = contract
        .statements
        .iter()
        .map(|statement| statement.table_name)
        .collect::<std::collections::BTreeSet<_>>();

    for table in [
        "tenant_rbac_policy_admissions",
        "tenant_rbac_group_close_rollups",
        "tenant_rbac_cross_service_workflow_plans",
        "tenant_rbac_incident_rollback_plans",
        "tenant_rbac_ops_commands",
    ] {
        assert!(tables.contains(table), "missing {table}");
    }
    assert!(contract.statements.iter().any(|statement| {
        statement.record_kind == TenantRbacPostgresRecordKind::CrossServiceWorkflowPlan
            && statement.table_name == "tenant_rbac_cross_service_workflow_plans"
    }));

    let docs = postgres_rls_write_contract_doc_urls(&contract);
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/sql-insert.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/sql-set.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/ddl-rowsecurity.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/libpq-exec.html"));
}

#[test]
fn postgres_rls_write_contract_preserves_parameterized_tenant_scoped_sql() {
    let contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");

    assert!(contract.statements.iter().all(|statement| {
        statement.tenant_context_sql == "SET LOCAL app.tenant_id = $1"
            && statement.insert_sql.starts_with("INSERT INTO tenant_rbac.")
            && statement
                .insert_sql
                .contains("VALUES ($1, $2, $3, $4::jsonb, $5, $6, statement_timestamp(), $7, $8)")
            && statement
                .insert_sql
                .contains("ON CONFLICT (tenant_id, idempotency_key) DO NOTHING")
            && statement
                .select_by_idempotency_sql
                .ends_with("WHERE tenant_id = $1 AND idempotency_key = $2")
            && statement.insert_parameter_order
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
            && statement.select_parameter_order == ["tenant_id", "idempotency_key"]
            && statement.uses_set_local_tenant_context
            && statement.uses_parameterized_values
            && statement.uses_on_conflict_do_nothing
            && statement.select_scoped_by_tenant_and_idempotency
            && statement.returns_schema_version
            && statement.delete_statement_forbidden
            && !statement.runtime_execution_attached
    }));
}

#[test]
fn postgres_rls_write_contract_rejects_missing_duplicate_and_doc_drift() {
    let mut contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");
    contract.statements.truncate(2);
    assert_eq!(
        validate_tenant_rbac_postgres_rls_write_contract(&contract),
        Err(TenantRbacPostgresRlsWriteContractError::MissingStatements)
    );

    let mut contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");
    contract.statements[1] = contract.statements[0].clone();
    assert_eq!(
        validate_tenant_rbac_postgres_rls_write_contract(&contract),
        Err(TenantRbacPostgresRlsWriteContractError::DuplicateStatement(
            "tenant_rbac_policy_admissions".to_owned()
        ))
    );

    let mut contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");
    contract.statements[0].official_doc_urls[0] = "https://example.com/postgres";
    assert_eq!(
        validate_tenant_rbac_postgres_rls_write_contract(&contract),
        Err(TenantRbacPostgresRlsWriteContractError::InvalidOfficialDocUrl)
    );
}

#[test]
fn postgres_rls_write_contract_rejects_unparameterized_sql_missing_controls_and_overclaims() {
    let mut contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");
    contract.statements[0].insert_sql = contract.statements[0]
        .insert_sql
        .replace("$1", "'tenant-inline'");
    assert_eq!(
        validate_tenant_rbac_postgres_rls_write_contract(&contract),
        Err(TenantRbacPostgresRlsWriteContractError::InvalidInsertSql)
    );

    let mut contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");
    contract.parameterized_insert_required = false;
    assert_eq!(
        validate_tenant_rbac_postgres_rls_write_contract(&contract),
        Err(
            TenantRbacPostgresRlsWriteContractError::MissingRequiredControl(
                "parameterized_insert_required"
            )
        )
    );

    let mut contract = tenant_rbac_postgres_rls_write_contract().expect("contract builds");
    contract.write_runtime_attached = true;
    assert_eq!(
        validate_tenant_rbac_postgres_rls_write_contract(&contract),
        Err(TenantRbacPostgresRlsWriteContractError::RuntimeAttachmentOverclaim)
    );
}
