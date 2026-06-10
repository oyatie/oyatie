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

use oya_identity_oidc_issuer_kernel::{IssuerError, IssuerUrl};
use oya_identity_workload_authz_cedar_adapter::{CedarAuthzError, CedarWorkloadAuthorizer};
use oya_identity_workload_oidc_adapter::ValidationConfig;

use crate::AppState;
use crate::config::Config;
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
        }
    }
}

impl std::error::Error for StartError {}

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
    Ok(Arc::new(WorkloadAuthzState::new(
        repository,
        denylist,
        authorizer,
        jwks,
        ValidationConfig::new(&config.issuer, &config.audience),
        TracingAuditSink::new(),
    )))
}

/// Boot the service: build state, bind both listeners, spawn both servers.
///
/// # Errors
/// Returns [`StartError`] when state composition or a bind fails.
pub async fn start(config: &Config) -> Result<ServiceHandle, StartError> {
    let state = build_state(config)?;

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
    // SCIM provisioning surface: same offline JWKS + issuer/audience material
    // as the authorize path, scim.manage scope required (fail-closed guard).
    let scim_state = Arc::new(ScimSurfaceState::new(
        format!("{}{}", config.issuer.trim_end_matches('/'), SCIM_BASE),
        jwks_from_json(&read_file(&config.jwks_path)?).map_err(StartError::Jwks)?,
        ValidationConfig::new(&config.issuer, &config.audience),
        default_now_epoch_seconds,
        Arc::clone(&state),
    ));
    rest_router = rest_router.merge(build_scim_router(scim_state));
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

    info!(rest = %rest_addr, grpc = %grpc_addr, "oya-identity serving");
    Ok(ServiceHandle {
        rest_addr,
        grpc_addr,
        shutdown_tx,
        rest_task,
        grpc_task,
    })
}
