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

use tonic::{Request, Response, Status};

use oya_identity_workload_app::{AuthorizeOutcome, RevocationDenylist, WorkloadPrincipalRepository, authorize_with_token};
use oya_identity_workload_authz_cedar_adapter::WorkloadAuthorizer;
use oya_identity_workload_domain::{Action, AuthorizationDecision, AuthorizationRequest, ClaimValue, Resource};
use oya_identity_workload_oidc_adapter::{OidcValidationError, validate_workload_token};

use crate::{AuditEvent, AuditRecord, AuditSink, WorkloadAuthzState, build_active_principal};

// Include tonic-generated stubs for oya.identity.workload.v1.
pub mod proto {
    tonic::include_proto!("oya.identity.workload.v1");
}

pub use proto::workload_authorizer_server::WorkloadAuthorizerServer;
pub use proto::workload_token_validator_server::WorkloadTokenValidatorServer;

use proto::{
    AuthorizeRequest as ProtoAuthorizeRequest,
    AuthorizeResponse as ProtoAuthorizeResponse,
    AuthorizeWithTokenRequest as ProtoAuthorizeWithTokenRequest,
    BatchAuthorizeRequest as ProtoBatchAuthorizeRequest,
    BatchAuthorizeResponse as ProtoBatchAuthorizeResponse,
    DecisionEffect,
    ValidateTokenRequest as ProtoValidateTokenRequest,
    ValidateTokenResponse as ProtoValidateTokenResponse,
    VerifiedPrincipal,
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
    let resource = req
        .resource
        .as_ref()
        .map(|r| Resource::new(r.resource_type.clone(), r.resource_id.clone()))
        .unwrap_or_else(|| Resource::new(String::new(), String::new()));
    let context = proto_context_to_domain(&req.context);
    (action, resource, context)
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
        OidcValidationError::AlgorithmMismatch
        | OidcValidationError::UnsupportedAlgorithm => Kind::AlgorithmMismatch,
        OidcValidationError::UnknownKey => Kind::UnknownKey,
        OidcValidationError::SignatureInvalid => Kind::SignatureInvalid,
        OidcValidationError::IssuerMismatch => Kind::IssuerMismatch,
        OidcValidationError::AudienceMismatch => Kind::AudienceMismatch,
        OidcValidationError::Expired => Kind::Expired,
        OidcValidationError::NotYetValid => Kind::NotYetValid,
        OidcValidationError::MissingClaim(_) | OidcValidationError::Domain(_) => {
            Kind::MissingClaim
        }
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
        let req = request.into_inner();
        let (action, resource, context) = decode_authorize_with_token_request(&req);
        let outcome = run_authorize_with_token_grpc(
            &self.state,
            &req.token,
            action,
            resource,
            context,
        )?;

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
        let req = request.into_inner();

        // Reuse the crate-level `build_active_principal` — same logic as REST /authorize.
        // We need to build an api AuthorizeRequest to reuse the helper.
        let api_req = oya_identity_workload_api::AuthorizeRequest {
            tenant_id: req.tenant_id.clone(),
            workload_id: req.workload_id.clone(),
            owning_capability: req.owning_capability.clone(),
            scopes: req.scopes.clone(),
            claims: Default::default(),
            context: Default::default(),
            action: req.action.clone(),
            resource: {
                let r = req.resource.as_ref();
                oya_identity_workload_api::ResourceDto {
                    resource_type: r.map(|x| x.resource_type.clone()).unwrap_or_default(),
                    resource_id: r.map(|x| x.resource_id.clone()).unwrap_or_default(),
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
            req.resource
                .as_ref()
                .map(|r| Resource::new(r.resource_type.clone(), r.resource_id.clone()))
                .unwrap_or_else(|| Resource::new(String::new(), String::new())),
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
        let req = request.into_inner();
        let mut decisions = Vec::with_capacity(req.requests.len());

        for item in &req.requests {
            let (action, resource, context) = decode_authorize_with_token_request(item);
            let outcome = run_authorize_with_token_grpc(
                &self.state,
                &item.token,
                action,
                resource,
                context,
            )?;

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
        let req = request.into_inner();
        let now = (self.state.now_provider_ref())();

        match validate_workload_token(&req.token, self.state.jwks_ref(), self.state.config_ref(), now) {
            Ok(principal) => {
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
    state: oya_identity_workload_domain::WorkloadState,
) -> proto::WorkloadState {
    use oya_identity_workload_domain::WorkloadState as WS;
    match state {
        WS::Provisioned => proto::WorkloadState::Provisioned,
        WS::Active => proto::WorkloadState::Active,
        WS::Suspended => proto::WorkloadState::Suspended,
        WS::Retired => proto::WorkloadState::Retired,
    }
}
