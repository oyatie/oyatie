//! LLM-gateway binary — composition root.
//!
//! Boots the gateway end-to-end:
//! 1. Load the declarative (non-secret) config from `$GATEWAY_CONFIG` (a
//!    ConfigMap-mounted JSON file). NO secrets are read from this file.
//! 2. Build the in-memory key-material repository from the existing
//!    OpenBao-backed key store in `oya-llm-gateway-rest` when a `BAO_TOKEN`
//!    is present; else fail closed with an honest [`Unimplemented::OpenBaoResolution`]
//!    error.
//! 3. Construct the auth verifier from `ADMIN_TOKEN` + `INGRESS_PROXY_KEYS`.
//! 4. Wire the gateway state (per-group kernel pools + channel adapters +
//!    auth verifier + metrics surface) via [`build_gateway_state`].
//! 5. Mount the OpenAI-canonical router on top of the per-group reverse-
//!    proxy router via [`build_router`].
//! 6. Bind on `listen_addr` and serve via [`serve`].
//!
//! Auth tokens for the two realms come from the environment (injected from a
//! k8s Secret at deploy): `ADMIN_TOKEN` and `INGRESS_PROXY_KEYS` (comma
//! separated). The gateway reads NO plaintext provider key from file/env;
//! provider keys are sourced from OpenBao.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use oya_llm_gateway_app::{
    GatewayBootError, GatewayConfigRepository, HyperUpstreamAdapter, KeyMaterialRepository,
    RepositoryError, build_gateway_state, build_router, serve,
};
use oya_llm_gateway_kernel::ProviderChannel;
use oya_llm_gateway_rest::keystore::OpenBaoKeyStore;
use oya_llm_gateway_rest::{AuthVerifier, GatewayConfig, GatewayMetrics, KeyMaterial};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber_init();

    let cfg_repo = EnvFileGatewayConfigRepository::new()?;
    let config = cfg_repo.load()?;
    let auth = build_auth_verifier()?;
    let metrics = GatewayMetrics::new()?;
    let key_repo = OpenBaoBackedKeyRepository::from_env_and_config(&config)?;
    let transport = Arc::new(HyperUpstreamAdapter::new());

    let default_group = config
        .groups
        .first()
        .map(|g| g.name.clone())
        .ok_or("config declares no groups")?;
    let state = build_gateway_state(&cfg_repo, &key_repo, auth, metrics)?;
    let app = build_router(state, transport, default_group)?;
    serve(&config.listen_addr, app).await?;
    Ok(())
}

/// Reads the gateway config from the file path in `$GATEWAY_CONFIG`.
struct EnvFileGatewayConfigRepository {
    path: String,
}

impl EnvFileGatewayConfigRepository {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let path = std::env::var("GATEWAY_CONFIG")
            .map_err(|_| "GATEWAY_CONFIG env var (path to ConfigMap JSON) is required")?;
        Ok(Self { path })
    }
}

impl GatewayConfigRepository for EnvFileGatewayConfigRepository {
    fn load(&self) -> Result<GatewayConfig, RepositoryError> {
        let text = std::fs::read_to_string(&self.path).map_err(|e| {
            RepositoryError::new(format!("failed to read GATEWAY_CONFIG {}: {e}", self.path))
        })?;
        GatewayConfig::from_json(&text)
            .map_err(|e| RepositoryError::new(format!("config parse failed: {e}")))
    }
}

/// OpenBao-backed [`KeyMaterialRepository`] adapter. Wraps the existing
/// `OpenBaoKeyStore` in the rest crate; loads per-path key material via the
/// async key store and adapts it to the synchronous repository trait by
/// caching the materialized maps at boot time (the production composition
/// root reads keys once at startup and then refreshes on a tokio task).
struct OpenBaoBackedKeyRepository {
    by_path: BTreeMap<String, KeyMaterial>,
}

impl OpenBaoBackedKeyRepository {
    /// Build the repository by loading every group's keys from OpenBao.
    /// Returns [`GatewayBootError`] when `BAO_TOKEN` is absent (honest
    /// boundary — production keys are vault-only).
    fn from_env_and_config(config: &GatewayConfig) -> Result<Self, GatewayBootError> {
        let store = Arc::new(
            OpenBaoKeyStore::from_env(
                config.openbao.address.clone(),
                config.openbao.kv_mount.clone(),
            )
            .map_err(|e| GatewayBootError::Repository(RepositoryError::new(e.to_string())))?,
        );
        let mut by_path = BTreeMap::new();
        let rt = tokio::runtime::Handle::current();
        for group_cfg in &config.groups {
            let channel =
                group_cfg
                    .parsed_channel()
                    .ok_or_else(|| GatewayBootError::UnknownChannel {
                        group: group_cfg.name.clone(),
                        channel: group_cfg.channel.clone(),
                    })?;
            let store = Arc::clone(&store);
            let path = group_cfg.bao_key_path.clone();
            let material = rt
                .block_on(async move {
                    use oya_llm_gateway_rest::keystore::KeyStore;
                    store.load(&path, channel).await
                })
                .map_err(|e| GatewayBootError::KeyMaterial {
                    group: group_cfg.name.clone(),
                    error: RepositoryError::new(e.to_string()),
                })?;
            by_path.insert(group_cfg.bao_key_path.clone(), material);
        }
        Ok(Self { by_path })
    }
}

impl KeyMaterialRepository for OpenBaoBackedKeyRepository {
    fn load(&self, path: &str, channel: ProviderChannel) -> Result<KeyMaterial, RepositoryError> {
        match self.by_path.get(path) {
            Some(m) if m.channel() == channel => Ok(m.clone()),
            _ => Err(RepositoryError::new(format!(
                "openbao-backed repository has no entry for path {path:?}"
            ))),
        }
    }
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

fn tracing_subscriber_init() {
    // Best-effort init; ignore the error if a global subscriber already exists
    // (e.g. in tests). The dispatch events carry only redacted fields.
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    );
}
