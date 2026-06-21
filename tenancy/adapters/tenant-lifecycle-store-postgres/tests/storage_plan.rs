//! Integration-level acceptance for the tenant lifecycle Postgres/RLS plan.
//!
//! These exercise the crate strictly through its public API (the consumer's
//! view a future durable adapter sees), proving the three slice obligations as
//! schema/statement SHAPE — hermetically, with no database connection:
//! CRUD round-trip, tenant isolation (RLS denies cross-tenant), and
//! idempotency-key replay.

use tenancy_tenant_lifecycle_store_postgres::{
    TenantLifecyclePostgresError, TenantLifecycleRecordKind,
    render_tenant_lifecycle_postgres_migration, tenant_lifecycle_postgres_storage_plan,
    tenant_lifecycle_postgres_write_statements, validate_tenant_lifecycle_postgres_storage_plan,
};

#[test]
fn plan_validates_and_renders_review_only_migration() {
    let plan = tenant_lifecycle_postgres_storage_plan();
    validate_tenant_lifecycle_postgres_storage_plan(&plan).expect("plan validates");
    let sql = render_tenant_lifecycle_postgres_migration(&plan).expect("migration renders");

    assert!(sql.contains("CREATE SCHEMA IF NOT EXISTS tenancy_lifecycle"));
    assert!(sql.contains("ENABLE ROW LEVEL SECURITY"));
    assert!(sql.contains("FORCE ROW LEVEL SECURITY"));
    assert!(sql.contains("AS RESTRICTIVE FOR ALL"));
    // Review-only: never destructive.
    assert!(!sql.contains("DROP "));
    assert!(!sql.contains("DELETE FROM"));
    assert!(!sql.contains("TRUNCATE"));
}

#[test]
fn rls_policy_denies_cross_tenant_by_construction() {
    // RLS isolation proof (SHAPE): every table forces RLS under a RESTRICTIVE
    // policy keyed on the per-transaction tenant setting, so a session bound to
    // tenant A can never see/write tenant B's rows.
    let plan = tenant_lifecycle_postgres_storage_plan();
    let sql = render_tenant_lifecycle_postgres_migration(&plan).expect("migration renders");
    for table in &plan.tables {
        let qualified = format!("tenancy_lifecycle.{}", table.table_name);
        assert!(sql.contains(&format!("ALTER TABLE {qualified} FORCE ROW LEVEL SECURITY")));
        assert!(sql.contains(&format!(
            "CREATE POLICY {} ON {qualified} AS RESTRICTIVE FOR ALL TO tenancy_lifecycle_runtime USING (tenant_id = current_setting('app.tenant_id', true)) WITH CHECK (tenant_id = current_setting('app.tenant_id', true))",
            table.rls_policy_name
        )));
    }
}

#[test]
fn idempotency_key_replay_is_a_tenant_scoped_no_op_insert() {
    let statements = tenant_lifecycle_postgres_write_statements().expect("statements build");
    let applied = statements
        .iter()
        .find(|s| s.record_kind == TenantLifecycleRecordKind::AppliedWrite)
        .expect("applied-writes statement present");

    assert_eq!(applied.tenant_context_sql, "SET LOCAL app.tenant_id = $1");
    assert!(
        applied
            .insert_sql
            .contains("ON CONFLICT (tenant_id, idempotency_key) DO NOTHING")
    );
    assert!(
        applied
            .select_by_primary_key_sql
            .ends_with("WHERE tenant_id = $1 AND idempotency_key = $2")
    );
}

#[test]
fn crud_round_trip_covers_every_record_family() {
    let statements = tenant_lifecycle_postgres_write_statements().expect("statements build");
    let kinds: Vec<_> = statements.iter().map(|s| s.record_kind).collect();
    assert!(kinds.contains(&TenantLifecycleRecordKind::Tenant));
    assert!(kinds.contains(&TenantLifecycleRecordKind::AppliedWrite));
    assert!(kinds.contains(&TenantLifecycleRecordKind::Operation));
    for statement in &statements {
        assert!(
            statement
                .insert_sql
                .contains("INSERT INTO tenancy_lifecycle.")
        );
        assert!(
            statement
                .select_by_primary_key_sql
                .contains("FROM tenancy_lifecycle.")
        );
        assert!(statement.insert_sql.contains("::jsonb"));
    }
}

#[test]
fn runtime_overclaim_is_fail_closed() {
    let mut plan = tenant_lifecycle_postgres_storage_plan();
    plan.cloud_database_attached = true;
    assert_eq!(
        validate_tenant_lifecycle_postgres_storage_plan(&plan),
        Err(TenantLifecyclePostgresError::RuntimeAttachmentOverclaim)
    );
}
