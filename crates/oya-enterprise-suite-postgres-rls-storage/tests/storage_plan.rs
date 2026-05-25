use oya_enterprise_suite_postgres_rls_storage::{
    EnterprisePostgresRlsStorageError, enterprise_postgres_rls_storage_plan,
    render_enterprise_postgres_rls_migration, validate_enterprise_postgres_rls_storage_plan,
};

#[test]
fn postgres_rls_storage_plan_covers_suite_tables_with_tenant_rls() {
    let plan = enterprise_postgres_rls_storage_plan();
    validate_enterprise_postgres_rls_storage_plan(&plan).expect("plan validates");

    assert_eq!(plan.schema_name, "enterprise_suite");
    assert_eq!(plan.tables.len(), 5);
    assert!(plan.default_deny_when_policy_missing);
    assert!(plan.owner_force_rls_required);
    assert!(plan.bypassrls_role_forbidden);
    assert!(plan.migration_sql_review_only);
    assert!(!plan.runtime_database_attached);
    assert!(!plan.postgres_connection_attached);
    assert!(!plan.migration_applied_attached);
    assert!(!plan.rls_runtime_verified_attached);
    assert!(!plan.durable_storage_runtime_attached);
    assert!(!plan.cloud_database_attached);

    for table in &plan.tables {
        assert!(table.enable_row_level_security);
        assert!(table.force_row_level_security);
        assert!(table.select_policy_required);
        assert!(table.insert_policy_required);
        assert!(table.update_policy_required);
        assert!(!table.delete_allowed);
        assert!(table.append_only);
        assert!(table.payload_encryption_required);
        assert_eq!(table.primary_key_columns, ["tenant_id", "idempotency_key"]);
        assert_eq!(
            table.unique_idempotency_scope_columns,
            ["tenant_id", "idempotency_key"]
        );
        assert!(
            table
                .columns
                .iter()
                .any(|column| column.name == "tenant_id")
        );
        assert!(
            table
                .columns
                .iter()
                .any(|column| column.name == "payload_json" && column.sql_type == "jsonb")
        );
    }
}

#[test]
fn postgres_rls_storage_plan_renders_review_only_migration_sql() {
    let plan = enterprise_postgres_rls_storage_plan();
    let sql = render_enterprise_postgres_rls_migration(&plan).expect("migration renders");

    assert!(sql.contains("CREATE SCHEMA IF NOT EXISTS enterprise_suite"));
    assert!(
        sql.contains("CREATE TABLE IF NOT EXISTS enterprise_suite.enterprise_policy_admissions")
    );
    assert!(sql.contains("PRIMARY KEY (tenant_id, idempotency_key)"));
    assert!(sql.contains(
        "ALTER TABLE enterprise_suite.enterprise_policy_admissions ENABLE ROW LEVEL SECURITY"
    ));
    assert!(sql.contains(
        "ALTER TABLE enterprise_suite.enterprise_policy_admissions FORCE ROW LEVEL SECURITY"
    ));
    assert!(sql.contains("CREATE POLICY enterprise_policy_admissions_tenant_rls"));
    assert!(sql.contains("AS RESTRICTIVE FOR ALL"));
    assert!(sql.contains("current_setting('app.tenant_id', true)"));
    assert!(sql.contains("WITH CHECK"));
    assert!(!sql.contains("DROP TABLE"));
    assert!(!sql.contains("DELETE FROM"));
}

#[test]
fn postgres_rls_storage_plan_rejects_missing_force_rls_or_tenant_column() {
    let mut plan = enterprise_postgres_rls_storage_plan();
    plan.tables[0].force_row_level_security = false;
    assert_eq!(
        validate_enterprise_postgres_rls_storage_plan(&plan),
        Err(EnterprisePostgresRlsStorageError::InvalidTable)
    );

    let mut plan = enterprise_postgres_rls_storage_plan();
    plan.tables[0]
        .columns
        .retain(|column| column.name != "tenant_id");
    assert_eq!(
        validate_enterprise_postgres_rls_storage_plan(&plan),
        Err(EnterprisePostgresRlsStorageError::MissingTenantColumn)
    );
}

#[test]
fn postgres_rls_storage_plan_rejects_delete_policy_and_runtime_overclaims() {
    let mut plan = enterprise_postgres_rls_storage_plan();
    plan.tables[0].delete_allowed = true;
    assert_eq!(
        validate_enterprise_postgres_rls_storage_plan(&plan),
        Err(EnterprisePostgresRlsStorageError::DeletePolicyForbidden)
    );

    let mut plan = enterprise_postgres_rls_storage_plan();
    plan.migration_applied_attached = true;
    assert_eq!(
        validate_enterprise_postgres_rls_storage_plan(&plan),
        Err(EnterprisePostgresRlsStorageError::RuntimeAttachmentOverclaim)
    );
}

#[test]
fn postgres_rls_storage_plan_rejects_duplicate_tables_and_bad_idempotency_scope() {
    let mut plan = enterprise_postgres_rls_storage_plan();
    let duplicate = plan.tables[0].clone();
    plan.tables.push(duplicate);
    assert!(matches!(
        validate_enterprise_postgres_rls_storage_plan(&plan),
        Err(EnterprisePostgresRlsStorageError::DuplicateTable(_))
    ));

    let mut plan = enterprise_postgres_rls_storage_plan();
    plan.tables[0].unique_idempotency_scope_columns = vec!["idempotency_key"];
    assert_eq!(
        validate_enterprise_postgres_rls_storage_plan(&plan),
        Err(EnterprisePostgresRlsStorageError::InvalidIdempotencyScope)
    );
}
