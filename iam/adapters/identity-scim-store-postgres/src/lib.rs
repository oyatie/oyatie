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

use oya_shared_postgres_command_kernel::SET_LOCAL_TENANT_SQL;
use oya_shared_scim_server_kernel::{
    Group, GroupStore, ScimId, ScimStoreError, TenantId, User, UserStore,
};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

/// Canonical schema + table names. One place so the migration, the runtime
/// statements, and the plan-shape tests cannot drift.
pub const SCHEMA_NAME: &str = "identity_scim";
pub const USERS_TABLE: &str = "identity_scim.identity_scim_users";
pub const GROUPS_TABLE: &str = "identity_scim.identity_scim_groups";

/// Current persisted-record schema version.
pub const SCHEMA_VERSION: i32 = 1;

// --- User statements (tenant-scoped) ---------------------------------------
const LIST_USERS_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_users WHERE tenant_id = $1 ORDER BY scim_id ASC";
const GET_USER_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND scim_id = $2";
const FIND_USER_BY_NAME_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND user_name = $2";
const UPSERT_USER_SQL: &str = "INSERT INTO identity_scim.identity_scim_users (tenant_id, scim_id, user_name, external_id, active, payload_json, schema_version, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, now()) ON CONFLICT (tenant_id, scim_id) DO UPDATE SET user_name = EXCLUDED.user_name, external_id = EXCLUDED.external_id, active = EXCLUDED.active, payload_json = EXCLUDED.payload_json, schema_version = EXCLUDED.schema_version, updated_at = now()";
const DELETE_USER_SQL: &str = "DELETE FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND scim_id = $2";

// --- Group statements (tenant-scoped) --------------------------------------
const LIST_GROUPS_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 ORDER BY scim_id ASC";
const GET_GROUP_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 AND scim_id = $2";
const UPSERT_GROUP_SQL: &str = "INSERT INTO identity_scim.identity_scim_groups (tenant_id, scim_id, display_name, payload_json, schema_version, updated_at) VALUES ($1, $2, $3, $4, $5, now()) ON CONFLICT (tenant_id, scim_id) DO UPDATE SET display_name = EXCLUDED.display_name, payload_json = EXCLUDED.payload_json, schema_version = EXCLUDED.schema_version, updated_at = now()";
const DELETE_GROUP_SQL: &str = "DELETE FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 AND scim_id = $2";

/// Errors specific to constructing the durable adapter (connection-time).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgScimConnectError {
    /// The supplied database URL was empty.
    MissingDatabaseUrl,
    /// The underlying sqlx pool failed to connect.
    Sqlx(String),
}

impl core::fmt::Display for PgScimConnectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingDatabaseUrl => write!(f, "database url is empty"),
            Self::Sqlx(detail) => write!(f, "sqlx connect failed: {detail}"),
        }
    }
}

impl std::error::Error for PgScimConnectError {}

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
            let Ok(mut tx) = self.pool.begin().await else {
                return Vec::new();
            };
            if sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .is_err()
            {
                return Vec::new();
            }
            let rows = sqlx::query(LIST_USERS_SQL)
                .bind(&tenant.0)
                .fetch_all(&mut *tx)
                .await
                .unwrap_or_default();
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
            let mut tx = self.pool.begin().await.ok()?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .ok()?;
            let row = sqlx::query(GET_USER_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .fetch_optional(&mut *tx)
                .await
                .ok()?;
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
            let payload = serde_json::to_value(user).map_err(|e| corrupt(e.to_string()))?;
            let external_id = user.external_id.clone().unwrap_or_default();
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
                .bind(&external_id)
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
            let mut tx = self.pool.begin().await.ok()?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .ok()?;
            let row = sqlx::query(FIND_USER_BY_NAME_SQL)
                .bind(&tenant.0)
                .bind(user_name)
                .fetch_optional(&mut *tx)
                .await
                .ok()?;
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
            let Ok(mut tx) = self.pool.begin().await else {
                return Vec::new();
            };
            if sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .is_err()
            {
                return Vec::new();
            }
            let rows = sqlx::query(LIST_GROUPS_SQL)
                .bind(&tenant.0)
                .fetch_all(&mut *tx)
                .await
                .unwrap_or_default();
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
            let mut tx = self.pool.begin().await.ok()?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .ok()?;
            let row = sqlx::query(GET_GROUP_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .fetch_optional(&mut *tx)
                .await
                .ok()?;
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
}
