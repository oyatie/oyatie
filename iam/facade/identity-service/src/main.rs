//! identity binary entrypoint.
//!
//! Boots the workload-identity service (REST + gRPC) from environment
//! configuration and drains gracefully on SIGTERM/ctrl-c (K8s-native pod
//! lifecycle). Panic-free start-up path: every failure is a structured log
//! line + non-zero exit (ADR-0083 Tier 3).

use iam_identity_service::config::Config;
use iam_identity_service::{observability, server};
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

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            error!(error = %err, "configuration rejected");
            std::process::exit(1);
        }
    };

    let mut handle = match server::start(&config).await {
        Ok(handle) => handle,
        Err(err) => {
            error!(error = %err, "boot failed");
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
