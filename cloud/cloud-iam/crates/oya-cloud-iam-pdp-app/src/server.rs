//! Service composition: config -> bundle -> embedded Cedar PDP -> bound
//! REST + gRPC servers.
//!
//! `start` is the single boot path used by both `main` and the E2E tests, so
//! the tested wiring IS the production wiring (the oya-identity precedent).
//! Boot is fail-closed: the policy bundle must load AND strict-validate
//! through the shared Cedar engine BEFORE either socket binds — a process
//! that cannot prove its policy set never serves a single request.

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tonic::transport::server::TcpIncoming;
use tracing::{error, info};

use oya_cloud_iam_pdp_bundle_file_adapter::FilePolicyBundleStore;
use oya_cloud_iam_pdp_kernel::{BundleStoreError, PdpConfig, PolicyBundleStore};
use oya_shared_pdp_adapter_cedar::CedarPdp;
use oya_shared_pdp_kernel::{PdpError, PolicyDecisionPoint as _};

use crate::audit::TracingDecisionAuditSink;
use crate::idgen::SystemUlidIdGenerator;
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
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bundle(err) => write!(f, "policy bundle unavailable, refusing to boot: {err}"),
            Self::PolicyLoad(err) => {
                write!(f, "policy bundle rejected, refusing to boot: {err}")
            }
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
    let store = FilePolicyBundleStore::new(&config.bundle_path);
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

/// Boot the service: build state, bind both listeners, spawn both servers.
///
/// # Errors
/// [`StartError`] when state composition or a bind fails (boot refusal).
pub async fn start(config: &PdpConfig) -> Result<ServiceHandle, StartError> {
    let state = build_state(config)?;

    let rest_listener =
        TcpListener::bind(&config.rest_addr)
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

    let grpc_listener =
        TcpListener::bind(&config.grpc_addr)
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

    let rest_router = rest::build_router(Arc::clone(&state));
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

    info!(rest = %rest_addr, grpc = %grpc_addr, "oya-cloud-iam-pdp serving");
    Ok(ServiceHandle {
        rest_addr,
        grpc_addr,
        shutdown_tx,
        rest_task,
        grpc_task,
    })
}
