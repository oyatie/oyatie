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
use axum::extract::{FromRequestParts, Path, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::routing::{get, post, put};
use serde::{Deserialize, Serialize};

pub mod authz;

pub use authz::{
    AuthzProviderConfigError, CallerCredential, CedarQuotaAuthorizer,
    ConfiguredBearerPrincipalVerifier, PrincipalVerificationError, PrincipalVerifier, QuotaAction,
    QuotaAuthorizationError, QuotaAuthorizer, QuotaAuthzProvider, VerifiedPrincipal,
};

pub use k8s_tenant_quota_api::{
    QuotaAdminPort, QuotaCheckResponse, QuotaDecisionPort, QuotaDto, QuotaPortError, UsageDto,
};
pub use k8s_tenant_quota_kernel::{ProvisionRequest, QuotaDecision, TenantId};

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
///
/// The [`QuotaAuthzProvider`] is REQUIRED and non-optional: there is no
/// constructor that yields state without it, so the router can NEVER be built
/// without a configured principal-verification + PDP authorization seam (no
/// default-allow fallback — AUTH-005 fail-closed boot doctrine; GitHub #979).
pub struct AppState<S> {
    /// The quota store implementing both `QuotaDecisionPort` and `QuotaAdminPort`.
    pub store: S,
    /// The fail-closed authorization provider (verifier port + PDP port).
    pub authz: QuotaAuthzProvider,
}

/// Type alias for the arc-wrapped state used by handlers.
pub type SharedState<S> = Arc<AppState<S>>;

// ============================================================
// VerifiedCaller — authn-BEFORE-body extractor
// ============================================================

/// A request extractor that authenticates the caller from the verified bearer
/// credential over the request PARTS — i.e. BEFORE the request body is read.
///
/// `FromRequestParts` extractors run BEFORE the body `FromRequest` extractor, so
/// placing `VerifiedCaller` ahead of `Json(body)` in a handler signature
/// GUARANTEES authentication runs before the body is buffered or deserialized:
/// an unauthenticated caller is short-circuited 401 WITHOUT the body ever being
/// parsed. It carries the verified [`VerifiedPrincipal`]; the handler then runs
/// the PDP decision via [`QuotaAuthzProvider::ensure_authorized`].
pub struct VerifiedCaller(pub VerifiedPrincipal);

impl<S> FromRequestParts<SharedState<S>> for VerifiedCaller
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState<S>,
    ) -> Result<Self, Self::Rejection> {
        let credential = CallerCredential {
            authorization: parts
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned()),
        };
        match state.authz.verify_principal(&credential) {
            Ok(principal) => Ok(Self(principal)),
            Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "a verified caller credential is required".to_owned(),
                }),
            )),
        }
    }
}

/// Map a [`QuotaAuthorizationError`] to a fail-closed HTTP 403 response. A PDP
/// deny AND a PDP fault both surface as 403 (never a 5xx) — fail-closed.
fn forbidden() -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorBody {
            error: "caller is not authorized for this quota operation".to_owned(),
        }),
    )
}

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

// ============================================================
// Handlers
// ============================================================

/// `PUT /tenants/{id}/quota` — set or replace tenant quota.
///
/// `VerifiedCaller` (a `FromRequestParts` extractor) precedes `Json(body)` so
/// authn runs BEFORE the body is parsed (401 on no/bad bearer), then
/// `ensure_authorized` runs the PDP decision (403 on deny/fault) BEFORE any
/// mutation — fail-closed.
pub async fn put_quota<S>(
    State(state): State<SharedState<S>>,
    VerifiedCaller(principal): VerifiedCaller,
    Path(tenant_id): Path<String>,
    Json(body): Json<QuotaDto>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)>
where
    S: QuotaAdminPort + Send + Sync,
{
    let target = TenantId::new(&tenant_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    state
        .authz
        .ensure_authorized(&principal, QuotaAction::Write, &target)
        .map_err(|_| forbidden())?;
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

/// `GET /tenants/{id}/quota` — read tenant quota. Authorized for `Read` against
/// the target tenant (cross-tenant reads are denied by the PDP — isolation).
pub async fn get_quota<S>(
    State(state): State<SharedState<S>>,
    VerifiedCaller(principal): VerifiedCaller,
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
    state
        .authz
        .ensure_authorized(&principal, QuotaAction::Read, &tid)
        .map_err(|_| forbidden())?;
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

/// `GET /tenants/{id}/usage` — read tenant cluster usage. Authorized for `Read`
/// against the target tenant (cross-tenant reads denied by the PDP — isolation).
pub async fn get_usage<S>(
    State(state): State<SharedState<S>>,
    VerifiedCaller(principal): VerifiedCaller,
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
    state
        .authz
        .ensure_authorized(&principal, QuotaAction::Read, &tid)
        .map_err(|_| forbidden())?;
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
///
/// A read-class decision (it reads the tenant's quota + usage), authorized for
/// `Read` against the target tenant. `VerifiedCaller` runs authn before the body
/// is parsed; the PDP decision runs before the check — fail-closed.
pub async fn check_quota<S>(
    State(state): State<SharedState<S>>,
    VerifiedCaller(principal): VerifiedCaller,
    Path(tenant_id): Path<String>,
    Json(body): Json<CheckRequestBody>,
) -> Result<Json<QuotaCheckResponse>, (StatusCode, Json<ErrorBody>)>
where
    S: QuotaDecisionPort + Send + Sync,
{
    let target = TenantId::new(&tenant_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                error: e.to_string(),
            }),
        )
    })?;
    state
        .authz
        .ensure_authorized(&principal, QuotaAction::Read, &target)
        .map_err(|_| forbidden())?;
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
/// `store` must implement both `QuotaDecisionPort` and `QuotaAdminPort`. The
/// REQUIRED [`QuotaAuthzProvider`] makes the surface fail-closed by construction:
/// there is no authz-less overload (no default-allow control plane).
pub fn build_router<S>(store: S, authz: QuotaAuthzProvider) -> Router
where
    S: QuotaDecisionPort + QuotaAdminPort + Clone + Send + Sync + 'static,
{
    let state: SharedState<S> = Arc::new(AppState { store, authz });
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
    /// TCP listener bind failure.
    Bind { address: String, error: String },
    /// Axum serve loop exited with an error.
    Serve(String),
    /// The authorization provider could not be composed (empty bearer secret /
    /// bound identity, or a Cedar policy compile failure). The service REFUSES
    /// to serve — there is no default-allow fallback when authz is unavailable.
    Authz(String),
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bind { address, error } => write!(f, "bind {address}: {error}"),
            Self::Serve(e) => write!(f, "serve error: {e}"),
            Self::Authz(e) => write!(f, "authz provider boot refused: {e}"),
        }
    }
}

impl std::error::Error for BootError {}

impl From<AuthzProviderConfigError> for BootError {
    fn from(error: AuthzProviderConfigError) -> Self {
        Self::Authz(error.to_string())
    }
}

/// Bind and serve the quota service with the REQUIRED authz provider.
///
/// # Errors
/// Returns [`BootError`] if the listener cannot bind or the serve loop exits.
pub async fn serve<S>(
    listen_addr: &str,
    store: S,
    authz: QuotaAuthzProvider,
) -> Result<(), BootError>
where
    S: QuotaDecisionPort + QuotaAdminPort + Clone + Send + Sync + 'static,
{
    let app = build_router(store, authz);
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
}
