//! Integration-level acceptance for the SCIM identity Postgres/RLS plan.
//!
//! Exercised strictly through the public API (a future durable adapter's view),
//! proving the slice obligations as schema/statement SHAPE — hermetically, with
//! no database connection: CRUD round-trip across the full store-port surface,
//! tenant isolation (RLS denies cross-tenant), and per-tenant userName
//! uniqueness.

use iam_identity_scim_store_postgres::{
    ScimPostgresError, ScimRecordKind, render_scim_postgres_migration, scim_postgres_storage_plan,
    scim_postgres_write_statements, validate_scim_postgres_storage_plan,
};

#[test]
fn plan_validates_and_renders_review_only_migration() {
    let plan = scim_postgres_storage_plan();
    validate_scim_postgres_storage_plan(&plan).expect("plan validates");
    let sql = render_scim_postgres_migration(&plan).expect("migration renders");

    assert!(sql.contains("CREATE SCHEMA IF NOT EXISTS identity_scim"));
    assert!(sql.contains("ENABLE ROW LEVEL SECURITY"));
    assert!(sql.contains("FORCE ROW LEVEL SECURITY"));
    assert!(sql.contains("AS RESTRICTIVE FOR ALL"));
    assert!(!sql.contains("DROP "));
    assert!(!sql.contains("TRUNCATE"));
}

#[test]
fn rls_policy_denies_cross_tenant_by_construction() {
    let plan = scim_postgres_storage_plan();
    let sql = render_scim_postgres_migration(&plan).expect("migration renders");
    for table in &plan.tables {
        let qualified = format!("identity_scim.{}", table.table_name);
        assert!(sql.contains(&format!("ALTER TABLE {qualified} FORCE ROW LEVEL SECURITY")));
        assert!(sql.contains(&format!(
            "CREATE POLICY {} ON {qualified} AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('app.tenant_id', true)) WITH CHECK (tenant_id = current_setting('app.tenant_id', true))",
            table.rls_policy_name
        )));
    }
}

#[test]
fn user_name_uniqueness_is_scoped_per_tenant() {
    let plan = scim_postgres_storage_plan();
    let sql = render_scim_postgres_migration(&plan).expect("migration renders");
    // Two tenants may reuse a userName; one tenant may not (the 409 Uniqueness
    // contract). UNIQUE is on (tenant_id, user_name), never on user_name alone.
    assert!(sql.contains("UNIQUE (tenant_id, user_name)"));
    assert!(!sql.contains("UNIQUE (user_name)"));
}

#[test]
fn crud_round_trip_covers_get_put_list_delete_and_find() {
    let statements = scim_postgres_write_statements().expect("statements build");
    assert_eq!(statements.len(), 2);
    for statement in &statements {
        assert!(statement.insert_sql.contains("INSERT INTO identity_scim."));
        assert!(
            statement
                .insert_sql
                .contains("ON CONFLICT (tenant_id, scim_id) DO UPDATE SET")
        );
        assert!(statement.select_by_id_sql.contains("FROM identity_scim."));
        assert!(
            statement
                .list_by_tenant_sql
                .contains("WHERE tenant_id = $1")
        );
        assert!(
            statement
                .delete_by_id_sql
                .contains("DELETE FROM identity_scim.")
        );
        assert!(statement.insert_sql.contains("::jsonb"));
    }
    let users = statements
        .iter()
        .find(|s| s.record_kind == ScimRecordKind::User)
        .expect("user statement");
    assert!(users.find_by_user_name_sql.is_some());
}

#[test]
fn runtime_overclaim_is_fail_closed() {
    let mut plan = scim_postgres_storage_plan();
    plan.durable_storage_runtime_attached = true;
    assert_eq!(
        validate_scim_postgres_storage_plan(&plan),
        Err(ScimPostgresError::RuntimeAttachmentOverclaim)
    );
}
