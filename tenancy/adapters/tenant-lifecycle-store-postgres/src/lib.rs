//! Real async sqlx/Postgres adapter for the [`TenantLifecycleStore`] port.
//!
//! This is a LIVE durable adapter (not a shape plan): it owns a `sqlx::PgPool`,
//! and every operation runs in its own transaction that sets the canonical
//! per-transaction tenant GUC FIRST —
//! `SELECT set_config('oyatie.tenant_id', $1, true)` (the shared command
//! kernel's [`SET_LOCAL_TENANT_SQL`]) — before any tenant-scoped statement, so
//! Postgres RLS (RESTRICTIVE, FORCE) isolates every row by `tenant_id`. The
//! runtime role must NOT carry BYPASSRLS or isolation is silently skipped.
//!
//! ## Doctrine: transient adapter behind an owned-shaped port
//!
//! The kernel port [`TenantLifecycleStore`] is the OWNED-destination contract
//! (the oya-data ordered-keyed KV shape). Postgres is a TRANSIENT adapter
//! behind it; the owned data substrate (G003/oya-data) cuts over later WITHOUT
//! changing the port. TLS is aws-lc-rs rustls only (ADR-0506; `ring` is
//! forbidden) — wired through the workspace `sqlx` feature
//! `tls-rustls-aws-lc-rs`.
//!
//! ## Tenant scope
//!
//! Lifecycle resources are `tenants/<id>`, so the tenant id IS the resource id;
//! that id is the RLS scope for the tenant aggregate row and for the per-tenant
//! idempotency + operation ledger rows. The idempotency dedup table replays
//! safely via `ON CONFLICT (tenant_id, idempotency_key) DO NOTHING`.
//!
//! ## Testing
//!
//! The always-on tests are hermetic plan-shape tests: they assert the generated
//! SQL and that the GUC-set statement precedes every tenant-scoped statement
//! (no database needed). Live RLS/idempotency integration tests live behind the
//! `OYA_BACKBONE_LIVE_POSTGRES` env gate (see `tests/`).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::future::Future;
use core::pin::Pin;

use oya_shared_platform_contracts_kernel::tenancy::{Tenant, TenantLifecycleState};
use oya_shared_postgres_command_kernel::SET_LOCAL_TENANT_SQL;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use tenancy_tenant_lifecycle_kernel::{
    AppliedWriteRecord, OperationRecord, StoreError, TenantLifecycleStore,
};

/// Canonical schema + table names. Kept in one place so the migration, the
/// runtime statements, and the plan-shape tests cannot drift.
pub const SCHEMA_NAME: &str = "tenancy_lifecycle";
pub const TENANTS_TABLE: &str = "tenancy_lifecycle.tenancy_lifecycle_tenants";
pub const APPLIED_TABLE: &str = "tenancy_lifecycle.tenancy_lifecycle_applied_writes";
pub const OPERATIONS_TABLE: &str = "tenancy_lifecycle.tenancy_lifecycle_operations";

/// The current persisted-record schema version (bumped on a breaking change to
/// the `payload_json` shape).
pub const SCHEMA_VERSION: i32 = 1;

// --- Tenant aggregate statements (tenant-scoped) ---------------------------
const SELECT_TENANT_SQL: &str = "SELECT payload_json FROM tenancy_lifecycle.tenancy_lifecycle_tenants WHERE tenant_id = $1 AND resource_name = $2";
const UPSERT_TENANT_SQL: &str = "INSERT INTO tenancy_lifecycle.tenancy_lifecycle_tenants (tenant_id, resource_name, display_name, lifecycle_state, payload_json, schema_version, updated_at) VALUES ($1, $2, $3, $4, $5, $6, now()) ON CONFLICT (tenant_id, resource_name) DO UPDATE SET display_name = EXCLUDED.display_name, lifecycle_state = EXCLUDED.lifecycle_state, payload_json = EXCLUDED.payload_json, schema_version = EXCLUDED.schema_version, updated_at = now()";
const DELETE_TENANT_SQL: &str = "DELETE FROM tenancy_lifecycle.tenancy_lifecycle_tenants WHERE tenant_id = $1 AND resource_name = $2";
const SCAN_TENANTS_SQL: &str = "SELECT resource_name, payload_json FROM tenancy_lifecycle.tenancy_lifecycle_tenants WHERE tenant_id = $1 AND resource_name >= $2 AND resource_name LIKE $3 || '%' ORDER BY resource_name ASC LIMIT $4";

// --- Idempotency dedup statements (tenant-scoped) --------------------------
const SELECT_APPLIED_SQL: &str = "SELECT payload_json FROM tenancy_lifecycle.tenancy_lifecycle_applied_writes WHERE tenant_id = $1 AND idempotency_key = $2";
const INSERT_APPLIED_SQL: &str = "INSERT INTO tenancy_lifecycle.tenancy_lifecycle_applied_writes (tenant_id, idempotency_key, payload_json, schema_version, created_at) VALUES ($1, $2, $3, $4, now()) ON CONFLICT (tenant_id, idempotency_key) DO NOTHING";

// --- Operation ledger statements (tenant-scoped) ---------------------------
const SELECT_OPERATION_SQL: &str = "SELECT payload_json FROM tenancy_lifecycle.tenancy_lifecycle_operations WHERE tenant_id = $1 AND operation_name = $2";
const UPSERT_OPERATION_SQL: &str = "INSERT INTO tenancy_lifecycle.tenancy_lifecycle_operations (tenant_id, operation_name, operation_seq, payload_json, schema_version, created_at) VALUES ($1, $2, $3, $4, $5, now()) ON CONFLICT (tenant_id, operation_name) DO UPDATE SET payload_json = EXCLUDED.payload_json, schema_version = EXCLUDED.schema_version";
const NEXT_SEQ_SQL: &str = "SELECT coalesce(max(operation_seq), 0) + 1 FROM tenancy_lifecycle.tenancy_lifecycle_operations WHERE tenant_id = $1";

/// The lifecycle resource-name collection prefix (`tenants/`).
const TENANT_COLLECTION_PREFIX: &str = "tenants/";

/// Errors specific to constructing the durable adapter (connection-time).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PgStoreConnectError {
    /// The supplied database URL was empty.
    MissingDatabaseUrl,
    /// The underlying sqlx pool failed to connect.
    Sqlx(String),
}

impl core::fmt::Display for PgStoreConnectError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingDatabaseUrl => write!(f, "database url is empty"),
            Self::Sqlx(detail) => write!(f, "sqlx connect failed: {detail}"),
        }
    }
}

impl std::error::Error for PgStoreConnectError {}

/// Real Postgres-backed [`TenantLifecycleStore`]: per-transaction tenant GUC +
/// RLS isolation over a shared `sqlx::PgPool`.
#[derive(Clone, Debug)]
pub struct PgTenantLifecycleStore {
    pool: PgPool,
}

/// Map an sqlx error to the kernel's availability/integrity error vocabulary.
fn store_unavailable(error: sqlx::Error) -> StoreError {
    StoreError::Unavailable {
        detail: error.to_string(),
    }
}

/// Decode a persisted aggregate JSON payload, mapping a decode failure to the
/// kernel's `Corrupt` variant (a row that violates the locked contracts).
fn decode<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> Result<T, StoreError> {
    serde_json::from_value(value).map_err(|error| StoreError::Corrupt {
        detail: error.to_string(),
    })
}

/// Encode an aggregate to its persisted JSON payload.
fn encode<T: serde::Serialize>(value: &T) -> Result<serde_json::Value, StoreError> {
    serde_json::to_value(value).map_err(|error| StoreError::Corrupt {
        detail: error.to_string(),
    })
}

/// The RLS tenant scope for a lifecycle resource name (`tenants/<id>` -> `<id>`).
/// Returns `None` for a name that is not a single tenant resource (e.g. the bare
/// `tenants/` collection prefix), which the caller treats as out-of-scope.
fn tenant_of_resource_name(resource_name: &str) -> Option<&str> {
    resource_name
        .strip_prefix(TENANT_COLLECTION_PREFIX)
        .filter(|id| !id.is_empty() && !id.contains('/'))
}

impl PgTenantLifecycleStore {
    /// Wrap an already-configured pool (the composition root owns pool/TLS
    /// configuration; this adapter only runs tenant-scoped transactions).
    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Connect a pool from a database URL. TLS is aws-lc-rs rustls (workspace
    /// `sqlx` feature `tls-rustls-aws-lc-rs`; `ring` is forbidden per ADR-0506).
    pub async fn connect(database_url: &str) -> Result<Self, PgStoreConnectError> {
        validate_database_url(database_url)?;
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(database_url)
            .await
            .map_err(|error| PgStoreConnectError::Sqlx(error.to_string()))?;
        Ok(Self { pool })
    }
}

/// Reject an empty database URL before opening a pool (kept synchronous so the
/// always-on unit lane can assert the guard without an async runtime).
fn validate_database_url(database_url: &str) -> Result<(), PgStoreConnectError> {
    if database_url.trim().is_empty() {
        return Err(PgStoreConnectError::MissingDatabaseUrl);
    }
    Ok(())
}

/// Reject an empty / whitespace-only tenant scope at the adapter boundary BEFORE
/// it is bound into the per-transaction tenant GUC. An empty GUC would make the
/// migration's restrictive `require_tenant_guc` policy deny every row (a silent
/// deny-all), so a blank tenant is a contract violation we surface explicitly
/// rather than emitting a `SET set_config('oyatie.tenant_id', '', true)` that
/// produces an opaque empty result set.
fn validate_tenant_id(tenant_id: &str) -> Result<(), StoreError> {
    if tenant_id.trim().is_empty() {
        return Err(StoreError::Corrupt {
            detail: "tenant scope is empty".to_owned(),
        });
    }
    Ok(())
}

impl TenantLifecycleStore for PgTenantLifecycleStore {
    fn get_tenant<'a>(
        &'a self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Tenant>, StoreError>> + Send + 'a>> {
        Box::pin(async move {
            let Some(tenant_id) = tenant_of_resource_name(name) else {
                return Ok(None);
            };
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            let row = sqlx::query(SELECT_TENANT_SQL)
                .bind(tenant_id)
                .bind(name)
                .fetch_optional(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            match row {
                None => Ok(None),
                Some(row) => {
                    let payload: serde_json::Value =
                        row.try_get("payload_json").map_err(store_unavailable)?;
                    Ok(Some(decode(payload)?))
                }
            }
        })
    }

    fn put_tenant<'a>(
        &'a mut self,
        name: &'a str,
        tenant: &'a Tenant,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            let Some(tenant_id) = tenant_of_resource_name(name) else {
                return Err(StoreError::Corrupt {
                    detail: format!("resource name {name:?} is not a tenant resource"),
                });
            };
            let payload = encode(tenant)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(UPSERT_TENANT_SQL)
                .bind(tenant_id)
                .bind(name)
                .bind(&tenant.display_name)
                .bind(tenant.state.slug())
                .bind(&payload)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn remove_tenant<'a>(
        &'a mut self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            let Some(tenant_id) = tenant_of_resource_name(name) else {
                return Ok(());
            };
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(DELETE_TENANT_SQL)
                .bind(tenant_id)
                .bind(name)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn scan_tenants<'a>(
        &'a self,
        prefix: &'a str,
        start_at: Option<&'a str>,
        limit: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(String, Tenant)>, StoreError>> + Send + 'a>> {
        Box::pin(async move {
            // The tenant aggregate is RLS-scoped per tenant_id; a scan is only
            // resolvable to a single tenant's own resource. The bare `tenants/`
            // collection scan is a privileged control-plane path NOT served by
            // this per-tenant adapter, so it returns an empty page rather than
            // leaking across the RLS boundary.
            let Some(tenant_id) = tenant_of_resource_name(prefix) else {
                return Ok(Vec::new());
            };
            if limit == 0 {
                return Ok(Vec::new());
            }
            let lower = match start_at {
                Some(start) if start >= prefix => start,
                _ => prefix,
            };
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            let rows = sqlx::query(SCAN_TENANTS_SQL)
                .bind(tenant_id)
                .bind(lower)
                .bind(prefix)
                .bind(i64::from(limit))
                .fetch_all(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                let resource_name: String =
                    row.try_get("resource_name").map_err(store_unavailable)?;
                let payload: serde_json::Value =
                    row.try_get("payload_json").map_err(store_unavailable)?;
                out.push((resource_name, decode(payload)?));
            }
            Ok(out)
        })
    }

    fn get_applied<'a>(
        &'a self,
        tenant_id: &'a str,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AppliedWriteRecord>, StoreError>> + Send + 'a>>
    {
        Box::pin(async move {
            validate_tenant_id(tenant_id)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            let row = sqlx::query(SELECT_APPLIED_SQL)
                .bind(tenant_id)
                .bind(key)
                .fetch_optional(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            match row {
                None => Ok(None),
                Some(row) => {
                    let payload: serde_json::Value =
                        row.try_get("payload_json").map_err(store_unavailable)?;
                    Ok(Some(decode(payload)?))
                }
            }
        })
    }

    fn put_applied<'a>(
        &'a mut self,
        tenant_id: &'a str,
        key: &'a str,
        record: &'a AppliedWriteRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant_id(tenant_id)?;
            let payload = encode(record)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            // ON CONFLICT DO NOTHING: replay under the same key is a no-op, so
            // the first-applied record is the durable one (idempotency).
            sqlx::query(INSERT_APPLIED_SQL)
                .bind(tenant_id)
                .bind(key)
                .bind(&payload)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn get_operation<'a>(
        &'a self,
        tenant_id: &'a str,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<OperationRecord>, StoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant_id(tenant_id)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            let row = sqlx::query(SELECT_OPERATION_SQL)
                .bind(tenant_id)
                .bind(operation_name)
                .fetch_optional(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            match row {
                None => Ok(None),
                Some(row) => {
                    let payload: serde_json::Value =
                        row.try_get("payload_json").map_err(store_unavailable)?;
                    Ok(Some(decode(payload)?))
                }
            }
        })
    }

    fn put_operation<'a>(
        &'a mut self,
        tenant_id: &'a str,
        operation_name: &'a str,
        record: &'a OperationRecord,
    ) -> Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant_id(tenant_id)?;
            let payload = encode(record)?;
            // The operation_seq is carried for ordering/observability; it is
            // derived from the minted name's trailing ordinal when present, else
            // 0 (the column has no uniqueness role — the name PK does).
            let seq = operation_seq_from_name(operation_name);
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(UPSERT_OPERATION_SQL)
                .bind(tenant_id)
                .bind(operation_name)
                .bind(seq)
                .bind(&payload)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn next_operation_seq<'a>(
        &'a mut self,
        tenant_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<u64, StoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant_id(tenant_id)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            let next: i64 = sqlx::query_scalar(NEXT_SEQ_SQL)
                .bind(tenant_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(next.max(0) as u64)
        })
    }
}

/// Extract the trailing `lifecycle-<seq>` ordinal from a minted operation name
/// (`operations/<tenant_id>/lifecycle-<seq>`). Returns 0 when the name lacks the
/// shape — the ordinal is observational; the `(tenant_id, operation_name)` PK is
/// the uniqueness authority.
fn operation_seq_from_name(operation_name: &str) -> i64 {
    operation_name
        .rsplit_once("lifecycle-")
        .and_then(|(_, tail)| tail.parse::<i64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The ordered tenant-scoped statement plan a method runs: the GUC-set
    // statement MUST be first, every following statement is tenant-scoped.
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
    fn get_tenant_plan_sets_guc_before_select() {
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, SELECT_TENANT_SQL]);
    }

    #[test]
    fn put_tenant_plan_sets_guc_before_upsert() {
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, UPSERT_TENANT_SQL]);
    }

    #[test]
    fn scan_plan_sets_guc_before_scan() {
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, SCAN_TENANTS_SQL]);
    }

    #[test]
    fn applied_plans_set_guc_before_statement() {
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, SELECT_APPLIED_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, INSERT_APPLIED_SQL]);
    }

    #[test]
    fn operation_plans_set_guc_before_statement() {
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, SELECT_OPERATION_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, UPSERT_OPERATION_SQL]);
        assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, NEXT_SEQ_SQL]);
    }

    #[test]
    fn applied_insert_is_idempotent_on_conflict_do_nothing() {
        // Replay safety: the dedup INSERT must be a no-op on key reuse.
        assert!(INSERT_APPLIED_SQL.contains("ON CONFLICT (tenant_id, idempotency_key) DO NOTHING"));
    }

    #[test]
    fn every_scoped_statement_filters_by_tenant_id() {
        for sql in [
            SELECT_TENANT_SQL,
            UPSERT_TENANT_SQL,
            DELETE_TENANT_SQL,
            SCAN_TENANTS_SQL,
            SELECT_APPLIED_SQL,
            INSERT_APPLIED_SQL,
            SELECT_OPERATION_SQL,
            UPSERT_OPERATION_SQL,
            NEXT_SEQ_SQL,
        ] {
            assert!(
                sql.contains("tenant_id"),
                "scoped statement must reference tenant_id: {sql}"
            );
        }
    }

    #[test]
    fn tenant_of_resource_name_extracts_single_tenant_only() {
        assert_eq!(tenant_of_resource_name("tenants/acme"), Some("acme"));
        assert_eq!(tenant_of_resource_name("tenants/"), None);
        assert_eq!(tenant_of_resource_name("tenants/a/b"), None);
        assert_eq!(tenant_of_resource_name("operations/x"), None);
    }

    #[test]
    fn operation_seq_parses_trailing_ordinal() {
        assert_eq!(
            operation_seq_from_name("operations/acme/lifecycle-000007"),
            7
        );
        assert_eq!(operation_seq_from_name("operations/acme/weird"), 0);
    }

    #[test]
    fn connect_rejects_empty_database_url() {
        assert_eq!(
            validate_database_url("   "),
            Err(PgStoreConnectError::MissingDatabaseUrl)
        );
        assert_eq!(
            validate_database_url("postgres://u:p@localhost/db?sslmode=require"),
            Ok(())
        );
    }

    #[test]
    fn validate_tenant_id_rejects_empty_or_blank() {
        assert!(matches!(
            validate_tenant_id(""),
            Err(StoreError::Corrupt { .. })
        ));
        assert!(matches!(
            validate_tenant_id("   "),
            Err(StoreError::Corrupt { .. })
        ));
        assert_eq!(validate_tenant_id("acme"), Ok(()));
    }

    #[test]
    fn lifecycle_state_slug_is_stable_snake_case() {
        // The persisted lifecycle_state column uses the stable slug, NOT the
        // Rust Debug formatting (which could drift from the contract).
        assert_eq!(TenantLifecycleState::Provisioning.slug(), "provisioning");
        assert_eq!(TenantLifecycleState::Active.slug(), "active");
        assert_eq!(TenantLifecycleState::Suspended.slug(), "suspended");
        assert_eq!(TenantLifecycleState::Retired.slug(), "retired");
    }
}
