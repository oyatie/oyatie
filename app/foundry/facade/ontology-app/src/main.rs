//! The `foundry-ontology` binary entrypoint.
//!
//! Boots from the environment and drains gracefully on SIGTERM or ctrl-c.
//! The start-up path is panic-free: every failure is a structured log line
//! and a non-zero exit. A durable store that cannot be opened is a boot
//! refusal, never a degraded serve — and there is no shutdown-time
//! persistence duty, because appends are transactional and the projection is
//! disposable derived state rebuilt by replay at the next boot.

use foundry_ontology_app::{Config, compose, observability, router};
use tracing::{error, info};

/// Resolve on SIGTERM (pod termination) or ctrl-c (local runs).
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
            Err(error) => {
                error!(error = %error, "SIGTERM handler failed; continuing to serve");
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
        Err(error) => {
            error!(error = %error, "configuration rejected");
            std::process::exit(1);
        }
    };
    let listen_addr = config.listen_addr.clone();

    let state = match compose(&config) {
        Ok(state) => state,
        Err(error) => {
            error!(error = %error, "boot refused");
            std::process::exit(1);
        }
    };
    let seen = foundry_ontology_app::observation::observe(&state);
    info!(
        tenants = state.tenant_count(),
        poisoned = seen.poisoned,
        // Reported beside the count so a zero is never read as "none" when
        // it is really "not sampled": at boot nothing is contended, so a
        // non-zero here is itself a finding.
        unsampled_tenants = seen.unknown,
        "composed"
    );

    let listener = match tokio::net::TcpListener::bind(&listen_addr).await {
        Ok(listener) => listener,
        Err(error) => {
            error!(error = %error, address = %listen_addr, "the listener could not bind");
            std::process::exit(1);
        }
    };
    info!(address = %listen_addr, "serving");

    if let Err(error) = axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!(error = %error, "the server exited unexpectedly");
        std::process::exit(1);
    }
    info!("drained");
}
