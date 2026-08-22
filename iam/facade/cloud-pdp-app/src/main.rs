//! cloud-iam-pdp binary entrypoint.
//!
//! Boots the cloud-iam policy-decision-point service (REST + gRPC) from
//! environment configuration and drains gracefully on SIGTERM/ctrl-c
//! (K8s-native pod lifecycle). Panic-free start-up path: every failure is a
//! structured log line + non-zero exit (ADR-0083 Tier 3) — a policy-load
//! failure is a BOOT REFUSAL, never a degraded serve.

use iam_cloud_pdp_app::{PdpConfig, observability, server};
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

    // Production boot (ADR-0561 slice-1b-iii-a/b): build an `MtlsContext` from the
    // delivered cert mount (`OYATIE_CLOUD_IAM_PDP_MTLS_CERT_DIR`, the kubernetes.io/tls
    // Secret projection) and boot over mTLS via `server::start_with_mtls`. This is
    // FAIL-CLOSED: an absent/empty/malformed mount is a BOOT REFUSAL (exit 1),
    // NEVER a downgrade to plain TCP. `server::boot_from_config` is the SAME body
    // the production-path closure E2E exercises. The in-cluster delivery source
    // (the SVID operator) is slice-1b-iii-c — until it lands the prod pod
    // fail-closes without the mount, which is correct.
    let mut handle = match server::boot_from_config(&config).await {
        Ok(handle) => handle,
        Err(err) => {
            error!(error = %err, "boot refused (mTLS fail-closed)");
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
