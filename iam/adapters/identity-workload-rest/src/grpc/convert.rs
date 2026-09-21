//! Proto <-> domain conversions and the shared per-RPC helpers.

use std::collections::BTreeMap;

use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};
use tonic::Status;

use iam_identity_workload_app::{
    AuthorizeOutcome, RevocationDenylist, WorkloadPrincipalRepository, authorize_with_token,
};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_domain::{Action, ClaimValue, Resource};
use iam_identity_workload_oidc::{OidcValidationError, validate_workload_token};

use crate::{
    AuditEvent, AuditRecord, AuditSink, DecisionAuthzRequest, VerifiedCaller, WorkloadAuthzState,
};

use super::proto::{
    self, AuthorizeResponse as ProtoAuthorizeResponse,
    AuthorizeWithTokenRequest as ProtoAuthorizeWithTokenRequest, DecisionEffect,
};

/// Convert an [`AuthorizeOutcome`] into a proto [`AuthorizeResponse`].
/// A deny is ALWAYS a response value (DECISION_EFFECT_DENY), never a tonic error.
pub(super) fn outcome_to_proto_response(outcome: &AuthorizeOutcome) -> ProtoAuthorizeResponse {
    let effect = if outcome.is_allow() {
        DecisionEffect::Allow as i32
    } else {
        DecisionEffect::Deny as i32
    };
    ProtoAuthorizeResponse {
        effect,
        reason: None,
    }
}

/// Machine outcome label for an authorize audit record, matching REST labels.
pub(super) fn authorize_outcome_label(outcome: &AuthorizeOutcome) -> &'static str {
    match outcome {
        AuthorizeOutcome::Decided(d) if d.is_allow() => "allow",
        AuthorizeOutcome::Decided(_) => "deny",
        AuthorizeOutcome::TokenRejected => "token-rejected",
        AuthorizeOutcome::PrincipalUnknown => "deny",
        AuthorizeOutcome::Revoked => "deny",
        AuthorizeOutcome::StoreUnavailable => "store-unavailable",
    }
}

/// Copy the `authorization` request metadatum into an http [`HeaderMap`] so the
/// shared header-based [`crate::CallerVerifier`] authenticates a gRPC caller
/// EXACTLY as it does a REST caller — one authn seam, both surfaces, no second
/// credential format to drift (AUTH-005). Only `authorization` is copied; no
/// caller-supplied identity metadatum can authorize.
pub(super) fn headers_from_metadata(metadata: &tonic::metadata::MetadataMap) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Some(value) = metadata.get("authorization").and_then(|v| v.to_str().ok())
        && let Ok(header_value) = HeaderValue::from_str(value)
    {
        headers.insert(AUTHORIZATION, header_value);
    }
    headers
}

/// Authenticate the gRPC caller from request metadata. `Err(unauthenticated)`
/// when no verified credential is present (default-deny; mirrors the REST `401`).
pub(super) fn verify_grpc_caller<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    metadata: &tonic::metadata::MetadataMap,
) -> Result<VerifiedCaller, Status>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let headers = headers_from_metadata(metadata);
    state
        .caller_verifier_ref()
        .verify_principal(&headers)
        .ok_or_else(|| Status::unauthenticated("verified caller credential required"))
}

/// Per-decision same-tenant caller-authz gate for the gRPC decision RPCs. `Ok(())`
/// permits; a policy deny (`Ok(false)`) or a PDP fault (`Err`) BOTH map to
/// `permission_denied` (fail-closed, never `internal`/5xx) and emit one deny audit
/// record with caller attribution. Mirrors the REST `decision_gate`.
#[allow(clippy::too_many_arguments)]
pub(super) fn decide_grpc<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    caller: &VerifiedCaller,
    subject_tenant: &str,
    subject_workload_id: &str,
    action: &str,
    resource_type: &str,
    resource_id: &str,
) -> Result<(), Status>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let request = DecisionAuthzRequest {
        caller_tenant: caller.caller_tenant(),
        caller_id: caller.caller_id(),
        subject_tenant,
        subject_workload_id,
        action,
        resource_type,
        resource_id,
    };
    let detail = match state.decision_authorizer_ref().decide(&request) {
        Ok(true) => return Ok(()),
        Ok(false) => "decision-forbidden",
        Err(_fault) => "decision-pdp-fault",
    };
    state.audit().record(
        AuditRecord::new(
            AuditEvent::Authorize,
            Some(subject_workload_id.to_owned()),
            "deny",
            Some(detail.to_owned()),
        )
        .with_authorization_target(action, resource_type, resource_id)
        .with_caller(caller.caller_id(), caller.caller_tenant()),
    );
    Err(Status::permission_denied("cross-tenant decision denied"))
}

/// Best-effort `(subject_tenant, subject_workload_id)` from a token, for the gRPC
/// same-tenant gate. `None` for a token that does not validate (forged/expired):
/// the gate is then skipped and the existing flow fail-closes the request.
pub(super) fn best_effort_subject<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    token: &str,
) -> Option<(String, String)>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let now = (state.now_provider_ref())();
    validate_workload_token(token, state.jwks_ref(), state.config_ref(), now)
        .ok()
        .map(|p| {
            (
                p.tenant_id().as_str().to_owned(),
                p.workload_id().as_str().to_owned(),
            )
        })
}

/// Run authorize_with_token using the (mutex-guarded) state. Always returns
/// `Ok(outcome)` (including `StoreUnavailable` and DENY). The caller maps
/// `StoreUnavailable` to `Status::unavailable` for unary RPCs and to a per-item
/// DENY decision for batch. The `Result` return is retained so callers thread
/// the outcome with `?` and a future fallible path can surface a `Status`.
pub(super) fn run_authorize_with_token_grpc<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    token: &str,
    action: Action,
    resource: Resource,
    context: BTreeMap<String, ClaimValue>,
) -> Result<AuthorizeOutcome, Status>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let now = (state.now_provider_ref())();
    let repo_guard = match state.repository_lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    let denylist_guard = match state.denylist_lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    Ok(authorize_with_token(
        &*repo_guard,
        &*denylist_guard,
        state.authorizer_ref(),
        state.jwks_ref(),
        state.config_ref(),
        now,
        token,
        action,
        resource,
        context,
    ))
}

/// Extract action/resource/context from a proto `AuthorizeWithTokenRequest`.
pub(super) fn decode_authorize_with_token_request(
    req: &ProtoAuthorizeWithTokenRequest,
) -> (Action, Resource, BTreeMap<String, ClaimValue>) {
    let action = Action::new(req.action.clone());
    let resource = proto_resource_to_domain(req.resource.as_ref());
    let context = proto_context_to_domain(&req.context);
    (action, resource, context)
}

/// Convert a proto resource reference plus attributes to the domain resource.
pub(super) fn proto_resource_to_domain(resource: Option<&proto::Resource>) -> Resource {
    let Some(resource) = resource else {
        return Resource::new(String::new(), String::new());
    };
    proto_context_to_domain(&resource.attributes)
        .into_iter()
        .fold(
            Resource::new(resource.resource_type.clone(), resource.resource_id.clone()),
            |domain, (key, value)| domain.with_attribute(key, value),
        )
}

/// Convert proto `map<string, ClaimValue>` to domain `BTreeMap<String, ClaimValue>`.
pub(super) fn proto_context_to_domain(
    map: &std::collections::HashMap<String, proto::ClaimValue>,
) -> BTreeMap<String, ClaimValue> {
    map.iter()
        .filter_map(|(k, v)| {
            let domain_val = proto_claim_value_to_domain(v)?;
            Some((k.clone(), domain_val))
        })
        .collect()
}

pub(super) fn proto_claim_value_to_domain(cv: &proto::ClaimValue) -> Option<ClaimValue> {
    use proto::claim_value::Value;
    match cv.value.as_ref()? {
        Value::Text(s) => Some(ClaimValue::Text(s.clone())),
        Value::Boolean(b) => Some(ClaimValue::Bool(*b)),
        Value::Integer(i) => Some(ClaimValue::Int(*i)),
        Value::TextList(list) => Some(ClaimValue::TextList(list.values.clone())),
    }
}

/// Map an [`OidcValidationError`] to the typed proto [`ValidationErrorKind`] a
/// mesh PEP / Envoy ext_authz consumer branches on. Mirrors the mapping table in
/// `docs/specs/slice-id-workload-grpc-surface.md`. The human-readable message is
/// carried separately in `ValidationError::detail`.
pub(super) fn oidc_error_to_kind(error: &OidcValidationError) -> proto::ValidationErrorKind {
    use proto::ValidationErrorKind as Kind;
    match error {
        OidcValidationError::MalformedToken
        | OidcValidationError::DecodeError
        | OidcValidationError::MalformedKey => Kind::Malformed,
        OidcValidationError::AlgNone => Kind::AlgNone,
        OidcValidationError::InvalidType => Kind::InvalidType,
        OidcValidationError::UntrustedKeySourceUrl => Kind::UntrustedKeySourceUrl,
        OidcValidationError::AlgorithmMismatch | OidcValidationError::UnsupportedAlgorithm => {
            Kind::AlgorithmMismatch
        }
        OidcValidationError::UnknownKey => Kind::UnknownKey,
        OidcValidationError::SignatureInvalid => Kind::SignatureInvalid,
        OidcValidationError::IssuerMismatch => Kind::IssuerMismatch,
        OidcValidationError::AudienceMismatch => Kind::AudienceMismatch,
        OidcValidationError::Expired => Kind::Expired,
        OidcValidationError::NotYetValid => Kind::NotYetValid,
        OidcValidationError::MissingClaim(_) | OidcValidationError::Domain(_) => Kind::MissingClaim,
    }
}

/// Best-effort extraction of workload_id from a token for audit records.
/// Returns `None` for tokens that do not validate (forged/expired).
pub(super) fn best_effort_workload_id<R, D, A, S>(
    state: &WorkloadAuthzState<R, D, A, S>,
    token: &str,
) -> Option<String>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    let now = (state.now_provider_ref())();
    validate_workload_token(token, state.jwks_ref(), state.config_ref(), now)
        .ok()
        .map(|p| p.workload_id().as_str().to_owned())
}

pub(super) fn workload_state_to_proto(
    state: iam_identity_workload_domain::WorkloadState,
) -> proto::WorkloadState {
    use iam_identity_workload_domain::WorkloadState as WS;
    match state {
        WS::Provisioned => proto::WorkloadState::Provisioned,
        WS::Active => proto::WorkloadState::Active,
        WS::Suspended => proto::WorkloadState::Suspended,
        WS::Retired => proto::WorkloadState::Retired,
    }
}
