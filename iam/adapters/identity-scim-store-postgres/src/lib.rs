//! Tenant-scoped SCIM/Postgres adapters with enforced RLS and atomic protected writes.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod passkey_schema;
pub mod passkeys;
pub use passkey_schema::{PASSKEY_AUTHENTICATION_SQL, provision_passkey_schema};

use core::future::Future;
use core::pin::Pin;

use iam_identity_user_session::{MembershipRecord, MembershipStoreError, UserMembershipStore};
use iam_scim_server_kernel::{
    Group, GroupStore, ScimId, ScimStoreError, TenantId, User, UserStore, protected_user_deletion,
    protected_user_transition,
};
use shared_postgres_command_adapter_sqlx::assert_rls_enforceable as assert_rls_enforceable_shared;
use shared_postgres_command_kernel::{RlsEnforceabilityError, SET_LOCAL_TENANT_SQL};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

/// Canonical schema + table names. One place so the migration, the runtime
/// statements, and the plan-shape tests cannot drift.
pub const SCHEMA_NAME: &str = "identity_scim";
pub const USERS_TABLE: &str = "identity_scim.identity_scim_users";
pub const GROUPS_TABLE: &str = "identity_scim.identity_scim_groups";
pub const PASSKEY_CHALLENGES_TABLE: &str = "identity_scim.identity_user_passkey_challenges";
pub const PASSKEYS_TABLE: &str = "identity_scim.identity_user_passkeys";
pub const SESSIONS_TABLE: &str = "identity_scim.identity_user_sessions";

/// The tables whose ENABLE+FORCE RLS the boot guard verifies. This is the SINGLE
/// list the runtime guard passes to `assert_rls_enforceable`; a unit test (see
/// `governed_tables_match_migration_force_rls`) asserts it EXACTLY matches the
/// set of tables `migrations/0001_*.sql` declares FORCE ROW LEVEL SECURITY, so a
/// new FORCE'd table can never silently escape the guard's per-table coverage.
pub const GOVERNED_TABLES: &[&str] = &[
    USERS_TABLE,
    GROUPS_TABLE,
    PASSKEY_CHALLENGES_TABLE,
    PASSKEYS_TABLE,
    SESSIONS_TABLE,
];

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
const FIND_NATIVE_USER_MEMBERSHIPS_SQL: &str = "SELECT payload_json, active FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND payload_json -> $2 ->> 'nativeUserId' = $3 ORDER BY scim_id ASC";
const INSERT_USER_SQL: &str = "INSERT INTO identity_scim.identity_scim_users (tenant_id, scim_id, user_name, external_id, active, payload_json, schema_version, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, now())";
const LOCK_USER_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND scim_id = $2 FOR UPDATE";
const UPDATE_USER_SQL: &str = "UPDATE identity_scim.identity_scim_users SET user_name = $3, external_id = $4, active = $5, payload_json = $6, schema_version = $7, updated_at = now() WHERE tenant_id = $1 AND scim_id = $2";
const DELETE_USER_SQL: &str =
    "DELETE FROM identity_scim.identity_scim_users WHERE tenant_id = $1 AND scim_id = $2";

// --- Group statements (tenant-scoped) --------------------------------------
const LIST_GROUPS_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 ORDER BY scim_id ASC";
const GET_GROUP_SQL: &str = "SELECT payload_json FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 AND scim_id = $2";
const UPSERT_GROUP_SQL: &str = "INSERT INTO identity_scim.identity_scim_groups (tenant_id, scim_id, display_name, payload_json, schema_version, updated_at) VALUES ($1, $2, $3, $4, $5, now()) ON CONFLICT (tenant_id, scim_id) DO UPDATE SET display_name = EXCLUDED.display_name, payload_json = EXCLUDED.payload_json, schema_version = EXCLUDED.schema_version, updated_at = now()";
const DELETE_GROUP_SQL: &str =
    "DELETE FROM identity_scim.identity_scim_groups WHERE tenant_id = $1 AND scim_id = $2";

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

mod connection;
pub use connection::*;
mod helpers;
use helpers::*;
mod groups;
mod membership;
#[cfg(test)]
use membership::membership_from_payload;
#[cfg(test)]
mod tests;
mod user_reads;
mod user_writes;
