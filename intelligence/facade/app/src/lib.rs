//! OAuth subscription-pool app crate (ADR-0384 Path B).
//!
//! This is the composition layer for the cloud-intelligence
//! µservice. It wires:
//!
//! - [`intelligence_kernel`] — pure-Rust pool + trait seams.
//! - [`intelligence_authz_cedar_adapter`] — transient adapter for
//!   the owned policy-engine [`AuthzGate`] seam.
//! - [`intelligence_rest`] — axum REST adapter + AnthropicAdapter.
//! - [`intelligence_openbao_adapter::OpenBaoTransitStore`] —
//!   transient backing adapter for the owned secret-provider port (D8).
//! - [`intelligence_eventsink_clickhouse_adapter::ClickHouseEventSink`] +
//!   [`intelligence_eventsink_valkey_adapter::ValkeyEventSink`] — real
//!   D6 event sinks fanned out via [`EventSinkFanout`].
//!
//! Entry-point for the binary is `src/main.rs`; `build_app` is the
//! production composition function. `build_app_for_tests` uses in-process
//! mocks and is available unconditionally for unit/integration tests.
//!
//! ADR-0083 Tier-3: no unwrap/expect/panic on the request path. Errors from
//! build_app propagate as `AppBuildError`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use intelligence_authz_cedar_adapter::CedarAuthzGate;
use intelligence_eventsink_clickhouse_adapter::ClickHouseEventSink;
use intelligence_eventsink_valkey_adapter::ValkeyEventSink;
use intelligence_kernel::{
    CredentialMode as KernelCredentialMode, EventSink, LlmGatewayEvent, OAuthSubscription,
    Provider, SeatId, SelectionStrategy, SubscriptionId, SubscriptionPool, SubscriptionState,
    TenantId, is_secret_handle_reference,
};
use intelligence_openbao_adapter::OpenBaoTransitStore;
use intelligence_rest::{
    AppState, BearerBinding, ConfiguredBearerIngressAuthenticator,
    ConfiguredBearerMapIngressAuthenticator, EventSinkFanout, IngressPrincipalAuthenticator,
    PoolRegistry, RestAdapterError, SecretProviderFuture, SecretProviderStore,
};
use oya_shared_olap_clickhouse_adapter::ClickHouseConfig;
use tracing::info;

// ---------------------------------------------------------------------------
// AppConfig — read from environment / caller
// ---------------------------------------------------------------------------

/// Credential mode selected for a provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialMode {
    ApiKey,
    OAuthSubscription,
}

impl CredentialMode {
    fn from_env_value(raw: &str, var_name: &str) -> Result<Self, AppBuildError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "api_key" | "apikey" | "api-key" => Ok(Self::ApiKey),
            "oauth_subscription" | "oauth-subscription" | "oauth" => Ok(Self::OAuthSubscription),
            other => Err(AppBuildError::Config(format!(
                "{var_name} has unsupported credential mode: {other}"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::OAuthSubscription => "oauth_subscription",
        }
    }

    fn to_kernel(self) -> KernelCredentialMode {
        match self {
            Self::ApiKey => KernelCredentialMode::ApiKey,
            Self::OAuthSubscription => KernelCredentialMode::OAuthSubscription,
        }
    }
}

/// Compliance gate status for provider OAuth subscription proxying.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderComplianceStatus {
    Approved,
    ApiOnly,
    Blocked,
    Pending,
}

impl ProviderComplianceStatus {
    fn from_env_value(raw: &str, var_name: &str) -> Result<Self, AppBuildError> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "APPROVED" => Ok(Self::Approved),
            "API_ONLY" | "API-ONLY" => Ok(Self::ApiOnly),
            "BLOCKED" => Ok(Self::Blocked),
            "PENDING" => Ok(Self::Pending),
            other => Err(AppBuildError::Config(format!(
                "{var_name} has unsupported provider compliance status: {other}"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "APPROVED",
            Self::ApiOnly => "API_ONLY",
            Self::Blocked => "BLOCKED",
            Self::Pending => "PENDING",
        }
    }
}

/// Per-provider compliance gate config. Production OAuth subscription mode is
/// fail-closed unless the provider/mode has explicit `APPROVED` evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderComplianceConfig {
    pub anthropic_auth_mode: CredentialMode, // data_class: INTERNAL_ONLY
    pub anthropic_oauth_status: ProviderComplianceStatus, // data_class: INTERNAL_ONLY
    pub codex_auth_mode: CredentialMode,     // data_class: INTERNAL_ONLY
    pub codex_oauth_status: ProviderComplianceStatus, // data_class: INTERNAL_ONLY
}

impl ProviderComplianceConfig {
    fn from_env() -> Result<Self, AppBuildError> {
        let defaults = Self::default();
        Ok(Self {
            anthropic_auth_mode: read_credential_mode_env(
                "OYA_CLOUD_INTEL_ANTHROPIC_AUTH_MODE",
                defaults.anthropic_auth_mode,
            )?,
            anthropic_oauth_status: read_provider_status_env(
                "OYA_CLOUD_INTEL_ANTHROPIC_OAUTH_STATUS",
                defaults.anthropic_oauth_status,
            )?,
            codex_auth_mode: read_credential_mode_env(
                "OYA_CLOUD_INTEL_CODEX_AUTH_MODE",
                defaults.codex_auth_mode,
            )?,
            codex_oauth_status: read_provider_status_env(
                "OYA_CLOUD_INTEL_CODEX_OAUTH_STATUS",
                defaults.codex_oauth_status,
            )?,
        })
    }
}

impl Default for ProviderComplianceConfig {
    fn default() -> Self {
        Self {
            // Preserve the current Anthropic OAuth adapter path, but require
            // explicit APPROVED evidence before production boot accepts it.
            anthropic_auth_mode: CredentialMode::OAuthSubscription,
            anthropic_oauth_status: ProviderComplianceStatus::Pending,
            // Codex is not wired into the production data plane yet, so default
            // to API-only until the OAuth compliance gate is explicitly approved.
            codex_auth_mode: CredentialMode::ApiKey,
            codex_oauth_status: ProviderComplianceStatus::ApiOnly,
        }
    }
}

/// Static boot-time subscription seat loaded from config.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticSeatConfig {
    pub seat_id: String,         // data_class: INTERNAL_ONLY
    pub subscription_id: String, // data_class: INTERNAL_ONLY
    pub secret_handle: String,   // data_class: INTERNAL_ONLY (opaque secret-provider handle)
}

/// Static boot-time pool config for one `(tenant, provider)` pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantProviderPoolConfig {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub provider: Provider,              // data_class: INTERNAL_ONLY
    pub credential_mode: CredentialMode, // data_class: INTERNAL_ONLY
    pub strategy: SelectionStrategy,     // data_class: INTERNAL_ONLY
    pub seats: Vec<StaticSeatConfig>,    // data_class: INTERNAL_ONLY
}

fn read_credential_mode_env(
    var_name: &str,
    default: CredentialMode,
) -> Result<CredentialMode, AppBuildError> {
    match std::env::var(var_name) {
        Ok(raw) => CredentialMode::from_env_value(&raw, var_name),
        Err(_) => Ok(default),
    }
}

fn read_provider_status_env(
    var_name: &str,
    default: ProviderComplianceStatus,
) -> Result<ProviderComplianceStatus, AppBuildError> {
    match std::env::var(var_name) {
        Ok(raw) => ProviderComplianceStatus::from_env_value(&raw, var_name),
        Err(_) => Ok(default),
    }
}

/// Application configuration. Populated from environment variables by `main.rs`;
/// unit tests can construct it directly.
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// TCP address to bind the axum listener (e.g. `0.0.0.0:8080`).
    pub listen_addr: String, // data_class: INTERNAL_ONLY
    /// Tenant ID this gateway instance serves. Must be non-empty.
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    /// Anthropic API base URL (override for testing; default is production URL).
    pub anthropic_base_url: String, // data_class: INTERNAL_ONLY
    /// Optional comma-separated initial seat handles for bootstrapping the pool.
    /// Format: `seat_id:handle,...`
    pub initial_seats: Vec<(String, String)>, // data_class: INTERNAL_ONLY
    /// Optional semicolon-separated static tenant/provider pools.
    /// Format per seat:
    /// `tenant|provider|credential_mode|strategy|seat_id|secret_handle;...`
    pub tenant_provider_pools: Vec<TenantProviderPoolConfig>, // data_class: INTERNAL_ONLY
    /// Secret-provider adapter base URL for handle resolution (D8).
    /// e.g. `https://cloud-secrets-adapter.infra.svc:8200`
    pub secret_provider_url: String, // data_class: INTERNAL_ONLY
    /// Secret-provider adapter token. Sourced from `OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN`.
    pub secret_provider_token: String, // data_class: SECRET
    /// Transit key name used for envelope-encryption of refresh tokens.
    pub transit_key_name: String, // data_class: INTERNAL_ONLY
    /// ClickHouse HTTP URL for OLAP event sink (D6).
    /// e.g. `http://clickhouse.analytics.svc:8123`
    pub clickhouse_url: String, // data_class: INTERNAL_ONLY
    /// ClickHouse user.
    pub clickhouse_user: String, // data_class: INTERNAL_ONLY
    /// ClickHouse password. Sourced from `OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD`.
    pub clickhouse_password: String, // data_class: SECRET
    /// Valkey/Redis URL for stream event sink (D6).
    /// e.g. `redis://valkey.infra.svc:6379` or `rediss://...` for TLS.
    pub valkey_url: String, // data_class: INTERNAL_ONLY
    /// Optional admin bearer token for tenant-scoped pool-management routes.
    /// If unset, admin routes fail closed with 401.
    pub admin_bearer_token: Option<String>, // data_class: SECRET
    /// Optional data-plane ingress bearer token for `/v1/*` routes. If unset,
    /// the REST adapter fails all data-plane routes closed with 401.
    pub ingress_bearer_token: Option<String>, // data_class: SECRET
    /// Optional multi-tenant ingress bearer map (AUTH-005 increment-3): each
    /// `(tenant, token)` binds one bearer credential to one VERIFIED principal
    /// tenant. When non-empty, the gateway authenticates with a multi-tenant
    /// authenticator instead of the single `ingress_bearer_token` binding. Empty
    /// => single-tenant behavior is unchanged.
    pub ingress_bearer_map: Vec<(TenantId, String)>, // data_class: SECRET
    /// Runtime environment. Production enforces provider-compliance gates.
    pub environment: String, // data_class: INTERNAL_ONLY
    /// Provider/mode compliance statuses used to fail-close OAuth subscription
    /// proxying until the source-review gate records explicit approval.
    pub provider_compliance: ProviderComplianceConfig, // data_class: INTERNAL_ONLY
}

impl AppConfig {
    /// Read config from environment variables.
    ///
    /// | Env var                              | Default                          |
    /// |--------------------------------------|----------------------------------|
    /// | `OYA_CLOUD_INTEL_LISTEN_ADDR`        | `0.0.0.0:8080`                   |
    /// | `OYA_CLOUD_INTEL_TENANT_ID`          | *(required)*                     |
    /// | `OYA_CLOUD_INTEL_ANTHROPIC_URL`      | `https://api.anthropic.com`      |
    /// | `OYA_CLOUD_INTEL_INITIAL_SEATS`      | *(empty)*                        |
    /// | `OYA_CLOUD_INTEL_TENANT_PROVIDER_POOLS`| *(empty)*                      |
    /// | `OYA_CLOUD_INTEL_SECRET_PROVIDER_URL`        | *(required)*                     |
    /// | `OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN`      | *(required)*                     |
    /// | `OYA_CLOUD_INTEL_TRANSIT_KEY_NAME`   | `cloud-intelligence-rt`                 |
    /// | `OYA_CLOUD_INTEL_CLICKHOUSE_URL`     | `http://clickhouse.analytics.svc:8123` |
    /// | `OYA_CLOUD_INTEL_CLICKHOUSE_USER`    | `default`                        |
    /// | `OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD`| *(required)*                     |
    /// | `OYA_CLOUD_INTEL_VALKEY_URL`         | `redis://valkey.infra.svc:6379`  |
    /// | `OYA_CLOUD_INTEL_ADMIN_BEARER_TOKEN` | *(unset: admin routes 401)*      |
    /// | `OYA_CLOUD_INTEL_INGRESS_BEARER_TOKEN` | *(unset: data-plane routes 401)* |
    /// | `OYA_CLOUD_INTEL_INGRESS_BEARER_MAP` | *(empty; `tenant\|token;...` multi-tenant)* |
    /// | `OYA_CLOUD_INTEL_ENVIRONMENT`        | `development`                    |
    /// | `OYA_CLOUD_INTEL_ANTHROPIC_AUTH_MODE`| `oauth_subscription`             |
    /// | `OYA_CLOUD_INTEL_ANTHROPIC_OAUTH_STATUS`| `PENDING`                     |
    /// | `OYA_CLOUD_INTEL_CODEX_AUTH_MODE`    | `api_key`                        |
    /// | `OYA_CLOUD_INTEL_CODEX_OAUTH_STATUS` | `API_ONLY`                       |
    pub fn from_env() -> Result<Self, AppBuildError> {
        let listen_addr = std::env::var("OYA_CLOUD_INTEL_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let environment = std::env::var("OYA_CLOUD_INTEL_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());
        let provider_compliance = ProviderComplianceConfig::from_env()?;
        let tenant_id = std::env::var("OYA_CLOUD_INTEL_TENANT_ID").map_err(|_| {
            AppBuildError::Config("OYA_CLOUD_INTEL_TENANT_ID is required".to_string())
        })?;
        let anthropic_base_url = std::env::var("OYA_CLOUD_INTEL_ANTHROPIC_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
        let initial_seats = parse_initial_seats(
            &std::env::var("OYA_CLOUD_INTEL_INITIAL_SEATS").unwrap_or_default(),
        )?;
        let tenant_provider_pools = parse_tenant_provider_pools(
            &std::env::var("OYA_CLOUD_INTEL_TENANT_PROVIDER_POOLS").unwrap_or_default(),
        )?;
        let secret_provider_url =
            std::env::var("OYA_CLOUD_INTEL_SECRET_PROVIDER_URL").map_err(|_| {
                AppBuildError::Config("OYA_CLOUD_INTEL_SECRET_PROVIDER_URL is required".to_string())
            })?;
        let secret_provider_token = std::env::var("OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN")
            .map_err(|_| {
                AppBuildError::Config(
                    "OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN is required".to_string(),
                )
            })?;
        let transit_key_name = std::env::var("OYA_CLOUD_INTEL_TRANSIT_KEY_NAME")
            .unwrap_or_else(|_| "cloud-intelligence-rt".to_string());
        let clickhouse_url = std::env::var("OYA_CLOUD_INTEL_CLICKHOUSE_URL")
            .unwrap_or_else(|_| "http://clickhouse.analytics.svc:8123".to_string());
        let clickhouse_user = std::env::var("OYA_CLOUD_INTEL_CLICKHOUSE_USER")
            .unwrap_or_else(|_| "default".to_string());
        let clickhouse_password =
            std::env::var("OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD").map_err(|_| {
                AppBuildError::Config("OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD is required".to_string())
            })?;
        let valkey_url = std::env::var("OYA_CLOUD_INTEL_VALKEY_URL")
            .unwrap_or_else(|_| "redis://valkey.infra.svc:6379".to_string());
        let admin_bearer_token = std::env::var("OYA_CLOUD_INTEL_ADMIN_BEARER_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty());
        let ingress_bearer_token = std::env::var("OYA_CLOUD_INTEL_INGRESS_BEARER_TOKEN")
            .ok()
            .filter(|token| !token.trim().is_empty());
        let ingress_bearer_map = parse_ingress_bearer_map(
            &std::env::var("OYA_CLOUD_INTEL_INGRESS_BEARER_MAP").unwrap_or_default(),
        )?;
        let config = Self {
            listen_addr,
            tenant_id,
            anthropic_base_url,
            initial_seats,
            tenant_provider_pools,
            secret_provider_url,
            secret_provider_token,
            transit_key_name,
            clickhouse_url,
            clickhouse_user,
            clickhouse_password,
            valkey_url,
            admin_bearer_token,
            ingress_bearer_token,
            ingress_bearer_map,
            environment,
            provider_compliance,
        };
        config.validate_provider_compliance()?;
        Ok(config)
    }

    /// Enforce the provider-compliance source-review gate.
    ///
    /// Production OAuth subscription proxying is fail-closed until each
    /// provider/mode records `APPROVED`. Provider API-key mode is allowed here
    /// because it uses documented direct provider APIs instead of the gated
    /// OAuth subscription path.
    pub fn validate_provider_compliance(&self) -> Result<(), AppBuildError> {
        if !self.environment.eq_ignore_ascii_case("production") {
            return Ok(());
        }
        require_oauth_approval(
            "anthropic",
            self.provider_compliance.anthropic_auth_mode,
            self.provider_compliance.anthropic_oauth_status,
        )?;
        require_oauth_approval(
            "codex",
            self.provider_compliance.codex_auth_mode,
            self.provider_compliance.codex_oauth_status,
        )?;

        // Fail-closed on the credential modes actually loaded into the
        // effective tenant/provider pools, not just the env-declared modes.
        // A config that declares ANTHROPIC_AUTH_MODE=api_key while loading an
        // Anthropic oauth_subscription pool must still require APPROVED.
        for pool in effective_tenant_provider_pools(self)? {
            if pool.credential_mode != CredentialMode::OAuthSubscription {
                continue;
            }
            let (provider_label, status) = match pool.provider {
                Provider::Anthropic => {
                    ("anthropic", self.provider_compliance.anthropic_oauth_status)
                }
                Provider::Codex => ("codex", self.provider_compliance.codex_oauth_status),
                Provider::Gemini => ("gemini", ProviderComplianceStatus::Blocked),
            };
            require_oauth_approval(provider_label, pool.credential_mode, status)?;
        }
        Ok(())
    }
}

fn require_oauth_approval(
    provider: &str,
    auth_mode: CredentialMode,
    status: ProviderComplianceStatus,
) -> Result<(), AppBuildError> {
    if auth_mode == CredentialMode::OAuthSubscription
        && status != ProviderComplianceStatus::Approved
    {
        return Err(AppBuildError::Config(format!(
            "provider compliance not approved: provider={provider} auth_mode={} status={}",
            auth_mode.as_str(),
            status.as_str()
        )));
    }
    Ok(())
}

fn parse_initial_seats(raw: &str) -> Result<Vec<(String, String)>, AppBuildError> {
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    raw.split(',')
        .map(|entry| {
            let mut parts = entry.splitn(2, ':');
            let seat_id = parts.next().unwrap_or_default().trim().to_string();
            let handle = parts.next().unwrap_or_default().trim().to_string();
            if seat_id.is_empty() || handle.is_empty() {
                return Err(AppBuildError::Config(
                    "initial seat entries must be seat_id:secret_handle".to_string(),
                ));
            }
            validate_secret_handle(&handle)?;
            Ok((seat_id, handle))
        })
        .collect()
}

/// Parse `OYA_CLOUD_INTEL_INGRESS_BEARER_MAP` (AUTH-005 increment-3): ';'-separated
/// entries, each `tenant|token`. Mirrors [`parse_tenant_provider_pools`]. An empty
/// input yields an empty map (single-tenant behavior unchanged). Each tenant is
/// validated into a [`TenantId`]; an empty token is rejected (a token that can
/// never authenticate is a config error, not a fail-open).
fn parse_ingress_bearer_map(raw: &str) -> Result<Vec<(TenantId, String)>, AppBuildError> {
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut bindings: Vec<(TenantId, String)> = Vec::new();
    for entry in raw
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        let parts = entry.split('|').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(AppBuildError::Config(format!(
                "OYA_CLOUD_INTEL_INGRESS_BEARER_MAP entry must have 2 fields (tenant|token): {entry}"
            )));
        }
        let tenant = TenantId::new(parts[0]).map_err(|_| {
            AppBuildError::Config(format!("ingress bearer map tenant is invalid: {entry}"))
        })?;
        let token = parts[1].to_string();
        if token.is_empty() {
            return Err(AppBuildError::Config(
                "ingress bearer map token is required".to_string(),
            ));
        }
        // Fail closed at boot on a duplicate token: a token shared across entries would silently
        // authenticate as whichever binding the verify loop matched last -> wrong-tenant attribution
        // and a cross-tenant lease. (Duplicate tenants are allowed: multiple tokens per tenant is
        // legitimate credential rotation.) ponytail: O(n^2) over a tiny tenant-count binding set.
        if bindings.iter().any(|(_, existing)| existing == &token) {
            return Err(AppBuildError::Config(
                "OYA_CLOUD_INTEL_INGRESS_BEARER_MAP has a duplicate token shared across entries \
                 (would silently bind to the wrong tenant)"
                    .to_string(),
            ));
        }
        bindings.push((tenant, token));
    }
    Ok(bindings)
}

fn parse_tenant_provider_pools(raw: &str) -> Result<Vec<TenantProviderPoolConfig>, AppBuildError> {
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut pools: Vec<TenantProviderPoolConfig> = Vec::new();
    for entry in raw
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        let parts = entry.split('|').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 6 {
            return Err(AppBuildError::Config(format!(
                "OYA_CLOUD_INTEL_TENANT_PROVIDER_POOLS entry must have 6 fields: {entry}"
            )));
        }
        let tenant_id = parts[0].to_string();
        if tenant_id.is_empty() {
            return Err(AppBuildError::Config(
                "tenant provider pool tenant_id is required".to_string(),
            ));
        }
        let provider = parse_provider(parts[1])?;
        let credential_mode =
            CredentialMode::from_env_value(parts[2], "OYA_CLOUD_INTEL_TENANT_PROVIDER_POOLS")?;
        let strategy = parse_selection_strategy(parts[3])?;
        let seat_id = parts[4].to_string();
        if seat_id.is_empty() {
            return Err(AppBuildError::Config(
                "tenant provider pool seat_id is required".to_string(),
            ));
        }
        let secret_handle = parts[5].to_string();
        validate_secret_handle(&secret_handle)?;
        let static_seat = StaticSeatConfig {
            subscription_id: format!("{seat_id}-sub"),
            seat_id,
            secret_handle,
        };

        if let Some(pool) = pools.iter_mut().find(|pool| {
            pool.tenant_id == tenant_id
                && pool.provider == provider
                && pool.credential_mode == credential_mode
                && pool.strategy == strategy
        }) {
            pool.seats.push(static_seat);
        } else {
            pools.push(TenantProviderPoolConfig {
                tenant_id,
                provider,
                credential_mode,
                strategy,
                seats: vec![static_seat],
            });
        }
    }

    Ok(pools)
}

fn parse_provider(raw: &str) -> Result<Provider, AppBuildError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "anthropic" | "claude" => Ok(Provider::Anthropic),
        "codex" | "openai" => Ok(Provider::Codex),
        "gemini" | "google" => Ok(Provider::Gemini),
        other => Err(AppBuildError::Config(format!(
            "unsupported provider in tenant pool: {other}"
        ))),
    }
}

fn parse_selection_strategy(raw: &str) -> Result<SelectionStrategy, AppBuildError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "round_robin" | "round-robin" => Ok(SelectionStrategy::RoundRobin),
        "fill_first" | "fill-first" => Ok(SelectionStrategy::FillFirst),
        "time_normalized_quota_percent"
        | "time-normalized-quota-percent"
        | "time_normalized"
        | "time-normalized" => Ok(SelectionStrategy::TimeNormalizedQuotaPercent),
        other => Err(AppBuildError::Config(format!(
            "unsupported tenant pool selection strategy: {other}"
        ))),
    }
}

fn validate_secret_handle(handle: &str) -> Result<(), AppBuildError> {
    if !is_secret_handle_reference(handle) {
        return Err(AppBuildError::Config(
            "tenant provider pool secret handle must be an opaque reference".to_string(),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// AppBuildError
// ---------------------------------------------------------------------------

/// Errors raised during `build_app`. All variants are fatal at start-up time
/// and are surfaced as non-zero exit codes in `main.rs`.
#[derive(Debug)]
pub enum AppBuildError {
    /// A required configuration value is missing or invalid.
    Config(String),
    /// Cedar policy failed to parse. This is a compile-time invariant in
    /// production (policy is bundled); reported here for explicit error
    /// surfacing during tests with custom policy text.
    CedarPolicy(String),
    /// `reqwest::Client` construction failed (platform TLS error).
    HttpClient(reqwest::Error),
    /// Kernel rejected a seat configuration at startup.
    PoolSetup(String),
}

impl std::fmt::Display for AppBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppBuildError::Config(msg) => write!(f, "config error: {msg}"),
            AppBuildError::CedarPolicy(msg) => write!(f, "cedar policy error: {msg}"),
            AppBuildError::HttpClient(e) => write!(f, "http client error: {e}"),
            AppBuildError::PoolSetup(msg) => write!(f, "pool setup error: {msg}"),
        }
    }
}

impl std::error::Error for AppBuildError {}

impl From<reqwest::Error> for AppBuildError {
    fn from(e: reqwest::Error) -> Self {
        AppBuildError::HttpClient(e)
    }
}

// ---------------------------------------------------------------------------
// EventSinkFanoutAdapter — thin EventSink wrapper around EventSinkFanout
// ---------------------------------------------------------------------------

/// Wraps [`EventSinkFanout`] so it can be stored as `Arc<dyn EventSink>`.
/// `EventSinkFanout` exposes `broadcast()` rather than `emit()` directly;
/// this adapter bridges the two.
struct EventSinkFanoutAdapter(EventSinkFanout);

impl EventSink for EventSinkFanoutAdapter {
    fn emit(&self, event: LlmGatewayEvent) {
        self.0.broadcast(event);
    }
}

// ---------------------------------------------------------------------------
// In-process stubs (kept for build_app_for_tests)
// ---------------------------------------------------------------------------

/// In-process stub secret store. Holds a plaintext map keyed by handle.
/// Stage-7: replace with a production secret-provider adapter.
///
/// NOTE: plaintext tokens in memory are acceptable for the local-foundation
/// phase only. See ADR-0384 D8 and the Stage-7 deferral note.
pub struct InProcessSecretStore {
    map: Mutex<HashMap<String, String>>, // data_class: INTERNAL_ONLY
}

impl InProcessSecretStore {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    /// Pre-load a handle → plaintext pair (used at startup when
    /// `OYA_CLOUD_INTEL_INITIAL_SEATS` is set).
    pub fn preload(&self, handle: &str, token: &str) {
        if let Ok(mut m) = self.map.lock() {
            m.insert(handle.to_string(), token.to_string());
        }
    }
}

impl Default for InProcessSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretProviderStore for InProcessSecretStore {
    fn fetch_refresh_token<'a>(&'a self, handle: &'a str) -> SecretProviderFuture<'a, String> {
        Box::pin(async move {
            self.map
                .lock()
                .map_err(|_| RestAdapterError::SecretStoreUnavailable("lock poisoned".to_string()))?
                .get(handle)
                .cloned()
                .ok_or(RestAdapterError::SecretNotFound)
        })
    }

    fn store_refresh_token<'a>(
        &'a self,
        handle: &'a str,
        plaintext: &'a str,
    ) -> SecretProviderFuture<'a, ()> {
        Box::pin(async move {
            if plaintext.is_empty() {
                return Err(RestAdapterError::InvalidSecret);
            }
            self.map
                .lock()
                .map_err(|_| RestAdapterError::SecretStoreUnavailable("lock poisoned".to_string()))?
                .insert(handle.to_string(), plaintext.to_string());
            Ok(())
        })
    }
}

/// In-process event sink — logs events via `tracing`. Stage-7: replace with
/// ClickHouse OLAP adapter + Valkey Stream adapter.
pub struct InProcessEventSink;

impl EventSink for InProcessEventSink {
    fn emit(&self, event: LlmGatewayEvent) {
        info!(
            request_id = %event.request_id,
            tenant_id  = %event.tenant_id.as_str(),
            seat_id    = %event.seat_id.as_str(),
            provider   = %event.provider,
            status     = ?event.status,
            ms_latency = event.ms_latency,
            "cloud-intelligence event"
        );
    }
}

// ---------------------------------------------------------------------------
// build_app — testable composition root
// ---------------------------------------------------------------------------

/// Wire up all components with **real production adapters** and return the
/// shared [`AppState`].
///
/// Uses:
/// - [`OpenBaoTransitStore`] as the current transient D8 backing adapter.
/// - [`ClickHouseEventSink`] + [`ValkeyEventSink`] fanned out via
///   [`EventSinkFanout`] for D6 event emission.
///
/// Reads all adapter config from `AppConfig` (populated from env vars).
/// Returns [`AppBuildError`] on any fatal configuration or connection failure.
pub fn build_app(config: AppConfig) -> Result<Arc<AppState>, AppBuildError> {
    config.validate_provider_compliance()?;

    // Cedar gate (loaded from bundled policy; fail-closed on parse error).
    let gate = CedarAuthzGate::with_default_policy()
        .map_err(|e| AppBuildError::CedarPolicy(e.to_string()))?;

    // Tenant ID validation.
    let tenant_id = TenantId::new(&config.tenant_id)
        .map_err(|_| AppBuildError::Config(format!("invalid tenant_id: {:?}", config.tenant_id)))?;

    // Build shared reqwest::Client used by both AppState and the secret-provider adapter.
    let http_client = Arc::new(
        reqwest::Client::builder()
            .build()
            .map_err(AppBuildError::HttpClient)?,
    );

    // Current transient secret-provider backing adapter (D8).
    let secret_store: Arc<dyn SecretProviderStore> = Arc::new(OpenBaoTransitStore::new(
        Arc::clone(&http_client),
        &config.secret_provider_url,
        &config.transit_key_name,
        &config.secret_provider_token,
    ));

    // Real D6 event sinks — ClickHouse + Valkey fanned out.
    let ch_sink = ClickHouseEventSink::new(ClickHouseConfig {
        url: config.clickhouse_url.clone(),
        user: config.clickhouse_user.clone(),
        password: config.clickhouse_password.clone(),
    });

    let valkey_sink = ValkeyEventSink::connect(&config.valkey_url)
        .map_err(|e| AppBuildError::Config(format!("valkey connect failed: {e}")))?;

    let mut fanout = EventSinkFanout::new();
    fanout.add_sink(Box::new(ch_sink));
    fanout.add_sink(Box::new(valkey_sink));
    let sink: Arc<dyn EventSink + Send + Sync> = Arc::new(EventSinkFanoutAdapter(fanout));

    // Subscription pools: one static boot-time pool per tenant/provider pair.
    let pool_configs = effective_tenant_provider_pools(&config)?;
    let (pool_registry, pool_arc) = build_pool_registry(tenant_id.clone(), &pool_configs)?;

    // AUTH-005 / ADR-0573 data-plane ingress authn/authz: the verified principal
    // tenant binding defaults to this service's own tenant.
    // OYA_CLOUD_INTEL_INGRESS_PRINCIPAL_TENANT overrides it so a cross-tenant
    // authz test is expressible. An empty/unset bearer => every data-plane
    // request 401 (fail-closed; the authenticator mints nothing).
    let ingress_principal_tenant = std::env::var("OYA_CLOUD_INTEL_INGRESS_PRINCIPAL_TENANT")
        .ok()
        .filter(|tenant| !tenant.trim().is_empty())
        .unwrap_or_else(|| config.tenant_id.clone());
    let ingress_principal_tenant = TenantId::new(&ingress_principal_tenant).map_err(|_| {
        AppBuildError::Config(format!(
            "invalid ingress principal tenant: {ingress_principal_tenant:?}"
        ))
    })?;
    let ingress_authenticator: Arc<dyn IngressPrincipalAuthenticator> =
        if config.ingress_bearer_map.is_empty() {
            // Single-tenant (unchanged): the configured bearer bound to this
            // service's own (or override) tenant. Empty token => every request 401.
            Arc::new(ConfiguredBearerIngressAuthenticator::new(
                config.ingress_bearer_token.clone().unwrap_or_default(),
                ingress_principal_tenant,
            ))
        } else {
            // Multi-tenant (AUTH-005 increment-3): each (tenant, token) binds a
            // bearer to a VERIFIED principal tenant. x-agent-id stays caller-supplied
            // (intra-tenant label), so the verified agent is None here.
            Arc::new(ConfiguredBearerMapIngressAuthenticator::new(
                config
                    .ingress_bearer_map
                    .iter()
                    .map(|(tenant, token)| BearerBinding {
                        token: token.clone(),
                        tenant: tenant.clone(),
                        agent: None,
                    })
                    .collect(),
            ))
        };

    // Build AppState (uses the shared reqwest::Client for upstream proxy calls).
    let state = AppState::new_with_pool_registry(
        pool_arc,
        pool_registry,
        Arc::new(gate),
        sink,
        secret_store,
        config.anthropic_base_url,
        tenant_id,
        config.ingress_bearer_token,
        config.admin_bearer_token,
        config.environment.clone(),
        oauth_approved_providers(&config.provider_compliance),
    )
    .map_err(AppBuildError::HttpClient)?
    .with_ingress_authenticator(ingress_authenticator);

    Ok(Arc::new(state))
}

/// Resolve the set of providers whose OAuth-subscription compliance status is
/// `APPROVED`. Threaded into [`AppState`] so the runtime admin-registration path
/// enforces the same fail-closed gate as boot-time `validate_provider_compliance`.
fn oauth_approved_providers(
    compliance: &ProviderComplianceConfig,
) -> std::collections::HashSet<Provider> {
    let mut approved = std::collections::HashSet::new();
    if compliance.anthropic_oauth_status == ProviderComplianceStatus::Approved {
        approved.insert(Provider::Anthropic);
    }
    if compliance.codex_oauth_status == ProviderComplianceStatus::Approved {
        approved.insert(Provider::Codex);
    }
    approved
}

/// Wire up all components with **in-process mocks** for unit and integration
/// tests. This constructor is always available (not gated behind a feature flag)
/// so the test suite never depends on live secret-provider / ClickHouse / Valkey.
pub fn build_app_for_tests(config: AppConfig) -> Result<Arc<AppState>, AppBuildError> {
    config.validate_provider_compliance()?;

    // Cedar gate.
    let gate = CedarAuthzGate::with_default_policy()
        .map_err(|e| AppBuildError::CedarPolicy(e.to_string()))?;

    // Tenant ID validation.
    let tenant_id = TenantId::new(&config.tenant_id)
        .map_err(|_| AppBuildError::Config(format!("invalid tenant_id: {:?}", config.tenant_id)))?;

    let pool_configs = effective_tenant_provider_pools(&config)?;

    // In-process secret store.
    let secret_store = Arc::new(InProcessSecretStore::new());
    for pool_config in &pool_configs {
        for seat in &pool_config.seats {
            secret_store.preload(&seat.secret_handle, "test-placeholder-token");
        }
    }

    // In-process event sink.
    let sink: Arc<dyn EventSink + Send + Sync> = Arc::new(InProcessEventSink);

    // Subscription pools.
    let (pool_registry, pool_arc) = build_pool_registry(tenant_id.clone(), &pool_configs)?;

    let state = AppState::new_with_pool_registry(
        pool_arc,
        pool_registry,
        Arc::new(gate),
        sink,
        secret_store,
        config.anthropic_base_url,
        tenant_id,
        config.ingress_bearer_token,
        config.admin_bearer_token,
        config.environment.clone(),
        oauth_approved_providers(&config.provider_compliance),
    )
    .map_err(AppBuildError::HttpClient)?;

    Ok(Arc::new(state))
}

fn effective_tenant_provider_pools(
    config: &AppConfig,
) -> Result<Vec<TenantProviderPoolConfig>, AppBuildError> {
    if !config.tenant_provider_pools.is_empty() {
        return Ok(config.tenant_provider_pools.clone());
    }

    Ok(vec![TenantProviderPoolConfig {
        tenant_id: config.tenant_id.clone(),
        provider: Provider::Anthropic,
        // The synthesized default Anthropic pool inherits the operator-declared
        // Anthropic auth mode so the fail-closed compliance gate sees the same
        // transport the proxy will actually use (instead of always asserting
        // OAuth and tripping the gate for an api-key deployment).
        credential_mode: config.provider_compliance.anthropic_auth_mode,
        strategy: SelectionStrategy::RoundRobin,
        seats: config
            .initial_seats
            .iter()
            .map(|(seat_id, secret_handle)| {
                validate_secret_handle(secret_handle)?;
                Ok(StaticSeatConfig {
                    seat_id: seat_id.clone(),
                    subscription_id: format!("{seat_id}-sub"),
                    secret_handle: secret_handle.clone(),
                })
            })
            .collect::<Result<Vec<_>, AppBuildError>>()?,
    }])
}

/// Build all static tenant/provider pools and return the default Anthropic
/// data-plane pool for this process's configured tenant.
fn build_pool_registry(
    default_tenant_id: TenantId,
    pool_configs: &[TenantProviderPoolConfig],
) -> Result<(PoolRegistry, Arc<Mutex<SubscriptionPool>>), AppBuildError> {
    let registry = PoolRegistry::new();
    let mut default_pool: Option<Arc<Mutex<SubscriptionPool>>> = None;

    for pool_config in pool_configs {
        let tenant_id = TenantId::new(pool_config.tenant_id.clone()).map_err(|_| {
            AppBuildError::Config(format!("invalid tenant_id: {:?}", pool_config.tenant_id))
        })?;
        let pool = build_pool_from_config(pool_config)?;
        if tenant_id == default_tenant_id && pool_config.provider == Provider::Anthropic {
            default_pool = Some(Arc::clone(&pool));
        }
        registry.insert_pool(tenant_id, pool_config.provider, pool);
    }

    if let Some(default_pool) = default_pool {
        return Ok((registry, default_pool));
    }

    let default_pool = Arc::new(Mutex::new(SubscriptionPool::new(
        default_tenant_id.clone(),
        Provider::Anthropic,
        SelectionStrategy::RoundRobin,
    )));
    registry.insert_pool(
        default_tenant_id,
        Provider::Anthropic,
        Arc::clone(&default_pool),
    );
    Ok((registry, default_pool))
}

/// Shared pool construction helper used by both `build_app` and
/// `build_app_for_tests`.
fn build_pool_from_config(
    pool_config: &TenantProviderPoolConfig,
) -> Result<Arc<Mutex<SubscriptionPool>>, AppBuildError> {
    let tenant_id = TenantId::new(pool_config.tenant_id.clone()).map_err(|_| {
        AppBuildError::Config(format!("invalid tenant_id: {:?}", pool_config.tenant_id))
    })?;
    let mut pool = SubscriptionPool::new(
        tenant_id.clone(),
        pool_config.provider,
        pool_config.strategy,
    );
    for seat in &pool_config.seats {
        let seat_id = SeatId::new(seat.seat_id.as_str()).map_err(|_| {
            AppBuildError::PoolSetup(format!("invalid seat_id in tenant pool: {}", seat.seat_id))
        })?;
        let sub_id = SubscriptionId::new(seat.subscription_id.clone()).map_err(|_| {
            AppBuildError::PoolSetup(format!("invalid subscription_id for: {}", seat.seat_id))
        })?;
        let sub = OAuthSubscription::new(
            tenant_id.clone(),
            seat_id,
            sub_id,
            pool_config.provider,
            SubscriptionState::Active,
            seat.secret_handle.clone(),
            0,
        )
        .with_credential_mode(pool_config.credential_mode.to_kernel());
        pool.add_seat(sub)
            .map_err(|e| AppBuildError::PoolSetup(format!("add_seat failed: {e:?}")))?;
    }
    Ok(Arc::new(Mutex::new(pool)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::Mutex as StdMutex;

    static ENV_LOCK: StdMutex<()> = StdMutex::new(());

    fn read_repo_file(path: &str) -> String {
        std::fs::read_to_string(path).unwrap_or_else(|err| {
            panic!(
                "failed to read {path} from {:?}: {err}",
                std::env::current_dir()
            )
        })
    }

    struct EnvOverride {
        previous: Vec<(&'static str, Option<OsString>)>,
    }

    impl EnvOverride {
        fn set(vars: &[(&'static str, &'static str)]) -> Self {
            let previous = vars
                .iter()
                .map(|(key, _)| (*key, std::env::var_os(key)))
                .collect();
            for (key, value) in vars {
                // SAFETY: this test serializes all process-environment mutation
                // through ENV_LOCK and restores every touched key in Drop.
                unsafe {
                    std::env::set_var(key, value);
                }
            }
            Self { previous }
        }
    }

    impl Drop for EnvOverride {
        fn drop(&mut self) {
            for (key, value) in &self.previous {
                // SAFETY: see EnvOverride::set; restoration is serialized by
                // the same ENV_LOCK guard held by the test body.
                unsafe {
                    match value {
                        Some(value) => std::env::set_var(key, value),
                        None => std::env::remove_var(key),
                    }
                }
            }
        }
    }

    fn test_config() -> AppConfig {
        AppConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            tenant_id: "test-tenant".to_string(),
            anthropic_base_url: "http://127.0.0.1:1".to_string(),
            initial_seats: vec![],
            // These fields are not used by build_app_for_tests (in-process mocks).
            secret_provider_url: "http://127.0.0.1:1".to_string(),
            secret_provider_token: "test-token".to_string(),
            transit_key_name: "cloud-intelligence-rt".to_string(),
            clickhouse_url: "http://127.0.0.1:1".to_string(),
            clickhouse_user: "default".to_string(),
            clickhouse_password: "test".to_string(),
            valkey_url: "redis://127.0.0.1:1".to_string(),
            admin_bearer_token: Some("admin-token".to_string()),
            ingress_bearer_token: Some("ingress-token".to_string()),
            ingress_bearer_map: vec![],
            environment: "test".to_string(),
            provider_compliance: ProviderComplianceConfig::default(),
            tenant_provider_pools: vec![],
        }
    }

    #[test]
    fn build_app_for_tests_returns_state_for_valid_config() {
        let config = test_config();
        let state = build_app_for_tests(config).unwrap();
        // Pool exists and has 0 seats (no initial_seats).
        let pool = state.pool.lock().unwrap();
        assert_eq!(pool.seat_count(), 0);
    }

    #[test]
    fn build_app_for_tests_registers_initial_seats() {
        let mut config = test_config();
        config.initial_seats = vec![
            (
                "seat-a".to_string(),
                "secret-ref://tenant-a/anthropic/seat-a".to_string(),
            ),
            (
                "seat-b".to_string(),
                "secret-ref://tenant-a/anthropic/seat-b".to_string(),
            ),
        ];
        let state = build_app_for_tests(config).unwrap();
        let pool = state.pool.lock().unwrap();
        assert_eq!(pool.seat_count(), 2);
    }

    #[test]
    fn build_app_for_tests_rejects_raw_initial_seat_secret() {
        let mut config = test_config();
        config.initial_seats = vec![("seat-a".to_string(), "sk-ant-api03-raw-secret".to_string())];

        let err = match build_app_for_tests(config) {
            Err(err) => err,
            Ok(_) => panic!("raw initial seat secret must be rejected"),
        };
        assert!(
            err.to_string().contains("secret handle"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn build_app_for_tests_fails_on_empty_tenant_id() {
        let config = AppConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            tenant_id: "".to_string(),
            anthropic_base_url: "http://127.0.0.1:1".to_string(),
            initial_seats: vec![],
            secret_provider_url: "http://127.0.0.1:1".to_string(),
            secret_provider_token: "test-token".to_string(),
            transit_key_name: "cloud-intelligence-rt".to_string(),
            clickhouse_url: "http://127.0.0.1:1".to_string(),
            clickhouse_user: "default".to_string(),
            clickhouse_password: "test".to_string(),
            valkey_url: "redis://127.0.0.1:1".to_string(),
            admin_bearer_token: None,
            ingress_bearer_token: None,
            ingress_bearer_map: vec![],
            environment: "test".to_string(),
            provider_compliance: ProviderComplianceConfig::default(),
            tenant_provider_pools: vec![],
        };
        match build_app_for_tests(config) {
            Err(err) => assert!(
                matches!(err, AppBuildError::Config(_)),
                "expected Config error, got: {err}"
            ),
            Ok(_) => panic!("expected error for empty tenant_id but got Ok"),
        }
    }

    #[test]
    fn parse_initial_seats_parses_correctly() {
        let seats = parse_initial_seats(
            "seat-a:secret-ref://tenant-a/anthropic/seat-a,seat-b:secret-ref://tenant-a/anthropic/seat-b",
        )
        .unwrap();
        assert_eq!(seats.len(), 2);
        assert_eq!(
            seats[0],
            (
                "seat-a".to_string(),
                "secret-ref://tenant-a/anthropic/seat-a".to_string()
            )
        );
        assert_eq!(
            seats[1],
            (
                "seat-b".to_string(),
                "secret-ref://tenant-a/anthropic/seat-b".to_string()
            )
        );
    }

    #[test]
    fn parse_initial_seats_empty_string_returns_empty() {
        assert!(parse_initial_seats("").unwrap().is_empty());
        assert!(parse_initial_seats("   ").unwrap().is_empty());
    }

    #[test]
    fn parse_initial_seats_rejects_raw_provider_secret() {
        let err = parse_initial_seats("seat-a:sk-ant-api03-raw-secret")
            .expect_err("raw initial provider secret must be rejected");
        assert!(
            err.to_string().contains("secret handle"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn parse_tenant_provider_pools_supports_multiple_tenants_and_providers() {
        let pools = parse_tenant_provider_pools(
            "tenant-a|anthropic|oauth_subscription|time_normalized_quota_percent|seat-a|secret-ref://tenant-a/anthropic/seat-a;\
             tenant-b|codex|api_key|round_robin|seat-b|secret-ref://tenant-b/codex/seat-b;\
             tenant-c|gemini|api_key|round_robin|seat-c|secret-ref://tenant-c/gemini/seat-c",
        )
        .expect("multi-tenant provider config parses");

        assert_eq!(pools.len(), 3);
        assert_eq!(pools[0].tenant_id, "tenant-a");
        assert_eq!(pools[0].provider, Provider::Anthropic);
        assert_eq!(pools[0].credential_mode, CredentialMode::OAuthSubscription);
        assert_eq!(
            pools[0].strategy,
            SelectionStrategy::TimeNormalizedQuotaPercent
        );
        assert_eq!(
            pools[0].seats[0].secret_handle,
            "secret-ref://tenant-a/anthropic/seat-a"
        );
        assert_eq!(pools[1].tenant_id, "tenant-b");
        assert_eq!(pools[1].provider, Provider::Codex);
        assert_eq!(pools[1].credential_mode, CredentialMode::ApiKey);
        assert_eq!(pools[2].tenant_id, "tenant-c");
        assert_eq!(pools[2].provider, Provider::Gemini);
        assert_eq!(pools[2].credential_mode, CredentialMode::ApiKey);
    }

    #[test]
    fn parse_tenant_provider_pools_rejects_obvious_raw_provider_secrets() {
        let err = parse_tenant_provider_pools(
            "tenant-a|anthropic|oauth_subscription|round_robin|seat-a|sk-ant-api03-raw-secret",
        )
        .expect_err("raw provider secret must be rejected");

        assert!(
            err.to_string().contains("secret handle"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn parse_ingress_bearer_map_parses_multi_tenant_bindings() {
        let bindings = parse_ingress_bearer_map("tenant-a|tok-a; tenant-b|tok-b").unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0.as_str(), "tenant-a");
        assert_eq!(bindings[0].1, "tok-a");
        assert_eq!(bindings[1].0.as_str(), "tenant-b");
        assert_eq!(bindings[1].1, "tok-b");

        // Empty input => empty map (single-tenant behavior preserved).
        assert!(parse_ingress_bearer_map("").unwrap().is_empty());
        assert!(parse_ingress_bearer_map("   ").unwrap().is_empty());
    }

    #[test]
    fn parse_ingress_bearer_map_rejects_malformed_entries() {
        // Missing token field.
        assert!(parse_ingress_bearer_map("tenant-a").is_err());
        // Empty token.
        assert!(parse_ingress_bearer_map("tenant-a|").is_err());
        // Empty tenant.
        assert!(parse_ingress_bearer_map("|tok-a").is_err());
    }

    #[test]
    fn parse_ingress_bearer_map_rejects_duplicate_token() {
        // A token shared across two tenants would silently authenticate as the wrong tenant
        // (last-wins in the verify loop) => fail closed at boot.
        assert!(parse_ingress_bearer_map("tenant-a|shared; tenant-b|shared").is_err());
        // Distinct tokens for the same tenant ARE allowed (credential rotation).
        let rotated = parse_ingress_bearer_map("tenant-a|tok-old; tenant-a|tok-new").unwrap();
        assert_eq!(rotated.len(), 2);
    }

    #[test]
    fn build_app_for_tests_registers_static_tenant_provider_pools() {
        let mut config = test_config();
        config.tenant_id = "tenant-a".to_string();
        config.tenant_provider_pools = parse_tenant_provider_pools(
            "tenant-a|anthropic|oauth_subscription|time_normalized_quota_percent|seat-a|secret-ref://tenant-a/anthropic/seat-a;\
             tenant-b|codex|api_key|round_robin|seat-b|secret-ref://tenant-b/codex/seat-b",
        )
        .unwrap();

        let state = build_app_for_tests(config).unwrap();

        assert_eq!(state.pool.lock().unwrap().seat_count(), 1);
        assert_eq!(state.pool_registry.pool_count(), 2);
        assert_eq!(
            state
                .pool_registry
                .pool_status(&TenantId::new("tenant-b").unwrap(), Provider::Codex)
                .expect("tenant-b codex pool")
                .total_seats,
            1
        );
    }

    #[test]
    fn production_oauth_subscription_without_provider_approval_fails_closed() {
        let _guard = ENV_LOCK.lock().unwrap();
        let _env = EnvOverride::set(&[
            ("OYA_CLOUD_INTEL_LISTEN_ADDR", "127.0.0.1:0"),
            ("OYA_CLOUD_INTEL_TENANT_ID", "tenant-a"),
            ("OYA_CLOUD_INTEL_ANTHROPIC_URL", "http://127.0.0.1:1"),
            (
                "OYA_CLOUD_INTEL_INITIAL_SEATS",
                "seat-a:secret-ref://tenant-a/anthropic/seat-a",
            ),
            (
                "OYA_CLOUD_INTEL_SECRET_PROVIDER_URL",
                "http://127.0.0.1:8200",
            ),
            ("OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN", "vault-token"),
            ("OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD", "clickhouse-password"),
            ("OYA_CLOUD_INTEL_ENVIRONMENT", "production"),
            ("OYA_CLOUD_INTEL_ANTHROPIC_AUTH_MODE", "oauth_subscription"),
            ("OYA_CLOUD_INTEL_ANTHROPIC_OAUTH_STATUS", "PENDING"),
        ]);

        let err = AppConfig::from_env().expect_err("production OAuth must fail closed");
        assert!(
            err.to_string().contains("provider compliance not approved"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn production_anthropic_api_key_mode_is_allowed_after_api_key_adapter_exists() {
        let mut config = test_config();
        config.environment = "production".to_string();
        config.provider_compliance.anthropic_auth_mode = CredentialMode::ApiKey;
        config.provider_compliance.anthropic_oauth_status = ProviderComplianceStatus::ApiOnly;

        build_app_for_tests(config).expect("Anthropic API-key production mode should boot");
    }

    #[test]
    fn production_oauth_pool_overrides_env_api_key_mode_and_fails_closed() {
        let mut config = test_config();
        config.environment = "production".to_string();
        // Env-declared mode claims api_key (which would pass the env-only gate)
        // but the loaded pool actually uses OAuth subscription credentials.
        config.provider_compliance.anthropic_auth_mode = CredentialMode::ApiKey;
        config.provider_compliance.anthropic_oauth_status = ProviderComplianceStatus::Pending;
        config.tenant_provider_pools = vec![TenantProviderPoolConfig {
            tenant_id: config.tenant_id.clone(),
            provider: Provider::Anthropic,
            credential_mode: CredentialMode::OAuthSubscription,
            strategy: SelectionStrategy::RoundRobin,
            seats: vec![StaticSeatConfig {
                seat_id: "seat-a".to_string(),
                subscription_id: "seat-a-sub".to_string(),
                secret_handle: "secret-ref://test-tenant/anthropic/seat-a".to_string(),
            }],
        }];

        let err = config
            .validate_provider_compliance()
            .expect_err("OAuth pool with pending status must fail closed");
        assert!(
            err.to_string().contains("provider compliance not approved"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn production_oauth_pool_passes_when_provider_status_approved() {
        let mut config = test_config();
        config.environment = "production".to_string();
        config.provider_compliance.anthropic_auth_mode = CredentialMode::ApiKey;
        config.provider_compliance.anthropic_oauth_status = ProviderComplianceStatus::Approved;
        config.tenant_provider_pools = vec![TenantProviderPoolConfig {
            tenant_id: config.tenant_id.clone(),
            provider: Provider::Anthropic,
            credential_mode: CredentialMode::OAuthSubscription,
            strategy: SelectionStrategy::RoundRobin,
            seats: vec![StaticSeatConfig {
                seat_id: "seat-a".to_string(),
                subscription_id: "seat-a-sub".to_string(),
                secret_handle: "secret-ref://test-tenant/anthropic/seat-a".to_string(),
            }],
        }];

        config
            .validate_provider_compliance()
            .expect("approved OAuth pool must pass the fail-closed gate");
    }

    #[test]
    fn production_api_key_only_pool_passes_with_pending_oauth_status() {
        let mut config = test_config();
        config.environment = "production".to_string();
        config.provider_compliance.anthropic_auth_mode = CredentialMode::ApiKey;
        config.provider_compliance.anthropic_oauth_status = ProviderComplianceStatus::Pending;
        config.tenant_provider_pools = vec![TenantProviderPoolConfig {
            tenant_id: config.tenant_id.clone(),
            provider: Provider::Anthropic,
            credential_mode: CredentialMode::ApiKey,
            strategy: SelectionStrategy::RoundRobin,
            seats: vec![StaticSeatConfig {
                seat_id: "seat-a".to_string(),
                subscription_id: "seat-a-sub".to_string(),
                secret_handle: "secret-ref://test-tenant/anthropic/seat-a".to_string(),
            }],
        }];

        config
            .validate_provider_compliance()
            .expect("api-key-only pool must pass even with pending OAuth status");
    }

    #[test]
    fn helm_template_declares_all_boot_required_env_vars() {
        let deployment_template =
            read_repo_file("intelligence/iac/k8s/helm/templates/deployment.yaml");
        for expected in [
            "OYA_CLOUD_INTEL_LISTEN_ADDR",
            "OYA_CLOUD_INTEL_TENANT_ID",
            "OYA_CLOUD_INTEL_ENVIRONMENT",
            "OYA_CLOUD_INTEL_INITIAL_SEATS",
            "OYA_CLOUD_INTEL_TENANT_PROVIDER_POOLS",
            "OYA_CLOUD_INTEL_SECRET_PROVIDER_URL",
            "OYA_CLOUD_INTEL_SECRET_PROVIDER_TOKEN",
            "OYA_CLOUD_INTEL_CLICKHOUSE_PASSWORD",
            "OYA_CLOUD_INTEL_ADMIN_BEARER_TOKEN",
            "OYA_CLOUD_INTEL_INGRESS_BEARER_TOKEN",
            "OYA_CLOUD_INTEL_ANTHROPIC_AUTH_MODE",
            "OYA_CLOUD_INTEL_ANTHROPIC_OAUTH_STATUS",
            "OYA_CLOUD_INTEL_CODEX_AUTH_MODE",
            "OYA_CLOUD_INTEL_CODEX_OAUTH_STATUS",
        ] {
            assert!(
                deployment_template.contains(expected),
                "deployment template missing env var {expected}"
            );
        }
    }

    #[test]
    fn core_boundaries_use_owned_secret_provider_port_not_transient_adapter_names() {
        let app_source = read_repo_file("intelligence/facade/app/src/lib.rs");
        let rest_source = read_repo_file("intelligence/adapters/rest/src/lib.rs");
        let deployment_template =
            read_repo_file("intelligence/iac/k8s/helm/templates/deployment.yaml");

        assert!(
            app_source.contains("SecretProvider") && rest_source.contains("SecretProvider"),
            "core cloud-intelligence Rust boundary should expose the owned secret-provider port"
        );
        assert!(
            deployment_template.contains("SECRET_PROVIDER"),
            "cloud deployment should expose owned secret-provider env vars"
        );
        let forbidden = [
            ["OpenBao", "SecretStore"].concat(),
            ["OYA_CLOUD_INTEL_", "OPEN", "BAO", "_URL"].concat(),
            ["OYA_CLOUD_INTEL_", "OPEN", "BAO", "_TOKEN"].concat(),
        ];
        for forbidden in forbidden {
            assert!(
                !app_source.contains(&forbidden)
                    && !rest_source.contains(&forbidden)
                    && !deployment_template.contains(&forbidden),
                "core cloud-intelligence boundary leaked transient adapter identifier {forbidden}"
            );
        }
    }

    #[test]
    fn probe_paths_are_consistent_between_helm_and_openapi() {
        let deployment_template =
            read_repo_file("intelligence/iac/k8s/helm/templates/deployment.yaml");
        let openapi_contract =
            read_repo_file("intelligence/contracts/cloud-intelligence.openapi.yaml");
        for path in ["/healthz", "/livez", "/readyz"] {
            assert!(
                deployment_template.contains(&format!("path: {path}"))
                    || deployment_template.contains(&format!("path: \"{path}\"")),
                "deployment template does not probe {path}"
            );
            assert!(
                openapi_contract.contains(&format!("  {path}:")),
                "OpenAPI contract does not declare {path}"
            );
        }
    }

    #[test]
    fn tenant_subscription_openapi_matches_runtime_registration_semantics() {
        let openapi_contract =
            read_repo_file("intelligence/contracts/cloud-intelligence.openapi.yaml");
        let operation_start = openapi_contract
            .find("  /admin/v1/tenants/{tenant_id}/providers/{provider}/subscriptions:\n")
            .expect("tenant subscription admin path missing from OpenAPI");
        let operation_end = operation_start
            + openapi_contract[operation_start..]
                .find("\ncomponents:")
                .expect("components section missing after tenant subscription path");
        let operation = &openapi_contract[operation_start..operation_end];

        assert!(
            operation
                .contains("$ref: '#/components/parameters/SubscriptionRegistrationIdempotencyKey'"),
            "subscription registration must use its bounded-token idempotency contract"
        );
        assert!(
            !operation.contains("$ref: '#/components/parameters/IdempotencyKey'"),
            "subscription registration runtime does not implement the global UUID/24h idempotency contract"
        );
        assert!(
            !operation.contains("        '404':"),
            "subscription registration creates missing tenant/provider pools instead of returning 404"
        );
        assert!(
            operation.contains("        '401':") && operation.contains("        '409':"),
            "subscription registration must document admin auth failures and duplicate-seat conflicts"
        );

        let parameter_start = openapi_contract
            .find("    SubscriptionRegistrationIdempotencyKey:\n")
            .expect("subscription registration idempotency parameter missing");
        let parameter_end = parameter_start
            + openapi_contract[parameter_start..]
                .find("\n    AdminTenantHeader:")
                .expect("admin tenant header should follow subscription idempotency parameter");
        let parameter = &openapi_contract[parameter_start..parameter_end];
        assert!(parameter.contains("pattern: '^[A-Za-z0-9_-]{1,128}$'"));
        assert!(!parameter.contains("format: uuid"));
        assert!(!parameter.contains("24h"));
    }

    #[test]
    fn external_secret_exposes_handles_not_raw_provider_credentials() {
        let external_secret_template =
            read_repo_file("intelligence/iac/k8s/helm/templates/externalsecret.yaml");
        for forbidden in [
            "anthropic_refresh_token",
            "openai_api_key",
            "gemini_api_key",
            "claude_api_key",
        ] {
            assert!(
                !external_secret_template.contains(forbidden),
                "ExternalSecret must not materialize raw provider credential key {forbidden}"
            );
        }
        for expected in [
            "initial_seats",
            "tenant_provider_pools",
            "secret_provider_token",
            "clickhouse_password",
            "admin_bearer_token",
            "ingress_bearer_token",
        ] {
            assert!(
                external_secret_template.contains(expected),
                "ExternalSecret missing launch secret {expected}"
            );
        }
    }

    #[tokio::test]
    async fn in_process_secret_store_roundtrips() {
        let store = InProcessSecretStore::new();
        store.preload("h1", "rt-1");
        let fetched = store.fetch_refresh_token("h1").await.unwrap();
        assert_eq!(fetched, "rt-1");
    }

    #[tokio::test]
    async fn in_process_secret_store_not_found() {
        let store = InProcessSecretStore::new();
        let err = store.fetch_refresh_token("missing").await.unwrap_err();
        assert_eq!(err, RestAdapterError::SecretNotFound);
    }

    #[tokio::test]
    async fn in_process_secret_store_rejects_empty_plaintext() {
        let store = InProcessSecretStore::new();
        let err = store.store_refresh_token("h1", "").await.unwrap_err();
        assert_eq!(err, RestAdapterError::InvalidSecret);
    }
}
