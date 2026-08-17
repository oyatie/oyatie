//! Real async sqlx/Postgres adapters for the SCIM [`UserStore`] / [`GroupStore`]
//! ports (RFC 7643/7644; ADR-0190).
//!
//! These are LIVE durable adapters (not shape plans): each owns a shared
//! `sqlx::PgPool`, and every operation runs in its own transaction that sets the
//! canonical per-transaction tenant GUC FIRST —
//! `SELECT set_config('oyatie.tenant_id', $1, true)` (the shared command
//! kernel's [`SET_LOCAL_TENANT_SQL`]) — before any tenant-scoped statement, so
//! Postgres RLS (RESTRICTIVE, FORCE) isolates every row by `tenant_id`. The
//! runtime role must NOT carry BYPASSRLS or isolation is silently skipped.
//!
//! ## Doctrine: transient adapter behind owned-shaped ports
//!
//! The kernel ports are the OWNED-destination contracts; Postgres is a TRANSIENT
//! adapter behind them. TLS is aws-lc-rs rustls only (ADR-0506; `ring` is
//! forbidden) via the workspace `sqlx` feature `tls-rustls-aws-lc-rs`.
//!
//! ## Tenant scope + uniqueness
//!
//! Every SCIM request is tenant-scoped (`TenantId`). The user table enforces
//! per-tenant `userName` uniqueness (`UNIQUE (tenant_id, user_name)`), the SCIM
//! 409 contract: two tenants may reuse a userName, one tenant may not.
//!
//! ## Testing
//!
//! Always-on tests are hermetic plan-shape tests (assert generated SQL + that
//! the GUC-set statement precedes tenant-scoped statements; no database). Live
//! RLS integration tests live behind `OYA_BACKBONE_LIVE_POSTGRES` (see `tests/`).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::future::Future;
use core::pin::Pin;

use oya_shared_postgres_command_adapter_sqlx::assert_rls_enforceable as assert_rls_enforceable_shared;
use oya_shared_postgres_command_kernel::{RlsEnforceabilityError, SET_LOCAL_TENANT_SQL};
use oya_shared_scim_server_kernel::{
    Group, GroupStore, ScimId, ScimStoreError, TenantId, User, UserStore,
};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

/// Canonical schema + table names. One place so the migration, the runtime
/// statements, and the plan-shape tests cannot drift.
pub const SCHEMA_NAME: &str = "identity_scim";
pub const USERS_TABLE: &str = "identity_scim.identity_scim_users";
pub const GROUPS_TABLE: &str = "identity_scim.identity_scim_groups";

/// The tables whose ENABLE+FORCE RLS the boot guard verifies. This is the SINGLE
/// list the runtime guard passes to `assert_rls_enforceable`; a unit test (see
/// `governed_tables_match_migration_force_rls`) asserts it EXACTLY matches the
/// set of tables `migrations/0001_*.sql` declares FORCE ROW LEVEL SECURITY, so a
/// new FORCE'd table can never silently escape the guard's per-table coverage.
pub const GOVERNED_TABLES: &[&str] = &[USERS_TABLE, GROUPS_TABLE];

/// Current persisted-record schema version.
pub const SCHEMA_VERSION: i32 = 1;

/// The Postgres role that is the subject of the RLS policies in
/// `migrations/0001_identity_scim_store.sql` (every `TO <role>` clause in that
/// migration names this role). The serving connection's `current_user` MUST be
/// this role or a member of it for the tenant-isolation policies to apply.
///
/// MUST match the `TO <role>` clauses in the migration — change both together.
pub const RUNTIME_ROLE: &str = "identity_scim_runtime";

// --- User statements (tenant-scoped) ---------------------------------------
const LIST_USERS_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_users WHERE tenant_id = $1 ORDER BY scim_id ASC";
const GET_USER_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND scim_id = $2";
const FIND_USER_BY_NAME_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND user_name = $2";
const UPSERT_USER_SQL: &str = "INSERT INTO identity_scim.identity_scim_users (tenant_id, scim_id, user_name, external_id, active, payload_json, schema_version, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, now()) ON CONFLICT (tenant_id, scim_id) DO UPDATE SET user_name = EXCLUDED.user_name, external_id = EXCLUDED.external_id, active = EXCLUDED.active, payload_json = EXCLUDED.payload_json, schema_version = EXCLUDED.schema_version, updated_at = now()";
const DELETE_USER_SQL: &str =
    "DELETE FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND scim_id = $2";

// --- Group statements (tenant-scoped) --------------------------------------
const LIST_GROUPS_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 ORDER BY scim_id ASC";
const GET_GROUP_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 AND scim_id = $2";
const UPSERT_GROUP_SQL: &str = "INSERT INTO identity_scim.identity_scim_groups (tenant_id, scim_id, display_name, payload_json, schema_version, updated_at) VALUES ($1, $2, $3, $4, $5, now()) ON CONFLICT (tenant_id, scim_id) DO UPDATE SET display_name = EXCLUDED.display_name, payload_json = EXCLUDED.payload_json, schema_version = EXCLUDED.schema_version, updated_at = now()";
const DELETE_GROUP_SQL: &str =
    "DELETE FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 AND scim_id = $2";

/// Errors specific to constructing the durable adapter (connection-time).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgScimConnectError {
    /// The supplied database URL was empty.
    MissingDatabaseUrl,
    /// The underlying sqlx pool failed to connect.
    Sqlx(String),
    /// The connected role carries `rolsuper` or `rolbypassrls`, which means
    /// Postgres RLS is silently skipped for that role. Serving a multi-tenant
    /// store under such a role is a tenant-isolation risk — the adapter REFUSES
    /// to serve rather than allow bypass.
    ///
    /// Note: this guard is necessary but not sufficient for full tenant
    /// isolation. Full isolation additionally requires that `RUNTIME_ROLE`
    /// exists provisioned with NOBYPASSRLS (the deferred
    /// `0000_runtime_role.sql` follow-up, mirroring oya-data-outbox-adapter-postgres
    /// and tenant-lifecycle-store-postgres).
    RlsUnenforceable { role: String },
    /// The connected role is neither the RLS policy-subject role
    /// (`RUNTIME_ROLE`) nor a member of it. The tenant-isolation policies would
    /// not apply to this role, which — if the migration has applied
    /// `FORCE ROW LEVEL SECURITY` — yields deny-all (a safe fail-closed outage,
    /// not a data leak). The adapter refuses to serve in either case: a
    /// misconfigured role must not slip through silently. Full isolation requires
    /// membership in `RUNTIME_ROLE` AND the migration having applied
    /// `FORCE ROW LEVEL SECURITY` on the relevant tables.
    RlsRoleMismatch { role: String, expected: String },
    /// A governed table is not under `ENABLE` + `FORCE ROW LEVEL SECURITY` (or
    /// is missing entirely): the migration mis-applied or drifted, so tenant
    /// isolation is not forced on that table. The adapter refuses to serve.
    RlsNotForcedOnTable { table: String },
}

impl core::fmt::Display for PgScimConnectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingDatabaseUrl => write!(f, "database url is empty"),
            Self::Sqlx(detail) => write!(f, "sqlx connect failed: {detail}"),
            Self::RlsUnenforceable { role } => write!(
                f,
                "runtime role '{role}' can bypass RLS (rolsuper/rolbypassrls); \
                 refusing to serve a multi-tenant store — full isolation also \
                 requires the role to exist provisioned with NOBYPASSRLS \
                 (see deferred 0000_runtime_role.sql)"
            ),
            Self::RlsRoleMismatch { role, expected } => write!(
                f,
                "connected role '{role}' is not a member of the RLS \
                 policy-subject role '{expected}'; tenant isolation policies \
                 would not apply — refusing to serve"
            ),
            Self::RlsNotForcedOnTable { table } => write!(
                f,
                "governed table '{table}' is not under ENABLE+FORCE ROW LEVEL \
                 SECURITY (migration mis-applied or drifted); tenant isolation \
                 is not forced — refusing to serve"
            ),
        }
    }
}

impl std::error::Error for PgScimConnectError {}

/// Map the shared guard's [`RlsEnforceabilityError`] into this adapter's own
/// connect-error vocabulary, preserving the pre-extraction fail-closed contract:
/// - `Unenforceable` → [`PgScimConnectError::RlsUnenforceable`];
/// - `RoleMismatch` → [`PgScimConnectError::RlsRoleMismatch`];
/// - `RoleSwitchInEffect` → [`PgScimConnectError::Sqlx`] (same fatal `Sqlx`
///   variant as before; the detail string now comes from the shared kernel Display);
/// - `ProbeFailed` → [`PgScimConnectError::Sqlx`] (same fatal `Sqlx` variant
///   preserved; detail string sourced from the shared kernel Display, not the prior
///   adapter-local wording);
/// - `RlsNotForced` / `GovernedTableMissing` → the new
///   [`PgScimConnectError::RlsNotForcedOnTable`] (the #799 FORCE-RLS hardening).
impl From<RlsEnforceabilityError> for PgScimConnectError {
    fn from(error: RlsEnforceabilityError) -> Self {
        match error {
            RlsEnforceabilityError::Unenforceable { role } => Self::RlsUnenforceable { role },
            RlsEnforceabilityError::RoleMismatch { role, expected } => {
                Self::RlsRoleMismatch { role, expected }
            }
            RlsEnforceabilityError::RoleSwitchInEffect { .. }
            | RlsEnforceabilityError::ProbeFailed { .. } => Self::Sqlx(error.to_string()),
            RlsEnforceabilityError::RlsNotForced { table, .. }
            | RlsEnforceabilityError::GovernedTableMissing { table } => {
                Self::RlsNotForcedOnTable { table }
            }
        }
    }
}

/// Reject an empty database URL before opening a pool (synchronous so the
/// always-on unit lane can assert the guard without an async runtime).
fn validate_database_url(database_url: &str) -> Result<(), PgScimConnectError> {
    if database_url.trim().is_empty() {
        return Err(PgScimConnectError::MissingDatabaseUrl);
    }
    Ok(())
}

/// Connect a shared pool for the SCIM stores. TLS is aws-lc-rs rustls (workspace
/// `sqlx` feature `tls-rustls-aws-lc-rs`; `ring` is forbidden per ADR-0506).
pub async fn connect_pool(database_url: &str) -> Result<PgPool, PgScimConnectError> {
    validate_database_url(database_url)?;
    PgPoolOptions::new()
        .max_connections(8)
        .connect(database_url)
        .await
        .map_err(|error| PgScimConnectError::Sqlx(error.to_string()))
}

/// Assert that the connected role is safe for serving multi-tenant SCIM traffic:
///
/// 1. It must NOT carry `rolsuper` or `rolbypassrls` (either flag silently skips
///    Postgres RLS, turning tenant isolation into a no-op).
/// 2. It MUST be [`RUNTIME_ROLE`] or a member of it (a policy `TO r` only applies
///    to the current user when that user is `r` or a member of `r`; a non-member
///    role under FORCE RLS gets deny-all — a safe outage, but still a
///    misconfiguration we refuse to serve).
///
/// Call this from the composition root that serves multi-tenant traffic AFTER
/// [`connect_pool`] but BEFORE accepting requests, so a mis-provisioned role
/// fails the service at boot rather than at runtime. It is a FREE function taking
/// `&PgPool` (not a store method) because the SCIM surface composes TWO stores
/// ([`PgScimUserStore`] + [`PgScimGroupStore`]) over a single shared pool — the
/// guard runs once against that pool, not redundantly per store.
///
/// ## Note on `SET ROLE` / `current_user`
///
/// This check queries `current_user` (the active role, possibly changed by
/// `SET ROLE`). It is valid here because these adapters never issue `SET ROLE`
/// and use a homogeneous pool (every serving connection has the same login role,
/// so `current_user == session_user` throughout). The query also asserts that
/// invariant: if `session_user <> current_user`, a role switch is in effect and
/// this function returns an error rather than checking the wrong role.
///
/// ## Guard scope
///
/// This guard is necessary but not sufficient for full tenant isolation. Full
/// isolation additionally requires that [`RUNTIME_ROLE`] exists in the database,
/// provisioned with `NOBYPASSRLS` (the deferred `0000_runtime_role.sql`
/// follow-up, mirroring oya-data-outbox-adapter-postgres / tenant-lifecycle).
///
/// # Errors
/// - [`PgScimConnectError::RlsUnenforceable`] if the current role carries
///   `rolsuper` or `rolbypassrls`.
/// - [`PgScimConnectError::RlsRoleMismatch`] if the current role is not a member
///   of [`RUNTIME_ROLE`] (policies would not apply).
/// - [`PgScimConnectError::Sqlx`] if the `pg_roles` / `pg_has_role` query fails
///   (e.g. [`RUNTIME_ROLE`] does not yet exist in the database — the deferred
///   provisioning migration has not run), or if a `SET ROLE` switch is detected.
pub async fn assert_rls_enforceable(pool: &PgPool) -> Result<(), PgScimConnectError> {
    // Thin delegate to the single shared boot guard (the #112 dedup): role
    // bypass/membership probe ('USAGE' = has_privs_of_role) + the
    // session==current_user SET ROLE check + per-table ENABLE+FORCE RLS check,
    // mapped via `From<RlsEnforceabilityError>` to preserve this adapter's
    // fail-closed connect-error contract. Still a FREE function taking `&PgPool`
    // (not a store method) because the SCIM surface composes TWO stores over a
    // single shared pool — the guard runs once against that pool.
    assert_rls_enforceable_shared(pool, RUNTIME_ROLE, GOVERNED_TABLES)
        .await
        .map_err(PgScimConnectError::from)
}

/// Map an sqlx error to the kernel's write-store error vocabulary.
fn store_unavailable(error: sqlx::Error) -> ScimStoreError {
    ScimStoreError::Unavailable {
        detail: error.to_string(),
    }
}

fn corrupt(detail: impl Into<String>) -> ScimStoreError {
    ScimStoreError::Corrupt {
        detail: detail.into(),
    }
}

/// Reject an empty / whitespace-only tenant scope at the adapter boundary BEFORE
/// it is bound into the per-transaction tenant GUC. An empty GUC makes the
/// migration's restrictive `require_tenant_guc` policy deny every row (a silent
/// deny-all), so a blank tenant is surfaced as a contract violation rather than
/// binding `SET set_config('oyatie.tenant_id', '', true)`.
fn validate_tenant(tenant: &TenantId) -> Result<(), ScimStoreError> {
    if tenant.0.trim().is_empty() {
        return Err(corrupt("tenant scope is empty"));
    }
    Ok(())
}

/// Real Postgres-backed SCIM [`UserStore`]: per-transaction tenant GUC + RLS
/// isolation over a shared `sqlx::PgPool`. Reads return `None`/`Vec` on absence
/// or backend failure (the port's read methods are infallible by shape; the
/// server layer surfaces 500s for the fallible write paths).
#[derive(Clone, Debug)]
pub struct PgScimUserStore {
    pool: PgPool,
}

impl PgScimUserStore {
    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Decode a persisted JSON payload into the SCIM aggregate.
fn decode_payload<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> Option<T> {
    serde_json::from_value(value).ok()
}

impl UserStore for PgScimUserStore {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<User>> + Send + 'a>> {
        Box::pin(async move {
            // Fail-closed parity with the write methods: a blank tenant would
            // bind an empty GUC (a silent deny-all), so short-circuit to an
            // empty page before opening the tx / setting the GUC.
            if validate_tenant(tenant).is_err() {
                return Vec::new();
            }
            let mut tx = match self.pool.begin().await {
                Ok(tx) => tx,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "user.list",
                        %error,
                        "begin tx failed; returning empty page"
                    );
                    return Vec::new();
                }
            };
            if let Err(error) = sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
            {
                tracing::warn!(
                    target: "identity_scim_store",
                    op = "user.list",
                    %error,
                    "set tenant GUC failed; returning empty page"
                );
                return Vec::new();
            }
            let rows = match sqlx::query(LIST_USERS_SQL)
                .bind(&tenant.0)
                .fetch_all(&mut *tx)
                .await
            {
                Ok(rows) => rows,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "user.list",
                        %error,
                        "list query failed; returning empty page"
                    );
                    return Vec::new();
                }
            };
            let _ = tx.commit().await;
            rows.into_iter()
                .filter_map(|row| row.try_get::<serde_json::Value, _>("payload_json").ok())
                .filter_map(decode_payload)
                .collect()
        })
    }

    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        Box::pin(async move {
            // Fail-closed parity with the write methods: a blank tenant would
            // bind an empty GUC (a silent deny-all), so short-circuit to None
            // before opening the tx / setting the GUC.
            validate_tenant(tenant).ok()?;
            let mut tx = self.pool.begin().await.ok()?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .ok()?;
            let row = match sqlx::query(GET_USER_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .fetch_optional(&mut *tx)
                .await
            {
                Ok(row) => row,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "user.get",
                        %error,
                        "get query failed; returning None"
                    );
                    return None;
                }
            };
            let _ = tx.commit().await;
            let payload: serde_json::Value = row?.try_get("payload_json").ok()?;
            decode_payload(payload)
        })
    }

    fn put<'a>(
        &'a self,
        user: &'a User,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let payload = serde_json::to_value(user).map_err(|e| corrupt(e.to_string()))?;
            // external_id is OPTIONAL in SCIM (RFC 7643 §3.1): bind the payload's
            // Option faithfully into the nullable column — NULL when absent,
            // never coerced to "" (which would diverge from the payload).
            let external_id: Option<&str> = user.external_id.as_deref();
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(UPSERT_USER_SQL)
                .bind(&tenant.0)
                .bind(&user.id.0)
                .bind(&user.user_name)
                .bind(external_id)
                .bind(user.active)
                .bind(&payload)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn delete<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(DELETE_USER_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn find_by_user_name<'a>(
        &'a self,
        tenant: &'a TenantId,
        user_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        Box::pin(async move {
            // Fail-closed parity with the write methods: a blank tenant would
            // bind an empty GUC (a silent deny-all), so short-circuit to None
            // before opening the tx / setting the GUC.
            validate_tenant(tenant).ok()?;
            let mut tx = self.pool.begin().await.ok()?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .ok()?;
            let row = match sqlx::query(FIND_USER_BY_NAME_SQL)
                .bind(&tenant.0)
                .bind(user_name)
                .fetch_optional(&mut *tx)
                .await
            {
                Ok(row) => row,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "user.find_by_user_name",
                        %error,
                        "find-by-user-name query failed; returning None"
                    );
                    return None;
                }
            };
            let _ = tx.commit().await;
            let payload: serde_json::Value = row?.try_get("payload_json").ok()?;
            decode_payload(payload)
        })
    }
}

/// Real Postgres-backed SCIM [`GroupStore`].
#[derive(Clone, Debug)]
pub struct PgScimGroupStore {
    pool: PgPool,
}

impl PgScimGroupStore {
    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl GroupStore for PgScimGroupStore {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<Group>> + Send + 'a>> {
        Box::pin(async move {
            // Fail-closed parity with the write methods: a blank tenant would
            // bind an empty GUC (a silent deny-all), so short-circuit to an
            // empty page before opening the tx / setting the GUC.
            if validate_tenant(tenant).is_err() {
                return Vec::new();
            }
            let mut tx = match self.pool.begin().await {
                Ok(tx) => tx,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "group.list",
                        %error,
                        "begin tx failed; returning empty page"
                    );
                    return Vec::new();
                }
            };
            if let Err(error) = sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
            {
                tracing::warn!(
                    target: "identity_scim_store",
                    op = "group.list",
                    %error,
                    "set tenant GUC failed; returning empty page"
                );
                return Vec::new();
            }
            let rows = match sqlx::query(LIST_GROUPS_SQL)
                .bind(&tenant.0)
                .fetch_all(&mut *tx)
                .await
            {
                Ok(rows) => rows,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "group.list",
                        %error,
                        "list query failed; returning empty page"
                    );
                    return Vec::new();
                }
            };
            let _ = tx.commit().await;
            rows.into_iter()
                .filter_map(|row| row.try_get::<serde_json::Value, _>("payload_json").ok())
                .filter_map(decode_payload)
                .collect()
        })
    }

    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<Group>> + Send + 'a>> {
        Box::pin(async move {
            // Fail-closed parity with the write methods: a blank tenant would
            // bind an empty GUC (a silent deny-all), so short-circuit to None
            // before opening the tx / setting the GUC.
            validate_tenant(tenant).ok()?;
            let mut tx = self.pool.begin().await.ok()?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .ok()?;
            let row = match sqlx::query(GET_GROUP_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .fetch_optional(&mut *tx)
                .await
            {
                Ok(row) => row,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "group.get",
                        %error,
                        "get query failed; returning None"
                    );
                    return None;
                }
            };
            let _ = tx.commit().await;
            let payload: serde_json::Value = row?.try_get("payload_json").ok()?;
            decode_payload(payload)
        })
    }

    fn put<'a>(
        &'a self,
        group: &'a Group,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let payload = serde_json::to_value(group).map_err(|e| corrupt(e.to_string()))?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(UPSERT_GROUP_SQL)
                .bind(&tenant.0)
                .bind(&group.id.0)
                .bind(&group.display_name)
                .bind(&payload)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn delete<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(DELETE_GROUP_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_guc_first_then_scoped(statements: &[&str]) {
        assert!(!statements.is_empty(), "plan must not be empty");
        assert_eq!(
            statements[0], SET_LOCAL_TENANT_SQL,
            "the tenant GUC set_config MUST be the first statement"
        );
        for stmt in &statements[1..] {
            assert_ne!(
                *stmt, SET_LOCAL_TENANT_SQL,
                "only the leading statement sets the GUC"
            );
        }
    }

    #[test]
    fn guc_set_statement_is_canonical_oyatie_tenant_id() {
        // The fix for the GUC divergence: this adapter uses the canonical GUC,
        // NOT app.tenant_id.
        assert_eq!(
            SET_LOCAL_TENANT_SQL,
            "SELECT set_config('oyatie.tenant_id', $1, true)"
        );
    }

    #[test]
    fn user_plans_set_guc_before_statement() {
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, LIST_USERS_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, GET_USER_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, FIND_USER_BY_NAME_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, UPSERT_USER_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, DELETE_USER_SQL]);
    }

    #[test]
    fn group_plans_set_guc_before_statement() {
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, LIST_GROUPS_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, GET_GROUP_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, UPSERT_GROUP_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, DELETE_GROUP_SQL]);
    }

    #[test]
    fn every_scoped_statement_filters_by_tenant_id() {
        for sql in [
            LIST_USERS_SQL,
            GET_USER_SQL,
            FIND_USER_BY_NAME_SQL,
            UPSERT_USER_SQL,
            DELETE_USER_SQL,
            LIST_GROUPS_SQL,
            GET_GROUP_SQL,
            UPSERT_GROUP_SQL,
            DELETE_GROUP_SQL,
        ] {
            assert!(
                sql.contains("tenant_id"),
                "scoped statement must reference tenant_id: {sql}"
            );
        }
    }

    #[test]
    fn connect_rejects_empty_database_url() {
        assert_eq!(
            validate_database_url("   "),
            Err(PgScimConnectError::MissingDatabaseUrl)
        );
        assert_eq!(
            validate_database_url("postgres://u:p@localhost/db?sslmode=require"),
            Ok(())
        );
    }

    #[test]
    fn validate_tenant_rejects_empty_or_blank() {
        assert!(matches!(
            validate_tenant(&TenantId(String::new())),
            Err(ScimStoreError::Corrupt { .. })
        ));
        assert!(matches!(
            validate_tenant(&TenantId("   ".to_owned())),
            Err(ScimStoreError::Corrupt { .. })
        ));
        assert_eq!(validate_tenant(&TenantId("acme".to_owned())), Ok(()));
    }

    // --- RlsEnforceabilityError -> PgScimConnectError mapping -----------------
    // The DB-free role/forced predicate decisions now live in the shared kernel
    // (oya-shared-postgres-command-kernel); this adapter keeps only the THIN
    // mapping that preserves its fail-closed connect-error contract.

    #[test]
    fn rls_error_mapping_preserves_fail_closed_contract() {
        assert_eq!(
            PgScimConnectError::from(RlsEnforceabilityError::Unenforceable {
                role: "pg_superuser".to_owned()
            }),
            PgScimConnectError::RlsUnenforceable {
                role: "pg_superuser".to_owned()
            }
        );
        assert_eq!(
            PgScimConnectError::from(RlsEnforceabilityError::RoleMismatch {
                role: "some_role".to_owned(),
                expected: RUNTIME_ROLE.to_owned(),
            }),
            PgScimConnectError::RlsRoleMismatch {
                role: "some_role".to_owned(),
                expected: RUNTIME_ROLE.to_owned(),
            }
        );
        // A SET ROLE switch and a probe failure both preserve today's `Sqlx`
        // surface (not the structured RLS variants).
        assert!(matches!(
            PgScimConnectError::from(RlsEnforceabilityError::RoleSwitchInEffect {
                session_role: "a".to_owned(),
                current_role: "b".to_owned(),
            }),
            PgScimConnectError::Sqlx(_)
        ));
        assert!(matches!(
            PgScimConnectError::from(RlsEnforceabilityError::ProbeFailed {
                detail: "role does not exist".to_owned()
            }),
            PgScimConnectError::Sqlx(_)
        ));
        // The #799 FORCE-RLS hardening: a not-forced or missing governed table
        // maps to the new RlsNotForcedOnTable variant.
        assert_eq!(
            PgScimConnectError::from(RlsEnforceabilityError::RlsNotForced {
                table: USERS_TABLE.to_owned(),
                row_security: true,
                force_row_security: false,
            }),
            PgScimConnectError::RlsNotForcedOnTable {
                table: USERS_TABLE.to_owned()
            }
        );
        assert_eq!(
            PgScimConnectError::from(RlsEnforceabilityError::GovernedTableMissing {
                table: GROUPS_TABLE.to_owned()
            }),
            PgScimConnectError::RlsNotForcedOnTable {
                table: GROUPS_TABLE.to_owned()
            }
        );
    }

    #[test]
    fn runtime_role_matches_migration_policy_subject() {
        // The const MUST equal the `TO <role>` subject in
        // migrations/0001_identity_scim_store.sql — the guard checks membership
        // in exactly this role, so a drift would silently let a non-policy role
        // pass the guard while the policies never apply to it.
        assert_eq!(RUNTIME_ROLE, "identity_scim_runtime");
    }

    #[test]
    fn governed_tables_match_migration_force_rls() {
        // The boot guard verifies ENABLE+FORCE RLS for exactly GOVERNED_TABLES.
        // If a dev FORCE-RLS'es a new table in the migration but forgets to add
        // it here (or vice versa), tenant isolation would be silently unverified
        // at boot. Asserting the SAME list the guard passes EXACTLY equals the
        // migration's FORCE'd-table set makes that drift impossible. DB-free —
        // pure string comparison, runs in the always-on unit lane.
        use oya_shared_postgres_command_kernel::force_rls_tables;
        let migration = include_str!("../migrations/0001_identity_scim_store.sql");
        let mut from_migration = force_rls_tables(migration);
        from_migration.sort();
        let mut governed: Vec<String> = GOVERNED_TABLES.iter().map(|t| (*t).to_owned()).collect();
        governed.sort();
        assert_eq!(
            governed, from_migration,
            "GOVERNED_TABLES must EXACTLY match the tables \
             migrations/0001_identity_scim_store.sql FORCE-RLS'es"
        );
    }
}
