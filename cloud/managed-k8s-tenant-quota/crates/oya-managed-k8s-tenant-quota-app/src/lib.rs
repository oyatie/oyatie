//! Composition root for the managed-K8s tenant quota service.
//!
//! This crate wires the kernel + api + adapters into an axum HTTP service
//! exposing the quota admin REST API:
//!
//! ```text
//! PUT  /tenants/{id}/quota    — set or replace tenant quota (TenantAdmin/PlatformOperator)
//! GET  /tenants/{id}/quota    — read tenant quota
//! GET  /tenants/{id}/usage    — read tenant cluster resource usage
//! POST /tenants/{id}/quota/check — check whether a provisioning request is within quota
//! GET  /healthz               — liveness probe
//! ```
//!
//! ## Layering (ADR-0131 / ADR-0376)
//!
//! - Depends path-inward on `-kernel`, `-api`, `-adapter-cedar`, `-adapter-inmemory`.
//! - Owns **no** policy algorithm; delegates to `evaluate()` in the kernel.
//! - Billing hooks = typed `Unimplemented::BillingEmission`; tracked in
//!   `registry/placeholder-debt/adr-follow-ups.yaml#adr-0376-billing-emission`.
//!
//! ## Honest boundaries
//!
//! Where a downstream is not yet wired, this crate surfaces a typed
//! [`Unimplemented`] code and a `registry/placeholder-debt` entry.
//! No stubbed `Ok(())` for paths the service claims but does not implement.

// ADR-0083 Tier-3: production code stays panic-free.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use oya_managed_k8s_tenant_quota_adapter_inmemory::InMemoryQuotaStore;
use serde::{Deserialize, Serialize};

pub use oya_managed_k8s_tenant_quota_api::{
    QuotaAdminPort, QuotaCheckResponse, QuotaDecisionPort, QuotaDto, QuotaPortError, UsageDto,
};
pub use oya_managed_k8s_tenant_quota_kernel::{ProvisionRequest, QuotaDecision, TenantId};

// ============================================================
// Unimplemented placeholders (honest-claims)
// ============================================================

/// Typed placeholder for unimplemented downstream integrations.
/// Each variant is tracked in `registry/placeholder-debt/adr-follow-ups.yaml`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unimplemented {
    /// Billing emission on quota set/change event.
    /// Tracked: `registry/placeholder-debt/adr-follow-ups.yaml#adr-0376-billing-emission`
    BillingEmission,
    /// Audit-chain event emission on quota decisions.
    /// Tracked: `registry/placeholder-debt/adr-follow-ups.yaml#adr-0376-audit-chain-emission`
    AuditChainEmission,
}

impl Unimplemented {
    /// Stable type slug for observability / log output.
    #[must_use]
    pub fn type_slug(&self) -> &'static str {
        match self {
            Self::BillingEmission => "quota_unimplemented_billing_emission",
            Self::AuditChainEmission => "quota_unimplemented_audit_chain_emission",
        }
    }
}

impl fmt::Display for Unimplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unimplemented({})", self.type_slug())
    }
}

// ============================================================
// Application state
// ============================================================

/// Shared application state injected into every axum handler.
pub struct AppState<S> {
    /// The quota store implementing both `QuotaDecisionPort` and `QuotaAdminPort`.
    pub store: S,
}

/// Type alias for the arc-wrapped state used by handlers.
pub type SharedState<S> = Arc<AppState<S>>;

// ============================================================
// Request / response bodies
// ============================================================

/// Body for `POST /tenants/{id}/quota/check`.
#[derive(Clone, Debug, Deserialize)]
pub struct CheckRequestBody {
    pub requested_clusters: u32,
    pub requested_nodes_per_cluster: u32,
    pub requested_vcpu_per_cluster: u32,
    pub requested_ram_gib_per_cluster: u32,
}

/// Error response body.
#[derive(Clone, Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

/// Durable quota store DSN expected by production boot.
pub const QUOTA_DATABASE_URL_ENV: &str = "OYA_MANAGED_K8S_TENANT_QUOTA_DATABASE_URL";
/// Explicit local/dev escape hatch for the fake in-memory quota store.
pub const ALLOW_IN_MEMORY_QUOTA_STORE_ENV: &str = "OYA_MANAGED_K8S_TENANT_QUOTA_ALLOW_IN_MEMORY";

// ============================================================
// Handlers
// ============================================================

/// `PUT /tenants/{id}/quota` — set or replace tenant quota.
pub async fn put_quota<S>(
    State(state): State<SharedState<S>>,
    Path(tenant_id): Path<String>,
    Json(body): Json<QuotaDto>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)>
where
    S: QuotaAdminPort + Send + Sync,
{
    // Enforce path-body tenant_id consistency.
    if body.tenant_id != tenant_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: format!(
                    "path tenant_id {tenant_id} does not match body tenant_id {}",
                    body.tenant_id
                ),
            }),
        ));
    }
    let quota = body.into_quota().map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;

    // Billing emission placeholder — not yet wired.
    tracing::debug!(
        unimplemented = Unimplemented::BillingEmission.type_slug(),
        "billing emission on quota set is not yet implemented (ADR-0376 follow-up)"
    );

    state.store.set_quota(quota).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /tenants/{id}/quota` — read tenant quota.
pub async fn get_quota<S>(
    State(state): State<SharedState<S>>,
    Path(tenant_id): Path<String>,
) -> Result<Json<QuotaDto>, (StatusCode, Json<ErrorBody>)>
where
    S: QuotaAdminPort + Send + Sync,
{
    let tid = TenantId::new(&tenant_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    let quota = state.store.get_quota(&tid).map_err(|e| match e {
        QuotaPortError::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        ),
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: other.to_string(),
            }),
        ),
    })?;
    Ok(Json(QuotaDto::from(quota)))
}

/// `GET /tenants/{id}/usage` — read tenant cluster usage.
pub async fn get_usage<S>(
    State(state): State<SharedState<S>>,
    Path(tenant_id): Path<String>,
) -> Result<Json<UsageDto>, (StatusCode, Json<ErrorBody>)>
where
    S: QuotaAdminPort + Send + Sync,
{
    let tid = TenantId::new(&tenant_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    let usage = state.store.get_usage(&tid).map_err(|e| match e {
        QuotaPortError::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        ),
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: other.to_string(),
            }),
        ),
    })?;
    Ok(Json(UsageDto::from(usage)))
}

/// `POST /tenants/{id}/quota/check` — check a provisioning request against quota.
pub async fn check_quota<S>(
    State(state): State<SharedState<S>>,
    Path(tenant_id): Path<String>,
    Json(body): Json<CheckRequestBody>,
) -> Result<Json<QuotaCheckResponse>, (StatusCode, Json<ErrorBody>)>
where
    S: QuotaDecisionPort + Send + Sync,
{
    let request = ProvisionRequest::new(
        &tenant_id,
        body.requested_clusters,
        body.requested_nodes_per_cluster,
        body.requested_vcpu_per_cluster,
        body.requested_ram_gib_per_cluster,
    )
    .map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;

    // Audit-chain emission placeholder — not yet wired.
    tracing::debug!(
        unimplemented = Unimplemented::AuditChainEmission.type_slug(),
        "audit-chain emission on quota check is not yet implemented (ADR-0376 follow-up)"
    );

    let decision = state.store.check_quota(&request).map_err(|e| match e {
        QuotaPortError::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        ),
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: other.to_string(),
            }),
        ),
    })?;

    Ok(Json(QuotaCheckResponse::from(decision)))
}

/// `GET /healthz` — liveness probe.
pub async fn healthz() -> StatusCode {
    StatusCode::OK
}

// ============================================================
// Router builder
// ============================================================

/// Build the axum router for the quota service.
///
/// `store` must implement both `QuotaDecisionPort` and `QuotaAdminPort`.
pub fn build_router<S>(store: S) -> Router
where
    S: QuotaDecisionPort + QuotaAdminPort + Clone + Send + Sync + 'static,
{
    let state: SharedState<S> = Arc::new(AppState { store });
    Router::new()
        .route("/tenants/{id}/quota", put(put_quota::<S>))
        .route("/tenants/{id}/quota", get(get_quota::<S>))
        .route("/tenants/{id}/usage", get(get_usage::<S>))
        .route("/tenants/{id}/quota/check", post(check_quota::<S>))
        .route("/healthz", get(healthz))
        .with_state(state)
}

/// Boot errors.
#[derive(Debug)]
pub enum BootError {
    /// Production boot did not receive a durable quota store adapter.
    ProductionAdapterUnavailable,
    /// TCP listener bind failure.
    Bind { address: String, error: String },
    /// Axum serve loop exited with an error.
    Serve(String),
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProductionAdapterUnavailable => write!(
                f,
                "durable quota store adapter is required; refusing implicit in-memory fallback \
                 (set {ALLOW_IN_MEMORY_QUOTA_STORE_ENV}=true only for local/dev tests)"
            ),
            Self::Bind { address, error } => write!(f, "bind {address}: {error}"),
            Self::Serve(e) => write!(f, "serve error: {e}"),
        }
    }
}

impl std::error::Error for BootError {}

/// Build the quota store from process environment.
///
/// The in-memory adapter is a test/dev fake, so production boot fails closed
/// unless callers explicitly opt in with
/// [`ALLOW_IN_MEMORY_QUOTA_STORE_ENV`]. A configured durable DSN never silently
/// falls back to the fake store.
///
/// # Errors
/// Returns [`BootError::ProductionAdapterUnavailable`] when no explicit dev
/// in-memory opt-in is present or when a durable DSN is configured but this
/// app slice cannot safely compose a fake fallback.
pub fn build_state_from_env() -> Result<InMemoryQuotaStore, BootError> {
    let database_url = std::env::var(QUOTA_DATABASE_URL_ENV).ok();
    let allow_in_memory = std::env::var(ALLOW_IN_MEMORY_QUOTA_STORE_ENV).ok();
    build_state_from_env_values(database_url.as_deref(), allow_in_memory.as_deref())
}

/// Deterministic companion to [`build_state_from_env`] for boot-policy tests.
///
/// # Errors
/// Returns [`BootError::ProductionAdapterUnavailable`] unless the durable DSN
/// is absent and `allow_in_memory` is an explicit truthy dev opt-in.
pub fn build_state_from_env_values(
    database_url: Option<&str>,
    allow_in_memory: Option<&str>,
) -> Result<InMemoryQuotaStore, BootError> {
    if database_url.is_some_and(|value| !value.trim().is_empty()) {
        return Err(BootError::ProductionAdapterUnavailable);
    }
    if env_value_is_truthy(allow_in_memory) {
        return Ok(InMemoryQuotaStore::new());
    }
    Err(BootError::ProductionAdapterUnavailable)
}

fn env_value_is_truthy(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("true" | "1" | "yes")
    )
}

/// Bind and serve the quota service.
///
/// # Errors
/// Returns [`BootError`] if the listener cannot bind or the serve loop exits.
pub async fn serve<S>(listen_addr: &str, store: S) -> Result<(), BootError>
where
    S: QuotaDecisionPort + QuotaAdminPort + Clone + Send + Sync + 'static,
{
    let app = build_router(store);
    let listener = tokio::net::TcpListener::bind(listen_addr)
        .await
        .map_err(|e| BootError::Bind {
            address: listen_addr.to_string(),
            error: e.to_string(),
        })?;
    tracing::info!(addr = listen_addr, "managed-k8s-tenant-quota listening");
    axum::serve(listener, app)
        .await
        .map_err(|e| BootError::Serve(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unimplemented_slugs_are_stable() {
        assert_eq!(
            Unimplemented::BillingEmission.type_slug(),
            "quota_unimplemented_billing_emission"
        );
        assert_eq!(
            Unimplemented::AuditChainEmission.type_slug(),
            "quota_unimplemented_audit_chain_emission"
        );
    }

    #[test]
    fn production_boot_without_durable_quota_store_config_fails_closed() {
        assert!(matches!(
            build_state_from_env_values(None, None),
            Err(BootError::ProductionAdapterUnavailable)
        ));
        assert!(matches!(
            build_state_from_env_values(None, Some("false")),
            Err(BootError::ProductionAdapterUnavailable)
        ));
    }

    #[test]
    fn in_memory_quota_store_requires_explicit_dev_opt_in() {
        assert!(build_state_from_env_values(None, Some("true")).is_ok());
    }

    #[test]
    fn durable_store_config_never_falls_back_to_in_memory() {
        assert!(matches!(
            build_state_from_env_values(Some("postgres://quota-store"), Some("true")),
            Err(BootError::ProductionAdapterUnavailable)
        ));
    }
}
