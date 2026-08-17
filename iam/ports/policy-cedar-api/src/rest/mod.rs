//! Control-plane REST edge for `cedar.policy.publish` (ADR-0090 amendment).
//!
//! Mounts `POST /policies/{policy_id}/versions/{version}` and wires it to the
//! existing typed boundary fns ([`validate_cedar_policy_publish_request`] /
//! [`publish_cedar_policy_from_api`]) with idempotency-ledger state, complete
//! `CedarPolicyPublishApiError` → HTTP status mapping, and OTel spans emitted
//! via [`tracing`].
//!
//! ## Route
//!
//! | Method + path                                     | Handler                  |
//! |---------------------------------------------------|--------------------------|
//! | `POST /policies/{policy_id}/versions/{version}`  | [`publish_handler`]      |
//!
//! ## Header contract
//!
//! All boundary context arrives in headers so the route is stateless w.r.t.
//! caller identity:
//!
//! | Header                         | Maps to                             |
//! |--------------------------------|-------------------------------------|
//! | `X-Request-Id`                 | `boundary.request_id`               |
//! | `X-Tenant-Id`                  | `boundary.tenant_id`                |
//! | `Idempotency-Key`              | `boundary.idempotency_key`          |
//! | `X-Principal-Tenant-Id`        | `principal.tenant_id`               |
//! | `X-Principal-Id`               | `principal.principal_id`            |
//! | `X-Authorization-Decision-Id`  | `authorization.decision_id`         |
//! | `X-Authorization-Tenant-Id`    | `authorization.tenant_id`           |
//! | `X-Authorization-Principal-Id` | `authorization.principal_id`        |
//! | `X-Authorization-Surfaces`     | `authorization.allowed_surfaces`    |
//!
//! ## Layering invariant (ADR-0131 / ADR-0509)
//!
//! This module is the delivery layer; it holds NO policy/domain logic.  All
//! validation and publication is delegated inward to the boundary fns in
//! `src/lib.rs`.

// ADR-0083 Tier 3: production code stays panic-free; tests may use
// unwrap/expect/panic under the cfg(test) exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use tracing::field;

use iam_policy_cedar_domain::PolicySet;

use crate::authz::{
    CallerCredential, CedarPolicyAuthzProvider, PrincipalVerificationError,
    PublishAuthorizationError, PublishResource, PublishScope, VerifiedPrincipal,
};
use crate::{
    CedarPolicyApiAuthorization, CedarPolicyApiBoundaryContext, CedarPolicyApiPrincipal,
    CedarPolicyPublishApiRequest, CedarPolicyPublishIdempotencyLedger, CedarPolicyPublishMetadata,
    CedarPolicyPublishRequest, CedarPolicyPublishSuccessResponse, CedarPolicyRecord,
    CedarPolicyRequiredAttribute, CedarPolicyRuleRef, CedarPolicyScopeRef,
    publish_cedar_policy_from_api,
};

// ==========================================================================
// Route constants
// ==========================================================================

/// `POST` — publish a Cedar policy version.
pub const PUBLISH_ROUTE: &str = "/policies/{policy_id}/versions/{version}";

// ==========================================================================
// Shared router state
// ==========================================================================

/// Shared state behind the axum router.
///
/// Both `policies` and `idempotency` are mutex-guarded because
/// [`publish_cedar_policy_from_api`] mutates them.  The `Arc` allows the
/// router to be cloned cheaply across Tokio worker threads.
///
/// `authz` is a REQUIRED, non-optional [`CedarPolicyAuthzProvider`]: there is no
/// constructor that yields state without it, so a router can NEVER be built
/// without a configured principal-verification + PDP authorization seam (no
/// default-allow fallback — AUTH-005 fail-closed boot doctrine; task #124 /
/// ADR-0572).
pub struct CedarPolicyRestState {
    policies: Mutex<PolicySet>, // data_class: INTERNAL_ONLY
    idempotency: Mutex<CedarPolicyPublishIdempotencyLedger>, // data_class: INTERNAL_ONLY
    authz: CedarPolicyAuthzProvider, // data_class: INTERNAL_ONLY
}

impl CedarPolicyRestState {
    /// Construct state from an existing policy set, ledger, and the REQUIRED
    /// authz provider. The provider is non-optional by type so the binary/router
    /// refuses to serve without a configured authz seam.
    #[must_use]
    pub fn new(
        policies: PolicySet,
        idempotency: CedarPolicyPublishIdempotencyLedger,
        authz: CedarPolicyAuthzProvider,
    ) -> Self {
        Self {
            policies: Mutex::new(policies),
            idempotency: Mutex::new(idempotency),
            authz,
        }
    }

    /// Construct state with an empty policy set + ledger and the REQUIRED authz
    /// provider. Convenience for embedding applications and tests.
    #[must_use]
    pub fn with_authz(authz: CedarPolicyAuthzProvider) -> Self {
        Self::new(
            PolicySet::default(),
            CedarPolicyPublishIdempotencyLedger::default(),
            authz,
        )
    }
}

/// Shared-state handle passed to axum via `.with_state(…)`.
pub type SharedCedarPolicyRestState = Arc<CedarPolicyRestState>;

// ==========================================================================
// JSON DTOs
// ==========================================================================

/// JSON DTO for the optional `required_attribute` field inside a rule.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequiredAttributeDto {
    pub key: String,   // data_class: INTERNAL_ONLY
    pub value: String, // data_class: INTERNAL_ONLY
}

/// JSON DTO for a single policy rule in the request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PolicyRuleDto {
    pub effect: String,                                   // data_class: INTERNAL_ONLY
    pub principal_role: String,                           // data_class: INTERNAL_ONLY
    pub action: String,                                   // data_class: INTERNAL_ONLY
    pub resource_prefix: String,                          // data_class: INTERNAL_ONLY
    pub required_attribute: Option<RequiredAttributeDto>, // data_class: INTERNAL_ONLY
}

/// JSON DTO for the policy scope inside the request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PolicyScopeDto {
    pub kind: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: Option<String>, // data_class: INTERNAL_ONLY
}

/// JSON request body for `POST /policies/{policy_id}/versions/{version}`.
#[derive(Clone, Debug, Deserialize)]
pub struct PublishRequestBody {
    pub policy_id: String,          // data_class: INTERNAL_ONLY
    pub version: String,            // data_class: INTERNAL_ONLY
    pub scope: PolicyScopeDto,      // data_class: INTERNAL_ONLY
    pub supersedes: Option<String>, // data_class: INTERNAL_ONLY
    pub rules: Vec<PolicyRuleDto>,  // data_class: INTERNAL_ONLY
}

/// JSON success response body for `201 Created`.
#[derive(Clone, Debug, Serialize)]
pub struct PublishSuccessResponse {
    pub data: PolicyRecordDto,        // data_class: INTERNAL_ONLY
    pub metadata: PublishMetadataDto, // data_class: INTERNAL_ONLY
}

/// JSON DTO for the policy record inside the success response.
#[derive(Clone, Debug, Serialize)]
pub struct PolicyRecordDto {
    pub policy_id: String,          // data_class: INTERNAL_ONLY
    pub version: String,            // data_class: INTERNAL_ONLY
    pub scope: PolicyScopeDto,      // data_class: INTERNAL_ONLY
    pub supersedes: Option<String>, // data_class: INTERNAL_ONLY
    pub rules: Vec<PolicyRuleDto>,  // data_class: INTERNAL_ONLY
    pub schema_version: u32,        // data_class: PUBLIC
}

/// JSON DTO for the metadata inside the success response.
#[derive(Clone, Debug, Serialize)]
pub struct PublishMetadataDto {
    pub request_id: String,         // data_class: INTERNAL_ONLY
    pub operator_tenant_id: String, // data_class: INTERNAL_ONLY
    pub principal_id: String,       // data_class: INTERNAL_ONLY
}

/// JSON error body for 4xx responses.
#[derive(Clone, Debug, Serialize)]
pub struct ErrorDetailDto {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

/// JSON error envelope for 4xx responses.
#[derive(Clone, Debug, Serialize)]
pub struct ErrorBodyDto {
    pub code: String,                      // data_class: INTERNAL_ONLY
    pub message: String,                   // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>, // data_class: INTERNAL_ONLY
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub details: Vec<ErrorDetailDto>,      // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,  // data_class: INTERNAL_ONLY
}

/// Wrapper around the error body (mirrors the typed `CedarPolicyPublishApiErrorResponse`).
#[derive(Clone, Debug, Serialize)]
pub struct ErrorResponseDto {
    pub error: ErrorBodyDto, // data_class: INTERNAL_ONLY
}

// ==========================================================================
// Router builder
// ==========================================================================

/// Build the Cedar policy publish control-plane REST router.
///
/// The router is fail-closed by construction: [`SharedCedarPolicyRestState`]
/// carries a REQUIRED [`crate::authz::CedarPolicyAuthzProvider`], so there is no
/// way to mount this control plane without a configured principal-verification +
/// PDP authorization seam (no default-allow fallback — task #124 / ADR-0572).
///
/// ```rust
/// use std::sync::Arc;
/// use iam_policy_cedar_api::authz::{
///     CedarPolicyAuthzProvider, ConfiguredBearerPrincipalVerifier, PublishAuthorizer,
///     PublishAuthorizationError, PublishResource, VerifiedPrincipal,
/// };
/// use iam_policy_cedar_api::rest::{CedarPolicyRestState, build_router};
///
/// struct DenyAll;
/// impl PublishAuthorizer for DenyAll {
///     fn ensure_authorized(
///         &self,
///         _p: &VerifiedPrincipal,
///         _r: &PublishResource,
///     ) -> Result<(), PublishAuthorizationError> {
///         Err(PublishAuthorizationError::Denied)
///     }
/// }
///
/// let verifier = Arc::new(
///     ConfiguredBearerPrincipalVerifier::new("secret", "usr_admin", "ten_platform").unwrap(),
/// );
/// let authz = CedarPolicyAuthzProvider::new(verifier, Arc::new(DenyAll));
/// let state = Arc::new(CedarPolicyRestState::with_authz(authz));
/// let _router = build_router(state);
/// ```
/// Maximum request body size for this control plane (64 KiB). A Cedar policy
/// publish request is small structured JSON; a large body is either malformed
/// or a DoS probe. Explicit limit ensures the body limit is not inherited from
/// a permissive host embedding.
const PUBLISH_BODY_LIMIT_BYTES: usize = 65_536;

pub fn build_router(state: SharedCedarPolicyRestState) -> Router {
    Router::new()
        .route(PUBLISH_ROUTE, post(publish_handler))
        // Bearer verification middleware runs on request Parts BEFORE the body
        // is deserialized by the handler's Json extractor. This satisfies the
        // "gate FIRST / before acting on the body" invariant: unauthenticated
        // callers are rejected with 401 before any body bytes are consumed.
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            bearer_auth_middleware,
        ))
        // Explicit tight body limit for this control plane (64 KiB). Do not
        // rely on the host's DefaultBodyLimit; set it explicitly so the limit
        // is enforced regardless of how the router is embedded.
        .layer(axum::extract::DefaultBodyLimit::max(
            PUBLISH_BODY_LIMIT_BYTES,
        ))
        .with_state(state)
}

// ==========================================================================
// Bearer verification middleware (runs BEFORE body deserialization)
// ==========================================================================

/// Axum middleware that verifies the `Authorization: Bearer` credential and
/// stores the resulting [`VerifiedPrincipal`] in the request extensions BEFORE
/// the body is deserialized by the downstream handler.
///
/// This satisfies the "gate FIRST / before acting on the body" invariant:
/// unauthenticated callers (no credential, wrong token) are rejected with 401
/// before any JSON parse occurs on the request body — `Json(body)` in the
/// handler only runs when this middleware returns `next.run(req)`.
async fn bearer_auth_middleware(
    State(state): State<SharedCedarPolicyRestState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let headers = req.headers();
    let request_id = header_str(headers, "x-request-id");
    let credential = CallerCredential {
        authorization: header_opt(headers, "authorization"),
        claimed_principal_id: header_str(headers, "x-principal-id"),
        claimed_tenant_id: header_str(headers, "x-principal-tenant-id"),
    };

    let verified = match state.authz.verify_principal(&credential) {
        Ok(v) => v,
        Err(err) => return principal_verification_response(&request_id, &err),
    };

    // Stash the verified principal in request extensions so the handler can
    // retrieve it without re-verifying (the credential is already consumed).
    req.extensions_mut().insert(verified);
    next.run(req).await
}

// ==========================================================================
// Handler
// ==========================================================================

/// `POST /policies/{policy_id}/versions/{version}`
///
/// The [`bearer_auth_middleware`] runs BEFORE this handler and rejects
/// unauthenticated callers (no/wrong `Authorization: Bearer`) with 401 before
/// the body is deserialized. The verified principal arrives via request
/// extensions.
///
/// This handler then:
/// 1. Extracts the [`VerifiedPrincipal`] inserted by the middleware (panic if
///    absent — the middleware MUST run, enforced by [`build_router`]).
/// 2. Runs the cross-tenant guard and PDP authorization (403 on denial).
/// 3. Builds the typed [`CedarPolicyPublishApiRequest`] and delegates to
///    [`publish_cedar_policy_from_api`], which ALSO requires a
///    [`VerifiedPrincipal`] so in-process callers cannot bypass the gate.
/// 4. Maps the outcome to an HTTP response with an OTel span.
async fn publish_handler(
    State(state): State<SharedCedarPolicyRestState>,
    Path((path_policy_id, path_version)): Path<(String, String)>,
    headers: HeaderMap,
    axum::extract::Extension(verified): axum::extract::Extension<VerifiedPrincipal>,
    Json(body): Json<PublishRequestBody>,
) -> Response {
    let span = tracing::info_span!(
        "cedar.policy.publish",
        cedar.policy.publish.policy_id = path_policy_id.as_str(),
        cedar.policy.publish.version = path_version.as_str(),
        cedar.policy.publish.status_code = field::Empty,
        cedar.policy.publish.idempotent_replay = field::Empty,
    );
    let _enter = span.enter();

    // Extract boundary headers.
    let request_id = header_str(&headers, "x-request-id");
    let tenant_id = header_str(&headers, "x-tenant-id");
    let idempotency_key = header_str(&headers, "idempotency-key");

    // ── Cross-tenant guard + PDP authorization ────────────────────────────────
    // The bearer middleware already verified the principal. Here we assert the
    // tenant axis and run the PDP decision. The self-attested x-principal-*
    // headers are CROSS-CHECK inputs only — the verified identity is authoritative.
    if let Err(response) = enforce_publish_authz(
        &state,
        &headers,
        &request_id,
        &path_policy_id,
        &tenant_id,
        &verified,
        &body,
    ) {
        return response;
    }

    // Use the VERIFIED identity for principal binding and audit fields.
    // The caller-supplied x-authorization-principal-id / x-authorization-tenant-id
    // headers are NOT recorded as authoritative audit evidence: a caller can forge
    // them. The authoritative principal and tenant always come from the verified
    // credential (set by bearer_auth_middleware before any body bytes are read).
    let principal_id = verified.principal_id().to_string();
    let principal_tenant_id = verified.tenant_id().to_string();

    // x-authorization-decision-id is a CALLER-SUPPLIED correlation id, NOT an
    // authorization grant. It is recorded as a correlation hint for log joins; the
    // real authorization decision was made by the PublishAuthorizer PDP port above.
    // FUTURE: when the PublishAuthorizer port is extended to return a decision
    // record (fast-follow), replace this with the server-derived decision id from
    // that record so the audit trail is fully authoritative end-to-end.
    let authz_correlation_id = header_str(&headers, "x-authorization-decision-id");
    let authz_surfaces = header_csv(&headers, "x-authorization-surfaces");

    let api_request = CedarPolicyPublishApiRequest {
        path_policy_id,
        path_version,
        boundary: CedarPolicyApiBoundaryContext {
            request_id: request_id.clone(),
            tenant_id,
            idempotency_key,
        },
        principal: CedarPolicyApiPrincipal {
            tenant_id: principal_tenant_id.clone(),
            principal_id: principal_id.clone(),
        },
        authorization: CedarPolicyApiAuthorization {
            // Derive tenant and principal from the verified identity, not headers.
            tenant_id: principal_tenant_id,
            principal_id,
            // Caller-supplied correlation id (see note above).
            decision_id: authz_correlation_id,
            allowed_surfaces: authz_surfaces,
        },
        body: dto_to_request_body(body),
    };

    // Lock the shared state; a poisoned mutex is recovered (fail-safe).
    let mut policies_guard = match state.policies.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut idempotency_guard = match state.idempotency.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    match publish_cedar_policy_from_api(
        &verified,
        &mut *policies_guard,
        &mut *idempotency_guard,
        api_request,
    ) {
        Ok(success) => {
            tracing::Span::current().record("cedar.policy.publish.status_code", 201u16);
            tracing::Span::current().record("cedar.policy.publish.idempotent_replay", false);
            (StatusCode::CREATED, Json(success_to_dto(success))).into_response()
        }
        Err(err) => {
            let status_code = err.cedar_policy_publish_status_code();
            tracing::Span::current().record("cedar.policy.publish.status_code", status_code);
            let error_response = err.error_response(request_id);
            let http_status =
                StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (http_status, Json(error_response_to_dto(error_response))).into_response()
        }
    }
}

// ==========================================================================
// Fail-closed authz gate (task #124 / ADR-0572) — router layer only
// ==========================================================================

/// FAIL-CLOSED authz enforcement for the publish surface (cross-tenant guard +
/// PDP decision). Bearer verification has already been done in
/// [`bearer_auth_middleware`] before the body was deserialized; the verified
/// principal arrives as a parameter.
///
/// 1. CROSS-CHECK the operator (`x-tenant-id`) and the self-attested principal
///    tenant against the VERIFIED tenant — a verified principal of tenant A may
///    not operate as tenant B (cross-tenant guard) → 403 on mismatch.
/// 2. AUTHORIZE the verified principal for `cedar.policy.publish` on the
///    target `{policy_id, scope, resource_tenant}` via the
///    [`crate::authz::PublishAuthorizer`] PDP port → 403 on deny/refusal.
///    The scope is passed EXPLICITLY so the PDP sees the true blast radius:
///    a global policy affects ALL tenants and must NOT be presented as a
///    tenant-scoped resource owned by the caller (which would silently
///    authorize tenant-admins for platform-wide authz-policy control).
///
/// Returns `Ok(())` on success, or `Err(Response)` (403) on any failure.
#[allow(clippy::too_many_arguments)]
fn enforce_publish_authz(
    state: &CedarPolicyRestState,
    headers: &HeaderMap,
    request_id: &str,
    path_policy_id: &str,
    operator_tenant_id: &str,
    verified: &VerifiedPrincipal,
    body: &PublishRequestBody,
) -> Result<(), Response> {
    // (1) Cross-check — the verified identity is authoritative.
    //
    // Header contract for this control plane:
    // - `x-principal-id` is REQUIRED and MUST match the verified principal id.
    //   Absent or empty → 403 (not 401: the bearer credential verified; the
    //   caller simply failed to assert an identity that can be cross-checked).
    //   Non-empty but mismatched → 403 (substitution attempt).
    // - `x-tenant-id` (operator tenant) and `x-principal-tenant-id` are checked
    //   when non-empty; if absent the verified tenant is authoritative and the
    //   PDP decision enforces the tenant axis.
    let claimed_principal_id = header_str(headers, "x-principal-id");
    let claimed_principal_tenant_id = header_str(headers, "x-principal-tenant-id");

    // x-principal-id is required: absent/empty means the caller did not assert
    // an identity, which this control plane does not allow.
    if claimed_principal_id.is_empty() {
        return Err(authorization_denied_response(
            request_id,
            "principal_id_missing",
        ));
    }
    if claimed_principal_id != verified.principal_id() {
        return Err(authorization_denied_response(request_id, "principal_id"));
    }
    if !operator_tenant_id.is_empty() && operator_tenant_id != verified.tenant_id() {
        return Err(authorization_denied_response(request_id, "operator_tenant"));
    }
    if !claimed_principal_tenant_id.is_empty()
        && claimed_principal_tenant_id != verified.tenant_id()
    {
        return Err(authorization_denied_response(
            request_id,
            "principal_tenant",
        ));
    }

    // (2) Reject unrecognised scope kinds BEFORE the PDP so the domain
    // validation layer can return 400 (bad request) rather than silently
    // mapping to Global and letting the PDP return 403.  The router layer only
    // checks that the kind is one of the known values; the domain layer in
    // `validate_cedar_policy_publish_request` → `parse_policy_scope` will
    // re-validate and emit the typed error downstream.
    match body.scope.kind.as_str() {
        "tenant" | "global" => {}
        _ => {
            return Err(invalid_scope_kind_response(request_id, &body.scope.kind));
        }
    }

    // (3) Authorize for cedar.policy.publish on {policy_id, scope, resource_tenant}.
    // The scope is explicit in the resource so the PDP sees the true blast radius.
    // A global-scoped policy applies to ALL tenants; the PDP must require
    // platform-admin authority for it, not mere tenant-admin authority.
    // Flattening global → caller's own tenant would be a CRITICAL escalation:
    // it lets a tenant-A admin publish a policy that affects every tenant.
    let resource = publish_resource(request_id, path_policy_id, body)?;
    state
        .authz
        .ensure_authorized(verified, &resource)
        .map_err(|err| publish_authorization_response(request_id, &err))?;

    Ok(())
}

/// Build the [`PublishResource`] for the PDP decision, carrying the scope
/// **explicitly** so the PDP sees the true blast radius of the action.
///
/// - `scope.kind == "tenant"`: resource is `Tenant`-scoped; `tenant_id` MUST be
///   present and non-empty (returns `Err(400)` otherwise — defaulting to the
///   verified tenant would hide a caller mistake and present the wrong resource
///   to the PDP). Cross-tenant publish — principal A, scope tenant B — is
///   denied by the PDP decision.
/// - `scope.kind == "global"` (or any other value): resource is `Global`;
///   `tenant_id` is empty. The PDP **must** key on `scope == Global` and
///   require platform-admin authority. A global policy affects ALL tenants and
///   MUST NOT be presented as a per-tenant resource (that would be the
///   CRITICAL escalation: tenant-A admin → platform-wide authz-policy control).
fn publish_resource(
    request_id: &str,
    policy_id: &str,
    body: &PublishRequestBody,
) -> Result<PublishResource, Response> {
    match body.scope.kind.as_str() {
        "tenant" => {
            let tenant_id = body
                .scope
                .tenant_id
                .as_deref()
                .filter(|t| !t.trim().is_empty())
                .ok_or_else(|| missing_scope_tenant_response(request_id))?;
            Ok(PublishResource {
                policy_id: policy_id.to_string(),
                scope: PublishScope::Tenant,
                tenant_id: tenant_id.to_string(),
            })
        }
        _ => {
            // Global (and any unrecognised variant) — blank tenant, Global scope.
            // The PDP must NOT authorize this as a per-tenant resource.
            Ok(PublishResource {
                policy_id: policy_id.to_string(),
                scope: PublishScope::Global,
                tenant_id: String::new(),
            })
        }
    }
}

/// Map a [`PrincipalVerificationError`] to an HTTP 401 response (fail-closed).
fn principal_verification_response(
    request_id: &str,
    _err: &PrincipalVerificationError,
) -> Response {
    tracing::Span::current().record("cedar.policy.publish.status_code", 401u16);
    let body = ErrorResponseDto {
        error: ErrorBodyDto {
            code: "CEDAR_POLICY_PRINCIPAL_UNVERIFIED".to_string(),
            message: "A verified caller principal is required to publish a Cedar policy"
                .to_string(),
            message_localized: None,
            request_id: request_id.to_string(),
            details: vec![ErrorDetailDto {
                field: "header.Authorization".to_string(),
                issue: "must present a verifiable caller credential".to_string(),
            }],
            retry_after_seconds: None,
        },
    };
    (StatusCode::UNAUTHORIZED, Json(body)).into_response()
}

/// Map a [`PublishAuthorizationError`] to an HTTP 403 response (fail-closed).
fn publish_authorization_response(request_id: &str, _err: &PublishAuthorizationError) -> Response {
    authorization_denied_response(request_id, "pdp_decision")
}

/// Build an HTTP 403 response for an authorization denial (fail-closed). The
/// `axis` identifies which check denied without leaking decision internals.
fn authorization_denied_response(request_id: &str, axis: &str) -> Response {
    tracing::Span::current().record("cedar.policy.publish.status_code", 403u16);
    let body = ErrorResponseDto {
        error: ErrorBodyDto {
            code: "CEDAR_POLICY_PUBLISH_FORBIDDEN".to_string(),
            message: "The verified principal is not authorized to publish this Cedar policy"
                .to_string(),
            message_localized: None,
            request_id: request_id.to_string(),
            details: vec![ErrorDetailDto {
                field: "authorization".to_string(),
                issue: format!("denied on the {axis} axis"),
            }],
            retry_after_seconds: None,
        },
    };
    (StatusCode::FORBIDDEN, Json(body)).into_response()
}

/// Build an HTTP 400 response for a tenant-scoped publish with no `tenant_id`.
/// Defaulting to the verified tenant would hide a caller mistake and present
/// the wrong resource to the PDP; reject explicitly as a bad request instead.
fn missing_scope_tenant_response(request_id: &str) -> Response {
    tracing::Span::current().record("cedar.policy.publish.status_code", 400u16);
    let body = ErrorResponseDto {
        error: ErrorBodyDto {
            code: "CEDAR_POLICY_SCOPE_TENANT_MISSING".to_string(),
            message: "Tenant-scoped policies require scope.tenant_id".to_string(),
            message_localized: None,
            request_id: request_id.to_string(),
            details: vec![ErrorDetailDto {
                field: "body.scope.tenant_id".to_string(),
                issue: "must be present and non-empty for tenant scope".to_string(),
            }],
            retry_after_seconds: None,
        },
    };
    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

/// Build an HTTP 400 response for an unrecognised scope kind. The router layer
/// rejects unrecognised scope kinds BEFORE the PDP decision so the caller sees
/// a typed 400 (bad request) rather than a 403 from the PDP.
fn invalid_scope_kind_response(request_id: &str, kind: &str) -> Response {
    tracing::Span::current().record("cedar.policy.publish.status_code", 400u16);
    let body = ErrorResponseDto {
        error: ErrorBodyDto {
            code: "CEDAR_POLICY_SCOPE_KIND_INVALID".to_string(),
            message: "Policy scope kind must be global or tenant".to_string(),
            message_localized: None,
            request_id: request_id.to_string(),
            details: vec![ErrorDetailDto {
                field: "body.scope.kind".to_string(),
                issue: format!("unrecognised scope kind: {kind:?}"),
            }],
            retry_after_seconds: None,
        },
    };
    (StatusCode::BAD_REQUEST, Json(body)).into_response()
}

// ==========================================================================
// Conversion helpers (router layer only — no domain logic)
// ==========================================================================

/// Extract a header value as a `String`; returns empty string when absent.
fn header_str(headers: &HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

/// Extract a header value as `Option<String>`; `None` when absent or non-UTF-8
/// (so a missing `Authorization` credential is `None`, not an empty string).
fn header_opt(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract a comma-separated header value as `Vec<String>`; returns empty vec when absent.
fn header_csv(headers: &HeaderMap, name: &str) -> Vec<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').map(|part| part.trim().to_string()).collect())
        .unwrap_or_default()
}

/// Convert the JSON request body DTO to the typed API boundary struct.
fn dto_to_request_body(dto: PublishRequestBody) -> CedarPolicyPublishRequest {
    CedarPolicyPublishRequest {
        policy_id: dto.policy_id,
        version: dto.version,
        scope: CedarPolicyScopeRef {
            kind: dto.scope.kind,
            tenant_id: dto.scope.tenant_id,
        },
        supersedes: dto.supersedes,
        rules: dto.rules.into_iter().map(dto_to_rule_ref).collect(),
    }
}

/// Convert a rule DTO to [`CedarPolicyRuleRef`].
fn dto_to_rule_ref(dto: PolicyRuleDto) -> CedarPolicyRuleRef {
    CedarPolicyRuleRef {
        effect: dto.effect,
        principal_role: dto.principal_role,
        action: dto.action,
        resource_prefix: dto.resource_prefix,
        required_attribute: dto
            .required_attribute
            .map(|a| CedarPolicyRequiredAttribute {
                key: a.key,
                value: a.value,
            }),
    }
}

/// Convert a [`CedarPolicyPublishSuccessResponse`] to the JSON success DTO.
fn success_to_dto(success: CedarPolicyPublishSuccessResponse) -> PublishSuccessResponse {
    PublishSuccessResponse {
        data: record_to_dto(success.data),
        metadata: metadata_to_dto(success.metadata),
    }
}

/// Convert a [`CedarPolicyRecord`] to [`PolicyRecordDto`].
fn record_to_dto(record: CedarPolicyRecord) -> PolicyRecordDto {
    PolicyRecordDto {
        policy_id: record.policy_id,
        version: record.version,
        scope: PolicyScopeDto {
            kind: record.scope.kind,
            tenant_id: record.scope.tenant_id,
        },
        supersedes: record.supersedes,
        rules: record.rules.into_iter().map(rule_ref_to_dto).collect(),
        schema_version: record.schema_version,
    }
}

/// Convert a [`CedarPolicyPublishMetadata`] to [`PublishMetadataDto`].
fn metadata_to_dto(metadata: CedarPolicyPublishMetadata) -> PublishMetadataDto {
    PublishMetadataDto {
        request_id: metadata.request_id,
        operator_tenant_id: metadata.operator_tenant_id,
        principal_id: metadata.principal_id,
    }
}

/// Convert a [`CedarPolicyRuleRef`] to [`PolicyRuleDto`].
fn rule_ref_to_dto(rule: CedarPolicyRuleRef) -> PolicyRuleDto {
    PolicyRuleDto {
        effect: rule.effect,
        principal_role: rule.principal_role,
        action: rule.action,
        resource_prefix: rule.resource_prefix,
        required_attribute: rule.required_attribute.map(|a| RequiredAttributeDto {
            key: a.key,
            value: a.value,
        }),
    }
}

/// Convert a typed error response to the JSON error DTO.
fn error_response_to_dto(resp: crate::CedarPolicyPublishApiErrorResponse) -> ErrorResponseDto {
    ErrorResponseDto {
        error: ErrorBodyDto {
            code: resp.error.code,
            message: resp.error.message,
            message_localized: resp.error.message_localized,
            request_id: resp.error.request_id,
            details: resp
                .error
                .details
                .into_iter()
                .map(|d| ErrorDetailDto {
                    field: d.field,
                    issue: d.issue,
                })
                .collect(),
            retry_after_seconds: resp.error.retry_after_seconds,
        },
    }
}
