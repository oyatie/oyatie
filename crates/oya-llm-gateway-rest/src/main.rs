//! Agent-dispatch gateway binary — composition root.
//!
//! Boots the reverse proxy:
//! 1. Load the declarative (non-secret) config from `$GATEWAY_CONFIG` (a
//!    ConfigMap-mounted JSON file). NO secrets are read from this file.
//! 2. Build the OpenBao key store from `BAO_TOKEN` (the only secret read from
//!    the environment) + the configured OpenBao address/mount.
//! 3. Load each group's pooled keys from OpenBao KV v2.
//! 4. Build the gateway state (per-group kernel pools + channel adapters +
//!    auth verifier + metrics surface).
//! 5. Spawn a periodic key-refresh task.
//! 6. Serve the axum app over hyper.
//!
//! Auth tokens for the two realms come from the environment too (injected from
//! a k8s Secret at deploy): `ADMIN_TOKEN` and `INGRESS_PROXY_KEYS` (comma
//! separated). The gateway reads NO plaintext provider key from file/env.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use oya_llm_gateway_kernel::PoolPolicy;
use oya_llm_gateway_rest::auth::AuthVerifier;
use oya_llm_gateway_rest::channel::ChannelAdapter;
use oya_llm_gateway_rest::config::{GatewayConfig, GroupConfig};
use oya_llm_gateway_rest::keystore::{KeyStore, KeyStoreError, OpenBaoKeyStore};
use oya_llm_gateway_rest::metrics::GatewayMetrics;
use oya_llm_gateway_rest::proxy::build_router;
use oya_llm_gateway_rest::state::{GatewayState, GroupRuntime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Structured JSON logging; the dispatch events are hash-only by design.
    tracing_subscriber_init();

    let config = load_config()?;
    config.validate()?;
    tracing::info!(
        target: "oya_llm_gateway::boot",
        groups = config.groups.len(),
        listen = %config.listen_addr,
        openbao = %config.openbao.address,
        "loaded declarative config (no secrets)"
    );

    let store = Arc::new(OpenBaoKeyStore::from_env(
        config.openbao.address.clone(),
        config.openbao.kv_mount.clone(),
    )?);

    let auth = build_auth_verifier()?;
    let metrics = GatewayMetrics::new()?;

    // Initial key load for every group.
    let mut groups = BTreeMap::new();
    for group_cfg in &config.groups {
        let channel = group_cfg
            .parsed_channel()
            .ok_or_else(|| format!("group {}: bad channel", group_cfg.name))?;
        let material = store.load(&group_cfg.bao_key_path, channel).await?;
        tracing::info!(
            target: "oya_llm_gateway::boot",
            group = %group_cfg.name,
            channel = channel.as_str(),
            keys = material.len(),
            "loaded pooled keys from OpenBao"
        );
        let adapter = ChannelAdapter::new(
            channel,
            group_cfg.upstream_base_url.clone(),
            group_cfg.anthropic_version.clone(),
        );
        let policy = pool_policy(group_cfg);
        let runtime = GroupRuntime::new(
            group_cfg.name.clone(),
            adapter,
            group_cfg.retry.clone(),
            policy,
            material,
        );
        groups.insert(group_cfg.name.clone(), runtime);
    }

    let state = Arc::new(GatewayState::new(groups, auth, metrics));

    // Periodic refresh task.
    if config.key_refresh_secs > 0 {
        spawn_refresh_task(
            Arc::clone(&state),
            Arc::clone(&store),
            config.clone(),
            Duration::from_secs(config.key_refresh_secs),
        );
    }

    let app = build_router(Arc::clone(&state));
    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!(
        target: "oya_llm_gateway::boot",
        addr = %config.listen_addr,
        "agent-dispatch gateway listening"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

fn pool_policy(group: &GroupConfig) -> PoolPolicy {
    PoolPolicy::new(
        group.blacklist_threshold,
        group.cooldown_base_millis,
        group.cooldown_jitter_millis,
    )
}

fn load_config() -> Result<GatewayConfig, Box<dyn std::error::Error>> {
    let path = std::env::var("GATEWAY_CONFIG")
        .map_err(|_| "GATEWAY_CONFIG env var (path to ConfigMap JSON) is required")?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read GATEWAY_CONFIG {path}: {e}"))?;
    GatewayConfig::from_json(&text).map_err(Into::into)
}

fn build_auth_verifier() -> Result<AuthVerifier, Box<dyn std::error::Error>> {
    let admin = std::env::var("ADMIN_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty())
        .ok_or("ADMIN_TOKEN env var is required and must be non-empty")?;
    let ingress_raw = std::env::var("INGRESS_PROXY_KEYS").unwrap_or_default();
    let ingress: Vec<String> = ingress_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if ingress.is_empty() {
        return Err("INGRESS_PROXY_KEYS env var must list at least one ingress key".into());
    }
    Ok(AuthVerifier::new(admin, ingress))
}

fn spawn_refresh_task(
    state: Arc<GatewayState>,
    store: Arc<OpenBaoKeyStore>,
    config: GatewayConfig,
    period: Duration,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(period);
        // Skip the immediate first tick (keys were just loaded at boot).
        ticker.tick().await;
        loop {
            ticker.tick().await;
            for group_cfg in &config.groups {
                let Some(channel) = group_cfg.parsed_channel() else {
                    continue;
                };
                match store.load(&group_cfg.bao_key_path, channel).await {
                    Ok(material) => {
                        if let Some(group) = state.group(&group_cfg.name) {
                            group.refresh_keys(pool_policy(group_cfg), &material);
                            tracing::info!(
                                target: "oya_llm_gateway::refresh",
                                group = %group_cfg.name,
                                keys = material.len(),
                                "refreshed pooled keys from OpenBao"
                            );
                        }
                    }
                    Err(KeyStoreError::Upstream(m)) => {
                        // Keep serving with the existing keys; just warn.
                        tracing::warn!(
                            target: "oya_llm_gateway::refresh",
                            group = %group_cfg.name,
                            error = %m,
                            "key refresh failed; retaining current keys"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            target: "oya_llm_gateway::refresh",
                            group = %group_cfg.name,
                            error = %e,
                            "key refresh error; retaining current keys"
                        );
                    }
                }
            }
            state.refresh_active_key_gauges();
        }
    });
}

fn tracing_subscriber_init() {
    // Best-effort init; ignore the error if a global subscriber already exists
    // (e.g. in tests). The dispatch events carry only redacted fields.
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    );
}
