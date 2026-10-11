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
use iam_identity_scim_store_postgres::{
    PgScimGroupStore, PgScimUserStore, assert_rls_enforceable, connect_pool,
};
use iam_identity_workload_app::{InMemoryRevocationDenylist, InMemoryWorkloadPrincipalRepository};
use iam_identity_workload_authz_cedar::{CedarAuthzError, CedarWorkloadAuthorizer};
use iam_identity_workload_oidc::ValidationConfig;
use iam_identity_workload_rest::{BearerCallerVerifier, WorkloadAuthzState};
use iam_scim_server_kernel::{InMemoryGroupStore, InMemoryUserStore};

use crate::AppState;
use crate::config::Config;
use crate::decision_authz::TenantScopedDecisionAuthorizer;
use crate::lifecycle_authz::TenantScopedLifecycleAuthorizer;
use crate::observability::TracingAuditSink;
use crate::oidc::issuer::{Es256FileSigner, IssuerState, build_issuer_router};
use crate::oidc::{JwksParseError, jwks_from_json};
use crate::passkeys::{PasskeyOrigin, build_router as build_passkey_router};
use crate::storage::{SeedError, seed_from_json};
use crate::users::{
    SCIM_BASE, ScimSurfaceState, authorize_native_user_admin, authorize_scim, build_scim_router,
};
use crate::{grpc, rest};

mod contracts;
pub use contracts::*;

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

mod composition;
pub use composition::build_state;
use composition::*;

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
    start_with_options(
        config,
        std::env::var(ENV_SCIM_DATABASE_URL).ok(),
        std::env::var(ENV_PASSKEY_ORIGIN).ok(),
    )
    .await
}

/// Like [`start`] but accepts the SCIM database URL directly (bypassing the
/// `OYATIE_BACKBONE_POSTGRES_URL` env read). Used by live E2E tests so they can
/// pass the app-role URL (`OYATIE_BACKBONE_POSTGRES_APP_URL`) without writing to
/// the process-global env and racing other parallel tokio tests.
///
/// # Errors
/// Returns [`StartError`] when state composition, the store guard, or a bind
/// fails.
pub async fn start_with_scim_url(
    config: &Config,
    scim_database_url: Option<String>,
) -> Result<ServiceHandle, StartError> {
    start_with_options(config, scim_database_url, None).await
}

/// Compose real passkey routes over the same durable, RLS-guarded SCIM pool.
/// The origin is an exact deployment value, never derived from a request.
pub async fn start_with_passkey_origin(
    config: &Config,
    scim_database_url: Option<String>,
    origin: String,
) -> Result<ServiceHandle, StartError> {
    start_with_options(config, scim_database_url, Some(origin)).await
}

mod serving;
use serving::start_with_options;

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
        // An empty OYATIE_BACKBONE_POSTGRES_URL is treated as "not configured" —
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
