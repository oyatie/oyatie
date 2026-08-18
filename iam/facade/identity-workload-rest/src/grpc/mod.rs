//! gRPC delivery surface for workload-identity.
//!
//! Implements both `WorkloadAuthorizer` and `WorkloadTokenValidator` tonic
//! service traits, delegating to the SAME `oya-identity-workload-app`
//! use-cases and OIDC adapter primitives that the REST surface uses — no
//! duplicated decision logic.
//!
//! ## Shared-core design
//!
//! - `AuthorizeWithToken` / `AuthorizeBatch` -> `authorize_with_token` app use-case.
//! - `Authorize` -> `build_active_principal` (crate fn in lib.rs) +
//!   `authorizer_ref().authorize`.
//! - `ValidateToken` -> `validate_workload_token` (OIDC adapter).
//!
//! ## Fail-closed contract
//!
//! - Authorization deny -> `AuthorizeResponse { effect: DECISION_EFFECT_DENY }` — never a tonic error.
//! - Token-validation failure -> `ValidateTokenResponse { ok: false, outcome: Error(...) }` — engine NOT consulted.
//! - Store unavailable -> `tonic::Status::unavailable` for unary RPCs;
//!   per-item DENY decision value in batch.
//! - One immutable `AuditRecord` emitted per authorize and per token-validation.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};
use tonic::{Request, Response, Status};

use iam_identity_workload_api::ClaimValueDto;
use iam_identity_workload_app::{
    AuthorizeOutcome, RevocationDenylist, WorkloadPrincipalRepository, authorize_with_token,
};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_domain::{Action, AuthorizationRequest, ClaimValue, Resource};
use iam_identity_workload_oidc::{OidcValidationError, validate_workload_token};

use crate::{
    AuditEvent, AuditRecord, AuditSink, DecisionAuthzRequest, VerifiedCaller, WorkloadAuthzState,
    build_active_principal,
};

// Include tonic-generated stubs for oya.identity.workload.v1.
pub mod proto {
    tonic::include_proto!("oya.identity.workload.v1");
}

pub use proto::workload_authorizer_server::WorkloadAuthorizerServer;
pub use proto::workload_token_validator_server::WorkloadTokenValidatorServer;

use proto::{
    AuthorizeRequest as ProtoAuthorizeRequest, AuthorizeResponse as ProtoAuthorizeResponse,
    AuthorizeWithTokenRequest as ProtoAuthorizeWithTokenRequest,
    BatchAuthorizeRequest as ProtoBatchAuthorizeRequest,
    BatchAuthorizeResponse as ProtoBatchAuthorizeResponse, DecisionEffect,
    ValidateTokenRequest as ProtoValidateTokenRequest,
    ValidateTokenResponse as ProtoValidateTokenResponse, VerifiedPrincipal,
    workload_authorizer_server::WorkloadAuthorizer as WorkloadAuthorizerTrait,
    workload_token_validator_server::WorkloadTokenValidator as WorkloadTokenValidatorTrait,
};

// =====================================================================
// Server handle
// =====================================================================

/// gRPC server handle wrapping the shared application state.
///
/// Implements both `WorkloadAuthorizer` and `WorkloadTokenValidator` tonic
/// server traits, delegating to the same use-case core as the REST surface.
pub struct WorkloadGrpcServer<R, D, A, S>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    state: Arc<WorkloadAuthzState<R, D, A, S>>,
}

impl<R, D, A, S> WorkloadGrpcServer<R, D, A, S>
where
    R: WorkloadPrincipalRepository + Send + 'static,
    D: RevocationDenylist + Send + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
{
    /// Build a gRPC server from the shared state Arc.
    #[must_use]
    pub fn new(state: Arc<WorkloadAuthzState<R, D, A, S>>) -> Self {
        Self { state }
    }
}

// =====================================================================
// Helpers
// =====================================================================

/// Convert an [`AuthorizeOutcome`] into a proto [`AuthorizeResponse`].
/// A deny is ALWAYS a response value (DECISION_EFFECT_DENY), never a tonic error.
fn outcome_to_proto_response(outcome: &AuthorizeOutcome) -> ProtoAuthorizeResponse {
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
fn authorize_outcome_label(outcome: &AuthorizeOutcome) -> &'static str {
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
fn headers_from_metadata(metadata: &tonic::metadata::MetadataMap) -> HeaderMap {
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
fn verify_grpc_caller<R, D, A, S>(
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
fn decide_grpc<R, D, A, S>(
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
fn best_effort_subject<R, D, A, S>(
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
fn run_authorize_with_token_grpc<R, D, A, S>(
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
fn decode_authorize_with_token_request(
    req: &ProtoAuthorizeWithTokenRequest,
) -> (Action, Resource, BTreeMap<String, ClaimValue>) {
    let action = Action::new(req.action.clone());
    let resource = proto_resource_to_domain(req.resource.as_ref());
    let context = proto_context_to_domain(&req.context);
    (action, resource, context)
}

/// Convert a proto resource reference plus attributes to the domain resource.
fn proto_resource_to_domain(resource: Option<&proto::Resource>) -> Resource {
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
fn proto_context_to_domain(
    map: &std::collections::HashMap<String, proto::ClaimValue>,
) -> BTreeMap<String, ClaimValue> {
    map.iter()
        .filter_map(|(k, v)| {
            let domain_val = proto_claim_value_to_domain(v)?;
            Some((k.clone(), domain_val))
        })
        .collect()
}

fn proto_claim_value_to_domain(cv: &proto::ClaimValue) -> Option<ClaimValue> {
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
fn oidc_error_to_kind(error: &OidcValidationError) -> proto::ValidationErrorKind {
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

// =====================================================================
// WorkloadAuthorizer tonic impl
// =====================================================================

#[tonic::async_trait]
impl<R, D, A, S> WorkloadAuthorizerTrait for WorkloadGrpcServer<R, D, A, S>
where
    R: WorkloadPrincipalRepository + Send + Sync + 'static,
    D: RevocationDenylist + Send + Sync + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + Send + Sync + 'static,
{
    async fn authorize_with_token(
        &self,
        request: Request<ProtoAuthorizeWithTokenRequest>,
    ) -> Result<Response<ProtoAuthorizeResponse>, Status> {
        let caller = verify_grpc_caller(&self.state, request.metadata())?;
        let req = request.into_inner();
        // Same-tenant gate: subject tenant from the VALIDATED token. A token that
        // does not validate is left to the existing token-rejected DENY path.
        if let Some((subject_tenant, subject_workload_id)) =
            best_effort_subject(&self.state, &req.token)
        {
            let resource = req.resource.as_ref();
            decide_grpc(
                &self.state,
                &caller,
                &subject_tenant,
                &subject_workload_id,
                &req.action,
                resource
                    .map(|r| r.resource_type.as_str())
                    .unwrap_or_default(),
                resource.map(|r| r.resource_id.as_str()).unwrap_or_default(),
            )?;
        }
        let (action, resource, context) = decode_authorize_with_token_request(&req);
        let outcome =
            run_authorize_with_token_grpc(&self.state, &req.token, action, resource, context)?;

        // Determine workload_id for audit (best-effort from token).
        let workload_id = best_effort_workload_id(&self.state, &req.token);
        let outcome_label = authorize_outcome_label(&outcome);

        // Emit exactly one audit record.
        self.state.audit().record(AuditRecord::new(
            AuditEvent::Authorize,
            workload_id,
            outcome_label,
            None,
        ));

        // Store outage -> tonic Unavailable (fail-closed).
        if matches!(outcome, AuthorizeOutcome::StoreUnavailable) {
            return Err(Status::unavailable("store unavailable"));
        }

        Ok(Response::new(outcome_to_proto_response(&outcome)))
    }

    async fn authorize(
        &self,
        request: Request<ProtoAuthorizeRequest>,
    ) -> Result<Response<ProtoAuthorizeResponse>, Status> {
        // AUTH-005: authenticate the caller from metadata, then a fail-closed
        // same-tenant gate (subject tenant = the body's tenant_id) BEFORE the
        // caller-asserted principal is built — a forged body can no longer obtain
        // an arbitrary cross-tenant decision over the unauthenticated socket.
        let caller = verify_grpc_caller(&self.state, request.metadata())?;
        let req = request.into_inner();
        let resource = req.resource.as_ref();
        decide_grpc(
            &self.state,
            &caller,
            &req.tenant_id,
            &req.workload_id,
            &req.action,
            resource
                .map(|r| r.resource_type.as_str())
                .unwrap_or_default(),
            resource.map(|r| r.resource_id.as_str()).unwrap_or_default(),
        )?;

        // Reuse the crate-level `build_active_principal` — same logic as REST /authorize.
        // We need to build an api AuthorizeRequest to reuse the helper.
        let api_req = iam_identity_workload_api::AuthorizeRequest {
            tenant_id: req.tenant_id.clone(),
            workload_id: req.workload_id.clone(),
            owning_capability: req.owning_capability.clone(),
            scopes: req.scopes.clone(),
            claims: proto_context_to_domain(&req.claims)
                .into_iter()
                .map(|(k, v)| (k, ClaimValueDto::from(&v)))
                .collect(),
            context: Default::default(),
            action: req.action.clone(),
            resource: {
                let r = req.resource.as_ref();
                iam_identity_workload_api::ResourceDto {
                    resource_type: r.map(|x| x.resource_type.clone()).unwrap_or_default(),
                    resource_id: r.map(|x| x.resource_id.clone()).unwrap_or_default(),
                    attributes: r
                        .map(|x| {
                            proto_context_to_domain(&x.attributes)
                                .into_iter()
                                .map(|(k, v)| (k, ClaimValueDto::from(&v)))
                                .collect()
                        })
                        .unwrap_or_default(),
                }
            },
        };

        let principal = match build_active_principal(&api_req) {
            Ok(p) => p,
            Err(_) => {
                // A malformed principal -> default deny (fail-closed).
                self.state.audit().record(AuditRecord::new(
                    AuditEvent::Authorize,
                    Some(req.workload_id.clone()),
                    "deny",
                    Some("invalid-principal".to_owned()),
                ));
                return Ok(Response::new(ProtoAuthorizeResponse {
                    effect: DecisionEffect::Deny as i32,
                    reason: None,
                }));
            }
        };

        let workload_id = principal.workload_id().as_str().to_owned();
        let mut authz_request = AuthorizationRequest::new(
            principal,
            Action::new(req.action.clone()),
            proto_resource_to_domain(req.resource.as_ref()),
        );
        authz_request.context = proto_context_to_domain(&req.context);

        let decision = self.state.authorizer_ref().authorize(&authz_request);
        let outcome = AuthorizeOutcome::Decided(decision);
        let outcome_label = authorize_outcome_label(&outcome);

        self.state.audit().record(AuditRecord::new(
            AuditEvent::Authorize,
            Some(workload_id),
            outcome_label,
            None,
        ));

        Ok(Response::new(outcome_to_proto_response(&outcome)))
    }

    async fn authorize_batch(
        &self,
        request: Request<ProtoBatchAuthorizeRequest>,
    ) -> Result<Response<ProtoBatchAuthorizeResponse>, Status> {
        let caller = verify_grpc_caller(&self.state, request.metadata())?;
        let req = request.into_inner();
        let mut decisions = Vec::with_capacity(req.requests.len());

        for item in &req.requests {
            // Per-item same-tenant gate: a cross-tenant (or PDP-faulted) item
            // collapses to a DENY decision in the batch (fail-closed), never an
            // Err and never a leaked allow. The gate emits the item's deny record.
            if let Some((subject_tenant, subject_workload_id)) =
                best_effort_subject(&self.state, &item.token)
            {
                let resource = item.resource.as_ref();
                if decide_grpc(
                    &self.state,
                    &caller,
                    &subject_tenant,
                    &subject_workload_id,
                    &item.action,
                    resource
                        .map(|r| r.resource_type.as_str())
                        .unwrap_or_default(),
                    resource.map(|r| r.resource_id.as_str()).unwrap_or_default(),
                )
                .is_err()
                {
                    decisions.push(ProtoAuthorizeResponse {
                        effect: DecisionEffect::Deny as i32,
                        reason: None,
                    });
                    continue;
                }
            }
            let (action, resource, context) = decode_authorize_with_token_request(item);
            let outcome =
                run_authorize_with_token_grpc(&self.state, &item.token, action, resource, context)?;

            let workload_id = best_effort_workload_id(&self.state, &item.token);
            let outcome_label = authorize_outcome_label(&outcome);

            self.state.audit().record(AuditRecord::new(
                AuditEvent::Authorize,
                workload_id,
                outcome_label,
                None,
            ));

            // Store outage on any item -> fail-closed DENY decision for that item (batch never Err).
            decisions.push(outcome_to_proto_response(&outcome));
        }

        Ok(Response::new(ProtoBatchAuthorizeResponse { decisions }))
    }
}

// =====================================================================
// WorkloadTokenValidator tonic impl
// =====================================================================

#[tonic::async_trait]
impl<R, D, A, S> WorkloadTokenValidatorTrait for WorkloadGrpcServer<R, D, A, S>
where
    R: WorkloadPrincipalRepository + Send + Sync + 'static,
    D: RevocationDenylist + Send + Sync + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + Send + Sync + 'static,
{
    async fn validate_token(
        &self,
        request: Request<ProtoValidateTokenRequest>,
    ) -> Result<Response<ProtoValidateTokenResponse>, Status> {
        let caller = verify_grpc_caller(&self.state, request.metadata())?;
        let req = request.into_inner();
        let now = (self.state.now_provider_ref())();

        match validate_workload_token(
            &req.token,
            self.state.jwks_ref(),
            self.state.config_ref(),
            now,
        ) {
            Ok(principal) => {
                // Same-tenant gate: a caller may only introspect a token within its
                // own tenant (cross-tenant introspection -> permission_denied).
                decide_grpc(
                    &self.state,
                    &caller,
                    principal.tenant_id().as_str(),
                    principal.workload_id().as_str(),
                    "identity.workload.ValidateToken",
                    "Workload",
                    principal.workload_id().as_str(),
                )?;
                self.state.audit().record(AuditRecord::new(
                    AuditEvent::TokenValidation,
                    Some(principal.workload_id().as_str().to_owned()),
                    "validated",
                    None,
                ));
                let resp = ProtoValidateTokenResponse {
                    ok: true,
                    outcome: Some(proto::validate_token_response::Outcome::Principal(
                        VerifiedPrincipal {
                            tenant_id: principal.tenant_id().as_str().to_owned(),
                            workload_id: principal.workload_id().as_str().to_owned(),
                            owning_capability: principal.owning_capability().as_str().to_owned(),
                            trust_domain: principal.trust_domain().as_str().to_owned(),
                            state: workload_state_to_proto(principal.state()) as i32,
                            scopes: principal.scopes().to_vec(),
                        },
                    )),
                };
                Ok(Response::new(resp))
            }
            Err(error) => {
                self.state.audit().record(AuditRecord::new(
                    AuditEvent::TokenValidation,
                    None,
                    "validation-failed",
                    Some(error.to_string()),
                ));
                let resp = ProtoValidateTokenResponse {
                    ok: false,
                    outcome: Some(proto::validate_token_response::Outcome::Error(
                        proto::ValidationError {
                            kind: oidc_error_to_kind(&error) as i32,
                            detail: error.to_string(),
                        },
                    )),
                };
                Ok(Response::new(resp))
            }
        }
    }
}

// =====================================================================
// Private helpers
// =====================================================================

/// Best-effort extraction of workload_id from a token for audit records.
/// Returns `None` for tokens that do not validate (forged/expired).
fn best_effort_workload_id<R, D, A, S>(
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

fn workload_state_to_proto(
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
