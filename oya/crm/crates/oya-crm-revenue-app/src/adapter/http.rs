//! CRM REST adapter.
//!
//! ## AUTH-005 fail-closed seam (ADR-0603)
//!
//! [`HttpRequest::tenant_id`] / [`HttpRequest::principal_id`] are CALLER-SUPPLIED
//! body fields. They are **non-authoritative** — they grant nothing and are
//! never the source of truth for who the caller is or which tenant they act on.
//! Every mutating route MUST go through [`HttpHandler::handle`], which runs the
//! [`crate::authz`] fail-closed gate FIRST (verify credential → 401, PDP authorize
//! against the VERIFIED tenant → 403) before any business logic. The gate takes
//! no request body; the resource scope is bound from the VERIFIED tenant in the
//! returned `AuthorizedCrmContext`, so the body `tenant_id` is structurally
//! ignored — never an authz input, never the resource tenant.
//!
//! ## Edge obligation (this crate is dead-until-edge)
//!
//! The business logic still returns `contract_stub` — there is no bound socket
//! yet. The edge that binds a real listener MUST, before the gate runs:
//!   * extract the bearer/SVID credential in transport middleware
//!     (`route_layer` / `FromRequestParts`) BEFORE body deserialization,
//!   * install `DefaultBodyLimit`, and
//!   * refuse to boot without a [`crate::authz::CrmAuthzProvider`] configured.

use crate::authz::{authorize_crm_command, AuthorizedCrmContext, CallerCredential, CrmAction, CrmAuthzProvider};
use crate::domain::Capability;
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HttpRoute { pub method: &'static str, pub path: &'static str, pub capability: &'static str, pub idempotent: bool }
/// REST request DTO. NOTE: `tenant_id` and `principal_id` are non-authoritative
/// caller-supplied fields (see module docs / ADR-0603). They are structurally
/// never read by the gate; they never authorize and never select the resource
/// tenant (the verified tenant is the sole scope).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HttpRequest { pub tenant_id: String, pub principal_id: String, pub request_id: String, pub idempotency_key: String, pub body: serde_json::Value }
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HttpResponse { pub status: u16, pub body: serde_json::Value }

pub struct HttpHandler;
impl HttpHandler {
    pub fn routes() -> Vec<HttpRoute> {
        vec![
            HttpRoute { method: "POST", path: "/v1/crm/account-masters:sync", capability: "account-master", idempotent: true },
            HttpRoute { method: "POST", path: "/v1/crm/opportunities:advance", capability: "opportunity", idempotent: true },
            HttpRoute { method: "POST", path: "/v1/crm/quotes:approve", capability: "quote", idempotent: true },
            HttpRoute { method: "POST", path: "/v1/crm/campaigns:launch", capability: "campaign", idempotent: true },
            HttpRoute { method: "POST", path: "/v1/crm/service-cases:route", capability: "service-case", idempotent: true },
        ]
    }

    /// Handle a mutating CRM request through the fail-closed authz gate.
    ///
    /// `credential` is supplied by the transport (bearer/SVID), NOT the body.
    /// `capability` is the matched-route metadata, NOT the body. The gate
    /// verifies the caller and authorizes the action against the VERIFIED tenant
    /// before any business logic. On gate failure this returns a
    /// distinct-kind [`ServiceError`] (`Unauthenticated`/401 or `Forbidden`/403).
    ///
    /// The resource scope is bound by [`Self::resolve_scope`] from the VERIFIED
    /// tenant in the returned [`AuthorizedCrmContext`], NEVER from
    /// `request.tenant_id`/`request.principal_id`. A forged body tenant is
    /// structurally ignored — it can be neither an authz input nor the resource
    /// tenant.
    ///
    /// # Errors
    /// `Unauthenticated`/`Forbidden` on a failed gate; `ContractStub` once
    /// authorized (the business handler is intentionally scaffolded until the
    /// impl packet lands).
    pub fn handle(provider: &CrmAuthzProvider, credential: &CallerCredential, capability: Capability, request: HttpRequest) -> Result<HttpResponse> {
        let scope = Self::resolve_scope(provider, credential, capability, &request)?;
        // The resource tenant is the VERIFIED tenant from the gate — the forged
        // body tenant (`request.tenant_id`) is never honored. Business logic
        // (scaffolded today) MUST use `scope.tenant_id()` as the resource tenant.
        let _ = scope.tenant_id();
        Err(ServiceError::contract_stub("http"))
    }

    /// Run the fail-closed gate and return the scope the handler MUST act within.
    /// The returned context's tenant is the VERIFIED tenant; `request.tenant_id`
    /// is structurally discarded so a forged body tenant can never select the
    /// resource. Exposed so the cross-tenant invariant is directly assertable.
    ///
    /// # Errors
    /// `Unauthenticated`/`Forbidden` on a failed gate.
    pub fn resolve_scope(provider: &CrmAuthzProvider, credential: &CallerCredential, capability: Capability, _request: &HttpRequest) -> Result<AuthorizedCrmContext> {
        authorize_crm_command(provider, credential, CrmAction(capability)).map_err(ServiceError::from)
    }
}

pub fn validate_routes(routes: &[HttpRoute]) -> Result<()> {
    if routes.len() < 5 { return Err(ServiceError::validation("http_routes", "scaffold requires at least five REST routes")); }
    if routes.iter().any(|route| !route.path.starts_with("/v1/")) { return Err(ServiceError::validation("http_routes", "all REST routes must be versioned under /v1")); }
    Ok(())
}
