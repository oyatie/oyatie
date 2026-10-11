//! Env-gated LIVE Postgres integration tests for the durable SCIM adapters.
//! They run ONLY when `OYATIE_BACKBONE_LIVE_POSTGRES` is truthy AND disposable
//! database URLs are supplied; otherwise every test returns cleanly so the
//! always-on lane stays database-free.
//!
//! Required environment when enabled:
//! - `OYATIE_BACKBONE_LIVE_POSTGRES`   = 1|true|yes|on
//! - `OYATIE_BACKBONE_POSTGRES_URL`    = SETUP superuser/owner URL (DDL + grants)
//! - `OYATIE_BACKBONE_POSTGRES_APP_URL`= APP runtime URL (NON-superuser,
//!   NON-BYPASSRLS role; the adapter's role)
//!
//! What they prove against a real database:
//! 1. RLS cross-tenant denial — tenant A cannot read or overwrite tenant B's
//!    SCIM user rows.
//! 2. BYPASSRLS absent — the app role carries neither rolsuper nor
//!    rolbypassrls.
//! 3. Per-tenant userName uniqueness round-trip + tenant-scoped CRUD.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_identity_scim_store_postgres::passkeys::{
    ChallengeWindow, PasskeyStoreError, PgPasskeyStore,
};
use iam_identity_scim_store_postgres::{
    PASSKEY_AUTHENTICATION_SQL, PgScimGroupStore, PgScimUserStore, SCHEMA_NAME, USERS_TABLE,
};
use iam_identity_user_session::UserMembershipStore;
use iam_scim_server_kernel::{
    Group, GroupStore, Meta, OyatieExtension, ScimId, TenantId, User, UserStore,
};
use shared_postgres_command_kernel::{SET_LOCAL_TENANT_SQL, split_migration_statements};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

const ENABLE_ENV: &str = "OYATIE_BACKBONE_LIVE_POSTGRES";
const SETUP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_URL";
const APP_URL_ENV: &str = "OYATIE_BACKBONE_POSTGRES_APP_URL";
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

fn bound_user(id: &str, user_name: &str, native_user_id: &str, active: bool) -> User {
    let mut user = user(id, user_name);
    user.schemas.push(User::OYATIE_SCHEMA.to_owned());
    user.active = active;
    user.oyatie = Some(OyatieExtension {
        native_user_id: Some(native_user_id.to_owned()),
        roles: Some(vec!["foundry-operator".to_owned()]),
        ..Default::default()
    });
    user
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
    let passkey_migration = PASSKEY_AUTHENTICATION_SQL;
    assert!(
        role_migration.contains(RUNTIME_ROLE),
        "0000 migration must provision the runtime role"
    );
    assert!(
        table_migration.contains(RUNTIME_ROLE),
        "0001 migration must bind the runtime role"
    );
    for migration in [role_migration, table_migration, passkey_migration] {
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

#[path = "live_rls/crud.rs"]
mod crud;
#[path = "live_rls/membership.rs"]
mod membership;
#[path = "live_rls/passkeys.rs"]
mod passkeys;
#[path = "live_rls/rls.rs"]
mod rls;
