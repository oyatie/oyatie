//! oya-cloud-iam-pdp binary entrypoint.
//!
//! Boots the cloud-iam policy-decision-point service (REST + gRPC) from
//! environment configuration and drains gracefully on SIGTERM/ctrl-c
//! (K8s-native pod lifecycle). Panic-free start-up path: every failure is a
//! structured log line + non-zero exit (ADR-0083 Tier 3) — a policy-load
//! failure is a BOOT REFUSAL, never a degraded serve.

use oya_cloud_iam_pdp_app::{PdpConfig, observability, server};
use tracing::{error, info};

/// Resolve on SIGTERM (K8s pod termination) or ctrl-c (local runs).
async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            error!("ctrl-c handler failed; continuing to serve");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sigterm) => {
                sigterm.recv().await;
            }
            Err(err) => {
                error!(error = %err, "SIGTERM handler failed; continuing to serve");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}

#[tokio::main]
async fn main() {
    observability::init();

    let config = match PdpConfig::from_env() {
        Ok(config) => config,
        Err(err) => {
            error!(error = %err, "configuration rejected");
            std::process::exit(1);
        }
    };

    // TODO(ADR-0561 slice-1b-iii): build an `MtlsContext::from_env` from the
    // operator-reconciled projected SVID Secret (K8s cert-delivery) + cloud-kms
    // signer, then call `server::start_with_mtls(&config, Some(ctx))`. The live
    // rustls transport + custom ClientCertVerifier + the PEP-at-call-site are
    // delivered (slice-1b-ii) and exercised by the real-handshake E2E fixtures;
    // only the runtime bundle-delivery source remains deferred — until it lands
    // the binary boots PLAIN TCP (no env source can satisfy a trust bundle yet).
    let mut handle = match server::start(&config).await {
        Ok(handle) => handle,
        Err(err) => {
            error!(error = %err, "boot refused");
            std::process::exit(1);
        }
    };

    let drain = tokio::select! {
        () = shutdown_signal() => true,
        () = handle.done() => false,
    };
    if drain {
        info!("shutdown signal received; draining");
        handle.shutdown().await;
    } else {
        error!("a server task exited unexpectedly");
        std::process::exit(1);
    }
}
