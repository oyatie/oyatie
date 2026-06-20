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
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use tracing::field;

use iam_policy_cedar_domain::PolicySet;

use crate::{
    CedarPolicyApiAuthorization, CedarPolicyApiBoundaryContext, CedarPolicyApiPrincipal,
    CedarPolicyPublishApiError, CedarPolicyPublishApiRequest, CedarPolicyPublishIdempotencyLedger,
    CedarPolicyPublishRequest, CedarPolicyRecord, CedarPolicyPublishMetadata,
    CedarPolicyPublishSuccessResponse, CedarPolicyRequiredAttribute, CedarPolicyRuleRef,
    CedarPolicyScopeRef, publish_cedar_policy_from_api,
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
pub struct CedarPolicyRestState {
    policies: Mutex<PolicySet>,                                      // data_class: INTERNAL_ONLY
    idempotency: Mutex<CedarPolicyPublishIdempotencyLedger>,         // data_class: INTERNAL_ONLY
}

impl CedarPolicyRestState {
    /// Construct state from an existing policy set and ledger.
    #[must_use]
    pub fn new(policies: PolicySet, idempotency: CedarPolicyPublishIdempotencyLedger) -> Self {
        Self {
            policies: Mutex::new(policies),
            idempotency: Mutex::new(idempotency),
        }
    }
}

impl Default for CedarPolicyRestState {
    fn default() -> Self {
        Self::new(PolicySet::default(), CedarPolicyPublishIdempotencyLedger::default())
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
    pub effect: String,          // data_class: INTERNAL_ONLY
    pub principal_role: String,  // data_class: INTERNAL_ONLY
    pub action: String,          // data_class: INTERNAL_ONLY
    pub resource_prefix: String, // data_class: INTERNAL_ONLY
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
    pub policy_id: String,           // data_class: INTERNAL_ONLY
    pub version: String,             // data_class: INTERNAL_ONLY
    pub scope: PolicyScopeDto,       // data_class: INTERNAL_ONLY
    pub supersedes: Option<String>,  // data_class: INTERNAL_ONLY
    pub rules: Vec<PolicyRuleDto>,   // data_class: INTERNAL_ONLY
}

/// JSON success response body for `201 Created`.
#[derive(Clone, Debug, Serialize)]
pub struct PublishSuccessResponse {
    pub data: PolicyRecordDto,       // data_class: INTERNAL_ONLY
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
    pub code: String,                          // data_class: INTERNAL_ONLY
    pub message: String,                       // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,     // data_class: INTERNAL_ONLY
    pub request_id: String,                    // data_class: INTERNAL_ONLY
    pub details: Vec<ErrorDetailDto>,          // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,      // data_class: INTERNAL_ONLY
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
/// ```rust
/// use std::sync::Arc;
/// use iam_policy_cedar_api::rest::{CedarPolicyRestState, build_router};
///
/// let state = Arc::new(CedarPolicyRestState::default());
/// let _router = build_router(state);
/// ```
pub fn build_router(state: SharedCedarPolicyRestState) -> Router {
    Router::new()
        .route(PUBLISH_ROUTE, post(publish_handler))
        .with_state(state)
}

// ==========================================================================
// Handler
// ==========================================================================

/// `POST /policies/{policy_id}/versions/{version}`
///
/// Extracts boundary / principal / authorization context from headers, builds
/// the typed [`CedarPolicyPublishApiRequest`], delegates to
/// [`publish_cedar_policy_from_api`], and maps the outcome to an HTTP response.
/// A [`tracing`] span is entered for every call so OTel instrumentation can
/// pick up `cedar.policy.publish.status_code` and companion attributes.
async fn publish_handler(
    State(state): State<SharedCedarPolicyRestState>,
    Path((path_policy_id, path_version)): Path<(String, String)>,
    headers: HeaderMap,
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

    // Extract principal headers.
    let principal_tenant_id = header_str(&headers, "x-principal-tenant-id");
    let principal_id = header_str(&headers, "x-principal-id");

    // Extract authorization headers.
    let authz_decision_id = header_str(&headers, "x-authorization-decision-id");
    let authz_tenant_id = header_str(&headers, "x-authorization-tenant-id");
    let authz_principal_id = header_str(&headers, "x-authorization-principal-id");
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
            tenant_id: principal_tenant_id,
            principal_id,
        },
        authorization: CedarPolicyApiAuthorization {
            tenant_id: authz_tenant_id,
            principal_id: authz_principal_id,
            decision_id: authz_decision_id,
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
        &mut *policies_guard,
        &mut *idempotency_guard,
        api_request,
    ) {
        Ok(success) => {
            tracing::Span::current().record("cedar.policy.publish.status_code", 201u16);
            tracing::Span::current().record("cedar.policy.publish.idempotent_replay", false);
            (
                StatusCode::CREATED,
                Json(success_to_dto(success)),
            )
                .into_response()
        }
        Err(err) => {
            let status_code = err.cedar_policy_publish_status_code();
            tracing::Span::current().record("cedar.policy.publish.status_code", status_code);
            let error_response = err.error_response(request_id);
            let http_status = StatusCode::from_u16(status_code)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (
                http_status,
                Json(error_response_to_dto(error_response)),
            )
                .into_response()
        }
    }
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
        required_attribute: dto.required_attribute.map(|a| CedarPolicyRequiredAttribute {
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
fn error_response_to_dto(
    resp: crate::CedarPolicyPublishApiErrorResponse,
) -> ErrorResponseDto {
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
