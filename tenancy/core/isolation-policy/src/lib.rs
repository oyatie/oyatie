//! Isolation policy — tenant-isolation law rendered as PostgreSQL RLS DDL
//! (IP-006) and tenant-scoped JWT claim validation (IP-007).
//!
//! This crate is the tenant-isolation kernel. Two halves, one module tree:
//!
//! - [`rls`] renders the row-level-security DDL that binds every tenant-bound
//!   table to `current_setting('app.current_tenant_id')`. Identifiers are the
//!   injection surface, so nothing is interpolated before it has passed
//!   [`rls::validate_identifier`], and nothing is emitted unless it carries
//!   BOTH `ENABLE ROW LEVEL SECURITY` and `FORCE ROW LEVEL SECURITY`
//!   (IP-006 halt condition: DDL that omits FORCE is refused, not warned about,
//!   by [`rls::validate_rendered_ddl`] at the boundary where DDL arrives from
//!   outside). It also owns [`rls::REQUIRED_TENANT_BOUND_TABLES`], the registry
//!   of tables that MUST be tenant-bound, so a partial manifest is a refusal
//!   rather than a green install over an unprotected table. Output is
//!   byte-for-byte deterministic and input-order independent so it can be
//!   golden-tested and diffed across releases.
//! - [`claims`] models the tenant-scoped access token — issuer, subject,
//!   audience, tenant, scopes, `iat`/`nbf`/`exp` — and validates its SHAPE
//!   against a [`claims::ClaimsPolicy`] and a caller-supplied instant.
//! - [`inmemory`] holds the deterministic test doubles for the ports the
//!   scaffold declares ([`RlsInstaller`], [`JwtIssuer`], [`JwtVerifier`],
//!   [`SigningKeyStore`]) so the pure logic is exercisable with no I/O.
//!
//! Time is always a parameter. No function in this crate reads a clock or draws
//! randomness, so every result is reproducible from its inputs alone.
//!
//! # Gaps
//!
//! Deliberately deferred, and honestly named rather than faked:
//!
//! - **NO CRYPTOGRAPHY. This crate does not verify signatures.** IP-007 calls
//!   for Ed25519 signing with keys from OpenBao and an explicit `alg` allowlist
//!   that refuses `none`/`HS*`/`RS*` (Invariant JWT-01). Signing needs a
//!   dependency and the lockfile is frozen for this wave, so what ships here is
//!   claim-SHAPE validation only: expiry, not-before, audience, issuer, tenant
//!   and scope. A token that carries well-formed claims will pass
//!   [`claims::ClaimsPolicy::validate`] no matter who minted it. This is a
//!   security-relevant limitation: [`inmemory::UnsignedTokenIssuer`] is a test
//!   double whose checksum is a non-cryptographic FNV-1a-64 that any caller can
//!   recompute, so it detects corruption and NOT forgery
//!   (`forged_token_is_accepted_because_there_is_no_signature` in
//!   `tests/isolation_policy.rs` proves exactly that). Nothing here may be put
//!   on a trust boundary until a real verifier implements [`JwtVerifier`].
//! - **No Postgres adapter.** IP-006's `adapter-postgres` executes the DDL via
//!   `sqlx` and reads back `pg_class.relforcerowsecurity` to prove FORCE landed.
//!   That needs a database driver; [`inmemory::InMemoryRlsInstaller`] stands in
//!   and verifies the rendered DDL rather than the server's catalog. The
//!   post-install catalog probe and the synthetic cross-tenant row probe are
//!   therefore NOT covered by this crate's tests.
//! - **No async.** The scaffold's ports are synchronous, and every port here
//!   stays synchronous; an async runtime is a dependency.
//! - **No key rotation worker, no manifest parsing.** The 30-day rotation cycle,
//!   the `JwtSigningKeyRotated` event emission and the YAML/waiver manifest
//!   readers all need I/O or a serializer. [`SigningKeyStore`] exposes the
//!   fingerprint seam they will plug into.
//! - **No audit-chain or sustainability emission.** ADR-0344 per-call cost/CO2
//!   rows are an adapter concern and are not emitted from this pure layer.
//! - **The tenant-id rule here is STRICTER than `tenancy/core/domain`'s, and
//!   the two are not reconciled.** `Tenant::new` accepts any id where
//!   `starts_with("ten_") && len() > 4`; [`claims::tenant_id_is_well_formed`]
//!   additionally requires `[a-z0-9_-]` and a 64-byte ceiling, because the
//!   validated tenant is interpolated into `SET app.current_tenant_id`. A
//!   tenant legally created as `ten_ACME` would therefore authenticate nowhere.
//!   This crate cannot close the divergence — it holds no dependency on the
//!   domain crate and the lockfile is frozen — so it is named here, named in
//!   the function's own doc, named in the rejection message an operator reads,
//!   and pinned by
//!   `tenant_id_rule_is_deliberately_stricter_than_the_domain_crate`. Narrowing
//!   `tenancy/core/domain::Tenant::new` to this class is follow-up work.
//! - **No manifest loader, so coverage is opt-in at the call site.** The
//!   registry and [`rls::check_required_coverage`] exist and are enforced by
//!   [`rls::render_required_manifest_ddl`], but nothing in-tree yet reads
//!   `policy/rls/*.yaml` and calls it; whoever writes that loader must use the
//!   covering entry point, not the raw renderer.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod claims;
pub mod inmemory;
pub mod rls;

pub use claims::{
    ClaimsError, ClaimsPolicy, TenantClaims, TokenError, UnixSeconds, ValidatedClaims,
};
pub use inmemory::{InMemoryRlsInstaller, UnsignedTokenIssuer};
pub use rls::{
    CANONICAL_TENANT_SETTING, IdentifierField, REQUIRED_TENANT_BOUND_TABLES, RlsError,
    canonical_predicate, check_required_coverage, render_manifest_ddl, render_policy_ddl,
    render_required_manifest_ddl, required_tenant_bound_tables, validate_identifier,
    validate_rendered_ddl,
};

/// A tenant-bound table that MUST have RLS FORCE applied.
///
/// Construct with [`TenantBoundTable::new`] to get identifier validation up
/// front; the fields stay public because the scaffold published them, so the
/// renderer re-validates on every call rather than trusting the type.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantBoundTable {
    pub schema: String,        // data_class: INTERNAL_ONLY
    pub table: String,         // data_class: INTERNAL_ONLY
    pub tenant_column: String, // data_class: INTERNAL_ONLY
}

impl TenantBoundTable {
    /// Build a tenant-bound table, rejecting any identifier that would not
    /// survive [`validate_identifier`].
    pub fn new(
        schema: impl Into<String>,
        table: impl Into<String>,
        tenant_column: impl Into<String>,
    ) -> Result<Self, RlsError> {
        let bound = Self {
            schema: schema.into(),
            table: table.into(),
            tenant_column: tenant_column.into(),
        };
        bound.validate()?;
        Ok(bound)
    }

    /// Re-check every identifier this table interpolates into DDL.
    pub fn validate(&self) -> Result<(), RlsError> {
        validate_identifier(IdentifierField::Schema, &self.schema)?;
        validate_identifier(IdentifierField::Table, &self.table)?;
        validate_identifier(IdentifierField::TenantColumn, &self.tenant_column)?;
        Ok(())
    }

    /// The `schema.table` form used in DDL, valid only after [`Self::validate`].
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.table)
    }

    /// Deterministic sort key: schema first, then table. Two manifests holding
    /// the same tables in different orders render identically because of this.
    pub fn sort_key(&self) -> (&str, &str) {
        (self.schema.as_str(), self.table.as_str())
    }
}

/// Row-level-security policy specification.
///
/// The predicate is not free-form: [`render_policy_ddl`] refuses to emit a
/// policy whose `using_expr` or `check_expr` differs from the canonical
/// tenant predicate for its own tenant column. A tenant-isolation crate that
/// let callers hand-write predicates would be a crate that lets callers turn
/// isolation off by typo.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RlsPolicy {
    pub table: TenantBoundTable, // data_class: INTERNAL_ONLY
    pub policy_name: String,     // data_class: INTERNAL_ONLY
    pub using_expr: String,      // data_class: INTERNAL_ONLY
    pub check_expr: String,      // data_class: INTERNAL_ONLY
}

impl RlsPolicy {
    /// The canonical tenant-isolation policy for `table`: read and write are
    /// both constrained to the current tenant setting.
    pub fn isolation_for(
        table: TenantBoundTable,
        policy_name: impl Into<String>,
    ) -> Result<Self, RlsError> {
        table.validate()?;
        let policy_name = policy_name.into();
        validate_identifier(IdentifierField::PolicyName, &policy_name)?;
        let predicate = canonical_predicate(&table.tenant_column)?;
        Ok(Self {
            table,
            policy_name,
            using_expr: predicate.clone(),
            check_expr: predicate,
        })
    }
}

/// Sealed port for installing / verifying / auditing RLS policies.
pub trait RlsInstaller {
    /// Apply `policy` to the target database. Implementations MUST refuse any
    /// policy whose rendered DDL omits `FORCE ROW LEVEL SECURITY`, via
    /// [`rls::validate_rendered_ddl`] applied to the bytes they are about to
    /// execute.
    fn install(&self, policy: &RlsPolicy) -> Result<(), IsolationKernelError>;
    /// Report whether `policy` is installed exactly as it would render today.
    fn verify(&self, policy: &RlsPolicy) -> Result<bool, IsolationKernelError>;
}

/// Ed25519 JWT issuer port (IP-007).
///
/// The real implementation signs; see the crate-level Gaps paragraph — the only
/// implementation in-tree today does not.
pub trait JwtIssuer {
    /// Encode `claims` (key/value pairs) into a token.
    fn issue(&self, claims: &[(String, String)]) -> Result<String, IsolationKernelError>;
}

/// JWT verifier port (IP-007).
pub trait JwtVerifier {
    /// Decode `token` back into claim pairs, rejecting anything malformed.
    fn verify(&self, token: &str) -> Result<Vec<(String, String)>, IsolationKernelError>;
}

/// OpenBao-backed signing key store port (IP-007).
pub trait SigningKeyStore {
    /// The fingerprint of the currently active signing key, as advertised to
    /// verifiers.
    fn current_key_fingerprint(&self) -> Result<String, IsolationKernelError>;
}

/// Failures crossing an isolation-policy port boundary.
///
/// Every variant that has a cause CARRIES that cause. An operator reading one
/// log line has to be able to see which table, which field and which character
/// was rejected; flattening a `RlsError::IdentifierBadLeadingChar { field,
/// found }` into a context-free sentence turns a twenty-table manifest into a
/// hand bisection. [`std::error::Error::source`] is implemented so the standard
/// error chain walks all the way down.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IsolationKernelError {
    /// A policy failed to render; the rendering error is retained verbatim.
    PolicyMalformed { source: RlsError },
    /// DDL was refused at the install boundary — it did not FORCE row-level
    /// security on the named table.
    InstallFailed {
        qualified_name: String,
        source: RlsError,
    },
    /// The installer could not answer whether a policy is installed.
    VerifyFailed,
    /// The issuer port could not mint a token.
    JwtSignFailed,
    /// The verifier port could not decode a token: wrong prefix, bad checksum,
    /// or a payload that is not the canonical wire form.
    JwtVerifyFailed,
    /// A token decoded, but its claims were refused; the claim error is
    /// retained verbatim rather than flattened.
    ClaimsRejected { source: ClaimsError },
    /// The signing key store had no usable key.
    KeyStoreUnavailable,
}

impl core::fmt::Display for IsolationKernelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PolicyMalformed { source } => {
                write!(f, "rls policy is malformed and was not rendered: {source}")
            }
            Self::InstallFailed {
                qualified_name,
                source,
            } => write!(
                f,
                "rls policy install failed for {qualified_name}: {source}"
            ),
            Self::VerifyFailed => f.write_str("rls policy verification failed"),
            Self::JwtSignFailed => f.write_str("token issuance failed"),
            Self::JwtVerifyFailed => f.write_str("token decoding failed"),
            Self::ClaimsRejected { source } => write!(f, "token claims rejected: {source}"),
            Self::KeyStoreUnavailable => f.write_str("signing key store is unavailable"),
        }
    }
}

impl std::error::Error for IsolationKernelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PolicyMalformed { source } | Self::InstallFailed { source, .. } => Some(source),
            Self::ClaimsRejected { source } => Some(source),
            Self::VerifyFailed
            | Self::JwtSignFailed
            | Self::JwtVerifyFailed
            | Self::KeyStoreUnavailable => None,
        }
    }
}

impl From<RlsError> for IsolationKernelError {
    fn from(source: RlsError) -> Self {
        Self::PolicyMalformed { source }
    }
}
