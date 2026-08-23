//! intelligence-app binary entry-point (ADR-0384 Path B, Stage-7).
//!
//! Reads config from environment variables, calls [`build_app`] to wire all
//! production components (secret-provider adapter + ClickHouse + Valkey sinks),
//! then serves the axum router. No panics on the start-up path: all errors are
//! surfaced as non-zero exit codes with a structured log message
//! (ADR-0083 Tier-3 panic-free).
//!
//! Environment variables (see [`AppConfig::from_env`]):
//! - `OYATIE_CLOUD_INTEL_LISTEN_ADDR`         — bind address (default: 0.0.0.0:8080)
//! - `OYATIE_CLOUD_INTEL_TENANT_ID`           — tenant ID (required)
//! - `OYATIE_CLOUD_INTEL_ANTHROPIC_URL`       — Anthropic base URL (default: production)
//! - `OYATIE_CLOUD_INTEL_INITIAL_SEATS`       — comma-separated seat_id:handle pairs
//! - `OYATIE_CLOUD_INTEL_TENANT_PROVIDER_POOLS` — semicolon-separated tenant/provider handle pools
//! - `OYATIE_CLOUD_INTEL_SECRET_PROVIDER_URL`         — secret-provider adapter base URL (required)
//! - `OYATIE_CLOUD_INTEL_SECRET_PROVIDER_TOKEN`       — secret-provider adapter token (required)
//! - `OYATIE_CLOUD_INTEL_TRANSIT_KEY_NAME`    — Transit key name (default: intelligence-app-rt)
//! - `OYATIE_CLOUD_INTEL_CLICKHOUSE_URL`      — ClickHouse HTTP URL (default: analytics svc)
//! - `OYATIE_CLOUD_INTEL_CLICKHOUSE_USER`     — ClickHouse user (default: default)
//! - `OYATIE_CLOUD_INTEL_CLICKHOUSE_PASSWORD` — ClickHouse password (required)
//! - `OYATIE_CLOUD_INTEL_VALKEY_URL`          — Valkey URL (default: redis://valkey.infra.svc:6379)
//! - `OYATIE_CLOUD_INTEL_ADMIN_BEARER_TOKEN`  — optional admin-route bearer token (unset = fail closed)
//! - `OYATIE_CLOUD_INTEL_INGRESS_BEARER_TOKEN` — optional data-plane bearer token (unset = fail closed)
//! - `OYATIE_CLOUD_INTEL_ENVIRONMENT`         — environment name (production enforces compliance)
//! - `OYATIE_CLOUD_INTEL_ANTHROPIC_AUTH_MODE` — api_key | oauth_subscription
//! - `OYATIE_CLOUD_INTEL_ANTHROPIC_OAUTH_STATUS` — APPROVED | API_ONLY | BLOCKED | PENDING
//! - `OYATIE_CLOUD_INTEL_CODEX_AUTH_MODE`     — api_key | oauth_subscription
//! - `OYATIE_CLOUD_INTEL_CODEX_OAUTH_STATUS`  — APPROVED | API_ONLY | BLOCKED | PENDING

use intelligence_app::{AppConfig, build_app};
use intelligence_rest::build_router;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber (env-filter from RUST_LOG; default = info).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Read config from environment.
    let config = match AppConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "failed to read AppConfig from environment");
            std::process::exit(1);
        }
    };

    let listen_addr = config.listen_addr.clone();

    // Build the composed AppState.
    let state = match build_app(config) {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "failed to build AppState");
            std::process::exit(1);
        }
    };

    // Build axum router.
    let router = build_router(state);

    // Bind listener.
    let listener = match tokio::net::TcpListener::bind(&listen_addr).await {
        Ok(l) => l,
        Err(e) => {
            error!(addr = %listen_addr, error = %e, "failed to bind TCP listener");
            std::process::exit(1);
        }
    };

    info!(addr = %listen_addr, "intelligence-app listening");

    // Serve. `axum::serve` is infallible until the OS closes the socket.
    if let Err(e) = axum::serve(listener, router).await {
        error!(error = %e, "axum serve error");
        std::process::exit(1);
    }
}
