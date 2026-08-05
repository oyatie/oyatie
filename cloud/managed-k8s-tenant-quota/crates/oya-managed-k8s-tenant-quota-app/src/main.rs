//! Binary entry point for the managed-K8s tenant quota service.
//!
//! Reads `LISTEN_ADDR` from the environment (default `127.0.0.1:8080`) and
//! starts the quota admin REST service only when a quota-store boot policy is
//! explicit. The in-memory adapter is local/dev only and never the production
//! default fallback.

use oya_managed_k8s_tenant_quota_app::{build_state_from_env, serve};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                "oya_managed_k8s_tenant_quota=info"
                    .parse()
                    .expect("valid directive"),
            ),
        )
        .json()
        .init();

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let store = match build_state_from_env() {
        Ok(store) => store,
        Err(e) => {
            tracing::error!(error = %e, "managed-k8s-tenant-quota boot failed");
            std::process::exit(1);
        }
    };

    if let Err(e) = serve(&addr, store).await {
        tracing::error!(error = %e, "managed-k8s-tenant-quota boot failed");
        std::process::exit(1);
    }
}
