//! Env-gated LIVE Postgres integration tests for the durable SCIM adapters.
//! They run ONLY when `OYA_BACKBONE_LIVE_POSTGRES` is truthy AND disposable
//! database URLs are supplied; otherwise every test returns cleanly so the
//! always-on lane stays database-free.
//!
//! Required environment when enabled:
//! - `OYA_BACKBONE_LIVE_POSTGRES`   = 1|true|yes|on
//! - `OYA_BACKBONE_POSTGRES_URL`    = SETUP superuser/owner URL (DDL + grants)
//! - `OYA_BACKBONE_POSTGRES_APP_URL`= APP runtime URL (NON-superuser,
//!                                    NON-BYPASSRLS role; the adapter's role)
//!
//! What they prove against a real database:
//! 1. RLS cross-tenant denial — tenant A cannot read or overwrite tenant B's
//!    SCIM user rows.
//! 2. BYPASSRLS absent — the app role carries neither rolsuper nor
//!    rolbypassrls.
//! 3. Per-tenant userName uniqueness round-trip + tenant-scoped CRUD.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_shared_postgres_command_kernel::SET_LOCAL_TENANT_SQL;
use oya_shared_scim_server_kernel::{Meta, ScimId, TenantId, User, UserStore};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use identity_scim_store_postgres::{PgScimUserStore, SCHEMA_NAME, USERS_TABLE};

const ENABLE_ENV: &str = "OYA_BACKBONE_LIVE_POSTGRES";
const SETUP_URL_ENV: &str = "OYA_BACKBONE_POSTGRES_URL";
const APP_URL_ENV: &str = "OYA_BACKBONE_POSTGRES_APP_URL";
const RUNTIME_ROLE: &str = "identity_scim_runtime";
const GROUPS_TABLE: &str = "identity_scim.identity_scim_groups";

fn enabled() -> bool {
    std::env::var(ENABLE_ENV)
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
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

/// Split a migration file into executable statements the way psql does: drop
/// `--` line comments, then split on `;` only OUTSIDE single-quoted string
/// literals. A naive `split(';')` is fragile — a `;` inside a comment or inside
/// a quoted string (e.g. a COMMENT ON ... IS '...; ...') would shatter the
/// statement — so this respects both.
fn split_statements(migration: &str) -> Vec<String> {
    // Strip `--` line comments first (no `--` appears inside our string
    // literals, so a line-prefix scan is sufficient and mirrors psql intent).
    let stripped: String = migration
        .lines()
        .map(|line| match line.find("--") {
            Some(idx) => &line[..idx],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut chars = stripped.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' if in_string && chars.peek() == Some(&'\'') => {
                // Escaped quote ('') inside a string literal.
                current.push(c);
                current.push(chars.next().unwrap());
            }
            '\'' => {
                in_string = !in_string;
                current.push(c);
            }
            ';' if !in_string => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_owned());
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let trailing = current.trim();
    if !trailing.is_empty() {
        statements.push(trailing.to_owned());
    }
    statements
}

async fn setup_schema(setup: &PgPool, app_role: &str) {
    sqlx::query(&format!("DROP SCHEMA IF EXISTS {SCHEMA_NAME} CASCADE"))
        .execute(setup)
        .await
        .unwrap();
    let migration = include_str!("../migrations/0001_identity_scim_store.sql");
    assert!(
        migration.contains(RUNTIME_ROLE),
        "migration must bind the runtime role"
    );
    for sql in split_statements(migration) {
        sqlx::query(&sql).execute(setup).await.unwrap_or_else(|e| {
            panic!("migration statement failed: {sql}\n{e}");
        });
    }
    for sql in [
        format!("GRANT USAGE ON SCHEMA {SCHEMA_NAME} TO {app_role}"),
        format!(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON {USERS_TABLE}, {GROUPS_TABLE} TO {app_role}"
        ),
    ] {
        sqlx::query(&sql).execute(setup).await.unwrap();
    }
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
    // denied by the RESTRICTIVE WITH CHECK policy.
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
