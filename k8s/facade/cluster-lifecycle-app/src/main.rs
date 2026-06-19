use k8s_cluster_lifecycle_app::{build_router, build_state_from_env, serve};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();
    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let state = match build_state_from_env() {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(%error, "managed-k8s cluster-lifecycle boot refused");
            std::process::exit(1);
        }
    };
    if let Err(error) = serve(&addr, build_router(state)).await {
        tracing::error!(%error, "managed-k8s cluster-lifecycle serve failed");
        std::process::exit(1);
    }
}
