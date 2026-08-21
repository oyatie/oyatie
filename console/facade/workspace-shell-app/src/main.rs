//! console-workspace-shell binary — Layer 6 composition root for the
//! workspace-shell cell. Boots a hyper server on `OYATIE_OPS_WORKSPACE_PORT`
//! (default 8080) bound to `127.0.0.1`.

use std::net::SocketAddr;
use std::sync::Arc;

use console_workspace_shell_app::{
    ConfiguredBearerAuthenticator, PrincipalAuthenticator, build_chain, build_dev_catalog,
    build_router,
};
use http_runtime_hyper_adapter::ServerConfig;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("OYATIE_OPS_WORKSPACE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();

    let catalog = build_dev_catalog();
    let router =
        Arc::new(build_router(catalog).map_err(|error| format!("router build failed: {error:?}"))?);

    // AUTH-005 increment-1: default-deny authn gate. The admin bearer is read
    // from a mounted-Secret env var (transitional until cloud-iam SVID/PDP).
    // Fail-closed: an unset/empty token verifies nothing, so every PROTECTED
    // route answers 401 (public health stays reachable).
    let admin_token = std::env::var("OYATIE_OPS_WORKSPACE_ADMIN_TOKEN").unwrap_or_default();
    if admin_token.trim().is_empty() {
        eprintln!(
            "console-workspace-shell: OYATIE_OPS_WORKSPACE_ADMIN_TOKEN unset/empty — \
             all protected routes will answer 401 (fail-closed)"
        );
    }
    let authenticator: Arc<dyn PrincipalAuthenticator> = Arc::new(
        ConfiguredBearerAuthenticator::new(admin_token, "ops-workspace-admin"),
    );
    let chain = Arc::new(build_chain(authenticator));

    eprintln!(
        "console-workspace-shell listening on http://{addr} ({} routes)",
        router.count()
    );

    // ADR-0092 Phase 8 (S3 + S4): default body cap + connection timeouts.
    let config = ServerConfig::default();
    http_runtime_hyper_adapter::serve(addr, router, chain, config).await?;
    Ok(())
}
