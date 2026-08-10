//! Workload-identity API DTO contract layer.
//!
//! Transport-neutral, serializable request/response shapes for the workload
//! authorize + token-validation + principal-lifecycle surface promised by
//! `iam/identity/workload-identity/PRD.md` §1.2. The DTOs convert
//! into (and out of) the pure [`iam_identity_workload_domain`] values; this
//! crate persists nothing, performs no crypto, evaluates no policy, and does no
//! I/O. The axum surface (`oya-identity-workload-rest`) and the use-case core
//! (`oya-identity-workload-app`) consume these shapes.
//!
//! ## Layering invariant (ADR-0131 / architecture-boundaries gate)
//!
//! This is the `api` ring: it depends inward on the `domain` crate only. The
//! decision/outcome DTOs are projected from the domain decision types so the
//! wire shape stays a stable, audit-legible contract independent of the policy
//! engine behind it.

// ADR-0083 Tier 3: production code stays panic-free; tests may use
// unwrap/expect/panic under the cfg(test) exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use iam_identity_workload_domain::{
    Action, AuthorizationDecision, ClaimValue, DecisionReason, Effect, Resource, WorkloadState,
};
use serde::{Deserialize, Serialize};

// =====================================================================
// Error envelope
// =====================================================================

/// Stable error envelope returned for every non-2xx response. Mirrors the flat
/// `oya-accounting-journal-api` envelope shape (code/message/details).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorBody, // data_class: INTERNAL_ONLY
}

/// Body of an [`ApiErrorEnvelope`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorBody {
    pub code: String,            // data_class: INTERNAL_ONLY
    pub message: String,         // data_class: INTERNAL_ONLY
    pub details: Option<String>, // data_class: INTERNAL_ONLY
}

impl ApiErrorEnvelope {
    /// Construct an envelope with a stable machine code + human message.
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            error: ApiErrorBody {
                code: code.into(),
                message: message.into(),
                details,
            },
        }
    }

    /// `TOKEN_INVALID` (HTTP 422) — the presented workload token failed
    /// validation; the policy engine was never consulted (PRD §3.4).
    #[must_use]
    pub fn token_invalid(details: Option<String>) -> Self {
        Self::new("TOKEN_INVALID", "workload token failed validation", details)
    }

    /// `UNAUTHENTICATED` (HTTP 401) — the request carried no verified caller
    /// credential. Used by the mutating lifecycle control plane (ADR-0581):
    /// a caller must present an unforgeable credential before any mutation; a
    /// self-attested header is never a credential.
    #[must_use]
    pub fn unauthorized(details: Option<String>) -> Self {
        Self::new(
            "UNAUTHENTICATED",
            "a verified caller credential is required",
            details,
        )
    }

    /// `FORBIDDEN` (HTTP 403) — the request authenticated but was denied by the
    /// authorization decision. Never a 404 (a deny must not leak existence).
    #[must_use]
    pub fn forbidden(details: Option<String>) -> Self {
        Self::new("FORBIDDEN", "authorization denied", details)
    }

    /// `DEPENDENCY_UNAVAILABLE` (HTTP 503) — a backing store or the JWKS was
    /// unavailable; the PEP treats this as a hard deny (fail-closed).
    #[must_use]
    pub fn dependency_unavailable(details: Option<String>) -> Self {
        Self::new(
            "DEPENDENCY_UNAVAILABLE",
            "a required dependency was unavailable; request denied (fail-closed)",
            details,
        )
    }

    /// `VALIDATION_ERROR` (HTTP 400) — the request body itself was malformed
    /// (bad id shape, empty field, ...).
    #[must_use]
    pub fn validation(message: impl Into<String>, details: Option<String>) -> Self {
        Self::new("VALIDATION_ERROR", message, details)
    }

    /// `NOT_FOUND` (HTTP 404) — a lifecycle target principal does not exist.
    /// Used ONLY by the control-plane lifecycle endpoints, never the authorize
    /// path (where a deny is a 403, never a 404).
    #[must_use]
    pub fn not_found(details: Option<String>) -> Self {
        Self::new("NOT_FOUND", "workload principal not found", details)
    }
}

// =====================================================================
// Shared sub-shapes
// =====================================================================

/// A typed claim value on the wire. Mirrors the domain [`ClaimValue`] closed
/// set so context/claims round-trip without a free-form JSON blob.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum ClaimValueDto {
    /// A textual claim.
    Text(String),
    /// A boolean claim.
    Bool(bool),
    /// An integer claim.
    Int(i64),
    /// A list of textual claims.
    TextList(Vec<String>),
}

impl From<ClaimValueDto> for ClaimValue {
    fn from(value: ClaimValueDto) -> Self {
        match value {
            ClaimValueDto::Text(text) => Self::Text(text),
            ClaimValueDto::Bool(flag) => Self::Bool(flag),
            ClaimValueDto::Int(int) => Self::Int(int),
            ClaimValueDto::TextList(items) => Self::TextList(items),
        }
    }
}

impl From<&ClaimValue> for ClaimValueDto {
    fn from(value: &ClaimValue) -> Self {
        match value {
            ClaimValue::Text(text) => Self::Text(text.clone()),
            ClaimValue::Bool(flag) => Self::Bool(*flag),
            ClaimValue::Int(int) => Self::Int(*int),
            ClaimValue::TextList(items) => Self::TextList(items.clone()),
        }
    }
}

/// A resource reference (type + id) on the wire, mirroring the domain
/// [`Resource`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDto {
    pub resource_type: String, // data_class: INTERNAL_ONLY
    pub resource_id: String,   // data_class: INTERNAL_ONLY
    /// Optional resource attributes visible to PDP resource conditions (for example
    /// `tenant_id` for same-tenant Cedar policies).
    #[serde(default)]
    pub attributes: BTreeMap<String, ClaimValueDto>, // data_class: INTERNAL_ONLY
}

impl ResourceDto {
    /// Convert into the domain resource.
    #[must_use]
    pub fn into_domain(self) -> Resource {
        self.attributes.into_iter().fold(
            Resource::new(self.resource_type, self.resource_id),
            |resource, (key, value)| resource.with_attribute(key, value.into()),
        )
    }
}

// =====================================================================
// Authorize (token-bearing) requests
// =====================================================================

/// `POST /authorize-with-token` — authorize a raw workload JWT against a PARC
/// action/resource/context. The token is validated and the persisted principal
/// resolved server-side; this is the full hot-path contract (PRD §3.3/§3.4).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeWithTokenRequest {
    pub token: String,         // data_class: AUTHENTICATION
    pub action: String,        // data_class: INTERNAL_ONLY
    pub resource: ResourceDto, // data_class: INTERNAL_ONLY
    /// Optional request context attributes (e.g. `mfa`, `source_ip`).
    #[serde(default)]
    pub context: BTreeMap<String, ClaimValueDto>, // data_class: INTERNAL_ONLY
}

impl AuthorizeWithTokenRequest {
    /// Project the action into the domain type.
    #[must_use]
    pub fn action(&self) -> Action {
        Action::new(self.action.clone())
    }

    /// Project the context map into domain claim values.
    #[must_use]
    pub fn context_domain(&self) -> BTreeMap<String, ClaimValue> {
        self.context
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().into()))
            .collect()
    }
}

/// `POST /authorize` — authorize an ALREADY-VERIFIED principal (the caller is a
/// trusted PEP that has authenticated the workload out of band) against a PARC
/// action/resource/context. Carries the principal identity fields explicitly.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeRequest {
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub workload_id: String,       // data_class: PII_IDENTIFYING
    pub owning_capability: String, // data_class: INTERNAL_ONLY
    #[serde(default)]
    pub scopes: Vec<String>, // data_class: INTERNAL_ONLY
    #[serde(default)]
    pub claims: BTreeMap<String, ClaimValueDto>, // data_class: INTERNAL_ONLY
    pub action: String,            // data_class: INTERNAL_ONLY
    pub resource: ResourceDto,     // data_class: INTERNAL_ONLY
    #[serde(default)]
    pub context: BTreeMap<String, ClaimValueDto>, // data_class: INTERNAL_ONLY
}

/// `POST /authorize:batch` — authorize many requests against one already-
/// verified principal-or-token set in a single round trip (PRD §1.2 batch).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchAuthorizeRequest {
    pub requests: Vec<AuthorizeWithTokenRequest>, // data_class: AUTHENTICATION
}

// =====================================================================
// Authorize responses
// =====================================================================

/// The binary effect on the wire.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EffectDto {
    Allow,
    Deny,
}

impl From<Effect> for EffectDto {
    fn from(value: Effect) -> Self {
        match value {
            Effect::Allow => Self::Allow,
            Effect::Deny => Self::Deny,
        }
    }
}

/// The decision reason on the wire, projected from the domain
/// [`DecisionReason`] so the audit chain + caller can distinguish an explicit
/// forbid from a deny-by-default from a non-operational principal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum DecisionReasonDto {
    /// An explicit `permit` policy matched.
    ExplicitPermit {
        #[serde(rename = "policyId")]
        policy_id: String, // data_class: INTERNAL_ONLY
    },
    /// An explicit `forbid` policy matched (forbid-wins).
    ExplicitForbid {
        #[serde(rename = "policyId")]
        policy_id: String, // data_class: INTERNAL_ONLY
    },
    /// No policy matched; deny-by-default.
    DefaultDeny,
    /// The principal was not in an operational lifecycle state.
    PrincipalNotOperational {
        state: String, // data_class: INTERNAL_ONLY
    },
}

impl From<&DecisionReason> for DecisionReasonDto {
    fn from(value: &DecisionReason) -> Self {
        match value {
            DecisionReason::ExplicitPermit { policy_id } => Self::ExplicitPermit {
                policy_id: policy_id.clone(),
            },
            DecisionReason::ExplicitForbid { policy_id } => Self::ExplicitForbid {
                policy_id: policy_id.clone(),
            },
            DecisionReason::DefaultDeny => Self::DefaultDeny,
            DecisionReason::PrincipalNotOperational { state } => Self::PrincipalNotOperational {
                state: state_label(*state).to_owned(),
            },
        }
    }
}

/// Lowercase lifecycle label mirroring the authz layer / `identity.cedar`.
#[must_use]
fn state_label(state: WorkloadState) -> &'static str {
    match state {
        WorkloadState::Provisioned => "provisioned",
        WorkloadState::Active => "active",
        WorkloadState::Suspended => "suspended",
        WorkloadState::Retired => "retired",
    }
}

/// `POST /authorize*` response: the decision effect + reason. A deny carries the
/// reason so the caller never has to guess why.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeResponse {
    pub effect: EffectDto,         // data_class: INTERNAL_ONLY
    pub reason: DecisionReasonDto, // data_class: INTERNAL_ONLY
}

impl AuthorizeResponse {
    /// Whether the decision allowed the request.
    #[must_use]
    pub fn is_allow(&self) -> bool {
        matches!(self.effect, EffectDto::Allow)
    }
}

impl From<&AuthorizationDecision> for AuthorizeResponse {
    fn from(decision: &AuthorizationDecision) -> Self {
        Self {
            effect: decision.effect().into(),
            reason: decision.reason().into(),
        }
    }
}

/// `POST /authorize:batch` response: one [`AuthorizeResponse`] per request, in
/// request order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchAuthorizeResponse {
    pub decisions: Vec<AuthorizeResponse>, // data_class: INTERNAL_ONLY
}

// =====================================================================
// Token validation
// =====================================================================

/// `POST /tokens/validate` — validate a raw workload JWT and return the
/// projected principal identity (no authorization is performed). A validation
/// failure is a 422 (PRD §3.4).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateTokenRequest {
    pub token: String, // data_class: AUTHENTICATION
}

/// The verified principal projection returned by `POST /tokens/validate`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateTokenResponse {
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub workload_id: String,       // data_class: PII_IDENTIFYING
    pub owning_capability: String, // data_class: INTERNAL_ONLY
    /// SPIFFE trust domain (`spiffe://<tenant>`), always equal to the tenant.
    pub trust_domain: String, // data_class: INTERNAL_ONLY
    pub state: String,             // data_class: INTERNAL_ONLY
    pub scopes: Vec<String>,       // data_class: INTERNAL_ONLY
}

// =====================================================================
// Lifecycle (control-plane) responses
// =====================================================================

/// Response for `POST /principals/{id}:suspend|retire`: the principal's id and
/// its new lifecycle state after the transition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalLifecycleResponse {
    pub workload_id: String, // data_class: PII_IDENTIFYING
    pub state: String,       // data_class: INTERNAL_ONLY
}

impl PrincipalLifecycleResponse {
    /// Build from a workload id + domain lifecycle state.
    #[must_use]
    pub fn new(workload_id: impl Into<String>, state: WorkloadState) -> Self {
        Self {
            workload_id: workload_id.into(),
            state: state_label(state).to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iam_identity_workload_domain::AuthorizationDecision;

    #[test]
    fn claim_value_dto_round_trips_through_domain() {
        for dto in [
            ClaimValueDto::Text("prod".into()),
            ClaimValueDto::Bool(true),
            ClaimValueDto::Int(7),
            ClaimValueDto::TextList(vec!["a".into(), "b".into()]),
        ] {
            let domain: ClaimValue = dto.clone().into();
            let back: ClaimValueDto = (&domain).into();
            assert_eq!(dto, back);
        }
    }

    #[test]
    fn authorize_response_projects_permit_reason() {
        let decision = AuthorizationDecision::permit("allow-acme");
        let response = AuthorizeResponse::from(&decision);
        assert!(response.is_allow());
        assert_eq!(response.effect, EffectDto::Allow);
        assert_eq!(
            response.reason,
            DecisionReasonDto::ExplicitPermit {
                policy_id: "allow-acme".into()
            }
        );
    }

    #[test]
    fn authorize_response_projects_forbid_and_default_deny() {
        let forbid = AuthorizeResponse::from(&AuthorizationDecision::forbid("freeze"));
        assert!(!forbid.is_allow());
        assert_eq!(forbid.effect, EffectDto::Deny);
        assert_eq!(
            forbid.reason,
            DecisionReasonDto::ExplicitForbid {
                policy_id: "freeze".into()
            }
        );

        let default_deny = AuthorizeResponse::from(&AuthorizationDecision::default_deny());
        assert_eq!(default_deny.reason, DecisionReasonDto::DefaultDeny);
    }

    #[test]
    fn lifecycle_response_uses_lowercase_state() {
        let response = PrincipalLifecycleResponse::new("wl_a", WorkloadState::Suspended);
        let body = serde_json::to_value(&response).expect("serialize");
        assert_eq!(body["workloadId"], "wl_a");
        assert_eq!(body["state"], "suspended");
    }
}
