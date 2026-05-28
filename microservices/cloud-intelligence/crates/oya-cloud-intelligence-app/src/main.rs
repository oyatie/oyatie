//! cloud-intelligence binary entry-point (ADR-0384 Path B, Stage-7).
//!
//! Reads config from environment variables, calls [`build_app`] to wire all
//! production components (real OpenBao Transit + ClickHouse + Valkey sinks),
//! then serves the axum router. No panics on the start-up path: all errors are
//! surfaced as non-zero exit codes with a structured log message
//! (ADR-0083 Tier-3 panic-free).
//!
//! Environment variables (see [`AppConfig::from_env`]):
//! - `OYA_CLOUD_INTEL_LISTEN_ADDR`         — bind address (default: 0.0.0.0:8080)
//! - `OYA_CLOUD_INTEL_TENANT_ID`           — tenant ID (required)
//! - `OYA_CLOUD_INTEL_ANTHROPIC_URL`       — Anthropic base URL (default: production)
//! - `OYA_CLOUD_INTEL_INITIAL_SEATS`       — comma-separated seat_id:handle pairs
//! - `OYA_CLOUD_INTEL_OPENBAO_URL`         — OpenBao base URL (required)
//! - `OYA_CLOUD_INTEL_OPENBAO_TOKEN`       — OpenBao vault token (required)
//! - `OYA_CLOUD_INTEL_TRANSIT_KEY_NAME`    — Transit key name (default: cloud-intelligence-rt)
//! - `OYA_CLOUD_INTEL_CLICKHOUSE_URL`      — ClickHouse HTTP URL (default: analytics svc)
//! - `OYA_CLOUD_INTEL_CLICKHOUSE_USER`     — ClickHouse user (default: default)
//! - `OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD` — ClickHouse password (required)
//! - `OYA_CLOUD_INTEL_VALKEY_URL`          — Valkey URL (default: redis://valkey.infra.svc:6379)

use oya_cloud_intelligence_app::{AppConfig, build_app};
use oya_cloud_intelligence_rest::build_router;
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

    info!(addr = %listen_addr, "cloud-intelligence listening");

    // Serve. `axum::serve` is infallible until the OS closes the socket.
    if let Err(e) = axum::serve(listener, router).await {
        error!(error = %e, "axum serve error");
        std::process::exit(1);
    }
}
