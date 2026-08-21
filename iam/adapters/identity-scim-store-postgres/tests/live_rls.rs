//! Env-gated LIVE Postgres integration tests for the durable SCIM adapters.
//! They run ONLY when `OYA_BACKBONE_LIVE_POSTGRES` is truthy AND disposable
//! database URLs are supplied; otherwise every test returns cleanly so the
//! always-on lane stays database-free.
//!
//! Required environment when enabled:
//! - `OYA_BACKBONE_LIVE_POSTGRES`   = 1|true|yes|on
//! - `OYA_BACKBONE_POSTGRES_URL`    = SETUP superuser/owner URL (DDL + grants)
//! - `OYA_BACKBONE_POSTGRES_APP_URL`= APP runtime URL (NON-superuser,
//!   NON-BYPASSRLS role; the adapter's role)
//!
//! What they prove against a real database:
//! 1. RLS cross-tenant denial — tenant A cannot read or overwrite tenant B's
//!    SCIM user rows.
//! 2. BYPASSRLS absent — the app role carries neither rolsuper nor
//!    rolbypassrls.
//! 3. Per-tenant userName uniqueness round-trip + tenant-scoped CRUD.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use identity_scim_store_postgres::{PgScimGroupStore, PgScimUserStore, SCHEMA_NAME, USERS_TABLE};
use shared_postgres_command_kernel::{SET_LOCAL_TENANT_SQL, split_migration_statements};
use shared_scim_server_kernel::{Group, GroupStore, Meta, ScimId, TenantId, User, UserStore};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

const ENABLE_ENV: &str = "OYA_BACKBONE_LIVE_POSTGRES";
const SETUP_URL_ENV: &str = "OYA_BACKBONE_POSTGRES_URL";
const APP_URL_ENV: &str = "OYA_BACKBONE_POSTGRES_APP_URL";
const RUNTIME_ROLE: &str = "identity_scim_runtime";
const GROUPS_TABLE: &str = "identity_scim.identity_scim_groups";

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

async fn pool(url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url)
        .await
        .expect("connect to live Postgres")
}

fn user(id: &str, user_name: &str) -> User {
    User {
        schemas: vec![User::CORE_SCHEMA.to_owned()],
        id: ScimId(id.to_owned()),
        external_id: None,
        user_name: user_name.to_owned(),
        name: None,
        display_name: Some(user_name.to_owned()),
        active: true,
        emails: Vec::new(),
        groups: Vec::new(),
        enterprise: None,
        oyatie: None,
        meta: Meta {
            resource_type: "User".to_owned(),
            created: "1970-01-01T00:00:00Z".to_owned(),
            last_modified: "1970-01-01T00:00:00Z".to_owned(),
            location: format!("https://identity.oyatie.com/scim/v2/Users/{id}"),
            version: "W/\"1\"".to_owned(),
        },
    }
}

fn group(id: &str, display_name: &str) -> Group {
    Group {
        schemas: vec![Group::CORE_SCHEMA.to_owned()],
        id: ScimId(id.to_owned()),
        display_name: display_name.to_owned(),
        members: Vec::new(),
        meta: Meta {
            resource_type: "Group".to_owned(),
            created: "1970-01-01T00:00:00Z".to_owned(),
            last_modified: "1970-01-01T00:00:00Z".to_owned(),
            location: format!("https://identity.oyatie.com/scim/v2/Groups/{id}"),
            version: "W/\"1\"".to_owned(),
        },
    }
}

/// Apply the durable schema and make the app role a USAGE-member of the runtime
/// role. The migration SQL is the committed `migrations/0000_runtime_role.sql` +
/// `migrations/0001_*.sql`, applied verbatim IN ORDER so the test path is the
/// production schema byte-for-byte: 0000 provisions the `identity_scim_runtime`
/// role (NOLOGIN, NOBYPASSRLS) + schema + USAGE grant, then 0001 creates the
/// tables, the `TO identity_scim_runtime` RLS policies, and the per-table
/// privilege grants to that role. 0000 MUST precede 0001 because every 0001
/// `TO <role>` clause requires the role to already exist.
///
/// After applying, the app login is GRANTed membership in the runtime role
/// (default INHERIT) — the deploy contract. Membership is what makes the
/// `TO identity_scim_runtime` policies apply to the app login AND makes the
/// shared boot guard's `pg_has_role(current_user, 'identity_scim_runtime',
/// 'USAGE')` check pass; the app login inherits the schema USAGE + table grants
/// transitively, so no direct grant to the app login is needed.
async fn setup_schema(setup: &PgPool, app_role: &str) {
    sqlx::query(&format!("DROP SCHEMA IF EXISTS {SCHEMA_NAME} CASCADE"))
        .execute(setup)
        .await
        .unwrap();
    let role_migration = include_str!("../migrations/0000_runtime_role.sql");
    let table_migration = include_str!("../migrations/0001_identity_scim_store.sql");
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
async fn live_app_role_has_no_bypassrls() {
    if !enabled() {
        return;
    }
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let app = pool(&app_url).await;
    let (name, rolsuper, rolbypassrls) = current_role_flags(&app).await;
    assert!(
        !rolsuper && !rolbypassrls,
        "app role {name} must NOT carry rolsuper/rolbypassrls (RLS would be skipped)"
    );
}

#[tokio::test]
async fn live_scim_tenant_scoped_crud_and_uniqueness() {
    if !enabled() {
        return;
    }
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let store = PgScimUserStore::from_pool(app.clone());
    let alpha = TenantId("alpha".to_owned());
    let beta = TenantId("beta".to_owned());

    store.put(&user("u-1", "alice"), &alpha).await.unwrap();
    // Same userName under a DIFFERENT tenant is allowed (per-tenant uniqueness).
    store.put(&user("u-2", "alice"), &beta).await.unwrap();

    assert_eq!(
        store.get(&alpha, &ScimId("u-1".to_owned())).await,
        Some(user("u-1", "alice"))
    );
    assert_eq!(
        store
            .find_by_user_name(&alpha, "alice")
            .await
            .map(|u| u.id.0),
        Some("u-1".to_owned())
    );
    // Tenant alpha lists only its own user.
    let alpha_users = store.list(&alpha).await;
    assert_eq!(alpha_users.len(), 1);
    assert_eq!(alpha_users[0].id.0, "u-1");
}

#[tokio::test]
async fn live_rls_denies_cross_tenant_read_and_write() {
    if !enabled() {
        return;
    }
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let store = PgScimUserStore::from_pool(app.clone());
    let alpha = TenantId("alpha".to_owned());
    let beta = TenantId("beta".to_owned());
    store.put(&user("a-1", "anna"), &alpha).await.unwrap();
    store.put(&user("b-1", "bob"), &beta).await.unwrap();

    // Cross-tenant read: tenant alpha cannot see tenant beta's row.
    assert_eq!(store.get(&alpha, &ScimId("b-1".to_owned())).await, None);

    // Cross-tenant write: under alpha's GUC, inserting a beta-scoped row is
    // denied by the PERMISSIVE policy's
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
        "INSERT INTO {USERS_TABLE} (tenant_id, scim_id, user_name, external_id, active, payload_json, schema_version, updated_at) VALUES ('beta', 'x-1', 'x', '', true, '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        cross.is_err(),
        "cross-tenant INSERT under tenant alpha's GUC must be denied by RLS"
    );
    let _ = tx.rollback().await;
}

#[tokio::test]
async fn live_rls_denies_cross_tenant_on_groups() {
    if !enabled() {
        return;
    }
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let store = PgScimGroupStore::from_pool(app.clone());
    let alpha = TenantId("alpha".to_owned());
    let beta = TenantId("beta".to_owned());
    store.put(&group("ga-1", "alphas"), &alpha).await.unwrap();
    store.put(&group("gb-1", "betas"), &beta).await.unwrap();

    // Each tenant lists only its own group.
    let alpha_groups = store.list(&alpha).await;
    assert_eq!(alpha_groups.len(), 1);
    assert_eq!(alpha_groups[0].id.0, "ga-1");

    // Cross-tenant read: tenant alpha cannot see tenant beta's group row.
    assert_eq!(store.get(&alpha, &ScimId("gb-1".to_owned())).await, None);

    // Cross-tenant write: under alpha's GUC, inserting a beta-scoped group row
    // is rejected by the PERMISSIVE WITH CHECK (tenant_id = GUC) — the row's
    // tenant_id ('beta') does not equal the session GUC ('alpha').
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("alpha")
        .execute(&mut *tx)
        .await
        .unwrap();
    let cross = sqlx::query(&format!(
        "INSERT INTO {GROUPS_TABLE} (tenant_id, scim_id, display_name, payload_json, schema_version, updated_at) VALUES ('beta', 'gx-1', 'x', '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        cross.is_err(),
        "cross-tenant groups INSERT under alpha's GUC must be denied by the permissive WITH CHECK"
    );
    let _ = tx.rollback().await;
}

#[tokio::test]
async fn live_rls_unset_guc_denies_all_access() {
    if !enabled() {
        return;
    }
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    // Seed a users row and a groups row (each via their own tenant GUC) so the
    // no-GUC SELECTs have something they MUST be denied from seeing.
    let user_store = PgScimUserStore::from_pool(app.clone());
    let group_store = PgScimGroupStore::from_pool(app.clone());
    let gamma = TenantId("gamma".to_owned());
    user_store
        .put(&user("u-g", "ugamma"), &gamma)
        .await
        .unwrap();
    group_store
        .put(&group("g-g", "ggamma"), &gamma)
        .await
        .unwrap();

    // With NO per-tx GUC set, the RESTRICTIVE require_tenant_guc policy hard-
    // denies access: current_setting('oyatie.tenant_id', true) is NULL, so the
    // restrictive USING/WITH CHECK fails and intersects to deny-all. Each table
    // must return 0 rows on SELECT — proving a missing SET_LOCAL_TENANT_SQL can
    // never fall back to an open scan.
    for (table, select_sql) in [
        (
            USERS_TABLE,
            format!("SELECT count(*)::bigint FROM {USERS_TABLE}"),
        ),
        (
            GROUPS_TABLE,
            format!("SELECT count(*)::bigint FROM {GROUPS_TABLE}"),
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
        "INSERT INTO {USERS_TABLE} (tenant_id, scim_id, user_name, external_id, active, payload_json, schema_version, updated_at) VALUES ('gamma', 'u-x', 'x', NULL, true, '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        insert.is_err(),
        "no GUC set: INSERT must be denied by the restrictive require_tenant_guc WITH CHECK"
    );
    let _ = tx.rollback().await;
}
