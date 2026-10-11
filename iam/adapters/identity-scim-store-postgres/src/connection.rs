use super::*;

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
    /// `0000_runtime_role.sql` follow-up, mirroring data-outbox-adapter-postgres
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
pub(super) fn validate_database_url(database_url: &str) -> Result<(), PgScimConnectError> {
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

/// Refuse serving when the runtime role bypasses RLS, lacks the policy role,
/// or any governed table lacks enforced row security.
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
