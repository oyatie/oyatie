//! Managed-Kubernetes control-plane-host binary — composition root (ADR-0376).
//!
//! Fail-closed boot:
//! 1. Read the MANAGEMENT-cluster kubeconfig path from `$OYATIE_MGMT_KUBECONFIG`.
//!    If absent/empty -> typed [`BootError::MissingMgmtKubeconfig`] and a
//!    non-zero exit (NEVER a silent fall-back to the in-memory fake).
//! 2. Build the kube-rs CAPI adapter from that kubeconfig (kube-rs stays
//!    isolated to the adapter crate).
//! 3. Compose [`AppState`] over the adapter and serve the axum admin/status
//!    API on `$OYATIE_LISTEN_ADDR` (default `0.0.0.0:8080`).
//!
//! The live Kamaji/Talos CRD reconcile is honest-deferred inside the adapter:
//! provision/status/teardown return HTTP 501 with a typed boundary until the
//! follow-on ADR wires the real reconcile.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use k8s_control_plane_host_app::{
    CapiControlPlaneHost, authz_from_env, build_router, build_state_capi,
    mgmt_kubeconfig_path_from_env, serve,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber_init();

    // Fail-closed: the platform-admin bearer is mandatory (refuses an empty
    // secret) and the management kubeconfig is mandatory in production.
    let authz = authz_from_env()?;
    let kubeconfig_path = mgmt_kubeconfig_path_from_env()?;
    let host = CapiControlPlaneHost::from_kubeconfig_path(&kubeconfig_path).await?;
    let state = build_state_capi(host, authz);
    let router = build_router(state);

    let listen_addr =
        std::env::var("OYATIE_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    serve(&listen_addr, router).await?;
    Ok(())
}

fn tracing_subscriber_init() {
    // Best-effort init; ignore the error if a global subscriber already exists.
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    );
}
