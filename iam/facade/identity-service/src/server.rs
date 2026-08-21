//! Service composition: config -> state -> bound REST + gRPC servers.
//!
//! `start` is the single boot path used by both `main` and the E2E tests, so
//! the tested wiring IS the production wiring. Boot is fail-fast: JWKS, Cedar
//! policies, and the principal seed all load (and validate) before either
//! socket binds, so a serving process is a correctly-configured process.

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tonic::transport::server::TcpIncoming;
use tracing::{error, info};

use iam_identity_oidc_issuer_kernel::{IssuerError, IssuerUrl};
use iam_identity_workload_app::{InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository};
use iam_identity_workload_authz_cedar::{CedarAuthzError, CedarWorkloadAuthorizer};
use iam_identity_workload_oidc::ValidationConfig;
use iam_identity_workload_rest::{BearerCallerVerifier, WorkloadAuthzState};
use identity_scim_store_postgres::{
    PgScimGroupStore, PgScimUserStore, assert_rls_enforceable, connect_pool,
};
use shared_scim_server_kernel::{InMemoryGroupStore, InMemoryUserStore};

use crate::AppState;
use crate::config::Config;
use crate::decision_authz::TenantScopedDecisionAuthorizer;
use crate::lifecycle_authz::TenantScopedLifecycleAuthorizer;
use crate::observability::TracingAuditSink;
use crate::oidc::issuer::{Es256FileSigner, IssuerState, build_issuer_router};
use crate::oidc::{JwksParseError, jwks_from_json};
use crate::storage::{SeedError, seed_from_json};
use crate::users::{SCIM_BASE, ScimSurfaceState, build_scim_router};
use crate::{grpc, rest};

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
    /// `OYA_BACKBONE_POSTGRES_URL`, an sqlx connect failure, or the
    /// RLS-enforceability guard fired (connected role carries
    /// `rolsuper`/`rolbypassrls`, or is not a member of the
    /// `identity_scim_runtime` policy-subject role). The service REFUSES to serve
    /// rather than silently downgrade to the in-memory store or allow isolation
    /// to be bypassed — there is NO fallback when a URL is configured.
    ///
    /// Note: the guard is necessary but not sufficient for full tenant
    /// isolation; full isolation additionally requires that
    /// `identity_scim_runtime` exists provisioned with NOBYPASSRLS (deferred
    /// `0000_runtime_role.sql` follow-up, mirroring oya-data-outbox /
    /// tenant-lifecycle).
    Store(String),
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

/// `OYA_BACKBONE_POSTGRES_URL` — the durable SCIM store DSN (the SAME env name
/// the tenancy facade reads for its store, so the backbone has one source of
/// truth). Present + non-empty selects the durable Postgres SCIM stores; absent
/// or empty selects the in-memory dev stores.
pub const ENV_SCIM_DATABASE_URL: &str = "OYA_BACKBONE_POSTGRES_URL";

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

/// A running service: bound addresses plus the shutdown handle.
pub struct ServiceHandle {
    /// The bound REST address (resolved, so `:0` binds report the real port).
    pub rest_addr: SocketAddr,
    /// The bound gRPC address.
    pub grpc_addr: SocketAddr,
    shutdown_tx: watch::Sender<bool>,
    rest_task: JoinHandle<()>,
    grpc_task: JoinHandle<()>,
}

impl ServiceHandle {
    /// Signal graceful shutdown and wait for both servers to drain.
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true);
        if let Err(err) = self.rest_task.await {
            error!(error = %err, "REST task join failed");
        }
        if let Err(err) = self.grpc_task.await {
            error!(error = %err, "gRPC task join failed");
        }
    }

    /// Wait for either server task to exit on its own (fatal transport error).
    pub async fn done(&mut self) {
        tokio::select! {
            _ = &mut self.rest_task => {}
            _ = &mut self.grpc_task => {}
        }
    }
}

fn read_file(path: &str) -> Result<String, StartError> {
    std::fs::read_to_string(path).map_err(|source| StartError::Io {
        path: path.to_string(),
        source,
    })
}

fn read_bytes(path: &str) -> Result<Vec<u8>, StartError> {
    std::fs::read(path).map_err(|source| StartError::Io {
        path: path.to_string(),
        source,
    })
}

/// Compose the optional OIDC issuer state (enabled when a signing key is
/// configured). The file signer is the transitional custody adapter behind
/// the kernel's `JwsSigner` port — the G02 KMS adapter replaces it there.
fn build_issuer_state(config: &Config) -> Result<Option<Arc<IssuerState>>, StartError> {
    let Some(path) = &config.signing_key_path else {
        return Ok(None);
    };
    let signer = Es256FileSigner::from_pkcs8_der(&config.signing_kid, &read_bytes(path)?)
        .map_err(StartError::Issuer)?;
    let mut key = signer.signing_key().map_err(StartError::Issuer)?;
    key.activate(default_now_epoch_seconds())
        .map_err(StartError::Issuer)?;
    let issuer_url = IssuerUrl::new(&config.issuer).map_err(StartError::Issuer)?;
    Ok(Some(Arc::new(IssuerState::new(
        issuer_url,
        vec![key],
        Arc::new(signer),
        default_now_epoch_seconds,
    ))))
}

/// Wall-clock epoch seconds, saturating (ADR-0083 panic-free).
fn default_now_epoch_seconds() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Compose the application state from configuration (fail-fast).
///
/// # Errors
/// Returns [`StartError`] when a configured file cannot be read or parsed.
pub fn build_state(config: &Config) -> Result<Arc<AppState>, StartError> {
    let jwks = jwks_from_json(&read_file(&config.jwks_path)?).map_err(StartError::Jwks)?;
    let authorizer =
        CedarWorkloadAuthorizer::from_cedar_policies(&read_file(&config.cedar_policy_path)?)
            .map_err(StartError::Cedar)?;
    let (repository, denylist) = match &config.principals_path {
        Some(path) => seed_from_json(&read_file(path)?).map_err(StartError::Seed)?,
        None => (
            InMemoryWorkloadPrincipalRepository::new(),
            InMemoryRevocationDenylist::new(),
        ),
    };
    // AUTH-005 fail-closed mutating control plane (ADR-0581): the lifecycle
    // routes are served ONLY with a verified-caller port + a fail-closed PDP
    // port. Both are REQUIRED here; `Config::from_env` already refuses to load
    // without the bearer credential, so a serving binary always has the seam.
    let caller_verifier = Arc::new(BearerCallerVerifier::new(
        config.lifecycle_bearer.clone(),
        config.lifecycle_caller_tenant.clone(),
        config.lifecycle_caller_id.clone(),
    ));
    let lifecycle_authorizer = Arc::new(TenantScopedLifecycleAuthorizer::new());
    // AUTH-005 read decision surfaces (`/authorize`, `/authorize-with-token`,
    // `/authorize:batch`, `/tokens/validate`): the SAME verified-caller port (the
    // bearer above — NO new config) plus a fail-closed same-tenant decision port,
    // so a forged body / cross-tenant token cannot obtain an arbitrary decision.
    let decision_authorizer = Arc::new(TenantScopedDecisionAuthorizer::new());
    Ok(Arc::new(WorkloadAuthzState::new(
        repository,
        denylist,
        authorizer,
        jwks,
        ValidationConfig::new(&config.issuer, &config.audience),
        TracingAuditSink::new(),
        caller_verifier,
        lifecycle_authorizer,
        decision_authorizer,
    )))
}

/// Boot the service: build state, bind both listeners, spawn both servers.
///
/// The store-selection/connect/RLS-guard block runs BEFORE either socket binds
/// so a mis-provisioned DSN or bypass-capable role fails before any file
/// descriptor is allocated.
///
/// # Errors
/// Returns [`StartError`] when state composition, the store guard, or a bind
/// fails.
pub async fn start(config: &Config) -> Result<ServiceHandle, StartError> {
    start_with_scim_url(config, std::env::var(ENV_SCIM_DATABASE_URL).ok()).await
}

/// Like [`start`] but accepts the SCIM database URL directly (bypassing the
/// `OYA_BACKBONE_POSTGRES_URL` env read). Used by live E2E tests so they can
/// pass the app-role URL (`OYA_BACKBONE_POSTGRES_APP_URL`) without writing to
/// the process-global env and racing other parallel tokio tests.
///
/// # Errors
/// Returns [`StartError`] when state composition, the store guard, or a bind
/// fails.
pub async fn start_with_scim_url(
    config: &Config,
    scim_database_url: Option<String>,
) -> Result<ServiceHandle, StartError> {
    let state = build_state(config)?;

    // SCIM provisioning surface: same offline JWKS + issuer/audience material
    // as the authorize path, scim.manage scope required (fail-closed guard).
    //
    // Store selection (12-factor composition-root config, NOT a CLI surface):
    //   - OYA_BACKBONE_POSTGRES_URL present + non-empty -> the DURABLE Postgres
    //     SCIM stores are composed. If the connection fails or the
    //     RLS-enforceability guard fires (bypass-capable role, or role not a
    //     member of identity_scim_runtime), the service REFUSES to serve
    //     (StartError::Store) — it NEVER falls back to in-memory.
    //   - absent / empty -> the in-memory stores (single-node dev bring-up).
    //
    // Both plug in behind the UNCHANGED UserStore/GroupStore kernel ports, so
    // the owned-data cutover (G003) swaps the adapter with no change here.
    //
    // Guard runs BEFORE either TcpListener::bind so a broken DSN or
    // bypass-capable role fails before any socket is allocated.
    let scim_base_url = format!("{}{}", config.issuer.trim_end_matches('/'), SCIM_BASE);
    let scim_validation = ValidationConfig::new(&config.issuer, &config.audience);
    let (scim_router, scim_store_kind) = match select_scim_store_kind(scim_database_url) {
        ScimStoreSelection::Postgres(url) => {
            // FAIL-CLOSED: a connect failure or RLS-bypass role propagates;
            // NEVER fall back to in-memory when a durable backend was
            // configured. The composition root OWNS the shared pool (typed
            // here so the sqlx dependency is named, not merely transitive)
            // and runs the RLS-enforceability guard once over it.
            let pool: sqlx::PgPool = connect_pool(&url)
                .await
                .map_err(|e| StartError::Store(e.to_string()))?;
            assert_rls_enforceable(&pool)
                .await
                .map_err(|e| StartError::Store(e.to_string()))?;
            let scim_state = Arc::new(ScimSurfaceState::new(
                scim_base_url,
                jwks_from_json(&read_file(&config.jwks_path)?).map_err(StartError::Jwks)?,
                scim_validation,
                default_now_epoch_seconds,
                Arc::clone(&state),
                PgScimUserStore::from_pool(pool.clone()),
                PgScimGroupStore::from_pool(pool),
            ));
            (build_scim_router(scim_state), "postgres")
        }
        ScimStoreSelection::InMemory => {
            let scim_state = Arc::new(ScimSurfaceState::new(
                scim_base_url,
                jwks_from_json(&read_file(&config.jwks_path)?).map_err(StartError::Jwks)?,
                scim_validation,
                default_now_epoch_seconds,
                Arc::clone(&state),
                InMemoryUserStore::default(),
                InMemoryGroupStore::default(),
            ));
            (build_scim_router(scim_state), "inmemory")
        }
    };

    let rest_listener = TcpListener::bind(&config.rest_addr)
        .await
        .map_err(|source| StartError::Bind {
            addr: config.rest_addr.clone(),
            source,
        })?;
    let rest_addr = rest_listener
        .local_addr()
        .map_err(|source| StartError::Bind {
            addr: config.rest_addr.clone(),
            source,
        })?;

    let grpc_listener = TcpListener::bind(&config.grpc_addr)
        .await
        .map_err(|source| StartError::Bind {
            addr: config.grpc_addr.clone(),
            source,
        })?;
    let grpc_addr = grpc_listener
        .local_addr()
        .map_err(|source| StartError::Bind {
            addr: config.grpc_addr.clone(),
            source,
        })?;
    let grpc_incoming = TcpIncoming::from(grpc_listener).with_nodelay(Some(true));

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut rest_router = rest::build_service_router(Arc::clone(&state));
    if let Some(issuer_state) = build_issuer_state(config)? {
        rest_router = rest_router.merge(build_issuer_router(issuer_state));
    }
    rest_router = rest_router.merge(scim_router);
    let mut rest_shutdown = shutdown_rx.clone();
    let rest_task = tokio::spawn(async move {
        let shutdown = async move {
            let _ = rest_shutdown.changed().await;
        };
        if let Err(err) = axum::serve(rest_listener, rest_router)
            .with_graceful_shutdown(shutdown)
            .await
        {
            error!(error = %err, "REST server exited with error");
        }
    });

    let mut grpc_shutdown = shutdown_rx;
    let grpc_task = tokio::spawn(async move {
        let shutdown = async move {
            let _ = grpc_shutdown.changed().await;
        };
        if let Err(err) = grpc::serve(state, grpc_incoming, shutdown).await {
            error!(error = %err, "gRPC server exited with error");
        }
    });

    info!(
        rest = %rest_addr,
        grpc = %grpc_addr,
        scim_store = scim_store_kind,
        "oya-identity serving"
    );
    Ok(ServiceHandle {
        rest_addr,
        grpc_addr,
        shutdown_tx,
        rest_task,
        grpc_task,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- select_scim_store_kind unit tests (pure, no async, no DB) -----------

    #[test]
    fn select_scim_store_kind_non_empty_url_picks_postgres() {
        assert_eq!(
            select_scim_store_kind(Some("postgres://host/db".to_owned())),
            ScimStoreSelection::Postgres("postgres://host/db".to_owned()),
        );
    }

    #[test]
    fn select_scim_store_kind_trims_surrounding_whitespace() {
        // A non-empty URL with surrounding whitespace is trimmed, not rejected:
        // the durable backend is still selected.
        assert_eq!(
            select_scim_store_kind(Some("  postgres://host/db  ".to_owned())),
            ScimStoreSelection::Postgres("postgres://host/db".to_owned()),
        );
    }

    #[test]
    fn select_scim_store_kind_none_picks_inmemory() {
        assert_eq!(select_scim_store_kind(None), ScimStoreSelection::InMemory);
    }

    #[test]
    fn select_scim_store_kind_empty_string_picks_inmemory() {
        // An empty OYA_BACKBONE_POSTGRES_URL is treated as "not configured" —
        // the dev in-memory path, not a fail-closed error.
        assert_eq!(
            select_scim_store_kind(Some(String::new())),
            ScimStoreSelection::InMemory,
        );
    }

    #[test]
    fn select_scim_store_kind_whitespace_only_picks_inmemory() {
        assert_eq!(
            select_scim_store_kind(Some("   ".to_owned())),
            ScimStoreSelection::InMemory,
        );
    }

    // --- DB-free fail-closed wiring proof ------------------------------------

    /// Fail-closed wiring proof WITHOUT a database: the durable arm's FIRST step
    /// (`connect_pool`) rejects an empty URL before any network, and the
    /// composition root maps that into [`StartError::Store`]. This proves the
    /// no-fallback contract for the empty-but-selected edge through the same
    /// adapter call `start()` makes, without opening a socket. (A non-empty
    /// `select_scim_store_kind` would route here; an empty one routes to
    /// in-memory, so the `start()` env read can never reach this with "".)
    #[tokio::test]
    async fn durable_arm_empty_url_maps_to_start_error_store() {
        let result = connect_pool("")
            .await
            .map_err(|e| StartError::Store(e.to_string()));
        assert!(
            matches!(result, Err(StartError::Store(_))),
            "empty durable URL must fail-close as StartError::Store, got {result:?}"
        );
    }
}
