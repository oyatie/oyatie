use super::*;

/// A failure on the boot path.
#[derive(Debug)]
pub enum StartError {
    /// A configured file could not be read.
    Io {
        path: String,
        source: std::io::Error,
    },
    /// The JWKS document was rejected.
    Jwks(JwksParseError),
    /// The Cedar policy set was rejected.
    Cedar(CedarAuthzError),
    /// The principal seed was rejected.
    Seed(SeedError),
    /// The issuer signing key or issuer identity was rejected.
    Issuer(IssuerError),
    /// A listener could not bind.
    Bind {
        addr: String,
        source: std::io::Error,
    },
    /// The durable SCIM store could not be composed: empty/invalid
    /// `OYATIE_BACKBONE_POSTGRES_URL`, an sqlx connect failure, or the
    /// RLS-enforceability guard fired (connected role carries
    /// `rolsuper`/`rolbypassrls`, or is not a member of the
    /// `identity_scim_runtime` policy-subject role). The service REFUSES to serve
    /// rather than silently downgrade to the in-memory store or allow isolation
    /// to be bypassed — there is NO fallback when a URL is configured.
    ///
    /// Note: the guard is necessary but not sufficient for full tenant
    /// isolation; full isolation additionally requires that
    /// `identity_scim_runtime` exists provisioned with NOBYPASSRLS (deferred
    /// `0000_runtime_role.sql` follow-up, mirroring data-outbox /
    /// tenant-lifecycle).
    Store(String),
    Passkey(String),
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "cannot read {path}: {source}"),
            Self::Jwks(err) => write!(f, "JWKS rejected: {err}"),
            Self::Cedar(err) => write!(f, "Cedar policy set rejected: {err}"),
            Self::Seed(err) => write!(f, "principal seed rejected: {err}"),
            Self::Issuer(err) => write!(f, "issuer signing setup rejected: {err:?}"),
            Self::Bind { addr, source } => write!(f, "cannot bind {addr}: {source}"),
            Self::Store(err) => {
                write!(f, "SCIM store unavailable, refusing to serve: {err}")
            }
            Self::Passkey(err) => write!(f, "passkey configuration rejected: {err}"),
        }
    }
}

impl std::error::Error for StartError {}

/// The SCIM store backend selected by [`select_scim_store_kind`] from the runtime
/// config.
///
/// This enum is the authoritative decision socket: the no-fallback property (a
/// configured Postgres URL must NEVER silently degrade to in-memory) is enforced
/// structurally — only `None`/empty URL maps to `InMemory`.
#[derive(Debug, PartialEq, Eq)]
pub enum ScimStoreSelection {
    /// Durable Postgres SCIM stores; the contained URL is non-empty.
    Postgres(String),
    /// In-memory SCIM stores (single-node dev bring-up; no URL configured).
    InMemory,
}

/// `OYATIE_BACKBONE_POSTGRES_URL` — the durable SCIM store DSN (the SAME env name
/// the tenancy facade reads for its store, so the backbone has one source of
/// truth). Present + non-empty selects the durable Postgres SCIM stores; absent
/// or empty selects the in-memory dev stores.
pub const ENV_SCIM_DATABASE_URL: &str = "OYATIE_BACKBONE_POSTGRES_URL";
pub const ENV_PASSKEY_ORIGIN: &str = "OYATIE_IDENTITY_PASSKEY_ORIGIN";

/// Pure SCIM store-selection function: maps the raw (pre-normalized) database URL
/// option onto a [`ScimStoreSelection`]. `None`, empty, or whitespace-only
/// strings all select `InMemory`; any non-empty trimmed URL selects `Postgres`.
///
/// Extracted from [`start`] so the no-fallback decision can be unit-tested
/// without a network, a runtime, or any side effects.
#[must_use]
pub fn select_scim_store_kind(database_url: Option<String>) -> ScimStoreSelection {
    match database_url
        .map(|u| u.trim().to_owned())
        .filter(|u| !u.is_empty())
    {
        Some(url) => ScimStoreSelection::Postgres(url),
        None => ScimStoreSelection::InMemory,
    }
}
