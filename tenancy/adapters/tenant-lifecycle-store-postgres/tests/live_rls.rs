//! Env-gated LIVE Postgres integration tests for the durable tenant-lifecycle
//! adapter. They run ONLY when `OYATIE_BACKBONE_LIVE_POSTGRES` is truthy AND a
//! disposable database URL is supplied; otherwise every test returns cleanly so
//! the always-on lane stays database-free (testing-standards-multilayer: the
//! live tier is opt-in, not skipped silently in CI by accident).
//!
//! Required environment when enabled:
//! - `OYATIE_BACKBONE_LIVE_POSTGRES`   = 1|true|yes|on
//! - `OYATIE_BACKBONE_POSTGRES_URL`    = SETUP superuser/owner URL (DDL + grants)
//! - `OYATIE_BACKBONE_POSTGRES_APP_URL`= APP runtime URL (a NON-superuser,
//!   NON-BYPASSRLS role; the adapter's role)
//!
//! What they prove against a real database:
//! 1. RLS cross-tenant denial — tenant A cannot read or overwrite tenant B's
//!    aggregate / ledger rows (the per-tx GUC scopes every statement).
//! 2. BYPASSRLS absent — the app role carries neither rolsuper nor rolbypassrls
//!    (otherwise RLS is silently skipped).
//! 3. Idempotency replay — the same operation/applied key yields a single
//!    durable effect (ON CONFLICT DO NOTHING).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use shared_platform_contracts_kernel::tenancy::{IsolationPosture, Tenant, TenantLifecycleState};
use shared_postgres_command_kernel::{SET_LOCAL_TENANT_SQL, split_migration_statements};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use tenancy_tenant_lifecycle_kernel::{AppliedWriteRecord, TenantLifecycleStore};
use tenancy_tenant_lifecycle_store_postgres::{
    APPLIED_TABLE, OPERATIONS_TABLE, PgStoreConnectError, PgTenantLifecycleStore, SCHEMA_NAME,
    TENANTS_TABLE,
};

const ENABLE_ENV: &str = "OYATIE_BACKBONE_LIVE_POSTGRES";
const SETUP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_URL";
const APP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_APP_URL";
const RUNTIME_ROLE: &str = "tenancy_lifecycle_runtime";

fn enabled() -> bool {
    std::env::var(ENABLE_ENV)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn require_enabled() {
    assert!(
        enabled(),
        "live test requires OYATIE_BACKBONE_LIVE_POSTGRES=1 (nextest --profile live --run-ignored only)"
    );
}

async fn pool(url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url)
        .await
        .expect("connect to live Postgres")
}

fn tenant(id: &str) -> Tenant {
    Tenant {
        tenant_id: id.to_owned(),
        display_name: format!("Tenant {id}"),
        state: TenantLifecycleState::initial(),
        isolation_posture: IsolationPosture::Pooled,
        cell_id: "cell-live".to_owned(),
        residency_zone: None,
    }
}

/// Apply the durable schema and make the app role a USAGE-member of the runtime
/// role. The migration SQL is the committed `migrations/0000_runtime_role.sql` +
/// `migrations/0001_*.sql`, applied verbatim IN ORDER so the test path is the
/// production schema byte-for-byte: 0000 provisions the `tenancy_lifecycle_runtime`
/// role (NOLOGIN, NOBYPASSRLS) + schema + USAGE grant, then 0001 creates the
/// tables, the `TO tenancy_lifecycle_runtime` RLS policies, and the per-table
/// privilege grants to that role. 0000 MUST precede 0001 because every 0001
/// `TO <role>` clause requires the role to already exist.
///
/// After applying, the app login is GRANTed membership in the runtime role
/// (default INHERIT) — the deploy contract. Membership is what makes the
/// `TO tenancy_lifecycle_runtime` policies apply to the app login AND makes the
/// shared boot guard's `pg_has_role(current_user, 'tenancy_lifecycle_runtime',
/// 'USAGE')` check pass; the app login inherits the schema USAGE + table grants
/// transitively, so no direct grant to the app login is needed.
async fn setup_schema(setup: &PgPool, app_role: &str) {
    sqlx::query(&format!("DROP SCHEMA IF EXISTS {SCHEMA_NAME} CASCADE"))
        .execute(setup)
        .await
        .unwrap();
    let role_migration = include_str!("../migrations/0000_runtime_role.sql");
    let table_migration = include_str!("../migrations/0001_tenant_lifecycle_store.sql");
    // The committed migrations name the runtime role literally; run them verbatim
    // (0000 then 0001) so the RLS policy binds the real production role.
    assert!(
        role_migration.contains(RUNTIME_ROLE),
        "0000 migration must provision the runtime role"
    );
    assert!(
        table_migration.contains(RUNTIME_ROLE),
        "0001 migration must bind the runtime role"
    );
    for migration in [role_migration, table_migration] {
        for sql in split_migration_statements(migration) {
            sqlx::query(&sql).execute(setup).await.unwrap_or_else(|e| {
                panic!("migration statement failed: {sql}\n{e}");
            });
        }
    }
    // Membership grant (default INHERIT): the app login becomes a USAGE-member of
    // the runtime role, inheriting its schema USAGE + table privileges and making
    // the `TO {RUNTIME_ROLE}` policies apply to it.
    sqlx::query(&format!("GRANT {RUNTIME_ROLE} TO \"{app_role}\""))
        .execute(setup)
        .await
        .unwrap();
}

async fn current_role_flags(p: &PgPool) -> (String, bool, bool) {
    let row = sqlx::query(
        "SELECT current_user::text AS name, rolsuper, rolbypassrls FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(p)
    .await
    .unwrap();
    (
        row.try_get("name").unwrap(),
        row.try_get("rolsuper").unwrap(),
        row.try_get("rolbypassrls").unwrap(),
    )
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_app_role_has_no_bypassrls() {
    require_enabled();
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let app = pool(&app_url).await;
    let (name, rolsuper, rolbypassrls) = current_role_flags(&app).await;
    assert!(
        !rolsuper && !rolbypassrls,
        "app role {name} must NOT carry rolsuper/rolbypassrls (RLS would be skipped)"
    );
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_rls_denies_cross_tenant_read_and_write() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let mut store = PgTenantLifecycleStore::from_pool(app.clone());

    // Tenant A writes its own aggregate; tenant B writes its own.
    store
        .put_tenant("tenants/alpha", &tenant("alpha"))
        .await
        .unwrap();
    store
        .put_tenant("tenants/beta", &tenant("beta"))
        .await
        .unwrap();

    // Each tenant sees only its own row.
    assert_eq!(
        store.get_tenant("tenants/alpha").await.unwrap(),
        Some(tenant("alpha"))
    );
    assert_eq!(
        store.get_tenant("tenants/beta").await.unwrap(),
        Some(tenant("beta"))
    );

    // Cross-tenant write attempt: under tenant A's GUC, an INSERT carrying
    // tenant B's id is denied by the PERMISSIVE policy's
    // `WITH CHECK (tenant_id = current_setting('oyatie.tenant_id'))` — the row's
    // tenant_id ('beta') does not equal the session GUC ('alpha'). The
    // RESTRICTIVE require_tenant_guc policy is NOT what denies this case: the GUC
    // IS set (to 'alpha'), so the restrictive WITH CHECK is satisfied; it is the
    // permissive tenant-equality WITH CHECK that rejects the wrong-tenant row.
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("alpha")
        .execute(&mut *tx)
        .await
        .unwrap();
    let cross = sqlx::query(&format!(
        "INSERT INTO {TENANTS_TABLE} (tenant_id, resource_name, display_name, lifecycle_state, payload_json, schema_version, updated_at) VALUES ('beta', 'tenants/beta', 'x', 'x', '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        cross.is_err(),
        "cross-tenant INSERT under tenant alpha's GUC must be denied by RLS"
    );
    let _ = tx.rollback().await;

    // Cross-tenant read: under tenant A's GUC a direct select of tenant B's row
    // returns nothing (USING clause filters it out).
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("alpha")
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*)::bigint FROM {TENANTS_TABLE} WHERE resource_name = 'tenants/beta'"
    ))
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    let _ = tx.rollback().await;
    assert_eq!(count, 0, "tenant alpha must not see tenant beta's row");
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_idempotency_replay_is_single_effect() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let mut store = PgTenantLifecycleStore::from_pool(app.clone());
    let first = AppliedWriteRecord::Create {
        name: "tenants/gamma".to_owned(),
        tenant: tenant("gamma"),
    };
    let replay = AppliedWriteRecord::Create {
        name: "tenants/gamma".to_owned(),
        tenant: tenant("gamma-DIFFERENT"),
    };

    store.put_applied("gamma", "key-1", &first).await.unwrap();
    // A replay under the same key is a no-op: the first record stays durable.
    store.put_applied("gamma", "key-1", &replay).await.unwrap();
    assert_eq!(
        store.get_applied("gamma", "key-1").await.unwrap(),
        Some(first)
    );

    // Exactly one row exists for that key.
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("gamma")
        .execute(&mut *tx)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*)::bigint FROM {APPLIED_TABLE} WHERE idempotency_key = 'key-1'"
    ))
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    let _ = tx.rollback().await;
    assert_eq!(count, 1, "idempotency replay must yield exactly one row");
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_rls_denies_cross_tenant_on_applied_and_operations() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let mut store = PgTenantLifecycleStore::from_pool(app.clone());

    // Seed one applied-write + one operation row for tenant beta (via its own
    // GUC), so we can prove tenant alpha can neither read nor overwrite them.
    let beta_applied = AppliedWriteRecord::Create {
        name: "tenants/beta".to_owned(),
        tenant: tenant("beta"),
    };
    store
        .put_applied("beta", "beta-key", &beta_applied)
        .await
        .unwrap();

    // Cross-tenant READ of applied_writes: under alpha's GUC, beta's row is
    // filtered out by the PERMISSIVE USING (tenant_id = GUC) clause.
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("alpha")
        .execute(&mut *tx)
        .await
        .unwrap();
    let applied_count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*)::bigint FROM {APPLIED_TABLE} WHERE idempotency_key = 'beta-key'"
    ))
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    let _ = tx.rollback().await;
    assert_eq!(
        applied_count, 0,
        "tenant alpha must not see tenant beta's applied-write row"
    );

    // Cross-tenant INSERT into applied_writes: under alpha's GUC, a beta-scoped
    // row is rejected by the PERMISSIVE WITH CHECK (tenant_id = GUC) — the row's
    // tenant_id ('beta') does not equal the session GUC ('alpha').
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("alpha")
        .execute(&mut *tx)
        .await
        .unwrap();
    let cross_applied = sqlx::query(&format!(
        "INSERT INTO {APPLIED_TABLE} (tenant_id, idempotency_key, payload_json, schema_version, created_at) VALUES ('beta', 'x-key', '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        cross_applied.is_err(),
        "cross-tenant applied_writes INSERT under alpha's GUC must be denied by the permissive WITH CHECK"
    );
    let _ = tx.rollback().await;

    // Cross-tenant INSERT into operations: same permissive WITH CHECK denial.
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("alpha")
        .execute(&mut *tx)
        .await
        .unwrap();
    let cross_op = sqlx::query(&format!(
        "INSERT INTO {OPERATIONS_TABLE} (tenant_id, operation_name, operation_seq, payload_json, schema_version, created_at) VALUES ('beta', 'operations/beta-lifecycle-000001', 1, '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        cross_op.is_err(),
        "cross-tenant operations INSERT under alpha's GUC must be denied by the permissive WITH CHECK"
    );
    let _ = tx.rollback().await;
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_rls_unset_guc_denies_all_access() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let mut store = PgTenantLifecycleStore::from_pool(app.clone());
    store
        .put_tenant("tenants/delta", &tenant("delta"))
        .await
        .unwrap();

    // With NO per-tx GUC set, the RESTRICTIVE require_tenant_guc policy hard-
    // denies access: current_setting('oyatie.tenant_id', true) is NULL, so the
    // restrictive USING/WITH CHECK fails and intersects to deny-all. Each table
    // must return 0 rows on SELECT and reject INSERT — proving a missing
    // SET_LOCAL_TENANT_SQL can never fall back to an open scan.
    for (table, select_sql) in [
        (
            TENANTS_TABLE,
            format!("SELECT count(*)::bigint FROM {TENANTS_TABLE}"),
        ),
        (
            APPLIED_TABLE,
            format!("SELECT count(*)::bigint FROM {APPLIED_TABLE}"),
        ),
        (
            OPERATIONS_TABLE,
            format!("SELECT count(*)::bigint FROM {OPERATIONS_TABLE}"),
        ),
    ] {
        let mut tx = app.begin().await.unwrap();
        // Deliberately do NOT set the tenant GUC.
        let count: i64 = sqlx::query_scalar(&select_sql)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        let _ = tx.rollback().await;
        assert_eq!(
            count, 0,
            "no GUC set: SELECT on {table} must return 0 rows (restrictive deny-all)"
        );
    }

    // INSERT with no GUC set is rejected by the restrictive WITH CHECK.
    let mut tx = app.begin().await.unwrap();
    let insert = sqlx::query(&format!(
        "INSERT INTO {TENANTS_TABLE} (tenant_id, resource_name, display_name, lifecycle_state, payload_json, schema_version, updated_at) VALUES ('delta', 'tenants/delta', 'x', 'x', '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        insert.is_err(),
        "no GUC set: INSERT must be denied by the restrictive require_tenant_guc WITH CHECK"
    );
    let _ = tx.rollback().await;
}

/// `assert_rls_enforceable()` must REJECT a bypass-capable role (the setup /
/// superuser pool) with `PgStoreConnectError::RlsUnenforceable`.
/// Aligns with `live_app_role_has_no_bypassrls` — both prove the same
/// invariant from different angles (raw pg_roles query vs. the adapter guard).
#[tokio::test]
#[ignore = "live postgres"]
async fn live_assert_rls_enforceable_rejects_bypass_capable_role() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let setup = pool(&setup_url).await;
    let (setup_role, rolsuper, rolbypassrls) = current_role_flags(&setup).await;
    assert!(
        rolsuper || rolbypassrls,
        "SETUP url must be bypass-capable so this test can prove the reject path (live job uses the postgres superuser); got role {setup_role}"
    );
    let store = PgTenantLifecycleStore::from_pool(setup);
    let result = store.assert_rls_enforceable().await;
    assert!(
        matches!(result, Err(PgStoreConnectError::RlsUnenforceable { .. })),
        "bypass-capable role '{setup_role}' must be rejected by assert_rls_enforceable, \
         got {result:?}"
    );
}

/// `assert_rls_enforceable()` must PASS for the non-privileged app role.
/// Complements `live_assert_rls_enforceable_rejects_bypass_capable_role` by
/// proving the guard is not a false-positive that rejects all roles.
#[tokio::test]
#[ignore = "live postgres"]
async fn live_assert_rls_enforceable_passes_for_app_role() {
    require_enabled();
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let app = pool(&app_url).await;
    let store = PgTenantLifecycleStore::from_pool(app);
    let result = store.assert_rls_enforceable().await;
    assert!(
        result.is_ok(),
        "non-privileged app role must pass assert_rls_enforceable, got {result:?}"
    );
}
