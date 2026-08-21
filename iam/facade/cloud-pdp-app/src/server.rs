//! Service composition: config -> bundle -> embedded Cedar PDP -> bound
//! REST + gRPC servers.
//!
//! [`boot_from_config`] is the single PRODUCTION boot body run by both `main`
//! and the production-path closure E2E, so the tested wiring IS the production
//! wiring (the oya-identity "tested wiring IS production wiring" precedent): it
//! builds the [`MtlsContext`] from the delivered cert mount and boots over mTLS
//! via [`start_with_mtls`], fail-closed. `start`/`start_with_mtls` remain the
//! shared boot bodies (and test helpers); `start` (plain TCP) is now unreachable
//! from `main`. Boot is fail-closed: the policy bundle must load AND
//! strict-validate through the shared Cedar engine, and the mTLS material must
//! compose, BEFORE either socket binds — a process that cannot prove its policy
//! set or its trust root never serves a single request.

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tonic::transport::server::TcpIncoming;
use tracing::{error, info};

use iam_cloud_pdp_bundle_file::FilePolicyBundleStore;
use iam_cloud_pdp_kernel::{BundleStoreError, PdpConfig, PolicyBundleStore};
use iam_pdp_cedar::CedarPdp;
use shared_pdp_kernel::{PdpError, PolicyDecisionPoint as _};

use std::path::Path;

use crate::audit::TracingDecisionAuditSink;
use crate::idgen::SystemUlidIdGenerator;
use crate::mtls::MtlsBootError;
use crate::mtls_transport::{self, MtlsContext, MtlsMaterialError};
use crate::{PdpState, grpc, rest};

/// A failure on the boot path. Every variant REFUSES the boot (exit
/// non-zero from `main`); none degrades to serving without policy.
#[derive(Debug)]
pub enum StartError {
    /// The policy-bundle store could not produce a bundle.
    Bundle(BundleStoreError),
    /// The bundle failed compile/template-link/strict-validation in the
    /// shared Cedar engine.
    PolicyLoad(PdpError),
    /// A listener could not bind.
    Bind {
        addr: String,
        source: std::io::Error,
    },
    /// The mTLS transport could not be composed (empty trust bundle or a
    /// rejected server identity) — boot-refused, mirroring [`Self::Bundle`]: a
    /// process that cannot prove a trust root or present its own identity must
    /// never serve a caller (the fail-closed mTLS boot, ADR-0561 slice-1b-ii).
    Mtls(MtlsBootError),
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bundle(err) => write!(f, "policy bundle unavailable, refusing to boot: {err}"),
            Self::PolicyLoad(err) => {
                write!(f, "policy bundle rejected, refusing to boot: {err}")
            }
            Self::Bind { addr, source } => write!(f, "cannot bind {addr}: {source}"),
            Self::Mtls(err) => write!(f, "mTLS transport unavailable, refusing to boot: {err}"),
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

    /// Wait for either server task to exit on its own (fatal transport
    /// error).
    pub async fn done(&mut self) {
        tokio::select! {
            _ = &mut self.rest_task => {}
            _ = &mut self.grpc_task => {}
        }
    }
}

/// Compose the decision state from configuration (fail-closed).
///
/// # Errors
/// [`StartError`] when the bundle cannot be loaded or rejected by the
/// engine — the caller MUST refuse to serve.
pub fn build_state(config: &PdpConfig) -> Result<Arc<PdpState>, StartError> {
    let store = FilePolicyBundleStore::new(&config.bundle_path, &config.bundle_trust_dir);
    let bundle = store.load().map_err(StartError::Bundle)?;
    let pdp = CedarPdp::load(
        &bundle,
        Arc::new(SystemUlidIdGenerator::new()),
        config.decision_cache_capacity,
    )
    .map_err(StartError::PolicyLoad)?;
    info!(
        source = %store.describe(),
        policy_version = %pdp.loaded_policy_version().as_str(),
        "policy bundle loaded and strict-validated",
    );
    Ok(Arc::new(PdpState::new(
        pdp,
        Arc::new(TracingDecisionAuditSink::new()),
    )))
}

/// Boot the service on PLAIN TCP (no caller authentication). This is the legacy
/// boot path; it delegates to [`start_with_mtls`] with no [`MtlsContext`] so the
/// plain and mTLS boots share ONE body and can never drift.
///
/// # Errors
/// [`StartError`] when state composition or a bind fails (boot refusal).
pub async fn start(config: &PdpConfig) -> Result<ServiceHandle, StartError> {
    start_with_mtls(config, None).await
}

/// Boot the service, optionally over mTLS.
///
/// When `mtls` is `Some`, both listeners terminate a rustls handshake REQUIRING a
/// verified client SVID, and the PEP binds the caller's tenant from the SVID
/// (the #717 closure, live). When `None`, plain TCP with the legacy
/// verbatim-tenant path runs. Both share the same fail-closed boot: the state
/// must compile and (for mTLS) the trust bundle + server identity must compose
/// BEFORE either socket binds.
///
/// # Errors
/// [`StartError`] when state composition, an mTLS-context build, or a bind fails
/// (boot refusal).
pub async fn start_with_mtls(
    config: &PdpConfig,
    mtls: Option<MtlsContext>,
) -> Result<ServiceHandle, StartError> {
    let state = build_state(config)?;

    // Compose the mTLS acceptor BEFORE binding: a rejected bundle/identity is a
    // boot refusal, never a degraded plain serve.
    let acceptor = match &mtls {
        Some(ctx) => Some(ctx.build_acceptor().map_err(StartError::Mtls)?),
        None => None,
    };
    let bundle = mtls.as_ref().map(MtlsContext::bundle);
    // The PDP's own cell authority (derived from its server SVID) pins a caller's
    // cell. ALWAYS present whenever an `MtlsContext` is in use: deriving it is a
    // fail-closed boot precondition in `MtlsContext::new`, so an mTLS serve can
    // NEVER run without the cell pin (belt-and-suspenders — this `Some(..)` is
    // guaranteed by construction, never an unpinned mTLS serve).
    let expected_cell = mtls
        .as_ref()
        .map(|ctx| ctx.expected_cell_authority().to_owned());

    // Belt-and-suspenders fail-closed gate: it is IMPOSSIBLE to serve mTLS
    // without a derived cell pin. `MtlsContext::new` already guarantees the pin,
    // so this can only fire on a future regression — and if it ever did, we
    // boot-refuse rather than serve mTLS with cell-isolation silently disabled.
    if mtls.is_some() && expected_cell.is_none() {
        return Err(StartError::Mtls(MtlsBootError::CellPinUndeterminable(
            "mTLS context present without a derived cell pin (cell-isolation would \
             be disabled) — refusing to serve"
                .to_string(),
        )));
    }

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

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let rest_task = match (acceptor.clone(), bundle.clone()) {
        (Some(acc), Some(bundle)) => {
            let router = rest::build_router_mtls(Arc::clone(&state), bundle, expected_cell.clone());
            let mut rest_shutdown = shutdown_rx.clone();
            tokio::spawn(async move {
                let shutdown = async move {
                    let _ = rest_shutdown.changed().await;
                };
                serve_rest_mtls(rest_listener, router, acc, shutdown).await;
            })
        }
        _ => {
            let router = rest::build_router(Arc::clone(&state));
            let mut rest_shutdown = shutdown_rx.clone();
            tokio::spawn(async move {
                let shutdown = async move {
                    let _ = rest_shutdown.changed().await;
                };
                if let Err(err) = axum::serve(rest_listener, router)
                    .with_graceful_shutdown(shutdown)
                    .await
                {
                    error!(error = %err, "REST server exited with error");
                }
            })
        }
    };

    let grpc_task = match (acceptor, bundle) {
        (Some(acc), Some(bundle)) => {
            let state = Arc::clone(&state);
            let mut grpc_shutdown = shutdown_rx;
            tokio::spawn(async move {
                let shutdown = async move {
                    let _ = grpc_shutdown.changed().await;
                };
                let incoming = grpc_tls_incoming(grpc_listener, acc);
                if let Err(err) =
                    grpc::serve_mtls(state, bundle, expected_cell, incoming, shutdown).await
                {
                    error!(error = %err, "gRPC server exited with error");
                }
            })
        }
        _ => {
            let grpc_incoming = TcpIncoming::from(grpc_listener).with_nodelay(Some(true));
            let mut grpc_shutdown = shutdown_rx;
            tokio::spawn(async move {
                let shutdown = async move {
                    let _ = grpc_shutdown.changed().await;
                };
                if let Err(err) = grpc::serve(state, grpc_incoming, shutdown).await {
                    error!(error = %err, "gRPC server exited with error");
                }
            })
        }
    };

    info!(
        rest = %rest_addr,
        grpc = %grpc_addr,
        mtls = mtls.is_some(),
        "oya-cloud-iam-pdp serving",
    );
    Ok(ServiceHandle {
        rest_addr,
        grpc_addr,
        shutdown_tx,
        rest_task,
        grpc_task,
    })
}

/// A production-boot failure: either the delivered cert mount could not be turned
/// into an [`MtlsContext`] (fail-closed — never plain TCP), or the service
/// composition/bind refused. `main` maps either to a non-zero exit (BOOT
/// REFUSAL).
#[derive(Debug)]
pub enum BootError {
    /// The mTLS cert mount was absent/empty/malformed — the production boot
    /// REFUSES rather than serving plain TCP (ADR-0561 slice-1b-iii).
    Material(MtlsMaterialError),
    /// State composition or a listener bind failed.
    Start(StartError),
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Material(err) => {
                write!(f, "mTLS cert material rejected, refusing to boot: {err}")
            }
            Self::Start(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for BootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Material(err) => Some(err),
            Self::Start(err) => Some(err),
        }
    }
}

/// THE production boot decision (G002 slice-1b-iii-a/b; ADR-0561). Resolve the
/// delivered mTLS cert mount from `config.mtls_cert_dir`, build an [`MtlsContext`]
/// from it (fail-closed — an absent/empty/malformed mount is a HARD
/// [`BootError::Material`], NEVER a downgrade to plain TCP), then boot the service
/// over mTLS via [`start_with_mtls`].
///
/// This is the SINGLE boot body `main` runs, so the tested wiring IS the
/// production wiring (the oya-identity precedent; see this module's doc). The
/// closure E2E exercises THIS function, not a parallel test-only path.
///
/// # Errors
/// [`BootError::Material`] when the cert mount cannot produce an `MtlsContext`;
/// [`BootError::Start`] when composition or a bind fails. Either is a boot
/// refusal (`main` exits non-zero).
pub async fn boot_from_config(config: &PdpConfig) -> Result<ServiceHandle, BootError> {
    let dir = Path::new(&config.mtls_cert_dir);
    let ctx = MtlsContext::from_path(dir).map_err(BootError::Material)?;
    start_with_mtls(config, Some(ctx))
        .await
        .map_err(BootError::Start)
}

/// Build the gRPC mTLS incoming stream: accept TCP, terminate the rustls
/// handshake (dropping rogue/no-cert connections), yield connected streams.
fn grpc_tls_incoming(
    listener: TcpListener,
    acceptor: tokio_rustls::TlsAcceptor,
) -> impl futures_core::Stream<Item = Result<mtls_transport::PeerCertTlsStream, std::io::Error>> {
    async_stream::stream! {
        loop {
            let tcp = match mtls_transport::accept_tcp(&listener).await {
                Ok(tcp) => tcp,
                Err(err) => {
                    error!(error = %err, "gRPC TCP accept failed");
                    continue;
                }
            };
            match mtls_transport::accept_grpc(&acceptor, tcp).await {
                Ok(stream) => yield Ok(stream),
                Err(err) => {
                    // Fail-closed: a failed handshake (rogue / no client cert)
                    // drops the connection; it never reaches a handler.
                    tracing::debug!(error = %err, "gRPC mTLS handshake rejected");
                }
            }
        }
    }
}

/// Serve the REST surface over mTLS with a manual hyper-util accept loop
/// (axum 0.8 has no built-in rustls serve helper). Each connection terminates a
/// rustls handshake requiring a verified client SVID; the captured peer leaf is
/// layered as a per-connection `Extension` so the PEP authenticates the caller.
async fn serve_rest_mtls<F>(
    listener: TcpListener,
    router: axum::Router,
    acceptor: tokio_rustls::TlsAcceptor,
    shutdown: F,
) where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto::Builder;
    use hyper_util::service::TowerToHyperService;
    use tower::ServiceExt as _;

    let shutdown = std::pin::pin!(shutdown);
    let mut shutdown = shutdown;
    loop {
        let tcp = tokio::select! {
            () = &mut shutdown => break,
            accepted = mtls_transport::accept_tcp(&listener) => match accepted {
                Ok(tcp) => tcp,
                Err(err) => {
                    error!(error = %err, "REST TCP accept failed");
                    continue;
                }
            },
        };
        let acceptor = acceptor.clone();
        let router = router.clone();
        tokio::spawn(async move {
            let (tls, peer) = match mtls_transport::accept_rest(&acceptor, tcp).await {
                Ok(pair) => pair,
                Err(err) => {
                    // Fail-closed: rogue / no client cert ⇒ drop the connection.
                    tracing::debug!(error = %err, "REST mTLS handshake rejected");
                    return;
                }
            };
            // Inject the verified peer leaf as a per-connection Extension so
            // `authorize` runs the PEP on it.
            let svc = router
                .layer(axum::Extension(peer))
                .into_service::<axum::body::Body>();
            let hyper_svc = TowerToHyperService::new(svc.map_request(
                |req: hyper::Request<hyper::body::Incoming>| req.map(axum::body::Body::new),
            ));
            let io = TokioIo::new(tls);
            if let Err(err) = Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(io, hyper_svc)
                .await
            {
                tracing::debug!(error = %err, "REST mTLS connection ended");
            }
        });
    }
}
