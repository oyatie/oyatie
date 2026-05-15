//! oya-ops-workspace-shell binary — Layer 6 composition root for the
//! workspace-shell cell. Boots a hyper server on `OYATIE_OPS_WORKSPACE_PORT`
//! (default 8080) bound to `127.0.0.1`.

use std::net::SocketAddr;
use std::sync::Arc;

use oya_http_runtime_hyper_adapter::ServerConfig;
use oya_ops_workspace_shell_app::{build_chain, build_dev_catalog, build_router};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("OYATIE_OPS_WORKSPACE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();

    let catalog = build_dev_catalog();
    let router = Arc::new(
        build_router(catalog).map_err(|error| format!("router build failed: {error:?}"))?,
    );
    let chain = Arc::new(build_chain());

    eprintln!(
        "oya-ops-workspace-shell listening on http://{addr} ({} routes)",
        router.count()
    );

    // ADR-0092 Phase 8 (S3 + S4): default body cap + connection timeouts.
    let config = ServerConfig::default();
    oya_http_runtime_hyper_adapter::serve(addr, router, chain, config).await?;
    Ok(())
}
